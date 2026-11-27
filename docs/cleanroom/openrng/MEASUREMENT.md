# AOCL-OpenRNG 6.0.0 measured (Windows x64; run by the PM session on CireSnave's "524: A", 2026-11-27)

**What this is.** A black-box measurement of the two Windows builds of `amd/openrng` (tag 6.0.0, commit `5c05f2eeb6effc696b0df6ce8be1fcbe855b631b`;
`git archive` sha256 `46336bbf6aac568875a235a36b672db073eef2cc02b7954962fdfc92d36c23c7`) built by the PM (VS18 clang-cl x64, CMake 4.5.1, presets
`rel-llvm-lp64` and `rel-llvm-ilp64`, `BUILD_TESTING=OFF BUILD_BENCH=OFF`, AOCL_ROOT = the AOCL 6.0.68 `amd-libm`). The measurement was **run by the
PM session on CireSnave's 524: A** (this lane's session was denied running the PM-built DLLs); this lane wrote the harness
(`docs/cleanroom/openrng/measurement-harness/`) and the PM read it first. **Black box:** OpenRNG's `src/` was NOT read; only the licensed public
header `include/openrng.h` (blob sha256 `ee3a48fcb50a9a3ded3300bfd342b8a7c7bf8b534f38233be93b26ced577386b`, route A) and behaviour. **OpenRNG's own tests were NOT run.**

## Commands (one per build; cmd.exe; the driver hashes the build dir against `C:\openrng-build\build-sha256.txt` before running anything)
```
set OPENRNG_LIB_DIR=C:\openrng-build\src\build\rel-llvm-lp64\install\lib  & python -I C:\openrng-measure\openrng_measure.py
set OPENRNG_LIB_DIR=C:\openrng-build\src\build\rel-llvm-ilp64\install\lib & python -I C:\openrng-measure\openrng_measure.py
```
Inside, per build: `cargo test --offline --no-run [--features ilp64]` (compile and link against `openrng.lib`), a negative-control compile, then
`cargo test --offline --test link_all --test width_probe --test kat_record --test loaded_modules [--features ilp64] -- --nocapture --test-threads=1`.

## Versions and host
Windows 11 10.0.26300 (AMD64); `cargo 1.100.0 (3acc79c0c 2026-11-05)`, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, host `x86_64-pc-windows-msvc`; Python 3.15.2.
Loaded modules (GetModuleFileNameA, hashed and compared by the driver): `openrng.dll` from the chosen build dir, `libalm.dll` from
`C:\Program Files\AMD\AOCL-Windows\amd-libm\lib` (AOCL 6.0.68; sha256 `005377e627dbfe32eecc75287cc2bce4de1a1318142c24c63ee4444e27f378bc`, 1732504 bytes) and the CRT.

| file | lp64 sha256 | ilp64 sha256 |
|---|---|---|
| openrng.dll | `1a4034467ca6399e9449f81cf31e8f3c712594e3f3973516f8f4e7da94340ba1` (1276928 B) | `1d3fd3a49fad0cf4b8ac62fd7583d5f77f5a9688d23ec30681bfc4fd3726b771` (1276416 B) |
| openrng.lib | `1d534f3e2b7dd87032a0cbad9e0d17ec8ad82189cbbc8d5a50f6355c6a84ab56` | same |
| openrng_static.lib | `edecdd6dd1539a38972aef58f230f6c3b0eca2b21f2469d922eb4834d1d4a9dc` | `acd07fe6c9eb37207058f039ea992ce0c8bfa709eba7b613a6d4ef07a82ebfd9` |

All hashes equal the PM's `build-sha256.txt` ("OK" in each raw file). `openrng_static.lib` was hashed, not linked or run.

