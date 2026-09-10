use crate::core::{AwakeManager, AwakeMode};
use chrono::{DateTime, Local};
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tray_icon::menu::MenuEvent;
use tray_icon::{TrayIcon, TrayIconBuilder};

fn parse_expire_time(hours: u32, minutes: u32) -> Option<SystemTime> {
    let now = Local::now();
    let today = now.date_naive();

    let (target_date, target_time) = if hours >= 24 {
        let tomorrow = today + chrono::Duration::days(1);
        let h = hours - 24;
        (tomorrow, chrono::NaiveTime::from_hms_opt(h, minutes, 0)?)
    } else {
        (today, chrono::NaiveTime::from_hms_opt(hours, minutes, 0)?)
    };

    let naive_dt = target_date.and_time(target_time);
    let local_dt: DateTime<Local> = naive_dt.and_local_timezone(Local).unwrap();
    let ts = local_dt.timestamp() as u64;
    Some(UNIX_EPOCH + Duration::from_secs(ts))
}

pub fn create_tray_icon(
    manager: Arc<Mutex<AwakeManager>>,
) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let menu = Menu::new();

    let passive_item = MenuItem::new("被动模式（禁用）", true, None);
    let indefinite_item = MenuItem::new("无限期", true, None);

    let timed_menu = Submenu::new("定时", true);
    let timed_30min = MenuItem::new("30 分钟", true, None);
    let timed_1hour = MenuItem::new("1 小时", true, None);
    let timed_2hours = MenuItem::new("2 小时", true, None);
    let timed_4hours = MenuItem::new("4 小时", true, None);
    let timed_8hours = MenuItem::new("8 小时", true, None);
    timed_menu.append(&timed_30min)?;
    timed_menu.append(&timed_1hour)?;
    timed_menu.append(&timed_2hours)?;
    timed_menu.append(&timed_4hours)?;
    timed_menu.append(&timed_8hours)?;

    let expirable_menu = Submenu::new("过期", true);
    let expirable_22 = MenuItem::new("今晚 22:00", true, None);
    let expirable_23 = MenuItem::new("今晚 23:00", true, None);
    let expirable_00 = MenuItem::new("今晚 24:00", true, None);
    let expirable_08 = MenuItem::new("明天 08:00", true, None);
    let expirable_12 = MenuItem::new("明天 12:00", true, None);
    expirable_menu.append(&expirable_22)?;
    expirable_menu.append(&expirable_23)?;
    expirable_menu.append(&expirable_00)?;
    expirable_menu.append(&expirable_08)?;
    expirable_menu.append(&expirable_12)?;

    let separator1 = PredefinedMenuItem::separator();
    let display_on_item = MenuItem::new("保持屏幕常亮", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let exit_item = MenuItem::new("退出", true, None);

    menu.append(&passive_item)?;
    menu.append(&indefinite_item)?;
    menu.append(&timed_menu)?;
    menu.append(&expirable_menu)?;
    menu.append(&separator1)?;
    menu.append(&display_on_item)?;
    menu.append(&separator2)?;
    menu.append(&exit_item)?;

    let icon = load_default_icon();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("aweak - 保持唤醒")
        .with_icon(icon)
        .build()?;

    let manager_clone = manager.clone();
    let event_receiver = MenuEvent::receiver();

    let passive_id = passive_item.id().clone();
    let indefinite_id = indefinite_item.id().clone();
    let timed_30min_id = timed_30min.id().clone();
    let timed_1hour_id = timed_1hour.id().clone();
    let timed_2hours_id = timed_2hours.id().clone();
    let timed_4hours_id = timed_4hours.id().clone();
    let timed_8hours_id = timed_8hours.id().clone();
    let expirable_22_id = expirable_22.id().clone();
    let expirable_23_id = expirable_23.id().clone();
    let expirable_00_id = expirable_00.id().clone();
    let expirable_08_id = expirable_08.id().clone();
    let expirable_12_id = expirable_12.id().clone();
    let display_on_id = display_on_item.id().clone();
    let exit_id = exit_item.id().clone();

    std::thread::spawn(move || loop {
        if let Ok(event) = event_receiver.recv() {
            let mut mgr = manager_clone.lock().unwrap();

            if event.id == passive_id {
                mgr.set_mode(AwakeMode::Passive);
                let _ = mgr.release();
                log::info!("模式已切换为被动模式");
            } else if event.id == indefinite_id {
                mgr.set_mode(AwakeMode::Indefinite);
                mgr.set_time_limit(0);
                let _ = mgr.apply();
                log::info!("模式已切换为无限期");
            } else if event.id == timed_30min_id {
                mgr.set_mode(AwakeMode::Timed);
                mgr.set_time_limit(30 * 60);
                let _ = mgr.apply();
                log::info!("模式已切换为定时 30 分钟");
            } else if event.id == timed_1hour_id {
                mgr.set_mode(AwakeMode::Timed);
                mgr.set_time_limit(60 * 60);
                let _ = mgr.apply();
                log::info!("模式已切换为定时 1 小时");
            } else if event.id == timed_2hours_id {
                mgr.set_mode(AwakeMode::Timed);
                mgr.set_time_limit(2 * 60 * 60);
                let _ = mgr.apply();
                log::info!("模式已切换为定时 2 小时");
            } else if event.id == timed_4hours_id {
                mgr.set_mode(AwakeMode::Timed);
                mgr.set_time_limit(4 * 60 * 60);
                let _ = mgr.apply();
                log::info!("模式已切换为定时 4 小时");
            } else if event.id == timed_8hours_id {
                mgr.set_mode(AwakeMode::Timed);
                mgr.set_time_limit(8 * 60 * 60);
                let _ = mgr.apply();
                log::info!("模式已切换为定时 8 小时");
            } else if event.id == expirable_22_id {
                mgr.set_mode(AwakeMode::Expirable);
                if let Some(t) = parse_expire_time(22, 0) {
                    mgr.set_expire_at(t);
                }
                let _ = mgr.apply();
                log::info!("模式已切换为过期 今晚 22:00");
            } else if event.id == expirable_23_id {
                mgr.set_mode(AwakeMode::Expirable);
                if let Some(t) = parse_expire_time(23, 0) {
                    mgr.set_expire_at(t);
                }
                let _ = mgr.apply();
                log::info!("模式已切换为过期 今晚 23:00");
            } else if event.id == expirable_00_id {
                mgr.set_mode(AwakeMode::Expirable);
                if let Some(t) = parse_expire_time(24, 0) {
                    mgr.set_expire_at(t);
                }
                let _ = mgr.apply();
                log::info!("模式已切换为过期 今晚 24:00");
            } else if event.id == expirable_08_id {
                mgr.set_mode(AwakeMode::Expirable);
                if let Some(t) = parse_expire_time(32, 0) {
                    mgr.set_expire_at(t);
                }
                let _ = mgr.apply();
                log::info!("模式已切换为过期 明天 08:00");
            } else if event.id == expirable_12_id {
                mgr.set_mode(AwakeMode::Expirable);
                if let Some(t) = parse_expire_time(36, 0) {
                    mgr.set_expire_at(t);
                }
                let _ = mgr.apply();
                log::info!("模式已切换为过期 明天 12:00");
            } else if event.id == display_on_id {
                let current = mgr.is_keep_display_on();
                mgr.set_keep_display_on(!current);
                let _ = mgr.apply();
                log::info!("保持屏幕常亮: {}", !current);
            } else if event.id == exit_id {
                let _ = mgr.release();
                log::info!("正在退出...");
                std::process::exit(0);
            }
        }
    });

    Ok(tray_icon)
}

fn load_default_icon() -> tray_icon::Icon {
    let size = 16;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let center_x = size as f32 / 2.0;
            let center_y = size as f32 / 2.0;
            let radius = size as f32 / 2.5;

            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < radius {
                rgba[idx] = 0;
                rgba[idx + 1] = 200;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, size as u32, size as u32).unwrap()
}