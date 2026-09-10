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
- 单例模式，确保只有一个实例运行
- 支持配置文件

## 安装

```bash
cargo build --release
```

## 使用方法

```bash
# 默认：无限期保持唤醒 + 屏幕常亮
aweak

# 保持屏幕常亮
aweak --display-on

# 定时 1 小时（单位：秒）
aweak --time-limit 3600

# 到指定时间点过期
aweak --expire-at "2026-09-10 22:00"

# 绑定到进程，进程退出时自动停止
aweak --pid 1234

# 使用配置文件
aweak --use-pt-config

# 使用指定配置文件
aweak --use-pt-config D:\path\settings.json
```

## 系统托盘

程序启动后会在系统托盘显示图标，右键可切换：

- 被动模式（禁用）
- 无限期
- 定时（30分钟/1小时/2小时/4小时/8小时）
- 过期（今晚/明天指定时间）
- 保持屏幕常亮
- 退出

## 配置文件

配置文件默认位于 exe 同级目录的 `settings.json`：

```json
{
  "properties": {
    "keep_display_on": false,
    "mode": 0,
    "interval_hours": 0,
    "interval_minutes": 0,
    "expiration_datetime": null,
    "custom_tray_times": {}
  },
  "name": "Awake",
  "version": "1.0"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `mode` | u8 | 0=被动, 1=无限期, 2=定时, 3=过期 |
| `keep_display_on` | bool | 是否保持屏幕常亮 |
| `interval_hours` | u32 | 定时模式-小时数 |
| `interval_minutes` | u32 | 定时模式-分钟数 |
| `expiration_datetime` | string | 过期时间，格式 `YYYY-MM-DD HH:MM:SS` |

## 命令行参数

| 参数 | 说明 |
|------|------|
| `--display-on` | 保持屏幕常亮 |
| `--time-limit <秒>` | 定时时长 |
| `--expire-at <时间>` | 过期时间，格式：`YYYY-MM-DD HH:MM:SS` |
| `--pid <PID>` | 绑定到指定进程 |
| `--use-parent-pid` | 绑定到父进程 |
| `--use-pt-config [路径]` | 使用配置文件（默认 exe 同级） |

## 许可证

MIT
