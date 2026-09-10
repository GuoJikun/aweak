#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod config;
mod core;
mod crash;
mod process;
mod tray;

use chrono::{DateTime, Local};
use clap::Parser;
use cli::Cli;
use config::Config;
use core::{AwakeManager, AwakeMode};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn parse_datetime(s: &str) -> Option<SystemTime> {
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        let ts = dt.timestamp() as u64;
        return Some(UNIX_EPOCH + Duration::from_secs(ts));
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        let ts = dt.timestamp() as u64;
        return Some(UNIX_EPOCH + Duration::from_secs(ts));
    }
    if let Ok(naive) = chrono::NaiveTime::parse_from_str(s, "%H:%M:%S") {
        let now = Local::now();
        let today = now.date_naive();
        let naive_dt = today.and_time(naive);
        let local_dt: DateTime<Local> = naive_dt.and_local_timezone(Local).unwrap();
        let ts = local_dt.timestamp() as u64;
        return Some(UNIX_EPOCH + Duration::from_secs(ts));
    }
    None
}

fn run_message_loop() {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, TranslateMessage, DispatchMessageW, MSG};

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(not(debug_assertions))]
fn attach_console_if_needed() {}

#[cfg(not(debug_assertions))]
fn show_error_box(msg: &str) {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};
        let hmsg = windows::core::HSTRING::from(msg);
        let htitle = windows::core::HSTRING::from("aweak");
        MessageBoxW(None, &hmsg, &htitle, MB_OK | MB_ICONERROR);
    }
}

#[cfg(debug_assertions)]
fn attach_console_if_needed() {}

fn init_logger() {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
    let log_dir = exe_dir.join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let log_file = chrono::Local::now().format("%Y-%m-%d.log").to_string();
    let log_path = log_dir.join(log_file);

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_path).ok().unwrap_or_else(|| {
            fern::log_file(exe_dir.join("fallback.log")).expect("无法创建日志文件")
        }))
        .apply()
        .ok();
}

