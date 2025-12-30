use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported value types for memory scanning
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    ByteArray(usize),
}

impl ValueType {
    /// Returns the size in bytes of this value type
    pub fn size(&self) -> usize {
        match self {
            ValueType::I8 | ValueType::U8 => 1,
            ValueType::I16 | ValueType::U16 => 2,
            ValueType::I32 | ValueType::U32 | ValueType::F32 => 4,
            ValueType::I64 | ValueType::U64 | ValueType::F64 => 8,
            ValueType::ByteArray(size) => *size,
        }
    }

    /// Returns the alignment requirement for this value type
    pub fn alignment(&self) -> usize {
        match self {
            ValueType::I8 | ValueType::U8 | ValueType::ByteArray(_) => 1,
            ValueType::I16 | ValueType::U16 => 2,
            ValueType::I32 | ValueType::U32 | ValueType::F32 => 4,
            ValueType::I64 | ValueType::U64 | ValueType::F64 => 8,
        }
    }

    /// Returns a human-readable name for this value type
    pub fn display_name(&self) -> &str {
        match self {
            ValueType::I8 => "Int8",
            ValueType::I16 => "Int16",
            ValueType::I32 => "Int32",
            ValueType::I64 => "Int64",
            ValueType::U8 => "UInt8",
            ValueType::U16 => "UInt16",
            ValueType::U32 => "UInt32",
            ValueType::U64 => "UInt64",
            ValueType::F32 => "Float",
            ValueType::F64 => "Double",
            ValueType::ByteArray(_) => "Byte Array",
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Represents a value to scan for in memory
#[derive(Clone, Debug, PartialEq)]
pub enum ScanValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    ByteArray(Vec<u8>),
}

impl ScanValue {
    /// Converts the value to a byte array (little-endian)
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ScanValue::I8(v) => vec![*v as u8],
            ScanValue::I16(v) => v.to_le_bytes().to_vec(),
            ScanValue::I32(v) => v.to_le_bytes().to_vec(),
            ScanValue::I64(v) => v.to_le_bytes().to_vec(),
            ScanValue::U8(v) => vec![*v],
            ScanValue::U16(v) => v.to_le_bytes().to_vec(),
            ScanValue::U32(v) => v.to_le_bytes().to_vec(),
            ScanValue::U64(v) => v.to_le_bytes().to_vec(),
            ScanValue::F32(v) => v.to_le_bytes().to_vec(),
            ScanValue::F64(v) => v.to_le_bytes().to_vec(),
            ScanValue::ByteArray(v) => v.clone(),
        }
    }

    /// Creates a ScanValue from bytes (little-endian)
    pub fn from_bytes(bytes: &[u8], value_type: ValueType) -> Option<Self> {
        match value_type {
            ValueType::I8 => Some(ScanValue::I8(i8::from_le_bytes([bytes[0]]))),
            ValueType::I16 if bytes.len() >= 2 => {
                Some(ScanValue::I16(i16::from_le_bytes([bytes[0], bytes[1]])))
            }
            ValueType::I32 if bytes.len() >= 4 => {
                Some(ScanValue::I32(i32::from_le_bytes(bytes[..4].try_into().ok()?)))
            }
            ValueType::I64 if bytes.len() >= 8 => {
                Some(ScanValue::I64(i64::from_le_bytes(bytes[..8].try_into().ok()?)))
            }
            ValueType::U8 => Some(ScanValue::U8(bytes[0])),
            ValueType::U16 if bytes.len() >= 2 => {
                Some(ScanValue::U16(u16::from_le_bytes([bytes[0], bytes[1]])))
            }
            ValueType::U32 if bytes.len() >= 4 => {
                Some(ScanValue::U32(u32::from_le_bytes(bytes[..4].try_into().ok()?)))
            }
            ValueType::U64 if bytes.len() >= 8 => {
                Some(ScanValue::U64(u64::from_le_bytes(bytes[..8].try_into().ok()?)))
            }
            ValueType::F32 if bytes.len() >= 4 => {
                Some(ScanValue::F32(f32::from_le_bytes(bytes[..4].try_into().ok()?)))
            }
            ValueType::F64 if bytes.len() >= 8 => {
                Some(ScanValue::F64(f64::from_le_bytes(bytes[..8].try_into().ok()?)))
            }
            ValueType::ByteArray(size) if bytes.len() >= size => {
                Some(ScanValue::ByteArray(bytes[..size].to_vec()))
            }
            _ => None,
        }
    }

