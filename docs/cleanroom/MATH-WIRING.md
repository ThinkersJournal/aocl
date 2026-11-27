# aocl-math-sys / aocl-math wired from the clean-room declarations (2026-11-27)

`aocl-math-sys` no longer runs bindgen and reads no AOCL header: `build.rs` only links `libalm` from the installed AOCL
(link-only mode of `aocl_build::Component`, no libclang needed). Its declarations are `src/decls_scalar.rs`, generated
mechanically (split, then `cargo fmt`; no hand edit) from `math_decls.rs`, which the clean-room writer produced from the
guide text only. Provenance: `math_provenance.csv`, whose new last column `wired` says what happened to each of the 383 rows.

| wired | rows | why |
|---|---|---|
| yes | 218 | scalar and array-form functions |
| no | 156 | by-value `__m128`..`__m512d` arguments: stable Rust rejects SIMD types in `extern "C"` (needs nightly `simd_ffi`; checked with rustc 1.100.0, hard error) |
| no | 8 | guide names `fc32_t`/`fc64_t`/`long double` but never defines them (`math-excluded-from-wiring.txt`) |
| no | 1 | `amd_vrd2_sincos`: UNKNOWN (garbled guide line L108016) |

## Breaking list (vs current main, which is bindgen over `amdlibm.h`+vector items)
**aocl-math (safe wrapper), public functions removed (8):** `cexp`, `cexpf`, `cpow`, `cpowf`, `cln`, `clnf`, `nexttoward`,
`nexttowardf`. Every other public function (and its signature) is unchanged: before/after `pub fn` lists differ only by
these 8. `Complex32`/`Complex64` re-exports stay. A published 0.1.0 of the wrapper is not affected by this repo change.
**aocl-math-sys, items removed:** the 313 old `pub fn` minus the wired 218 = **115 names**, in
`math-sys-removed-vs-current-bindings.txt` (8 of them are the ones above; the rest are the `amd_vrd2_*`, `amd_vrd4_*`,
`amd_vrs4_*`, `amd_vrs8_*` SIMD-argument functions that the old bindgen output declared with `__m128`/`__m256` union types).
Also removed: the bindgen helper types `_Dcomplex`, `_Fcomplex`, `_Lcomplex`, `fc32_t`, `fc64_t`, `fc128_t`,
`_C_*_complex`, `__m128*`, `__m256*`. The `libc` dependency (unused) is dropped. Added: nothing public except the 218 functions
(all were already in the old set).

## Tests (installed AOCL-LibM 6.0.68, Windows, LP64, host x86_64-pc-windows-msvc)
`cargo test -p aocl-math -p aocl-math-sys`: 11 wrapper tests + 2 `tests/smoke.rs` pass, linking the real `libalm`; `cargo fmt --check` and
`cargo clippy --all-targets -D warnings` clean; `DOCS_RS=1 cargo check -p aocl-math` builds. The guide's LibM chapter prints no
known-answer values (signatures only, checked L106064-L111896 for example/output text), so there are none to quote; the
smoke test uses values exact by definition (`sin 0`, `sqrt 4`, ...), not values copied from AMD.
Not run: Linux; the self-hosted rig (this was the lane's own Windows host).
