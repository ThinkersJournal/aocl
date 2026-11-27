# AOCL-RNG C declarations: notes

Sources read: input-rng-guide6.txt (ch. 9, L111897-L115986; AOCL-RNG API reference is L112005-L113765, SecureRNG ignored), rng42_html__sources_rng.rst, rng42_html__sources_rngapi.rst. Nothing else.

## Counts
- Declared in rng_decls.rs: 80 (80 CSV rows, all DECLARED, 0 UNKNOWN, no duplicate names).
- 39 s/d families x 2 = 78, plus drandinitialize_secrng (1, double only) and get_rngversion (1) = 80.
- Each declaration was machine-checked against the guide text: 64 matched the text exactly (whitespace ignored); 16 had the page-header watermark interleaved and were checked to be an exact interleaving of the declaration with a contiguous piece of "AOCLAPI Guide (68552)" (no mismatches).
- rng_int_t-width-dependent: every function except get_rngversion uses rng_int_t.

## Variants the text gives per family
- Every s/d family: three forms: UPPERCASE (SRANDxxx/DRANDxxx, all args by pointer), lowercase with trailing underscore (all args by pointer), lowercase no underscore (C style, scalar inputs by value; array/output/inout args by pointer). Only the last is declared. 4.2 rng.rst:74-82 states the same split (underscore = Fortran names, scalars by value in C).
- drandinitialize_secrng (L112152): the text gives ONE form only: lowercase, no underscore, no s-variant, genid/subid by value (garbled, see below).
- get_rngversion: GET_RNGVERSION(void), get_rngversion_(void), get_rngversion(void), all `const char *`, no params (L113756-L113758). Declared the lowercase one.
- Text oddity, not declared (underscore variant): drandnegativebinomialreference_ at L113553 shows `double p` by value, unlike its siblings; excluded as a Fortran-style variant.
- Const-ness: the text never writes `const` on any parameter (only on the return of get_rngversion), including [in] arrays (seed, xmu, c, ref, p). All pointers are therefore `*mut`. Parameter docs say [in]/[out]/[inout] but that is not C const.
- Floating types: `float` -> core::ffi::c_float (s-variants), `double` -> core::ffi::c_double (d-variants). `char` -> core::ffi::c_char.
- Rust keyword: parameter `ref` is written `r#ref` (name as given in the text).
- Written as `extern "C" { ... }` literally as requested (no `unsafe`, no `#[link]`), no imports; paths are fully qualified core::ffi.

## Unresolved types (not defined by the text; used as opaque alias names, not defined in rng_decls.rs)
1. `rng_int_t` - used by every declaration. The text never defines it (see widths below).
2. `RNG_SRANDINITIALIZEUSER_UINI`, `RNG_SRANDINITIALIZEUSER_UGEN` - used by srandinitializeuser (L112211), never defined in either source (4.2 rngapi.rst:70-71 also lists only the DRAND typedefs by name).
3. `RNG_DRANDINITIALIZEUSER_UINI`, `RNG_DRANDINITIALIZEUSER_UGEN` - the text DOES state these typedefs (L112219-L112222) but rng_decls.rs is "extern block only", so they are referenced by name and the stated shapes are recorded here:
   - `typedef void (*RNG_DRANDINITIALIZEUSER_UINI)(rng_int_t*, rng_int_t*, rng_int_t*, rng_int_t*, rng_int_t*, rng_int_t*, rng_int_t*)` (L112219-L112220; 7 pointers)
   - `typedef void (*RNG_DRANDINITIALIZEUSER_UGEN)(rng_int_t*, rng_int_t*, double*, rng_int_t*)` (L112222)
   The s-variant typedef shapes are NOT stated; do not infer them.

## What the text says about integer widths / ILP64 / rng_int_t
- No statement of rng_int_t's width, signedness, or any ILP64/LP64/int64 option anywhere in the three files (grep for ilp, lp64, int64, "integer size", "width", rng_int in the 4.2 .rst files: 0 hits; positive control: the same grep on the guide finds rng_int_t lines, so it works; the 4.2 files simply do not contain the word).
- Only incidental uses: `RNG_INT_T(...)` used like a cast/macro in lref size formulas (L113485, L113573, L113575), undefined; the Sobol example declares `rng_int_t genid, subid, ldimension, lstate, info;` (L114336) while its wrapper function takes plain `int` arguments (L114331-L114333).
- Fortran examples use plain `INTEGER` (4.2 rng.rst, guide L114441 on); no width is stated for it either.
- 4.2 rng.rst:74-82 describes the C vs Fortran differences (underscore, no workspace args, scalars by value, char by value) but says nothing about integer width.
- The only fixed-width types in the text are uint16_t/uint32_t/uint64_t in the SecureRNG section (ignored).

## 4.2 sources
Neither 4.2 file contains a single C prototype body. rngapi.rst consists of Doxygen `.. doxygenfunction::` directives (names only; the six forms per family plus two `doxygentypedef`). rng.rst has only prose and Fortran snippets. They were used only to cross-reference names (CSV note column gives the rngapi.rst line of the lowercase directive). Every declared name appears in rngapi.rst (checked by script; see CSV notes).

## Garbled prototypes (watermark "AOCLAPI Guide (68552)" interleaved in the prototype text)
Declared, because the garbled text is an exact interleaving of the stated declaration and the watermark (unique and verified by script), the function name and `#` terminator are intact, and (for the d/s pairs) the sibling variant lines are clean:
drandinitialize L112043-44; drandinitialize_secrng L112152-53 (the only damaged copy of this function; genid/subid by value reconstructed from the interleaving, no sibling to confirm: lowest-confidence entry); drandinitializebbs L112243-44; drandblumblumshub L112349; srandbeta L112414; drandcauchy L112446; drandgamma L112574; drandlogistic L112644; dranduniform L112780; srandmultinormal L112884; drandmultistudentst L112935-36; drandmultinormalreference L112981-82; drandmultistudentstreference L113067-68; drandpoissonreference L113600; drandmultinomial L113640-41; drandskipahead L113687.
Also garbled but in excluded underscore variants: srandinitialize_ L112034, drandinitializeuser_ L112209, srandhypergeometric (UPPERCASE) L113233.
UNKNOWN count: 0.

## Other
- source_line in the CSV and the `// file:L<n>` comments are the L-numbers of the line where the lowercase prototype starts (the original document line numbers from the `L<n>|` prefix).
- The doc contains a few oddities not corrected: generic descriptions say "double precision" for the s-variants, lstate/lseed notes say "-1" or "< 1" inconsistently, and the guide says seven base generators (genid 1-7, incl. Sobol) while 4.2 says six. Not relevant to signatures.
- Generator script (not a deliverable): scratchpad/cr/rng/gen/gen_rng.py.
