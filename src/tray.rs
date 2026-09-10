use crate::core::{AwakeManager, AwakeMode};
use muda::{Menu, MenuItem, PredefinedMenuItem};
use std::sync::{Arc, Mutex};
use tray_icon::menu::MenuEvent;
use tray_icon::{TrayIcon, TrayIconBuilder};

pub fn create_tray_icon(
    manager: Arc<Mutex<AwakeManager>>,
) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let menu = Menu::new();

    let passive_item = MenuItem::new("被动模式（禁用）", true, None);
    let indefinite_item = MenuItem::new("无限期", true, None);
    let timed_item = MenuItem::new("定时", true, None);
    let expirable_item = MenuItem::new("过期", true, None);

    let separator1 = PredefinedMenuItem::separator();
    let display_on_item = MenuItem::new("保持屏幕常亮", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let exit_item = MenuItem::new("退出", true, None);

    menu.append(&passive_item)?;
    menu.append(&indefinite_item)?;
    menu.append(&timed_item)?;
    menu.append(&expirable_item)?;
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
    let timed_id = timed_item.id().clone();
    let expirable_id = expirable_item.id().clone();
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
                let _ = mgr.apply();
                log::info!("模式已切换为无限期");
            } else if event.id == timed_id {
                mgr.set_mode(AwakeMode::Timed);
                let _ = mgr.apply();
                log::info!("模式已切换为定时");
            } else if event.id == expirable_id {
                mgr.set_mode(AwakeMode::Expirable);
                let _ = mgr.apply();
                log::info!("模式已切换为过期");
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