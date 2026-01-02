/// Unreal Engine backend implementation details

use super::scanner::{scan_pattern, Pattern};
use super::signatures::VersionSignatures;
use super::structures::{FNamePool, FUObjectArray, UObject};
use super::{EngineError, Result, UnrealEngine};
use crate::platform::windows::{read_process_memory, HANDLE};
use windows::Win32::Foundation::HANDLE as WinHandle;

impl UnrealEngine {
    /// GNames のアドレスを検索
    pub(super) fn find_gnames_impl(&self) -> Result<usize> {
        let handle = unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };

        let module_base = self.module_base;
        let module_size = self.module_size;

        tracing::info!("Scanning for GNames in range 0x{:X} - 0x{:X} (size: 0x{:X})",
            module_base, module_base + module_size, module_size);

        let patterns = VersionSignatures::all();

        for (i, pattern_str) in patterns.gnames_patterns.iter().enumerate() {
            tracing::info!("Trying GNames pattern {}: {}", i + 1, pattern_str);
            let pattern = Pattern::from_string(pattern_str);

            match scan_pattern(handle, &pattern, module_base, module_size) {
                Ok(results) => {
                    tracing::info!("Pattern {} found {} matches", i + 1, results.len());

                    // 最大10個のマッチを試す
                    let max_tries = results.len().min(10);
                    for (idx, result) in results.iter().take(max_tries).enumerate() {
                        if idx > 0 {
                            tracing::info!("Trying match {} at 0x{:X}", idx + 1, result.address);
                        } else {
                            tracing::info!("First match at 0x{:X}", result.address);
                        }

                        // パターンに応じてオフセット位置を調整
                        let (offset_pos, instruction_end) = match pattern_str {
                            // 48 8D 0D (lea rcx, [rip+offset])
                            s if s.starts_with("48 8D 0D") => (3, 7),
                            // 48 8B 1D (mov rbx, [rip+offset])
                            s if s.starts_with("48 8B 1D") => (3, 7),
                            // 長いパターン (ALT2)
                            s if s.len() > 50 => (pattern.len() - 7, pattern.len() - 3),
                            // デフォルト: 48 8B 05 (mov rax, [rip+offset])
                            _ => (3, 7),
                        };

                        // RIP相対アドレスを解決
                        let inst_data = match read_process_memory(handle, result.address, pattern.len() + 8) {
                            Ok(data) => data,
                            Err(_) => continue,
                        };

                        if inst_data.len() < offset_pos + 4 {
                            continue;
                        }

                        let rel_offset = i32::from_le_bytes([
                            inst_data[offset_pos],
                            inst_data[offset_pos + 1],
                            inst_data[offset_pos + 2],
                            inst_data[offset_pos + 3],
                        ]);

                        let gnames_ptr = (result.address as i64 + instruction_end as i64 + rel_offset as i64) as usize;

                        if idx == 0 {
                            tracing::info!("GNames pointer calculated at 0x{:X} (rel_offset: 0x{:X})", gnames_ptr, rel_offset);
                        }

                        // GNames ポインタのアドレスが有効かチェック
                        // ポインタ自体を返す（値は後で読み取る）
                        if gnames_ptr > module_base && gnames_ptr < module_base + module_size + 0x10000000 {
                            if idx == 0 {
                                tracing::info!("Found GNames pointer location at 0x{:X}", gnames_ptr);
                            }
                            // ポインタが読み取り可能かテスト
                            if let Ok(_) = read_process_memory(handle, gnames_ptr, 8) {
                                tracing::info!("Found valid GNames pointer at 0x{:X} (match {})", gnames_ptr, idx + 1);
                                return Ok(gnames_ptr);
                            }
                        }
                    }

                    if results.len() > 10 {
                        tracing::warn!("Skipped {} additional matches", results.len() - 10);
                    }
                }
                Err(e) => {
                    tracing::warn!("Pattern {} scan failed: {}", i + 1, e);
                }
            }
        }

