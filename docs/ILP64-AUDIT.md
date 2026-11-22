# ILP64 audit of every `ilp64` feature (2026-11-22, lane aocl)

Task from the cluster PM: list every crate with an `ilp64` feature, say whether it links the ILP64 library, what integer
width its declarations use, and give a measured test per crate. Measured on Windows with the AOCL for Windows install
(release notes 5.3.0), ILP64 DLL directories first on `PATH`, `cargo test --features ilp64`, with the build scripts as
they are on branch `lane/real-aocl-wrappers`. A pass is evidence for the paths the tests cover, not a proof.

Crates that have the feature: `git grep '^ilp64'` finds `aocl-{blas,lapack,sparse,rng,data-analytics,scalapack}-sys`, the
six wrappers that forward to them, and the `aocl-sys` and `aocl` umbrellas. Control: `aocl-scalapack` (the crate where the
problem was first found) is in that list. `aocl-fft`, `-math`, `-securerng`, `-utils`, `-compression`, `-crypto` have none.

| crate (sys + wrapper) | links (ILP64 dir) | integer width of the declarations | result under `--features ilp64` | action |
|---|---|---|---|---|
| `aocl-blas` | `amd-blis\lib\ILP64` (`AOCL-LibBlis-Win-MT-dll`) | i64 (`dim_t`, `inc_t`, `f77_int`; build passes `BLIS_ENABLE_ILP64`); wrapper has 0 pointer casts and 0 `i32` in its API | 53/53 pass (LP64 control: 53/53) | none |
| `aocl-lapack` | `amd-libflame\lib\ILP64` | `lapack_int` = int64 (`LAPACK_ILP64` passed) | **wrapper does not compile**: 428 E0308 errors (128 `i32` sites in its API) | `compile_error!` in `aocl-lapack` |
| `aocl-sparse` | `amd-sparse\lib\ILP64\shared` | before: **i32** (the header selects 64-bit with `aoclsparse_ILP64`, which was not passed); now i64 | before: **STATUS_ACCESS_VIOLATION**; now 21/21 pass (LP64 control: 21/21) | per-component `ilp64_defines` in `aocl-build` |
| `aocl-rng` | `amd-rng\lib\ILP64` (`rng_amd`) | before: **c_int** (the header needs `INTEGER64`, and `WIN64` on Windows); now c_longlong | before: **tests fail** (e.g. `reproducible_with_same_seed`, `gamma_positive_and_mean`); now 11/11 pass (LP64 control: 11/11) | same |
| `aocl-data-analytics` | `amd-da\lib\ILP64\shared` | before: i32 (the header needs `AOCLDA_ILP64`; before-run not measured); now i64 | with the macro it compiles and then **STATUS_ACCESS_VIOLATION**: 11 pointer casts pass `&[i32]` label, index and class slices to the library as 64-bit `da_int` | macro added; `compile_error!` in `aocl-data-analytics` |
| `aocl-scalapack` | `amd-scalapack\lib\ILP64\shared` | hand-written **i32** | `numroc` returns garbage, `pdgesv_mpi` indexes out of bounds (LP64 control passes) | `compile_error!` in `aocl-scalapack-sys` |
| `aocl-sys`, `aocl` | forward `ilp64` to the crates above | per crate | inherit the rows above | none |

`aocl_build::Component` gains a public field `ilp64_defines` (macros the component's own headers need to select 64-bit
integers); it is a breaking change for any build script that builds the struct literally (all 12 in this repo are updated).

## What the published 0.1.0 packages carry (read from the downloaded packages)

`aocl`, `aocl-sys`, `aocl-blas(-sys)`, `aocl-lapack(-sys)`, `aocl-rng(-sys)`, `aocl-sparse(-sys)`,
`aocl-data-analytics(-sys)` and `aocl-scalapack(-sys)` 0.1.0 each have an `ilp64` feature; `aocl-fft`, `aocl-fft-sys`,
`aocl-math`, `aocl-math-sys` and `aocl-build` do not. Published `aocl-build` 0.1.0 passes only
`-DAOCL_ILP64 -DBLIS_ENABLE_ILP64 -DLAPACK_ILP64` (lines 201-204 of its `src/lib.rs`), and published `aocl-scalapack-sys`
0.1.0 declares `c_int`. So as shipped, the sparse, rng, data-analytics and scalapack `ilp64` features are the unsound
combination measured above (32-bit declarations against the ILP64 DLL); the `aocl-lapack` wrapper's `ilp64` does not
compile; `aocl-blas`'s works for the tested paths. This is inferred from identical code paths, not re-run on the
published packages. Whether it warrants a yank is the owner's call (board 494).

— lane: aocl
