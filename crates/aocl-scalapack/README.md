# aocl-scalapack

Safe Rust wrappers for AOCL-ScaLAPACK — distributed linear algebra over MPI.

**Wrappers written, distributed routines untested.** ScaLAPACK ships only Fortran-style symbols (no public C headers), so `aocl-scalapack-sys` hand-declares them. This crate has a `Grid` (BLACS) wrapper, a `BlockCyclic` descriptor, `numroc`, and wrappers for the `p?gesv`, `p?gemm`, `p?potrf`, `p?getrf`/`getrs`, `p?potrs`, `p?syev`, `p?gels` and `p?geqrf` families. Only `numroc`, which needs no MPI initialisation, has a test. Nothing that builds a grid or calls a distributed routine has ever been run in CI, because that needs a multi-rank MPI host. On Windows, loading AOCL 5.3.2 `scalapack.dll` also needs Intel MPI (`impi.dll`), the Intel Fortran runtime, and AOCL BLIS, libFLAME and utils DLLs on `PATH`.

Built on top of [`aocl-scalapack-sys`](../aocl-scalapack-sys/).

Dual-licensed under MIT or Apache-2.0.
