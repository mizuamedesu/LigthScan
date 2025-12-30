use crate::scanner::{MemoryScanner, Process};
use crate::types::{ScanOptions, ScanResult, ScanResults, ScanType, ScanValue, ValueType};
use anyhow::Result;

/// Main scanner for performing memory scans
pub struct Scanner {
    process: Process,
    results: ScanResults,
}

impl Scanner {
    /// Creates a new scanner for the given process
    pub fn new(process: Process) -> Self {
        Self {
            process,
            results: ScanResults::new(ValueType::I32), // Default type
        }
    }

    /// Gets a reference to the process
    pub fn process(&self) -> &Process {
        &self.process
    }

    /// Gets a reference to the current scan results
    pub fn results(&self) -> &ScanResults {
        &self.results
    }

    /// Gets a mutable reference to the current scan results
    pub fn results_mut(&mut self) -> &mut ScanResults {
        &mut self.results
    }

    /// Performs a first scan for the given value
    pub fn first_scan(
        &mut self,
        value: &ScanValue,
        scan_type: ScanType,
        options: &ScanOptions,
    ) -> Result<usize> {
        // Reset previous results
        self.results = ScanResults::new(options.value_type);

        let memory = MemoryScanner::new(&self.process);

        // Get all memory regions
        let regions = memory.query_regions()?;

        // Filter regions based on options
        let regions = memory.filter_regions(
            regions,
            options.readable_only,
            options.writable_only,
            options.executable_only,
        );

        tracing::info!("Scanning {} memory regions", regions.len());

        // Scan regions sequentially (HANDLE is not thread-safe)
        let mut results = Vec::new();
        for region in &regions {
            let region_results = scan_region_first(region, value, scan_type, options, &memory);
            results.extend(region_results);
        }

        for result in results {
            self.results.add_result(result);
        }

        self.results.increment_scan_count();
        Ok(self.results.len())
    }

    /// Performs a subsequent scan to filter previous results
    pub fn next_scan(&mut self, value: &ScanValue, scan_type: ScanType) -> Result<usize> {
        if self.results.is_empty() {
            return Ok(0);
        }

        let memory = MemoryScanner::new(&self.process);

        // Re-read values at known addresses (sequential for thread safety)
        let filtered: Vec<ScanResult> = self
            .results
            .results
            .iter()
            .filter_map(|result| {
                rescan_address(result, value, scan_type, self.results.value_type, &memory)
            })
            .collect();

        self.results.results = filtered;
        self.results.increment_scan_count();

        Ok(self.results.len())
    }

    /// Resets the scanner
    pub fn reset(&mut self) {
        self.results.clear();
    }

    /// Writes a value to a specific address
    pub fn write_value(&self, address: usize, value: &ScanValue) -> Result<()> {
        let memory = MemoryScanner::new(&self.process);
        memory.write_memory(address, &value.to_bytes())
    }

    /// Reads the current value at an address
    pub fn read_value(&self, address: usize, value_type: ValueType) -> Result<ScanValue> {
        let memory = MemoryScanner::new(&self.process);
        let bytes = memory.read_memory(address, value_type.size())?;
        ScanValue::from_bytes(&bytes, value_type)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse value"))
    }
}

/// Scans a single memory region for the first time
fn scan_region_first(
    region: &crate::platform::MemoryRegion,
    value: &ScanValue,
    scan_type: ScanType,
    options: &ScanOptions,
    memory: &MemoryScanner,
) -> Vec<ScanResult> {
    let mut results = Vec::new();

    // Read the entire region
    let data = match memory.read_region(region) {
        Ok(data) => data,
        Err(_) => return results,
    };

    let value_size = options.value_type.size();
    let alignment = options.alignment;

    // Scan through the memory
    let mut offset = 0;
    while offset + value_size <= data.len() {
        // Check alignment
        if (region.base_address + offset) % alignment == 0 {
            let chunk = &data[offset..offset + value_size];

            if let Some(found_value) = ScanValue::from_bytes(chunk, options.value_type) {
                let matches = match scan_type {
                    ScanType::Unknown => true, // Match everything on unknown scan
                    _ => found_value.compare(value, scan_type),
                };

                if matches {
                    results.push(ScanResult::new(
                        region.base_address + offset,
                        chunk.to_vec(),
                    ));
                }
            }
        }

        offset += alignment;
    }

    results
}

/// Re-scans a specific address with filter criteria
fn rescan_address(
    previous: &ScanResult,
    value: &ScanValue,
    scan_type: ScanType,
    value_type: ValueType,
    memory: &MemoryScanner,
) -> Option<ScanResult> {
    let size = value_type.size();

    // Read current value
    let current_bytes = memory.read_memory(previous.address, size).ok()?;
    let current_value = ScanValue::from_bytes(&current_bytes, value_type)?;
    let previous_value = ScanValue::from_bytes(previous.get_current_value(), value_type)?;

    let matches = match scan_type {
        ScanType::Exact | ScanType::GreaterThan | ScanType::LessThan | ScanType::Between(_, _) => {
            current_value.compare(value, scan_type)
        }
        ScanType::Increased => current_value.as_f64() > previous_value.as_f64(),
        ScanType::Decreased => current_value.as_f64() < previous_value.as_f64(),
        ScanType::Changed => current_value != previous_value,
        ScanType::Unchanged => current_value == previous_value,
        ScanType::Unknown => true,
    };

    if matches {
        let mut result = previous.clone();
        result.update_value(current_bytes);
        Some(result)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let current_pid = std::process::id();
        let process = Process::open(current_pid, "self".to_string())
            .expect("Failed to open current process");

        let scanner = Scanner::new(process);
        assert_eq!(scanner.results().len(), 0);
    }
}
