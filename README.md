# aweak

类似 PowerToys Awake 的 Windows 防休眠工具，使用 Rust 实现。

## 功能特性

- 使用 `SetThreadExecutionState` API 防止系统休眠，不修改电源设置
- 支持四种工作模式：
  - **被动模式**：程序运行但不执行防休眠操作
  - **无限期**：持续保持唤醒直到手动停止
  - **定时**：保持唤醒指定时长后自动恢复
  - **过期**：保持唤醒到指定时间点
- 支持屏幕常亮控制
- 系统托盘图标和右键菜单（中文界面）
- 支持绑定到指定进程，进程退出时自动停止

## 安装

```bash
cargo build --release
```

## 使用方法

```bash
# 无限期保持唤醒
aweak

# 保持屏幕常亮
aweak --display-on

# 定时 1 小时（单位：秒）
aweak --time-limit 3600

# 到指定时间点过期
aweak --expire-at "2026-09-10 22:00"

# 绑定到进程，进程退出时自动停止
aweak --pid 1234
```

## 系统托盘

程序启动后会在系统托盘显示图标，右键可切换：

- 被动模式（禁用）
- 无限期
- 定时
- 过期
- 保持屏幕常亮
- 退出

## 命令行参数

| 参数 | 说明 |
|------|------|
| `--display-on` | 保持屏幕常亮 |
| `--time-limit <秒>` | 定时时长 |
| `--expire-at <时间>` | 过期时间，格式：`YYYY-MM-DD HH:MM:SS` |
| `--pid <PID>` | 绑定到指定进程 |
| `--use-parent-pid` | 绑定到父进程 |
| `--use-pt-config` | 使用配置文件 |

## 依赖

- windows 0.62
- tray-icon 0.24
- muda 0.19
- clap 4
- chrono 0.4
- dirs 7
- serde / serde_json
- log / env_logger

## 许可证

MIT
