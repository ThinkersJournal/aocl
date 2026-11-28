# Safe-API audit: first / last affected PUBLISHED version per crate and function family

Measured, not read: for every published version of the crate the guard-page harness (`tools/gen-blas-harness.py`, `tools/gen-lapack-harness.py`, and the hand-written sparse files `vector_lengths.rs`, `dense_lengths.rs`, `raw_array_lengths.rs`) was built against that version's own `.crate` and dependencies and every test run in its own process (`tools/safe-api-versions.py`). HOLE = the process died with an access violation on a call with one buffer one element short. Result files: `results-*`/`<crate>-<version>*.csv` in this directory. A family not listed was refused or accepted without fault in every version measured. `gesv` (lapack) is the only measured LAPACK function that never faults. Yank and GHSA are CireSnave's (board 526 B / 527).

## aocl-sparse

Versions measured: 0.1.0, 0.2.2, 0.2.4, 0.2.5, 0.2.26, 0.2.40, 0.2.43, 0.2.46.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |
|---|---|---|---|---|
| bsrmv | 0.2.2 | 0.2.46 | 6 | PENDING: #70 |
| csr2bsr | 0.2.2 | 0.2.46 | 14 | PENDING: #70 |
| csr2dia | 0.2.2 | 0.2.43 | 8 | 0.2.46 (published 2026-11-28) |
| csr2ell | 0.2.2 | 0.2.43 | 10 | 0.2.46 (published 2026-11-28) |
| csrmm | 0.2.2 | 0.2.43 | 32 | 0.2.46 (published 2026-11-28) |
| csrmv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (published 2026-11-28) |
| csrsv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (published 2026-11-28) |
| dotmv | 0.2.2 | 0.2.43 | 8 | 0.2.46 (published 2026-11-28) |
| ellmv | 0.2.2 | 0.2.43 | 2 | 0.2.46 (published 2026-11-28) |
| mv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (published 2026-11-28) |
| sp2md | 0.2.2 | 0.2.43 | 16 | 0.2.46 (published 2026-11-28) |
| spmmd | 0.2.2 | 0.2.43 | 16 | 0.2.46 (published 2026-11-28) |
| symgs | 0.2.2 | 0.2.43 | 8 | 0.2.46 (published 2026-11-28) |
| symgs_mv | 0.2.2 | 0.2.43 | 12 | 0.2.46 (published 2026-11-28) |
| syprd | 0.2.2 | 0.2.43 | 32 | 0.2.46 (published 2026-11-28) |
| syrkd | 0.2.2 | 0.2.43 | 16 | 0.2.46 (published 2026-11-28) |
| trsm | 0.2.2 | 0.2.43 | 8 | 0.2.46 (published 2026-11-28) |
| trsm_row | 0.2.2 | 0.2.43 | 8 | 0.2.46 (published 2026-11-28) |
| trsv | 0.2.2 | 0.2.43 | 4 | 0.2.46 (published 2026-11-28) |

19 families with at least one HOLE.

## aocl-blas

Versions measured: 0.1.0, 0.2.5, 0.2.40, 0.2.43, 0.2.46.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |
|---|---|---|---|---|
| gemmt | 0.1.0 | 0.2.46 | 12 | 0.2.47 (merged, not yet published) |

1 families with at least one HOLE.

## aocl-lapack

