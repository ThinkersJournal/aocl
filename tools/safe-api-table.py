#!/usr/bin/env python3
"""Build docs/safe-api-audit/AFFECTED-VERSIONS.md from the per-version result files (tools/safe-api-versions.py): per crate and function
family, the first and last PUBLISHED version in which any short case of the family faulted (HOLE) when the generated / hand-written
guard-page harness ran against that version's own .crate, and the version that fixes it. `FIXED` is the only hand-entered column
(from the merge/publish record); everything else is read from the csv files.
    python tools/safe-api-table.py > docs/safe-api-audit/AFFECTED-VERSIONS.md
"""
import csv
import glob
import os
import re

D = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs", "safe-api-audit")
# fixed-in, as of the audit head (PUBLISHED = on crates.io; MERGED = on main, not yet published; ARMED = in the gate; PENDING)
FIXED = {
    ("aocl-sparse", "*"): "0.2.46 (published 2026-11-28)",
    ("aocl-blas", "gemmt"): "0.2.47 (published in 0.2.49; 0.2.49 measured clean)",
    ("aocl-lapack", "generic"): "0.2.48 (group A #66, merged; unpublished)",
    ("aocl-lapack", "typed"): "0.2.49 (group B #67, merged; unpublished)",
    ("aocl-lapack", "eigen"): "PENDING: #68 (0.2.50 armed)",
    ("aocl-data-analytics", "*"): "PENDING: #71 (kernels, 0.2.53)",
    ("aocl-fft", "*"): "PENDING: #72 (alignment, 0.2.54)",
    ("aocl-lapack", "factor"): "PENDING: #69 (next after #68)",
}
HEADERS = {
    "aocl-sparse": "MEASURED families (313 tests: vector_lengths 48, dense_lengths 152, raw_array_lengths 72, remaining_lengths 41): the safe CSR/CSC/ELL/DIA/BSR level-2/3 kernels, conversions and solvers of real f32/f64 and complex c32/c64. NOT MEASURED: `blkcsrmv`, `create_csc`, `create_tcsr` (no safe constructor / sink), anything not named above.",
    "aocl-blas": "MEASURED families (506 tests, generated): the level-1/2/3 free functions of f32/f64/c32/c64 including gemmt. NOT MEASURED: the trait methods with explicit `inc`/`ld`, pack/compute/batch, `rotm`, `sdsdot`, `csrot`, `zdrot`.",
    "aocl-lapack": "MEASURED families (586 tests, generated): the typed d/s/z/c free functions and the generic `Scalar` trait methods listed below (370 holes in 120 families before the fixes). NOT MEASURED: the band functions `gb*`, `gesvdx`, `orgbr`, `lasrt`, the trait-level `RealScalar::geev` / `ComplexScalar::geev` called directly. `?lacpy` / `?lange` panic by documented contract (12 of the 586 tests exit with a panic, no fault).",
    "aocl-data-analytics": "MEASURED families (kernels and statistics 274 tests; handle models 58 tests in the section below): the free statistics functions, the five kernel / distance functions, KMeans, Pca, KNearestNeighbours, Linmod, Dbscan, DecisionTree, DecisionForest, Svm. NOT MEASURED: `Nlls` (callback-driven), `Nlls::set_weights`, `Datastore::extract_f64`, `Handle` options.",
    "aocl-fft": "MEASURED families (40 tests): `Plan` DFT / r2c / c2r length checks and the plan-time alignment rule (f64 and f32). NOT MEASURED: plan creation flags other than the defaults, the 2-D / 3-D plan constructors beyond the length cases listed.",
}
EIGEN = {"geev", "syevd", "heevd", "syevr", "heevr", "sygv", "hegv", "gees", "ggev"}
FACTOR = {"gebrd", "larfg", "orgtr", "ungtr", "orglq", "unglq", "ormqr", "unmqr", "ormlq", "unmlq", "gesvd", "gesdd", "gelsd", "gelsy"}
GENERIC = {"gesv", "getrf", "getrs", "getri", "posv", "potrf", "potrs", "potri", "sysv", "sytrf", "sytrs", "hesv", "hetrf", "hetrs", "gtsv", "ptsv", "gels", "geqrf", "gelqf", "syev", "heev"}


def vkey(v):
    return tuple(int(x) for x in v.split("."))


