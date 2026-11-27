//! Links the installed AOCL-LibM and calls clean-room declarations. The guide prints no known-answer values for LibM
//! (chapter 8 has signatures only), so the expected values here are exact by mathematical definition, not copied from AMD.
use aocl_math_sys as sys;

#[test]
fn scalar_exact_values() {
    unsafe {
        assert_eq!(sys::amd_sin(0.0), 0.0);
        assert_eq!(sys::amd_cos(0.0), 1.0);
        assert_eq!(sys::amd_sqrt(4.0), 2.0);
        assert_eq!(sys::amd_sqrtf(9.0), 3.0);
        assert_eq!(sys::amd_exp(0.0), 1.0);
        assert_eq!(sys::amd_log(1.0), 0.0);
        assert_eq!(sys::amd_fabs(-2.5), 2.5);
    }
}

#[test]
fn array_variant_matches_scalar() {
    // `amd_vrda_sqrt` (array form) is declared by the guide; check it against the scalar result.
    let src = [1.0_f64, 4.0, 9.0, 16.0];
    let mut dst = [0.0_f64; 4];
    unsafe { sys::amd_vrda_sqrt(4, src.as_ptr(), dst.as_mut_ptr()) };
    assert_eq!(dst, [1.0, 2.0, 3.0, 4.0]);
}
