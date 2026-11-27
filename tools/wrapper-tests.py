#!/usr/bin/env python3
"""Run the SAFE WRAPPERS' tests on the real installed AOCL (Windows; the self-hosted runner's job) and print a table:
per crate: tests declared (#[test] count), passed / failed / ignored from `cargo test`, and the verdict.
The PATH is built here per width (never inherited: an ILP64 DLL ahead of LP64 breaks LP64 runs). Usage (repo root):
    python tools/wrapper-tests.py [--ilp64] [crate ...]      # exit 1 if any wrapper test fails or none ran
A wrapper test "calls into the DLL" when it runs and passes here (the wrappers have no mock layer: the -sys crate links the
installed library); the positive control (a deliberately wrong expected value must FAIL) is `tools/wrapper-control.py`."""
import glob, os, re, shutil, subprocess, sys

ROOT = os.environ.get("AOCL_ROOT") or os.path.join(os.environ.get("ProgramFiles", r"C:\Program Files"), "AMD", "AOCL-Windows")
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WRAPPERS = ["blas", "lapack", "fft", "utils", "compression", "sparse", "crypto", "data-analytics", "math", "rng", "securerng", "scalapack"]
LLVM = r"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin"

ILP64 = "--ilp64" in sys.argv
NO_AOCL = "--control-no-aocl" in sys.argv  # CONTROL: PATH without any AOCL directory: every wrapper test run must FAIL (they really call into the DLLs)
EXTRA_PATH = [a.split("=", 1)[1] for a in sys.argv if a.startswith("--path=")]   # dirs appended LAST to PATH (e.g. an OpenSSL 3 libcrypto-3-x64.dll for aocl-crypto)
EXTRA_ENV = dict(a[6:].split("=", 1) for a in sys.argv if a.startswith("--env="))  # e.g. --env=KMP_DUPLICATE_LIB_OK=TRUE (aocl-data-analytics, see AMD issue 2)
crates = [a for a in sys.argv[1:] if not a.startswith("--")] or WRAPPERS
dirs = sorted({os.path.dirname(p) for p in glob.glob(os.path.join(ROOT, "**", "*.dll"), recursive=True)})
ilp = [d for d in dirs if "ILP64" in d]
lp = [d for d in dirs if "ILP64" not in d]
oneapi = sorted(d for d in glob.glob(r"C:\Program Files (x86)\Intel\oneAPI\compiler\*") if re.search(r"\\\d", d))
mpi = sorted(glob.glob(r"C:\Program Files (x86)\Intel\oneAPI\mpi\*\bin"))
cargo = shutil.which("cargo") or r"C:\rust\.cargo\bin\cargo.exe"
env = dict(os.environ)
env["PATH"] = ";".join(([] if NO_AOCL else (ilp + lp if ILP64 else lp)) + ([os.path.join(oneapi[-1], "bin")] if oneapi else []) + mpi + [LLVM, r"C:\Windows\System32", r"C:\Windows", os.path.dirname(cargo), r"C:\Program Files\Git\cmd"] + EXTRA_PATH)
env.update(EXTRA_ENV)
env["AOCL_ROOT"] = ROOT
env["LIBCLANG_PATH"] = LLVM
env["CARGO_BUILD_BUILD_DIR"] = os.path.join(REPO, "target-wrapper-tests-" + ("ILP64" if ILP64 else "LP64"))

bad = 0
print(f"{'crate':22} {'#[test]':>8} {'passed':>7} {'failed':>7} {'ignored':>8}  verdict")
for c in crates:
    name = f"aocl-{c}"
    d = os.path.join(REPO, "crates", name)
    n_tests = sum(open(f, encoding="utf8", errors="replace").read().count("#[test]") for f in glob.glob(os.path.join(d, "**", "*.rs"), recursive=True))
    cmd = [cargo, "test", "-p", name] + (["--features", "ilp64"] if ILP64 and re.search(r"^ilp64\s*=", open(os.path.join(d, "Cargo.toml"), encoding="utf8").read(), re.M) else [])
    r = subprocess.run(cmd, cwd=REPO, env=env, capture_output=True, text=True, errors="replace", timeout=3600)
    out = r.stdout + r.stderr
    p = f = i = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", out):
        p += int(m.group(1)); f += int(m.group(2)); i += int(m.group(3))
    if NO_AOCL:
        verdict = "CONTROL OK (fails without the AOCL DLLs)" if r.returncode != 0 or p == 0 else "CONTROL BROKEN (passed without the DLLs: the tests do not call into them)"
        bad += verdict.startswith("CONTROL BROKEN")
        print(f"{name:22} {n_tests:8} {p:7} {f:7} {i:8}  {verdict}")
        continue
    verdict = "OK" if r.returncode == 0 and p > 0 and f == 0 else ("NO TESTS RAN" if r.returncode == 0 and p == 0 else "FAILED/ERROR")
    if verdict != "OK":
        bad += 1
        tail = [l for l in out.splitlines() if re.search(r"error|FAILED|panicked|STATUS_|failed", l)][:3]
        verdict += "  " + " | ".join(t.strip()[:120] for t in tail)
    print(f"{name:22} {n_tests:8} {p:7} {f:7} {i:8}  {verdict}")
sys.exit(1 if bad else 0)
