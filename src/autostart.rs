use std::io;
use windows_registry::CURRENT_USER;

const AUTOSTART_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const AUTOSTART_VALUE_NAME: &str = "aweak";

fn get_exe_path() -> io::Result<String> {
    let path = std::env::current_exe()?;
    Ok(path.to_string_lossy().to_string())
}

pub fn is_autostart_enabled() -> bool {
    let key = match CURRENT_USER.open(AUTOSTART_KEY) {
        Ok(k) => k,
        Err(_) => return false,
    };
    match key.get_string(AUTOSTART_VALUE_NAME) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn enable_autostart() -> io::Result<()> {
    let key = CURRENT_USER
        .create(AUTOSTART_KEY)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let exe_path = get_exe_path()?;
    key.set_string(AUTOSTART_VALUE_NAME, &exe_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    log::info!("已启用开机自启动: {}", exe_path);
    Ok(())
}

pub fn disable_autostart() -> io::Result<()> {
    let key = CURRENT_USER
        .open(AUTOSTART_KEY)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    key.remove_value(AUTOSTART_VALUE_NAME)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    log::info!("已禁用开机自启动");
    Ok(())
}

pub fn toggle_autostart() -> io::Result<bool> {
    if is_autostart_enabled() {
        disable_autostart()?;
        Ok(false)
    } else {
        enable_autostart()?;
        Ok(true)
    }
}
