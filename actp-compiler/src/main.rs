// ==========================================
// ACTP 編譯器原型 v0.1.1 (State-Machine Parser Patch)
// ==========================================

#[derive(Debug, Clone, PartialEq)]
enum Channel {
    Stable,
    Nightly,
}

#[derive(Debug, Clone)]
struct ArcaeaComponent {
    name: String,
    channel: Channel,
    feature_flag: Option<String>,
    code_body: String,
}

#[derive(Debug)]
struct EnvironmentConfig {
    status: String,
    enabled_features: Vec<String>,
}

// ==========================================
// 1. 狀態機解析器 (State-Machine Parser)
// ==========================================
fn parse_arcaea_source(source: &str) -> Vec<ArcaeaComponent> {
    let mut components = Vec::new();
    
    // 狀態機變量：用於在掃描過程中「記住」當前的屬性
    let mut current_name = String::new();
    let mut current_channel = Channel::Stable;
    let mut current_feature: Option<String> = None;
    let mut current_body = String::new();
    
    let mut in_body = false;
    let mut brace_count = 0; // 用於處理嵌套大括號

    for line in source.lines() {
        let trimmed = line.trim();
        
                if in_body {
            // 【核心修復】先更新大括號計數，再決定是否收集！
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();
            
            // 當大括號閉合為 0 時，模組結束
            if brace_count == 0 {
                in_body = false;
                if !current_name.is_empty() {
                    components.push(ArcaeaComponent {
                        name: current_name.clone(),
                        channel: current_channel.clone(),
                        feature_flag: current_feature.clone(),
                        code_body: current_body.clone(),
                    });
                }
                // 重置狀態
                current_name.clear();
                current_body.clear();
                current_feature = None;
                
                // 【關鍵】攔截並丟棄最後的 '}'，不讓它進入 body！
                continue; 
            }

            // 只有還沒結束時，才將當前行加入 body
            current_body.push_str(line);
            current_body.push('\n');
            continue;
        }

        // 如果不在主體內，則解析屬性標記
        if trimmed.starts_with("#[component(name = \"") {
            current_name = trimmed.split('"').nth(1).unwrap_or("").to_string();
        } else if trimmed == "#[channel = \"Stable\"]" {
            current_channel = Channel::Stable;
        } else if trimmed == "#[channel = \"Nightly\"]" {
            current_channel = Channel::Nightly;
        } else if trimmed.starts_with("#[feature(flag = \"") {
            current_feature = Some(trimmed.split('"').nth(1).unwrap_or("").to_string());
        } else if trimmed.starts_with("module ") && trimmed.ends_with('{') {
            // 遇到 module 關鍵字，開始收集主體
            in_body = true;
            brace_count = 1; // 已經包含了一個 '{'
                  } else if trimmed.starts_with("module ") && trimmed.ends_with('{') {
            // 遇到 module 關鍵字，開始收集主體
            in_body = true;
            brace_count = 1; // 已經包含了一個 '{'
            // 【核心修復】剝離 Arcaea 的 module 語法外殼！
            // 我們不需要把 "module xxx {" 放進 body，因為 Codegen 會生成 "pub mod xxx {"
            // current_body.push_str(line); 
            // current_body.push('\n');
        }
    }
    
    components
}

// ==========================================
// 2. FSM 驗證與物理裁剪
// ==========================================
fn fsm_prune_components(
    components: Vec<ArcaeaComponent>,
    env: &EnvironmentConfig
) -> Vec<ArcaeaComponent> {
    let mut survivors = Vec::new();
    
    println!("[FSM] 開始驗證組件拓撲...");
    println!("[FSM] 環境狀態: {}", env.status);
    
    let is_wave_active = env.status == "DeepSpace_Wave_Active";

    for comp in components {
        let mut survive = true;
        let mut reason = String::new();

        if is_wave_active && comp.channel == Channel::Nightly {
            survive = false;
            reason = "FSM 協議：深空波動期強制剔除 Nightly 組件".to_string();
        }

        if let Some(flag) = &comp.feature_flag {
            if !env.enabled_features.contains(flag) {
                survive = false;
                reason = format!("特性開關 [{}] 未在環境配置中開啟", flag);
            }
        }

        if survive {
            println!("[FSM] ✅ 組件 [{}] 驗證通過，保留。", comp.name);
            survivors.push(comp);
        } else {
            println!("[FSM] ❌ 組件 [{}] 被物理裁剪。原因: {}", comp.name, reason);
        }
    }
    
    survivors
}

