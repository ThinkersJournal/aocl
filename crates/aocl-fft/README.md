# aocl-fft

Safe Rust wrappers for AOCL-FFTW.

Built on top of [`aocl-fft-sys`](../aocl-fft-sys/).

Dual-licensed under MIT or Apache-2.0.

**Licence condition of the library you link.** The AMD `libfftw3`/`libfftw3f` in AOCL is GPL-licensed (AOCL's `NOTICE.txt` says GPL-2.0; the shipped `COPYRIGHT` says version 2 or any later version: the two disagree). This crate ships no library. A program that links it and is distributed must comply with the library's licence. Whether a source-only crate of `extern` declarations is itself a combined work is UNKNOWN (no FSF text found). This is not legal advice. See `THIRD-PARTY-NOTICES.md` and `docs/PROVENANCE-QUESTIONS.md` question 2.

### Affected versions (safe-API memory safety)

Measured per published version with a guard-page harness built against each `.crate`: executing a DFT plan on an array that is not 16-byte aligned like the array it was planned on (the plan was created in place but executed out of place) faulted (f64) in 0.1.0 through 0.2.49; fixed in 0.2.54 (plan scratch is aligned, executors refuse a misaligned array). The length checks were measured clean in every version. which of these versions are yanked is shown by crates.io, which is the authority (yanking and advisories are the maintainer's, not this file's). Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
