use crate::{Expr};

/// 将 Arc 的 AST 编译为 LLVM IR 文本
pub fn compile_to_llvm_ir(ast: &[Expr]) -> String {
    let mut ir = String::new();

    // 1. 声明 C 标准库的 printf 函数
    // i8* 是字符指针，... 表示可变参数
    ir.push_str("declare i32 @printf(i8*, ...)\n\n");

    // 2. 在内存中定义格式化字符串 "%d\n\0"
    // [4 x i8] 表示 4 个字节。'\n' 在 LLVM 中是 \0A，字符串结尾需要 \00
    ir.push_str("@.fmt_int = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n\n");

    // 3. 定义程序的入口 main 函数，返回 i32 (整数)
    ir.push_str("define i32 @main() {\n");
    ir.push_str("entry:\n");

    // 4. 遍历 AST，生成对应的 LLVM 指令
    for expr in ast {
        if let Expr::Print(inner_expr) = expr {
            // 目前我们的 MVP 只支持 print(数字)
            if let Expr::Number(n) = **inner_expr {
                // 获取字符串 ".fmt_int" 的内存指针 (GEP 指令)
                ir.push_str("  %fmt_ptr = getelementptr inbounds [4 x i8], [4 x i8]* @.fmt_int, i64 0, i64 0\n");
                
                // 调用 printf，传入指针和我们的数字常量
                ir.push_str(&format!("  call i32 (i8*, ...) @printf(i8* %fmt_ptr, i32 {})\n", n));
            }
        }
    }

    // 5. main 函数必须返回 0 (表示程序正常退出)
    ir.push_str("  ret i32 0\n");
    ir.push_str("}\n");

    ir
}