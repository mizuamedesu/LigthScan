use super::{ScanValue, ValueType};
use serde::{Deserialize, Serialize};

/// Represents a single scan result - an address and its value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub address: usize,
    pub previous_value: Vec<u8>,
    pub current_value: Option<Vec<u8>>,
}

impl ScanResult {
    pub fn new(address: usize, value: Vec<u8>) -> Self {
        Self {
            address,
            previous_value: value.clone(),
            current_value: Some(value),
        }
    }

    pub fn get_current_value(&self) -> &[u8] {
        self.current_value.as_ref().unwrap_or(&self.previous_value)
    }

    pub fn update_value(&mut self, new_value: Vec<u8>) {
        self.previous_value = self.current_value.take().unwrap_or(self.previous_value.clone());
        self.current_value = Some(new_value);
    }

    /// Parses the current value as a ScanValue of the given type
    pub fn parse_value(&self, value_type: ValueType) -> Option<ScanValue> {
        ScanValue::from_bytes(self.get_current_value(), value_type)
    }
}

/// Collection of scan results with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResults {
    pub results: Vec<ScanResult>,
    pub value_type: ValueType,
    pub scan_count: u32,
}

impl ScanResults {
    pub fn new(value_type: ValueType) -> Self {
        Self {
            results: Vec::new(),
            value_type,
            scan_count: 0,
        }
    }

    pub fn add_result(&mut self, result: ScanResult) {
        self.results.push(result);
    }

    pub fn clear(&mut self) {
        self.results.clear();
        self.scan_count = 0;
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn increment_scan_count(&mut self) {
        self.scan_count += 1;
    }

    /// Get a slice of results for display (pagination support)
    pub fn get_page(&self, offset: usize, limit: usize) -> &[ScanResult] {
        let start = offset.min(self.results.len());
        let end = (offset + limit).min(self.results.len());
        &self.results[start..end]
    }
}

/// Scan options for configuring how a scan is performed
#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub value_type: ValueType,
    pub alignment: usize,
    pub writable_only: bool,
    pub readable_only: bool,
    pub executable_only: bool,
}

impl ScanOptions {
    pub fn new(value_type: ValueType) -> Self {
        Self {
            value_type,
            alignment: value_type.alignment(),
            writable_only: false,
            readable_only: true,
            executable_only: false,
        }
    }

    pub fn with_alignment(mut self, alignment: usize) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn writable_only(mut self) -> Self {
        self.writable_only = true;
        self
    }
}
