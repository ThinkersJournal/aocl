# aocl-sparse: behaviours the wrapper tests found (AOCL 6.0.68, Windows, 2026-11-27)

Found while writing `crates/aocl-sparse/tests/wrappers.rs` (every exposed wrapper function with a result-checked test). Measured
on the real library, Linux unmeasured. These are RECORDED here, not kept as `#[ignore]`d tests; none is reported to AMD
(standing ruling: not now). "Header" = the AOCL-Sparse documentation text, not read from a header file by this lane's clean-room
rules beyond what the tests quote.

1. **`set_value_c64` crashes the process (access violation).** The C `aoclsparse_double_complex` is `alignas(16)`; the
   generated `-sys` binding `aoclsparse_double_complex_` has no `align(16)`, so the by-value argument reaches the library
   8-byte aligned. `set_value_c32` (8 bytes) works; other by-value `Complex64` calls (symgs, trsv, dotmv) passed here but could be
   affected by the same defect. This one is OURS to fix (a `-sys` binding layout defect): needs its own row (give the type
   `#[repr(C, align(16))]` through the bindgen config and test the by-value calls). `update_values_c64` is tested.
2. **`symgs` ignores `alpha` when the initial guess is zero** (documented `op(A) x = alpha b`; alpha = 2 returned the alpha = 1 result:
   0.1629 instead of 0.3258). `symgs_mv` was only tested at alpha = 1.
3. **`s::sorv_f32` has no length check** (unlike `sorv<T>` and the complex `sorv`): a short `x` made the library read/write out of
   bounds and crash the process. The test skips the short-buffer case for that function.
4. **`complex::sorv` (c32/c64) always returns "not implemented"**, which the documentation states (SOR: single and double only); the
   test pins the error.
5. **`roti` accepts a negative index** although the documentation says `invalid_index_value`: `indx = [1, -1, 2]` returned Ok and heap
   corruption followed. That case is not in the suite (no isolated reproduction committed).
6. **Legacy `csrsv` with a one-based CSR returns "not implemented"** (undocumented); it needs a `General` descriptor and
   `Trans::No`. Tested: base zero; `Trans::T` is an error.
7. **TCSR handles cannot be exported or converted to CSR** (`export_csr` invalid_value, `convert_csr` not_implemented, symgs rejects
   them); only `trsv` works, so the safe types cannot wrap a TCSR handle. Probably by design.

Expected values for csr2ell (row-major slots), csr2dia (`val[d*m+i]`) and the strided gather (`x[i] = y[stride*i]`) are this lane's
reading of the documentation; all matched the library on the first run. The `version()` test only checks "AOCL-Sparse" and a "6."
substring.
