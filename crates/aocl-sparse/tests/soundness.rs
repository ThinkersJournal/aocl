//! SAFE-CODE-ONLY reproductions: each function below is a SAFE `pub fn`; before the fix a call with the arguments shown
//! faulted or corrupted the heap inside the library (Windows, AOCL 6.0.68; Linux unmeasured). After the fix each returns
//! `Err(InvalidArgument)`. There is no `unsafe` in this file. Each case is its own test so a crash names itself; run them
//! one at a time with `-- --exact <name>`.
use aocl_sparse::*;

fn diag2() -> SparseMatrix<f32> {
    SparseMatrix::<f32>::from_csr(IndexBase::Zero, 2, 2, &[0, 1, 2], &[0, 1], &[2.0, 4.0]).unwrap()
}

#[test]
fn sorv_f32_short_x_is_an_error_not_an_out_of_bounds_access() {
    let a = diag2();
    let d = MatDescr::new().unwrap();
    let mut x = [0.0f32; 1]; // matrix is 2 x 2: x must hold 2
    let b = [1.0f32, 2.0];
    let r = sorv_f32(SorType::Forward, &d, &a, 1.0, 1.0, &mut x, &b);
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
}

#[test]
fn sorv_f32_short_b_is_an_error() {
    let a = diag2();
    let d = MatDescr::new().unwrap();
    let mut x = [0.0f32; 2];
    let b = [1.0f32; 1];
    let r = sorv_f32(SorType::Forward, &d, &a, 1.0, 1.0, &mut x, &b);
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
}

#[test]
fn roti_negative_index_is_an_error_not_heap_corruption() {
    let mut x = [1.0f64, 2.0, 3.0];
    let mut y = [10.0f64, 20.0, 30.0];
    let r = roti_f64(&mut x, &[1, -1, 2], &mut y, 0.6, 0.8);
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
    assert_eq!(
        y,
        [10.0, 20.0, 30.0],
        "y must be untouched when the call is rejected"
    );
}

#[test]
fn roti_index_past_the_end_is_an_error() {
    let mut x = [1.0f64, 2.0];
    let mut y = [10.0f64, 20.0, 30.0];
    let r = roti_f64(&mut x, &[0, 3], &mut y, 0.6, 0.8); // y.len() == 3: index 3 is out of range
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
}

#[test]
fn roti_short_index_array_is_an_error() {
    let mut x = [1.0f32, 2.0, 3.0];
    let mut y = [10.0f32, 20.0, 30.0];
    let r = roti_f32(&mut x, &[0, 1], &mut y, 0.6, 0.8); // 3 values but 2 indices
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
}

#[test]
fn roti_valid_call_still_rotates() {
    // control: the validation must not reject a good call. (x, y) at indices [2, 0]: x' = c x + s y, y' = c y - s x.
    let mut x = [1.0f64, 2.0];
    let mut y = [10.0f64, 20.0, 30.0];
    let (c, s) = (0.6, 0.8);
    roti_f64(&mut x, &[2, 0], &mut y, c, s).unwrap();
    let (x0, x1, y2, y0) = (
        c * 1.0 + s * 30.0,
        c * 2.0 + s * 10.0,
        c * 30.0 - s * 1.0,
        c * 10.0 - s * 2.0,
    );
    assert!(
        (x[0] - x0).abs() < 1e-12 && (x[1] - x1).abs() < 1e-12,
        "{x:?}"
    );
    assert!(
        (y[2] - y2).abs() < 1e-12 && (y[0] - y0).abs() < 1e-12 && y[1] == 20.0,
        "{y:?}"
    );
}

// ---- the sibling index-array functions (found in review of #45: only `roti` validated the index RANGE) ----
// Each function below takes an index array into a dense vector. Out-of-range indices wrote (or read) outside the dense vector inside
// the library. Every case is a safe call; it must be refused with `InvalidArgument` and leave the dense vector untouched.
use aocl_sparse::complex as cx;
use aocl_sparse::{Complex32, Complex64};

fn refused<T: core::fmt::Debug>(r: Result<T>) {
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{r:?}");
}

#[test]
fn axpyi_index_past_the_dense_vector_is_refused() {
    let mut y = [10.0f64, 20.0, 30.0];
    refused(axpyi(2.0, &[1.0, 2.0], &[0, 5], &mut y));
    assert_eq!(y, [10.0, 20.0, 30.0]);
    refused(axpyi(2.0, &[1.0, 2.0], &[0, -1], &mut y));
    assert_eq!(y, [10.0, 20.0, 30.0]);
    // control: a legal call still works and is the textbook result
    axpyi(2.0, &[1.0, 2.0], &[0, 2], &mut y).unwrap();
    assert_eq!(y, [12.0, 20.0, 34.0]);
}

