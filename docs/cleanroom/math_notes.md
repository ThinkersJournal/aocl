# AOCL-LibM clean-room notes (source: input-libm.txt only; L numbers = original doc line numbers)

Counts: 383 distinct amd_ names; DECLARED 382; UNKNOWN 1 (amd_vrd2_sincos, L108016).

## Unresolved types (guide names them, never defines them)
- `fc32_t`, `fc64_t` (complex scalar, amd_cexpf/cexp/clogf/clog/cpowf/cpow, L111727-L111786): opaque placeholder structs; by-value ABI not derivable from the guide.
- `long double` (amd_nexttowardf L107461, amd_nexttoward L107473): no core::ffi type; aliased to `long_double` (= c_void, placeholder, NOT ABI-correct).
- `__m128/__m128d/__m256/__m256d/__m512/__m512d`: used by name in the guide (no definition); declarations import core::arch::x86_64 types of the same names.
- Mapped without definition in guide: `long int` -> c_long, `long long int` -> c_longlong (guide states those C types).

## What the guide states about types / typedefs / ILP64
- Nothing about ILP64/LP64, no typedef or struct definitions anywhere in the chapter (grep for ilp64, typedef, struct, _Complex: no hits; positive control: the same grep finds __m128 etc.).
- Sort note (L111862): workspace size is returned as `int`; largest supported len is 536851936.
- Naming convention (L106104 ff): `amd_vr<s|d><4|8|16|2|a>_<func>`; `f` suffix = single precision; `amd_fast*` = fast scalar.

## Garbled prototypes (page-header watermark "AOCLAPI Guide (68552)" interleaved)
- 56 declared lines were garbled by the watermark; each was reconstructed by a mechanical check that the garbled text is exactly an interleave of the reconstructed prototype and the watermark string (provenance note says so). Only the shape (arg names/types) came from sibling clean lines' pattern, and the interleave check confirmed it.
- UNKNOWN: amd_vrd2_sincos L108016: garbled text is one character short of a full interleave, so the check fails; the likely `void amd_vrd2_sincos(__m128d x, __m128d *sin, __m128d *cos)` (matching clean siblings L108008, L108677, L109280) was NOT written.
- Wrapped prototypes (continuation on next line) were joined: the 4 fmaxi/fmini, 4 add/sub/mul/div i-variants x2, linearfrac, stablesort_ascend.

## Duplicates / discrepancies
- 20 amd_fast* scalar prototypes are stated twice (first at L107720-L107916, again in the per-domain section L110528-L111023); declared once at the first line, identical signatures; provenance note names the later line.
- Intro usage examples (L106139, L106143, L106147) are stated with `;`. amd_vrs4_expf and amd_vrd8_sin match the formal ones. amd_vrsa_cosf example is `void amd_vrsa_cosf(int n, float *x, float *y);` which DIFFERS from the formal L109818 `void amd_vrsa_cosf(int len, const float *src, float *dst)`; the formal one is declared (ABI identical, names/const differ).
- Linear Fraction Functions domain page says "Contents need to be added." (L111833); the linearfrac prototypes are under Arithmetic Functions.
- Guide declares `amd_fastexp` etc. without the trailing `#` anchor marker of other entries; no other difference.
- Only double-precision cdfnorm/cdfnorminv exist per the guide (L111076).
