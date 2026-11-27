# AOCL-RNG integer width (`rng_int_t`): a behavioural MEASUREMENT, not a guide declaration

The guide (6.0, 68552) never states the width of `rng_int_t` (see `rng_notes.md`). This is a measurement of the installed
binaries, made by a sub-agent that was given only the guide chapter 9.1.3+ text, the DLL paths and `dumpbin` output (the
binaries' own export table and import list). It read no header. Probe source and raw output: `probe/`.

**Measured on:** AOCL-RNG `get_rngversion()` = `"AOCL-RNG 6.0.0 Build 20261027"` (AOCL install 6.0.68), Windows 11 Pro 10.0.26300,
x86_64 MS x64 ABI, rustc 1.100.0 (`rustc --edition 2021 -O probe.rs`). Files: `amd-rng\lib\LP64\rng_amd.dll` and
`...\ILP64\rng_amd.dll` (same version string). **Linux and the self-hosted rig were NOT measured. `amdsecrng.dll` was NOT probed.**

**Commands:** `pwsh probe/run.ps1 <LP64|ILP64> <argwidth-bytes> <width|kat1|kat2|kat3>` (positives: LP64 4, ILP64 8;
negative controls: LP64 8, ILP64 4; kat1/kat3 re-run for determinism). PATH is set per child process; `KMP_DUPLICATE_LIB_OK` is not set.
Calls: `drandinitialize` (genid, subid by value; seed, lseed, state, lstate, info by pointer) and `dranduniform(n, a, b, state, x, info)`.
Every integer slot is pre-filled with 0xAA; bytes of `info` that changed are counted. Raw output: `probe/raw_output.txt`.

| build | `rng_int_t` | positive evidence | negative control (wrong width) |
|---|---|---|---|
| LP64 `rng_amd.dll` | **4 bytes** | `info` write changed exactly 4 of 16 bytes in every call; output bit-identical to ILP64 for kat1/kat2/kat3 | int64 args: kat2, kat3 misbehave (info -3 / -4, error text); **kat1 output is NOT affected** (little-endian; only the 4-byte `info` write shows it) |
| ILP64 `rng_amd.dll` | **8 bytes** | `info` write changed exactly 8 of 16 bytes in every call | int32 args: all four cases misbehave (8 bytes written into a 4-byte slot, init info 1, uniform info -4, output left at fill) |

Same seed gives the same doubles in both builds (kat1 first value `0x3fef20e1d46d4343` = 0.9727639340363584). State lengths 16 / 20 / 633
for genid 1 / 2 / 3 in both. No crashes.

**OpenMP:** `rng_amd.dll` imports `libalm.dll`, `AOCL-LibBlis-Win-dll.dll`, `libmmd.dll`, `svml_dispmd.dll` and the CRT (`probe/dumpbin-dependents-LP64.txt`):
**no OpenMP runtime** (libomp140 / libiomp5md). None was loaded in any of 20 runs for these two routines; nothing is claimed for other routines.

## Forms: guide vs exports (names only)

- Guide lists UPPERCASE (78 `S/D RAND*` + `GET_RNGVERSION`), trailing-underscore (78 + `get_rngversion_`) and lowercase no-underscore forms.
- **Exported by both DLLs:** the 78 UPPERCASE `RAND*` names and the lowercase forms (`get_rngversion`, `drandinitialize_secrng` included).
  `GET_RNGVERSION` is not exported. **No trailing-underscore form is exported** (0 matches; the control `drandinitialize`/`get_rngversion` are found).
- **OLD bindings (names only, from main 98dbce0, 238 fns):** 78 UPPERCASE `RAND*`, 78 lowercase-underscore `*rand*_`, 78 lowercase `*rand*`, plus `drandinitialize_secrng`, `get_rngversion`, `get_rngversion_`, `GET_RNGVERSION` (234 + 4).
  So the old crate declared **80 symbols the 6.0.68 DLLs do not export** (78 underscore forms + `get_rngversion_` + `GET_RNGVERSION`): any use is a link error. `aocl-rng` used only lowercase forms.
- **Declared now:** only the 79 lowercase no-underscore functions the guide states (`rng_decls.rs`, minus `srandinitializeuser`, whose callback typedefs the guide does not state).
  **Recommendation (a list for the PM):** do NOT declare the UPPERCASE forms in this release (all-by-pointer, exported, guide-stated, but the safe wrapper uses lowercase; add on demand); do NOT declare the underscore forms (not exported).

## Rust side

`rng_int_t` = `c_int` by default, `i64` under the existing `ilp64` feature (the same scheme as `aocl-blas-sys`, which has only `ilp64`; there is no `lp64` feature). `tests/width_kat.rs` (links the real DLLs; known answer above)
passes for both builds: `pwsh docs/cleanroom/probe/cargo-test.ps1 LP64` and `... ILP64`.
