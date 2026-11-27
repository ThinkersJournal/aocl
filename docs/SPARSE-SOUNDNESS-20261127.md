# aocl-sparse: soundness findings, affected versions, reproductions (2026-11-27)

Found while writing the wrapper tests (PR #43). Platform: Windows 11, AOCL 6.0.68; **Linux unmeasured**. Nothing reported to AMD
(standing ruling). RustSec is not filed (bots may not); a GHSA candidate and the yank question are the PM's board item.

Affected versions were read from the **published `.crate` files** (downloaded from crates.io on 2026-11-27 and inspected, not the
branch): `aocl-sparse` 0.1.0, 0.2.2, 0.2.4, 0.2.5 (the only published versions; none yanked); `aocl-sparse-sys` 0.1.0, 0.2.2,
0.2.4, 0.2.5, 0.2.11 (0.2.4 not downloaded; same file set as its neighbours is not claimed). Latest published as of that read:
`aocl-sparse` 0.2.5, `aocl-sparse-sys` 0.2.11.

| # | Bug | Kind | First published version with the code path | Latest published still has it? | Fixed in (this branch / PR) |
|---|---|---|---|---|---|
| 1 | `aoclsparse_double_complex` / `aoclsparse_float_complex` bindings lack the header's `alignas(16)` / `alignas(8)`; the by-value `z*` calls fault (`aoclsparse_zset_value`: access violation) | memory safety, `-sys` | `aocl-sparse-sys` 0.1.0 (binding is `#[repr(C)]`, `aoclsparse_zset_value` declared; same in 0.2.2, 0.2.5, 0.2.11) | **yes**, 0.2.11 | `lane/sparse-align16` (separate PR) |
| 2 | `sorv_f32` has no length check (`sorv<T>` and the complex `sorv` do): a short `x`/`b` is read/written out of bounds | memory safety, safe API | `aocl-sparse` 0.1.0 (read: no `dims()` check in `sorv_f32`; same in 0.2.2, 0.2.4, 0.2.5) | **yes**, 0.2.5 | this PR |
| 3 | `roti_f64`/`roti_f32` accept a negative or too-large index and fewer indices than values; the library then writes outside `y` (heap corruption) or reads past `indx` (access violation) | memory safety, safe API | `aocl-sparse` 0.1.0 (no validation; same in 0.2.2, 0.2.4, 0.2.5) | **yes**, 0.2.5 | this PR |
| 4 | `symgs`: `alpha` has no effect from a zero initial guess although the documentation says `op(A) x = alpha b` | semantic, not memory safety | `aocl-sparse` 0.1.0 (`symgs_f64` present) | yes | documented (measurement below); not "fixed": the library's behaviour is passed through |

## Safe-code-only reproductions (no `unsafe`)

`crates/aocl-sparse/tests/soundness.rs` (6 tests, one per case, run each with `-- --exact <name>`). **Red before the fix**
(measured on origin/main, Windows): `sorv_f32` short `x` and short `b` returned `Ok` (out-of-bounds access, no error);
`roti_f64` with `indx = [1, -1, 2]` returned `Ok` (heap write outside `y`), `indx = [0, 3]` into a 3-vector returned `Ok`;
`roti_f32` with 3 values and 2 indices exited with `STATUS_ACCESS_VIOLATION`; the valid-call control passed. **Green after the
fix:** all six pass; the rejected `roti` call leaves `y` untouched; a valid rotation still gives the hand-computed values.
Bug 1 has its own red-first test in `crates/aocl-sparse-sys/tests/by_value_complex.rs` (it needs `unsafe`: `-sys` is the raw
layer): before the fix `zset_value_by_value` exits `STATUS_ACCESS_VIOLATION`; after, all five by-value tests pass (LP64 and ILP64).

## `symgs` alpha, measured

`aoclsparse_dsymgs` on `[[4,1],[1,3]]` (symmetric, lower), `b = [1, 2]`:

| alpha | x0 | result |
|---|---|---|
| 1 | [0, 0] | [0.10416666666666666, 0.5833333333333334] |
| 2 | [0, 0] | [0.10416666666666666, 0.5833333333333334] (same) |
| 3 | [0, 0] | [0.10416666666666666, 0.5833333333333334] (same) |
| 1 | [1, 1] | [0.08333333333333334, 0.6666666666666666] |
| 2 | [1, 1] | [0.0625, 0.75] |
| 0.5 | [1, 1] | [0.09375, 0.625] |

So `alpha` is ignored for a zero guess and acts non-linearly otherwise. The wrapper docs now say so. Not a memory-safety issue.

## Same class, not fixed here (source read, NOT demonstrated by a crash)

Many other safe `pub fn`s in the wrapper pass `slice.as_ptr()` to the library without comparing lengths to the matrix
dimensions (for example `mv_f64`, `symgs_f64`, `symgs_mv_*`). A short slice is then out-of-bounds. This is the same defect class
as bug 2 and is a candidate for one validation pass over the whole safe API (a row for pm-kernels); no claim is made about which
of them actually fault.
