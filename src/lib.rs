#![cfg_attr(feature = "nightly", feature(stdsimd, avx512_target_feature))]

#[inline]
pub fn memset(slice: &mut [u8], byte: u8) {
    arch::memset(slice, byte);
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        use x86_64 as arch;
    } else {
        mod arch {
            pub(crate) use super::fallback as memset;
        }
    }
}

fn fallback(slice: &mut [u8], byte: u8) {
    unsafe {
        std::ptr::write_bytes(slice.as_mut_ptr(), byte, slice.len());
    }
}

#[cfg(test)]
mod tests {
    use super::memset;

    #[test]
    fn empty() {
        memset(&mut [], 0);
        let mut arr = [];
        memset(&mut arr, 0);
    }

    #[test]
    fn smoke() {
        test_impl(super::memset);
    }

    #[test]
    fn fallback() {
        test_impl(super::fallback);
    }

    pub fn test_impl(f: fn(&mut [u8], u8)) {
        f(&mut [], 0);
        let mut arr = [0; 128];
        for i in 0..arr.len() {
            f(&mut arr[..i], i as u8);
            for b in arr.iter().take(i) {
                assert_eq!(*b, i as u8);
            }
        }
    }
}
