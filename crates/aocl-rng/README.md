# aocl-rng

Safe Rust wrappers for AOCL-RNG.

Built on top of [`aocl-rng-sys`](../aocl-rng-sys/).

Dual-licensed under MIT or Apache-2.0.

### Affected versions (safe-API memory safety, and one breaking change)

Measured per published version with a guard-page harness built against each `.crate`: `multinormal`, `multi_students_t` and `multinomial` (10 of 10 faulting cases) in 0.1.0 through 0.2.54, fixed in 0.4.0; `general_discrete` accepted any slice for a library-written reference vector in the same versions: it now takes a `DiscreteReference` built only by the library's `*_reference` set-up functions (BREAKING; the old slice entry is `unsafe fn general_discrete_raw`). which of these versions are yanked is shown by crates.io, which is the authority (yanking and advisories are the maintainer's, not this file's). Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
