use anyhow::Result;
use std::ptr;
use windows::Win32::Foundation::HWND;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

/// Checks if the current process is running with elevated privileges (administrator)
pub fn is_elevated() -> Result<bool> {
    unsafe {
        let mut token = Default::default();

        // Get current process token
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        // Query elevation status
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0u32;

        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        )?;

        Ok(elevation.TokenIsElevated != 0)
    }
}

/// Restarts the current application with administrator privileges
/// This will show the Windows UAC dialog
pub fn restart_as_admin() -> Result<()> {
    unsafe {
        // Get the current executable path
        let exe_path = std::env::current_exe()?;
        let exe_path_str = exe_path.to_string_lossy();

        // Convert to wide string (UTF-16)
        let mut exe_wide: Vec<u16> = exe_path_str.encode_utf16().collect();
        exe_wide.push(0); // Null terminator

        let operation: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

        // ShellExecute with "runas" verb shows UAC dialog
        let result = ShellExecuteW(
            HWND::default(),
            PCWSTR(operation.as_ptr()),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR(ptr::null()),
            PCWSTR(ptr::null()),
            SW_SHOWNORMAL,
        );

        if result.0 as usize <= 32 {
            return Err(anyhow::anyhow!("Failed to restart as administrator"));
        }

        // Exit current process
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated() {
        // Just test that it doesn't crash
        let _ = is_elevated();
    }
}
