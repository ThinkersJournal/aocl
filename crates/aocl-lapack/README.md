# aocl-lapack

Safe Rust wrappers for AOCL-LAPACK (libFLAME) via the LAPACKE C interface.

Built on top of [`aocl-lapack-sys`](../aocl-lapack-sys/). Uses [`aocl-types`](../aocl-types/) for `Layout` and [`aocl-error`](../aocl-error/) for `Error`.

Dual-licensed under MIT or Apache-2.0.

### Affected versions (safe-API memory safety)

Measured per published version with a guard-page harness built against each `.crate`: 370 one-element-short cases in 120 function families (generic and typed solvers, eigen, factorisation, SVD, least squares) faulted in every version from 0.1.0 (source read) through 0.2.43; the typed families through 0.2.46, the eigen / factor families through 0.2.54 (fixed in 0.2.49 for 70 families and 0.2.54 for 50); the band solvers `gbsv` / `gbtrf` / `gbtrs`, the direct trait `geev`, `gesvdx` and `orgbr` / `ungbr` (23 faulting cases) in every version through 0.5.2, fixed in 0.5.3. NOT MEASURED: `?lasrt`, the f32 / Complex32 copies of the band and `gesvdx` functions. The yanked versions were yanked because of these defects. Which of these versions are yanked is shown by crates.io, which is the authority (yanking and advisories are the maintainer's, not this file's). Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
