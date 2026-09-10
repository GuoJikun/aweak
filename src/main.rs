#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod config;
mod core;
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

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let mut manager = AwakeManager::new();

    if let Some(pid) = cli.pid {
        log::info!("等待进程 {} 退出", pid);
        manager.set_keep_display_on(cli.display_on);
        manager.set_mode(AwakeMode::Indefinite);
        let _ = manager.apply();

        let manager = Arc::new(Mutex::new(manager));
        let _tray_icon = tray::create_tray_icon(manager.clone()).unwrap();

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

            let manager = Arc::new(Mutex::new(manager));
            let _tray_icon = tray::create_tray_icon(manager.clone()).unwrap();

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

    if cli.use_pt_config {
        let config = Config::load();
        manager.set_mode(Config::mode_from_u8(config.properties.mode));
        manager.set_keep_display_on(config.properties.keep_display_on);

        if config.properties.mode == 2 {
            let total_seconds =
                config.properties.interval_hours * 3600 + config.properties.interval_minutes * 60;
            manager.set_time_limit(total_seconds as u64);
        }

        if let Some(ref expire_str) = config.properties.expiration_datetime {
            if let Some(expire_time) = parse_datetime(expire_str) {
                manager.set_expire_at(expire_time);
            }
        }
    } else {
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
    }

    let _ = manager.apply();

    let mode = manager.get_mode();

    if mode == AwakeMode::Timed {
        if let Some(seconds) = manager.get_time_limit() {
            let manager = Arc::new(Mutex::new(manager));
            let _tray_icon = tray::create_tray_icon(manager.clone()).unwrap();

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
            let _tray_icon = tray::create_tray_icon(manager.clone()).unwrap();

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

    let manager = Arc::new(Mutex::new(manager));
    let _tray_icon = tray::create_tray_icon(manager.clone()).unwrap();

    run_message_loop();
}