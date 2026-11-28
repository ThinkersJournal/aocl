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

Not covered (UNMEASURED, so not claimed): thread-safety of a `Stream` (`!Send`/`!Sync` stays), `Wh`/`Mt2203` families (`Stream::with_brng_id`, `unsafe`), NONDETERM, abstract streams, `vslRegisterBrng`, `vslGetBrngProperties`, Linux, bit-reproducibility against oneMKL.
