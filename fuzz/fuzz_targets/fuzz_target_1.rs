#![no_main]

use libfuzzer_sys::fuzz_target;

use arrs_buffer::cold_copy;

fuzz_target!(|src: &[u8]| {
    let len = src.len();

    let mut dst = vec![0; len];

    unsafe {
        cold_copy(src.as_ptr(), dst.as_mut_ptr(), len);
    }
    
    assert_eq!(src, &dst);
});
