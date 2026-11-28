#!/usr/bin/env python3
"""Run a guard-page test binary one test per process and record the outcome per test (the "run" of the safe-API audit).

    python tools/safe-api-harness.py <package> <test-target> [--out docs/safe-api-audit/results-<package>.csv] [--timeout 60] [--features ...]

For every test of `cargo test -p <package> --test <test-target>` it runs `<exe> --exact <name> --nocapture` in a fresh process with a
timeout and classifies:
  HOLE      process died with an access violation (0xc0000005) or any non-zero exit on a *_short_* test that is not an assertion failure
  CONTROL-FAIL  a *_control test (the call with the right sizes) did not succeed: the rule of that row is wrong, fix the harness
  OK-refused    a short case returned Err before the FFI call (the wrapper validates)
  OK-accepted   a short case was accepted and nothing faulted (the library tolerated it or reads inside the page; NOT proof of safety
                for other sizes: see the audit doc)
  TIMEOUT       the process did not finish in --timeout seconds
The AOCL DLL directories (LP64 shared dirs + the oneAPI runtime) are put on PATH for the child only. Windows only (the harness uses
VirtualAlloc guard pages); Linux unmeasured. Output: one CSV row per test plus a summary on stdout.
"""
import csv
import glob
import json
import os
import re
import subprocess
import sys
import time

AOCL = r"C:\Program Files\AMD\AOCL-Windows"
ONEAPI = r"C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin"
LIBCLANG = r"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin"


def arg(name, default=None):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def child_env():
    env = dict(os.environ)
    dirs = sorted({os.path.dirname(p) for p in glob.glob(AOCL + r"\**\*.dll", recursive=True) if "ILP64" not in p})
    env["PATH"] = ";".join(dirs + [ONEAPI]) + ";" + env.get("PATH", "") + ";" + os.environ.get("OPENSSL_DIR", "C:/Program Files/Git/mingw64/bin")  # OpenSSL 3 libcrypto for aocl-crypto (AOCL does not ship it)
    env["PATH"] += ";" + ";".join([r"C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin", r"C:\Program Files (x86)\Intel\oneAPI\mpi\2021.18\bin", r"C:\Program Files (x86)\Intel\oneAPI\mpi\2021.18\opt\mpi\libfabric\bin"])  # Intel MPI runtime for aocl-scalapack (impi.dll)
    env.setdefault("LIBCLANG_PATH", LIBCLANG)
    return env


def main():
    pkg, target = sys.argv[1], sys.argv[2]
    timeout = int(arg("--timeout", "60"))
    out = arg("--out", f"docs/safe-api-audit/results-{pkg}.csv")
    feats = arg("--features")
    cwd = arg("--cwd")
    env = child_env()
    cmd = ["cargo", "test", "-p", pkg, "--test", target] + ([] if "--online" in sys.argv else ["--offline"]) + ["--no-run", "--message-format=json"] + (["--features", feats] if feats else [])
    r = subprocess.run(cmd, capture_output=True, text=True, env=env, cwd=cwd)
    exe = None
    for line in r.stdout.splitlines():
        try:
            j = json.loads(line)
        except ValueError:
            continue
        if j.get("executable") and j.get("target", {}).get("name") == target:
            exe = j["executable"]
    if not exe:
        sys.exit("build failed or no executable:\n" + r.stderr[-2000:])
    lst = subprocess.run([exe, "--list"], capture_output=True, text=True, env=env).stdout
    names = [l.rsplit(": test", 1)[0] for l in lst.splitlines() if l.endswith(": test")]
    rows, counts = [], {}
    t0 = time.time()
    for n in names:
        t1 = time.time()
        try:
            p = subprocess.run([exe, "--exact", n, "--nocapture"], capture_output=True, text=True, env=env, timeout=timeout, errors="replace")
            rc, o = p.returncode, p.stdout + p.stderr
        except subprocess.TimeoutExpired:
            rc, o = None, ""
        res = re.search(r"RESULT \S+ (refused|accepted)", o)
        if rc is None:
            k = "TIMEOUT"
        elif n.endswith("_control"):
            k = "OK-control" if rc == 0 else "CONTROL-FAIL"
        elif rc in (-1073741819, 3221225477):
            k = "HOLE"
        elif rc == 0 and res:
            k = "OK-" + res.group(1)
        elif rc == 0:
            k = "OK-refused(asserted)"  # an asserting test (regression file of a fix PR): passing means the Err was returned
        else:
            k = f"OTHER(rc={rc})"
        counts[k] = counts.get(k, 0) + 1
        rows.append({"test": n, "outcome": k, "exit_code": rc, "seconds": f"{time.time() - t1:.2f}"})
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    with open(out, "w", newline="", encoding="utf8") as fh:
        w = csv.DictWriter(fh, fieldnames=["test", "outcome", "exit_code", "seconds"])
        w.writeheader()
        w.writerows(rows)
    print(f"{pkg}/{target}: {len(names)} tests in {time.time() - t0:.0f}s; " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))
    print(f"wrote {out}")
    holes = [r["test"] for r in rows if r["outcome"] in ("HOLE", "TIMEOUT")]
    bad = [r["test"] for r in rows if r["outcome"] in ("CONTROL-FAIL",) or r["outcome"].startswith("OTHER")]
    if holes:
        print("HOLES/TIMEOUTS:", len(holes))
    if bad:
        print("HARNESS PROBLEMS (control failed / other):", len(bad), bad[:20])
    sys.exit(0)


if __name__ == "__main__":
    main()
