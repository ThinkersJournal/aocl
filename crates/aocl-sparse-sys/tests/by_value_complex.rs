//! `aoclsparse_double_complex` / `aoclsparse_float_complex` are passed BY VALUE (alpha, beta, omega, val). The C type is
//! `alignas(2 * sizeof(double))` (16) / `alignas(2 * sizeof(float))` (8) (aoclsparse_types.h; the header defines `alignas`
//! away in pure C, so bindgen, which parses it as C, produced an 8-aligned Rust struct and the by-value `z*` calls faulted:
//! `aoclsparse_zset_value` raised an access violation). These tests call every kind of by-value path on the real library
//! and check the VALUE that arrives, one test per call so a crash in one names itself (run them one by one with
//! `cargo test -p aocl-sparse-sys --test by_value_complex -- --exact <name>` to see which would crash).
//! The compile-time guard is in `src/lib.rs` (size_of / align_of asserts against the header's alignas).
use aocl_sparse_sys::*;
use std::ptr::null_mut;

type Z = aoclsparse_double_complex;
fn z(re: f64, im: f64) -> Z {
    Z { real: re, imag: im }
}
fn mul(a: Z, b: Z) -> Z {
    z(
        a.real * b.real - a.imag * b.imag,
        a.real * b.imag + a.imag * b.real,
    )
}
fn add(a: Z, b: Z) -> Z {
    z(a.real + b.real, a.imag + b.imag)
}
fn div(a: Z, b: Z) -> Z {
    let d = b.real * b.real + b.imag * b.imag;
    z(
        (a.real * b.real + a.imag * b.imag) / d,
        (a.imag * b.real - a.real * b.imag) / d,
    )
}
fn near(a: Z, b: Z) -> bool {
    (a.real - b.real).abs() < 1e-12 && (a.imag - b.imag).abs() < 1e-12
}
const OK: aoclsparse_status = 0;

struct Csr {
    mat: aoclsparse_matrix,
    descr: aoclsparse_mat_descr,
    _row_ptr: Vec<aoclsparse_int>,
    _col: Vec<aoclsparse_int>,
    _val: Vec<Z>,
}
impl Drop for Csr {
    fn drop(&mut self) {
        unsafe {
            aoclsparse_destroy(&mut self.mat);
            aoclsparse_destroy_mat_descr(self.descr);
        }
    }
}
/// 2 x 2 lower-triangular diagonal matrix diag(2, 4) as complex CSR (zero-based), general or triangular descriptor.
fn diag24(ty: aoclsparse_matrix_type) -> Csr {
    let mut row_ptr: Vec<aoclsparse_int> = vec![0, 1, 2];
    let mut col: Vec<aoclsparse_int> = vec![0, 1];
    let mut val = vec![z(2.0, 0.0), z(4.0, 0.0)];
    let mut mat: aoclsparse_matrix = null_mut();
    let mut descr: aoclsparse_mat_descr = null_mut();
    unsafe {
        assert_eq!(
            aoclsparse_create_zcsr(
                &mut mat,
                aoclsparse_index_base__aoclsparse_index_base_zero,
                2,
                2,
                2,
                row_ptr.as_mut_ptr(),
                col.as_mut_ptr(),
                val.as_mut_ptr()
            ),
            OK
        );
        assert_eq!(aoclsparse_create_mat_descr(&mut descr), OK);
        assert_eq!(aoclsparse_set_mat_type(descr, ty), OK);
        assert_eq!(
            aoclsparse_set_mat_fill_mode(descr, aoclsparse_fill_mode__aoclsparse_fill_mode_lower),
            OK
        );
        assert_eq!(
            aoclsparse_set_mat_diag_type(
                descr,
                aoclsparse_diag_type__aoclsparse_diag_type_non_unit
            ),
            OK
        );
    }
    Csr {
        mat,
        descr,
        _row_ptr: row_ptr,
        _col: col,
        _val: val,
    }
}

#[test]
fn zset_value_by_value() {
    let a = diag24(aoclsparse_matrix_type__aoclsparse_matrix_type_general);
    unsafe {
        assert_eq!(aoclsparse_zset_value(a.mat, 1, 1, z(7.5, -2.5)), OK);
        // read it back through the library: C = A * e1 (column 1 of A is (0, 7.5-2.5i))
        let b = [z(0.0, 0.0), z(1.0, 0.0)];
        let mut c = [z(9.0, 9.0), z(9.0, 9.0)];
        assert_eq!(
            aoclsparse_zcsrmm(
                aoclsparse_operation__aoclsparse_operation_none,
                z(1.0, 0.0),
                a.mat,
                a.descr,
                aoclsparse_order__aoclsparse_order_column,
                b.as_ptr(),
                1,
                2,
                z(0.0, 0.0),
                c.as_mut_ptr(),
                2
            ),
            OK
        );
        assert!(
            near(c[1], z(7.5, -2.5)),
            "A[1][1] after set_value: {:?}",
            (c[1].real, c[1].imag)
        );
        assert!(near(c[0], z(0.0, 0.0)));
    }
}

