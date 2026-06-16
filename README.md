# 🌟 Arc Programming Language

**Arc: Bridge the gap between performance and productivity.**

Arc 是一门全新的、追求极致开发者体验的全能编程语言。它结合了 Rust 的内存安全、函数式编程的优雅，以及革命性的跨语言互操作性（Arc Bridge）。

## 🚀 核心特性

- **智能内存管理**：基于自适应 ARC (Automatic Reference Counting)，兼顾安全与性能。
- **函数式一等公民**：原生支持模式匹配、管道操作符 (`|>`) 和代数数据类型。
- **Arc Bridge (杀手锏)**：像 Wine 一样，允许在 Linux 上直接调用 Windows API，或无缝桥接 Python/Node.js 生态。
- **开箱即用的工具链**：内置 LSP 语言服务器，在 VS Code 中提供完美的智能提示和实时报错。
- **AOT 原生编译**：基于 LLVM IR 和 Clang，生成高度优化的原生机器码。

## 🛠️ 快速开始

### 1. 编译 Arc 工具链
确保你已安装 Rust 和 Node.js。
```bash
cargo build --workspace
cd arc-language && npm install
```

### 2. 运行你的第一段 Arc 代码
```bash
echo 'print(42)' > test.arc
cargo run --bin arc-cli -- run test.arc
```

### 3. 体验 AOT 原生编译
```bash
cargo run --bin arc-cli -- build test.arc
./arc_app
```

## 📂 项目结构

- `arc-core`: 语言核心（词法分析、语法解析、AST、解释器、LLVM IR 代码生成）。
- `arc-cli`: 命令行工具（支持 `run` 解释执行和 `build` AOT 编译）。
- `arc-lsp`: 语言服务器（为 VS Code 提供实时语法检查和智能提示）。
- `arc-language`: VS Code 插件外壳（TypeScript）。

## 🗺️ 路线图 (Roadmap)

- [x] 核心词法与语法分析 (Lexer & Parser)
- [x] VS Code LSP 智能提示集成
- [x] 基础解释器 (Interpreter)
- [x] AOT 原生编译器 (LLVM IR + Clang)
- [ ] 变量声明与数学运算
- [ ] 函数定义与闭包支持
- [ ] 管道操作符 (`|>`) 实现
- [ ] **Arc Bridge**: 跨语言/跨平台 FFI 调用

## 🤝 贡献

Arc 目前处于早期开发阶段 (v0.1)。欢迎提出 Issue 或 Pull Request！

---
*Made with ❤️ and Rust.*
```