use std::arch::x86_64::*;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

#[inline]
pub fn memset(slice: &mut [u8], byte: u8) {
    type FnSig = fn(&mut [u8], u8);

    #[inline]
    fn get_impl() -> &'static AtomicUsize {
        static mut IMPL: FnSig = delegate;
        unsafe { &*(ptr::addr_of_mut!(IMPL) as *const AtomicUsize) }
    }

    fn delegate(slice: &mut [u8], byte: u8) {
        let actual = if std::is_x86_feature_detected!("avx512f") && cfg!(feature = "nightly") {
            #[cfg(feature = "nightly")]
            {
                memset_avx512f as usize
            }
            #[cfg(not(feature = "nightly"))]
            unreachable!()
        } else if std::is_x86_feature_detected!("avx") {
            memset_avx as usize
        } else if std::is_x86_feature_detected!("sse2") {
            memset_sse2 as usize
        } else {
            super::fallback as usize
        };

        get_impl().store(actual, Ordering::Relaxed);

        unsafe { mem::transmute::<usize, FnSig>(actual)(slice, byte) }
    }

    let ptr = get_impl().load(Ordering::Relaxed);
    let ptr = unsafe { mem::transmute::<usize, FnSig>(ptr) };
    ptr(slice, byte);

    // Required before returning to code that may set atomic flags that invite concurrent reads,
    // as LLVM will not lower `atomic store ... release`, thus `AtomicBool::store(true, Release)`
    // on x86-64 to emit SFENCE, even though it is required in the presence of nontemporal stores.
    unsafe { _mm_sfence() };
}

#[repr(packed)]
struct UnalignedI32(i32);

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
unsafe fn memset_avx512f(slice: &mut [u8], byte: u8) {
    let (prefix, body, suffix) = slice.align_to_mut::<__m512i>();
    memset_small(prefix, byte);
    let pattern = _mm512_set1_epi8(byte as i8);
    let mut i = 0;
    let ptr = body.as_mut_ptr();
    let len = body.len() as isize;
    while i + 3 < len {
        _mm512_stream_si512(ptr.offset(i + 0).cast(), pattern);
        _mm512_stream_si512(ptr.offset(i + 1).cast(), pattern);
        _mm512_stream_si512(ptr.offset(i + 2).cast(), pattern);
        _mm512_stream_si512(ptr.offset(i + 3).cast(), pattern);
        i += 4;
    }
    while i < len {
        _mm512_stream_si512(ptr.offset(i).cast(), pattern);
        i += 1;
    }
    memset_small(suffix, byte);
}

#[target_feature(enable = "avx")]
unsafe fn memset_avx(slice: &mut [u8], byte: u8) {
    let (prefix, body, suffix) = slice.align_to_mut::<__m256i>();
    memset_small(prefix, byte);
    let pattern = _mm256_set1_epi8(byte as i8);
    let mut i = 0;
    while i + 3 < body.len() {
        _mm256_stream_si256(body.get_unchecked_mut(i + 0), pattern);
        _mm256_stream_si256(body.get_unchecked_mut(i + 1), pattern);
        _mm256_stream_si256(body.get_unchecked_mut(i + 2), pattern);
        _mm256_stream_si256(body.get_unchecked_mut(i + 3), pattern);
        i += 4;
    }
    while i < body.len() {
        _mm256_stream_si256(&mut body[i], pattern);
        i += 1;
    }
    memset_small(suffix, byte);
}

unsafe fn memset_small(slice: &mut [u8], byte: u8) {
    let (prefix, body, suffix) = slice.align_to_mut::<UnalignedI32>();
    for slot in prefix {
        *slot = byte;
    }
    let pat = i32::from(byte);
    let pat = pat | (pat << 8);
    let pat = pat | (pat << 16);
    for slot in body {
        _mm_stream_si32(ptr::addr_of_mut!(slot.0), pat);
    }
    for slot in suffix {
        *slot = byte;
    }
}

#[target_feature(enable = "sse2")]
unsafe fn memset_sse2(slice: &mut [u8], byte: u8) {
    let (prefix, body, suffix) = slice.align_to_mut::<__m128i>();
    super::fallback(prefix, byte);
    let pattern = _mm_set1_epi8(byte as i8);
    let mut i = 0;
    while i + 3 < body.len() {
        _mm_stream_si128(body.get_unchecked_mut(i + 0), pattern);
        _mm_stream_si128(body.get_unchecked_mut(i + 1), pattern);
        _mm_stream_si128(body.get_unchecked_mut(i + 2), pattern);
        _mm_stream_si128(body.get_unchecked_mut(i + 3), pattern);
        i += 4;
    }
    while i < body.len() {
        _mm_stream_si128(&mut body[i], pattern);
        i += 1;
    }
    super::fallback(suffix, byte);
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "nightly")]
    fn avx512f() {
        if std::is_x86_feature_detected!("avx512f") {
            crate::tests::test_impl(|a, b| unsafe { super::memset_avx512f(a, b) })
        }
    }

    #[test]
    fn avx() {
        if std::is_x86_feature_detected!("avx") {
            crate::tests::test_impl(|a, b| unsafe { super::memset_avx(a, b) })
        }
    }

    #[test]
    fn sse2() {
        if std::is_x86_feature_detected!("sse2") {
            crate::tests::test_impl(|a, b| unsafe { super::memset_sse2(a, b) })
        }
    }
}
