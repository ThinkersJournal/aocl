//! LAPACKE passes `lapack_complex_float` / `lapack_complex_double` BY VALUE (`float _Complex` / `double _Complex` in the header;
//! bindgen declares them as `__BindgenComplex<f32|f64>`, an 8- / 16-byte `repr(C)` struct, align 4 / 8). On the Windows x64 ABI
//! an 8-byte aggregate travels in one integer register and a 16-byte one by hidden reference, for `_Complex` and for a plain
//! struct alike, so the Rust declaration should agree with the library. This test CALLS the library with by-value complex
//! arguments and checks the results (a layout/ABI mismatch would produce wrong numbers or fault, as the sparse crate's
//! `aoclsparse_zset_value` did). Layout of the type itself: `src/lib.rs` has no const assert because bindgen generates the
//! generic `__BindgenComplex<T>` (size 2*sizeof(T), align alignof(T)); asserted here instead.
#![allow(warnings)]
use aocl_lapack_sys::*;
use core::mem::{align_of, size_of};

const COL: i32 = 102;

#[test]
fn bindgen_complex_layout_is_the_c_complex_layout() {
    // C: `float _Complex` = 8 bytes align 4; `double _Complex` = 16 bytes align 8 (clang 22 -x c, Windows x64, measured by
    // docs/SYS-LAYOUT-AUDIT-20261127.md); `std::complex<T>` (LAPACK_COMPLEX_CPP) has the same size and alignment.
    assert_eq!(
        (
            size_of::<__BindgenComplex<f32>>(),
            align_of::<__BindgenComplex<f32>>()
        ),
        (8, 4)
    );
    assert_eq!(
        (
            size_of::<__BindgenComplex<f64>>(),
            align_of::<__BindgenComplex<f64>>()
        ),
        (16, 8)
    );
}

#[test]
fn zlaset_takes_two_double_complex_by_value() {
    let alpha = __BindgenComplex {
        re: 1.5f64,
        im: -2.5,
    };
    let beta = __BindgenComplex {
        re: 3.0f64,
        im: 4.0,
    };
    let mut a = [__BindgenComplex {
        re: 9.0f64,
        im: 9.0,
    }; 9];
    let info = unsafe {
        LAPACKE_zlaset(
            COL as _,
            b'A' as _,
            3 as _,
            3 as _,
            alpha,
            beta,
            a.as_mut_ptr(),
            3 as _,
        )
    };
    assert_eq!(info, 0);
    for j in 0..3 {
        for i in 0..3 {
            let want = if i == j { beta } else { alpha };
            assert_eq!(a[i + 3 * j], want, "element ({i},{j})");
        }
    }
}

#[test]
fn claset_takes_two_float_complex_by_value() {
    let alpha = __BindgenComplex {
        re: 1.5f32,
        im: -2.5,
    };
    let beta = __BindgenComplex {
        re: 3.0f32,
        im: 4.0,
    };
    let mut a = [__BindgenComplex {
        re: 9.0f32,
        im: 9.0,
    }; 9];
    let info = unsafe {
        LAPACKE_claset(
            COL as _,
            b'A' as _,
            3 as _,
            3 as _,
            alpha,
            beta,
            a.as_mut_ptr(),
            3 as _,
        )
    };
    assert_eq!(info, 0);
    for j in 0..3 {
        for i in 0..3 {
            let want = if i == j { beta } else { alpha };
            assert_eq!(a[i + 3 * j], want, "element ({i},{j})");
        }
    }
}

#[test]
fn zsyr_uses_its_double_complex_alpha() {
    // A := alpha * x * x^T + A, upper triangle, A = 0, x = (1, 2), alpha = 2 + 3i.
    let alpha = __BindgenComplex {
        re: 2.0f64,
        im: 3.0,
    };
    let x = [
        __BindgenComplex {
            re: 1.0f64,
            im: 0.0,
        },
        __BindgenComplex {
            re: 2.0f64,
            im: 0.0,
        },
    ];
    let mut a = [__BindgenComplex {
        re: 0.0f64,
        im: 0.0,
    }; 4];
    let info = unsafe {
        LAPACKE_zsyr(
            COL as _,
            b'U' as _,
            2 as _,
            alpha,
            x.as_ptr(),
            1 as _,
            a.as_mut_ptr(),
            2 as _,
        )
    };
    assert_eq!(info, 0);
    assert_eq!(a[0], __BindgenComplex { re: 2.0, im: 3.0 }); // alpha * 1
    assert_eq!(a[2], __BindgenComplex { re: 4.0, im: 6.0 }); // alpha * 2  (row 0, col 1)
    assert_eq!(a[3], __BindgenComplex { re: 8.0, im: 12.0 }); // alpha * 4
}

#[test]
fn csyr_uses_its_float_complex_alpha() {
    let alpha = __BindgenComplex {
        re: 2.0f32,
        im: 3.0,
    };
    let x = [
        __BindgenComplex {
            re: 1.0f32,
            im: 0.0,
        },
        __BindgenComplex {
            re: 2.0f32,
            im: 0.0,
        },
    ];
    let mut a = [__BindgenComplex {
        re: 0.0f32,
        im: 0.0,
    }; 4];
    let info = unsafe {
        LAPACKE_csyr(
            COL as _,
            b'U' as _,
            2 as _,
            alpha,
            x.as_ptr(),
            1 as _,
            a.as_mut_ptr(),
            2 as _,
        )
    };
    assert_eq!(info, 0);
    assert_eq!(a[0], __BindgenComplex { re: 2.0, im: 3.0 });
    assert_eq!(a[2], __BindgenComplex { re: 4.0, im: 6.0 });
    assert_eq!(a[3], __BindgenComplex { re: 8.0, im: 12.0 });
}

#[test]
fn complex_by_value_return() {
    // `lapack_make_complex_*` take two reals and RETURN the complex by value.
    let z = unsafe { lapack_make_complex_double(1.25, -7.5) };
    assert_eq!(z, __BindgenComplex { re: 1.25, im: -7.5 });
    let c = unsafe { lapack_make_complex_float(1.25, -7.5) };
    assert_eq!(
        c,
        __BindgenComplex {
            re: 1.25f32,
            im: -7.5
        }
    );
}
