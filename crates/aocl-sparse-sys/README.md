# aocl-sparse-sys

Raw FFI bindings to AOCL-Sparse. For the safe API see [`aocl-sparse`](../aocl-sparse/).

**`static-link` (Windows):** links `aoclsparse` from the install's `static/` directory (it asked for a non-existent `aoclsparse-static` before). The static library needs the OpenMP runtime `libiomp5md.lib`, which this crate does NOT link: add it to your link line.

Dual-licensed under MIT or Apache-2.0.
