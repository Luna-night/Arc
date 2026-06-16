use crate::{Expr};

/// 将 Arc 的 AST 编译为 LLVM IR 文本
pub fn compile_to_llvm_ir(ast: &[Expr]) -> String {
    let mut ir = String::new();
use crate::Expr;
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[Expr]) -> String {
    let mut ir = String::new();

    // 1. 声明 C 标准库的 printf 函数
    ir.push_str("declare i32 @printf(i8*, ...)\n\n");

    // 2. 定义全局格式化字符串
    // %d\n 用于打印整数，%s\n 用于打印字符串
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");

    // 3. 扫描 AST，收集所有需要打印的字符串字面量，生成全局常量
    let mut string_globals: HashMap<String, String> = HashMap::new(); // content -> global_name
    let mut str_counter = 0;

    for expr in ast {
        if let Expr::Print(inner) = expr {
            if let Expr::StringLit(s) = **inner {
                if !string_globals.contains_key(s) {
                    let name = format!(".str_{}", str_counter);
                    string_globals.insert(s.clone(), name);
                    str_counter += 1;
                }
            }
        }
    }

    // 定义收集到的字符串常量 (例如: @.str_0 = constant [10 x i8] c"Hello Arc\00")
    for (content, name) in &string_globals {
        let len = content.len() + 1; // +1 for null terminator
        ir.push_str(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n", name, len, content));
    }
    ir.push_str("\n");

    // 4. 定义 main 函数
    ir.push_str("define i32 @main() {\n");
    ir.push_str("entry:\n");

    for expr in ast {
        if let Expr::Print(inner_expr) = expr {
            match **inner_expr {
                Expr::Number(n) => {
                    // 打印整数
                    ir.push_str("  %fmt_int_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                    ir.push_str(&format!("  call i32 (i8*, ...) @printf(i8* %fmt_int_ptr, i32 {})\n", n));
                }
                Expr::StringLit(ref s) => {
                    // 打印字符串
                    let name = string_globals.get(s).unwrap();
                    let len = s.len() + 1;
                    
                    // 获取 "%s\n" 的指针
                    ir.push_str("  %fmt_str_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_str, i64 0, i64 0\n");
                    // 获取字符串常量的指针
                    ir.push_str(&format!("  %str_ptr = getelementptr inbounds [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n", len, len, name));
                    // 调用 printf
                    ir.push_str("  call i32 (i8*, ...) @printf(i8* %fmt_str_ptr, i8* %str_ptr)\n");
                }
                _ => {} // 暂时不支持打印其他类型
            }
        }
    }

    ir.push_str("  ret i32 0\n");
    ir.push_str("}\n");

    ir
}