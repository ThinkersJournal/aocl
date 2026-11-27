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
Staging only: nothing here is compiled into a crate, nothing is published, 0.1.0 stays yanked. Wiring the crates and the
linking tests against an installed AOCL wait for the PM's reading of this table and a ruling on `rng_int_t`.
