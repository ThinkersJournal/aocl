//! The umbrella crate only re-exports, so each feature's test checks that its module is the component it names:
//! a call with a known answer where one is simple, otherwise a path that exists only in that component.
//! Run with all features: `cargo test -p aocl --features "blas lapack fft math sparse rng securerng utils compression crypto data-analytics scalapack"`.

#[test]
fn shared_types_are_the_aocl_types_ones() {
    assert_eq!(aocl::Layout::RowMajor, aocl_types::Layout::RowMajor);
    let _: aocl::Error = aocl_error::Error::InvalidArgument("x".into());
}

#[cfg(feature = "blas")]
#[test]
fn blas_is_aocl_blas() {
    assert_eq!(aocl::blas::dot(&[1.0_f64, 2.0], &[3.0, 4.0]).unwrap(), 11.0);
}

#[cfg(feature = "lapack")]
#[test]
fn lapack_is_aocl_lapack() {
    // Only aocl-lapack has `getrf`; solve a 1x1 system through `gesv` is not needed, the path is the check.
    let _ = aocl::lapack::getrf::<f64>;
}

#[cfg(feature = "fft")]
#[test]
fn fft_is_aocl_fft() {
    let mut d = [[1.0_f64, 0.0], [0.0, 0.0]];
    aocl::fft::forward_inplace(&mut d).unwrap();
    assert_eq!(d, [[1.0, 0.0], [1.0, 0.0]]);
}

#[cfg(feature = "math")]
#[test]
fn math_is_aocl_math() {
    assert_eq!(aocl::math::cos(0.0), 1.0);
    assert_eq!(aocl::math::sin(0.0), 0.0);
}

#[cfg(feature = "sparse")]
#[test]
fn sparse_is_aocl_sparse() {
    aocl::sparse::MatDescr::new().unwrap();
}

#[cfg(feature = "rng")]
#[test]
fn rng_is_aocl_rng() {
    assert_eq!(aocl::rng::BaseGenerator::MersenneTwister as i32, 3);
}

#[cfg(feature = "securerng")]
#[test]
fn securerng_is_aocl_securerng() {
    let _ = aocl::securerng::is_rdrand_supported();
    assert_eq!(aocl::securerng::DEFAULT_RETRY_COUNT, 10);
}

#[cfg(feature = "utils")]
#[test]
fn utils_is_aocl_utils() {
    let _ = aocl::utils::cpuid::is_amd(aocl::utils::cpuid::Cpu::Current);
}

#[cfg(feature = "compression")]
#[test]
fn compression_is_aocl_compression() {
    let z = aocl::compression::compress(aocl::compression::Codec::Zstd, b"aaaaaaaaaaaaaaaa", 3)
        .unwrap();
    assert_eq!(&z[..4], &[0x28, 0xB5, 0x2F, 0xFD]);
}

#[cfg(feature = "crypto")]
#[test]
fn crypto_is_aocl_crypto() {
    let _ = aocl::crypto::aead::Mode::AesGcm;
}

#[cfg(feature = "data-analytics")]
#[test]
fn data_analytics_is_aocl_data_analytics() {
    // Mean of the whole 1 x 4 row (see aocl-data-analytics' own tests for the Axis / dof conventions).
    let x = [1.0_f64, 2.0, 3.0, 6.0];
    let mut mu = [0.0_f64; 1];
    aocl::data_analytics::mean(aocl::data_analytics::Axis::All, 1, 4, &x, &mut mu).unwrap();
    assert_eq!(mu[0], 3.0);
}

#[cfg(feature = "scalapack")]
#[test]
fn scalapack_is_aocl_scalapack() {
    assert_eq!(aocl::scalapack::numroc(10, 2, 0, 0, 3), 4);
}