#[test]
fn gthr_and_sctr_out_of_range_indices_are_refused() {
    let y = [1.0f64, 2.0, 3.0];
    let mut x = [0.0f64; 2];
    refused(gthr(&y, &[0, 3], &mut x));
    assert_eq!(x, [0.0, 0.0]);
    let mut yy = [1.0f64, 2.0, 3.0];
    refused(sctr(&[9.0, 9.0], &[1, 3], &mut yy));
    assert_eq!(yy, [1.0, 2.0, 3.0]);
    gthr(&y, &[2, 0], &mut x).unwrap(); // control
    assert_eq!(x, [3.0, 1.0]);
}

#[test]
fn doti_out_of_range_index_is_refused() {
    refused(doti_f64(&[1.0, 2.0], &[0, 3], &[1.0, 1.0, 1.0]));
    refused(doti_f32(&[1.0, 2.0], &[-1, 1], &[1.0, 1.0, 1.0]));
    assert_eq!(
        doti_f64(&[1.0, 2.0], &[0, 2], &[1.0, 1.0, 5.0]).unwrap(),
        11.0
    ); // control
}

#[test]
fn gthrz_out_of_range_index_is_refused() {
    let mut y = [1.0f64, 2.0, 3.0];
    let mut x = [0.0f64; 2];
    refused(gthrz_f64(&mut y, &[0, 3], &mut x));
    assert_eq!((y, x), ([1.0, 2.0, 3.0], [0.0, 0.0]));
    let mut yf = [1.0f32, 2.0, 3.0];
    let mut xf = [0.0f32; 2];
    refused(gthrz_f32(&mut yf, &[-1, 0], &mut xf));
    assert_eq!(yf, [1.0, 2.0, 3.0]);
}

#[test]
fn strided_gather_and_scatter_beyond_the_dense_vector_are_refused() {
    let y = [1.0f64, 2.0, 3.0];
    let mut x = [0.0f64; 3];
    refused(gthrs_f64(&y, &mut x, 2)); // needs y[0], y[2], y[4]
    assert_eq!(x, [0.0; 3]);
    refused(gthrs_f64(&y, &mut x, 0));
    refused(gthrs_f64(&y, &mut x, -1));
    let mut yy = [1.0f64, 2.0, 3.0];
    refused(sctrs_f64(&[7.0, 7.0, 7.0], &mut yy, 2));
    assert_eq!(yy, [1.0, 2.0, 3.0]);
    refused(sctrs_f32(&[7.0], &mut [1.0f32; 3], 0));
    gthrs_f64(&y, &mut x[..2], 2).unwrap(); // control: y[0], y[2]
    assert_eq!(&x[..2], &[1.0, 3.0]);
}

#[test]
fn complex_index_array_functions_refuse_out_of_range_indices() {
    let z = |v: f64| Complex64::new(v, -v);
    let mut y = [z(1.0), z(2.0), z(3.0)];
    refused(cx::axpyi(z(1.0), &[z(1.0), z(1.0)], &[0, 3], &mut y));
    refused(cx::sctr(&[z(1.0), z(1.0)], &[3, 0], &mut y));
    let mut x = [z(0.0); 2];
    refused(cx::gthr(&y, &[0, 3], &mut x));
    refused(cx::gthrz_c64(&mut y, &[0, 3], &mut x));
    assert_eq!(y, [z(1.0), z(2.0), z(3.0)]);
    refused(cx::dotui_c64(&[z(1.0), z(1.0)], &[0, 3], &y));
    refused(cx::dotci_c64(&[z(1.0), z(1.0)], &[-2, 0], &y));
    let yf = [Complex32::new(1.0, 0.0); 3];
    refused(cx::dotui_c32(&[Complex32::new(1.0, 0.0); 2], &[0, 3], &yf));
    refused(cx::dotci_c32(&[Complex32::new(1.0, 0.0); 2], &[5, 0], &yf));
    let mut yz = [Complex32::new(1.0, 0.0); 3];
    refused(cx::gthrz_c32(
        &mut yz,
        &[0, 3],
        &mut [Complex32::new(0.0, 0.0); 2],
    ));
    refused(cx::gthrs_c64(&y, &mut [z(0.0); 3], 2));
    refused(cx::sctrs_c32(&[Complex32::new(1.0, 0.0); 3], &mut yz, 2));
}
