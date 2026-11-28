# aocl-openrng

Safe Rust wrappers for AOCL-OpenRNG (AMD's fork of Arm's OpenRNG; the VSL-compatible random number API), on top of
[`aocl-openrng-sys`](../aocl-openrng-sys/). `publish = false` until its `-sys` crate is read and published.

* `Stream` (create from a seed, clone, copy state, leapfrog, skip-ahead, save/restore), 10 generators (`Brng`), uniform bits,
  15 continuous distributions in `f64` and `f32`, 9 integer distributions, multivariate Gaussian, multinomial.
* Lengths the library derives from other arguments are checked in Rust; arbitrary-bytes loading and the `n`-parameter initialisers
  are `unsafe` until measured (`docs/cleanroom/openrng/WRAPPER-HANDOFF.md`).
* **Nothing is measured by the lane**: the library is a PM-built DLL. Linux unmeasured. The `ilp64` feature must match the library build.

Dual-licensed under MIT or Apache-2.0. See `NOTICE`.
