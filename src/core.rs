use std::io;
use windows::Win32::System::Power::{
    ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, EXECUTION_STATE,
    SetThreadExecutionState,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AwakeMode {
    Passive,
    Indefinite,
    Timed,
    Expirable,
}

pub struct AwakeManager {
    mode: AwakeMode,
    keep_display_on: bool,
    time_limit_seconds: Option<u64>,
    expire_at: Option<std::time::SystemTime>,
}

impl AwakeManager {
    pub fn new() -> Self {
        Self {
            mode: AwakeMode::Passive,
            keep_display_on: false,
            time_limit_seconds: None,
            expire_at: None,
        }
    }

    pub fn set_mode(&mut self, mode: AwakeMode) {
        self.mode = mode;
    }

    pub fn set_keep_display_on(&mut self, keep: bool) {
        self.keep_display_on = keep;
    }

    pub fn set_time_limit(&mut self, seconds: u64) {
        self.time_limit_seconds = Some(seconds);
    }

    pub fn set_expire_at(&mut self, time: std::time::SystemTime) {
        self.expire_at = Some(time);
    }

    pub fn get_mode(&self) -> AwakeMode {
        self.mode
    }

    pub fn is_keep_display_on(&self) -> bool {
        self.keep_display_on
    }

    pub fn get_time_limit(&self) -> Option<u64> {
        self.time_limit_seconds
    }

    pub fn get_expire_at(&self) -> Option<std::time::SystemTime> {
        self.expire_at
    }

    pub fn apply(&self) -> io::Result<()> {
        let flags = self.compute_flags();
        let result = unsafe { SetThreadExecutionState(flags) };

        if result == EXECUTION_STATE(0) {
            return Err(io::Error::last_os_error());
        }

        log::info!("已应用执行状态: {:#x}", flags.0);
        Ok(())
    }

    pub fn release(&self) -> io::Result<()> {
        let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };

        if result == EXECUTION_STATE(0) {
            return Err(io::Error::last_os_error());
        }

        log::info!("已释放执行状态");
        Ok(())
    }

    fn compute_flags(&self) -> EXECUTION_STATE {
        if self.mode == AwakeMode::Passive {
            return ES_CONTINUOUS;
        }

        let mut flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;

        if self.keep_display_on {
            flags |= ES_DISPLAY_REQUIRED;
        }

        flags
    }
}
