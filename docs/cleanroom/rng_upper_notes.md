# AOCL-RNG uppercase forms, guide-6.0 chapter 9 (input: guide-ch9.txt, 4090 lines, read as latin-1)

Counts: 79 DECLARED, 0 UNKNOWN. That is 78 SRAND/DRAND forms (39 S/D pairs) plus GET_RNGVERSION.
Per section: 9.3.1 Base 8, 9.3.2 Continuous 40, 9.3.3 Discrete 26, 9.3.4 Multiple Streams 4, 9.3.5 Utility 1 (CSV). (SRAND/DRAND pairs: 39.)

Verification: each declaration was regenerated as C text from the Rust and compared whitespace-insensitively to the
guide prototype (joined across line wraps, ending at `)#`); 79/79 matched. The one exception is the watermark reconstruction below.
Enumeration: prototypes starting at `void|const char *` + UPPERCASE name + `(`; positive control: strict-grammar regex flagged exactly the one garbled prototype.

Watermark reconstruction (1): SRANDHYPERGEOMETRIC, L1337. Garbled `(rng_AiOntC_tL*An,PrInGgu_iidnet_(6t 8*n55p2, )rng_int_t *ns,...`.
Removing the subsequence "AOCLAPIGuide(68552)" (watermark spaces ignored; the extraction also dropped a cand space) leaves a unique residual
`rng_int_t*n,rng_int_t*np,` (checked exhaustively by script; 2 split paths, same residual). Result equals DRANDHYPERGEOMETRIC's first params.

Oddities:
- SRANDINITIALIZEUSER / DRANDINITIALIZEUSER: genid and subid are BY VALUE (rng_int_t), and uini/ugen are RNG_*_UINI/UGEN aliases by value, contrary to the "all by pointer" premise. Declared as stated. Aliases not defined here (the guide gives typedefs for DRAND only, as function-pointer types; not defined per instruction).
- Uppercase SRANDINITIALIZE / DRANDINITIALIZE (and others) use pointers for genid/subid, but the lowercase non-underscore forms take genid/subid by value (ignored here).
- GET_RNGVERSION returns `const char *` (guide states const here), declared `*const core::ffi::c_char`; takes (void) -> no params.
- Rust keyword parameter `ref` written r#ref (SRAND/DRAND MULTINORMALREFERENCE, MULTINORMALR, MULTISTUDENTSTREFERENCE, MULTISTUDENTSTR, GENERALDISCRETE, *REFERENCE forms).
- Discrete generators use rng_int_t *x; S-forms use float, D-forms double, as stated.
- Excluded: lowercase and trailing-underscore forms, drandinitialize_secrng (lowercase only), AOCL-SecureRNG section, typedefs.
- Guide pairs SRAND* with float and DRAND* with double; return values are void except GET_RNGVERSION.
