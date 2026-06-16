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
    ir_globals.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");
    ir_globals.push_str("@.fmt_float = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\"\n\n"); // 【新增】

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
                    match ty.as_str() {
                        "Int" => "i32".to_string(),
                        "Float" => "double".to_string(),
                        "String" => "i8*".to_string(), // 【新增】
                        _ => "i32".to_string(),
                    }
                }).collect();
                let rt = match ret_ty.as_str() {
                    "Int" => "i32",
                    "Float" => "double",
                    _ => "i32",
                };
                ir_globals.push_str(&format!("declare {} @{}({})\n", rt, name, pt.join(", ")));
            }
        }
    }

    // 3. 预分配变量 (alloca i64) - 简化：所有变量都是 i64
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
                // 如果值是 double，需要转 i64 存储 (fptosi)，或者我们暂时只允许 let 存 Int
                // 简化：假设 let 只存 Int
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
        Stmt::Expr(e) => { gen_expr(e, blocks, current_block, reg, vars, strings, ast); }
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
        Stmt::While(cond, body_stmts) => {
            let cond_lbl = new_label(block_cnt);
            let body_lbl = new_label(block_cnt);
            let merge_lbl = new_label(block_cnt);
            blocks.push((cond_lbl.clone(), String::new()));
            blocks.push((body_lbl.clone(), String::new()));
            blocks.push((merge_lbl.clone(), String::new()));
            emit(blocks, current_block, &format!("  br label %{}\n", cond_lbl));
            *current_block = cond_lbl.clone();
            let (cond_reg, _) = gen_expr(cond, blocks, current_block, reg, vars, strings, ast);
            emit(blocks, current_block, &format!("  br i1 {}, label %{}, label %{}\n", cond_reg, body_lbl, merge_lbl));
            *current_block = body_lbl.clone();
            for s in body_stmts { gen_stmt(s, blocks, current_block, reg, block_cnt, vars, strings, ast); }
            emit(blocks, current_block, &format!("  br label %{}\n", cond_lbl));
            *current_block = merge_lbl;
        }
    }
}

