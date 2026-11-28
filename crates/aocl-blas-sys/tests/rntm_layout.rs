//! `rntm_t` (BLIS runtime object): the header declares it `BLIS_ATTRIB_ALIGN` (`__attribute__((aligned(64)))`; on `_WIN32` ONLY
//! when the compiler is clang), and `bool` is `typedef gint_t bool` in C. bindgen (clang, C) gives it `repr(align(64))` with
//! `bool_` = `gint_t` fields. This test has the LIBRARY write a `rntm_t` (`bli_rntm_init_from_global`) into a buffer poisoned
//! with 0x5A and checks that every field, at the offsets the Rust struct uses, was overwritten by the library with a plausible
//! value: a library built with another layout (1-byte bools, another alignment) would leave poison where Rust looks.
//! Needs `--features static-link` (`bli_rntm_init_from_global` is static-library-only on AOCL 6.0.68 Windows).
#![cfg(feature = "static-link")]
#![allow(warnings)]
use aocl_blas_sys::*;
use core::mem::{align_of, size_of, MaybeUninit};

#[test]
fn rntm_t_layout_matches_library() {
    assert_eq!(align_of::<rntm_t>(), 64);
    let mut raw = MaybeUninit::<rntm_t>::uninit();
    unsafe {
        core::ptr::write_bytes(raw.as_mut_ptr() as *mut u8, 0x5A, size_of::<rntm_t>());
        bli_rntm_init_from_global(raw.as_mut_ptr());
    }
    let bytes =
        unsafe { core::slice::from_raw_parts(raw.as_ptr() as *const u8, size_of::<rntm_t>()) };
    eprintln!(
        "rntm_t size {} align {}: {:02x?}",
        size_of::<rntm_t>(),
        align_of::<rntm_t>(),
        bytes
    );
    let r = unsafe { raw.assume_init() };
    let poison_gint = gint_t::from_ne_bytes([0x5A; core::mem::size_of::<gint_t>()]);
    let small = |v: gint_t| (0..=1).contains(&v);
    assert!(
        small(r.auto_factor as gint_t),
        "auto_factor {:#x}",
        r.auto_factor as i128
    );
    assert!(
        small(r.pack_a as gint_t)
            && small(r.pack_b as gint_t)
            && small(r.l3_sup as gint_t)
            && small(r.blis_mt as gint_t),
        "pack_a/pack_b/l3_sup/blis_mt {:#x} {:#x} {:#x} {:#x}",
        r.pack_a as i128,
        r.pack_b as i128,
        r.l3_sup as i128,
        r.blis_mt as i128
    );
    assert!(
        small(r.stop_on_error as gint_t) && small(r.print_on_error as gint_t),
        "stop_on_error/print_on_error {:#x} {:#x}",
        r.stop_on_error as i128,
        r.print_on_error as i128
    );
    assert!(
        r.num_threads >= -1 && r.num_threads < 4096,
        "num_threads {}",
        r.num_threads
    );
    assert_eq!(
        r.num_threads as i128,
        unsafe { bli_thread_get_num_threads() } as i128
    );
    assert_ne!(
        r.info_value, poison_gint,
        "info_value untouched by the library"
    );
}
