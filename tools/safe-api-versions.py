#!/usr/bin/env python3
"""MEASURE, per published version of a crate, which functions of the generated guard-page harness fault: the per-crate
first-affected / last-affected table of the safe-API audit (board 527).

    python tools/safe-api-versions.py <crate> <generator: blas|lapack> [--work DIR] [--versions 0.2.5,0.2.40] [--timeout 60]

For every version listed by crates.io (yanked ones too) it downloads the .crate, extracts it into `<work>/src/<crate>-<ver>/` (own
folder: the tree is untrusted data; this script lives elsewhere and Python runs with -I), generates the harness restricted to the
functions that version actually exports (the `pub fn` / `pub trait` names of its src, found with tools/safe-api-audit.py's parser),
builds it against the version's OWN dependencies (crates.io), runs every test in its own process (tools/safe-api-harness.py) and
writes `<work>/results/<crate>-<ver>.csv`. A function row is AFFECTED in a version when any of its short cases is a HOLE there.
Then prints the table: function | first affected | last affected | affected versions. Windows only; needs network for crates.io.
"""
import csv
import importlib.util
import io
import json
import os
import re
import subprocess
import sys
import tarfile
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))


def load(name, file):
    sp = importlib.util.spec_from_file_location(name, os.path.join(HERE, file))
    m = importlib.util.module_from_spec(sp)
    sp.loader.exec_module(m)
    return m


def get(url, binary=False):
    req = urllib.request.Request(url, headers={"User-Agent": "aocl-lane-audit"})
    d = urllib.request.urlopen(req, timeout=120).read()
    return d if binary else json.loads(d)


