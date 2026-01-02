/// UE version-specific offsets

use super::UEVersion;

pub struct UEOffsets {
    pub uobject_name: usize,
    pub uobject_class: usize,
    pub uobject_outer: usize,
    pub ufield_next: usize,
    pub ustruct_children: usize,
    pub ufunction_func: usize,
}

impl UEOffsets {
    pub fn for_version(version: UEVersion) -> Self {
        match version {
            UEVersion::UE4_27 => Self {
                uobject_name: 0x18,
                uobject_class: 0x10,
                uobject_outer: 0x20,
                ufield_next: 0x28,
                ustruct_children: 0x50,
                ufunction_func: 0xB0,
            },
            UEVersion::UE5_3 => Self {
                uobject_name: 0x18,
                uobject_class: 0x10,
                uobject_outer: 0x20,
                ufield_next: 0x28,
                ustruct_children: 0x50,
                ufunction_func: 0xB8,
            },
            _ => Self::default(),
        }
    }
}

impl Default for UEOffsets {
    fn default() -> Self {
        Self {
            uobject_name: 0x18,
            uobject_class: 0x10,
            uobject_outer: 0x20,
            ufield_next: 0x28,
            ustruct_children: 0x50,
            ufunction_func: 0xB0,
        }
    }
}
