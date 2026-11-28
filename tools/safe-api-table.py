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
        print(f"## {crate}\n\nVersions measured: {', '.join(vers)}.\n")
        print("| function family | first affected | last affected (published so far) | holes (short cases) at last affected | fixed in |\n|---|---|---|---|---|")
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
    print("""## aocl-crypto RSA and aocl-rng multivariate (measured per published version; fixes PENDING #73 crypto RSA, #74 rng multivariate)

**aocl-crypto RSA** (`tests/guard_lengths.rs`, report mode, 24 short cases): HOLE in 0.2.28 0.2.40 0.2.43 0.2.46 0.2.49 for the ten families `encrypt` / `decrypt(Padding::None)` plaintext / `encrypt_oaep` ciphertext AND seed / `encrypt_pkcs1v15` ciphertext AND pad / `sign_pss` / `sign_pkcs1v15` / `sign_hash_pss` / `sign_hash_pkcs1v15` (10 holes each version). In 0.2.2 0.2.4 0.2.5 seven of the ten fault (`encrypt`, `encrypt_oaep` seed, `encrypt_pkcs1v15` pad, the four signing buffers); the other three could not be measured there (those versions return an error or panic in key set-up before the call, `OTHER(rc=101)`), so they are "not run" and counted by the yank range (first affected 0.2.2). 0.1.0 does not build with the harness: source read, `src/rsa.rs` 137-146 `encrypt` and 215-227 `sign_pss` call `ciphertext.as_mut_ptr()` / `signature.as_mut_ptr()` with no length check (no `len() <` anywhere in the file): HOLE by source read, not run. First affected 0.1.0, last published affected 0.2.49, fixed in PENDING #73. Cipher (CBC/CTR/OFB/CFB/ChaCha20), AEAD (GCM/CCM) and EC: CLEAR (0 faults, 19 tests, 0.2.50 tree; `results-aocl-crypto-cipher-aead.csv`; older versions not re-run: the check code those paths use predates 0.2.2 per the source).

**aocl-rng** `multinormal` / `multi_students_t` / `multinomial` (`tests/guard_lengths.rs`, 10 short cases): HOLE (10 of 10) in 0.1.0 0.2.40 0.2.43 0.2.46 0.2.49. First affected 0.1.0, last published affected 0.2.49, fixed in PENDING #74. `general_discrete` (any slice accepted): affected in every published version containing it, fixed in PENDING #75 (typed by construction; not a guard-page row: the hole is a library read driven by an encoded length).
""")
    print("## Not measured (read pending)\n\nsparse `blkcsrmv` (no safe mask constructor), `create_csc` / `create_tcsr` (raw handle, no safe sink); lapack band `gb*`, `gesvdx`, `orgbr`, `lasrt`, and the trait-level `RealScalar::geev` / `ComplexScalar::geev` called directly (the typed free functions are measured); blas trait methods with explicit `inc`/`ld`, pack/compute/batch/rotm/sdsdot/csrot/zdrot; and the crates data-analytics, crypto, fft, rng, scalapack, math, compression, securerng, utils, openrng. `aocl-sparse` 0.1.0 did not compile with the harness (2 errors in its own lib against current dependencies) and is unmeasured; the functions are present in its source.\n")


if __name__ == "__main__":
    main()
