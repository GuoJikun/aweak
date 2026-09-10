# AGENTS

## 规则

- 全程使用中文
- 代码注释使用中文
- Git 提交信息使用中文
- 与用户沟通使用中文

## 项目信息

- 项目名称：aweak
- 编程语言：Rust
- 目标平台：Windows
- 功能：防止系统休眠

## 技术栈

- windows crate：调用 Windows API
- tray-icon：系统托盘
- muda：菜单
- clap：命令行解析
- chrono：时间处理
- serde/serde_json：配置文件

## 常用命令

```bash
# 编译
cargo build --release

# 运行
cargo run --release

# 检查
cargo check
```