#[test]
fn ztrsv_alpha_by_value() {
    let a = diag24(aoclsparse_matrix_type__aoclsparse_matrix_type_triangular);
    let alpha = z(2.0, 1.0);
    let b = [z(1.0, 2.0), z(3.0, -1.0)];
    let mut x = [z(0.0, 0.0); 2];
    unsafe {
        assert_eq!(
            aoclsparse_ztrsv(
                aoclsparse_operation__aoclsparse_operation_none,
                alpha,
                a.mat,
                a.descr,
                b.as_ptr(),
                x.as_mut_ptr()
            ),
            OK
        );
    }
    // A x = alpha b with A = diag(2, 4)
    assert!(
        near(x[0], div(mul(alpha, b[0]), z(2.0, 0.0))),
        "{:?}",
        (x[0].real, x[0].imag)
    );
    assert!(
        near(x[1], div(mul(alpha, b[1]), z(4.0, 0.0))),
        "{:?}",
        (x[1].real, x[1].imag)
    );
}

#[test]
fn zcsrmm_alpha_and_beta_by_value() {
    let a = diag24(aoclsparse_matrix_type__aoclsparse_matrix_type_general);
    let (alpha, beta) = (z(2.0, 1.0), z(0.5, -1.0));
    let b = [z(1.0, 2.0), z(3.0, -1.0)];
    let c0 = [z(1.0, 1.0), z(2.0, 0.0)];
    let mut c = c0;
    unsafe {
        assert_eq!(
            aoclsparse_zcsrmm(
                aoclsparse_operation__aoclsparse_operation_none,
                alpha,
                a.mat,
                a.descr,
                aoclsparse_order__aoclsparse_order_column,
                b.as_ptr(),
                1,
                2,
                beta,
                c.as_mut_ptr(),
                2
            ),
            OK
        );
    }
    // C = alpha * A * B + beta * C0 with A = diag(2, 4)
    let d = [z(2.0, 0.0), z(4.0, 0.0)];
    for i in 0..2 {
        let want = add(mul(alpha, mul(d[i], b[i])), mul(beta, c0[i]));
        assert!(
            near(c[i], want),
            "row {i}: {:?} != {:?}",
            (c[i].real, c[i].imag),
            (want.real, want.imag)
        );
    }
}

#[test]
fn zdotmv_alpha_and_beta_by_value() {
    let a = diag24(aoclsparse_matrix_type__aoclsparse_matrix_type_general);
    let (alpha, beta) = (z(2.0, 1.0), z(0.5, -1.0));
    let x = [z(1.0, 2.0), z(3.0, -1.0)];
    let mut y = [z(1.0, 1.0), z(2.0, 0.0)];
    let mut d = z(0.25, 0.5);
    unsafe {
        let st = aoclsparse_zdotmv(
            aoclsparse_operation__aoclsparse_operation_none,
            alpha,
            a.mat,
            a.descr,
            x.as_ptr(),
            beta,
            y.as_mut_ptr(),
            &mut d,
        );
        assert_eq!(st, OK);
    }
    // y := alpha * A * x + beta * y (documented); d = x^T y (not asserted: conjugation convention).
    let dd = [z(2.0, 0.0), z(4.0, 0.0)];
    let y0 = [z(1.0, 1.0), z(2.0, 0.0)];
    for i in 0..2 {
        let want = add(mul(alpha, mul(dd[i], x[i])), mul(beta, y0[i]));
        assert!(
            near(y[i], want),
            "row {i}: {:?} != {:?}",
            (y[i].real, y[i].imag),
            (want.real, want.imag)
        );
    }
}

#[test]
fn zsymgs_alpha_by_value_runs() {
    let a = diag24(aoclsparse_matrix_type__aoclsparse_matrix_type_symmetric);
    let alpha = z(1.0, 0.0);
    let b = [z(2.0, 0.0), z(8.0, 0.0)];
    let mut x = [z(0.0, 0.0); 2];
    unsafe {
        assert_eq!(
            aoclsparse_zsymgs(
                aoclsparse_operation__aoclsparse_operation_none,
                a.mat,
                a.descr,
                alpha,
                b.as_ptr(),
                x.as_mut_ptr()
            ),
            OK
        );
    }
    // diagonal system: one Gauss-Seidel sweep from x = 0 solves it exactly: x = b / diag
    assert!(
        near(x[0], z(1.0, 0.0)) && near(x[1], z(2.0, 0.0)),
        "{:?}",
        (x[0].real, x[0].imag, x[1].real, x[1].imag)
    );
}
