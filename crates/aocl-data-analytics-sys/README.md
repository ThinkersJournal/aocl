# aocl-data-analytics-sys

Raw FFI bindings to AOCL-DA. For the safe API see [`aocl-data-analytics`](../aocl-data-analytics/).

**`static-link` needs more on the link line (Windows, AOCL 6.0.68, measured; this crate does not link it).** A static link fails first on `LINK : fatal error LNK1104: cannot open file 'ifconsol.lib'` (the Intel oneAPI Fortran runtime: put `oneAPI/compiler/<version>/lib` on the `LIB` path), and then needs the OpenMP runtime (`libiomp5md.lib`) and the BLAS/LAPACK routines (`DGEMM`, `DGESV`, `DPOTRF`, ... : the static BLIS and libflame). All 285 declared functions resolve in the dynamic build.

Dual-licensed under MIT or Apache-2.0.
