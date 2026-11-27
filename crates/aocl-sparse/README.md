# aocl-sparse

Safe Rust wrappers for AOCL-Sparse (sparse BLAS).

Built on top of [`aocl-sparse-sys`](../aocl-sparse-sys/). Uses the shared [`aocl-types::Trans`](../aocl-types/) for matrix-operation enums.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4 and 0.2.5 are yanked: `sorv_f32` had no length check and `roti` no index validation (out-of-bounds access from safe code). Fixed in 0.2.26 (`sorv_f32`, `roti`; the sibling index-array functions follow in a later patch).