// ==========================================
// 3. 代碼生成 (Codegen)
// ==========================================
fn generate_rust_code(components: &[ArcaeaComponent]) -> String {
    let mut rust_code = String::new();
    rust_code.push_str("// ==========================================\n");
    rust_code.push_str("// 由 arcaea-rebuild (ACTP 協議) 自動生成\n");
    rust_code.push_str("// 警告：請勿手動修改此文件\n");
    rust_code.push_str("// ==========================================\n\n");
    
    for comp in components {
        rust_code.push_str(&format!("pub mod {} {{\n", comp.name));
        rust_code.push_str(&comp.code_body);
        rust_code.push_str("}\n\n");
    }
    
    rust_code
}

// ==========================================
// 主函數
// ==========================================
fn main() {
    let arcaea_source = r#"
        #[component(name = "lca_driver", version = "1.0.0")]
        #[channel = "Stable"]
        module lca_driver {
            // 【核心修復】使用靜態數組，確保在合法內存範圍內
            static mut TEST_BUFFER: [u8; 4096] = [0; 4096];
            
            pub fn init() {
                crate::sbi_println!("[LCA] Initializing Crystal Storage...");
                
                unsafe {
                    let ptr = TEST_BUFFER.as_mut_ptr();
                    
                    // 【測試 1】寫入
                    core::ptr::write_bytes(ptr, 0xAA, 4096);
                    crate::sbi_println!("[LCA] Data written: 0xAA pattern to static buffer");

                    // 【測試 2】讀取驗證
                    let first = core::ptr::read_volatile(ptr);
                    let last = core::ptr::read_volatile(ptr.add(4095));
                    
                    if first == 0xAA && last == 0xAA {
                        crate::sbi_println!("[LCA] Verification PASSED! Memory R/W OK.");
                    } else {
                        crate::sbi_println!("[LCA] Verification FAILED!");
                    }
                }
                
                crate::sbi_println!("[LCA] Crystal Storage Online.");
            }
        }

        #[component(name = "hal_riscv", version = "1.0.0")]
        #[channel = "Stable"]
        module hal_riscv {
            pub fn enter_hibernation() -> ! {
                loop { unsafe { core::arch::asm!("wfi"); } }
            }
        }
    "#;

    // 【測試指令】嘗試修改這裡的 status！
    // 改為 "Normal" 將釋放 Nightly 組件
    let env = EnvironmentConfig {
        status: "DeepSpace_Wave_Active".to_string(), 
        enabled_features: vec!["enable_rant_prediction".to_string()],
    };

    println!("========================================");
    println!("   ACTP COMPILER PROTOTYPE v0.1.1      ");
    println!("========================================");

    let parsed_components = parse_arcaea_source(arcaea_source);
    let surviving_components = fsm_prune_components(parsed_components, &env);
    let final_rust_code = generate_rust_code(&surviving_components);

    println!("\n========================================");
    println!("   生成的 Rust 代碼 (Generation)       ");
    println!("========================================\n");
    println!("{}", final_rust_code);
    // 【核心協議】將生成的代碼物理寫入 arcaea-os 的 src 目錄
    let output_path = "/home/luna/a/Arc/arcaea-os/src/generated.rs";
    std::fs::write(output_path, &final_rust_code).expect("[ACTP] 檔案寫入失敗！");
    println!("\n[ACTP] 已成功將世代代碼注入: {}", output_path);
}