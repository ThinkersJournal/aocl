# aocl-scalapack

Safe Rust wrappers for AOCL-ScaLAPACK — distributed linear algebra over MPI.

**Wrappers written; `Grid`, `BlockCyclic`, `pdgesv`, `pdgemm`, `pdgetrf`/`pdgetrs` and `pdpotrf`/`pdpotrs` tested on 1-3 MPI ranks, the other distributed routines untested.** ScaLAPACK ships only Fortran-style symbols (no public C headers), so `aocl-scalapack-sys` hand-declares them. This crate has a `Grid` (BLACS) wrapper, a `BlockCyclic` descriptor, `numroc`, and wrappers for the `gesv`, `gemm`, `potrf`, `getrf`, `getrs`, `potrs`, `syev`, `gels` and `geqrf` routines in real single (`ps*`) and double (`pd*`) precision only; the only complex wrappers are `pcgesv` and `pzgesv`. Tests: `numroc` (no MPI needed) and `tests/pdgesv_mpi.rs`, which builds a grid and solves an asymmetric 6x6 system with `pdgesv` and checks `pdgemm` (`N` and `T`) against a serial product, solves with `pdgetrf`+`pdgetrs` (`N`, `T`) and `pdpotrf`+`pdpotrs` (both triangles), on one rank under `cargo test` and on 1, 2 and 3 ranks under `mpiexec`; it was run by hand on Windows with AOCL 5.3.2 and Intel MPI 2021.18, and CI runs it only on the self-hosted runner (`.github/workflows/selfhosted.yml`, dispatch-only, never yet run there). No other distributed routine has been run. On Windows, loading AOCL 5.3.2 `scalapack.dll` also needs Intel MPI (`impi.dll`), the Intel Fortran runtime, and AOCL BLIS, libFLAME and utils DLLs on `PATH`.

Built on top of [`aocl-scalapack-sys`](../aocl-scalapack-sys/).

Dual-licensed under MIT or Apache-2.0.
