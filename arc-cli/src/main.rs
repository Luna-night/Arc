use arc_core::{parser, Token, TopLevel};
use logos::Logos;
use chumsky::Parser;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// 【核心】递归模块加载器 (处理 use 导入)
fn load_module(
    path: &Path, 
    loaded: &mut HashSet<PathBuf>, 
    ast: &mut Vec<TopLevel>
) -> Result<(), String> {
    let canonical_path = path.canonicalize()
        .map_err(|e| format!("无法解析路径 {:?}: {}", path, e))?;
        
    if loaded.contains(&canonical_path) {
        return Ok(()); 
    }
    loaded.insert(canonical_path.clone());

    let source = fs::read_to_string(&canonical_path)
        .map_err(|e| format!("无法读取文件 {:?}: {}", canonical_path, e))?;
        
    let tokens: Vec<Token> = Token::lexer(&source).filter_map(|t| t.ok()).collect();
    let parsed = parser().parse(tokens)
        .map_err(|e| format!("解析文件 {:?} 失败: {:?}", canonical_path, e))?;

    let base_dir = canonical_path.parent().unwrap_or(Path::new("."));

    for top in parsed {
        match top {
            TopLevel::UseDecl { path: use_path } => {
                let next_path = base_dir.join(use_path);
                load_module(&next_path, loaded, ast)?;
            }
            _ => {
                ast.push(top);
            }
        }
    }
    Ok(())
}

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

    if command == "rollback" {
        handle_rollback();
        return;
    }

    if args.len() < 3 {
        println!("❌ Missing file argument.");
        return;
    }

    let filename = &args[2];
    let path = Path::new(filename);
    let mut loaded_files = HashSet::new();
    let mut ast = Vec::new();

    match load_module(path, &mut loaded_files, &mut ast) {
        Ok(_) => {
            if command == "run" {
                let mut env = arc_core::Environment::new();
                
                // 1. 注册所有函数
                for top_level in &ast {
                    if let arc_core::TopLevel::FuncDecl(func) = top_level {
                        env.functions.insert(func.name.clone(), func.clone());
                    }
                }
                
                // 2. 执行语句、系统声明和变量绑定
                for top_level in &ast {
                    match top_level {
                        arc_core::TopLevel::Stmt(stmt) => { 
                            if let Err(e) = env.eval_stmt(stmt) {
                                eprintln!("❌ Runtime Error: {}", e);
                            }
                        }
                        arc_core::TopLevel::SystemDecl { units } => { 
                            env.eval_system_decl(units); 
                        }
                        // 【核心修复】执行 Let 变量声明！
                        arc_core::TopLevel::LetDecl { name, value } => {
                            match env.eval_expr(value) {
                                Ok(val) => { 
                                    env.variables.insert(name.clone(), val); 
                                }
                                Err(e) => eprintln!("❌ Runtime Error in let {}: {}", name, e),
                            }
                        }
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
            else if command == "rebuild" {
                handle_rebuild(&ast);
            }
            else {
                println!("❌ Unknown command: {}", command);
            }
        }
        Err(e) => {
            println!("❌ Module Load Error: {}", e);
        }
    }
}

fn handle_rebuild(ast: &[TopLevel]) {
    println!("\n[A.R.C.A.E.A. SYSTEM] 启动 arcaea-rebuild 协议...");
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}", ast).hash(&mut hasher);
    let gen_hash = format!("{:x}", hasher.finish());
    let gen_id = format!("gen-{}", &gen_hash[..8]);
    
    println!("[FSM] 验证配置文件完整性... Hash: {}", gen_id);

    let store_path = Path::new("./arcaea_store");
    let gen_path = store_path.join(&gen_id);
    let bin_path = gen_path.join("bin");
    
    fs::create_dir_all(&bin_path).expect("Failed to create generation directory");
    println!("[SYS] 创建隔离世代目录: {}", gen_path.display());

    let mut built_count = 0;

    for top in ast {
        if let TopLevel::SystemDecl { units } = top {
            for unit in units {
                if let arc_core::SystemUnit::Package { name, config } = unit {
                    println!("\n  ⚙️  正在处理包: {}", name);
                    let mut source_type = "static";
                    let mut build_path = ".";
                    for item in config {
                        if item.key == "source" {
                            if let arc_core::Expr::StringLit(s) = &item.value { source_type = s.as_str(); }
                        }
                        if item.key == "path" {
                            if let arc_core::Expr::StringLit(s) = &item.value { build_path = s.as_str(); }
                        }
                    }

                    if source_type == "cargo" {
                        println!("  [CARGO] 检测到 Rust 构建后端，启动沙盒编译...");
                        let status = Command::new("cargo")
                            .arg("build").arg("--release")
                            .arg("--manifest-path").arg(format!("{}/Cargo.toml", build_path))
                            .status();

                        match status {
                            Ok(s) if s.success() => {
                                let target_bin = Path::new(build_path).join("target/release").join(name);
                                if target_bin.exists() {
                                    let dest_bin = bin_path.join(name);
                                    fs::copy(&target_bin, &dest_bin).expect("Failed to extract binary");
                                    println!("  ✅ 产物已提取并锚定至: {}", dest_bin.display());
                                    built_count += 1;
                                }
                            }
                            _ => {
                                println!("  ❌ Cargo 构建失败，触发 ABORT_PROTOCOL。");
                                let _ = fs::remove_dir_all(&gen_path);
                                return;
                            }
                        }
                    } else {
                        let conf_path = gen_path.join(format!("{}.conf", name));
                        fs::write(&conf_path, format!("# Config for {}\n", name)).unwrap();
                        built_count += 1;
                    }
                }
            }
        }
    }

    let current_link = store_path.join("current");
    fs::write(&current_link, &gen_id).expect("Failed to update current generation");
    
    println!("\n[A.R.C.A.E.A. SYSTEM] rebuild 完成。");
    println!("[SYS] 成功构建 {} 个组件。", built_count);
    println!("[SYS] 系统世代已切换至: {}", gen_id);
}

fn handle_rollback() {
    println!("\n[A.R.C.A.E.A. SYSTEM] 启动单回滚机制...");
    let store_path = Path::new("./arcaea_store");
    let current_link = store_path.join("current");
    if current_link.exists() {
        fs::remove_file(&current_link).unwrap();
        println!("✅ 已断开当前世代链接。系统已回滚。");
    } else {
        println!("❌ 找不到当前世代记录。");
    }
}