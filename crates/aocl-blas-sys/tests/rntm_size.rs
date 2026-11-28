//! `rntm_t` size follows the BLIS integer type on Windows (`bool` is `typedef gint_t bool` in C, `dim_t` is `gint_t`): measured with clang on
//! AOCL 6.0.68's blis.h, `gint_t` = 8 gives 192 bytes and, for the same header text compiled with `-DBLIS_INT_TYPE_SIZE=32
//! -DBLIS_BLAS_INT_SIZE=32`, `gint_t` = 4 gives 128 bytes, alignment 64 in both. This checks the installed headers' bindgen output against
//! that table (the crate's build asserts only the alignment and a multiple of it, so a BLIS with another field set still compiles).
#![cfg(windows)]
use aocl_blas_sys::{gint_t, rntm_t};
use core::mem::{align_of, size_of};

#[test]
fn rntm_t_size_follows_gint_t() {
    assert_eq!(align_of::<rntm_t>(), 64);
    let want = match size_of::<gint_t>() {
        8 => 192,
        4 => 128,
        n => panic!("unmeasured gint_t size {n}"),
    };
    assert_eq!(
        size_of::<rntm_t>(),
        want,
        "gint_t is {} bytes",
        size_of::<gint_t>()
    );
}
