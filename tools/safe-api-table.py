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
    ("aocl-blas", "gemmt"): "0.2.47 (merged, not yet published)",
    ("aocl-lapack", "generic"): "PENDING: #66 armed as 0.2.48 (group A: 21 generic trait methods)",
    ("aocl-lapack", "typed"): "PENDING: group B branch lane/lapack-typed-lengths (opens after #66)",
}
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
    if crate == "aocl-sparse":
        m = re.match(r"(real_f64|real_f32|cplx_c64|cplx_c32)::(\w+?)_(short|col|rowptr|no_|t_|controls?)", test)
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
    for crate in ("aocl-sparse", "aocl-blas", "aocl-lapack"):
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
            if crate == "aocl-sparse":
                fx = FIXED[(crate, "*")]
            elif crate == "aocl-blas":
                fx = FIXED[(crate, "gemmt")]
            else:
                fx = FIXED[(crate, "generic" if f in GENERIC else "typed")]
            print(f"| {f} | {hv[0]} | {hv[-1]} | {n} | {fx} |")
            rows += 1
        print(f"\n{rows} families with at least one HOLE.\n")
    print("## Not measured (read pending)\n\nsparse `bsrmv`, `blkcsrmv`, `csr2bsr(_nnz)`, `create_csc`, `create_tcsr`, `solve`; lapack band / `geev` / `gesvd` / `gesdd` / `gelsd` / `gelsy` / `syevd` / `syevr` / `gesvdx` / `ggev` / `gees` / `sygv` / `orm*` / `orgtr` / `orgbr` / `orglq` / `larfg` / `lasrt` / `gebrd`; blas trait methods with explicit `inc`/`ld`, pack/compute/batch/rotm/sdsdot/csrot/zdrot; and the crates data-analytics, crypto, fft, rng, scalapack, math, compression, securerng, utils, openrng. `aocl-sparse` 0.1.0 did not compile with the harness (2 errors in its own lib against current dependencies) and is unmeasured; the functions are present in its source.\n")


if __name__ == "__main__":
    main()
