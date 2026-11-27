# aocl-fft-sys

Raw FFI bindings to AOCL-FFTW. For the safe API see [`aocl-fft`](../aocl-fft/).

## Features (default OFF; each adds a link to another DLL of the installed AOCL-FFTW)
Measured with a link-all test on AOCL 6.0.68 for Windows (`tests/link_all.rs` takes the address of every declared function for the enabled feature set); **Linux is unmeasured**.
Without a feature, 84 of the 216 declared functions do NOT link (they live in DLLs this crate did not link: that was a link error when used):

| feature | links | makes these declared names link |
|---|---|---|
| `long-double` | `libfftw3l` | the 66 `fftwl_*` long-double functions |
| `threads-omp` | `libfftw3_omp`, `libfftw3f_omp` (+ `libfftw3l_omp` with `long-double`) | the 12 `fftw_`/`fftwf_` threads functions (`*_init_threads`, `*_plan_with_nthreads`, `*_planner_nthreads`, `*_cleanup_threads`, `*_make_planner_thread_safe`, `*_threads_set_callback`); with `long-double` also the 6 `fftwl_` ones |
| `mpi` | `libfftw3_mpi`, `libfftw3f_mpi` (+ `libfftw3l_mpi`) | no further declared name (the threads functions are also exported there); needs an MPI runtime when you use it |

Measured counts (declared 216): default 132 resolve; `long-double` 198; `threads-omp` 144; `long-double,threads-omp` 216; `+mpi` 216. **Run-time requirement:** `libfftw3_omp.dll` imports `libomp.dll` (the LLVM OpenMP runtime), which AOCL does not ship (here it came from the Visual Studio LLVM `bin`): it must be on `PATH` when `threads-omp` is used.

Dual-licensed under MIT or Apache-2.0.

**Licence condition of the library you link.** The AMD `libfftw3`/`libfftw3f` in AOCL is GPL-licensed (AOCL's `NOTICE.txt` says GPL-2.0; the shipped `COPYRIGHT` says version 2 or any later version: the two disagree). This crate ships no library. A program that links it and is distributed must comply with the library's licence. Whether a source-only crate of `extern` declarations is itself a combined work is UNKNOWN (no FSF text found). This is not legal advice. See `THIRD-PARTY-NOTICES.md` and `docs/PROVENANCE-QUESTIONS.md` question 2.