## Results
| check | LP64 build | ILP64 build |
|---|---|---|
| link-all: address of all 60 declared functions taken (our `-sys` declarations from the header) | 60 / 60 resolve | 60 / 60 resolve |
| negative control: a made-up symbol must NOT link | fails: `LNK2019 unresolved external symbol openrng_measure_control_does_not_exist` | same |
| `openrng_int_t` width, by behaviour (below) | **4 bytes** | **8 bytes** |
| PHILOX4X32X10 seed 42, `viRngUniformBits32` x8 | `9ceaf053 77f5493b 12bf50ad 5742b3d7 fcdb2127 53ba6cfd 838f5a6e 744e06fb` | identical |
| PHILOX4X32X10 seed 42, `vdRngUniform[0,1)` x4 (bit patterns) | `3fbceaf053000000 3feefea927600000 3fe257ea15a00000 3feae8567ae00000` (0.11295988..., 0.96858651..., 0.57323173..., 0.84086154...) | identical |
| PHILOX4X32X10 seed 0, `viRngUniformBits32` x8 | `6627e8d5 e169c58d bc57ac4c 9b00dbd8 f8e4cca4 5cb200db b1a574eb 097eff67` | identical |
| each stream repeated from the same seed | identical | identical |

**Width method.** A probe with its OWN `extern` block passes every `openrng_int_t` argument as 64-bit. On Win64 a 32-bit parameter reads only the low half of the
argument register; a 64-bit one reads all of it. `vdRngUniform(method, stream, n, ...)`: `n = 4` -> status 0, 4 doubles (both builds); `n = -4` -> status -2
(refused), 0 doubles (both); `n = 0xFFFFFFFF_00000004` (low half 4, high half all ones) -> **LP64: status 0, 4 doubles written (only the low half read)**;
**ILP64: status -2, 0 doubles (the high half read: negative)**. A positive huge `n` was deliberately not passed.

**Known answer.** Seed 0's first four words, `6627e8d5 e169c58d bc57ac4c 9b00dbd8`, **equal the published Random123 Philox4x32-10 vector for counter 0, key 0**: the file
`tests/kat_vectors` of `github.com/DEShawResearch/random123` (read through the GitHub API at its latest commit `d8a0c25e49992685c0b9d91b553c1d04f81b6b4b`, 2021-01-17) has the line
`philox4x32 10 00000000 00000000 00000000 00000000 00000000 00000000   6627e8d5 e169c58d bc57ac4c 9b00dbd8`. That the generator's seed 0 corresponds to counter 0 / key 0 is
**an observed equality, not a documented mapping**; the seed-42 words have no published reference (no reference: reproducibility across the two builds only). Bit-for-bit reproducibility
against oneMKL is **not measured** (no oneMKL output and no cited source here).

## Runs and why there are two pairs
`measurement-raw/RAW-lp64-20261127-151750.txt` + `RAW-ilp64-20261127-151756.txt` is the first pair: every number above, but the driver exited 2 because its final
loaded-module check parsed the `MODULE` line with an anchored regex while the test harness prints it after `test record_loaded_modules ... ` on the same line (a driver bug, measurement
unchanged). The driver was fixed (unanchored pattern, line 144) and re-run: `RAW-lp64-20261127-152006.txt` + `RAW-ilp64-20261127-152011.txt` (exit 0, hashes OK before running, tests exit 0,
negative control fails to link naming the symbol, each `openrng.dll` loaded from its own build dir with the recorded hash) are the RECORD; the numbers are identical.

## NOT measured
Linux; any other compiler or CPU; the `openrng_static.lib` link; OpenRNG's own tests; every distribution beyond Philox uniform and the bit generators beyond PHILOX4X32X10; thread safety;
bit-reproducibility vs oneMKL; the Random123 equality for any seed other than 0; the library's behaviour on a width mismatch beyond the one probe above.

## Added after the measurement (compiled against both builds, NOT run by this lane)
`crates/aocl-openrng-sys/tests/width_match.rs` (a wide `n` is refused only by an ILP64 library when the `ilp64` feature is on: a width mismatch fails it), the known-answer
asserts in `tests/stream.rs` (seed 0 = the Random123 vector above; seed 42 = the recorded words) and the build-time width warning in `build.rs` were written from the recorded
numbers and compiled against both builds (the warning was seen for an ILP64 dir with the feature off). Running the new tests needs the PM-built DLLs, which this lane may not run; they
are for the PM's session or CI to run: `set OPENRNG_LIB_DIR=...\rel-llvm-<lp64|ilp64>\install\lib` then
`cargo test -p aocl-openrng-sys [--features ilp64]` (PATH with the build's `lib` dir and `amd-libm\lib`).
