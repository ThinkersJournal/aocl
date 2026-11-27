# aocl-fft

Safe Rust wrappers for AOCL-FFTW.

Built on top of [`aocl-fft-sys`](../aocl-fft-sys/).

Dual-licensed under MIT or Apache-2.0.

**Licence condition of the library you link.** The AMD `libfftw3`/`libfftw3f` in AOCL is GPL-licensed (AOCL's `NOTICE.txt` says GPL-2.0; the shipped `COPYRIGHT` says version 2 or any later version: the two disagree). This crate ships no library. A program that links it and is distributed must comply with the library's licence. Whether a source-only crate of `extern` declarations is itself a combined work is UNKNOWN (no FSF text found). This is not legal advice. See `THIRD-PARTY-NOTICES.md` and `docs/PROVENANCE-QUESTIONS.md` question 2.
