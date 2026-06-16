use crate::{TopLevel, Expr, Stmt, BridgeParam};
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[TopLevel]) -> String {
    let mut ir_globals = String::new();
    let mut blocks: Vec<(String, String)> = vec![("entry".to_string(), String::new())];
    let mut current_block = "entry".to_string();

    let mut reg_counter = 0;
    let mut block_counter = 0;
    let mut vars: HashMap<String, String> = HashMap::new();

    ir_globals.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir_globals.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir_globals.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n\n");

    // 1. 递归收集所有字符串
    let mut string_globals: HashMap<String, String> = HashMap::new();
    let mut str_counter = 0;
    for top in ast {
        match top {
            TopLevel::Stmt(stmt) => collect_stmt_strings(stmt, &mut string_globals, &mut str_counter),
            TopLevel::LetDecl { value, .. } => collect_expr_strings(value, &mut string_globals, &mut str_counter),
            _ => {}
        }
    }
    for (content, name) in &string_globals {
        let len = content.len() + 1;
        ir_globals.push_str(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", name, len, content));
    }

    // 2. 生成 Bridge 声明
    for top in ast {
        if let TopLevel::BridgeDecl { lang: l, name, params, ret_ty, .. } = top {
            if l == "c" {
                let pt: Vec<String> = params.iter().map(|p| {
                    let BridgeParam::Param { ty, .. } = p;
                    if ty == "Int" { "i32" } else { "double" }.to_string()
                }).collect();
                let rt = if ret_ty == "Int" { "i32" } else { "double" };
                ir_globals.push_str(&format!("declare {} @{}({})\n", rt, name, pt.join(", ")));
            }
        }
    }

    // 3. 预分配变量 (alloca)
    for top in ast {
        if let TopLevel::LetDecl { name, .. } = top {
            let ptr = format!("%{}", reg_counter); reg_counter += 1;
            vars.insert(name.clone(), ptr.clone());
            emit(&mut blocks, &current_block, &format!("  {} = alloca i64\n", ptr));
        }
    }

    // 4. 生成执行代码
    for top in ast {
        match top {
            TopLevel::LetDecl { name, value } => {
                let (val_reg, _) = gen_expr(value, &mut blocks, &mut current_block, &mut reg_counter, &vars, &string_globals, ast);
                let ptr = vars.get(name).unwrap();
                emit(&mut blocks, &current_block, &format!("  store i64 {}, i64* {}\n", val_reg, ptr));
            }
            TopLevel::Stmt(stmt) => {
                gen_stmt(stmt, &mut blocks, &mut current_block, &mut reg_counter, &mut block_counter, &vars, &string_globals, ast);
            }
            _ => {}
        }
    }

    emit(&mut blocks, &current_block, "  ret i32 0\n");

    let mut final_ir = ir_globals;
    final_ir.push_str("define i32 @main() {\n");
    for (name, code) in &blocks {
        final_ir.push_str(&format!("{}:\n", name));
        final_ir.push_str(code);
    }
    final_ir.push_str("}\n");

    final_ir
}

fn emit(blocks: &mut Vec<(String, String)>, block_name: &str, instruction: &str) {
    if let Some((_, code)) = blocks.iter_mut().find(|(name, _)| name == block_name) {
        code.push_str(instruction);
    }
}

fn new_label(counter: &mut i32) -> String {
    let name = format!("block_{}", counter);
    *counter += 1;
    name
}

fn gen_stmt(stmt: &Stmt, blocks: &mut Vec<(String, String)>, current_block: &mut String, reg: &mut i32, block_cnt: &mut i32, vars: &HashMap<String, String>, strings: &HashMap<String, String>, ast: &[TopLevel]) {
    match stmt {
        Stmt::Expr(e) => {
            gen_expr(e, blocks, current_block, reg, vars, strings, ast);
        }
        Stmt::If(cond, then_stmts, else_stmts) => {
            let (cond_reg, _cond_ty) = gen_expr(cond, blocks, current_block, reg, vars, strings, ast);
            
            let then_lbl = new_label(block_cnt);
            let else_lbl = new_label(block_cnt);
            let merge_lbl = new_label(block_cnt);
            
            blocks.push((then_lbl.clone(), String::new()));
            blocks.push((else_lbl.clone(), String::new()));
            blocks.push((merge_lbl.clone(), String::new()));

            emit(blocks, current_block, &format!("  br i1 {}, label %{}, label %{}\n", cond_reg, then_lbl, else_lbl));

            *current_block = then_lbl.clone();
            for s in then_stmts { gen_stmt(s, blocks, current_block, reg, block_cnt, vars, strings, ast); }
            emit(blocks, current_block, &format!("  br label %{}\n", merge_lbl));

            *current_block = else_lbl.clone();
            for s in else_stmts { gen_stmt(s, blocks, current_block, reg, block_cnt, vars, strings, ast); }
            emit(blocks, current_block, &format!("  br label %{}\n", merge_lbl));

            *current_block = merge_lbl;
        }
        // 【新增】While 循环代码生成
        Stmt::While(cond, body_stmts) => {
            let cond_lbl = new_label(block_cnt);
            let body_lbl = new_label(block_cnt);
            let merge_lbl = new_label(block_cnt);

            blocks.push((cond_lbl.clone(), String::new()));
            blocks.push((body_lbl.clone(), String::new()));
            blocks.push((merge_lbl.clone(), String::new()));

            // 1. 无条件跳转到条件判断块
            emit(blocks, current_block, &format!("  br label %{}\n", cond_lbl));

            // 2. 生成条件判断块
            *current_block = cond_lbl.clone();
            let (cond_reg, _) = gen_expr(cond, blocks, current_block, reg, vars, strings, ast);
            emit(blocks, current_block, &format!("  br i1 {}, label %{}, label %{}\n", cond_reg, body_lbl, merge_lbl));

            // 3. 生成循环体块
            *current_block = body_lbl.clone();
            for s in body_stmts { gen_stmt(s, blocks, current_block, reg, block_cnt, vars, strings, ast); }
            // 循环体执行完后，无条件跳回条件判断块 (Back-edge)
            emit(blocks, current_block, &format!("  br label %{}\n", cond_lbl));

            // 4. 切换到合并块继续后续代码
            *current_block = merge_lbl;
        }
    }
}

fn gen_expr(expr: &Expr, blocks: &mut Vec<(String, String)>, current_block: &mut String, reg: &mut i32, vars: &HashMap<String, String>, strings: &HashMap<String, String>, ast: &[TopLevel]) -> (String, String) {
    match expr {
        Expr::Number(n) => {
            let r = format!("%{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = add i64 0, {}\n", r, n));
            (r, "i64".to_string())
        }
        Expr::Bool(b) => {
            let r = format!("%{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = add i1 0, {}\n", r, if *b { 1 } else { 0 }));
            (r, "i1".to_string())
        }
        Expr::Identifier(name) => {
            let ptr = vars.get(name).unwrap();
            let r = format!("%{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = load i64, i64* {}\n", r, ptr));
            (r, "i64".to_string())
        }
        Expr::BinOp(l, op, r_expr) => {
            let (l_reg, _) = gen_expr(l, blocks, current_block, reg, vars, strings, ast);
            let (r_reg, _) = gen_expr(r_expr, blocks, current_block, reg, vars, strings, ast);
            let res = format!("%{}", reg); *reg += 1;
            let inst = match op.as_str() {
                "+" => "add", "-" => "sub", "*" => "mul", "/" => "sdiv",
                _ => panic!("Unknown op"),
            };
            emit(blocks, current_block, &format!("  {} = {} i64 {}, {}\n", res, inst, l_reg, r_reg));
            (res, "i64".to_string())
        }
        Expr::Compare(l, op, r_expr) => {
            let (l_reg, _) = gen_expr(l, blocks, current_block, reg, vars, strings, ast);
            let (r_reg, _) = gen_expr(r_expr, blocks, current_block, reg, vars, strings, ast);
            let res = format!("%{}", reg); *reg += 1;
            let inst = match op.as_str() {
                "==" => "icmp eq", "!=" => "icmp ne",
                "<" => "icmp slt", ">" => "icmp sgt",
                "<=" => "icmp sle", ">=" => "icmp sge",
                _ => panic!("Unknown cmp"),
            };
            emit(blocks, current_block, &format!("  {} = {} i64 {}, {}\n", res, inst, l_reg, r_reg));
            (res, "i1".to_string())
        }
        Expr::Assign(name, value) => {
            let (val_reg, _) = gen_expr(value, blocks, current_block, reg, vars, strings, ast);
            let ptr = vars.get(name).unwrap();
            emit(blocks, current_block, &format!("  store i64 {}, i64* {}\n", val_reg, ptr));
            (val_reg, "i64".to_string())
        }
        Expr::Call(func_name, args) => {
            let mut ret_ty = "Int".to_string();
            for t in ast { if let TopLevel::BridgeDecl { name, ret_ty: rt, .. } = t { if name == func_name { ret_ty = rt.clone(); break; } } }
            
            let mut call_args = Vec::new();
            for arg in args {
                let (r, ty) = gen_expr(arg, blocks, current_block, reg, vars, strings, ast);
                if ty == "i64" {
                    let tr = format!("%{}", reg); *reg += 1;
                    emit(blocks, current_block, &format!("  {} = trunc i64 {} to i32\n", tr, r));
                    call_args.push(format!("i32 {}", tr));
                } else {
                    call_args.push(format!("{} {}", ty, r));
                }
            }
            let res = format!("%{}", reg); *reg += 1;
            let rt = if ret_ty == "Int" { "i32" } else { "double" };
            emit(blocks, current_block, &format!("  {} = call {} @{}({})\n", res, rt, func_name, call_args.join(", ")));
            
            if ret_ty == "Int" {
                let ext = format!("%{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = sext i32 {} to i64\n", ext, res));
                return (ext, "i64".to_string());
            }
            (res, rt.to_string())
        }
        Expr::Print(inner) => {
            if let Expr::StringLit(s) = &**inner {
                let name = strings.get(s).unwrap();
                let len = s.len() + 1;
                let fmt_ptr_reg = format!("%{}", reg); *reg += 1;
                let str_ptr_reg = format!("%{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0\n", fmt_ptr_reg));
                emit(blocks, current_block, &format!("  {} = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", str_ptr_reg, len, len, name));
                let call_res = format!("%{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = call i32 (i8*, ...) @printf(i8* {}, i8* {})\n", call_res, fmt_ptr_reg, str_ptr_reg));
                (call_res, "i32".to_string())
            } else {
                let (r, ty) = gen_expr(inner, blocks, current_block, reg, vars, strings, ast);
                let fmt_int_ptr_reg = format!("%{}", reg); *reg += 1;
                let tr = format!("%{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n", fmt_int_ptr_reg));
                
                let val_to_print = if ty == "i64" {
                    emit(blocks, current_block, &format!("  {} = trunc i64 {} to i32\n", tr, r));
                    tr
                } else {
                    r
                };
                
                let call_res = format!("%{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = call i32 (i8*, ...) @printf(i8* {}, i32 {})\n", call_res, fmt_int_ptr_reg, val_to_print));
                (call_res, "i32".to_string())
            }
        }
        _ => {
            let r = format!("%{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = add i64 0, 0\n", r));
            (r, "i64".to_string())
        }
    }
}

