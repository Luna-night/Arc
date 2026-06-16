use crate::Expr;
use std::collections::HashMap;

pub fn compile_to_llvm_ir(ast: &[Expr]) -> String {
    let mut ir = String::new();

    ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
    ir.push_str("@.fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n");

    let mut string_globals: HashMap<String, String> = HashMap::new();
    let mut str_counter = 0;

    for expr in ast {
        if let Expr::Print(inner) = expr {
            // 【关键修复 1】使用 &**inner 进行引用匹配，避免移动所有权
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

    ir.push_str("define i32 @main() {\n");
    ir.push_str("entry:\n");

    for expr in ast {
        if let Expr::Print(inner_expr) = expr {
            // 【关键修复 2】同样使用 &**inner_expr 进行引用匹配
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

    ir.push_str("  ret i32 0\n");
    ir.push_str("}\n");

    ir
}