// 返回 (寄存器名, LLVM 类型名 "i64", "double", "i1", "i8*")
fn gen_expr(expr: &Expr, blocks: &mut Vec<(String, String)>, current_block: &mut String, reg: &mut i32, vars: &HashMap<String, String>, strings: &HashMap<String, String>, ast: &[TopLevel]) -> (String, String) {
    match expr {
        Expr::Number(n) => {
            let r = format!("%{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = add i64 0, {}\n", r, n));
            (r, "i64".to_string())
        }
        Expr::FloatLit(n) => {
            let r = format!("%{}", reg); *reg += 1;
            // 使用 fadd 加载浮点常量
            emit(blocks, current_block, &format!("  {} = fadd double 0.000000e+00, {:e}\n", r, n));
            (r, "double".to_string())
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
            let (l_reg, l_ty) = gen_expr(l, blocks, current_block, reg, vars, strings, ast);
            let (r_reg, r_ty) = gen_expr(r_expr, blocks, current_block, reg, vars, strings, ast);
            let res = format!("%{}", reg); *reg += 1;
            
            // 简单处理：如果两边都是 double，用浮点运算
            if l_ty == "double" && r_ty == "double" {
                let inst = match op.as_str() {
                    "+" => "fadd", "-" => "fsub", "*" => "fmul", "/" => "fdiv",
                    _ => panic!("Unknown float op"),
                };
                emit(blocks, current_block, &format!("  {} = {} double {}, {}\n", res, inst, l_reg, r_reg));
                (res, "double".to_string())
            } else {
                // 默认整数运算
                let inst = match op.as_str() {
                    "+" => "add", "-" => "sub", "*" => "mul", "/" => "sdiv",
                    _ => panic!("Unknown op"),
                };
                emit(blocks, current_block, &format!("  {} = {} i64 {}, {}\n", res, inst, l_reg, r_reg));
                (res, "i64".to_string())
            }
        }
        Expr::Compare(l, op, r_expr) => {
            let (l_reg, l_ty) = gen_expr(l, blocks, current_block, reg, vars, strings, ast);
            let (r_reg, r_ty) = gen_expr(r_expr, blocks, current_block, reg, vars, strings, ast);
            let res = format!("%{}", reg); *reg += 1;
            
            if l_ty == "double" && r_ty == "double" {
                let inst = match op.as_str() {
                    "==" => "fcmp oeq", "!=" => "fcmp one",
                    "<" => "fcmp olt", ">" => "fcmp ogt",
                    "<=" => "fcmp ole", ">=" => "fcmp oge",
                    _ => panic!("Unknown float cmp"),
                };
                emit(blocks, current_block, &format!("  {} = {} double {}, {}\n", res, inst, l_reg, r_reg));
            } else {
                let inst = match op.as_str() {
                    "==" => "icmp eq", "!=" => "icmp ne",
                    "<" => "icmp slt", ">" => "icmp sgt",
                    "<=" => "icmp sle", ">=" => "icmp sge",
                    _ => panic!("Unknown cmp"),
                };
                emit(blocks, current_block, &format!("  {} = {} i64 {}, {}\n", res, inst, l_reg, r_reg));
            }
            (res, "i1".to_string())
        }
        Expr::Assign(name, value) => {
            let (val_reg, _) = gen_expr(value, blocks, current_block, reg, vars, strings, ast);
            let ptr = vars.get(name).unwrap();
            emit(blocks, current_block, &format!("  store i64 {}, i64* {}\n", val_reg, ptr));
            (val_reg, "i64".to_string())
        }
        Expr::Call(func_name, args) => {
            // 查找 Bridge 声明获取参数类型和返回类型
            let mut param_types: Vec<String> = vec![];
            let mut ret_ty = "Int".to_string();
            for t in ast {
                if let TopLevel::BridgeDecl { name, params, ret_ty: rt, .. } = t {
                    if name == func_name {
                        ret_ty = rt.clone();
                        for p in params {
                            let BridgeParam::Param { ty, .. } = p;
                            param_types.push(ty.clone());
                        }
                        break;
                    }
                }
            }
            
            let mut call_args = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                let expected_ty = param_types.get(i).map(|s| s.as_str()).unwrap_or("Int");
                let (r, actual_ty) = gen_expr(arg, blocks, current_block, reg, vars, strings, ast);
                
                if expected_ty == "Int" {
                    if actual_ty == "i64" {
                        let tr = format!("%{}", reg); *reg += 1;
                        emit(blocks, current_block, &format!("  {} = trunc i64 {} to i32\n", tr, r));
                        call_args.push(format!("i32 {}", tr));
                    }
                } else if expected_ty == "Float" {
                    if actual_ty == "double" {
                        call_args.push(format!("double {}", r));
                    } else if actual_ty == "i64" {
                        // 整数转浮点
                        let cv = format!("%{}", reg); *reg += 1;
                        emit(blocks, current_block, &format!("  {} = sitofp i64 {} to double\n", cv, r));
                        call_args.push(format!("double {}", cv));
                    }
                } else if expected_ty == "String" {
                    // 必须是字符串字面量，获取指针
                    if let Expr::StringLit(s) = arg {
                        let name = strings.get(s).unwrap();
                        let len = s.len() + 1;
                        let ptr_reg = format!("%{}", reg); *reg += 1;
                        emit(blocks, current_block, &format!("  {} = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", ptr_reg, len, len, name));
                        call_args.push(format!("i8* {}", ptr_reg));
                    }
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
                let fmt_ptr_reg = format!("%{}", reg); *reg += 1;
                
                if ty == "double" {
                    // 【新增】打印浮点数
                    emit(blocks, current_block, &format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_float, i64 0, i64 0\n", fmt_ptr_reg));
                    let call_res = format!("%{}", reg); *reg += 1;
                    emit(blocks, current_block, &format!("  {} = call i32 (i8*, ...) @printf(i8* {}, double {})\n", call_res, fmt_ptr_reg, r));
                    (call_res, "i32".to_string())
                } else {
                    // 打印整数
                    let tr = format!("%{}", reg); *reg += 1;
                    emit(blocks, current_block, &format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n", fmt_ptr_reg));
                    emit(blocks, current_block, &format!("  {} = trunc i64 {} to i32\n", tr, r));
                    let call_res = format!("%{}", reg); *reg += 1;
                    emit(blocks, current_block, &format!("  {} = call i32 (i8*, ...) @printf(i8* {}, i32 {})\n", call_res, fmt_ptr_reg, tr));
                    (call_res, "i32".to_string())
                }
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