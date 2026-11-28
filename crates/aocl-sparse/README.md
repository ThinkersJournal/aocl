# aocl-sparse

Safe Rust wrappers for AOCL-Sparse (sparse BLAS).

Built on top of [`aocl-sparse-sys`](../aocl-sparse-sys/). Uses the shared [`aocl-types::Trans`](../aocl-types/) for matrix-operation enums.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4 and 0.2.5 are yanked: `sorv_f32` had no length check and `roti` no index validation (out-of-bounds access from safe code). Fixed in 0.2.26 (`sorv_f32`, `roti`; the sibling index-array functions follow in a later patch).

### Affected versions (safe-API memory safety)

Measured per published version with a guard-page harness built against each `.crate` (one buffer one element short, or one index out of range, must be refused): 19 function families of the CSR / CSC / ELL / DIA kernels and conversions read or wrote past the caller's arrays in every version from 0.1.0 (0.1.0 by source read, it does not build with the harness) through 0.2.43; the block-CSR (`bsrmv`, `csr2bsr`, `csr2bsr_nnz`) family through 0.2.49 (fixed in 0.2.54: 17 families first measured clean in 0.2.46). `blkcsrmv_f64` (block-CSR arrays; 7 faulting cases) is affected in every published version through 0.6.1 (0.1.0 by source read) and fixed in 0.6.2 (measured on the published `.crate`: 0 faults); 0.6.1 and every earlier version are yanked. NOT MEASURED: `create_csc`, `create_tcsr` (they return a raw library handle, there is no safe sink). The yanked versions were yanked because of these defects. Which of these versions are yanked is shown by crates.io, which is the authority (yanking and advisories are the maintainer's, not this file's). Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
