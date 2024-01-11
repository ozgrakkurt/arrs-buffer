#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub unsafe fn cold_copy(mut src: *const u8, mut dst: *mut u8, len: usize) {
    use core::arch::x86_64::{__m256i, _mm256_lddqu_si256, _mm256_stream_si256};

    const STEP: usize = 32;

    for _ in 0..len / STEP {
        _mm256_stream_si256(
            dst as *mut __m256i,
            _mm256_lddqu_si256(src as *const __m256i),
        );

        src = src.add(STEP);
        dst = dst.add(STEP);
    }

    for _ in 0..len % STEP {
        *dst = *src;

        src = src.add(1);
        dst = dst.add(1);
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
#[inline(always)]
pub unsafe fn cold_copy(src: *const u8, dst: *mut u8, len: usize) {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
