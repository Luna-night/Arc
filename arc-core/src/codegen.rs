use crate::{TopLevel, Expr};
use std::collections::HashMap;

// 注意：参数类型改为 &[TopLevel]
pub fn compile_to_llvm_ir(ast: &[TopLevel]) -> String {
    let mut ir = String::new();

    ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");

    // 收集字符串常量 (和之前一样)
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

    // 【关键】第一遍扫描：生成 Bridge 的 declare 语句
    for top in ast {
        if let TopLevel::BridgeDecl { name, .. } = top {
            // 假设所有 bridge 函数都返回 i32 且无参数 (MVP 简化版)
            ir.push_str(&format!("declare i32 @{}()\n", name));
        }
    }
    if ast.iter().any(|t| matches!(t, TopLevel::BridgeDecl { .. })) {
        ir.push_str("\n");
    }

    // 定义 main 函数
    ir.push_str("define i32 @main() {\n");
    ir.push_str("entry:\n");

    // 第二遍扫描：生成 main 函数体内的执行代码
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
                        // 【关键】处理 Bridge 函数调用
                        Expr::Call(func_name) => {
                            // 1. 调用外部函数 (例如 getpid)
                            ir.push_str(&format!("  %pid = call i32 @{}()\n", func_name));
                            // 2. 打印结果
                            ir.push_str("  %fmt_int_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                            ir.push_str("  call i32 (i8*, ...) @printf(i8* %fmt_int_ptr, i32 %pid)\n");
                        }
                        _ => {}
                    }
                }
                _ => {} // 忽略非 Print 语句
            }
        }
    }

    ir.push_str("  ret i32 0\n");
    ir.push_str("}\n");

    ir
}