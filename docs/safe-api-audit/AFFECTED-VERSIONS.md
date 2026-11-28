# Safe-API audit: first / last affected PUBLISHED version per crate and function family

Measured, not read: for every published version of the crate the guard-page harness (`tools/gen-blas-harness.py`, `tools/gen-lapack-harness.py`, and the hand-written sparse files `vector_lengths.rs`, `dense_lengths.rs`, `raw_array_lengths.rs`) was built against that version's own `.crate` and dependencies and every test run in its own process (`tools/safe-api-versions.py`). HOLE = the process died with an access violation on a call with one buffer one element short. Result files: `results-*`/`<crate>-<version>*.csv` in this directory. A family not listed was refused or accepted without fault in every version measured. `gesv` (lapack) is the only measured LAPACK function that never faults. Yank and GHSA are CireSnave's (board 526 B / 527).

## aocl-sparse

Versions measured: 0.1.0, 0.2.2, 0.2.4, 0.2.5, 0.2.26, 0.2.40, 0.2.43, 0.2.46, 0.2.49, 0.2.54.

**MEASURED families (313 tests: vector_lengths 48, dense_lengths 152, raw_array_lengths 72, remaining_lengths 41): the safe CSR/CSC/ELL/DIA/BSR level-2/3 kernels, conversions and solvers of real f32/f64 and complex c32/c64. NOT MEASURED: `blkcsrmv`, `create_csc`, `create_tcsr` (no safe constructor / sink), anything not named above.** "0 holes" means 0 holes in the measured families only: it is NOT a statement that the crate is clean.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | family fixed in (first published version with 0 holes for this family) |
|---|---|---|---|---|
| bsrmv | 0.2.2 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| csr2bsr | 0.2.2 | 0.2.49 | 14 | 0.2.54 (measured 0 holes) |
| csr2dia | 0.2.2 | 0.2.43 | 8 | 0.2.46 (measured 0 holes) |
| csr2ell | 0.2.2 | 0.2.43 | 10 | 0.2.46 (measured 0 holes) |
| csrmm | 0.2.2 | 0.2.43 | 32 | 0.2.46 (measured 0 holes) |
| csrmv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (measured 0 holes) |
| csrsv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (measured 0 holes) |
| dotmv | 0.2.2 | 0.2.43 | 8 | 0.2.46 (measured 0 holes) |
| ellmv | 0.2.2 | 0.2.43 | 2 | 0.2.46 (measured 0 holes) |
| mv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (measured 0 holes) |
| sp2md | 0.2.2 | 0.2.43 | 16 | 0.2.46 (measured 0 holes) |
| spmmd | 0.2.2 | 0.2.43 | 16 | 0.2.46 (measured 0 holes) |
| symgs | 0.2.2 | 0.2.43 | 8 | 0.2.46 (measured 0 holes) |
| symgs_mv | 0.2.2 | 0.2.43 | 12 | 0.2.46 (measured 0 holes) |
| syprd | 0.2.2 | 0.2.43 | 32 | 0.2.46 (measured 0 holes) |
| syrkd | 0.2.2 | 0.2.43 | 16 | 0.2.46 (measured 0 holes) |
| trsm | 0.2.2 | 0.2.43 | 8 | 0.2.46 (measured 0 holes) |
| trsm_row | 0.2.2 | 0.2.43 | 8 | 0.2.46 (measured 0 holes) |
| trsv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (measured 0 holes) |

19 families with at least one HOLE.

## aocl-blas

Versions measured: 0.1.0, 0.2.5, 0.2.40, 0.2.43, 0.2.46, 0.2.49, 0.2.54.

**MEASURED families (506 tests, generated): the level-1/2/3 free functions of f32/f64/c32/c64 including gemmt. NOT MEASURED: the trait methods with explicit `inc`/`ld`, pack/compute/batch, `rotm`, `sdsdot`, `csrot`, `zdrot`.** "0 holes" means 0 holes in the measured families only: it is NOT a statement that the crate is clean.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | family fixed in (first published version with 0 holes for this family) |
|---|---|---|---|---|
| gemmt | 0.1.0 | 0.2.46 | 12 | 0.2.49 (measured 0 holes) |

1 families with at least one HOLE.

## aocl-lapack

