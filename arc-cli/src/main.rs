use arc_core::{parser, Token, codegen};
use logos::Logos;
use chumsky::Parser;
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // 简单的命令解析：arc-cli <命令> <文件>
    if args.len() < 3 {
        println!("Usage:");
        println!("  arc-cli run <file.arc>   (解释执行)");
        println!("  arc-cli build <file.arc> (编译为原生机器码)");
        return;
    }

    let command = &args[1];
    let filename = &args[2];
    let source = fs::read_to_string(filename).expect("Failed to read file");
    
    // 1. 词法与语法分析 (和之前一样)
    let tokens: Vec<Token> = Token::lexer(&source).filter_map(|t| t.ok()).collect();
    
    match parser().parse(tokens) {
        Ok(ast) => {
            if command == "run" {
                // 【解释模式】
                let mut env = arc_core::Environment::new();
                for expr in &ast {
                    // eval 现在返回 Result<Value, String>，我们直接忽略 Ok 里的值，或者可以打印出来
                    let _ = env.eval(expr); 
                }
            } 
            else if command == "build" {
                // 【AOT 编译模式】🚀
                println!("⚙️  Generating LLVM IR...");
                
                // 2. 生成 LLVM IR 文本
                let llvm_ir = codegen::compile_to_llvm_ir(&ast);
                
                // 3. 将 IR 写入 .ll 文件
                let ll_filename = "arc_out.ll";
                fs::write(ll_filename, &llvm_ir).expect("Failed to write .ll file");
                println!("✅ Saved IR to {}", ll_filename);
                
                // 你可以取消注释下面这行，看看生成的 LLVM IR 有多优雅！
                // println!("--- LLVM IR Preview ---\n{}\n---------------------", llvm_ir);

                // 4. 调用系统的 clang 编译器，将 .ll 编译成真正的机器码可执行文件
                println!("🔨 Compiling to native binary using Clang...");
                let output_bin = "arc_app"; // 输出的可执行文件名
                
                let status = Command::new("clang")
                    .arg(ll_filename)
                    .arg("-O3") // 开启 Clang 的极致优化！
                    .arg("-o")
                    .arg(output_bin)
                    .status()
                    .expect("❌ Failed to execute clang. Is LLVM/Clang installed?");

                if status.success() {
                    println!("🎉 Success! Native binary generated: ./{}", output_bin);
                    println!("👉 Run it with: ./{}", output_bin);
                } else {
                    println!("❌ Compilation failed.");
                }
            } else {
                println!("Unknown command: {}. Use 'run' or 'build'.", command);
            }
        }
        Err(e) => {
            for err in e {
                println!("❌ Parse Error: {}", err);
            }
        }
    }
}