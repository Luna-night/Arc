use arc_core::{parser, Token, codegen};
use logos::Logos;
use chumsky::Parser;
use std::env;
use std::fs;
use std::process::Command;
use pyo3::prelude::*;

// Python 包装函数示例
// 这些函数将在 LLVM IR 中被调用

#[pyfunction]
fn py_math_sqrt() -> PyResult<f64> {
    Python::with_gil(|py| {
        let math = PyModule::import(py, "math")?;
        let result: f64 = math.getattr("sqrt")?.call0()?.extract()?;
        Ok(result)
    })
}

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
                println!("Interpreter mode: Python Bridge not fully supported yet. Please use 'build'.");
            } 
            else if command == "build" {
                println!("⚙️  Generating LLVM IR with Python Bridge...");
                
                let llvm_ir = codegen::compile_to_llvm_ir(&ast);
                
                let ll_filename = "arc_out.ll";
                fs::write(ll_filename, &llvm_ir).expect("Failed to write .ll file");
                println!("✅ Saved IR to {}", ll_filename);

                println!("🔨 Compiling to native binary using Clang...");
                let output_bin = "arc_app";
                
                // 需要链接 Python 库
                let status = Command::new("clang")
                    .arg(ll_filename)
                    .arg("-O3")
                    .arg("-o")
                    .arg(output_bin)
                    // 这里需要添加 pyo3 的链接 flags
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