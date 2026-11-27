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
