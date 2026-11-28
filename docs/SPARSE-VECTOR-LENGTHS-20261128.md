# aocl-sparse: length validation of the matrix-handle vector functions (2026-11-28)

Measured on Windows x64, AOCL 6.0.68 (LP64), `crates/aocl-sparse/tests/vector_lengths.rs` (Linux unmeasured). The library does not check that the dense vectors passed to `aoclsparse_?mv`, `?trsv`, `?symgs`, `?symgs_mv`, `?dotmv` are as long as the matrix needs. Each case calls a SAFE wrapper with a 3-element slice for a 4 x 4 matrix, the slice placed at the end of a committed page followed by a PAGE_NOACCESS page.

**Before the fix** (main at 0.2.43; one process per case; record `C:/Projects/.lane-state/sparse-vector-lengths-before.txt`): **36 of 44 cases ACCESS VIOLATION (0xc0000005)**: real `f64` and `f32` (22: every function, every short argument), complex `c64` and `c32` `symgs`, `symgs_mv`, `dotmv` (14). The other 8 were already refused with `InvalidArgument`: the complex `mv` and `trsv` (which validated before); that is the positive control that the cases detect a check where one exists. 4 control tests (exact lengths succeed, one per precision) passed before and after.

**After:** 48 of 48 tests pass, each in its own process; `cargo test -p aocl-sparse` 21 + 13 + 48 + 32 pass; clippy `--all-targets -D warnings` and fmt clean.

Rule added: `x`, `y`, `b` must be AT LEAST as long as the matrix needs (as `sorv` and the complex `mv`/`trsv` already did): `mv`/`dotmv`: `x >= n, y >= m` for `Trans::No`, swapped for `T`/`C`; `trsv`: `b, x >= m`; `symgs`: `b >= m, x >= n`; `symgs_mv`: also `y >= m`. A longer slice is accepted (the library reads the first elements).

**Not covered here (same defect class suspected, NOT demonstrated, listed so the next PR can take them):** the dense-matrix functions (`csrmm`, `trsm`, `spmmd`, `sp2md`, `syrkd`, `syprd`, real and complex: output sizes depend on `ldb`/`ldc`/`order`/`n_rhs`), and the array-argument functions `csrmv`, `ellmv`, `bsrmv`, `blkcsrmv`. The iterative solvers were not read.

## Raw-array routines (`tests/raw_array_lengths.rs`, same method)

`csrmv`, `csrsv`, `ellmv`, `csr2ell_f64/f32`, `csr2dia_f64/f32` take the CSR / ELL / DIA arrays directly, so `create_csr`'s validation does not apply. 72 tests (f64 and f32): 10 controls (valid call) and 62 broken-input cases. Before (record `C:/Projects/.lane-state/sparse-raw-array-before.txt`): **28 ACCESS VIOLATION, 10 accepted silently, 24 refused**. Per function (f64; f32 identical): `csrmv` column past `n` and `row_ptr` past nnz AV, negative column and decreasing `row_ptr` accepted, five length cases refused; `csrsv` short `col_ind` and short `val` AV, column past `m` and `row_ptr` past nnz accepted, short `row_ptr`/`x`/`y` refused; `ellmv` column past `n` AV, four length cases refused; `csr2ell` all five cases AV; `csr2dia` four AV and a one-short `dia_val` accepted (the library writes only the in-matrix diagonal entries, so the last slots are untouched; `m*ndiag` is the documented size and is now required). After: 72 of 72 pass, each in its own process. `csr2ell` padding probed: `col = -1, val = 0` (kept legal for `ellmv`).

Remaining families of the sparse audit (not covered by #62, #63 or this PR): `bsrmv`, `blkcsrmv`, `csr2bsr`, `csr2bsr_nnz`, `create_csc`, `create_tcsr` (real and complex), `solve`; `update_values` and the sorv/ILU/roti/index-vector families already validated.
