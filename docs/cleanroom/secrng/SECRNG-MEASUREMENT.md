# AOCL-SecureRNG (`amdsecrng.dll`): clean-room declarations and behavioural measurement

**Inputs used (only these):** (a) `secrng-guide.txt` (AMD AOCL API Guide 6.0, doc 68552, SecureRNG text); (b) the binaries
`amd-rng\lib\LP64\amdsecrng.dll`, `amd-rng\lib\ILP64\amdsecrng.dll`, `amd-securerng\lib\LP64\amdsecrng.dll` and their `dumpbin /exports`, `/dependents`, `/headers` output; (c) the method of
the earlier RNG probe (`probe.rs`/`run.ps1` shape, `RNG-WIDTH-MEASUREMENT.md`). **No header, no source, no existing Rust binding or crate was read; no prior knowledge of AMD headers was used.**
No disassembly. Only `lib\` directories of the install were listed.

**Machine:** AMD Ryzen 9 7940HX, Windows 11 Pro 10.0.26300, x86_64 MS x64 ABI, `rustc 1.100.0 (0ef1bc386 2026-11-09)`, `rustc --edition 2021 -O`.
**Library:** `get_secrngversion()` = `"AOCL-SecureRNG 6.0.0 Build 20261027"` in all three DLLs (AOCL install 6.0.68). Linux and CI runners were NOT measured.

## Declarations and names
- `secrng_decls.rs`: **15** guide-stated C prototypes, declared (compiles as a lib); two of them (`get_rdrand64u_arr` L141, `get_rdseed64u_arr` L271) are page-header-interleaved in the text and are *reconstructed* (extra letters spell "AOCL API Guide (68552)").
  Types literal: `uint16_t/32/64 -> u16/u32/u64`, `int -> c_int`, `unsigned int -> c_uint`, `unsigned char -> c_uchar`, `const char -> c_char`.
- Exports (21, identical name table in LP64, ILP64, securerng-copy; `diff` of `dumpbin /exports` differs only in the "Dump of file" line).
- **Guide but not exported: none (0 of 15).**
- **Exported but not in guide (6, names only, no prototype guessed, none declared, none called):** `aoclsecrng_version`, `l_get_rdrand16u`, `l_get_rdrand32u`, `l_get_rdrand64u`, `trdrand_status`, `trdseed_status`.
  `aoclsecrng_version`, `trdrand_status`, `trdseed_status` have RVAs 0x4008/0x4000/0x4004 inside `.data` (data, not code); the first 16 raw bytes read at each are in `raw-*-info.txt` (`trdrand_status`/`trdseed_status` start `01 00 00 00`), meaning UNKNOWN.
- **UNKNOWN (guide never states):** numeric values of `SECRNG_SUCCESS`, `SECRNG_FAILURE`, `SECRNG_NOTSUPPORTED`. The guide gives only "1 - Success" for the two `is_*_supported` functions.
- Guide typos noted: "suuported" (L58, L188); `is_*_supported` documented as "1 - Success".

## Do LP64 and ILP64 differ?
- Different files (sha256 differ), same size 24984. `cmp -l` (`dll-compare.txt`): bytes inside the sections (file offsets 0x400-0x39FF: .text .rdata .data .pdata .rsrc .reloc) are **identical**; differences are only in 5 header bytes (offsets 129-130, 209-211, 1-based) and the trailing signature blob (from 14860). The `amd-securerng` LP64 copy differs from `amd-rng` LP64 only in header bytes 209-210.
- Exports, dependents, version string and every measurement below are identical across the three. **SecureRNG has no width-dependent ABI: no `rng_int_t`-like type; all integer parameters are `int`/`unsigned int` in both builds.**
  (A control showing ILP64 differs from LP64 is therefore impossible; "LP64 == ILP64" is evidence only of equality, not of a probe's power. VACUOUS as a control by construction.)

## OpenMP
`dumpbin /dependents` (all three): `KERNEL32.dll`, `VCRUNTIME140.dll`, `api-ms-win-crt-runtime-l1-1-0.dll`. **No OpenMP runtime** (no libomp/libiomp5md). Loaded-module list after the calls (15 runs, `modules_loaded=7`) has `openmp_like_modules=0`.

## Measurements (identical for LP64, ILP64, securerng-LP64 unless stated)
Method: each output slot pre-filled with sentinel 0xAA and (alternate trials) 0x55, 32 trials; a byte counts as written if it differed from its sentinel in any trial (a random byte equals a sentinel w.p. 1/256 per trial, so a written byte stays "unwritten" only w.p. 256^-32). Raw: `raw-<build>-<case>.txt`.

| function | measured | positive evidence | negative control | non-vacuous? |
|---|---|---|---|---|
| `get_secrngversion` | returns pointer to NUL-terminated string `AOCL-SecureRNG 6.0.0 Build 20261027` | read via `*const c_char` | none attempted | n/a |
| `is_RDRAND_supported`, `is_RDSEED_supported` | return **1** (CPU supports both) | matches the guide's "1" | no CPU lacking RDRAND available; the unsupported value is **never observed** | VACUOUS (cannot be produced here) |
| return code of the `get_*` functions | success = **2**, failure = **-2**; -3 on NULL output pointer; guide's `SECRNG_NOTSUPPORTED` never observed (HW supports) | 2 returned in ~all rdrand calls; RDSEED returns -2 when retries exhausted (below) | values -2 / -3 differ from 2 | non-vacuous for 2 vs -2 vs -3; the **name-to-number mapping of -2 and -3 is inferred, not guide-stated** (-3 might be NOTSUPPORTED-or-invalid-arg: unknown) |
| `get_rdrand16u/32u/64u`, `get_rdseed16u/32u/64u` | write exactly **2 / 4 / 8** bytes at offset 0 of a 16-byte slot, offsets 0..w-1 only | `bytes_written=2/4/8`, `max_offset_written=1/3/7` | pairwise: the 32u function leaves bytes 4..15 untouched (wrote 4 where a 64-bit slot would hold 8), the 16u one leaves 2..15 untouched | non-vacuous (2 != 4 != 8) |
| value-vs-pointer for `rng_val` | pointer | positive: ptr -> ret 2 | control: pass integer 0x1000 where pointer goes -> access violation 0xC0000005; swap args (retry first, ptr second) -> 0xC0000005 | non-vacuous |
| NULL `rng_val`/`rng_arr` (all 12 value/array `get_*` functions; NULL with N=0 too for 2) | returns **-3**, slot untouched, no crash (also N=0 with NULL) | `crash_*_null` raw files, 14 NULL-case files x 3 builds | vs positive (ret 2, slot written) | non-vacuous |
| `get_rdrand32u_arr`, `get_rdseed32u_arr` (N elements) | write exactly **4*N** bytes, N counted in elements (N=1,3,13,16,17 -> 4,12,52,64,68) | `bytes_written` equals `4*N`, byte 4N never written | vs `64u_arr`: 8*N (N=3: 24 not 12); vs reading N as bytes (would be 3): 12 measured | non-vacuous |
| `get_rdrand64u_arr`, `get_rdseed64u_arr` | write exactly **8*N** bytes | N=1,3,13,16,17 -> 8,24,104,128,136 | vs 32u_arr (4*N) | non-vacuous |
| `get_rdrand_bytes_arr`, `get_rdseed_bytes_arr` (N bytes) | write exactly **N** bytes, not rounded up to 4/8/16 (N=1,3,7,9,13,17,31) | `bytes_written == N` | vs 32u_arr N=3 (12) | non-vacuous |
| **N = 0** (all array forms) | returns **-2** (failure), writes **0** bytes | `raw-*-arr.txt` | vs N=1 (ret 2, writes) | non-vacuous (the guide does not state N=0 behaviour) |
| `retry_count` (RDSEED) | number of extra attempts: failure rate falls with retry_count. 1 thread, `get_rdseed32u`, 50000 calls: retry 0 -> 28624 x(-2) (LP64), 1 -> 19705, 10 -> 4555, 1000 -> 0. 16 threads: 467470/800000, 281037, 5962, 0. Same in ILP64/securerng. On failure the single-value slot is **untouched**; the array form leaves a partial prefix written | `raw-*-retry.txt` | retry=0 vs retry=1000 differ (-2 rate 57% vs 0%) | **non-vacuous for RDSEED** |
| `retry_count` (RDRAND) | **no failure observed in 850,000 calls per setting (0/1/10/1000, 1 and 16 threads)**; retry semantics for RDRAND cannot be measured here | | | **VACUOUS** (all retry settings indistinguishable) |
| `retry_count` width (u32 vs u64) | NOT measured: a 64-bit value with garbage high half could mean a 4G-element write, so not attempted | | | no control; stays UNMEASURED (declared as `unsigned int` per guide) |
| `int` return width | NOT measured: values 2, -2, -3 all fit in 32 bits; a control reading rax as i64 would give the same 2 for the positive case | | | VACUOUS |
| statistical sanity (65536 bytes; chi2 dof 255, expect 255+/-22.6; ones fraction 0.5+/-0.0007) | all 6 bulk sources in the final runs: chi2 216-282 over the 18 final bulk rows across the three builds (e.g. LP64 `rdrand_bytes_arr` 225.9, `rdseed_bytes_arr` 216.4; see `raw-*-stat.txt`), 256 distinct values each, ones fraction 0.4983-0.5011 (within ~2.5 sigma, sigma 0.0007); 4096 single calls per width: all byte lanes 256/256 distinct; two successive 4096-byte calls: 13-21 equal bytes (expect ~16) | `raw-*-stat.txt` | controls: all-zero chi2 16.7M; `&0x7f` chi2 65.8k, ones 0.4375; N=0 untouched chi2 16.7M; half-written buffer chi2 4.18M, ones 0.25 | non-vacuous (the statistic rejects each) |

Notes and caveats
- A first `stat` run used retry_count 10 for RDSEED bulk calls and got ret -2 with a **partly filled buffer** (chi2 16.7M, 16 distinct bytes): when RDSEED retries are exhausted the array/bytes forms return -2 and leave the remainder as it was. The final runs use retry 1000 (all ret 2); this is itself the failure-path measurement.
- Under contention (the RDSEED arr tests ran with a busy machine) all 32 trials at `rdseed32u_arr N>=13, retry=10` failed in a few runs (bytes_written 32 or 48 of 52-68; `raw-*-arr.txt`, the only LP64/ILP64/securerng diff, random contention). With retry 1000 (retry case, above) the arr form never fails. So the exact-N byte counts for RDSEED rows come from the runs where trials succeeded; RDRAND rows never failed.
- Success value 2 (not 1) and the -2/-3 values are measured facts about these DLLs, not guide statements.
- `aoclsecrng_version` data export and `l_get_rdrand*` functions, `trdrand_status`, `trdseed_status`: not declared and not called (the first 16 bytes of the three data symbols were read only).

## Reproduce
```
rustc --edition 2021 -O probe/probe.rs -o probe/probe.exe
pwsh -NoProfile -File probe/run-all.ps1          # ~12 min; writes out/raw-<LP64|ILP64|SECLP64>-<case>.txt; PATH set per child only
pwsh -NoProfile -File probe/run.ps1 LP64 single   # one case; cases: info single arr retry stat crash_<fn>_<null|null0|ptrasval|swapped>
dumpbin /exports|/dependents  <dll>             # (via MSVC 14.52 Hostx64; MSYS2_ARG_CONV_EXCL='*' from Git Bash)
```
