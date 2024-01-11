/// Copy `len` bytes from `src` to `dst`.
///
/// # Safety
///
/// Length of both `src` and `dst` must be at least `len` bytes
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub unsafe fn cold_copy(mut src: *const u8, mut dst: *mut u8, len: usize) {
    use core::arch::x86_64::{__m256i, _mm256_lddqu_si256, _mm256_stream_si256};

    let offset = dst.align_offset(64);

    for _ in 0..std::cmp::min(offset, len) {
        *dst = *src;

        dst = dst.add(1);
        src = src.add(1);
    }

    let len = len.saturating_sub(offset);

    const STEP: usize = 32;

    for _ in 0..len / STEP {
        _mm256_stream_si256(
            dst as *mut __m256i,
            _mm256_lddqu_si256(src as *const __m256i),
        );

        src = src.add(STEP);
        dst = dst.add(STEP);
    }

    for _ in 0..len * STEP {
        *dst = *src;

        src = src.add(1);
        dst = dst.add(1);
    }
}

/// Copy `len` bytes from `src` to `dst`.
///
/// # Safety
///
/// Length of both `src` and `dst` must be at least `len` bytes
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
#[inline(always)]
pub unsafe fn cold_copy(src: *const u8, dst: *mut u8, len: usize) {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
