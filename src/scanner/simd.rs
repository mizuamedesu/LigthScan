// SIMD-optimized scanning routines
// This module provides vectorized search implementations for common value types

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD-accelerated scan for i32 values
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn simd_scan_i32_avx2(data: &[u8], target: i32, alignment: usize) -> Vec<usize> {
    let mut results = Vec::new();

    if data.len() < 32 {
        // Fall back to scalar for small buffers
        return scalar_scan_i32(data, target, alignment);
    }

    // Load target into all lanes of AVX2 register (8 x i32)
    let target_vec = _mm256_set1_epi32(target);

    let chunks = data.len() / 32;

    for chunk_idx in 0..chunks {
        let chunk_offset = chunk_idx * 32;

        // Load 32 bytes = 8 x i32 values
        let data_vec = _mm256_loadu_si256(data.as_ptr().add(chunk_offset) as *const __m256i);

        // Compare
        let cmp_result = _mm256_cmpeq_epi32(data_vec, target_vec);

        // Extract comparison mask
        let mask = _mm256_movemask_epi8(cmp_result);

        if mask != 0 {
            // Check each potential match
            for i in 0..8 {
                let bit_pos = i * 4;
                if (mask & (0xF << bit_pos)) != 0 {
                    let addr = chunk_offset + i * 4;
                    if addr % alignment == 0 {
                        results.push(addr);
                    }
                }
            }
        }
    }

    // Handle remaining bytes with scalar code
    let mut offset = chunks * 32;
    while offset + 4 <= data.len() {
        if offset % alignment == 0 {
            let value = i32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            if value == target {
                results.push(offset);
            }
        }
        offset += alignment;
    }

    results
}

/// SIMD-accelerated scan for f32 values
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn simd_scan_f32_avx2(data: &[u8], target: f32, alignment: usize) -> Vec<usize> {
    let mut results = Vec::new();

    if data.len() < 32 {
        return scalar_scan_f32(data, target, alignment);
    }

    let target_vec = _mm256_set1_ps(target);

    let chunks = data.len() / 32;

    for chunk_idx in 0..chunks {
        let chunk_offset = chunk_idx * 32;

        let data_vec = _mm256_loadu_ps(data.as_ptr().add(chunk_offset) as *const f32);

        let cmp_result = _mm256_cmp_ps(data_vec, target_vec, _CMP_EQ_OQ);

        let mask = _mm256_movemask_ps(cmp_result);

        if mask != 0 {
            for i in 0..8 {
                if (mask & (1 << i)) != 0 {
                    let addr = chunk_offset + i * 4;
                    if addr % alignment == 0 {
                        results.push(addr);
                    }
                }
            }
        }
    }

    // Handle remaining bytes
    let mut offset = chunks * 32;
    while offset + 4 <= data.len() {
        if offset % alignment == 0 {
            let value = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            if value == target {
                results.push(offset);
            }
        }
        offset += alignment;
    }

    results
}

/// Scalar fallback for i32 scanning
pub fn scalar_scan_i32(data: &[u8], target: i32, alignment: usize) -> Vec<usize> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset + 4 <= data.len() {
        if offset % alignment == 0 {
            let value = i32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            if value == target {
                results.push(offset);
            }
        }
        offset += alignment;
    }

    results
}

/// Scalar fallback for f32 scanning
pub fn scalar_scan_f32(data: &[u8], target: f32, alignment: usize) -> Vec<usize> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset + 4 <= data.len() {
        if offset % alignment == 0 {
            let value = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            if value == target {
                results.push(offset);
            }
        }
        offset += alignment;
    }

    results
}

/// Auto-dispatching SIMD scan for i32
/// Automatically uses AVX2 if available, falls back to scalar
pub fn scan_i32(data: &[u8], target: i32, alignment: usize) -> Vec<usize> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { simd_scan_i32_avx2(data, target, alignment) }
        } else {
            scalar_scan_i32(data, target, alignment)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        scalar_scan_i32(data, target, alignment)
    }
}

/// Auto-dispatching SIMD scan for f32
pub fn scan_f32(data: &[u8], target: f32, alignment: usize) -> Vec<usize> {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { simd_scan_f32_avx2(data, target, alignment) }
        } else {
            scalar_scan_f32(data, target, alignment)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        scalar_scan_f32(data, target, alignment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_scan_i32() {
        let target = 42i32;
        let mut data = vec![0u8; 1024];

        // Insert target values at specific offsets
        let target_bytes = target.to_le_bytes();
        data[0..4].copy_from_slice(&target_bytes);
        data[100..104].copy_from_slice(&target_bytes);
        data[500..504].copy_from_slice(&target_bytes);

        let results = scalar_scan_i32(&data, target, 4);

        assert!(results.contains(&0));
        assert!(results.contains(&100));
        assert!(results.contains(&500));
    }

    #[test]
    fn test_scan_i32_dispatcher() {
        let target = 12345i32;
        let mut data = vec![0u8; 2048];

        let target_bytes = target.to_le_bytes();
        data[0..4].copy_from_slice(&target_bytes);
        data[256..260].copy_from_slice(&target_bytes);

        let results = scan_i32(&data, target, 4);

        assert!(results.contains(&0));
        assert!(results.contains(&256));
    }
}
