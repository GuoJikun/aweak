use std::sync::OnceLock;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{
    RegisterPowerSettingNotification, POWERBROADCAST_SETTING,
};
use windows::Win32::System::SystemServices::{
    GUID_CONSOLE_DISPLAY_STATE, GUID_MONITOR_POWER_ON, GUID_SESSION_DISPLAY_STATUS,
};
use windows::Win32::UI::WindowsAndMessaging::*;

struct SendHWND(HWND);
unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

static POWER_HWND: OnceLock<SendHWND> = OnceLock::new();

unsafe extern "system" fn power_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_POWERBROADCAST {
        match wparam.0 {
            32 => {
                // PBT_POWERSETTINGCHANGE
                let pb = unsafe { &*(lparam.0 as *const POWERBROADCAST_SETTING) };
                if pb.PowerSetting == GUID_CONSOLE_DISPLAY_STATE
                    || pb.PowerSetting == GUID_SESSION_DISPLAY_STATUS
                    || pb.PowerSetting == GUID_MONITOR_POWER_ON
                {
                    let state = unsafe { *(pb.Data.as_ptr() as *const u32) };
                    match state {
                        0 => log::info!("[电源事件] 显示器已关闭"),
                        1 => log::info!("[电源事件] 显示器已打开"),
                        _ => log::info!("[电源事件] 显示器状态: {}", state),
                    }
                }
            }
            18 => log::info!("[电源事件] 系统从休眠中唤醒"),
            7 => log::info!("[电源事件] 系统自动恢复"),
            4 => log::info!("[电源事件] 系统即将进入休眠"),
            _ => {}
        }
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn start_power_monitor() {
    std::thread::spawn(|| unsafe {
        let class_name = w!("AweakPowerMonitor");
        let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default().into();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(power_wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..std::mem::zeroed()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Aweak Power Monitor"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hinstance),
            None,
        )
        .unwrap_or_default();

        let _ = POWER_HWND.set(SendHWND(hwnd));

        for guid in [
            GUID_CONSOLE_DISPLAY_STATE,
            GUID_SESSION_DISPLAY_STATUS,
            GUID_MONITOR_POWER_ON,
        ] {
            let _ = RegisterPowerSettingNotification(
                HANDLE(hwnd.0),
                &guid,
                DEVICE_NOTIFY_WINDOW_HANDLE,
            );
        }

        log::info!("电源监控已启动");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}
