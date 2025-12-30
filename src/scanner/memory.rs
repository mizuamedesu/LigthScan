use crate::platform::{self, MemoryRegion};
use crate::scanner::Process;
use anyhow::Result;

/// Chunk size for reading memory (1 MB)
const CHUNK_SIZE: usize = 1024 * 1024;

/// Memory scanner for reading and writing process memory
pub struct MemoryScanner<'a> {
    process: &'a Process,
}

impl<'a> MemoryScanner<'a> {
    pub fn new(process: &'a Process) -> Self {
        Self { process }
    }

    /// Queries all memory regions in the process
    pub fn query_regions(&self) -> Result<Vec<MemoryRegion>> {
        platform::query_memory_regions(self.process.handle())
    }

    /// Filters regions based on criteria
    pub fn filter_regions(
        &self,
        regions: Vec<MemoryRegion>,
        readable_only: bool,
        writable_only: bool,
        executable_only: bool,
    ) -> Vec<MemoryRegion> {
        regions
            .into_iter()
            .filter(|region| {
                (!readable_only || region.is_readable)
                    && (!writable_only || region.is_writable)
                    && (!executable_only || region.is_executable)
            })
            .collect()
    }

    /// Reads memory at a specific address
    pub fn read_memory(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        platform::read_process_memory(self.process.handle(), address, size)
    }

    /// Writes memory at a specific address
    pub fn write_memory(&self, address: usize, data: &[u8]) -> Result<()> {
        platform::write_process_memory(self.process.handle(), address, data)
    }

    /// Reads an entire memory region in chunks
    pub fn read_region(&self, region: &MemoryRegion) -> Result<Vec<u8>> {
        if region.size <= CHUNK_SIZE {
            // Read the entire region at once if it's small enough
            self.read_memory(region.base_address, region.size)
        } else {
            // Read in chunks for larger regions
            let mut buffer = Vec::with_capacity(region.size);
            let mut offset = 0;

            while offset < region.size {
                let chunk_size = (region.size - offset).min(CHUNK_SIZE);
                match self.read_memory(region.base_address + offset, chunk_size) {
                    Ok(mut chunk) => {
                        buffer.append(&mut chunk);
                        offset += chunk_size;
                    }
                    Err(_) => {
                        // If we fail to read a chunk, pad with zeros and continue
                        buffer.resize(offset + chunk_size, 0);
                        offset += chunk_size;
                    }
                }
            }

            Ok(buffer)
        }
    }

    /// Iterator over memory region chunks
    pub fn iter_region_chunks(
        &self,
        region: &MemoryRegion,
    ) -> impl Iterator<Item = (usize, Vec<u8>)> + '_ {
        let base = region.base_address;
        let size = region.size;
        let handle = self.process.handle();

        (0..size)
            .step_by(CHUNK_SIZE)
            .filter_map(move |offset| {
                let chunk_size = (size - offset).min(CHUNK_SIZE);
                let address = base + offset;

                platform::read_process_memory(handle, address, chunk_size)
                    .ok()
                    .map(|data| (address, data))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_regions() {
        // Open current process for testing
        let current_pid = std::process::id();
        let process = Process::open(current_pid, "self".to_string())
            .expect("Failed to open current process");

        let scanner = MemoryScanner::new(&process);
        let regions = scanner.query_regions().expect("Failed to query regions");

        assert!(!regions.is_empty(), "Should find at least one memory region");
    }
}
