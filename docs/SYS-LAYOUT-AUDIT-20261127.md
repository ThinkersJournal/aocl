# aocl-*-sys struct layout audit (2026-11-27): Windows x64, AOCL 6.0.68 only

**Defect class.** A header declares a struct with an alignment attribute, the header (or the compiler mode) hides the attribute from
bindgen, so the generated Rust struct has the natural alignment while the library was built with the attribute. Structs passed BY
VALUE, or as arrays, then fault or corrupt. Found in `aocl-sparse-sys` (`aoclsparse_double_complex`, `alignas(16)`, fixed in the alignment PR #44).

**Scope and refs.** Branch `lane/sys-layout-audit` on main (the sparse rows assume the alignment fix of PR #44). AOCL 6.0.68 Windows install,
x86_64, MS x64 ABI, clang 22 from VS 18 (`clang++ -std=c++17`, `clang -std=gnu17`, `-msse4.2 -mavx2 -mfma`, same `-I` and ILP64
`-D` as `aocl-build`). **Linux was not measured.** LP64 and ILP64 were both built and probed for every crate that has the
`ilp64` feature (blas, lapack, sparse, data-analytics): every size and alignment below is identical in both.

## Method

1. Bindings: `cargo build -p aocl-X-sys` (LP64) and `--features ilp64`; the generated `OUT_DIR/bindings.rs` was parsed for every
   `struct`/`union`, and every declared function and callback typedef for the type used by value, or behind a pointer (a
   pointer parameter is an array when the C parameter is `T*` of several elements: reported as "pointer/array").
2. Rust size/align: a throw-away `rustc` probe `include!`-ing that `bindings.rs` and printing `size_of`/`align_of` of every type.
3. C and C++ size/align: a probe program including the crate's `wrapper/*.h`, printing `sizeof`/`alignof` of the same names,
   compiled once as C++ and once as C. (`clang++ -fdump-record-layouts` was not needed: the probe gives size and alignment, the
   two numbers the defect class changes; field order comes from the one header the bindings are generated from.)
4. Header grep for `alignas`, `_Alignas`, `aligned(`, `declspec(align`, `pragma pack`, `__m128/__m256/__m512`, and a read of the
   `#ifndef __cplusplus ... #define alignas` style neutralisation.
5. Positive control for the method: the probe reports `aoclsparse_double_complex` as Rust 16/16 = C++ 16/16 but **C 16/8** (and the
   float one C++ 8/8, C 8/4): the probe sees exactly the defect this audit looks for, in the crate where it was known.
6. Runtime checks where a type is passed by value to the library or its layout is not stated by one number
   (`crates/aocl-lapack-sys/tests/by_value_complex.rs`, `crates/aocl-blas-sys/tests/rntm_layout.rs`).

## Result

**MISMATCH outside sparse: none.** 124 struct/union types in the generated bindings (plus the generic LAPACKE complex
instantiations): 0 MISMATCH still open, 2 MISMATCH already fixed (sparse), 0 UNKNOWN in the bindgen-generated crates, 8 opaque
(incomplete) types that are only ever pointed to. The UNKNOWNs are in the two hand-written
clean-room crates (below).

| crate | types | OK | MISMATCH | opaque | UNKNOWN |
|---|---|---|---|---|---|
| aocl-blas-sys | 61 | 61 | 0 | 0 | 0 |
| aocl-lapack-sys | 2 (`__BindgenComplex<f32>`, `<f64>`) | 2 | 0 | 0 | 0 |
| aocl-fft-sys | 6 | 3 | 0 | 3 (`fftw_plan_s` and two more: opaque (a handle type), NOT compared: only ever pointed to) | 0 |
| aocl-sparse-sys | 5 | 0 | 2 (fixed by the alignment PR #44) | 3 (`_aoclsparse_*`: opaque (a handle type), NOT compared: only ever pointed to) | 0 |
| aocl-utils-sys | 10 | 10 | 0 | 0 | 0 |
| aocl-compression-sys | 26 | 26 | 0 | 0 | 0 |
| aocl-crypto-sys | 14 | 14 | 0 | 0 | 0 |
| aocl-data-analytics-sys | 2 | 0 | 0 | 2 (`_da_handle`-style handle types: opaque (a handle type), NOT compared: only ever pointed to) | 0 |
| aocl-securerng-sys | 0 | 0 | 0 | 0 | 0 |
| aocl-math-sys (hand-written) | 2 (`fc32_t`, `fc64_t`) + 156 vector fns | 0 | 0 | 0 | 2 + 156 (UNKNOWN: clean-room, no header read; the 156 vector fns are behind the nightly `simd_ffi` feature, not run) |
| aocl-rng-sys, aocl-scalapack-sys (hand-written) | 0 struct types | - | - | - | - |

## Where a header states an alignment (every one found, all `amd-*` headers of the wrappers' components)

| header : line | what | in the bindings? | verdict |
|---|---|---|---|
| `amd-sparse/include/aoclsparse_types.h:61-64,102` | `#ifndef __cplusplus / #define alignas(s)` ... `#undef alignas` (comment: "C standard doesn't allow alignment on the typedef of the sructure") | yes | the defect; fixed in 5b6b0cb |
| `aoclsparse_types.h:77` / `:94` | `typedef struct alignas(2 * sizeof(float))` / `alignas(2 * sizeof(double)) aoclsparse_double_complex_` | yes (by value, 18 fns each) | Rust now 8/8 and 16/16 = C++ |
| `amd-blis/include/{LP64,ILP64}/blis.h:2428-2442` (second copy `:55462-55476`), `cblas.h:2425-2439` | `#define BLIS_ALIGN 64`; `BLIS_ATTRIB_ALIGN` = `__attribute__((aligned(64)))` on non-Windows, on `_WIN32` **only if `__clang__`**; `typedef struct BLIS_ATTRIB_ALIGN rntm_s` | yes, `rntm_t` `repr(align(64))` (bindgen is clang) | OK; see rntm_t below |
| `amd-crypto/include/alcp/cipher.h:79-81` | three `__attribute__((aligned(64))) Uint64 [64]` members of `alc_cipher_state_t` | yes, struct `repr(align(64))`, size 1600 | OK (C = C++ = Rust) |
| `amd-compression/include/7zTypes.h:92-101` | `MY_ALIGN(n)` macro (`__declspec(align)` / `aligned`) | defined, never used by a type in the included headers | no effect |
| `amd-utils/include/Capi/au/cpuid/au_cpuid_header_only.h:140-143`, `Au/Defs.hh:35-38` | `AUD_ALIGNED(x)` macro | defined, never used by any type (`Au/*.hh` is C++ and not in the wrapper) | no effect |
| `amd-libflame/include/*/FLAME.h:2904,2908,2916` (`aligned(FLA_CACHE_LINE_SIZE_BYTES)`), `:1181,1187,2146,...` (`__m128`/`__m128d` unions) | libFLAME native API | **no**: `wrapper/lapack.h` deliberately does not include `FLAME.h` | not bound; the defect class WOULD apply if it were |
| `amd-libm/include/amdlibm_vec.h` (`__m128`...`__m512d` by value, 156 fns) | vector LibM | only behind the nightly `simd_ffi` feature of aocl-math-sys | UNKNOWN (not run), below |
| `amd-fftw`, `amd-rng`, `amd-securerng`, `amd-da` headers | none found: no `alignas`, `aligned(`, `pragma pack`, `__m*` | - | nothing to lose |

`#pragma pack`, `_Alignas`, `__declspec(align` (outside `MY_ALIGN`): 0 hits in all of `amd-blis amd-compression amd-crypto amd-da
amd-fftw amd-fftz amd-libflame amd-libm amd-rng amd-securerng amd-sparse amd-utils` headers (the same grep finds the sparse and
BLIS lines above, so it works).

## The by-value types, specifically

| crate | type | declared functions using it BY VALUE | C++ | C | Rust | verdict |
|---|---|---|---|---|---|---|
| sparse | `aoclsparse_double_complex` (16/16) / `_float_` (8/8) | 18 each (`aoclsparse_zadd`, `_zcsrmm`, `_zset_value`, ...) | 16/16, 8/8 | 16/8, 8/4 | 16/16, 8/8 | MISMATCH, fixed 5b6b0cb (+ const asserts, tests) |
| lapack | `__BindgenComplex<f64>` / `<f32>` (`double _Complex` / `float _Complex`, `lapack.h:50,68`; `lapacke.h:154`) | 16 fns `LAPACKE_{c,z}{larfx,laset,syr,tfsm}[_work]` (args) + `lapack_make_complex_{float,double}` (return) | 16/8, 8/4 (`std::complex`, `-DLAPACK_COMPLEX_CPP`; measured with `lapack_complex_*` sizeof/alignof) | 16/8, 8/4 (`_Complex`) | 16/8, 8/4 | OK; called with by-value args on the real library, LP64 and ILP64 (tests/by_value_complex.rs, 6 tests pass; control with a swapped expectation FAILS 2 of 6) |
| blas | `bli_pthread_t` | 1 (`bli_pthread_join`, static-link only) | 16/8 | 16/8 | 16/8 | OK by layout; not run (needs static BLIS + threads) |
| blas | `scomplex` (8/4), `dcomplex` (16/8) | none by value (621 / 629 pointer/array params, e.g. `bli_zaxpyv`) | same | same | same | OK |
| fft | `fftw_complex` is `double[2]` (not a struct), `fftw_iodim` (12/4), `fftw_iodim64` (24/8) | arrays of iodim in 21 plan fns each | same | same | same | OK |
| math (hand-written) | `fc32_t` (8/4), `fc64_t` (16/8) | 8 `amd_c*` fns; `fc32_t` in a register, `fc64_t` by hidden reference | no header read (clean-room) | - | 8/4, 16/8 | UNKNOWN (no header states it; ABI measured behaviourally, `docs/cleanroom/types/TYPES-MEASUREMENT.md`, and `tests/link_all.rs` calls `amd_cexp`/`amd_cpow`/`amd_clog`) |

### `rntm_t` (BLIS), the one compiler-conditional alignment

`BLIS_ATTRIB_ALIGN` is non-empty on Windows only for clang, so a header parsed by clang (bindgen) says 64; if the DLL had been
built with MSVC `cl` it would say nothing. Evidence the library has the header's clang layout: (a) the static BLIS objects
reference LLVM OpenMP symbols (`__kmpc_global_thread_num`, link error LNK2019 without `libiomp5md.lib`), i.e. they are
clang-built; (b) `tests/rntm_layout.rs` (feature `static-link`, needs `-l libiomp5md`): a 192-byte `rntm_t` filled with 0x5A, then
`bli_rntm_init_from_global`: all 192 bytes were overwritten by the library, `num_threads` (offset 8) = 32 =
`bli_thread_get_num_threads()`, bool fields in {0,1}: a library with another layout would leave poison at the offsets Rust reads.
It is passed by pointer only (412 declarations), never by value.

### BLIS: C vs C++ views of the same header differ (7 types), harmless here

In C, `blis.h:783-785` does `typedef gint_t bool;`; in C++ `bool` is the language's 1-byte `bool`. Hence `mbool_s` 32/8 (C) vs 4/1
(C++), `cntx_s` 8216 vs 7544, `rntm_s` 192 vs 128, `thrinfo_s` 64 vs 56, `packm_params_s` 56 vs 40, `gemmtiny_ukr_info_t`
48 vs 40, `lpgemm_jit_inputs_t` 48 vs 32. BLIS is a C library, bindgen parses C, Rust = C for all 61 types, so this is **not a
defect of the crate**; it would bite a C++ consumer of `blis.h` (AMD's own `blis.hh`), which is not ours.

## What was changed

- `crates/aocl-lapack-sys/tests/by_value_complex.rs` (new): 6 tests, by-value complex arguments and return on the real library.
- `crates/aocl-blas-sys/tests/rntm_layout.rs` (new, feature `static-link`): the runtime `rntm_t` layout check above.
- `const _: () = assert!(size_of/align_of ...)` blocks in `src/lib.rs` of aocl-blas-sys (`scomplex`, `dcomplex`, `rntm_t`),
  aocl-lapack-sys (`__BindgenComplex<f32|f64>`), aocl-fft-sys (`fftw_iodim`, `fftw_iodim64`), aocl-crypto-sys
  (`alc_cipher_state_t` 64/1600). The vendored `bindings/bindings.rs` (docs.rs path) of those four also satisfies the asserts.
- No generated-code fix was needed outside sparse.

## UNKNOWN (findings, not zeros)

1. `aocl-math-sys` `fc32_t` / `fc64_t`: no header was read (clean-room), alignment of the C complex types is not stated by any
   document in the crate; Rust uses 4 and 8. Behaviour tested (`link_all.rs` calls them), layout not stated. If `amd_cexp` required
   16-byte alignment of its by-reference `fc64_t` copy it would fault on some call sites, none observed.
2. `aocl-math-sys` 156 vector functions (`__m128`..`__m512d` by value): behind the nightly `simd_ffi` feature, never default; not
   compiled or run in this audit. Their alignment is the intrinsic types' (16/32/64), which Rust's `core::arch` types also have.
3. Linux: every number above is Windows x64; `long double`, `_Complex` and `bool` ABIs differ and are not measured.
4. LibFLAME `FLAME.h` native API and AOCL-FFTZ headers are not bound by any crate (see table), so not audited.
