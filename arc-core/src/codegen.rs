use crate::{TopLevel, Expr, BridgeParam};
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[TopLevel]) -> String {
    let mut ir = String::new();

    ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");
    ir.push_str("@.fmt_float = private unnamed_addr constant [6 x i8] c\"%f\\0A\\00\"\n\n");

    // 收集字符串
    let mut string_globals: HashMap<String, String> = HashMap::new();
    let mut str_counter = 0;
    for top in ast {
        if let TopLevel::Statement(Expr::Print(inner)) = top {
            if let Expr::StringLit(s) = &**inner {
                if !string_globals.contains_key(s) {
                    let name = format!(".str_{}", str_counter);
                    string_globals.insert(s.clone(), name);
                    str_counter += 1;
                }
            }
        }
    }
    for (content, name) in &string_globals {
        let len = content.len() + 1;
        ir.push_str(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", name, len, content));
    }
    ir.push_str("\n");

    // 生成 Bridge 声明
    // 对于 bridge py，我们声明调用 Rust 包装函数
    for top in ast {
        if let TopLevel::BridgeDecl { lang, lib, name, params, ret_ty } = top {
            if lang == "py" {
                // 生成 Rust 包装函数的声明
                // 例如: declare double @py_math_sqrt(double)
                let param_types: Vec<String> = params.iter().map(|_| "double".to_string()).collect();
                let ret_type = if ret_ty == "Float" { "double" } else { "i32" };
                ir.push_str(&format!("declare {} @py_{}_{}({})\n", 
                    ret_type, 
                    lib.replace("\"", ""), 
                    name, 
                    param_types.join(", ")
                ));
            } else {
                // C bridge
                ir.push_str(&format!("declare i32 @{}()\n", name));
            }
        }
    }
    ir.push_str("\n");

    // 生成 main 函数
    ir.push_str("define i32 @main() {\n");
    ir.push_str("entry:\n");

    for top in ast {
        if let TopLevel::Statement(expr) = top {
            match expr {
                Expr::Print(inner_expr) => {
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
                        Expr::Call(func_name) => {
                            // 查找对应的 bridge 声明
                            for bridge_top in ast {
                                if let TopLevel::BridgeDecl { lang, lib, name, params, ret_ty } = bridge_top {
                                    if name == func_name {
                                        if lang == "py" {
                                            // 调用 Python 包装函数（目前硬编码无参）
                                            let lib_clean = lib.replace("\"", "");
                                            if ret_ty == "Float" {
                                                ir.push_str(&format!("  %result = call double @py_{}_{}()\n", lib_clean, name));
                                                ir.push_str("  %fmt_float_ptr = getelementptr inbounds [6 x i8], [6 x i8]* @.fmt_float, i64 0, i64 0\n");
                                                ir.push_str("  call i32 (i8*, ...) @printf(i8* %fmt_float_ptr, double %result)\n");
                                            } else {
                                                ir.push_str(&format!("  %result = call i32 @py_{}_{}()\n", lib_clean, name));
                                                ir.push_str("  %fmt_int_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                                                ir.push_str("  call i32 (i8*, ...) @printf(i8* %fmt_int_ptr, i32 %result)\n");
                                            }
                                        } else {
                                            // C bridge
                                            ir.push_str(&format!("  %pid = call i32 @{}()\n", func_name));
                                            ir.push_str("  %fmt_int_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                                            ir.push_str("  call i32 (i8*, ...) @printf(i8* %fmt_int_ptr, i32 %pid)\n");
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
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