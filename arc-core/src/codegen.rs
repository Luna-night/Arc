use crate::{TopLevel, Expr, BridgeParam};
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[TopLevel]) -> String {
    let mut ir = String::new();
    let mut reg_counter = 0; 
    let mut vars: HashMap<String, String> = HashMap::new();

    ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n\n");

    // 1. 收集字符串
    let mut string_globals: HashMap<String, String> = HashMap::new();
    let mut str_counter = 0;
    for top in ast {
        if let TopLevel::Statement(Expr::Print(inner)) = top { 
            collect_strings(inner, &mut string_globals, &mut str_counter); 
        }
    }
    for (content, name) in &string_globals {
        let len = content.len() + 1;
        ir.push_str(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", name, len, content));
    }
    ir.push_str("\n");

    // 2. 生成 Bridge 声明
    for top in ast {
        if let TopLevel::BridgeDecl { lang: l, name, params, ret_ty, .. } = top {
            if l == "c" {
                let pt: Vec<String> = params.iter().map(|p| {
                    let BridgeParam::Param { ty, .. } = p;
                    if ty == "Int" { "i32" } else { "double" }.to_string()
                }).collect();
                let rt = if ret_ty == "Int" { "i32" } else { "double" };
                ir.push_str(&format!("declare {} @{}({})\n", rt, name, pt.join(", ")));
            }
        }
    }
    if ast.iter().any(|t| matches!(t, TopLevel::BridgeDecl { lang: l, .. } if l == "c")) { 
        ir.push_str("\n"); 
    }

    // 3. 生成 main 函数
    ir.push_str("define i32 @main() {\nentry:\n");

    // 3.1 预分配所有变量的内存 (alloca)
    for top in ast {
        if let TopLevel::LetDecl { name, .. } = top {
            let ptr = format!("%{}", reg_counter);
            reg_counter += 1;
            vars.insert(name.clone(), ptr.clone());
            ir.push_str(&format!("  {} = alloca i64\n", ptr));
        }
    }

    // 3.2 生成执行代码
    for top in ast {
        match top {
            TopLevel::LetDecl { name, value } => {
                let val_reg = gen_expr(value, &mut ir, &mut reg_counter, &vars, &string_globals, ast);
                let ptr = vars.get(name).unwrap();
                ir.push_str(&format!("  store i64 {}, i64* {}\n", val_reg, ptr));
            }
            TopLevel::Statement(expr) => {
                let _ = gen_expr(expr, &mut ir, &mut reg_counter, &vars, &string_globals, ast);
            }
            _ => {}
        }
    }

    ir.push_str("  ret i32 0\n}\n");
    ir
}

// 辅助：生成表达式的 LLVM IR，返回存放结果的寄存器名
fn gen_expr(expr: &Expr, ir: &mut String, reg: &mut i32, vars: &HashMap<String, String>, strings: &HashMap<String, String>, ast: &[TopLevel]) -> String {
    match expr {
        Expr::Number(n) => {
            let r = format!("%{}", reg); *reg += 1;
            ir.push_str(&format!("  {} = add i64 0, {}\n", r, n));
            r
        }
        Expr::Identifier(name) => {
            let ptr = vars.get(name).unwrap();
            let r = format!("%{}", reg); *reg += 1;
            ir.push_str(&format!("  {} = load i64, i64* {}\n", r, ptr));
            r
        }
        Expr::BinOp(l, op, r_expr) => {
            let l_reg = gen_expr(l, ir, reg, vars, strings, ast);
            let r_reg = gen_expr(r_expr, ir, reg, vars, strings, ast);
            let res = format!("%{}", reg); *reg += 1;
            let inst = match op.as_str() {
                "+" => "add", "-" => "sub", "*" => "mul", "/" => "sdiv",
                _ => panic!("Unknown op"),
            };
            ir.push_str(&format!("  {} = {} i64 {}, {}\n", res, inst, l_reg, r_reg));
            res
        }
        Expr::Call(func_name, args) => {
            let mut ret_ty = "Int".to_string();
            for t in ast { 
                if let TopLevel::BridgeDecl { name, ret_ty: rt, .. } = t { 
                    if name == func_name { ret_ty = rt.clone(); break; } 
                } 
            }
            
            let mut call_args = Vec::new();
            for arg in args {
                if let Expr::Number(n) = arg { 
                    call_args.push(format!("i32 {}", n)); 
                } else { 
                    let r = gen_expr(arg, ir, reg, vars, strings, ast); 
                    let tr = format!("%{}", reg); *reg += 1;
                    ir.push_str(&format!("  {} = trunc i64 {} to i32\n", tr, r));
                    call_args.push(format!("i32 {}", tr));
                }
            }
            let res = format!("%{}", reg); *reg += 1;
            let rt = if ret_ty == "Int" { "i32" } else { "double" };
            ir.push_str(&format!("  {} = call {} @{}({})\n", res, rt, func_name, call_args.join(", ")));
            
            if ret_ty == "Int" {
                let ext = format!("%{}", reg); *reg += 1;
                ir.push_str(&format!("  {} = sext i32 {} to i64\n", ext, res));
                return ext;
            }
            res
        }
        Expr::Print(inner) => {
            if let Expr::StringLit(s) = &**inner {
                let name = strings.get(s).unwrap();
                let len = s.len() + 1;
                
                let fmt_ptr_reg = format!("%{}", reg); *reg += 1;
                let str_ptr_reg = format!("%{}", reg); *reg += 1;
                
                ir.push_str(&format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0\n", fmt_ptr_reg));
                ir.push_str(&format!("  {} = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", str_ptr_reg, len, len, name));
                
                // 【核心修复】显式接收 printf 的返回值
                let call_res = format!("%{}", reg); *reg += 1;
                ir.push_str(&format!("  {} = call i32 (i8*, ...) @printf(i8* {}, i8* {})\n", call_res, fmt_ptr_reg, str_ptr_reg));
                
                return call_res;
            } else {
                let r = gen_expr(inner, ir, reg, vars, strings, ast);
                
                let fmt_int_ptr_reg = format!("%{}", reg); *reg += 1;
                let tr = format!("%{}", reg); *reg += 1;
                
                ir.push_str(&format!("  {} = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n", fmt_int_ptr_reg));
                ir.push_str(&format!("  {} = trunc i64 {} to i32\n", tr, r));
                
                // 【核心修复】显式接收 printf 的返回值
                let call_res = format!("%{}", reg); *reg += 1;
                ir.push_str(&format!("  {} = call i32 (i8*, ...) @printf(i8* {}, i32 {})\n", call_res, fmt_int_ptr_reg, tr));
                
                return call_res;
            }
        }
        _ => {
            let r = format!("%{}", reg); *reg += 1;
            ir.push_str(&format!("  {} = add i64 0, 0\n", r));
            r
        }
    }
}

fn collect_strings(expr: &Expr, globals: &mut HashMap<String, String>, counter: &mut i32) {
    if let Expr::StringLit(s) = expr { 
        if !globals.contains_key(s) { 
            globals.insert(s.clone(), format!(".str_{}", counter)); 
            *counter += 1; 
        } 
    }
    if let Expr::Print(inner) = expr { 
        collect_strings(inner, globals, counter); 
    }
}