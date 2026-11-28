# aocl-sparse: length validation of the matrix-handle vector functions (2026-11-28)

Measured on Windows x64, AOCL 6.0.68 (LP64), `crates/aocl-sparse/tests/vector_lengths.rs` (Linux unmeasured). The library does not check that the dense vectors passed to `aoclsparse_?mv`, `?trsv`, `?symgs`, `?symgs_mv`, `?dotmv` are as long as the matrix needs. Each case calls a SAFE wrapper with a 3-element slice for a 4 x 4 matrix, the slice placed at the end of a committed page followed by a PAGE_NOACCESS page.

**Before the fix** (main at 0.2.43; one process per case; record `C:/Projects/.lane-state/sparse-vector-lengths-before.txt`): **36 of 44 cases ACCESS VIOLATION (0xc0000005)**: real `f64` and `f32` (22: every function, every short argument), complex `c64` and `c32` `symgs`, `symgs_mv`, `dotmv` (14). The other 8 were already refused with `InvalidArgument`: the complex `mv` and `trsv` (which validated before); that is the positive control that the cases detect a check where one exists. 4 control tests (exact lengths succeed, one per precision) passed before and after.

**After:** 48 of 48 tests pass, each in its own process; `cargo test -p aocl-sparse` 21 + 13 + 48 + 32 pass; clippy `--all-targets -D warnings` and fmt clean.

Rule added: `x`, `y`, `b` must be AT LEAST as long as the matrix needs (as `sorv` and the complex `mv`/`trsv` already did): `mv`/`dotmv`: `x >= n, y >= m` for `Trans::No`, swapped for `T`/`C`; `trsv`: `b, x >= m`; `symgs`: `b >= m, x >= n`; `symgs_mv`: also `y >= m`. A longer slice is accepted (the library reads the first elements).

**Not covered here (same defect class suspected, NOT demonstrated, listed so the next PR can take them):** the dense-matrix functions (`csrmm`, `trsm`, `spmmd`, `sp2md`, `syrkd`, `syprd`, real and complex: output sizes depend on `ldb`/`ldc`/`order`/`n_rhs`), and the array-argument functions `csrmv`, `ellmv`, `bsrmv`, `blkcsrmv`. The iterative solvers were not read.

## Dense-matrix arguments (`tests/dense_lengths.rs`, same method)

`csrmm`, `spmmd`, `sp2md`, `trsm`, `syrkd`, `syprd` (real and complex): 152 tests = 128 one-element-short cases + 24 exact-size controls (per precision: f64, f32, Complex64, Complex32; both layouts; `op` none and transpose; the complex `trsm` has a fixed row-major order). Before the fix (record `C:/Projects/.lane-state/sparse-dense-lengths-before.txt`): **128 of 128 ACCESS VIOLATION**; the 24 controls pass. After: 152 of 152 pass, each in its own process; `cargo test -p aocl-sparse` all pass. Rule: `ld >= minor extent` and `len >= (major - 1) * ld + minor` (column-major: major = cols, minor = rows). The complex `trsm` size was probed directly: 12 elements for a 4 x 3 row-major system is accepted, 11 faults.

Cleared by probe: `SparseMatrix::from_csr` rejects (the library refuses) all five malformed structures tried (column index past `n`, negative column, `row_ptr` entry past nnz, decreasing `row_ptr`, last `row_ptr` != nnz), so the handle-based functions cannot be sent past `x` by a bad matrix.
