# aocl-openrng wrapper: what the PM runs (the lane never runs the OpenRNG DLLs)

Script: `C:/openrng-measure/run_wrapper_tests.py` (same contract as `run_crate_tests.py`: hash check of the build, no network, nothing written outside `C:/openrng-measure/{target-wrapper-*,out}`).

    set OPENRNG_LIB_DIR=C:\openrng-build\src\build\rel-llvm-lp64\install\lib   & python -I C:\openrng-measure\run_wrapper_tests.py
    set OPENRNG_LIB_DIR=C:\openrng-build\src\build\rel-llvm-ilp64\install\lib  & python -I C:\openrng-measure\run_wrapper_tests.py

Output: `out/WRAPPER-TESTS-<width>-<stamp>.txt`. Read these lines:

| Prefix | From | Meaning |
|---|---|---|
| test result of `wrapper`, `domain` | tests/wrapper.rs, tests/domain.rs | `wrapper` ASSERTS (Philox seed 0 = Random123 vector, clone/save/restore equality, support of every distribution, shape refusals). A failure is a finding about the wrapper or the library: do not fix the test to pass |
| `RECORD ...` | wrapper.rs | version, empty-output behaviour, `new`/`save` per generator, leapfrog and skip-ahead on Philox, multivariate Gaussian per storage |
| `DOMAIN ... -> ...` | domain.rs | what each distribution returns for an out-of-domain parameter (BADARGS, OK, NaN output...): decides whether a Rust-side check is needed |
| `GUARD from_params(...)`, `GUARD skip_ahead_ex(...)` and the exit code of each `--exact` run | tests/params_guard.rs | a process exit of -1073741819 (0xc0000005) = the library read past `n` parameter words: that initialiser stays `unsafe` for that generator and `n`; a normal return = no read past n observed for that n only |
| exit code of each `out_guard` test | tests/out_guard.rs | every generating method given a slice that ENDS at a PAGE_NOACCESS page (n = 1, 3, 7, 33), the multinomial / Gaussian-MV / Poisson-V families at their exact sizes, and the input arrays (mean, factor, p, lambda) at a guard page: a normal return = no write or read past the stated length; -1073741819 = the library reads or writes outside it, report which test |
| `threads` test result | tests/threads.rs | 8 separate streams on 8 threads equal the sequential results (the wrapper holds ONE lock around every library call; this does not measure the library alone) |

Not covered (UNMEASURED, so not claimed): thread-safety of a `Stream` (`!Send`/`!Sync` stays), `Wh`/`Mt2203` families (`Stream::with_brng_id`, `unsafe`), NONDETERM, abstract streams, `vslRegisterBrng`, `vslGetBrngProperties`, Linux, bit-reproducibility against oneMKL.
