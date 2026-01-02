/// Module enumeration for process

use anyhow::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

#[derive(Clone, Debug)]
pub struct ModuleInfo {
    pub name: String,
    pub base_address: usize,
    pub size: usize,
}

/// プロセスのモジュール一覧を取得
pub fn list_modules(process_id: u32) -> Result<Vec<ModuleInfo>> {
    let snapshot = unsafe {
        CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process_id)?
    };

    let mut modules = Vec::new();
    let mut entry = MODULEENTRY32W {
        dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };

    unsafe {
        if Module32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szModule[..entry
                        .szModule
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szModule.len())],
                );

                modules.push(ModuleInfo {
                    name,
                    base_address: entry.modBaseAddr as usize,
                    size: entry.modBaseSize as usize,
                });

                if Module32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }

    Ok(modules)
}

/// メインモジュール（実行ファイル）を取得
pub fn get_main_module(process_id: u32) -> Result<ModuleInfo> {
    let modules = list_modules(process_id)?;
    modules
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No modules found"))
}
