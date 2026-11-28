# aocl-sparse-sys

Raw FFI bindings to AOCL-Sparse. For the safe API see [`aocl-sparse`](../aocl-sparse/).

**`static-link` (Windows):** links `aoclsparse` from the install's `static/` directory (it asked for a non-existent `aoclsparse-static` before). The static library needs the OpenMP runtime `libiomp5md.lib`, which this crate does NOT link: add it to your link line.

**`static-link` needs more on the link line (Windows, AOCL 6.0.68, measured; this crate does not link it).** The static `aoclsparse` library references the OpenMP runtime (`__kmpc_*`): add `libiomp5md.lib` (Intel oneAPI `compiler/<version>/lib`, which must be on the `LIB` path). With it on the link line all 286 declared functions resolve.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4, 0.2.5 and 0.2.11 are yanked: the complex types `aoclsparse_double_complex` / `aoclsparse_float_complex` lacked the header's alignment, so by-value calls such as `aoclsparse_zset_value` faulted (access violation). Fixed in 0.2.26.
