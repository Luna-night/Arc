use arc_core::{parser, Token, TopLevel, SystemUnit, Expr};
use logos::Logos;
use chumsky::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage:");
        println!("  arc-cli run <file.arc>      (解释执行)");
        println!("  arc-cli build <file.arc>    (AOT 编译为原生机器码)");
        println!("  arc-cli rebuild <file.arc>  (声明式系统构建 / 调用 Cargo)");
        println!("  arc-cli rollback            (回滚到上一个系统世代)");
        return;
    }

    let command = &args[1];

    // 【新增】处理 rebuild 和 rollback 命令
    if command == "rebuild" {
        if args.len() < 3 {
            println!("❌ Usage: arc-cli rebuild <system.arcaea>");
            return;
        }
        handle_rebuild(&args[2]);
        return;
    }

    if command == "rollback" {
        handle_rollback();
        return;
    }

    // 原有的 run 和 build 逻辑
    if args.len() < 3 {
        println!("❌ Missing file argument.");
        return;
    }

    let filename = &args[2];
    let source = fs::read_to_string(filename).expect("Failed to read file");
    let tokens: Vec<Token> = Token::lexer(&source).filter_map(|t| t.ok()).collect();
    
    match parser().parse(tokens) {
        Ok(ast) => {
            if command == "run" {
                let mut env = arc_core::Environment::new();
                for top_level in &ast {
                    if let arc_core::TopLevel::FuncDecl(func) = top_level {
                        env.functions.insert(func.name.clone(), func.clone());
                    }
                }
                for top_level in &ast {
                    match top_level {
                        arc_core::TopLevel::Stmt(stmt) => { let _ = env.eval_stmt(stmt); }
                        arc_core::TopLevel::SystemDecl { units } => { env.eval_system_decl(units); }
                        _ => {}
                    }
                }
            } 
            else if command == "build" {
                println!("⚙️  Generating LLVM IR with Arc Bridge...");
                let llvm_ir = arc_core::codegen::compile_to_llvm_ir(&ast);
                let ll_filename = "arc_out.ll";
                fs::write(ll_filename, &llvm_ir).expect("Failed to write .ll file");
                println!("✅ Saved IR to {}", ll_filename);

                println!("🔨 Compiling to native binary using Clang...");
                let output_bin = "arc_app";
                let status = Command::new("clang")
                    .arg(ll_filename).arg("-O3").arg("-lm").arg("-o").arg(output_bin)
                    .status().expect("❌ Failed to execute clang.");

                if status.success() {
                    println!("🎉 Success! Native binary generated: ./{}", output_bin);
                } else {
                    println!("❌ Compilation failed.");
                }
            }
        }
        Err(e) => {
            for err in e { println!("❌ Parse Error: {}", err); }
        }
    }
}

// ==========================================
// arcaeaOS 核心：rebuild 与 世代管理
// ==========================================

fn handle_rebuild(filename: &str) {
    println!("\n[A.R.C.A.E.A. SYSTEM] 启动 arcaea-rebuild 协议...");
    let source = fs::read_to_string(filename).expect("Failed to read system.arcaea");
    
    // 1. 计算配置文件的 Hash，生成唯一的 Generation ID
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let gen_hash = format!("{:x}", hasher.finish());
    let gen_id = format!("gen-{}", &gen_hash[..8]);
    
    println!("[FSM] 验证配置文件完整性... Hash: {}", gen_id);

    // 2. 解析 AST
    let tokens: Vec<Token> = Token::lexer(&source).filter_map(|t| t.ok()).collect();
    let ast = parser().parse(tokens).expect("Failed to parse system.arcaea");

    // 3. 创建不可变的世代目录 (模拟 /arcaea/world/generations/<gen_id>)
    let store_path = Path::new("./arcaea_store");
    let gen_path = store_path.join(&gen_id);
    let bin_path = gen_path.join("bin");
    
    fs::create_dir_all(&bin_path).expect("Failed to create generation directory");
    println!("[SYS] 创建隔离世代目录: {}", gen_path.display());

    let mut built_count = 0;

    // 4. 遍历声明，执行构建
    for top in &ast {
        if let TopLevel::SystemDecl { units } = top {
            for unit in units {
                if let SystemUnit::Package { name, config } = unit {
                    println!("\n  ⚙️  正在处理包: {}", name);
                    
                    // 提取配置项
                    let mut source_type = "static";
                    let mut build_path = ".";
                    
                    for item in config {
                        if item.key == "source" {
                            if let Expr::StringLit(s) = &item.value { source_type = s.as_str(); }
                        }
                        if item.key == "path" {
                            if let Expr::StringLit(s) = &item.value { build_path = s.as_str(); }
                        }
                    }

                    // 【核心】如果声明了 source = "cargo"，则唤起 Rust 工具链
                    if source_type == "cargo" {
                        println!("  [CARGO] 检测到 Rust 构建后端，启动沙盒编译...");
                        println!("  [CARGO] 执行: cargo build --release --manifest-path {}/Cargo.toml", build_path);
                        
                        let status = Command::new("cargo")
                            .arg("build")
                            .arg("--release")
                            .arg("--manifest-path")
                            .arg(format!("{}/Cargo.toml", build_path))
                            .status();

                        match status {
                            Ok(s) if s.success() => {
                                // 提取产物到不可变世代目录
                                let target_bin = Path::new(build_path).join("target/release").join(name);
                                if target_bin.exists() {
                                    let dest_bin = bin_path.join(name);
                                    fs::copy(&target_bin, &dest_bin).expect("Failed to extract binary");
                                    println!("  ✅ 产物已提取并锚定至: {}", dest_bin.display());
                                    built_count += 1;
                                } else {
                                    println!("  ⚠️  警告: 未找到编译产物 {}", target_bin.display());
                                }
                            }
                            _ => {
                                println!("  ❌ Cargo 构建失败，触发 ABORT_PROTOCOL，中止 rebuild。");
                                fs::remove_dir_all(&gen_path).unwrap();
                                return;
                            }
                        }
                    } else {
                        println!("  [STATIC] 生成配置文件...");
                        // 这里可以生成普通的配置文件
                        let conf_path = gen_path.join(format!("{}.conf", name));
                        fs::write(&conf_path, format!("# Config for {}\n", name)).unwrap();
                        built_count += 1;
                    }
                }
            }
        }
    }

    // 5. 原子切换：更新 current 软链接/记录
    let current_link = store_path.join("current");
    fs::write(&current_link, &gen_id).expect("Failed to update current generation");
    
    println!("\n[A.R.C.A.E.A. SYSTEM] rebuild 完成。");
    println!("[SYS] 成功构建 {} 个组件。", built_count);
    println!("[SYS] 系统世代已切换至: {}", gen_id);
    println!("[SYS] 运行 'arc-cli rollback' 可恢复至上一状态。");
}

fn handle_rollback() {
    println!("\n[A.R.C.A.E.A. SYSTEM] 启动单回滚机制...");
    let store_path = Path::new("./arcaea_store");
    let current_link = store_path.join("current");
    
    if !current_link.exists() {
        println!("❌ 找不到当前世代记录。");
        return;
    }
    
    let current_gen = fs::read_to_string(&current_link).unwrap();
    println!("[SYS] 当前世代: {}", current_gen);
    
    // 简单的回滚逻辑：删除 current 记录，模拟回退
    // (在真实的 arcaeaOS 中，这里会读取 generation-history 链表)
    fs::remove_file(&current_link).unwrap();
    println!("✅ 已断开当前世代链接。系统已回滚至基础底座 (Base Generation)。");
}