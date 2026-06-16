use crate::{TopLevel, Expr, BridgeParam};
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[TopLevel]) -> String {
    let mut ir = String::new();
    let mut reg_counter = 1; // 用于生成唯一的寄存器名 %0, %1...

    ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n\n");

    // 1. 收集字符串常量
    let mut string_globals: HashMap<String, String> = HashMap::new();
    let mut str_counter = 0;
    for top in ast {
        if let TopLevel::Statement(Expr::Print(inner)) = top {
            collect_strings(inner, &mut string_globals, &mut str_counter);
        }
        if let TopLevel::Statement(Expr::Call(_, args)) = top {
            for arg in args {
                if let Expr::StringLit(s) = arg {
                    if !string_globals.contains_key(s) {
                        let name = format!(".str_{}", str_counter);
                        string_globals.insert(s.clone(), name);
                        str_counter += 1;
                    }
                }
            }
        }
    }
    for (content, name) in &string_globals {
        let len = content.len() + 1;
        ir.push_str(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", name, len, content));
    }
    ir.push_str("\n");

    // 2. 生成 Bridge 声明
    for top in ast {
        if let TopLevel::BridgeDecl { lang, name, params, ret_ty, .. } = top {
            if lang == "c" {
                let param_types: Vec<String> = params.iter().map(|p| {
                    match p {
                        BridgeParam::Param { ty, .. } => if ty == "Int" { "i32".to_string() } else { "double".to_string() }
                    }
                }).collect();
                
                let ret_type = if ret_ty == "Int" { "i32" } else { "double" };
                ir.push_str(&format!("declare {} @{}({})\n", ret_type, name, param_types.join(", ")));
            }
        }
    }
    if ast.iter().any(|t| matches!(t, TopLevel::BridgeDecl { lang: l, .. } if l == "c")) {
        ir.push_str("\n");
    }

    // 3. 生成 main 函数
    ir.push_str("define i32 @main() {\n");
    ir.push_str("entry:\n");

    for top in ast {
        if let TopLevel::Statement(expr) = top {
            match expr {
                Expr::Print(inner_expr) => {
                    // 如果是 Print(Call(...))，我们需要先执行 Call，再打印结果
                    if let Expr::Call(func_name, args) = &**inner_expr {
                        // 查找对应的 bridge 声明获取返回类型
                        let mut ret_ty = "Int".to_string();
                        for t in ast {
                            if let TopLevel::BridgeDecl { name, ret_ty: rt, .. } = t {
                                if name == func_name { ret_ty = rt.clone(); break; }
                            }
                        }

                        // 生成参数
                        let mut call_args = Vec::new();
                        for arg in args {
                            if let Expr::Number(n) = arg {
                                call_args.push(format!("i32 {}", n));
                            } else if let Expr::StringLit(s) = arg {
                                let name = string_globals.get(s).unwrap();
                                let len = s.len() + 1;
                                // 这里简化处理，假设字符串参数是指针
                                call_args.push(format!("i8* getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0", len, len, name));
                            }
                        }

                        let res_reg = format!("%{}", reg_counter); reg_counter += 1;
                        let ret_type = if ret_ty == "Int" { "i32" } else { "double" };
                        
                        ir.push_str(&format!("  {} = call {} @{}({})\n", res_reg, ret_type, func_name, call_args.join(", ")));

                        // 打印结果
                        if ret_ty == "Int" {
                            ir.push_str("  %fmt_int_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                            ir.push_str(&format!("  call i32 (i8*, ...) @printf(i8* %fmt_int_ptr, i32 {})\n", res_reg));
                        }
                    } else {
                        // 普通的 Print (数字或字符串)
                        match &**inner_expr {
                            Expr::Number(n) => {
                                ir.push_str("  %fmt_int_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                                ir.push_str(&format!("  call i32 (i8*, ...) @printf(i8* %fmt_int_ptr, i32 {})\n", n));
                            }
                            Expr::StringLit(s) => {
                                let name = string_globals.get(s).unwrap();
                                let len = s.len() + 1;
                                ir.push_str("  %fmt_str_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0\n");
                                ir.push_str(&format!("  %str_ptr = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", len, len, name));
                                ir.push_str("  call i32 (i8*, ...) @printf(i8* %fmt_str_ptr, i8* %str_ptr)\n");
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    ir.push_str("  ret i32 0\n");
    ir.push_str("}\n");

    ir
}

// 辅助函数：收集字符串
fn collect_strings(expr: &Expr, globals: &mut HashMap<String, String>, counter: &mut i32) {
    if let Expr::StringLit(s) = expr {
        if !globals.contains_key(s) {
            let name = format!(".str_{}", counter);
            globals.insert(s.clone(), name);
            *counter += 1;
        }
    }
    if let Expr::Print(inner) = expr {
        collect_strings(inner, globals, counter);
    }
}