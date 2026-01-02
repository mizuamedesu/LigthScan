use crate::platform::{self, ProcessInfo};
use anyhow::Result;
use windows::Win32::Foundation::HANDLE;

/// Represents an opened process for memory scanning
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub handle: HANDLE,
}

impl Process {
    /// Lists all running processes
    pub fn list_all() -> Result<Vec<ProcessInfo>> {
        platform::list_processes()
    }

    /// Opens a process by PID
    pub fn open(pid: u32, name: String) -> Result<Self> {
        let handle = platform::open_process(pid)?;
        Ok(Self { pid, name, handle })
    }

    /// Opens a process from ProcessInfo
    pub fn from_info(info: &ProcessInfo) -> Result<Self> {
        Self::open(info.pid, info.name.clone())
    }

    /// Gets the process handle
    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    /// Gets the process handle as usize (for engine abstraction)
    pub fn handle_as_usize(&self) -> usize {
        unsafe { std::mem::transmute(self.handle) }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _ = platform::close_process(self.handle);
    }
}
