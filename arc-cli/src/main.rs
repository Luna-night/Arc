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
                let mut env = arc_core::Environment::new();
                for top_level in &ast {
                    // 【注意这里】变体名是 Stmt，不是 Statement
                    if let arc_core::TopLevel::Stmt(stmt) = top_level {
                        let _ = env.eval(stmt);
                    }
                }
            } 
            else if command == "build" {
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
                    .arg("-lm") // 【新增】链接数学库
                    .arg("-o")
                    .arg(output_bin)
                    .status()
                    .expect("❌ Failed to execute clang.");

                if status.success() {
                    println!("🎉 Success! Native binary generated: ./{}", output_bin);
                } else {
                    println!("❌ Compilation failed.");
                }
            }
        }
        Err(e) => {
            for err in e {
                println!("❌ Parse Error: {}", err);
            }
        }
    }
}