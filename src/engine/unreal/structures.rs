/// Unreal Engine internal structures

use crate::platform::windows::{read_process_memory, HANDLE};

/// UObject 基底クラス
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UObject {
    pub vtable: usize,
    pub object_flags: u32,
    pub internal_index: i32,
    pub class: usize, // UClass*
    pub name: FName,
    pub outer: usize, // UObject*
}

impl UObject {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        let data = read_process_memory(handle, address, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }
}

/// FName - UE の文字列表現
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FName {
    pub comparison_index: u32,
    pub number: u32,
}

impl FName {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        let data = read_process_memory(handle, address, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }
}

/// FNameEntry - GNames の要素 (UE4.23+)
#[repr(C)]
pub struct FNameEntry {
    pub header: u16,   // フラグ + 長さ
    pub name: [u8; 1024], // 可変長だが固定サイズで近似
}

impl FNameEntry {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        let data = read_process_memory(handle, address, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }

    pub fn is_wide(&self) -> bool {
        (self.header & 1) != 0
    }

    pub fn len(&self) -> usize {
        (self.header >> 6) as usize
    }

    pub fn get_string(&self) -> String {
        let len = self.len();
        if len == 0 {
            return String::new();
        }

        if self.is_wide() {
            // UTF-16
            let wide_chars: Vec<u16> = self.name[..len * 2]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&wide_chars)
        } else {
            // ASCII
            String::from_utf8_lossy(&self.name[..len]).to_string()
        }
    }
}

/// FNamePool - GNames の実体 (UE4.23+)
#[repr(C)]
pub struct FNamePool {
    pub lock: usize,
    pub current_block: u32,
    pub current_byte_cursor: u32,
    pub blocks: usize, // FNameEntryAllocator::TBlockPtr*
}

impl FNamePool {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        let data = read_process_memory(handle, address, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }

    /// FName index から FNameEntry のアドレスを取得
    pub fn get_entry_address(
        &self,
        handle: HANDLE,
        index: u32,
    ) -> Result<usize, anyhow::Error> {
        const ENTRIES_PER_BLOCK: u32 = 16384;
        const STRIDE: usize = 2; // FNameEntry へのポインタのストライド

        let block_index = index / ENTRIES_PER_BLOCK;
        let offset_in_block = (index % ENTRIES_PER_BLOCK) as usize;

        // blocks[block_index] を読み取る
        let block_ptr_addr = self.blocks + (block_index as usize * 8);
        let block_ptr_data = read_process_memory(handle, block_ptr_addr, 8)?;
        let block_ptr = usize::from_le_bytes(block_ptr_data[..8].try_into().unwrap());

        if block_ptr == 0 {
            return Err(anyhow::anyhow!("Invalid block pointer"));
        }

        // block[offset] を読み取る
        let entry_ptr_addr = block_ptr + (offset_in_block * STRIDE);
        let entry_data = read_process_memory(handle, entry_ptr_addr, 8)?;
        let entry_addr = usize::from_le_bytes(entry_data[..8].try_into().unwrap());

        Ok(entry_addr)
    }
}

/// FUObjectItem - GObjects の要素
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FUObjectItem {
    pub object: usize,  // UObject*
    pub flags: i32,
    pub cluster_root_index: i32,
    pub serial_number: i32,
}

impl FUObjectItem {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        let data = read_process_memory(handle, address, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }

    pub fn is_valid(&self) -> bool {
        self.object != 0 && (self.flags & 1) == 0 // RF_NoFlags
    }
}

/// FUObjectArray - GObjects の実体
#[repr(C)]
pub struct FUObjectArray {
    pub obj_first_gc_index: i32,
    pub obj_last_non_gc_index: i32,
    pub max_objects_not_consid_by_gc: i32,
    pub open_for_disregard_for_gc: bool,
    pub obj_objects: usize, // FUObjectItem*
    pub max_elements: i32,
    pub num_elements: i32,
}

impl FUObjectArray {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        let data = read_process_memory(handle, address, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }

    /// インデックスから UObject のアドレスを取得
    pub fn get_object_address(&self, handle: HANDLE, index: i32) -> Result<usize, anyhow::Error> {
        if index < 0 || index >= self.num_elements {
            return Err(anyhow::anyhow!("Index out of bounds"));
        }

        let item_addr = self.obj_objects + (index as usize * std::mem::size_of::<FUObjectItem>());
        let item = FUObjectItem::read(handle, item_addr)?;

        if !item.is_valid() {
            return Err(anyhow::anyhow!("Invalid object"));
        }

        Ok(item.object)
    }

    /// すべての有効な UObject のアドレスを取得
    pub fn get_all_objects(&self, handle: HANDLE) -> Vec<usize> {
        let mut objects = Vec::new();

        for i in 0..self.num_elements {
            if let Ok(addr) = self.get_object_address(handle, i) {
                objects.push(addr);
            }
        }

        objects
    }
}

/// UField - メタデータの基底
#[repr(C)]
pub struct UField {
    // UObject を継承 (省略)
    pub next: usize, // UField*
}

/// UStruct - 構造化された型情報
#[repr(C)]
pub struct UStruct {
    // UField を継承
    pub super_struct: usize, // UStruct*
    pub children: usize,     // UField*
    pub child_properties: usize,
    pub properties_size: i32,
    pub min_alignment: i32,
}

impl UStruct {
    pub fn read(handle: HANDLE, address: usize) -> Result<Self, anyhow::Error> {
        // UObject のサイズを飛ばして UStruct 部分を読む
        let offset = std::mem::size_of::<UObject>();
        let data = read_process_memory(handle, address + offset, std::mem::size_of::<Self>())?;
        Ok(unsafe { std::ptr::read(data.as_ptr() as *const Self) })
    }
}

/// UFunction - 関数情報
#[repr(C)]
pub struct UFunction {
    pub function_flags: u32,
    pub num_params: u8,
    pub params_size: u16,
    pub return_value_offset: u16,
    pub rpc_id: u16,
    pub rpc_response_id: u16,
    pub first_property_to_init: usize, // UProperty*
    pub native_func: usize,            // 関数ポインタ
}

impl UFunction {
    /// UFunction の flag をチェック
    pub fn is_native(&self) -> bool {
        (self.function_flags & 0x00000400) != 0 // FUNC_Native
    }

    pub fn is_blueprint_callable(&self) -> bool {
        (self.function_flags & 0x00000001) != 0 // FUNC_BlueprintCallable
    }
}
