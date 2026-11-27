# Exposed vs tested: the safe wrappers (2026-11-27)

`python tools/wrapper-coverage.py --gaps` compares, per wrapper, the `pub fn` it exposes with the names its tests mention. **Heuristic, stated plainly:** a name counts as tested when it appears as a word in the crate's test code (`#[cfg(test)]` modules and `tests/*.rs`; for `aocl-scalapack` that includes four `harness = false` MPI binaries that `cargo test` runs as one rank and `mpiexec -n P` runs distributed). Generic names are counted by presence, so the "mentioned" column OVERSTATES coverage; the gaps are the exposed names no test mentions at all. Whether a mentioned function is called with a checked result is what `tools/wrapper-tests.py` plus the wrong-expected-value control (docs/WRAPPER-TESTS-20261127.md) establish per test.

```
wrapper                 #[test]  exposed pub fns  mentioned in tests   gaps
aocl-blas                    53               71                  42     29
   gaps: cgemm3m, cgemm_batch, crotg, csrot, dcabs1, dgemm_batch, dgemm_compute, dgemm_pack, dgemm_pack_get_size, drotm, drotmg, hbmv, hpmv, hpr2, sbmv, scabs1, sgemm_batch, sgemm_compute, sgemm_pack, sgemm_pack_get_size, spmv, spr2, srotmg, tbmv, tbsv, zdrot, zgemm3m, zgemm_batch, zrotg
aocl-lapack                  20              132                  18    114
   gaps: cgebrd, cgecon, cgeev, cgehrd, cgelss, cgeqp3, cgesvdx, cggev, cheevd, cheevr, chegv, chetrf, chetri, chetrs, clacpy, clange, clarfg, cpocon, csytrf, csytri, csytrs, cungbr, cunglq, cungqr, cungtr, cunmlq, cunmqr, dgebrd, dgecon, dgees, dgeev, dgehrd, dgelss, dgeqp3, dgesvdx, dggev, dlacpy, dlange, dlarfg, dlasrt, dorgbr, dorglq, dorgqr, dorgtr, dormlq, dormqr, dpocon, dpotri, dsyevd, dsyevr, dsygv, dsytrf, dsytri, dsytrs, dtrtri, dtrtrs, gelqf, hesv, sgebrd, sgecon, sgees, sgeev, sgehrd, sgelss, sgeqp3, sgesvdx, sggev, slacpy, slange, slarfg, slasrt, sorgbr, sorglq, sorgqr, sorgtr, sormlq, sormqr, spocon, spotri, ssyevd, ssyevr, ssygv, ssytrf, ssytri, ssytrs, strtri, strtrs, zgebrd, zgecon, zgeev, zgehrd, zgelss, zgeqp3, zgesvdx, zggev, zheevd, zheevr, zhegv, zhetrf, zhetri, zhetrs, zlacpy, zlange, zlarfg, zpocon, zsytrf, zsytri, zsytrs, zungbr, zunglq, zungqr, zungtr, zunmlq, zunmqr
aocl-fft                     11               20                  11      9
   gaps: dft_1d_inplace, dft_1d_inplace_f32, dft_2d, dft_2d_inplace_f32, dft_3d, dft_3d_inplace, dft_3d_inplace_f32, execute_c2r, execute_r2c
aocl-utils                    2               10                   7      3
   gaps: pin_threads, pin_threads_custom, x86_64_level
aocl-compression             13                9                   9      0
aocl-sparse                  21              122                  24     98
   gaps: blkcsrmv_f64, copy_mat_descr, create_csc_c32, create_csc_c64, create_csc_f32, create_csc_f64, create_tcsr_c32, create_tcsr_c64, create_tcsr_f32, create_tcsr_f64, csr2dia_f32, csr2dia_f64, csr2ell_f32, csr2ell_f64, csr_to_csc, csrsv, diag_type, dotci_c32, dotci_c64, doti_f32, doti_f64, dotmv_c32, dotmv_c64, dotmv_f32, dotmv_f64, dotui_c32, dotui_c64, fill_mode, from_library_owned, gthrs_c32, gthrs_c64, gthrs_f32, gthrs_f64, gthrz_c32, gthrz_c64, gthrz_f32, gthrz_f64, ilu_smoother, index_base, mv_f32, mv_f64, optimize, roti_f32, roti_f64, sctrs_c32, sctrs_c64, sctrs_f32, sctrs_f64, set_2m_hint, set_diag_type, set_dotmv_hint, set_fill_mode, set_index_base, set_lu_smoother_hint, set_memory_hint, set_mm_hint, set_mv_hint, set_sm_hint, set_sorv_hint, set_sv_hint, set_symgs_hint, set_type, set_value_c32, set_value_c64, set_value_f32, set_value_f64, sorv, sorv_f32, sp2md, symgs_c32, symgs_c64, symgs_f32, symgs_f64, symgs_mv_c32, symgs_mv_c64, symgs_mv_f32, symgs_mv_f64, syprd_c32, syprd_c64, syprd_f32, syprd_f64, syrkd_c32, syrkd_c64, syrkd_f32, syrkd_f64, trsm_c32, trsm_c64, trsm_f32, trsm_f64, trsv, trsv_f32, trsv_f64, ty, update_values_c32, update_values_c64, update_values_f32, update_values_f64, version
aocl-crypto                  19               37                  13     24
   gaps: add_digest, add_mgf, as_raw, copy_context_into, decrypt_oaep, decrypt_pkcs1v15, default_tag_len, digest_info_index, digest_info_size, encrypt_oaep, encrypt_pkcs1v15, key_len, key_size_bytes, output_len, set_ccm_lengths, set_public_key, sign_hash_pkcs1v15, sign_hash_pss, sign_pkcs1v15, sign_pss, verify_hash_pkcs1v15, verify_hash_pss, verify_pkcs1v15, verify_pss
aocl-data-analytics          24               57                  36     21
   gaps: classes, core_sample_indices, decision_function, geometric_mean, harmonic_mean, kneighbors, last_error_message, moment, n_cols, new_logistic, new_nu_svc, new_nu_svr, new_svr, polynomial_kernel, predict_log_proba, predict_proba, select_rows, set_bounds, set_weights, sigmoid_kernel, with_jacobian
aocl-math                    11              110                  53     57
   gaps: acosf, acoshf, asinf, asinhf, atan2f, atanf, atanhf, cbrtf, ceilf, copysignf, cosf, coshf, erff, exp10f, exp2f, expm1f, fabsf, fdimf, finitef, floorf, fmaxf, fminf, fmodf, frexpf, hypotf, ilogbf, ldexpf, llrint, llrintf, llroundf, ln_1pf, lnf, log10f, log2f, logbf, lrint, lrintf, lroundf, modff, nanf, nearbyint, nearbyintf, nextafterf, powf, remainderf, remquof, rint, rintf, roundf, scalbln, scalblnf, scalbnf, sincosf, sinf, tanf, tanhf, truncf
aocl-rng                     11               29                  10     19
   gaps: cauchy, chi_squared, exponential, f_dist, general_discrete, geometric, hypergeometric, leapfrog, logistic, lognormal, multi_students_t, multinomial, multinormal, negative_binomial, sample, skip_ahead, students_t, von_mises, with_subid
aocl-securerng                6               13                   9      4
   gaps: rdrand_u16, rdseed_bytes, rdseed_u16, rdseed_u64
aocl-scalapack                1               28                  25      3
   gaps: as_raw, barrier, context
TOTAL                                        638                 257    381
```

**Reading (origin/main 3d6b5ca):** 638 exposed public functions, 257 mentioned in tests, **381 not mentioned by any test** (lapack 114, sparse 98, math 57, blas 29, crypto 24, data-analytics 21, rng 19, fft 9, securerng 4, utils 3, scalapack 3; compression 0). Next (pm-kernels, comprehensive coverage): tests until every exposed function has at least one call into the DLL, smallest crates first, with the wrong-expected-value control.
