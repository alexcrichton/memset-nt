#![no_main]

use libfuzzer_sys::fuzz_target;
use std::alloc::{self, Layout};

#[derive(arbitrary::Arbitrary, Debug)]
struct Input {
    byte: u8,
    misalignment: u8,
    size: u16,
}

fuzz_target!(|data: Input| {
    unsafe {
        let layout =
            Layout::from_size_align(data.size as usize + usize::from(data.misalignment), 256)
                .unwrap();
        let ptr = alloc::alloc(layout);
        assert!(!ptr.is_null());
        let slice = std::slice::from_raw_parts_mut(
            ptr.offset(data.misalignment.into()),
            data.size as usize,
        );

        memset_nt::memset(slice, data.byte);

        for b in slice {
            assert_eq!(*b, data.byte);
        }

        alloc::dealloc(ptr, layout);
    }
});
