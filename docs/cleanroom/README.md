# Clean-room declarations for aocl-math-sys / aocl-rng-sys (STAGING, not wired into any crate)

**Plain statement: this lane (aocl) has read the AOCL headers. The declarations in `math_decls.rs` and `rng_decls.rs` were
written by separate Sonnet sessions that were given ONLY extracted text of AMD's published documentation, never the
headers, never our earlier bindings, never the 0.1.0 crates, and no prior knowledge. This lane did not edit a declaration
by hand.** (Reviewer note: the lane read the headers earlier, and measured the name lists below against our current bindings
AFTER the writers had finished.)

## Source (the only input)
AMD "AOCL API Guide" 6.0, document 68552 (dated 2026-11-10), saved by CireSnave and converted by the PM with
`pdftotext -layout` to `C:/aocl-docs/guide-6.0.txt` (outside any repo; the text is not committed here).
Read 2026-11-27. AOCL version stated: 6.0.

| Library | Input | Original line range |
|---|---|---|
| aocl-math-sys | guide chapter 8 AOCL-LibM | `guide-6.0.txt` L106064-L111896 |
| aocl-rng-sys | guide chapter 9 (AOCL-RNG C API reference; SecureRNG ignored) | L111897-L115986, plus AOCL-RNG 4.2 `rng.rst`, `rngapi.rst` (cited by file:line; they hold no C prototype) |

## Provenance table
One row per declaration: `math_provenance.csv` (383 rows: 382 DECLARED, 1 UNKNOWN) and `rng_provenance.csv` (80 rows, all
DECLARED). Columns: name, source line in the guide text, guide section, status, note (watermark-reconstructed rows flagged).

**Taken:** function name, parameter names and C types as the guide states them. **NOT taken:** any header text, comments,
macro, typedef or struct body, constant, documentation prose, example code, or any declaration the guide does not state
as a C prototype. Anything the guide does not state is UNKNOWN and left out.

## Open (UNKNOWN: the guide does not say; needs a decision, not a guess)
- `rng_int_t` width and signedness is never defined in the guide (nothing about LP64/ILP64 either); every RNG declaration uses it.
- `RNG_SRANDINITIALIZEUSER_UINI/UGEN` are used but not defined (the DRAND versions are, at L112219-L112222).
- `fc32_t`/`fc64_t` (6 LibM complex functions) and `long double` (`amd_nexttoward`, `amd_nexttowardf`): ABI not derivable; the file uses opaque placeholders that are NOT ABI-correct. Do not publish these.
- `amd_vrd2_sincos` (L108016): watermark garbling too uncertain to confirm; left out.
- 56 LibM and 16 RNG prototypes had page-header watermark text interleaved; each is reconstructed only where the garbled text is exactly the reconstruction interleaved with the watermark (flagged in the CSV).
- 20 `amd_fast*` appear twice in the guide (declared once, at first line); `amd_vrsa_cosf` intro example (L106147) differs from the formal prototype (L109818): the formal one is declared.
- RNG: only the lowercase no-underscore C forms are declared; the guide also gives UPPERCASE and trailing-underscore Fortran-style forms (our current bindings expose all three). Every guide parameter pointer is `*mut` (the guide never uses `const`).

## Differences against the current main bindings, by name only (reported, no guess)
- LibM: `math-only-in-guide.txt` (70 names: 20 `amd_fast*`, `amd_vrd8_*`, `amd_vrs16_*`); `math-only-in-current-bindings.txt` (1 name: `amd_vrd2_sincos`, the one the writer left UNKNOWN). The PM's brief counted 17 only-ours; this lane's name comparison against `crates/aocl-math-sys/bindings/bindings.rs` (313 names) finds 1. The method of the 17 is not known to this lane; `RECONCILE.md` explains the gap (383 - 366 = 17) and lists the names.
- RNG: the 80 lowercase C names match our current lowercase names 80/80 both ways (0 only-ours, 0 only-guide); signatures were NOT compared.

