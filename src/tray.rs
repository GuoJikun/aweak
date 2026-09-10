use crate::core::{AwakeManager, AwakeMode};
use chrono::{DateTime, Local};
use muda::{CheckMenuItem, Menu, MenuEvent, PredefinedMenuItem, Submenu};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tray_icon::{TrayIcon, TrayIconBuilder};

struct SendSyncWrapper<T>(T);
unsafe impl<T> Send for SendSyncWrapper<T> {}
unsafe impl<T> Sync for SendSyncWrapper<T> {}

static PASSIVE_ITEM: OnceLock<SendSyncWrapper<CheckMenuItem>> = OnceLock::new();
static INDEFINITE_ITEM: OnceLock<SendSyncWrapper<CheckMenuItem>> = OnceLock::new();
static DISPLAY_ON_ITEM: OnceLock<SendSyncWrapper<CheckMenuItem>> = OnceLock::new();
static TRAY_ICON_REF: OnceLock<SendSyncWrapper<TrayIcon>> = OnceLock::new();

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

fn update_menu_checked(current_mode: AwakeMode, keep_display_on: bool) {
    if let Some(item) = PASSIVE_ITEM.get() {
        item.0.set_checked(current_mode == AwakeMode::Passive);
    }
    if let Some(item) = INDEFINITE_ITEM.get() {
        item.0.set_checked(current_mode == AwakeMode::Indefinite);
    }
    if let Some(item) = DISPLAY_ON_ITEM.get() {
        item.0.set_checked(keep_display_on);
    }
}

