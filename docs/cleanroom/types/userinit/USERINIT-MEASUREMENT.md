# AOCL-RNG `(s|d)randinitializeuser`: is the ~128 KB out-of-bounds write our error or a library defect?

Behavioural measurement, clean-room. Date 2026-11-27. Everything is under
`C:/Users/cires/AppData/Local/Temp/claude/4b8c15632be00fc5/83c2e441-df7a-4c4b-b87f-7c2eddc43692/scratchpad/userinit/`
(`uprobe.rs` harness; `out/` results; `out/harness/` copies of the scripts).

## 0. Verdict (details and evidence below)

**OUR error for the ~128 KB write: the write is not a fixed library behaviour but a consequence of our UINI callback leaving `state[0]` at 0.**
Measured protocol (behavioural, NOT stated in the allowed guide lines): after UINI returns, the library reads `n = state[0]` (the
user's state length, set by UINI) and appends the UGEN address as two integer slots `state[n]=hi32`, `state[n+1]=lo32`, then
**re-reads `state[0]`** and writes `state[state[0]+2] = state[1]`. With `n >= 1` every write stays inside `state[0..n+3)`,
independent of buffer size, and the documented generation call then works with **no harness patch**. With `n = 0` (the earlier
probe's UINI wrote nothing) `state[0]` is overwritten by `hi32` in the first store, the re-read picks up `hi32`, and the third store lands at
`state[hi32+2]`: 0x7ff6+2 = slot 32760 = ~128 KB (LP64) / ~256 KB (ILP64).

Not a clean bill for AMD, and not proven either way on documentation:
* The allowed guide lines (112000-114100) say **nothing** about the user-generator state length, the meaning of `state[0]`, or what UINI/UGEN must do. So our earlier call did not violate any rule stated in those lines; the rule we violated is an undocumented one that we inferred from behaviour.
* Library weaknesses that are real regardless of our call: (a) `n = 0` is a plausible user-state size, yet it makes the library write `hi32+2` slots away (aliasing: it re-reads a slot it just overwrote); (b) the library never validates `n` against `lstate` (`n=98`, `lstate=100` writes slot 100; `n=1000` writes slot 1000); (c) with `lstate=-1` the guide's Note says the function "returns without initialising the generator", but for the user variant UINI is called (with `lstate_arg=-4`) and the state is written; `lstate` is not updated, so the documented length query cannot return a length for the user variant.
* Not consulted: the guide chapter "User's Own Custom Generator" (programmer's guide 9.5, TOC at lines 114072/114092; its body is outside the allowed range). It may define `state[0]`. **Read it before reporting a defect to AMD**; if it defines this protocol, the verdict is firmly "our error".

## 1. Inputs, method, environment

Inputs used: guide `C:/aocl-docs/guide-6.0.txt` lines 112000-114100 only; the two DLLs
`C:/Program Files/AMD/AOCL-Windows/amd-rng/lib/{LP64,ILP64}/rng_amd.dll` and `dumpbin /exports,/dependents` of them (`out/dumpbin-*.txt`;
exports include `drandinitializeuser` 0x27b10 / `srandinitializeuser` 0x295b0 etc. in LP64); the earlier probe
`aocl-wt-cleanroom2/docs/cleanroom/types/` (TYPES-MEASUREMENT.md, probe-src/rng_user_probe.rs, run-rng.ps1, raw-rng-*) and `tools/link-all.ps1` for the per-process PATH method. **No header or source under the AOCL install was read, no existing bindings, no disassembly.**
Disclosure: one `grep -n -i "User's Own Custom Generator"` over the whole guide printed two lines outside the allowed window
(a TOC line at 114108 and one sentence at 114419: "User's Own Custom Generator could be used as base generator for all distribution generator."). Nothing was read from the chapter body and nothing here relies on those lines.

Machine: AMD Ryzen 9 7940HX, Windows 11 Pro 10.0.26300. rustc 1.100.0 (0ef1bc386 2026-11-09), `rustc --edition 2021 -O uprobe.rs`.
AOCL: the task states 6.0.68 and the guide is 68552 (6.0); **the DLLs carry no version resource** (FileVersion empty), so 6.0.68 is not verifiable from the binaries. LP64 DLL SHA-256 `CFAFE3D5...A613F5` (467352 B), ILP64 `F882D52B...6194F` (514456 B).

Harness (`uprobe.rs`): `uprobe parent <timeout> ...` spawns itself as `child` (one process per call, 40 s timeout, records exit code); child
reserves a 64 MiB PAGE_NOACCESS region, commits read/write only the pages of the state buffer, with the end of the state flush with the last committed byte, so **any access from 1 MiB before to ~47 MiB after the state (past the exact byte end) raises 0xC0000005**; a vectored handler prints the fault offset from the state start and whether RIP is inside `rng_amd.dll`. Buffer fill 0x00 (default) so a value written is visible; the changed-slot scan covers the buffer. PATH is built per child (`run.ps1`, copied from link-all.ps1): LP64 sees no ILP64 dir, ILP64 sees ILP64 dirs first; the caller PATH is not inherited. No process is killed by image name (only the timed-out child handle). Commands: `pwsh -File run.ps1 <LP64|ILP64> 40 key=val ...` driven by `battery2.ps1`, `battery3.ps1` (and `battery.ps1`, the first, superseded run); summary `summarize.ps1` -> `SUMMARY-user-runs.txt` (in the evidence archive: `docs/cleanroom/evidence/`) (212 user-init rows). Raw per-run output: `out/raw-<build>-<fn>-<case>.txt` (233 files; first line is the exact command). Slot = `rng_int_t`: 4 B LP64, 8 B ILP64.

## 2. Item 1: what the guide says, and what the earlier call passed

Verbatim (guide-6.0.txt, line numbers absolute):

(a) State length for the user init: **the allowed lines contain no statement of any state length for `randinitializeuser`**. The only length text for the neighbouring entry points is generic:
* l.112137-112142 (`randinitialize`): `lstate - [inout]` / `Length of the state vector` / `Value of lstate is dependent on the generator being used.` / `If , then lstate is set to the value required by the currently selected generator and the function returns.` (the `-1` was lost in extraction; the Note at l.112050-51 restores it)
* l.112050-51: `If lstate = -1, or lseed = -1 then function returns without initialising the generator.` / `If both lstate = -1 and lseed = -1 then both are set to their before returning.`
* l.112171: `Vector that will define the generator and hold its current state, state(lstate).`
(b) `lstate`: only the generic `[inout]` text above; nothing specific to the user variant (the user entry at l.112194-112222 has no parameter descriptions at all, only prototypes and two typedefs).
(c) Callback typedefs, l.112219-112222, verbatim:
```
typedef void (*RNG_DRANDINITIALIZEUSER_UINI)(rng_int_t*, rng_int_t*, rng_int_t*, rng_int_t*,
rng_int_t*, rng_int_t*, rng_int_t*)#
typedef void (*RNG_DRANDINITIALIZEUSER_UGEN)(rng_int_t*, rng_int_t*, double*, rng_int_t*)#
```
What UINI/UGEN must do: **not stated** in the window (no prose). The S typedefs are named in the prototypes (l.112198-112216) but not given.
(d) User-generator state layout: **not stated**. (Only BBS, l.112300ff., describes a state layout, which is not this function.)

Earlier probe's exact call (rng_user_probe.rs, cases `init/gen/lquery`, `genid=5 subid=7`):
`drandinitializeuser(uini_cb, ugen_cb, 5, 7, seed, &lseed, state, &lstate, &info)`; `seed` = 8 slots 100..107; `lseed`=3; **`lstate`=100 (or -1 in `lquery`)**;
`state` = zero-filled `Buf::new(1_000_000, w, 0)` = 1,000,000 slots = 4,000,064 B LP64 / 8,000,064 B ILP64; `info` 2 slots =0xAA. The UINI callback only recorded its arguments (in `lquery` it wrote `*arg5`); **it never wrote `state[0]`**.
Buffer vs required length: the guide states no required length, so "buffer >= required" cannot be evaluated against the guide. Against our own `lstate=100`, the buffer (1,000,000 slots) was 10,000x larger. The overrun is relative to `lstate`, not to the buffer: with a 4096-slot buffer and `lstate=100` the earlier probe faulted (`out` reproduces: `*-noprotocol-buf4096` FAULT at slot 32760).
Re-run of that exact call with an end guard (`out/raw-LP64-duser-earlier-1M-slots-lstate100.txt`): `info=0`, nonzero slots `state[0]=0x7ff6, state[1]=0x26a76d60, state[32760]=0x26a76d60`, i.e. last written byte 131044 of a 4,000,000 B buffer, while `lstate*4 = 400`. ILP64 same call: `state[0]=0x7ff6, [1]=0x26a76d60, [32760]` = byte 0x3ffc0 = 262080.

**Control: non-user `(d|s)randinitialize` with its queried length** (`lstate=-1,lseed=-1` per l.112050-51 gives lstate/lseed; then init with buffer == lstate*w exactly):

| build | fn | genid | queried lstate / lseed | buffer | result | last changed slot (end byte) |
|---|---|---|---|---|---|---|
| LP64 | d,s | 3 (MT) | 633 / 624 (info=1) | 633 slots = 2532 B | info=0, no fault | 632 (2532 = buffer end exactly) |
| ILP64 | d,s | 3 | 633 / 624 | 633 slots = 5064 B | info=0, no fault | 632 (5064 = end) |
| LP64 / ILP64 | d,s | 1 | 16 / 1 | 16 slots | info=0 | 15 (end) |
| LP64 / ILP64 | d,s | 2 | 20 / 4 | 20 slots | info=0 | 19 (end) |
| LP64 / ILP64 | d,s | 4 | 61 / 6 | 61 slots | info=0 | 60 (end) |

Writes stay within the stated length and use it to the last slot. NON-VACUOUS negative control: same genid=3 call with the buffer **one slot short** (632 slots, lstate=633 still passed): `FAULT WRITE` at byte 2528 (LP64) / 5056 (ILP64) = slot 632, **inside rng_amd.dll**, in all of d/s x LP64/ILP64 (`raw-*-genid3-auto-minus1-NEGCTL.txt`). Guard self-test (`raw-selftest.txt`): harness writes one byte past a 400 B state -> fault at offset 400, RIP outside the DLL (so the guard detects an exact one-byte overrun and distinguishes harness faults from DLL faults).
=> The non-user init obeys its guide-stated length; the user init does not, under the same call pattern, *when UINI leaves `state[0]=0`*.

## 3. Item 2: scaling, address dependence, ILP64 (all rows in `SUMMARY-user-runs.txt` (in the evidence archive: `docs/cleanroom/evidence/`); D shown; S rows identical)

Model tested: `n=state[0]` after UINI; `state[n]=hi32, state[n+1]=lo32` of the UGEN address; then `state[state[0]+2]=state[1]` (re-read).

| case (LP64 / ILP64 same unless noted) | observed | what it shows |
|---|---|---|
| UINI writes nothing (n=0), buffer 1M slots, lstate=100 | writes `0=0x7ff6, 1=lo, 32760=lo` (byte 131040 / 262080) | the earlier ~128 KB write |
| n=0, buffer = lstate = 100 / 4096 / 32760 | FAULT WRITE, in-DLL, slot 32760 (byte 131040 LP64, 262080 ILP64) | overrun detected exactly |
| n=0, buffer = lstate = 32761 / 32762 / 32768 / 65536 | ok, last write slot 32760 | boundary pins index 32760 exactly |
| n=0, buffer 1 MiB or 16 MiB, lstate 100 or = slots | same three writes at the same offsets (131040 B LP64) | **does NOT scale with buffer or lstate: fixed**, = (hi32+2)*slot |
| UGEN = fn a / fn b (same hi32=0x7ff6, lo 0x..6d60 / 0x..6ed0) | `state[1]` and `state[32760]` follow lo; index unchanged | lo32 is stored; index from hi32 only |
| UGEN = trampoline at hi32 = 0x0 / 0x1 / 0x5 / 0x20 / 0x700 / 0x7000 (lo=0x123; hi=0: lo=0x10000123) | `state[0]=hi32, state[1]=lo`, second write at slot **2 / 3 / 7 / 34 / 1794 / 28674** = hi32+2 in all 6, LP64 and ILP64, D and S | **the library re-reads `state[0]` (now hi32) as an index** |
| UINI address varied (fn b; trampoline hi32=5) with UGEN fixed | stored values and index unchanged | UGEN address (not UINI) is stored |
| ILP64 (8-byte slots) | hi32 and lo32 still go in two separate 64-bit slots (`0x7ff6`, `0x26a76d60`), index hi32+2, byte offset 8x | splits the pointer into two integer slots regardless of width; not a 64-bit store |
| UINI sets `state[0]=0x5100`(seq 0x5100+i for 8 slots) | `state[20736]=0x7ff6, [20737]=lo, [20738]=0x5101` | index = UINI's `state[0]`; third write = copy of `state[1]` (0x5101), not hi32/lo32 |
| UINI sets `state[0]=n`, n=1 | `{1, hi, lo, hi}` in slots 0..3, buffer 4 / 100 / 1 MiB / 16 MiB identical | writes scale with n, not with the buffer |
| n=5 | `state[5]=hi, [6]=lo, [7]=0x7001 (=state[1])` | non-vacuous: 0x7001 differs from hi/lo |
| n=97, lstate=100, buffer exactly 100 slots | ok, last written slot 99 | conformant maximum (UINI is told `lstate-3` = 97) fits exactly |
| n=98 / 99, lstate=100, buffer 100 | FAULT WRITE, slot 100 (byte 400 / 800) | library trusts `n`; needs n+3 slots; no validation |
| n=1000, lstate=100, buffer 100 | FAULT WRITE slot 1000 | same |
| n=1, lstate=4, buffer 4 / buffer 3 (NEGCTL) | ok / FAULT WRITE slot 3 in-DLL | needs exactly n+3 = lstate slots |
| lstate 1, 2, 3 | `info=-6` ("parameter number 6 had an illegal value"), UINI not called, nothing written; lstate 4,5,6 and 0 accepted (lstate=0 and -1 are accepted yet UINI is called with lstate-3) | UINI is handed `lstate-3` (3 header slots); min accepted 4 |
| lstate = -1 (query), UINI n=1 | UINI called with arg `-4`, `{1,hi,lo,hi}` written, `info=0`, caller `lstate` stays -1 | query not honoured as documented; no length returned. (UINI writing `*lstate` not copied back: from the earlier probe, not re-measured) |
| lstate = -1, n=0, buffer 100 | FAULT at slot 32760 | a *query* call can overrun |
| UINI leaves garbage `state[0]` (fill 0xA5, earlier probe state otherwise zero) | FAULT at slot -1515870811 = signed 0xA5A5A5A5 (LP64), reads in ILP64 | `state[0]` is consumed as written by UINI/caller; uninitialised buffer is an index |
| **End-to-end, no patch**: UINI n=1 / 5 / 97, lstate=100, 100-slot buffer, then `(d|s)randuniform(n=4,a=0.5,b=2.0)` | `info=0`, UGEN called once with `n=4`, `arg1==state`, x = `[0.6875, 0.875, 1.0625, 1.25]` = 0.5+1.5*(i+1)/8, no byte beyond n*width changed; D and S, LP64 and ILP64 (16 runs: `raw-*-GEN-*`) | the documented API works with a conformant UINI; `{1,hi,lo,hi}` is exactly the earlier probe's "harness patch" (so the earlier "cannot be driven as documented" conclusion was our `state[0]`) |
| NEGCTL: same generation after n=0 init (big buffer) | `info=-4`, UGEN never called, x untouched | the earlier "-4 / unreachable" finding reproduced; non-vacuous contrast with the line above |

## 4. Item 3: the UGEN 5th parameter

The guide defines **four** UGEN parameters (l.112222: `(rng_int_t*, rng_int_t*, double*, rng_int_t*)`); there is no 5th, so none was measured. The end-to-end rows above use the 4-parameter callback only.

## 5. Vacuous or weak controls (and how each was handled)

1. First run used a single guard page after the state: an overrun far past it can land in other mapped memory with no fault. `ILP64-suser-noprotocol-buf100` printed "ok, 2 slots written" while the identical `duser` call faulted. **Discarded**; the whole battery was redone with the 64 MiB reserved window (now only accesses beyond ~47 MiB after / 1 MiB before the state could go undetected).
2. Zero fill makes a written 0 invisible: first trampolines had lo32=0, hiding the second write (e.g. `state[hi+2]=0`). **Redone** with code at page offset 0x123 (lo32 non-zero). With hi32=0 the writes at slots 1 and 2 both equal lo32, so hi=0 does not separately prove `hi+2` versus a fixed slot 2; the hi=1,5,0x20,0x700,0x7000 rows do.
3. First "buf3/lstate4" negative control faulted in my harness (UINI peeked 4 slots of a 3-slot buffer): vacuous. Fixed (peek 1 slot) and rerun; the in-DLL fault shown is the library's.
4. For n=1 the third write `state[3]` equals both `hi` and `state[1]`; only n=5 (0x7001) and the 0x5100 run discriminate the "copy of `state[1]`" rule.
5. Runs at fill 0xA5 (`*-fillA5`, run1 directory `out/run1-fill-a5-default/` kept for the record: pre-zero-fill, 1-guard-page results) show the library consuming uninitialised `state[0]`; they were not used for the offset conclusions.
6. The model (read `state[0]` after UINI, re-read before the third store) is inferred from 100+ writes, not observed in code; alternatives that give the same numbers for every row above were not found, but it is a behavioural fit.
7. "ok, no fault" means no access in the window, not that nothing was written: the changed-slot scan covers only the buffer (writes beyond it fault in the window, so "ok" rows have no writes outside it).

## 6. What remains unproven

* Whether the guide chapter outside the allowed range documents `state[0]` (decides "our error" vs "undocumented library contract"). A real UINI that sets `state[0]=n` is needed in any wrapper regardless.
* `genid` other than 5 and NULL `uini`/`ugen` (the earlier probe says UINI is always called; not re-measured); other AOCL versions; non-Windows; behaviour of `n` in `[lstate-2, lstate]` beyond the faults shown.
* That 6.0.68 is the version of these DLLs (no version resource).
