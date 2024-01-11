use std::sync::Arc;

use crate::Buffer;

/// An immutable reference to a buffer. Can be used for shared zero copy views
///  into a single buffer.
#[derive(Clone)]
pub struct BufferRef {
    inner: Arc<Buffer>,
    start: usize,
    len: usize,
}

impl BufferRef {
    /// Creates a shared reference to given buffer.
    ///
    /// # Panics
    ///
    /// Panics if start + len overflows or if start + len is greater than inner.len().
    pub fn new(inner: Arc<Buffer>, start: usize, len: usize) -> Self {
        assert!(start.checked_add(len).unwrap() <= inner.len());

        Self { inner, start, len }
    }

    /// Slices into this ref.
    ///
    /// # Panics
    ///
    /// Panics if start + len overflows or if start + len is greater than self.len().
    /// Or if self.start + start overflows.
    pub fn slice(&self, start: usize, len: usize) -> Self {
        assert!(start.checked_add(len).unwrap() <= self.len);

        Self {
            inner: self.inner.clone(),
            start: self.start.checked_add(start).unwrap(),
            len,
        }
    }

    /// Length of the reference
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns if length of buffer is zero
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Length of the underlying buffer
    pub fn inner_len(&self) -> usize {
        self.inner.len()
    }

    /// Start index of this reference relative to the inner buffer.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Return a pointer to the underlying memory at offset
    pub fn as_ptr(&self) -> *const u8 {
        unsafe { self.inner.as_ptr().add(self.start) }
    }

    /// Get a slice
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_buffer() {
        unsafe {
            let mut buffer = Buffer::new(32);
            *buffer.as_mut_ptr().add(17) = 69;

            let buf_ref = buffer.into_ref();
            let buf_ref = buf_ref.slice(16, 3);

            assert_eq!(*buf_ref.as_ptr().add(1), 69);

            assert_eq!(buf_ref.as_slice()[1], 69);
        }
    }
}