Versions measured: 0.1.0, 0.2.5, 0.2.40, 0.2.43, 0.2.46.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |
|---|---|---|---|---|
| cgebrd | 0.1.0 | 0.2.46 | 5 | PENDING: #69 (next after #68) |
| cgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| cgeev | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| cgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| cgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| cgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| cggev | 0.1.0 | 0.2.46 | 6 | PENDING: #68 (0.2.50 armed) |
| cheevd | 0.1.0 | 0.2.46 | 2 | PENDING: #68 (0.2.50 armed) |
| cheevr | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| chegv | 0.1.0 | 0.2.46 | 3 | PENDING: #68 (0.2.50 armed) |
| chetrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| chetri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| chetrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| clacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| clange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| clarfg | 0.1.0 | 0.2.46 | 1 | PENDING: #69 (next after #68) |
| cpocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| csytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| csytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| csytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| cunglq | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| cungqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| cungtr | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| cunmlq | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| cunmqr | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| dgebrd | 0.1.0 | 0.2.46 | 5 | PENDING: #69 (next after #68) |
| dgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| dgees | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| dgeev | 0.1.0 | 0.2.46 | 5 | PENDING: #68 (0.2.50 armed) |
| dgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| dgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| dgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| dggev | 0.1.0 | 0.2.46 | 7 | PENDING: #68 (0.2.50 armed) |
| dlacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| dlange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| dlarfg | 0.1.0 | 0.2.46 | 1 | PENDING: #69 (next after #68) |
| dorglq | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| dorgqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| dorgtr | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| dormlq | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| dormqr | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| dpocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| dpotri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| dsyevd | 0.1.0 | 0.2.46 | 2 | PENDING: #68 (0.2.50 armed) |
| dsyevr | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| dsygv | 0.1.0 | 0.2.46 | 3 | PENDING: #68 (0.2.50 armed) |
| dsytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| dsytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| dsytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| dtrtri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| dtrtrs | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| gelqf | 0.1.0 | 0.2.46 | 4 | 0.2.48 (group A #66, merged; unpublished) |
| gelsd | 0.1.0 | 0.2.46 | 8 | PENDING: #69 (next after #68) |
| gelsy | 0.1.0 | 0.2.46 | 8 | PENDING: #69 (next after #68) |
| geqrf | 0.1.0 | 0.2.46 | 4 | 0.2.48 (group A #66, merged; unpublished) |
| gesdd | 0.1.0 | 0.2.46 | 16 | PENDING: #69 (next after #68) |
| gesvd | 0.1.0 | 0.2.46 | 20 | PENDING: #69 (next after #68) |
| getrf | 0.1.0 | 0.2.46 | 2 | 0.2.48 (group A #66, merged; unpublished) |
| getri | 0.1.0 | 0.2.46 | 2 | 0.2.48 (group A #66, merged; unpublished) |
| getrs | 0.1.0 | 0.2.46 | 6 | 0.2.48 (group A #66, merged; unpublished) |
| gtsv | 0.1.0 | 0.2.46 | 6 | 0.2.48 (group A #66, merged; unpublished) |
| heev | 0.1.0 | 0.2.46 | 4 | 0.2.48 (group A #66, merged; unpublished) |
| hesv | 0.1.0 | 0.2.46 | 4 | 0.2.48 (group A #66, merged; unpublished) |
| posv | 0.1.0 | 0.2.46 | 2 | 0.2.48 (group A #66, merged; unpublished) |
| potrf | 0.1.0 | 0.2.46 | 2 | 0.2.48 (group A #66, merged; unpublished) |
| potrs | 0.1.0 | 0.2.46 | 2 | 0.2.48 (group A #66, merged; unpublished) |
| ptsv | 0.1.0 | 0.2.46 | 8 | 0.2.48 (group A #66, merged; unpublished) |
| sgebrd | 0.1.0 | 0.2.46 | 5 | PENDING: #69 (next after #68) |
| sgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| sgees | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| sgeev | 0.1.0 | 0.2.46 | 5 | PENDING: #68 (0.2.50 armed) |
| sgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| sgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| sgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| sggev | 0.1.0 | 0.2.46 | 7 | PENDING: #68 (0.2.50 armed) |
| slacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| slange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| slarfg | 0.1.0 | 0.2.46 | 1 | PENDING: #69 (next after #68) |
| sorglq | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| sorgqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| sorgtr | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| sormlq | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| sormqr | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| spocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| spotri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| ssyevd | 0.1.0 | 0.2.46 | 2 | PENDING: #68 (0.2.50 armed) |
| ssyevr | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| ssygv | 0.1.0 | 0.2.46 | 3 | PENDING: #68 (0.2.50 armed) |
| ssytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| ssytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| ssytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| strtri | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| strtrs | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| syev | 0.1.0 | 0.2.46 | 4 | 0.2.48 (group A #66, merged; unpublished) |
| sysv | 0.1.0 | 0.2.46 | 6 | 0.2.48 (group A #66, merged; unpublished) |
| zgebrd | 0.1.0 | 0.2.46 | 5 | PENDING: #69 (next after #68) |
| zgecon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| zgeev | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| zgehrd | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zgelss | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| zgeqp3 | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| zggev | 0.1.0 | 0.2.46 | 6 | PENDING: #68 (0.2.50 armed) |
| zheevd | 0.1.0 | 0.2.46 | 2 | PENDING: #68 (0.2.50 armed) |
| zheevr | 0.1.0 | 0.2.46 | 4 | PENDING: #68 (0.2.50 armed) |
| zhegv | 0.1.0 | 0.2.46 | 3 | PENDING: #68 (0.2.50 armed) |
| zhetrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zhetri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zhetrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| zlacpy | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zlange | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| zlarfg | 0.1.0 | 0.2.46 | 1 | PENDING: #69 (next after #68) |
| zpocon | 0.1.0 | 0.2.46 | 1 | 0.2.49 (group B #67, merged; unpublished) |
| zsytrf | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zsytri | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zsytrs | 0.1.0 | 0.2.46 | 3 | 0.2.49 (group B #67, merged; unpublished) |
| zunglq | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| zungqr | 0.1.0 | 0.2.46 | 2 | 0.2.49 (group B #67, merged; unpublished) |
| zungtr | 0.1.0 | 0.2.46 | 2 | PENDING: #69 (next after #68) |
| zunmlq | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |
| zunmqr | 0.1.0 | 0.2.46 | 3 | PENDING: #69 (next after #68) |

120 families with at least one HOLE.

## aocl-data-analytics

Versions measured: 0.2.2, 0.2.4, 0.2.5, 0.2.28, 0.2.40, 0.2.43, 0.2.46, 0.2.49.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |
|---|---|---|---|---|
| linear_kernel | 0.2.2 | 0.2.49 | 6 | PENDING: #71 (kernels, 0.2.53) |
| pairwise_distances | 0.2.2 | 0.2.49 | 6 | PENDING: #71 (kernels, 0.2.53) |
| polynomial_kernel | 0.2.2 | 0.2.49 | 6 | PENDING: #71 (kernels, 0.2.53) |
| rbf_kernel | 0.2.2 | 0.2.49 | 6 | PENDING: #71 (kernels, 0.2.53) |
| sigmoid_kernel | 0.2.2 | 0.2.49 | 6 | PENDING: #71 (kernels, 0.2.53) |

5 families with at least one HOLE.

## aocl-fft

Versions measured: 0.1.0, 0.2.6, 0.2.40, 0.2.43, 0.2.46, 0.2.49.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |
|---|---|---|---|---|
| plan_dft_misaligned_input | 0.1.0 | 0.2.49 | 1 | PENDING: #72 (alignment, 0.2.54) |
| plan_dft_misaligned_output | 0.1.0 | 0.2.49 | 1 | PENDING: #72 (alignment, 0.2.54) |

2 families with at least one HOLE.

## Not measured (read pending)

sparse `blkcsrmv` (no safe mask constructor), `create_csc` / `create_tcsr` (raw handle, no safe sink); lapack band `gb*`, `gesvdx`, `orgbr`, `lasrt`, and the trait-level `RealScalar::geev` / `ComplexScalar::geev` called directly (the typed free functions are measured); blas trait methods with explicit `inc`/`ld`, pack/compute/batch/rotm/sdsdot/csrot/zdrot; and the crates data-analytics, crypto, fft, rng, scalapack, math, compression, securerng, utils, openrng. `aocl-sparse` 0.1.0 did not compile with the harness (2 errors in its own lib against current dependencies) and is unmeasured; the functions are present in its source.

