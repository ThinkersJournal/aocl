# aocl-blas

Safe Rust wrappers for AOCL-BLAS (BLIS) via the CBLAS interface.

Built on top of [`aocl-blas-sys`](../aocl-blas-sys/). Uses [`aocl-types`](../aocl-types/) for `Layout` / `Trans` and [`aocl-error`](../aocl-error/) for the shared `Error` type.

Dual-licensed under MIT or Apache-2.0.

### Affected versions (safe-API memory safety)

Measured per published version with a guard-page harness built against each `.crate`: `gemmt` read or wrote past the caller's matrices in 0.1.0 through 0.2.46 (fixed in 0.2.49); `?gemm_pack`, `?gemm_compute`, `cgemm3m` and `zgemm3m` (14 faulting cases) in all of 0.1.0 through 0.5.4, fixed in 0.6.1, where they now return `Result<()>` (BREAKING: they returned `()`). The trait methods with explicit layout / leading dimension / increment were measured clean in every version (259 tests). NOT MEASURED: the f32 / Complex32 copies of the trait methods, `?gemm_pack` / `?gemm_compute` flag combinations other than NoTrans / Packed, `?gemm_batch` (`unsafe`). Which of these versions are yanked is shown by crates.io, which is the authority (yanking and advisories are the maintainer's, not this file's). Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
