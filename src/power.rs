use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{
    ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, RegisterPowerSettingNotification,
    SetThreadExecutionState, EXECUTION_STATE, POWERBROADCAST_SETTING,
};
use windows::Win32::System::SystemServices::{
    GUID_CONSOLE_DISPLAY_STATE, GUID_MONITOR_POWER_ON, GUID_SESSION_DISPLAY_STATUS,
};
use windows::Win32::UI::WindowsAndMessaging::*;

struct SendHWND(HWND);
unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

static POWER_HWND: OnceLock<SendHWND> = OnceLock::new();
static SHOULD_DISPLAY_BE_ON: OnceLock<AtomicBool> = OnceLock::new();

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
    } else if msg == WM_WTSSESSION_CHANGE {
        match wparam.0 as u32 {
            7 => {
                // WTS_SESSION_LOCK - 锁屏
                log::info!("[锁屏] 系统已锁屏，暂时禁用屏幕常亮");
                if let Some(state) = SHOULD_DISPLAY_BE_ON.get() {
                    if state.load(Ordering::SeqCst) {
                        // 仅移除 ES_DISPLAY_REQUIRED，保留 ES_SYSTEM_REQUIRED
                        let flags = EXECUTION_STATE(ES_CONTINUOUS.0 | ES_SYSTEM_REQUIRED.0);
                        unsafe {
                            let _ = SetThreadExecutionState(flags);
                        }
                        log::info!("[锁屏] 已临时禁用屏幕常亮，仅保持阻止系统休眠");
                    }
                }
            }
            8 => {
                // WTS_SESSION_UNLOCK - 解锁
                log::info!("[锁屏] 系统已解锁，恢复屏幕常亮设置");
                if let Some(state) = SHOULD_DISPLAY_BE_ON.get() {
                    if state.load(Ordering::SeqCst) {
                        // 恢复 ES_DISPLAY_REQUIRED
                        let flags = EXECUTION_STATE(
                            ES_CONTINUOUS.0 | ES_SYSTEM_REQUIRED.0 | ES_DISPLAY_REQUIRED.0,
                        );
                        unsafe {
                            let _ = SetThreadExecutionState(flags);
                        }
                        log::info!("[锁屏] 已恢复屏幕常亮");
                    }
                }
            }
            _ => {}
        }
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub fn init_power_monitor(keep_display_on: bool) {
    let _ = SHOULD_DISPLAY_BE_ON.set(AtomicBool::new(keep_display_on));
}

pub fn update_display_on_state(keep_display_on: bool) {
    if let Some(state) = SHOULD_DISPLAY_BE_ON.get() {
        state.store(keep_display_on, Ordering::SeqCst);
    }
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

        // 注册会话通知以检测锁屏
        use windows::Win32::System::RemoteDesktop::{
            NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
        };
        let _ = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);

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

        log::info!("电源监控已启动（包含锁屏检测）");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}
