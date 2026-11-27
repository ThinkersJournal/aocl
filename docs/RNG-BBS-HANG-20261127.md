# `BlumBlumShub::new` (`drandinitializebbs`) never returns for some prime pairs (measured 2026-11-27)

**Predicate / ref.** AOCL 6.0.68, Windows 11, LP64; `aocl-rng` at the branch of #47. Each probe ran in a CHILD PROCESS with a hard
timeout (a hang inside the library cannot be timed out in-process): "HANG" = no return within 3 to 6 s at 100 % CPU (a working
call returns in under 0.1 s). Single-limb primes (base 2^30 digit), `p != q`, both prime, both = 3 (mod 4), `nbits` 8, seed 12345
(coprime to pq) unless the row says otherwise. Linux unmeasured.

## What the guide states (`C:/aocl-docs/guide-6.0.txt`, "AOCL API Guide (68552)", randinitializebbs, lines 112224-112300 and 114244+)
- `p`, `q`, `s` are stored base `B2 = 2**30`, `lp <= 25`, `lq <= 25`, `ls <= 25` digits ("the number of digits ... <= 25").
- `p mod 4 = 3`, `q mod 4 = 3`; "two large prime values p and q"; seed constraint `SEED(i) > 0`; `s` between 1 and m-1 with gcd(s, m) = 1.
- `nbits`: "if nbits < 1, then nbits = 1, if nbits > 15, then nbits = 15" (so 1, 8, 15 are all legal).
- **No bound on the size of `p` or `q` is stated** ("As p, q and s can be of an arbitrary size"). (Elsewhere in the same guide, the generic
  `randinitialize` text garbles the base as `B = 224`; the BBS routine text says 2**30.) So there is no documented limit to validate against.

## Measurements
| probe | p | q | pq | result |
|---|---|---|---|---|
| decade 1e3 | 991 | 983 | 9.7e5 | OK |
| decade 1e4 | 9967 | 9931 | 9.9e7 | OK |
| decade 1e5 | 99991 | 99971 | 1.0e10 | OK |
| decade 1e6 | 999983 | 999979 | 1.0e12 | HANG |
| decade 1e7 / 1e8 / 1e9 | 9999991 / 99999971 / 999999883 | previous 3 mod 4 prime | 1e14 / 1e16 / 1e18 | HANG (all three) |
| scan | 225307 | 225299 | 5.1e10 | **HANG** |
| scan | 279047 | 279023 | 7.8e10 | OK |
| scan | 369751 | 369739 | 1.4e11 | **HANG** |
| scan | 406951 | 406907 | 1.7e11 | OK |
| scan / bisect | 541271 / 549979 / 662483 / 663959 / 675083 / 676523 / 683567 / 687083 | each with the previous 3 mod 4 prime | 2.9e11 to 4.7e11 | OK (all) |
| scan / bisect | 644731, 646159, 688799, 690607, 693799, 718723, 719639, 730399, 774959, 966011, 1005527 | each with the previous 3 mod 4 prime | 4.2e11 to 1.0e12 | HANG |
| scan | 812527 | 812519 | 6.6e11 | **OK** |
| mixed | 999983 | 30011 | 3.0e10 | HANG (one big prime is enough) |
| mixed | 30011 | 1000003 | 3.0e10 | HANG |
| seed | 30011 / 40031, s = 1, 2, 12345 | | | all OK (s = 1 is NOT a cause; its output is the constant 65793) |
| nbits | 30011 / 40031, nbits 1, 8, 15 | | | all OK (nbits is NOT a cause) |
| seed / nbits | 999983 / 1000003, any s or nbits | | | HANG regardless |

## Conclusion
- Every pair with both primes <= 99,991 returned at once (10 of 10 probes below 1e5). Above about 2e5 some pairs hang and some do not, **not
  monotonically in size** (225307/225299 hangs; 541271/541267 and 812527/812519 work), and none of the simple properties tested
  (`(p-1)/2` prime, `p mod 8`, `p mod 3`) separates the working from the hanging pairs. A hang needs only one bad prime. The seed
  and `nbits` are not involved (the earlier "seed 1 / nbits hang" observation came from probes that also used large primes).
- Cause **not determinable** from this data, and the library documents no bound: per the rule "no guess", the wrapper does **not**
  validate (an `Err` on a legal pair would be a second defect). The documented warning on `BlumBlumShub::new` now states the measured
  facts, and `BlumBlumShub` is deprecated by AMD itself in favour of SecureRNG (guide caution at line 114250).
- Published versions with `BlumBlumShub::new`: `aocl-rng` (the safe wrapper crate) has exactly ONE published version, 0.1.0 (crates.io
  API lists only that; the .crate URLs for 0.2.2/0.2.4/0.2.5 return 404), not yanked; its `src/lib.rs` contains `pub struct BlumBlumShub` calling
  `drandinitializebbs`. Current main's `aocl-rng` (0.2.24) also has the constructor (the doc comment fixed in this PR). No version in between exists.
  Tests in this PR only use the verified prompt pair 30011 / 40031 (and other pairs below 100 000 that returned in under 0.1 s); no test calls
  the library with a pair known to hang, so a plain test cannot hang CI. This is a liveness defect (a safe function that spins forever), not memory unsafety: no advisory.
