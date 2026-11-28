# Safe-API audit: first / last affected PUBLISHED version per crate and function family

Measured, not read: for every published version of the crate the guard-page harness (`tools/gen-blas-harness.py`, `tools/gen-lapack-harness.py`, and the hand-written sparse files `vector_lengths.rs`, `dense_lengths.rs`, `raw_array_lengths.rs`) was built against that version's own `.crate` and dependencies and every test run in its own process (`tools/safe-api-versions.py`). HOLE = the process died with an access violation on a call with one buffer one element short. Result files: `results-*`/`<crate>-<version>*.csv` in this directory. A family not listed was refused or accepted without fault in every version measured. `gesv` (lapack) is the only measured LAPACK function that never faults. Yank and GHSA are CireSnave's (board 526 B / 527).

## aocl-sparse

Versions measured: 0.1.0, 0.2.2, 0.2.4, 0.2.5, 0.2.26, 0.2.40, 0.2.43, 0.2.46.

| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |
|---|---|---|---|---|
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

17 families with at least one HOLE.

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
| cgecon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| cgehrd | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| cgelss | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| cgeqp3 | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| chetrf | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| chetri | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| chetrs | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| clacpy | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| clange | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| cpocon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| csytrf | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| csytri | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| csytrs | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| cungqr | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dgecon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dgehrd | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dgelss | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dgeqp3 | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dlacpy | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dlange | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dorgqr | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dpocon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dpotri | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dsytrf | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dsytri | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dsytrs | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dtrtri | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| dtrtrs | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| gelqf | 0.1.0 | 0.2.46 | 4 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| geqrf | 0.1.0 | 0.2.46 | 4 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| getrf | 0.1.0 | 0.2.46 | 2 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| getri | 0.1.0 | 0.2.46 | 2 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| getrs | 0.1.0 | 0.2.46 | 6 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| gtsv | 0.1.0 | 0.2.46 | 6 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| heev | 0.1.0 | 0.2.46 | 4 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| hesv | 0.1.0 | 0.2.46 | 4 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| posv | 0.1.0 | 0.2.46 | 2 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| potrf | 0.1.0 | 0.2.46 | 2 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| potrs | 0.1.0 | 0.2.46 | 2 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| ptsv | 0.1.0 | 0.2.46 | 8 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| sgecon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| sgehrd | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| sgelss | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| sgeqp3 | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| slacpy | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| slange | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| sorgqr | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| spocon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| spotri | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| ssytrf | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| ssytri | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| ssytrs | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| strtri | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| strtrs | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| syev | 0.1.0 | 0.2.46 | 4 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| sysv | 0.1.0 | 0.2.46 | 6 | PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods) |
| zgecon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zgehrd | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zgelss | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zgeqp3 | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zhetrf | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zhetri | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zhetrs | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zlacpy | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zlange | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zpocon | 0.1.0 | 0.2.46 | 1 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zsytrf | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zsytri | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zsytrs | 0.1.0 | 0.2.46 | 3 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |
| zungqr | 0.1.0 | 0.2.46 | 2 | PENDING: group B branch lane/lapack-typed-lengths (opens after #66) |

70 families with at least one HOLE.

## Not measured (read pending)

sparse `bsrmv`, `blkcsrmv`, `csr2bsr(_nnz)`, `create_csc`, `create_tcsr`, `solve`; lapack band / `geev` / `gesvd` / `gesdd` / `gelsd` / `gelsy` / `syevd` / `syevr` / `gesvdx` / `ggev` / `gees` / `sygv` / `orm*` / `orgtr` / `orgbr` / `orglq` / `larfg` / `lasrt` / `gebrd`; blas trait methods with explicit `inc`/`ld`, pack/compute/batch/rotm/sdsdot/csrot/zdrot; and the crates data-analytics, crypto, fft, rng, scalapack, math, compression, securerng, utils, openrng. `aocl-sparse` 0.1.0 did not compile with the harness (2 errors in its own lib against current dependencies) and is unmeasured; the functions are present in its source.