fn collect_stmt_strings(stmt: &Stmt, globals: &mut HashMap<String, String>, counter: &mut i32) {
    match stmt {
        Stmt::Expr(e) => collect_expr_strings(e, globals, counter),
        Stmt::If(_, then_stmts, else_stmts) => {
            for s in then_stmts { collect_stmt_strings(s, globals, counter); }
            for s in else_stmts { collect_stmt_strings(s, globals, counter); }
        }
        // 【新增】递归收集 While 块内的字符串
        Stmt::While(_, body_stmts) => {
            for s in body_stmts { collect_stmt_strings(s, globals, counter); }
        }
    }
}

fn collect_expr_strings(expr: &Expr, globals: &mut HashMap<String, String>, counter: &mut i32) {
    match expr {
        Expr::StringLit(s) => { 
            if !globals.contains_key(s) { 
                globals.insert(s.clone(), format!(".str_{}", counter)); 
                *counter += 1; 
            } 
        },
        Expr::Print(inner) => collect_expr_strings(inner, globals, counter),
        Expr::BinOp(l, _, r) | Expr::Compare(l, _, r) => {
            collect_expr_strings(l, globals, counter);
            collect_expr_strings(r, globals, counter);
        },
        Expr::Call(_, args) => {
            for arg in args { collect_expr_strings(arg, globals, counter); }
        },
        Expr::Assign(_, e) => collect_expr_strings(e, globals, counter),
        _ => {}
    }
}