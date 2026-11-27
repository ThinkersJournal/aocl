# aocl-lapack-sys

Raw FFI bindings to AOCL-LAPACK (libFLAME). For the safe API see [`aocl-lapack`](../aocl-lapack/).

**Measured gates (AOCL 6.0.68 Windows, LP64 and ILP64 identical; Linux unmeasured).** (1) 150 functions the headers declare (100 `LAPACKE_*` extended-precision routines `*x`/`*xx` with their `_work` forms, and 50 Fortran `*x_`/`clagge_`-type routines) are exported by NO installed library, dynamic or static: they are left out unless the feature `header-only-unexported` (default OFF) is enabled, which only declares them; it does not make them link on Windows. (2) `lsame_` is exported by the BLIS library, not by libflame: it is declared only with the feature `blis` (default OFF), which links `AOCL-LibBlis-Win-MT-dll` (or the static `AOCL-LibBlis-Win-MT` with `static-link`) from `amd-blis/lib/{LP64,ILP64}`. Measured: default 3697 of 3697 declared functions resolve; with `blis` 3698 of 3698 (previously 151 of 3848 declared names did not link). `tests/link_all.rs` checks every declared function per feature set.

**`static-link` needs more on the link line (Windows, AOCL 6.0.68, measured; this crate does not link it).** The static libflame references BLIS (`bli_*` and BLAS routines) and the OpenMP runtime (`__kmpc_*`): add the static `AOCL-LibBlis-Win-MT.lib` and `libiomp5md.lib` (Intel oneAPI `compiler/<version>/lib`, on the `LIB` path).

Dual-licensed under MIT or Apache-2.0.