Versions measured: 0.1.0, 0.2.5, 0.2.40, 0.2.43, 0.2.46, 0.2.49, 0.2.54, 0.5.3.

**MEASURED families (586 tests, generated): the typed d/s/z/c free functions and the generic `Scalar` trait methods listed below (370 holes in 120 families before the fixes). NOT MEASURED: the band functions `gb*`, `gesvdx`, `orgbr`, `lasrt`, the trait-level `RealScalar::geev` / `ComplexScalar::geev` called directly. `?lacpy` / `?lange` panic by documented contract (12 of the 586 tests exit with a panic, no fault).** "0 holes" means 0 holes in the measured families only: it is NOT a statement that the crate is clean.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | family fixed in (first published version with 0 holes for this family) |
|---|---|---|---|---|
| cgebrd | 0.1.0 | 0.2.49 | 5 | 0.2.54 (measured 0 holes) |
| cgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| cgeev | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| cgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| cgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| cgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| cggev | 0.1.0 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| cheevd | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| cheevr | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| chegv | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| chetrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| chetri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| chetrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| clacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| clange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| clarfg | 0.1.0 | 0.2.49 | 1 | 0.2.54 (measured 0 holes) |
| cpocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| csytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| csytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| csytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| cunglq | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| cungqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| cungtr | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| cunmlq | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| cunmqr | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| dgebrd | 0.1.0 | 0.2.49 | 5 | 0.2.54 (measured 0 holes) |
| dgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| dgees | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| dgeev | 0.1.0 | 0.2.49 | 5 | 0.2.54 (measured 0 holes) |
| dgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| dgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| dgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| dggev | 0.1.0 | 0.2.49 | 7 | 0.2.54 (measured 0 holes) |
| dlacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| dlange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| dlarfg | 0.1.0 | 0.2.49 | 1 | 0.2.54 (measured 0 holes) |
| dorglq | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| dorgqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| dorgtr | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| dormlq | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| dormqr | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| dpocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| dpotri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| dsyevd | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| dsyevr | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| dsygv | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| dsytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| dsytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| dsytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| dtrtri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| dtrtrs | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| gelqf | 0.1.0 | 0.2.46 | 4 | 0.2.49 (measured 0 holes) |
| gelsd | 0.1.0 | 0.2.49 | 8 | 0.2.54 (measured 0 holes) |
| gelsy | 0.1.0 | 0.2.49 | 8 | 0.2.54 (measured 0 holes) |
| geqrf | 0.1.0 | 0.2.46 | 4 | 0.2.49 (measured 0 holes) |
| gesdd | 0.1.0 | 0.2.49 | 16 | 0.2.54 (measured 0 holes) |
| gesvd | 0.1.0 | 0.2.49 | 20 | 0.2.54 (measured 0 holes) |
| getrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| getri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| getrs | 0.1.0 | 0.2.46 | 6 | 0.2.49 (measured 0 holes) |
| gtsv | 0.1.0 | 0.2.46 | 6 | 0.2.49 (measured 0 holes) |
| heev | 0.1.0 | 0.2.46 | 4 | 0.2.49 (measured 0 holes) |
| hesv | 0.1.0 | 0.2.46 | 4 | 0.2.49 (measured 0 holes) |
| posv | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| potrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| potrs | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| ptsv | 0.1.0 | 0.2.46 | 8 | 0.2.49 (measured 0 holes) |
| sgebrd | 0.1.0 | 0.2.49 | 5 | 0.2.54 (measured 0 holes) |
| sgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| sgees | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| sgeev | 0.1.0 | 0.2.49 | 5 | 0.2.54 (measured 0 holes) |
| sgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| sgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| sgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| sggev | 0.1.0 | 0.2.49 | 7 | 0.2.54 (measured 0 holes) |
| slacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| slange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| slarfg | 0.1.0 | 0.2.49 | 1 | 0.2.54 (measured 0 holes) |
| sorglq | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| sorgqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| sorgtr | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| sormlq | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| sormqr | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| spocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| spotri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| ssyevd | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| ssyevr | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| ssygv | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| ssytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| ssytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| ssytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| strtri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| strtrs | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| syev | 0.1.0 | 0.2.46 | 4 | 0.2.49 (measured 0 holes) |
| sysv | 0.1.0 | 0.2.46 | 6 | 0.2.49 (measured 0 holes) |
| zgebrd | 0.1.0 | 0.2.49 | 5 | 0.2.54 (measured 0 holes) |
| zgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| zgeev | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| zgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| zgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| zggev | 0.1.0 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| zheevd | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| zheevr | 0.1.0 | 0.2.49 | 4 | 0.2.54 (measured 0 holes) |
| zhegv | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| zhetrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zhetri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zhetrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| zlacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zlange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| zlarfg | 0.1.0 | 0.2.49 | 1 | 0.2.54 (measured 0 holes) |
| zpocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (measured 0 holes) |
| zsytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zsytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zsytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (measured 0 holes) |
| zunglq | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| zungqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (measured 0 holes) |
| zungtr | 0.1.0 | 0.2.49 | 2 | 0.2.54 (measured 0 holes) |
| zunmlq | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |
| zunmqr | 0.1.0 | 0.2.49 | 3 | 0.2.54 (measured 0 holes) |

