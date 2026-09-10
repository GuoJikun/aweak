use std::time::Duration;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

pub fn is_process_running(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        match handle {
            Ok(h) => {
                let _ = CloseHandle(h);
                true
            }
            Err(_) => false,
        }
    }
}

pub fn wait_for_process_exit(pid: u32, check_interval: Duration) {
    loop {
        if !is_process_running(pid) {
            log::info!("进程 {} 已退出", pid);
            break;
        }
        std::thread::sleep(check_interval);
    }
}

pub fn get_parent_pid() -> Option<u32> {
    log::warn!("父进程检测功能未完全实现，请使用 --pid 参数");
    None
}