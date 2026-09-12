use std::sync::OnceLock;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

struct SendHWND(HWND);
unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

static NOTIFY_HWND: OnceLock<SendHWND> = OnceLock::new();
static NOTIFY_ICON_ID: u32 = 1;

unsafe extern "system" fn notify_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_notify_window() -> HWND {
    if let Some(wrapper) = NOTIFY_HWND.get() {
        return wrapper.0;
    }

    unsafe {
        let class_name = w!("AweakNotifyWindow");
        let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default().into();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(notify_wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..std::mem::zeroed()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("Aweak Notify"),
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

        let _ = NOTIFY_HWND.set(SendHWND(hwnd));
        hwnd
    }
}

fn copy_str_to_wide(dest: &mut [u16], src: &str) {
    let wide: Vec<u16> = src.encode_utf16().chain(std::iter::once(0)).collect();
    let copy_len = std::cmp::min(wide.len(), dest.len());
    dest[..copy_len].copy_from_slice(&wide[..copy_len]);
}

pub fn show_notification_timed(title: &str, message: &str, duration_ms: u32) {
    let title = title.to_string();
    let message = message.to_string();

    std::thread::spawn(move || {
        let hwnd = ensure_notify_window();

        // 使用 NIM_ADD + NIF_INFO 同时添加图标并显示气泡通知
        let (nid, hicon) = unsafe {
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = NOTIFY_ICON_ID;
            nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE | NIF_INFO;
            nid.uCallbackMessage = WM_APP + 1;
            nid.hIcon = LoadIconW(None, IDI_INFORMATION).unwrap_or_default();
            nid.dwInfoFlags = NIIF_INFO;
            copy_str_to_wide(&mut nid.szTip, "aweak");
            copy_str_to_wide(&mut nid.szInfoTitle, &title);
            copy_str_to_wide(&mut nid.szInfo, &message);
            (nid, nid.hIcon)
        };

        unsafe {
            let result = Shell_NotifyIconW(NIM_ADD, &nid);
            if !result.as_bool() {
                log::error!("Shell_NotifyIconW NIM_ADD 失败");
                return;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(duration_ms as u64));

        unsafe {
            let mut nid_del: NOTIFYICONDATAW = std::mem::zeroed();
            nid_del.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid_del.hWnd = hwnd;
            nid_del.uID = NOTIFY_ICON_ID;
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid_del);
            let _ = DestroyIcon(hicon);
        }
    });
}
