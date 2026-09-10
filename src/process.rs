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

#[allow(non_snake_case)]
pub fn get_parent_pid() -> Option<u32> {
    unsafe {
        use std::mem;
        use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessId};

        // 使用 NtQueryInformationProcess 获取父进程 ID
        #[repr(C)]
        struct ProcessBasicInformation {
            Reserved1: *mut std::ffi::c_void,
            PebBaseAddress: *mut std::ffi::c_void,
            AffinityMask: usize,
            BasePriority: i32,
            UniqueProcessId: *mut std::ffi::c_void,
            InheritedFromUniqueProcessId: *mut std::ffi::c_void,
        }

        let ntdll = windows::Win32::System::LibraryLoader::LoadLibraryW(
            windows::core::PCWSTR::from_raw(windows::core::HSTRING::from("ntdll.dll").as_ptr()),
        )
        .ok()?;

        let nt_query = windows::Win32::System::LibraryLoader::GetProcAddress(
            ntdll,
            windows::core::PCSTR(b"NtQueryInformationProcess\0".as_ptr()),
        );

        if let Some(func) = nt_query {
            type NtQueryInformationProcessFn = unsafe extern "system" fn(
                windows::Win32::Foundation::HANDLE,
                u32,
                *mut std::ffi::c_void,
                u32,
                *mut u32,
            ) -> i32;

            let func: NtQueryInformationProcessFn = std::mem::transmute(func);

            let handle = GetCurrentProcess();
            let mut pbi: ProcessBasicInformation = mem::zeroed();
            let mut return_length = 0u32;

            let status = func(
                handle,
                0, // ProcessBasicInformation
                &mut pbi as *mut _ as *mut std::ffi::c_void,
                mem::size_of::<ProcessBasicInformation>() as u32,
                &mut return_length,
            );

            if status == 0 {
                let parent_pid = pbi.InheritedFromUniqueProcessId as u32;
                let current_pid = GetProcessId(handle);
                if parent_pid != 0 && parent_pid != current_pid {
                    return Some(parent_pid);
                }
            }
        }
    }
    None
}

/// 检查进程是否为终端或系统进程（不应绑定的进程）
pub fn is_terminal_or_system_process(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if handle.is_err() {
            return true;
        }
        let handle = handle.unwrap();

        // 获取进程名称
        use windows::Win32::System::ProcessStatus::GetModuleBaseNameA;

        let mut buffer = [0u8; 260];
        let size = GetModuleBaseNameA(handle, None, &mut buffer);

        let _ = CloseHandle(handle);

        if size > 0 {
            let name = String::from_utf8_lossy(&buffer[..size as usize]).to_lowercase();
            // 终端和系统进程列表
            let terminal_processes = [
                "cmd.exe",
                "powershell.exe",
                "pwsh.exe",
                "wt.exe",
                "explorer.exe",
                "conhost.exe",
                "sihost.exe",
                "taskhostw.exe",
                "startmenuexperiencehost.exe",
                "searchui.exe",
                "searchapp.exe",
                "system",
                "smss.exe",
                "csrss.exe",
                "wininit.exe",
                "winlogon.exe",
                "services.exe",
                "lsass.exe",
                "svchost.exe",
            ];

            for term in &terminal_processes {
                if name == *term {
                    return true;
                }
            }
        }

        false
    }
}