## Status
Math: `aocl-math-sys` and `aocl-math` are wired from the 218 scalar rows (`MATH-WIRING.md`, with the breaking list). RNG stays UNWIRED and UNPUBLISHED (pm-kernels ruling 2026-11-27: `rng_int_t` width, `fc32_t`/`fc64_t`/`long double` and the two `UINI`/`UGEN` names are not in the guide; a later behavioural probe by a session that has not read headers may settle `rng_int_t`, recorded as a measurement). Nothing is published; 0.1.0 stays yanked.

## Added on lane/cleanroom-rng2 (2026-11-27): provenance for the correction rows
| Rows | What | Input | Provenance table | Measurement |
|---|---|---|---|---|
| 78 UPPERCASE `S/DRAND*` (77 declared + `SRANDINITIALIZEUSER` via the types probe; `GET_RNGVERSION` stated by the guide, not exported, not declared) | fresh Sonnet writer, guide chapter 9 text only | `guide-ch9` text | `rng_upper_provenance.csv` (79 rows), `rng_upper_decls.rs` | link-all + known answer: `crates/aocl-rng-sys/tests/upper_kat.rs` |
| 15 SecureRNG (crate `aocl-securerng-sys`; was a feature of aocl-rng-sys, never published; the crate previously held bindgen output of `secrng.h`: 15 fns + 5 consts) | Sonnet, guide SecureRNG text + DLL exports only | `secrng-guide.txt` | `secrng/secrng_provenance.csv` | `secrng/SECRNG-MEASUREMENT.md` (vacuous controls listed) |
| 8 LibM types + S/D user callbacks | Sonnet, guide excerpts + DLLs only, a different session from the writers | `types/` | `types/TYPES-MEASUREMENT.md`, `types/types_decls.rs` | `types/` (vacuous controls listed) |
| 156 SIMD (154 + 2) | mechanical from `math_decls.rs` | n/a | `math_provenance.csv` | `crates/aocl-math-sys/tests/simd.rs` (nightly) |
Link-all (every declared symbol resolves in the installed AOCL): `tools/link-all.ps1`; the 20 + 2 guide-stated functions 6.0.68 does not export: `math-guide-stated-not-linkable.txt`.
**This lane (aocl) read the AOCL headers earlier; every declaration above was written by a session that was given only the guide text and/or the binaries, and this lane did not edit a declaration by hand.**

## Raw evidence (one archive, not 537 files)
Everything bulky is ONE gzip archive: `docs/cleanroom/evidence/cleanroom-evidence-20261127.tar.gz` (185,569 bytes, 537 files, paths relative to
`docs/cleanroom/`), sha256 `44c52d9dc66729e91a7af11b46d54da8c4447f403e469a6e26f63228e9d5c687`; a second copy is on the lane host at `C:ocl-docs\evidence\`. It holds the raw probe outputs
(`raw-*.txt`, `types/userinit/run1-fill-a5-default/`, `secrng/run-all.log`, the 212-row `types/userinit/SUMMARY-user-runs.txt`), every `dumpbin-*.txt`, `secrng/dll-compare.txt` (12,130 lines of
`cmp -l`), the probe SOURCES (`secrng/probe/`, `types/probe-src/`, `types/userinit/src/` and `harness/`) and the RNG writer's staging file
`rng_upper_decls.rs` (the crate's `decls_upper.rs` is generated from it). Unpack: `cd docs/cleanroom && tar -xzf evidence/cleanroom-evidence-20261127.tar.gz`.
The measurement documents (`secrng/SECRNG-MEASUREMENT.md`, `types/TYPES-MEASUREMENT.md`, `types/userinit/USERINIT-MEASUREMENT.md`), the
per-run findings (tables inside the three documents), the provenance CSVs and `probe/` stay in git. To regenerate the raw files, run the
probes against AOCL 6.0.68 (Windows 11, x86_64) as each measurement document describes.