120 families with at least one HOLE.

## aocl-data-analytics

Versions measured: 0.2.2, 0.2.4, 0.2.5, 0.2.28, 0.2.40, 0.2.43, 0.2.46, 0.2.49, 0.2.54, 0.5.0, 0.5.1.

**MEASURED families (kernels and statistics 274 tests; handle models 58 tests in the section below): the free statistics functions, the five kernel / distance functions, KMeans, Pca, KNearestNeighbours, Linmod, Dbscan, DecisionTree, DecisionForest, Svm. NOT MEASURED by a harness: `Handle` options, and the `Nlls` callbacks (`resfun`, `resgrd`: CLASSIFIED BY SOURCE READ, with the reason: the C shims `nlls_resfun_shim` / `nlls_resgrd_shim` build the callback slices with the wrapper's own `n_coef` / `n_res` (and `n_res * n_coef` for the Jacobian), never with a length the library reports, `catch_unwind` keeps a panic from crossing the FFI boundary, `Nlls::new` / `set_bounds` / `set_weights` / `fit` check `coef`, `weights`, `lower`, `upper` against the same `n_coef` / `n_res` before the call, and the library is called under a global lock; the one residual risk is the library handing the shim a `res` / `jac` pointer smaller than the sizes it was told, which only a harness inside the library could observe). `Datastore::extract_f64` and the handle models are in their own sections.** "0 holes" means 0 holes in the measured families only: it is NOT a statement that the crate is clean.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | family fixed in (first published version with 0 holes for this family) |
|---|---|---|---|---|
| linear_kernel | 0.2.2 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| pairwise_distances | 0.2.2 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| polynomial_kernel | 0.2.2 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| rbf_kernel | 0.2.2 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |
| sigmoid_kernel | 0.2.2 | 0.2.49 | 6 | 0.2.54 (measured 0 holes) |

5 families with at least one HOLE.

## aocl-fft

Versions measured: 0.1.0, 0.2.6, 0.2.40, 0.2.43, 0.2.46, 0.2.49, 0.2.54.

**MEASURED families (40 tests): `Plan` DFT / r2c / c2r length checks and the plan-time alignment rule (f64 and f32). NOT MEASURED: plan creation flags other than the defaults, the 2-D / 3-D plan constructors beyond the length cases listed.** "0 holes" means 0 holes in the measured families only: it is NOT a statement that the crate is clean.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | family fixed in (first published version with 0 holes for this family) |
|---|---|---|---|---|
| plan_dft_misaligned_input | 0.1.0 | 0.2.49 | 1 | 0.2.54 (measured 0 holes) |
| plan_dft_misaligned_output | 0.1.0 | 0.2.49 | 1 | 0.2.54 (measured 0 holes) |

2 families with at least one HOLE.

## Source-read rows (NOT RUN: the version does not build with the harness; PM ruling 2026-11-28, only published versions count)

Method: the published `.crate` was downloaded and its source read: does it contain the function, and does the code between the safe signature and the FFI call lack a length check.

| crate | version | functions | evidence (published 0.1.0 source) | status |
|---|---|---|---|---|
| aocl-data-analytics | 0.1.0 | `linear_kernel`, `rbf_kernel`, `polynomial_kernel`, `sigmoid_kernel`, `pairwise_distances` | `src/lib.rs` 1725-1735: `linear_kernel` computes `ldd` and calls `T::linear_kernel(Layout::RowMajor, m, n, k, x, k, y, k, d, ldd)`; the `KernelScalar` impl (1572-1599) goes straight to `x.as_ptr()`, `y_ptr`, `d.as_mut_ptr()` in `sys::$linear(...)`; no `x.len()` / `d.len()` comparison anywhere (the same shape for the other four) | not run; HOLE by source read |
| aocl-sparse | 0.1.0 | `mv_f64` (and the `mv` / `trsv` / `symgs*` families of `complex.rs`/`lib.rs`) | `src/lib.rs`: `mv_f64(op, alpha, a, descr, x, beta, y)` calls `sys::aoclsparse_dmv(..., x.as_ptr(), &beta, y.as_mut_ptr())` directly; no `x.len()` / `y.len()` check | not run; HOLE by source read for the families measured as HOLE in 0.2.2 (the per-family rows above list which) |

The yank range per crate is first affected .. last published version before the first fixed one, not only the versions measured above.

## aocl-crypto RSA and aocl-rng multivariate (measured per published version; families fixed in 0.4.0: measured 0 holes)

**aocl-crypto RSA** (`tests/guard_lengths.rs`, report mode, 24 short cases): HOLE in 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 for the ten families `encrypt` / `decrypt(Padding::None)` plaintext / `encrypt_oaep` ciphertext AND seed / `encrypt_pkcs1v15` ciphertext AND pad / `sign_pss` / `sign_pkcs1v15` / `sign_hash_pss` / `sign_hash_pkcs1v15` (10 holes each version). In 0.2.2 0.2.4 0.2.5 seven of the ten fault (`encrypt`, `encrypt_oaep` seed, `encrypt_pkcs1v15` pad, the four signing buffers); the other three could not be measured there (those versions return an error or panic in key set-up before the call, `OTHER(rc=101)`), so they are "not run" and counted by the yank range (first affected 0.2.2). 0.1.0 does not build with the harness: source read, `src/rsa.rs` 137-146 `encrypt` and 215-227 `sign_pss` call `ciphertext.as_mut_ptr()` / `signature.as_mut_ptr()` with no length check (no `len() <` anywhere in the file): HOLE by source read, not run. First affected 0.1.0, last published affected 0.2.54 (0.2.55 and 0.2.56 were never published: batch 4 published as 0.4.0), RSA family fixed in 0.4.0: MEASURED against the published 0.4.0 crate, 25 tests, 0 faults (`results aocl-crypto-0.4.0.csv`). Cipher (CBC/CTR/OFB/CFB/ChaCha20), AEAD (GCM/CCM) and EC: CLEAR (0 faults, 19 tests, 0.2.50 tree; `results-aocl-crypto-cipher-aead.csv`; older versions not re-run: the check code those paths use predates 0.2.2 per the source).

**aocl-rng** `multinormal` / `multi_students_t` / `multinomial` (`tests/guard_lengths.rs`, 10 short cases): HOLE (10 of 10) in 0.1.0 0.2.40 0.2.43 0.2.46 0.2.49. First affected 0.1.0, last published affected 0.2.54, multivariate family fixed in 0.4.0: MEASURED against the published 0.4.0 crate, 11 tests, 0 faults. `general_discrete` (any slice accepted): affected in every published version containing it up to 0.2.54, family fixed in 0.4.0 by construction (`DiscreteReference`; harness row `tests/typed_general_discrete.rs` run against the published 0.4.0 crate: 5 constructors sampled into a guard-page output with no fault, 3 invalid parameters refused; the unsafe raw entry remains by contract).

## aocl-scalapack (f64 harness, single MPI rank; families fixed in 0.4.0: measured 0 holes)

Measured per published version (`tests/guard_lengths.rs`, 38 cases, one process each): **18 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54** (the slices `a` of gesv/gemm/potrf/getrf/getrs/potrs/syev/gels/geqrf, `b` of gesv/gemm/getrs/potrs/gels, `c` of gemm, `w` and `z` of syev, `tau` of geqrf); the 6 short `ipiv` / `work` cases and the `n` past the descriptor cases never fault (accepted or refused by the library; `pdgemm` with `n` past its descriptor aborts the process via PXERBLA). 0.1.0 does not build with the harness: source read, `src/lib.rs` 264-295 `pdgesv` passes `a.as_mut_ptr()`, `ipiv.as_mut_ptr()`, `b.as_mut_ptr()` and the descriptors to `sys::pdgesv_` with no `.len()` check (0 `.len()` in the function): HOLE by source read, not run, same shape as the other functions. First affected 0.1.0, last published affected 0.2.54, family fixed in 0.4.0: MEASURED against the published 0.4.0 crate: 38 tests, 0 faults (29 refused, 9 controls), and the crate's four MPI tests (`pdgesv_mpi`, `psgesv_mpi`, `complex_mpi`, `grid2d_mpi`) pass on 1, 2 and 4 MPI ranks (2 x 2 grid) from the published sources, so valid calls still work. `pdgemm` / `psgemm` now return `Result<()>`.

## aocl-compression bzip2 decode (family fixed in 0.5.0 by refusal: measured; `unsafe decompress_bzip2_trusted` remains)

Measured per published version (`tests/guard_lengths.rs`, corrupt-input sweep): **`decompress(Codec::Bzip2, ..)` HOLE in 0.2.2 0.2.4 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54** (3 of 218 single-bit flips of a valid 109-byte stream fault inside the decoder, no truncation does; a crafted bzip2 stream crashes the process, memory corruption is not excluded, exploitability unmeasured). LZ4, LZMA, Snappy, Zlib, Zstd decode and the native lz4/snappy short-dst paths: no fault in every measured version. 0.1.0 did not build with the harness: source read, `src/lib.rs` 96-112 `decompress` passes `src.as_ptr()` and `out.as_mut_ptr()` to `aocl_llc_decompress` for any `Codec` including `Bzip2` with no stream check: affected by source read, not run. First affected 0.1.0, last published affected 0.4.0 (0.4.0 re-run: the same HOLE; 9 earlier versions plus 0.4.0 are live), **family fixed in 0.5.0: MEASURED against the published 0.5.0 crate: the corrupt-stream sweep shows 0 faults because the safe `decompress(Codec::Bzip2, ..)` now returns an error for EVERY bzip2 stream, valid ones too (the documented functional regression: the round-trip control `roundtrip_control` fails at 0.5.0 for that reason alone); the crate's `tests/bzip2_upstream_defect.rs` passes against the published sources: the safe call refuses all three repro streams and the valid one, and a child process decoding each flip through the unsafe entry still dies with 0xC0000005, so the upstream decoder defect is still present in AOCL 6.0.68.** Yank range: 0.1.0 .. 0.4.0. The bzip2 encoder is not affected by what was measured.

## aocl-data-analytics handle models (families fixed in 0.5.1: measured 0 holes)

Measured per published version (`tests/handle_models_lengths.rs`, 58 tests = 7 controls + 51 one-element-short cases over KMeans, Pca, KNearestNeighbours, Linmod, Dbscan, DecisionTree, DecisionForest, Svm; files `aocl-dam-<version>.csv`): **19 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54** (identical set in each): tree and forest `predict_proba` / `predict_log_proba` (`x_test` and output) and `score` (`x_test`); `KMeans::predict` `y`; `kneighbors` `x_test` and `n_dist`; kNN `predict_proba` `x_test` and `proba`; `Svm::score` `x_test` and `y_test`; `Svm::decision_function` `x_test` and output. Plus one heap overflow the guard page cannot see (`kneighbors` with `n_ind` shorter than `n_queries * k`). 0.1.0 did not build with the harness: source read, `src/lib.rs` 2281-2297 (`KNearestNeighbours::predict_proba`) and 3346 (`decision_function`) pass `x_test.as_ptr()` and `proba.as_mut_ptr()` straight to the library with no length check: affected by source read, not run. **0.5.0 (positive control, built from the published .crate): the same 19 HOLE and the same accepted `kneighbors` short-`n_ind` case (the heap write) as every earlier version. 0.5.1 (published .crate): 0 HOLE, all 51 short cases refused (including `kneighbors` short `n_ind`, now `InvalidArgument`), 7 controls pass; kernels and statistics at 0.5.0 and 0.5.1: 274 tests, 0 holes.** First affected 0.1.0, last published affected 0.5.0, family fixed in 0.5.1 (measured). Yank range: 0.1.0 .. 0.5.0 (first affected .. last published before the first fixed version).

## aocl-data-analytics `Datastore::extract_f64` (fix PENDING #79; memory-corruption class: a write past the caller's buffer)

Measured per published version (`tests/datastore_lengths.rs`: 1 control + 4 cases; files `aocl-dads-<version>.csv`): **3 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54 0.4.0 0.5.0 0.5.1** (destination one element short for a column selection and for a row selection; `lddata` larger than the buffer allows); the fourth case (a key never selected) is refused by the library everywhere. **0.5.1, the only live data-analytics version, is affected.** 0.1.0 did not build with the harness: source read, `src/lib.rs` 1356-1372 passes `data.as_mut_ptr()` and `lddata` to `da_data_extract_selection_real_d` with no `data.len()` check: affected by source read, not run. First affected 0.1.0, last published affected 0.5.1, family fixed in PENDING #79. NOT MEASURED in the same crate: the `Nlls` callback closures' own slices (`Nlls::new`, `set_bounds`, `set_weights`, `fit` check lengths against `n_coef` / `n_res` by source read).

## aocl-openrng (publish = false; PR #55, stacked on #34; no published crate, so no yank row)

**MEASURED families** (the wrapper's tests `tests/out_guard.rs`, 4 tests: every real f64 / f32 generator and every integer generator writes at most `n` elements into a guard-page output; `tests/params_guard.rs`, 16 tests: `Stream::from_params`, `skip_ahead_ex`, `poisson_v` (lambda length), `multinomial` (p length), `gaussian_mv` / `gaussian_mv_f32` (mean, covariance)). **Measured column:** the PM's `run_wrapper_tests.py` (5,448 B, sha256 prefix 94f9c1de) at #55 head `ffc318d8` on a clean worktree, both builds (LP64 and ILP64) against the build-record `openrng.dll` hashes, wrapper exit 0, params_guard 16/16 and out_guard 4/4 each exit 0, 23 child processes exit 0 including the cargo test steps, no access violation. Records: `C:/openrng-measure/out/WRAPPER-TESTS-lp64-20261128-040434.txt` and `-ilp64-20261128-040536.txt`, copies `C:/Projects/.lane-state/wrapper-tests-{lp64,ilp64}.out`. The file times (04:04 / 04:05) are the box-local clock; the PM's mail was about 11:1x UTC. An earlier run at `776758a4` stays history. Platform: Windows, AOCL-OpenRNG 6.0.0; Linux unmeasured. I did not run the PM-built DLLs myself (standing rule).
**NOT MEASURED** (the 59 exposed names minus the two files' set; grep over `src/lib.rs` `pub fn` and the `dist!` / `dist0!` macro names, positive control: the same query lists the 42 covered names): `as_bytes`, `as_raw`, `brng_id`, `copy_state_from`, `from_bytes` (unsafe), `from_id`, `id`, `leapfrog`, `load_from_file` (unsafe), `load_from_memory` (unsafe), `registered_brngs`, `restore`, `save`, `save_to_file`, `skip_ahead`, `try_clone`, `version`, `with_brng_id` (unsafe). The untrusted-input entry points (`load_from_file`, `load_from_memory`, `from_bytes`, `with_brng_id`, `from_id`) are `unsafe` or return `Option` by construction in #55; `restore` decodes bytes held in a `SavedStream`, which only `save` (library output) or the `unsafe` `from_bytes` can create.

## aocl-lapack band solvers, direct trait `geev`, `gesvdx`, `orgbr` / `ungbr` (families fixed in 0.5.3: measured 0 holes)

Measured per published version (`tests/rest_lengths.rs`: 44 tests = 10 controls + 34 one-element-short cases over f64 and Complex64; files `aocl-lrest-<version>.csv`): **23 HOLE in every one of 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54 0.4.0 0.5.0 0.5.1 0.5.2** (identical set): `gbsv` / `gbtrs` `b`, `gbsv` / `gbtrf` `ipiv`, every matrix and vector of the direct `RealScalar::geev` and `ComplexScalar::geev`, `gesvdx` a / s / u / vt, `orgbr` a / tau. (The other 11 short cases, among them `gbtrs` `ipiv` and `gesvdx` `superb`, do not fault in any version.) **The live versions 0.2.54, 0.4.0, 0.5.0, 0.5.1 and 0.5.2 are affected.** 0.1.0 did not build with the harness: source read, `src/lib.rs` 5017-5035 (`dorgbr`) and the trait impls at 869 / 1260 / 1642 / 2078 pass the slices straight to LAPACKE with no length check: affected by source read, not run. **0.5.2 (control, same run): the same 23 HOLE; 0.5.3 (published .crate): 0 HOLE, all 34 short cases refused with `InvalidArgument`, 10 controls pass; the generated lapack harness (586 tests) at 0.5.3: 0 HOLE.** First affected 0.1.0, last published affected 0.5.2, family fixed in 0.5.3 (measured). Yank range: 0.1.0 .. 0.5.2. NOT MEASURED in the same crate: `?lasrt` (length = `d.len()`), the f32 / Complex32 copies.

## aocl-blas `?gemm_pack` / `?gemm_compute` / `cgemm3m` / `zgemm3m` (families fixed in 0.6.1: measured 0 holes; fix #82, BREAKING `Result<()>`)

Measured per published version (`tests/pack_lengths.rs`, 22 tests = 6 controls + 16 one-element-short cases over `sgemm_pack` / `dgemm_pack` (src, dest), `sgemm_compute` / `dgemm_compute` (a, b, c), `cgemm3m` / `zgemm3m` (a, b, c); files `aocl-blpk-<version>.csv`; the harness accepts both the old `()` and the new `Result<()>` return so one file builds against every published `.crate`): **14 HOLE in every one of 0.1.0 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54 0.4.0 0.5.0 0.5.1 0.5.2 0.5.3 0.5.4** (all 13 published versions before 0.6.1 build with the harness, so 0.1.0 is measured, not source-read): `?gemm_pack` `src`; `?gemm_compute` `a`, `b`, `c`; `cgemm3m` / `zgemm3m` `a`, `b`, `c` (the two pack `dest` cases never fault but violate the size, which is in BYTES: `dgemm_pack_get_size(A, 6, 5, 4)` = 1152). **Control, same run: 0.5.4 still faults the 14. 0.6.1 (published .crate, `v0.6.1` on 593ff3df): 0 HOLE, all 16 short cases refused with `InvalidArgument`, 6 controls pass.** First affected 0.1.0, last published affected 0.5.4, family fixed in 0.6.1 (measured). Yank range: 0.1.0 .. 0.5.4 (per crates.io: 0.5.4 is the last version before 0.6.1; 0.6.0 was never published alone).

**MEASURED families in aocl-blas beyond the generated free-function harness (506 tests):** the trait methods with explicit layout / `ld` / `inc` (`tests/trait_lengths.rs`, 259 tests, f64 and Complex64, both layouts, strided and padded; files `aocl-bltr-<version>.csv`): **0 faults in every version measured** (0.1.0 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54 0.4.0 0.5.0 0.5.1 0.5.2 0.5.3 0.5.4 0.6.1; 73 controls pass, 176 short cases refused, 10 level-1 two-vector cases tolerated). NOT MEASURED in aocl-blas: the f32 / Complex32 copies of the trait methods (same macro paths), `scal` / `nrm2` / `asum` / `iamax` (length is `x.len()`), `?gemm_pack` / `?gemm_compute` with flag combinations other than (NoTrans, Packed), `?gemm_batch` (`unsafe` by signature), `srotm` / `drotm` / `sdsdot` / `csrot` / `zdrot` (length is the shorter slice, source read).

## aocl-sparse `blkcsrmv_f64` (block-CSR arrays; fix PENDING #84, armed 0.6.2)

Measured per published version (`tests/blk_lengths.rs`: 1 control + 10 broken-input cases + 1 valid-shapes test; files `aocl-blk-<version>.csv`): **7 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.26 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54 0.4.0 0.5.0 0.5.1 0.5.2 0.5.3 0.5.4 0.6.1** (short `x`, `y`, `masks`, `bv`, `bci`; a block column that reads past `x`; `brp` past `bv`): **every live sparse version is affected**. 0.1.0 did not build with the harness: source read, `blkcsrmv_f64` passes the arrays straight to `aoclsparse_dblkcsrmv` (no `check_*`): affected by source read, not run. First affected 0.1.0, last published affected 0.6.1, family fixed in PENDING #84. Structure measured on AOCL 6.0.68 with `n_rows_blk` = 4 only (`csr2blkcsr` refused 8). NOT MEASURED: `create_csc`, `create_tcsr` (raw library handle, no safe sink).

## Cleared, no fix needed

- **aocl-securerng** (`tests/guard_lengths.rs`, 5 cases: exact-length `rdrand_bytes`, `rdseed_bytes`, `rdrand_u32_array`, `rdrand_u64_array` at a guard page): 0 faults, `n` counts elements. Source: every function passes `out.len()` as the count.
- **aocl-utils** (`tests/guard_lengths.rs`, 6 cases: `Cpu::Specific(1_000_000)` and `Specific(u32::MAX - 1)` into `has_flag`, `has_flags_all`, `has_flags_any`, `vendor_info`): 0 faults (the library tolerates an out-of-range CPU index). `has_flags_*` build NUL-terminated strings and pass the pointer count.
- **aocl-math** (`ldexp`, `ldexpf`, `scalbn`, `scalbnf`, `scalbln`, `scalblnf`): value in, value out, no pointer or slice argument: nothing to fault by source read.
- **aocl-compression** other codecs and **aocl-crypto** cipher / AEAD / EC: see the sections above.

## Coverage of the crates described in sections above (MEASURED / NOT MEASURED)

"0 holes" in any row means 0 holes in the measured families only; it is not a statement that the crate is clean.

- **aocl-crypto**: MEASURED RSA (25 tests: 24 one-byte-short cases + control; encrypt, decrypt, OAEP, PSS, PKCS#1 v1.5, hash variants), cipher and AEAD constructors / buffers (19 tests: CBC, CTR, OFB, CFB, ChaCha20 iv / key / partial block; GCM iv / key / tag; CCM nonce). NOT MEASURED: digest, mac, EC (read by source only: EC validates before the FFI call), `set_aad`, `Rsa::set_public_key` / `set_private_key` beyond the length checks already present.
- **aocl-rng**: MEASURED `multinormal`, `multi_students_t`, `multinomial` (11 tests). NOT MEASURED: `BlumBlumShub::new`, the univariate generators (read by source: `n = out.len()`), the typed `general_discrete` path (type-enforced; harness row owed), `leapfrog` / `skip_ahead`.
- **aocl-scalapack**: MEASURED f64 `pdgesv pdgemm pdgetrf pdgetrs pdpotrf pdpotrs pdsyev pdgels pdgeqrf` on a single rank, 1 x 1 grid (38 tests); fixed-version multi-rank runs are valid-call checks only. NOT MEASURED: the f32 / complex copies, multi-rank guard cases, `Grid`, `BlockCyclic::new`.
- **aocl-compression**: MEASURED corrupt-input decode of all six codecs (truncation + bit flips) and the native lz4 / snappy short-destination paths (10 tests). NOT MEASURED: the bzip2 / lzma / others' `compress` with extreme levels, `lz4::compress_bound` (returns the source length, not the bound: a logic defect, not a memory one).
- **aocl-securerng**, **aocl-utils**, **aocl-math**: see "Cleared" below; **aocl-openrng**: see its section below.

## Not measured (read pending)

sparse `blkcsrmv` (no safe mask constructor), `create_csc` / `create_tcsr` (raw handle, no safe sink); lapack band `gb*`, `gesvdx`, `orgbr`, `lasrt`, and the trait-level `RealScalar::geev` / `ComplexScalar::geev` called directly (the typed free functions are measured); blas trait methods with explicit `inc`/`ld`, pack/compute/batch/rotm/sdsdot/csrot/zdrot; and the rows data-analytics handle models (kmeans/pca/knn/linear/rf, set_weights, extract_f64), crypto digest/mac (value+len, source only), aocl-openrng (after its PRs merge). `aocl-sparse` 0.1.0 did not compile with the harness (2 errors in its own lib against current dependencies) and is unmeasured; the functions are present in its source.

