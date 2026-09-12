use std::io;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW, HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, REG_OPTION_NON_VOLATILE,
    REG_SZ,
};

const AUTOSTART_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const AUTOSTART_VALUE_NAME: &str = "aweak";

fn get_exe_path() -> io::Result<String> {
    let path = std::env::current_exe()?;
    Ok(path.to_string_lossy().to_string())
}

pub fn is_autostart_enabled() -> bool {
    unsafe {
        let key_wide = windows::core::HSTRING::from(AUTOSTART_KEY);
        let name_wide = windows::core::HSTRING::from(AUTOSTART_VALUE_NAME);

        let mut hkey = std::mem::zeroed();
        let result = RegOpenKeyExW(HKEY_CURRENT_USER, &key_wide, None, KEY_READ, &mut hkey);

        if result.is_err() {
            return false;
        }

        let mut buf_len: u32 = 0;
        let query_result =
            RegQueryValueExW(hkey, &name_wide, None, None, None, Some(&mut buf_len));

        let _ = RegCloseKey(hkey);
        query_result.is_ok()
    }
}

pub fn enable_autostart() -> io::Result<()> {
    unsafe {
        let key_wide = windows::core::HSTRING::from(AUTOSTART_KEY);
        let name_wide = windows::core::HSTRING::from(AUTOSTART_VALUE_NAME);
        let exe_path = get_exe_path()?;
        let value_wide: Vec<u16> = exe_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = std::mem::zeroed();
        let result = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            &key_wide,
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            None,
            &mut hkey,
            None,
        );

        if result.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("无法创建注册表键: 错误代码 {:?}", result),
            ));
        }

        let set_result = RegSetValueExW(
            hkey,
            &name_wide,
            None,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                value_wide.as_ptr() as *const u8,
                value_wide.len() * 2,
            )),
        );

        let _ = RegCloseKey(hkey);

        if set_result.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("无法设置注册表值: 错误代码 {:?}", set_result),
            ));
        }

        log::info!("已启用开机自启动: {}", exe_path);
        Ok(())
    }
}

pub fn disable_autostart() -> io::Result<()> {
    unsafe {
        let key_wide = windows::core::HSTRING::from(AUTOSTART_KEY);
        let name_wide = windows::core::HSTRING::from(AUTOSTART_VALUE_NAME);

        let mut hkey = std::mem::zeroed();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &key_wide,
            None,
            KEY_ALL_ACCESS,
            &mut hkey,
        );

        if result.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("无法打开注册表键: 错误代码 {:?}", result),
            ));
        }

        let del_result = RegDeleteValueW(hkey, &name_wide);
        let _ = RegCloseKey(hkey);

        if del_result.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("无法删除注册表值: 错误代码 {:?}", del_result),
            ));
        }

        log::info!("已禁用开机自启动");
        Ok(())
    }
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