def load(crate):
    out = {}
    for p in glob.glob(os.path.join(D, f"{crate}-*.csv")):
        m = re.match(re.escape(crate) + r"-(\d+\.\d+\.\d+)\.csv$", os.path.basename(p))
        if not m:
            continue
        out[m.group(1)] = list(csv.DictReader(open(p, encoding="utf8")))
    return out


def family(crate, test):
    if crate == "aocl-fft":
        m = re.match(r"(f64s|f32s)::(.+)", test)
        return m.group(2) if m else None
    if crate == "aocl-sparse":
        m = re.match(r"(real_f64|real_f32|cplx_c64|cplx_c32)::(\w+?)_(short|col|block|rowptr|no_|t_|controls?|nnz_)", test)
        return (m.group(2) if m else None)
    m = re.match(r"(.+?)_(f64|f32|c64|c32)_", test)
    return m.group(1) if m else None


def main():
    print("# Safe-API audit: first / last affected PUBLISHED version per crate and function family\n")
    print("Measured, not read: for every published version of the crate the guard-page harness (`tools/gen-blas-harness.py`, "
          "`tools/gen-lapack-harness.py`, and the hand-written sparse files `vector_lengths.rs`, `dense_lengths.rs`, `raw_array_lengths.rs`) was built "
          "against that version's own `.crate` and dependencies and every test run in its own process (`tools/safe-api-versions.py`). "
          "HOLE = the process died with an access violation on a call with one buffer one element short. Result files: `results-*`/`<crate>-<version>*.csv` "
          "in this directory. A family not listed was refused or accepted without fault in every version measured. `gesv` (lapack) is the only measured LAPACK "
          "function that never faults. Yank and GHSA are CireSnave's (board 526 B / 527).\n")
    for crate in ("aocl-sparse", "aocl-blas", "aocl-lapack", "aocl-data-analytics", "aocl-fft"):
        data = load(crate)
        vers = sorted(data, key=vkey)
        fam = {}
        for v in vers:
            for r in data[v]:
                f = family(crate, r["test"])
                if not f or "control" in r["test"]:
                    continue
                fam.setdefault(f, {}).setdefault(v, []).append(r["outcome"])
        print(f"## {crate}\n\nVersions measured: {', '.join(vers)}.\n\n**{HEADERS[crate]}** \"0 holes\" means 0 holes in the measured families only: it is NOT a statement that the crate is clean.\n")
        print("| function family | first affected | last affected (published so far) | holes (short cases) at last affected | family fixed in (first published version with 0 holes for this family) |\n|---|---|---|---|---|")
        rows = 0
        for f in sorted(fam):
            hv = [v for v in vers if v in fam[f] and "HOLE" in fam[f][v]]
            if not hv:
                continue
            last = hv[-1]
            n = sum(1 for o in fam[f][last] if o == "HOLE")
            # fixed-in is MEASURED: the first published version after the last affected one in which the harness shows 0 holes for the family
            later = [v for v in vers if vkey(v) > vkey(last) and v in fam[f]]
            clean = [v for v in later if "HOLE" not in fam[f][v]]
            if clean:
                fx = f"{clean[0]} (measured 0 holes)"
            elif last == vers[-1]:
                fx = f"STILL FAULTING at {vers[-1]} (finding)"
            else:
                fx = "not measured"
            print(f"| {f} | {hv[0]} | {hv[-1]} | {n} | {fx} |")
            rows += 1
        print(f"\n{rows} families with at least one HOLE.\n")
    print("""## Source-read rows (NOT RUN: the version does not build with the harness; PM ruling 2026-11-28, only published versions count)

Method: the published `.crate` was downloaded and its source read: does it contain the function, and does the code between the safe signature and the FFI call lack a length check.

| crate | version | functions | evidence (published 0.1.0 source) | status |
|---|---|---|---|---|
| aocl-data-analytics | 0.1.0 | `linear_kernel`, `rbf_kernel`, `polynomial_kernel`, `sigmoid_kernel`, `pairwise_distances` | `src/lib.rs` 1725-1735: `linear_kernel` computes `ldd` and calls `T::linear_kernel(Layout::RowMajor, m, n, k, x, k, y, k, d, ldd)`; the `KernelScalar` impl (1572-1599) goes straight to `x.as_ptr()`, `y_ptr`, `d.as_mut_ptr()` in `sys::$linear(...)`; no `x.len()` / `d.len()` comparison anywhere (the same shape for the other four) | not run; HOLE by source read |
| aocl-sparse | 0.1.0 | `mv_f64` (and the `mv` / `trsv` / `symgs*` families of `complex.rs`/`lib.rs`) | `src/lib.rs`: `mv_f64(op, alpha, a, descr, x, beta, y)` calls `sys::aoclsparse_dmv(..., x.as_ptr(), &beta, y.as_mut_ptr())` directly; no `x.len()` / `y.len()` check | not run; HOLE by source read for the families measured as HOLE in 0.2.2 (the per-family rows above list which) |

The yank range per crate is first affected .. last published version before the first fixed one, not only the versions measured above.
""")
    print("""## aocl-crypto RSA and aocl-rng multivariate (measured per published version; families fixed in 0.4.0: measured 0 holes)

**aocl-crypto RSA** (`tests/guard_lengths.rs`, report mode, 24 short cases): HOLE in 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 for the ten families `encrypt` / `decrypt(Padding::None)` plaintext / `encrypt_oaep` ciphertext AND seed / `encrypt_pkcs1v15` ciphertext AND pad / `sign_pss` / `sign_pkcs1v15` / `sign_hash_pss` / `sign_hash_pkcs1v15` (10 holes each version). In 0.2.2 0.2.4 0.2.5 seven of the ten fault (`encrypt`, `encrypt_oaep` seed, `encrypt_pkcs1v15` pad, the four signing buffers); the other three could not be measured there (those versions return an error or panic in key set-up before the call, `OTHER(rc=101)`), so they are "not run" and counted by the yank range (first affected 0.2.2). 0.1.0 does not build with the harness: source read, `src/rsa.rs` 137-146 `encrypt` and 215-227 `sign_pss` call `ciphertext.as_mut_ptr()` / `signature.as_mut_ptr()` with no length check (no `len() <` anywhere in the file): HOLE by source read, not run. First affected 0.1.0, last published affected 0.2.54 (0.2.55 and 0.2.56 were never published: batch 4 published as 0.4.0), RSA family fixed in 0.4.0: MEASURED against the published 0.4.0 crate, 25 tests, 0 faults (`results aocl-crypto-0.4.0.csv`). Cipher (CBC/CTR/OFB/CFB/ChaCha20), AEAD (GCM/CCM) and EC: CLEAR (0 faults, 19 tests, 0.2.50 tree; `results-aocl-crypto-cipher-aead.csv`; older versions not re-run: the check code those paths use predates 0.2.2 per the source).

**aocl-rng** `multinormal` / `multi_students_t` / `multinomial` (`tests/guard_lengths.rs`, 10 short cases): HOLE (10 of 10) in 0.1.0 0.2.40 0.2.43 0.2.46 0.2.49. First affected 0.1.0, last published affected 0.2.54, multivariate family fixed in 0.4.0: MEASURED against the published 0.4.0 crate, 11 tests, 0 faults. `general_discrete` (any slice accepted): affected in every published version containing it up to 0.2.54, family fixed in 0.4.0 by construction (`DiscreteReference`; harness row `tests/typed_general_discrete.rs` run against the published 0.4.0 crate: 5 constructors sampled into a guard-page output with no fault, 3 invalid parameters refused; the unsafe raw entry remains by contract).
""")
    print("""## aocl-scalapack (f64 harness, single MPI rank; families fixed in 0.4.0: measured 0 holes)

Measured per published version (`tests/guard_lengths.rs`, 38 cases, one process each): **18 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54** (the slices `a` of gesv/gemm/potrf/getrf/getrs/potrs/syev/gels/geqrf, `b` of gesv/gemm/getrs/potrs/gels, `c` of gemm, `w` and `z` of syev, `tau` of geqrf); the 6 short `ipiv` / `work` cases and the `n` past the descriptor cases never fault (accepted or refused by the library; `pdgemm` with `n` past its descriptor aborts the process via PXERBLA). 0.1.0 does not build with the harness: source read, `src/lib.rs` 264-295 `pdgesv` passes `a.as_mut_ptr()`, `ipiv.as_mut_ptr()`, `b.as_mut_ptr()` and the descriptors to `sys::pdgesv_` with no `.len()` check (0 `.len()` in the function): HOLE by source read, not run, same shape as the other functions. First affected 0.1.0, last published affected 0.2.54, family fixed in 0.4.0: MEASURED against the published 0.4.0 crate: 38 tests, 0 faults (29 refused, 9 controls), and the crate's four MPI tests (`pdgesv_mpi`, `psgesv_mpi`, `complex_mpi`, `grid2d_mpi`) pass on 1, 2 and 4 MPI ranks (2 x 2 grid) from the published sources, so valid calls still work. `pdgemm` / `psgemm` now return `Result<()>`.

## aocl-compression bzip2 decode (family fixed in 0.5.0 by refusal: measured; `unsafe decompress_bzip2_trusted` remains)

Measured per published version (`tests/guard_lengths.rs`, corrupt-input sweep): **`decompress(Codec::Bzip2, ..)` HOLE in 0.2.2 0.2.4 0.2.5 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54** (3 of 218 single-bit flips of a valid 109-byte stream fault inside the decoder, no truncation does; a crafted bzip2 stream crashes the process, memory corruption is not excluded, exploitability unmeasured). LZ4, LZMA, Snappy, Zlib, Zstd decode and the native lz4/snappy short-dst paths: no fault in every measured version. 0.1.0 did not build with the harness: source read, `src/lib.rs` 96-112 `decompress` passes `src.as_ptr()` and `out.as_mut_ptr()` to `aocl_llc_decompress` for any `Codec` including `Bzip2` with no stream check: affected by source read, not run. First affected 0.1.0, last published affected 0.4.0 (0.4.0 re-run: the same HOLE; 9 earlier versions plus 0.4.0 are live), **family fixed in 0.5.0: MEASURED against the published 0.5.0 crate: the corrupt-stream sweep shows 0 faults because the safe `decompress(Codec::Bzip2, ..)` now returns an error for EVERY bzip2 stream, valid ones too (the documented functional regression: the round-trip control `roundtrip_control` fails at 0.5.0 for that reason alone); the crate's `tests/bzip2_upstream_defect.rs` passes against the published sources: the safe call refuses all three repro streams and the valid one, and a child process decoding each flip through the unsafe entry still dies with 0xC0000005, so the upstream decoder defect is still present in AOCL 6.0.68.** Yank range: 0.1.0 .. 0.4.0. The bzip2 encoder is not affected by what was measured.
""")
    print("""## aocl-data-analytics handle models (families fixed in 0.5.1: measured 0 holes)

Measured per published version (`tests/handle_models_lengths.rs`, 58 tests = 7 controls + 51 one-element-short cases over KMeans, Pca, KNearestNeighbours, Linmod, Dbscan, DecisionTree, DecisionForest, Svm; files `aocl-dam-<version>.csv`): **19 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54** (identical set in each): tree and forest `predict_proba` / `predict_log_proba` (`x_test` and output) and `score` (`x_test`); `KMeans::predict` `y`; `kneighbors` `x_test` and `n_dist`; kNN `predict_proba` `x_test` and `proba`; `Svm::score` `x_test` and `y_test`; `Svm::decision_function` `x_test` and output. Plus one heap overflow the guard page cannot see (`kneighbors` with `n_ind` shorter than `n_queries * k`). 0.1.0 did not build with the harness: source read, `src/lib.rs` 2281-2297 (`KNearestNeighbours::predict_proba`) and 3346 (`decision_function`) pass `x_test.as_ptr()` and `proba.as_mut_ptr()` straight to the library with no length check: affected by source read, not run. **0.5.0 (positive control, built from the published .crate): the same 19 HOLE and the same accepted `kneighbors` short-`n_ind` case (the heap write) as every earlier version. 0.5.1 (published .crate): 0 HOLE, all 51 short cases refused (including `kneighbors` short `n_ind`, now `InvalidArgument`), 7 controls pass; kernels and statistics at 0.5.0 and 0.5.1: 274 tests, 0 holes.** First affected 0.1.0, last published affected 0.5.0, family fixed in 0.5.1 (measured). Yank range: 0.1.0 .. 0.5.0 (first affected .. last published before the first fixed version).
""")
    print("""## aocl-data-analytics `Datastore::extract_f64` (fix PENDING #79; memory-corruption class: a write past the caller's buffer)

Measured per published version (`tests/datastore_lengths.rs`: 1 control + 4 cases; files `aocl-dads-<version>.csv`): **3 HOLE in every one of 0.2.2 0.2.4 0.2.5 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 0.2.54 0.4.0 0.5.0 0.5.1** (destination one element short for a column selection and for a row selection; `lddata` larger than the buffer allows); the fourth case (a key never selected) is refused by the library everywhere. **0.5.1, the only live data-analytics version, is affected.** 0.1.0 did not build with the harness: source read, `src/lib.rs` 1356-1372 passes `data.as_mut_ptr()` and `lddata` to `da_data_extract_selection_real_d` with no `data.len()` check: affected by source read, not run. First affected 0.1.0, last published affected 0.5.1, family fixed in PENDING #79. NOT MEASURED in the same crate: the `Nlls` callback closures' own slices (`Nlls::new`, `set_bounds`, `set_weights`, `fit` check lengths against `n_coef` / `n_res` by source read).
""")
    print("""## Cleared, no fix needed

- **aocl-securerng** (`tests/guard_lengths.rs`, 5 cases: exact-length `rdrand_bytes`, `rdseed_bytes`, `rdrand_u32_array`, `rdrand_u64_array` at a guard page): 0 faults, `n` counts elements. Source: every function passes `out.len()` as the count.
- **aocl-utils** (`tests/guard_lengths.rs`, 6 cases: `Cpu::Specific(1_000_000)` and `Specific(u32::MAX - 1)` into `has_flag`, `has_flags_all`, `has_flags_any`, `vendor_info`): 0 faults (the library tolerates an out-of-range CPU index). `has_flags_*` build NUL-terminated strings and pass the pointer count.
- **aocl-math** (`ldexp`, `ldexpf`, `scalbn`, `scalbnf`, `scalbln`, `scalblnf`): value in, value out, no pointer or slice argument: nothing to fault by source read.
- **aocl-compression** other codecs and **aocl-crypto** cipher / AEAD / EC: see the sections above.
""")
    print("""## Coverage of the crates described in sections above (MEASURED / NOT MEASURED)

"0 holes" in any row means 0 holes in the measured families only; it is not a statement that the crate is clean.

- **aocl-crypto**: MEASURED RSA (25 tests: 24 one-byte-short cases + control; encrypt, decrypt, OAEP, PSS, PKCS#1 v1.5, hash variants), cipher and AEAD constructors / buffers (19 tests: CBC, CTR, OFB, CFB, ChaCha20 iv / key / partial block; GCM iv / key / tag; CCM nonce). NOT MEASURED: digest, mac, EC (read by source only: EC validates before the FFI call), `set_aad`, `Rsa::set_public_key` / `set_private_key` beyond the length checks already present.
- **aocl-rng**: MEASURED `multinormal`, `multi_students_t`, `multinomial` (11 tests). NOT MEASURED: `BlumBlumShub::new`, the univariate generators (read by source: `n = out.len()`), the typed `general_discrete` path (type-enforced; harness row owed), `leapfrog` / `skip_ahead`.
- **aocl-scalapack**: MEASURED f64 `pdgesv pdgemm pdgetrf pdgetrs pdpotrf pdpotrs pdsyev pdgels pdgeqrf` on a single rank, 1 x 1 grid (38 tests); fixed-version multi-rank runs are valid-call checks only. NOT MEASURED: the f32 / complex copies, multi-rank guard cases, `Grid`, `BlockCyclic::new`.
- **aocl-compression**: MEASURED corrupt-input decode of all six codecs (truncation + bit flips) and the native lz4 / snappy short-destination paths (10 tests). NOT MEASURED: the bzip2 / lzma / others' `compress` with extreme levels, `lz4::compress_bound` (returns the source length, not the bound: a logic defect, not a memory one).
- **aocl-securerng**, **aocl-utils**, **aocl-math**: see "Cleared" below; **aocl-openrng**: NOT MEASURED (waiting on its PRs).
""")
    print("## Not measured (read pending)\n\nsparse `blkcsrmv` (no safe mask constructor), `create_csc` / `create_tcsr` (raw handle, no safe sink); lapack band `gb*`, `gesvdx`, `orgbr`, `lasrt`, and the trait-level `RealScalar::geev` / `ComplexScalar::geev` called directly (the typed free functions are measured); blas trait methods with explicit `inc`/`ld`, pack/compute/batch/rotm/sdsdot/csrot/zdrot; and the rows data-analytics handle models (kmeans/pca/knn/linear/rf, set_weights, extract_f64), crypto digest/mac (value+len, source only), aocl-openrng (after its PRs merge). `aocl-sparse` 0.1.0 did not compile with the harness (2 errors in its own lib against current dependencies) and is unmeasured; the functions are present in its source.\n")


if __name__ == "__main__":
    main()
