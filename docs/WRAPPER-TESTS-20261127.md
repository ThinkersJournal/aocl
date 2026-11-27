# The safe wrappers' tests on the real installed AOCL (Windows 11, AOCL 6.0.68, LP64), 2026-11-27

`python tools/wrapper-tests.py` runs `cargo test -p aocl-<x>` for every safe wrapper with a PATH built per width (never inherited). The wrappers have no mock layer: each `-sys` crate links the installed library, so a test that passes here called into the DLL.

| wrapper | `#[test]` | passed | failed | runtime requirement found |
|---|---:|---:|---:|---|
| aocl-blas | 53 | 53 | 0 | |
| aocl-lapack | 20 | 20 | 0 | |
| aocl-fft | 11 | 11 | 0 | |
| aocl-utils | 2 | 2 | 0 | |
| aocl-compression | 13 | 13 | 0 | |
| aocl-sparse | 21 | 21 | 0 | |
| aocl-crypto | 19 | 19 | 0 | **`libcrypto-3-x64.dll` (OpenSSL 3) must be on `PATH`**: `alcp.dll` imports it and AOCL does not ship it; without it the test binary dies with `STATUS_DLL_NOT_FOUND` (here it came from Git's `mingw64\bin`: `--path="C:\Program Files\Git\mingw64\bin"`) |
| aocl-data-analytics | 24 | 24 | 0 | **`KMP_DUPLICATE_LIB_OK=TRUE`**: `aocl-da.dll` loads two OpenMP runtimes and aborts with "OMP: Error #15" in the first parallel kernel (amd/aocl-data-analytics issue 2); without it the run exits with code 3 after 11 passing tests (`--env=KMP_DUPLICATE_LIB_OK=TRUE`) |
| aocl-math | 11 | 11 | 0 | |
| aocl-rng | 11 | 11 | 0 | |
| aocl-securerng | 6 | 6 | 0 | |
| aocl-scalapack | 1 | 1 | 0 | |

All 12 wrappers have tests that call into the DLLs (192 tests, 0 failed, 0 ignored, with the two runtime requirements above). `aocl-scalapack` has ONE test, `aocl-utils` two: the thinnest coverage.

**Controls.** (1) `--control-no-aocl` (PATH without any AOCL directory): every wrapper's test run FAILS (all 12 "CONTROL OK"): the tests depend on the DLLs. (2) A deliberately wrong expected value fails: `aocl-math` (`ilogb(8.0)` expected 4) -> `exp_log_family` FAILED; `aocl-blas` (`scal` expected `[2.0, 4.0, 7.0]`) -> `scal_f64` FAILED; both reverted. Linux and ILP64 not run here (`--ilp64` is supported by the tool). Not on a CI runner: none is registered.