pub fn create_tray_icon(
    manager: Arc<Mutex<AwakeManager>>,
) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let menu = Menu::new();

    let passive_item = CheckMenuItem::with_id("passive", "被动模式（禁用）", true, true, None);
    let indefinite_item = CheckMenuItem::with_id("indefinite", "无限期", true, false, None);

    let timed_menu = Submenu::new("定时", true);
    let timed_30min = muda::MenuItem::with_id("timed_30min", "30 分钟", true, None);
    let timed_1hour = muda::MenuItem::with_id("timed_1hour", "1 小时", true, None);
    let timed_2hours = muda::MenuItem::with_id("timed_2hours", "2 小时", true, None);
    let timed_4hours = muda::MenuItem::with_id("timed_4hours", "4 小时", true, None);
    let timed_8hours = muda::MenuItem::with_id("timed_8hours", "8 小时", true, None);
    timed_menu.append(&timed_30min)?;
    timed_menu.append(&timed_1hour)?;
    timed_menu.append(&timed_2hours)?;
    timed_menu.append(&timed_4hours)?;
    timed_menu.append(&timed_8hours)?;

    let expirable_menu = Submenu::new("过期", true);
    let expirable_22 = muda::MenuItem::with_id("expirable_22", "今晚 22:00", true, None);
    let expirable_23 = muda::MenuItem::with_id("expirable_23", "今晚 23:00", true, None);
    let expirable_00 = muda::MenuItem::with_id("expirable_00", "今晚 24:00", true, None);
    let expirable_08 = muda::MenuItem::with_id("expirable_08", "明天 08:00", true, None);
    let expirable_12 = muda::MenuItem::with_id("expirable_12", "明天 12:00", true, None);
    expirable_menu.append(&expirable_22)?;
    expirable_menu.append(&expirable_23)?;
    expirable_menu.append(&expirable_00)?;
    expirable_menu.append(&expirable_08)?;
    expirable_menu.append(&expirable_12)?;

    let separator1 = PredefinedMenuItem::separator();
    let display_on_item = CheckMenuItem::with_id("display_on", "保持屏幕常亮", true, false, None);
    let separator2 = PredefinedMenuItem::separator();
    let exit_item = muda::MenuItem::with_id("exit", "退出", true, None);

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
        .with_tooltip("aweak - 被动模式")
        .with_icon(icon)
        .build()?;

    let _ = PASSIVE_ITEM.set(SendSyncWrapper(passive_item));
    let _ = INDEFINITE_ITEM.set(SendSyncWrapper(indefinite_item));
    let _ = DISPLAY_ON_ITEM.set(SendSyncWrapper(display_on_item));
    let _ = TRAY_ICON_REF.set(SendSyncWrapper(tray_icon.clone()));

    MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        let mut mgr = manager.lock().unwrap();

        let id_str = &event.id.0;

        if id_str == "passive" {
            mgr.set_mode(AwakeMode::Passive);
            let _ = mgr.release();
            log::info!("模式已切换为被动模式");
        } else if id_str == "indefinite" {
            mgr.set_mode(AwakeMode::Indefinite);
            mgr.set_time_limit(0);
            let _ = mgr.apply();
            log::info!("模式已切换为无限期");
        } else if id_str == "timed_30min" {
            mgr.set_mode(AwakeMode::Timed);
            mgr.set_time_limit(30 * 60);
            let _ = mgr.apply();
            log::info!("模式已切换为定时 30 分钟");
        } else if id_str == "timed_1hour" {
            mgr.set_mode(AwakeMode::Timed);
            mgr.set_time_limit(60 * 60);
            let _ = mgr.apply();
            log::info!("模式已切换为定时 1 小时");
        } else if id_str == "timed_2hours" {
            mgr.set_mode(AwakeMode::Timed);
            mgr.set_time_limit(2 * 60 * 60);
            let _ = mgr.apply();
            log::info!("模式已切换为定时 2 小时");
        } else if id_str == "timed_4hours" {
            mgr.set_mode(AwakeMode::Timed);
            mgr.set_time_limit(4 * 60 * 60);
            let _ = mgr.apply();
            log::info!("模式已切换为定时 4 小时");
        } else if id_str == "timed_8hours" {
            mgr.set_mode(AwakeMode::Timed);
            mgr.set_time_limit(8 * 60 * 60);
            let _ = mgr.apply();
            log::info!("模式已切换为定时 8 小时");
        } else if id_str == "expirable_22" {
            mgr.set_mode(AwakeMode::Expirable);
            if let Some(t) = parse_expire_time(22, 0) {
                mgr.set_expire_at(t);
            }
            let _ = mgr.apply();
            log::info!("模式已切换为过期 今晚 22:00");
        } else if id_str == "expirable_23" {
            mgr.set_mode(AwakeMode::Expirable);
            if let Some(t) = parse_expire_time(23, 0) {
                mgr.set_expire_at(t);
            }
            let _ = mgr.apply();
            log::info!("模式已切换为过期 今晚 23:00");
        } else if id_str == "expirable_00" {
            mgr.set_mode(AwakeMode::Expirable);
            if let Some(t) = parse_expire_time(24, 0) {
                mgr.set_expire_at(t);
            }
            let _ = mgr.apply();
            log::info!("模式已切换为过期 今晚 24:00");
        } else if id_str == "expirable_08" {
            mgr.set_mode(AwakeMode::Expirable);
            if let Some(t) = parse_expire_time(32, 0) {
                mgr.set_expire_at(t);
            }
            let _ = mgr.apply();
            log::info!("模式已切换为过期 明天 08:00");
        } else if id_str == "expirable_12" {
            mgr.set_mode(AwakeMode::Expirable);
            if let Some(t) = parse_expire_time(36, 0) {
                mgr.set_expire_at(t);
            }
            let _ = mgr.apply();
            log::info!("模式已切换为过期 明天 12:00");
        } else if id_str == "display_on" {
            let current = mgr.is_keep_display_on();
            mgr.set_keep_display_on(!current);
            let _ = mgr.apply();
            log::info!("保持屏幕常亮: {}", !current);
        } else if id_str == "exit" {
            let _ = mgr.release();
            log::info!("正在退出...");
            std::process::exit(0);
        }

        let current_mode = mgr.get_mode();
        let keep_display = mgr.is_keep_display_on();
        drop(mgr);

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            update_menu_checked(current_mode, keep_display);

            let tooltip = match current_mode {
                AwakeMode::Passive => "aweak - 被动模式",
                AwakeMode::Indefinite => "aweak - 无限期",
                AwakeMode::Timed => "aweak - 定时",
                AwakeMode::Expirable => "aweak - 过期",
            };
            if let Some(tray) = TRAY_ICON_REF.get() {
                let _ = tray.0.set_tooltip(Some(tooltip));
            }
        });
    }));

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