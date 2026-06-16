use arc_core::{parser, Token, codegen};
use logos::Logos;
use chumsky::Parser;
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("Usage:");
        println!("  arc-cli run <file.arc>   (解释执行)");
        println!("  arc-cli build <file.arc> (编译为原生机器码)");
        return;
    }

    let command = &args[1];
    let filename = &args[2];
    let source = fs::read_to_string(filename).expect("Failed to read file");
    
    let tokens: Vec<Token> = Token::lexer(&source).filter_map(|t| t.ok()).collect();
    
    match parser().parse(tokens) {
        Ok(ast) => {
            if command == "run" {
                // 解释模式
                let mut env = arc_core::Environment::new();
                for top_level in &ast {
                    // 因为 AST 顶层现在是 TopLevel，我们需要提取里面的 Statement 来执行
                    if let arc_core::TopLevel::Statement(expr) = top_level {
                        let _ = env.eval(expr);
                    }
                }
            } 
            else if command == "build" {
                // AOT 编译模式
                println!("⚙️  Generating LLVM IR with Arc Bridge...");
                
                let llvm_ir = codegen::compile_to_llvm_ir(&ast);
                
                let ll_filename = "arc_out.ll";
                fs::write(ll_filename, &llvm_ir).expect("Failed to write .ll file");
                println!("✅ Saved IR to {}", ll_filename);

                println!("🔨 Compiling to native binary using Clang...");
                let output_bin = "arc_app";
                
                let status = Command::new("clang")
                    .arg(ll_filename)
                    .arg("-O3")
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
} // <--- 确保这个最后的括号存在！