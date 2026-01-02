/// Pattern scanning utilities for UE structure detection

use crate::platform::windows::{query_memory_regions, read_process_memory, HANDLE};
use windows::Win32::System::Memory::MEM_COMMIT;

/// パターンスキャン結果
pub struct ScanResult {
    pub address: usize,
    pub offset: usize,
}

/// バイトパターン（0x00 = ワイルドカード）
pub struct Pattern {
    bytes: Vec<u8>,
    mask: Vec<bool>, // true = マッチが必要, false = ワイルドカード
}

impl Pattern {
    /// パターンを文字列から作成
    /// 例: "48 8B 05 ?? ?? ?? ?? 48 85 C0"
    pub fn from_string(pattern: &str) -> Self {
        let mut bytes = Vec::new();
        let mut mask = Vec::new();

        for part in pattern.split_whitespace() {
            if part == "??" || part == "?" {
                bytes.push(0);
                mask.push(false);
            } else if let Ok(byte) = u8::from_str_radix(part, 16) {
                bytes.push(byte);
                mask.push(true);
            }
        }

        Self { bytes, mask }
    }

    /// バイト配列とマッチするか
    fn matches(&self, data: &[u8]) -> bool {
        if data.len() < self.bytes.len() {
            return false;
        }

        for i in 0..self.bytes.len() {
            if self.mask[i] && data[i] != self.bytes[i] {
                return false;
            }
        }

        true
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

/// メモリ領域内でパターンをスキャン
pub fn scan_pattern(
    handle: HANDLE,
    pattern: &Pattern,
    module_base: usize,
    module_size: usize,
) -> Result<Vec<ScanResult>, anyhow::Error> {
    let mut results = Vec::new();

    // メモリ領域を取得
    let regions = query_memory_regions(handle)?;

    for region in regions {
        // モジュール範囲内かつコミット済みメモリのみ
        if region.base_address < module_base
            || region.base_address >= module_base + module_size
            || region.state != MEM_COMMIT.0
            || !region.is_readable
        {
            continue;
        }

        // メモリを読み取り
        if let Ok(data) = read_process_memory(handle, region.base_address, region.size) {
            // パターン検索
            for i in 0..data.len().saturating_sub(pattern.len()) {
                if pattern.matches(&data[i..]) {
                    results.push(ScanResult {
                        address: region.base_address + i,
                        offset: i,
                    });
                }
            }
        }
    }

    Ok(results)
}

/// RIP相対アドレスを解決（x64）
/// 例: 48 8B 05 [XX XX XX XX] → RIP + offset + 7
pub fn resolve_rip_relative(instruction_addr: usize, data: &[u8], offset: usize) -> usize {
    if data.len() < offset + 4 {
        return 0;
    }

    let rel_offset = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);

    // instruction_addr + 命令長 + relative offset
    let instruction_end = instruction_addr + data.len();
    (instruction_end as i64 + rel_offset as i64) as usize
}