def arg(name, default=None):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def main():
    crate, gen = sys.argv[1], sys.argv[2]
    work = arg("--work", r"C:\Projects\.lane-state\versions-work")
    timeout = arg("--timeout", "60")
    audit = load("audit", "safe-api-audit.py")
    meta = get(f"https://crates.io/api/v1/crates/{crate}")
    versions = [v["num"] for v in sorted(meta["versions"], key=lambda v: v["created_at"])]
    if arg("--versions"):
        versions = arg("--versions").split(",")
    os.makedirs(os.path.join(work, "results"), exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(work, "target"), KMP_DUPLICATE_LIB_OK="TRUE")
    for ver in versions:
        out_csv = os.path.join(work, "results", f"{crate}-{ver}.csv")
        if os.path.exists(out_csv):
            continue
        d = os.path.join(work, "src", f"{crate}-{ver}")
        if not os.path.isdir(d):
            blob = get(f"https://static.crates.io/crates/{crate}/{crate}-{ver}.crate", True)
            os.makedirs(os.path.join(work, "src"), exist_ok=True)
            with tarfile.open(fileobj=io.BytesIO(blob)) as t:
                t.extractall(os.path.join(work, "src"), filter="data")
        src = ""
        for f in subprocess.run(["git", "ls-files"], capture_output=True, text=True, cwd=d).stdout.split() or []:
            pass
        names = set()
        for root, _, files in os.walk(os.path.join(d, "src")):
            for f in files:
                if f.endswith(".rs"):
                    for r in audit.parse(open(os.path.join(root, f), encoding="utf8", errors="replace").read(), f):
                        names.add(r["fn"].split("::")[-1])
        os.makedirs(os.path.join(d, "tests"), exist_ok=True)
        if gen in ("sparse", "fft", "crypto", "rng", "scalapack", "compression", "da-models", "da-ds", "lapack-rest"):
            targets = ["vector_lengths", "dense_lengths", "raw_array_lengths", "remaining_lengths"] if gen == "sparse" else ["handle_models_lengths"] if gen == "da-models" else ["datastore_lengths"] if gen == "da-ds" else ["rest_lengths"] if gen == "lapack-rest" else ["guard_lengths"]  # crypto / rng: hand-written guard_lengths.rs
            repo_tests = os.path.join(os.path.dirname(HERE), "crates", "aocl-sparse", "tests") if gen == "sparse" else {"sparse": None, "fft": os.environ.get("FFT_TESTS_DIR", "C:/Projects/aocl-wt-fft/crates/aocl-fft/tests"), "crypto": os.environ.get("CRYPTO_TESTS_DIR", "C:/Projects/aocl-wt-audit2/crates/aocl-crypto/tests"), "rng": os.environ.get("RNG_TESTS_DIR", "C:/Projects/aocl-wt-rng/crates/aocl-rng/tests"), "scalapack": os.environ.get("SCAL_TESTS_DIR", "C:/Projects/aocl-wt-audit2/crates/aocl-scalapack/tests"), "compression": "C:/Projects/aocl-wt-audit2/crates/aocl-compression/tests", "da-models": "C:/Projects/aocl-wt-audit2/crates/aocl-data-analytics/tests", "da-ds": "C:/Projects/aocl-wt-dam/crates/aocl-data-analytics/tests", "lapack-rest": "C:/Projects/aocl-wt-dam/crates/aocl-lapack/tests"}[gen]
            man = os.path.join(d, "Cargo.toml")
            t = open(man, encoding="utf8").read()
            for tg in targets:
                open(os.path.join(d, "tests", tg + ".rs"), "w", encoding="utf8").write(open(os.path.join(repo_tests, tg + ".rs"), encoding="utf8").read())
                if f'name = "{tg}"' not in t:
                    open(man, "a", encoding="utf8").write(chr(10) + "[[test]]" + chr(10) + f'name = "{tg}"' + chr(10) + f'path = "tests/{tg}.rs"' + chr(10) + ("harness = false" + chr(10) if gen == "scalapack" else ""))
            rows = []
            for tg in targets:
                oc = os.path.join(work, "results", f"{crate}-{ver}-{tg}.csv")
                r = subprocess.run([sys.executable, os.path.join(HERE, "safe-api-harness.py"), crate, tg, "--cwd", d, "--out", oc, "--timeout", timeout, "--online"],
                                   capture_output=True, text=True, env=env)
                print(ver, tg, (r.stdout.strip().splitlines() or ["(no output)"])[0], "|", (r.stderr.strip().splitlines() or [""])[-1][:160], flush=True)
                if os.path.exists(oc):
                    rows += list(csv.DictReader(open(oc, encoding="utf8")))
            with open(out_csv, "w", newline="", encoding="utf8") as fh:
                w = csv.DictWriter(fh, fieldnames=["test", "outcome", "exit_code", "seconds"])
                w.writeheader()
                w.writerows(rows)
            continue
        gens = {"blas": "gen-blas-harness.py", "lapack": "gen-lapack-harness.py", "da": "gen-da-harness.py"}[gen]
        code = subprocess.run([sys.executable, os.path.join(HERE, gens), "--fns", ",".join(sorted(names))], capture_output=True, text=True).stdout
        open(os.path.join(d, "tests", "generated_guard.rs"), "w", encoding="utf8").write(code)
        man = os.path.join(d, "Cargo.toml")
        t = open(man, encoding="utf8").read()
        if "generated_guard" not in t:  # published manifests set autotests = false
            open(man, "a", encoding="utf8").write(chr(10) + "[[test]]" + chr(10) + 'name = "generated_guard"' + chr(10) + 'path = "tests/generated_guard.rs"' + chr(10))
        r = subprocess.run([sys.executable, os.path.join(HERE, "safe-api-harness.py"), crate, "generated_guard", "--cwd", d, "--out", out_csv,
                            "--timeout", timeout, "--online"], capture_output=True, text=True, env=env)
        print(ver, (r.stdout.strip().splitlines() or ["(no output)"])[0], "|", (r.stderr.strip().splitlines() or [""])[-1][:200], flush=True)
    # table
    fam = {}
    for ver in versions:
        p = os.path.join(work, "results", f"{crate}-{ver}.csv")
        if not os.path.exists(p):
            continue
        for row in csv.DictReader(open(p, encoding="utf8")):
            if gen in ("crypto", "rng", "scalapack", "compression"):
                if "control" in row["test"]:
                    continue
                fam.setdefault(row["test"], {}).setdefault(ver, []).append(row["outcome"])
                continue
            if gen == "fft":
                m = re.match(r"(f64s|f32s)::(.+)", row["test"])
                if not m or "control" in row["test"]:
                    continue
                fam.setdefault(m.group(2), {}).setdefault(ver, []).append(row["outcome"])
                continue
            if gen == "sparse":
                m = re.match(r"(real_f64|real_f32|cplx_c64|cplx_c32)::(\w+?)_(short|col|rowptr|no_|t_|controls?)", row["test"])
                if not m or "control" in row["test"]:
                    continue
                fam.setdefault(m.group(2) + " (" + m.group(1) + ")", {}).setdefault(ver, []).append(row["outcome"])
                continue
            m = re.match(r"(.+?)_(f64|f32|c64|c32)_(.+)$", row["test"])
            if not m or "_control" in row["test"]:
                continue
            fam.setdefault(m.group(1), {}).setdefault(ver, []).append(row["outcome"])
    print(f"\n{crate}: function family | versions measured with that family | HOLE in versions")
    for f in sorted(fam):
        present = [v for v in versions if v in fam[f]]
        hole = [v for v in present if "HOLE" in fam[f][v]]
        print(f"{f:14} present {present[0] if present else '-'}..{present[-1] if present else '-'}  HOLE in {hole if hole else 'none'}")


if __name__ == "__main__":
    main()