    /// Returns the ValueType of this ScanValue
    pub fn value_type(&self) -> ValueType {
        match self {
            ScanValue::I8(_) => ValueType::I8,
            ScanValue::I16(_) => ValueType::I16,
            ScanValue::I32(_) => ValueType::I32,
            ScanValue::I64(_) => ValueType::I64,
            ScanValue::U8(_) => ValueType::U8,
            ScanValue::U16(_) => ValueType::U16,
            ScanValue::U32(_) => ValueType::U32,
            ScanValue::U64(_) => ValueType::U64,
            ScanValue::F32(_) => ValueType::F32,
            ScanValue::F64(_) => ValueType::F64,
            ScanValue::ByteArray(v) => ValueType::ByteArray(v.len()),
        }
    }

    /// Compares this value with another using the given scan type
    pub fn compare(&self, other: &ScanValue, scan_type: ScanType) -> bool {
        use ScanType::*;

        match scan_type {
            Exact => self == other,
            GreaterThan => self.as_f64() > other.as_f64(),
            LessThan => self.as_f64() < other.as_f64(),
            Between(min, max) => {
                let val = self.as_f64();
                val >= min && val <= max
            }
            _ => false, // Other scan types are handled differently
        }
    }

    /// Converts the value to f64 for comparison purposes
    pub fn as_f64(&self) -> f64 {
        match self {
            ScanValue::I8(v) => *v as f64,
            ScanValue::I16(v) => *v as f64,
            ScanValue::I32(v) => *v as f64,
            ScanValue::I64(v) => *v as f64,
            ScanValue::U8(v) => *v as f64,
            ScanValue::U16(v) => *v as f64,
            ScanValue::U32(v) => *v as f64,
            ScanValue::U64(v) => *v as f64,
            ScanValue::F32(v) => *v as f64,
            ScanValue::F64(v) => *v,
            ScanValue::ByteArray(_) => 0.0,
        }
    }
}

impl fmt::Display for ScanValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanValue::I8(v) => write!(f, "{}", v),
            ScanValue::I16(v) => write!(f, "{}", v),
            ScanValue::I32(v) => write!(f, "{}", v),
            ScanValue::I64(v) => write!(f, "{}", v),
            ScanValue::U8(v) => write!(f, "{}", v),
            ScanValue::U16(v) => write!(f, "{}", v),
            ScanValue::U32(v) => write!(f, "{}", v),
            ScanValue::U64(v) => write!(f, "{}", v),
            ScanValue::F32(v) => write!(f, "{}", v),
            ScanValue::F64(v) => write!(f, "{}", v),
            ScanValue::ByteArray(v) => {
                write!(f, "[")?;
                for (i, byte) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{:02X}", byte)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// Types of scans supported
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScanType {
    Exact,
    GreaterThan,
    LessThan,
    Between(f64, f64),
    Unknown,
    Increased,
    Decreased,
    Changed,
    Unchanged,
}

impl ScanType {
    pub fn display_name(&self) -> &str {
        match self {
            ScanType::Exact => "Exact Value",
            ScanType::GreaterThan => "Greater Than",
            ScanType::LessThan => "Less Than",
            ScanType::Between(_, _) => "Between",
            ScanType::Unknown => "Unknown Initial Value",
            ScanType::Increased => "Increased",
            ScanType::Decreased => "Decreased",
            ScanType::Changed => "Changed",
            ScanType::Unchanged => "Unchanged",
        }
    }

    /// Returns true if this scan type requires a value input
    pub fn requires_value(&self) -> bool {
        matches!(
            self,
            ScanType::Exact | ScanType::GreaterThan | ScanType::LessThan | ScanType::Between(_, _)
        )
    }

    /// Returns true if this scan type can only be used for subsequent scans (not first scan)
    pub fn is_next_scan_only(&self) -> bool {
        matches!(
            self,
            ScanType::Increased | ScanType::Decreased | ScanType::Changed | ScanType::Unchanged
        )
    }
}

impl fmt::Display for ScanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