fn main() {
    init_logger();
    crash::install();

    // 单例模式：使用命名互斥锁确保只有一个实例运行
    let mutex_name = windows::core::HSTRING::from("Global\\aweak_single_instance");
    let _mutex_guard = unsafe {
        windows::Win32::System::Threading::CreateMutexW(
            None,
            true,
            &mutex_name,
        )
    };

    // 检查是否已有实例在运行（互斥锁已存在）
    let already_exists = unsafe {
        windows::Win32::Foundation::GetLastError().0 == 183 // ERROR_ALREADY_EXISTS
    };
    if already_exists {
        #[cfg(not(debug_assertions))]
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONINFORMATION};
            let msg = windows::core::HSTRING::from("aweak 已经在运行中！");
            let title = windows::core::HSTRING::from("aweak");
            MessageBoxW(None, &msg, &title, MB_OK | MB_ICONINFORMATION);
        }
        #[cfg(debug_assertions)]
        eprintln!("aweak 已经在运行中！");
        std::process::exit(0);
    }

    log::info!("========================================");
    log::info!("aweak 已启动");
    log::info!("========================================");

    let cli = Cli::parse();

    // 检测到命令行参数时，附加到父进程控制台以便输出错误信息
    if cli.display_on || cli.time_limit.is_some() || cli.expire_at.is_some() || cli.use_pt_config.is_some() || cli.pid.is_some() || cli.use_parent_pid {
        attach_console_if_needed();
    }

    let mut manager = AwakeManager::new();

    if let Some(pid) = cli.pid {
        log::info!("等待进程 {} 退出", pid);
        manager.set_keep_display_on(cli.display_on);
        manager.set_mode(AwakeMode::Indefinite);
        let _ = manager.apply();

        let manager = Arc::new(Mutex::new(manager));
        let _tray_icon = tray::create_tray_icon(manager.clone(), manager.lock().unwrap().get_mode(), manager.lock().unwrap().is_keep_display_on()).unwrap();

        let manager_clone = manager.clone();
        std::thread::spawn(move || {
            process::wait_for_process_exit(pid, Duration::from_secs(1));
            let mgr = manager_clone.lock().unwrap();
            let _ = mgr.release();
            log::info!("进程已退出，正在清理...");
            std::process::exit(0);
        });

        run_message_loop();
        return;
    }

    if cli.use_parent_pid {
        if let Some(parent_pid) = process::get_parent_pid() {
            log::info!("等待父进程 {} 退出", parent_pid);
            manager.set_keep_display_on(cli.display_on);
            manager.set_mode(AwakeMode::Indefinite);
            let _ = manager.apply();

            let mode = manager.get_mode();
            let keep_display = manager.is_keep_display_on();
            let manager = Arc::new(Mutex::new(manager));
            let _tray_icon = tray::create_tray_icon(manager.clone(), mode, keep_display).unwrap();

            let manager_clone = manager.clone();
            std::thread::spawn(move || {
                process::wait_for_process_exit(parent_pid, Duration::from_secs(1));
                let mgr = manager_clone.lock().unwrap();
                let _ = mgr.release();
                log::info!("父进程已退出，正在清理...");
                std::process::exit(0);
            });

            run_message_loop();
            return;
        }
    }

    // 有命令行参数时使用命令行参数，否则默认屏幕常亮
    let has_cli_options = cli.display_on || cli.time_limit.is_some() || cli.expire_at.is_some();

    if has_cli_options {
        manager.set_keep_display_on(cli.display_on);

        if let Some(time_limit) = cli.time_limit {
            manager.set_mode(AwakeMode::Timed);
            manager.set_time_limit(time_limit);
        } else if let Some(ref expire_at_str) = cli.expire_at {
            if let Some(expire_time) = parse_datetime(expire_at_str) {
                manager.set_mode(AwakeMode::Expirable);
                manager.set_expire_at(expire_time);
            }
        } else {
            manager.set_mode(AwakeMode::Indefinite);
        }
    } else if cli.use_pt_config.is_some() {
        // --use-pt-config 加载配置文件
        let config_path = cli.use_pt_config.as_ref().and_then(|p| p.as_deref());
        match Config::load(config_path) {
            Ok(config) => {
                manager.set_mode(Config::mode_from_u8(config.properties.mode));
                manager.set_keep_display_on(config.properties.keep_display_on);

                if config.properties.mode == 2 {
                    let total_seconds = config.properties.interval_hours * 3600
                        + config.properties.interval_minutes * 60;
                    manager.set_time_limit(total_seconds as u64);
                }

                if let Some(ref expire_str) = config.properties.expiration_datetime {
                    if let Some(expire_time) = parse_datetime(expire_str) {
                        manager.set_expire_at(expire_time);
                    }
                }
            }
            Err(e) => {
                log::error!("配置文件错误: {}", e);
                #[cfg(not(debug_assertions))]
                show_error_box(&format!("错误: {}", e));
                #[cfg(debug_assertions)]
                eprintln!("错误: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // 默认屏幕常亮模式
        manager.set_mode(AwakeMode::Indefinite);
        manager.set_keep_display_on(true);
    }

    let _ = manager.apply();

    let mode = manager.get_mode();
    let keep_display = manager.is_keep_display_on();

    if mode == AwakeMode::Timed {
        if let Some(seconds) = manager.get_time_limit() {
            let manager = Arc::new(Mutex::new(manager));
            let _tray_icon = tray::create_tray_icon(manager.clone(), mode, keep_display).unwrap();

            let manager_clone = manager.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(seconds));
                let mgr = manager_clone.lock().unwrap();
                let _ = mgr.release();
                log::info!("定时模式已过期");
                std::process::exit(0);
            });

            run_message_loop();
            return;
        }
    } else if mode == AwakeMode::Expirable {
        if let Some(expire_at) = manager.get_expire_at() {
            let manager = Arc::new(Mutex::new(manager));
            let _tray_icon = tray::create_tray_icon(manager.clone(), mode, keep_display).unwrap();

            let manager_clone = manager.clone();
            std::thread::spawn(move || {
                if let Ok(duration) = expire_at.duration_since(SystemTime::now()) {
                    std::thread::sleep(duration);
                }
                let mgr = manager_clone.lock().unwrap();
                let _ = mgr.release();
                log::info!("过期模式已过期");
                std::process::exit(0);
            });

            run_message_loop();
            return;
        }
    }

    log::info!("系统运行中，右键托盘图标可操作");

    let manager = Arc::new(Mutex::new(manager));
    let _tray_icon = tray::create_tray_icon(manager.clone(), mode, keep_display).unwrap();

    run_message_loop();

    log::info!("程序退出");
}