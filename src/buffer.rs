use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr::NonNull,
    sync::Arc,
};

use crate::{BufferRef, ALIGNMENT};

/// Buffer is a mutable byte container that is aligned to [crate::ALIGNMENT] bytes,
/// and has a padding so the size of the underlying allocation is a multiple of [crate::ALIGNMENT].
///
/// The alignment is because we want it to be aligned to the cacheline. The padding is because we
/// want it to be easily usable with SIMD instructions without checking length.
pub struct Buffer {
    ptr: NonNull<u8>,
    layout: Layout,
    len: usize,
}

impl Buffer {
    /// Create a new buffer. This function will allocate a padded and aligned memory chunk that
    ///  is able to hold the given len number of bytes. The allocated memory will be zeroed.
    ///
    /// Doesn't allocate if `len` is zero. In this case the buffer will have a dangling pointer.
    ///
    /// # Panics
    ///
    /// Panics if `len` overflows `isize` when padded to [crate::ALIGNMENT] bytes. Or if memory
    ///  can't be allocated.
    pub fn new(len: usize) -> Self {
        // Don't allocate if len is 0
        if len == 0 {
            return Self {
                ptr: std::ptr::NonNull::dangling(),
                layout: Layout::from_size_align(64, 64).unwrap(),
                len,
            };
        }

        let padded_len = len.checked_next_multiple_of(ALIGNMENT).unwrap();
        let layout = Layout::from_size_align(padded_len, ALIGNMENT).unwrap();

        let ptr = unsafe { alloc_zeroed(layout) };

        let ptr = NonNull::new(ptr).unwrap();

        Self { ptr, layout, len }
    }

    /// Get a pointer to the underlying memory.
    ///
    /// The underlying memory is aligned to [crate::ALIGNMENT] bytes and padded to [crate::ALIGNMENT] bytes.
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// Get a mutable pointer to the underlying memory.
    ///
    /// The underlying memory is aligned to [crate::ALIGNMENT] bytes and padded to [crate::ALIGNMENT] bytes.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Get a slice to the underlying memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    /// Get a mutable slice to the underlying memory
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// Loads data from `src` to the underlying memory. Lenghth of `self` must be greater than or equal to the length of `src`
    ///
    /// This might be faster than the regular memcopy, especially for large copies. Because it bypasses the
    /// CPU cache when writing if possible. This is only advantageus if the user won't be reading the memory
    ///  stored in `self`, for example it can be good when reading bulk data from disk but not so much if
    ///  summing small/medium (fits in CPU cache) sized integer arrays and then doing other computation with them.
    ///
    /// # Panics
    ///
    /// Panics if self.len() < src.len()
    pub fn cold_load(&mut self, src: &[u8]) {
        assert!(self.len >= src.len());

        unsafe { crate::cold_load::cold_copy(src.as_ptr(), self.as_mut_ptr(), src.len()) }
    }

    /// Length of the buffer. Keep in mind that the underlying memory is padded to [crate::ALIGNMENT] bytes
    /// so might be bigger than the returned value.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns if length of buffer is zero
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a Buffer from given slice.
    pub fn from_slice(src: &[u8]) -> Self {
        // Has to be mut because we write to it with buf.ptr
        #[allow(unused_mut)]
        let mut buf = Self::new(src.len());

        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), buf.as_mut_ptr(), buf.len);
        }

        buf
    }

    /// Similar to [Self::from_slice] but bypasses CPU cache for writes if possible.
    ///
    /// See [Self::cold_load] for tradeoffs.
    pub fn from_slice_cold(src: &[u8]) -> Self {
        let mut buf = Self::new(src.len());
        buf.cold_load(src);
        buf
    }

    /// Convert this buffer into a reference for zero copy shared usage.
    pub fn into_ref(self) -> BufferRef {
        let len = self.len;
        BufferRef::new(Arc::new(self), 0, len)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            // Don't dealloc if we have 0 len, because we didn't alloc at the start.
            if self.len > 0 {
                dealloc(self.as_mut_ptr(), self.layout);
            }
        }
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        unsafe {
            // Has to be mut because we write to it with other.ptr
            #[allow(unused_mut)]
            let mut other = Self::new(self.len);

            std::ptr::copy_nonoverlapping(self.as_ptr(), other.as_mut_ptr(), self.len);
            other
        }
    }
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_sized() {
        let _buf = Buffer::new(0);
    }

    #[test]
    fn big_sized() {
        let _buf = Buffer::new(1331);
    }

    #[test]
    fn cold_copy() {
        let src = &[1, 2, 3];
        let mut buf = Buffer::from_slice_cold(src);
        assert_eq!(buf.as_slice(), src);

        let src = &[5, 4];
        buf.cold_load(src);
        assert_eq!(buf.as_slice(), &[5, 4, 3]);

        let src = (0..244).collect::<Vec<u8>>();
        let buf = Buffer::from_slice_cold(&src);
        assert_eq!(buf.as_slice(), src);
    }
}