        Err(EngineError::InitializationFailed(
            "GNames not found".into(),
        ))
    }

    /// GObjects のアドレスを検索
    pub(super) fn find_gobjects_impl(&self) -> Result<usize> {
        let handle = unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };

        let module_base = self.module_base;
        let module_size = self.module_size;

        tracing::info!("Scanning for GObjects...");
        let patterns = VersionSignatures::all();

        for (i, pattern_str) in patterns.gobjects_patterns.iter().enumerate() {
            tracing::info!("Trying GObjects pattern {}: {}", i + 1, pattern_str);
            let pattern = Pattern::from_string(pattern_str);

            match scan_pattern(handle, &pattern, module_base, module_size) {
                Ok(results) => {
                    tracing::info!("Pattern {} found {} matches", i + 1, results.len());

                    let max_tries = results.len().min(10);
                    for (idx, result) in results.iter().take(max_tries).enumerate() {
                        if idx > 0 {
                            tracing::info!("Trying match {} at 0x{:X}", idx + 1, result.address);
                        }

                        let inst_data = match read_process_memory(handle, result.address, pattern.len() + 8) {
                            Ok(data) => data,
                            Err(_) => continue,
                        };

                        let rel_offset = i32::from_le_bytes([
                            inst_data[3],
                            inst_data[4],
                            inst_data[5],
                            inst_data[6],
                        ]);

                        let gobjects_ptr = (result.address as i64 + 7 + rel_offset as i64) as usize;

                        if gobjects_ptr > module_base && gobjects_ptr < module_base + module_size + 0x10000000 {
                            if let Ok(ptr_data) = read_process_memory(handle, gobjects_ptr, 8) {
                                // ポインタの値を読んで検証
                                let gobjects_val = usize::from_le_bytes(ptr_data[..8].try_into().unwrap());

                                if idx == 0 || idx < 3 {
                                    tracing::info!("Match {} at instruction 0x{:X} -> pointer at 0x{:X}, value: 0x{:X}",
                                        idx + 1, result.address, gobjects_ptr, gobjects_val);
                                }

                                // nullでないポインタを見つけたら、それを使用
                                if gobjects_val != 0 {
                                    tracing::info!("Found valid GObjects pointer at 0x{:X} with non-null value 0x{:X} (match {})",
                                        gobjects_ptr, gobjects_val, idx + 1);
                                    return Ok(gobjects_ptr);
                                }
                            }
                        }
                    }

                    // すべてnullだった場合、警告を出す
                    if results.len() > 0 {
                        tracing::warn!("All {} GObjects matches had null values, will retry later", results.len());
                    }
                }
                Err(e) => {
                    tracing::warn!("Pattern {} scan failed: {}", i + 1, e);
                }
            }
        }

        Err(EngineError::InitializationFailed(
            "GObjects not found".into(),
        ))
    }

    /// ProcessEvent のアドレスを検索
    pub(super) fn find_process_event_impl(&self) -> Result<usize> {
        let handle = unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };

        let module_base = self.module_base;
        let module_size = self.module_size;

        let patterns = VersionSignatures::all();

        for pattern_str in patterns.process_event_patterns {
            let pattern = Pattern::from_string(pattern_str);
            if let Ok(results) = scan_pattern(handle, &pattern, module_base, module_size) {
                if let Some(result) = results.first() {
                    tracing::info!("Found ProcessEvent at 0x{:X}", result.address);
                    return Ok(result.address);
                }
            }
        }

        Err(EngineError::InitializationFailed(
            "ProcessEvent not found".into(),
        ))
    }

    /// FName から文字列を取得
    pub(super) fn get_fname_impl(&self, index: u32) -> Result<String> {
        let handle = unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };

        let name_pool = FNamePool::read(handle, self.gnames)?;
        let entry_addr = name_pool.get_entry_address(handle, index)?;

        let entry_header_data = read_process_memory(handle, entry_addr, 2)?;
        let header = u16::from_le_bytes([entry_header_data[0], entry_header_data[1]]);

        let is_wide = (header & 1) != 0;
        let len = (header >> 6) as usize;

        if len == 0 {
            return Ok(String::new());
        }

        let string_data = read_process_memory(handle, entry_addr + 2, if is_wide { len * 2 } else { len })?;

        if is_wide {
            let wide_chars: Vec<u16> = string_data
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            Ok(String::from_utf16_lossy(&wide_chars))
        } else {
            Ok(String::from_utf8_lossy(&string_data).to_string())
        }
    }

    /// UObject の名前を取得
    pub(super) fn get_object_name_impl(&self, obj_addr: usize) -> Result<String> {
        let handle = unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };

        let obj = UObject::read(handle, obj_addr)?;
        self.get_fname_impl(obj.name.comparison_index)
    }

    /// GObjects から全オブジェクトを取得
    pub(super) fn get_all_objects_impl(&self) -> Result<Vec<usize>> {
        let handle = unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };

        let uobject_array = FUObjectArray::read(handle, self.gobjects)?;
        Ok(uobject_array.get_all_objects(handle))
    }

    /// クラス名で UClass を検索
    pub(super) fn find_class_by_name_impl(&self, name: &str) -> Result<usize> {
        let all_objects = self.get_all_objects_impl()?;

        for obj_addr in all_objects {
            if let Ok(obj_name) = self.get_object_name_impl(obj_addr) {
                if obj_name == name {
                    // UClass かどうかを確認（Class->Class == Class なら UClass）
                    let handle =
                        unsafe { std::mem::transmute::<usize, WinHandle>(self.process_handle) };
                    let obj = UObject::read(handle, obj_addr)?;

                    if obj.class != 0 {
                        let class_obj = UObject::read(handle, obj.class)?;
                        if class_obj.class == obj.class {
                            // これは UClass
                            return Ok(obj_addr);
                        }
                    }
                }
            }
        }

        Err(EngineError::ClassNotFound(name.to_string()))
    }
}
