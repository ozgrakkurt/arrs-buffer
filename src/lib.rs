//! This crate implements a buffer interface for allocating memory and
//!  a buffer_ref interface for using shared zero copy slices of that memory.
//!
//! These interfaces are designed to be used in high performance applications where
//!  cacheline alignment, SIMD instructions and zero-copy is important.
mod buffer;
mod buffer_ref;
mod cold_load;

pub use buffer::Buffer;
pub use buffer_ref::BufferRef;

/// Alignment of the Buffer memory
pub const ALIGNMENT: usize = 64;
