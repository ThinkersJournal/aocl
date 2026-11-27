# Reconciling this lane's name counts with the PM's (2026-11-27)

All comparisons are **by name only** (signatures are not compared). "Ours" = the committed `bindings/bindings.rs` of
`aocl-math-sys` (313 `pub fn`) and `aocl-rng-sys` (238 `pub fn`) at origin/main 98dbce0. The four name lists are in this directory.

## LibM
| | PM | this lane |
|---|---|---|
| guide names | 366 | **383** distinct `amd_` names (382 DECLARED + 1 UNKNOWN) |
| how extracted | not known to this lane | the Sonnet writer read all of guide L106064-L111896 and listed every `amd_` prototype, incl. 56 whose lines carry interleaved page-header watermark (each reconstructed only when the garbling is exactly reconstruction + watermark) |
| only in guide | 70 | **70** (`math-only-in-guide.txt`) |
| only in ours | 17 | **1** (`amd_vrd2_sincos`, UNKNOWN in the guide run) |

**383 - 366 = 17.** The most likely reading: the PM's extraction found 366 names and missed 17 that are in the guide; those 17 are in
our bindings, so the PM saw them as "only ours". They are **guide-declared in this lane's 382 rows, not only-ours.** This lane
does not have the PM's list of 17, so it cannot name them exactly; it can only check the PM's stated categories. All of these
are DECLARED here, with their guide line (from `math_provenance.csv`):
- `amd_stablesort_ascend_64f` L111876
- `amd_vrda_addi` L111404
- `amd_vrda_subi` L111456
- `amd_vrda_muli` L111507
- `amd_vrda_divi` L111559
- `amd_vrda_fmaxi` L110401
- `amd_vrda_fmini` L110431
- `amd_vrsa_addfi` L111418
- `amd_vrsa_subfi` L111470
- `amd_vrsa_mulfi` L111521
- `amd_vrsa_divfi` L111573
- `amd_vrsa_fmaxfi` L110387
- `amd_vrsa_fminfi` L110417
- `amd_vrd2_linearfrac` L111590
- `amd_vrd4_linearfrac` L111623
- `amd_vrd8_linearfrac` L111656
- `amd_vrda_linearfrac` L111689
- `amd_vrs4_linearfracf` L111606
- `amd_vrs8_linearfracf` L111639
- `amd_vrs16_linearfracf` L111672
- `amd_vrsa_linearfracf` L111704
(21 names; the three `linearfracf` rows at L111606/L111639/L111672 are watermark-garbled lines, which may be why a plain-text extraction misses them.)
To finish the reconciliation the PM intersects its 17 with this list; any of its 17 not on this list is a real discrepancy to report.

`amd_vrd2_sincos`: guide text line **L108016** (garbled; the neighbouring lines L108010-L108022 give its description): the raw line reads
`void amd_vrd2_sincos(__m128d xA, _O_CmL1A28PdI G*suind,e_(_6m8515228)d *cos)#`. The writer left it out because it could not confirm the reconstruction.

## RNG
A "name" here is a function-head token, case-sensitive. The guide states three forms per family (UPPERCASE, lowercase+trailing `_`, lowercase C).
- **249** (PM) = this lane's regexp count over guide L111897-L115986 of every prototype head (`rng-guide-all-prototype-names-249.txt`): 78 UPPERCASE + 78 trailing-underscore + 91 lowercase + 2 mixed-case (`is_RDRAND_supported`, `is_RDSEED_supported`). **Same 249.**
- **235** (PM, match ours) = 249 - 14. The 14 are the **SecureRNG** functions in the same chapter (`get_rdrand*`, `get_rdseed*`, `is_RDRAND/RDSEED_supported`), which are not in aocl-rng-sys.
- **238** names in ours = 235 + 3: `GET_RNGVERSION`, `get_rngversion`, `get_rngversion_` are in ours but not in the 249 list (this lane's regexp does not match that function's head line; the writer found `get_rngversion` in the text, so this is a counting-method gap, not a missing function).
- **80** (this lane's writer) = the lowercase C forms only, SecureRNG excluded: 39 s/d families x 2 = 78, plus `drandinitialize_secrng` (double only) and `get_rngversion` (`rng_provenance.csv`, 80 rows). They match ours' 80 lowercase no-underscore names 80/80 both ways. The other 158 of ours (UPPERCASE and `_` forms) are the Fortran-style forms the writer was told not to declare.

## Integer type of the n / seed / state arguments (rng_int_t)
The guide **never defines `rng_int_t`** (no typedef, nothing on LP64/ILP64). What it states:
- C prototypes use `rng_int_t` for every integer argument, by value for `genid`, `subid`, `n`, `ldx`, and by pointer for `seed`, `lseed`, `state`, `lstate`, `info` (L112030-L112044 and throughout), e.g. L112040-L112041: `void srandinitialize(rng_int_t genid, rng_int_t subid, rng_int_t *seed, rng_int_t *lseed, rng_int_t *state, rng_int_t *lstate, rng_int_t *info)`.
- Prose calls them "integers" only: L113118 "n - [in] Number of integers to generate, n > 0"; L111922 "random numbers of type Integer, Single Precision and Double Precision".
- The Fortran examples declare `INTEGER LSTATE,N` / `INTEGER SEED(1),STATE(LSTATE)` etc. (e.g. L114443-L114445, 113 `INTEGER` lines), with no kind or width.
- A user-generator callback typedef at L112219-L112222 also uses `rng_int_t*`.
So width and signedness are UNKNOWN from the guide; a ruling is needed (the PM's).
