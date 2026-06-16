use crate::{TopLevel, Expr, Stmt, FuncDecl, BridgeParam};
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[TopLevel]) -> String {
    let mut ir_globals = String::new();
    let mut block_counter = 0;

    ir_globals.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir_globals.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir_globals.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");
    ir_globals.push_str("@.fmt_float = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\"\n\n");

    // 1. 收集字符串
    let mut string_globals: HashMap<String, String> = HashMap::new();
    let mut str_counter = 0;
    for top in ast {
        match top {
            TopLevel::Stmt(stmt) => collect_stmt_strings(stmt, &mut string_globals, &mut str_counter),
            TopLevel::LetDecl { value, .. } => collect_expr_strings(value, &mut string_globals, &mut str_counter),
            TopLevel::FuncDecl(func) => {
                for stmt in &func.body {
                    collect_stmt_strings(stmt, &mut string_globals, &mut str_counter);
                }
            }
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
                        "String" => "i8*".to_string(),
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

    let mut final_ir = ir_globals;

    // 3. 生成所有自定义函数的 IR
    for top in ast {
        if let TopLevel::FuncDecl(func) = top {
            let mut func_reg_counter = 0;
            final_ir.push_str(&gen_function(func, &mut func_reg_counter, &mut block_counter, &string_globals, ast));
        }
    }

    // 4. 生成 main 函数
    final_ir.push_str("define i32 @main() {\n");
    let mut main_blocks: Vec<(String, String)> = vec![("entry".to_string(), String::new())];
    let mut main_current_block = "entry".to_string();
    let mut main_vars: HashMap<String, String> = HashMap::new();
    let mut main_reg_counter = 0;

    // 4.1 预分配 main 中的变量 (使用命名寄存器 %tX)
    for top in ast {
        if let TopLevel::LetDecl { name, .. } = top {
            let ptr = format!("%t{}", main_reg_counter); main_reg_counter += 1;
            main_vars.insert(name.clone(), ptr.clone());
            emit(&mut main_blocks, &main_current_block, &format!("  {} = alloca i64\n", ptr));
        }
    }

    // 4.2 生成 main 的执行代码
    for top in ast {
        match top {
            TopLevel::LetDecl { name, value } => {
                let (val_reg, _) = gen_expr(value, &mut main_blocks, &mut main_current_block, &mut main_reg_counter, &main_vars, &string_globals, ast);
                let ptr = main_vars.get(name).unwrap();
                emit(&mut main_blocks, &main_current_block, &format!("  store i64 {}, i64* {}\n", val_reg, ptr));
            }
            TopLevel::Stmt(stmt) => {
                gen_stmt(stmt, &mut main_blocks, &mut main_current_block, &mut main_reg_counter, &mut block_counter, &main_vars, &string_globals, ast);
            }
            _ => {}
        }
    }

    emit(&mut main_blocks, &main_current_block, "  ret i32 0\n");

    for (name, code) in &main_blocks {
        final_ir.push_str(&format!("{}:\n", name));
        final_ir.push_str(code);
    }
    final_ir.push_str("}\n");

    final_ir
}

// 【终极修复】生成单个自定义函数的 IR (全面使用命名寄存器 %argX 和 %tX)
fn gen_function(func: &FuncDecl, reg: &mut i32, block_cnt: &mut i32, strings: &HashMap<String, String>, ast: &[TopLevel]) -> String {
    let mut blocks: Vec<(String, String)> = vec![("entry".to_string(), String::new())];
    let mut current_block = "entry".to_string();
    let mut vars: HashMap<String, String> = HashMap::new();

    // 1. 生成函数签名 (【核心】参数使用命名寄存器 %arg0, %arg1，彻底避免占用数字空间)
    let mut params_sig = Vec::new();
    let mut param_names = Vec::new();
    for (i, p) in func.params.iter().enumerate() {
        let p_reg = format!("%arg{}", i); 
        params_sig.push(format!("i64 {}", p_reg));
        param_names.push((p.name.clone(), p_reg));
    }

    let mut ir = String::new();
    ir.push_str(&format!("define i64 @{}({}) {{\n", func.name, params_sig.join(", ")));

    // 2. 在 entry 块中为参数分配内存并存储 (使用 %tX)
    for (name, p_reg) in param_names {
        let ptr = format!("%t{}", reg); *reg += 1;
        emit(&mut blocks, &current_block, &format!("  {} = alloca i64\n", ptr));
        emit(&mut blocks, &current_block, &format!("  store i64 {}, i64* {}\n", p_reg, ptr));
        vars.insert(name, ptr);
    }

    // 3. 生成函数体
    for stmt in &func.body {
        gen_stmt(stmt, &mut blocks, &mut current_block, reg, block_cnt, &vars, strings, ast);
    }

    // 4. 确保函数有 return 指令
    let has_ret_i64 = blocks.iter().any(|(_, code)| code.contains("ret i64"));
    if !has_ret_i64 {
        emit(&mut blocks, &current_block, "  ret i64 0\n");
    }

    // 5. 组装函数块
    ir.push_str("entry:\n");
    if let Some((_, entry_code)) = blocks.iter().find(|(n, _)| n == "entry") {
        ir.push_str(entry_code);
    }
    for (name, code) in &blocks {
        if name != "entry" {
            ir.push_str(&format!("{}:\n", name));
            ir.push_str(code);
        }
    }
    ir.push_str("}\n\n");
    ir
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
        Stmt::Return(e) => {
            let (val_reg, _ty) = gen_expr(e, blocks, current_block, reg, vars, strings, ast);
            emit(blocks, current_block, &format!("  ret i64 {}\n", val_reg));
        }
    }
}

fn gen_expr(expr: &Expr, blocks: &mut Vec<(String, String)>, current_block: &mut String, reg: &mut i32, vars: &HashMap<String, String>, strings: &HashMap<String, String>, ast: &[TopLevel]) -> (String, String) {
    match expr {
        Expr::Number(n) => {
            let r = format!("%t{}", reg); *reg += 1; // 【核心】使用 %tX
            emit(blocks, current_block, &format!("  {} = add i64 0, {}\n", r, n));
            (r, "i64".to_string())
        }
        Expr::FloatLit(n) => {
            let r = format!("%t{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = fadd double 0.000000e+00, {:e}\n", r, n));
            (r, "double".to_string())
        }
        Expr::Bool(b) => {
            let r = format!("%t{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = add i1 0, {}\n", r, if *b { 1 } else { 0 }));
            (r, "i1".to_string())
        }
        Expr::Identifier(name) => {
            let ptr = vars.get(name).unwrap();
            let r = format!("%t{}", reg); *reg += 1;
            emit(blocks, current_block, &format!("  {} = load i64, i64* {}\n", r, ptr));
            (r, "i64".to_string())
        }
        Expr::BinOp(l, op, r_expr) => {
            let (l_reg, l_ty) = gen_expr(l, blocks, current_block, reg, vars, strings, ast);
            let (r_reg, r_ty) = gen_expr(r_expr, blocks, current_block, reg, vars, strings, ast);
            let res = format!("%t{}", reg); *reg += 1;
            if l_ty == "double" && r_ty == "double" {
                let inst = match op.as_str() { "+" => "fadd", "-" => "fsub", "*" => "fmul", "/" => "fdiv", _ => panic!("Unknown float op") };
                emit(blocks, current_block, &format!("  {} = {} double {}, {}\n", res, inst, l_reg, r_reg));
                (res, "double".to_string())
            } else {
                let inst = match op.as_str() { "+" => "add", "-" => "sub", "*" => "mul", "/" => "sdiv", _ => panic!("Unknown op") };
                emit(blocks, current_block, &format!("  {} = {} i64 {}, {}\n", res, inst, l_reg, r_reg));
                (res, "i64".to_string())
            }
        }
        Expr::Compare(l, op, r_expr) => {
            let (l_reg, l_ty) = gen_expr(l, blocks, current_block, reg, vars, strings, ast);
            let (r_reg, r_ty) = gen_expr(r_expr, blocks, current_block, reg, vars, strings, ast);
            let res = format!("%t{}", reg); *reg += 1;
            if l_ty == "double" && r_ty == "double" {
                let inst = match op.as_str() { "==" => "fcmp oeq", "!=" => "fcmp one", "<" => "fcmp olt", ">" => "fcmp ogt", "<=" => "fcmp ole", ">=" => "fcmp oge", _ => panic!("Unknown float cmp") };
                emit(blocks, current_block, &format!("  {} = {} double {}, {}\n", res, inst, l_reg, r_reg));
            } else {
                let inst = match op.as_str() { "==" => "icmp eq", "!=" => "icmp ne", "<" => "icmp slt", ">" => "icmp sgt", "<=" => "icmp sle", ">=" => "icmp sge", _ => panic!("Unknown cmp") };
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
            let mut is_custom = false;
            for t in ast {
                if let TopLevel::FuncDecl(f) = t {
                    if f.name == *func_name { is_custom = true; break; }
                }
            }

            if is_custom {
                let mut call_args = Vec::new();
                for arg in args {
                    let (r, _ty) = gen_expr(arg, blocks, current_block, reg, vars, strings, ast);
                    call_args.push(format!("i64 {}", r));
                }
                let res = format!("%t{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = call i64 @{}({})\n", res, func_name, call_args.join(", ")));
                return (res, "i64".to_string());
            } else {
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
                            let tr = format!("%t{}", reg); *reg += 1;
                            emit(blocks, current_block, &format!("  {} = trunc i64 {} to i32\n", tr, r));
                            call_args.push(format!("i32 {}", tr));
                        }
                    } else if expected_ty == "Float" {
                        if actual_ty == "double" {
                            call_args.push(format!("double {}", r));
                        } else if actual_ty == "i64" {
                            let cv = format!("%t{}", reg); *reg += 1;
                            emit(blocks, current_block, &format!("  {} = sitofp i64 {} to double\n", cv, r));
                            call_args.push(format!("double {}", cv));
                        }
                    } else if expected_ty == "String" {
                        if let Expr::StringLit(s) = arg {
                            let name = strings.get(s).unwrap();
                            let len = s.len() + 1;
                            let ptr_reg = format!("%t{}", reg); *reg += 1;
                            emit(blocks, current_block, &format!("  {} = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", ptr_reg, len, len, name));
                            call_args.push(format!("i8* {}", ptr_reg));
                        }
                    }
                }
                
                let res = format!("%t{}", reg); *reg += 1;
                let rt = if ret_ty == "Int" { "i32" } else { "double" };
                emit(blocks, current_block, &format!("  {} = call {} @{}({})\n", res, rt, func_name, call_args.join(", ")));
                
                if ret_ty == "Int" {
                    let ext = format!("%t{}", reg); *reg += 1;
                    emit(blocks, current_block, &format!("  {} = sext i32 {} to i64\n", ext, res));
                    return (ext, "i64".to_string());
                }
                (res, rt.to_string())
            }
        }
        Expr::Print(inner) => {
            if let Expr::StringLit(s) = &**inner {
                let name = strings.get(s).unwrap();
                let len = s.len() + 1;
                let fmt_ptr_reg = format!("%t{}", reg); *reg += 1;
                let str_ptr_reg = format!("%t{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0\n", fmt_ptr_reg));
                emit(blocks, current_block, &format!("  {} = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", str_ptr_reg, len, len, name));
                let call_res = format!("%t{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = call i32 (i8*, ...) @printf(i8* {}, i8* {})\n", call_res, fmt_ptr_reg, str_ptr_reg));
                (call_res, "i32".to_string())
            } else {
                let (r, ty) = gen_expr(inner, blocks, current_block, reg, vars, strings, ast);
                let fmt_int_ptr_reg = format!("%t{}", reg); *reg += 1;
                
                let val_to_print = if ty == "double" {
                    r
                } else {
                    let tr = format!("%t{}", reg); *reg += 1;
                    emit(blocks, current_block, &format!("  {} = trunc i64 {} to i32\n", tr, r));
                    tr
                };
                
                let fmt_to_use = if ty == "double" { "@.fmt_float" } else { "@.fmt_int" };
                let arg_to_use = if ty == "double" { format!("double {}", val_to_print) } else { format!("i32 {}", val_to_print) };
                
                let call_res = format!("%t{}", reg); *reg += 1;
                emit(blocks, current_block, &format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* {}, i64 0, i64 0\n", fmt_int_ptr_reg, fmt_to_use));
                emit(blocks, current_block, &format!("  {} = call i32 (i8*, ...) @printf(i8* {}, {})\n", call_res, fmt_int_ptr_reg, arg_to_use));
                (call_res, "i32".to_string())
            }
        }
        _ => {
            let r = format!("%t{}", reg); *reg += 1;
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
        Stmt::Return(e) => collect_expr_strings(e, globals, counter),
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