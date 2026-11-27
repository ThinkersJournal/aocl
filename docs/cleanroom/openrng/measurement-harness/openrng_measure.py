"""OpenRNG measurement driver (black box; OpenRNG's own tests and src/ are NOT used or read).

READS : C:/openrng-build/build-sha256.txt (expected hashes); the three files of the chosen build dir (openrng.dll, openrng.lib,
        openrng_static.lib: hashed only); C:/Program Files/AMD/AOCL-Windows/amd-libm/lib/libalm.dll (hashed + loaded);
        ./sys (copy of the crate's src/ + build.rs: OUR declarations from the licensed header) and ./harness (our tests).
LOADS : openrng.dll of the build in OPENRNG_LIB_DIR, libalm.dll, the Windows CRT/system DLLs. Nothing else. PATH is set per
        child process only (never machine-wide): [build lib dir; amd-libm lib dir; System32; Windows; the cargo/rustc dir; Git cmd].
WRITES: only under C:/openrng-measure: ./target-<width>/ (cargo output), ./harness/Cargo.lock, ./out/RAW-<width>-*.txt (the record),
        and a temporary harness/tests/link_all_control.rs (created and deleted by this script). Nothing under C:/openrng-build, no
        repo, no registry write (cargo is run --offline; there are no registry dependencies).
NETWORK: none (cargo --offline, CARGO_NET_OFFLINE=true). ADMIN: not needed. Every child process has a timeout.
RUNS  : cargo test (compiles our harness against openrng.lib, then runs the harness tests which call openrng.dll):
        link_all (60 functions' addresses), width_probe, kat_record, loaded_modules. Plus a negative control that must fail to link.
USAGE : set OPENRNG_LIB_DIR=C:\\openrng-build\\src\\build\\rel-llvm-lp64\\install\\lib     (then ...-ilp64\\install\\lib)
        python -I C:\\openrng-measure\\openrng_measure.py
EXIT  : 0 = everything ran and every loaded DLL hash equals build-sha256.txt; 2 = hash mismatch / wrong dir; 1 = a step failed.
"""
import datetime, hashlib, os, platform, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
BUILD_HASHES = r"C:\openrng-build\build-sha256.txt"
LIBM_DIR = r"C:\Program Files\AMD\AOCL-Windows\amd-libm\lib"
HARNESS = os.path.join(HERE, "harness")
OUT = os.path.join(HERE, "out")
TIMEOUT = 900

lib_dir = os.environ.get("OPENRNG_LIB_DIR", "")
norm = os.path.normcase(os.path.normpath(lib_dir)) if lib_dir else ""
m = re.search(r"rel-llvm-(lp64|ilp64)[\\]install[\\]lib$", norm)
PREFIX = os.path.normcase("C:" + chr(92) + "openrng-build" + chr(92) + "src" + chr(92) + "build") + os.sep
if not m or not norm.startswith(PREFIX):
    sys.exit("FAIL: OPENRNG_LIB_DIR must be C:/openrng-build/src/build/rel-llvm-<lp64|ilp64>/install/lib (got %r)" % lib_dir)
width = m.group(1)
lib_dir = os.path.normpath(lib_dir)

stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
os.makedirs(OUT, exist_ok=True)
raw_path = os.path.join(OUT, f"RAW-{width}-{stamp}.txt")
raw = open(raw_path, "w", encoding="utf8")


def say(s=""):
    print(s, flush=True)
    raw.write(s + "\n")
    raw.flush()


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def expected_hashes():
    d = {}
    for line in open(BUILD_HASHES, encoding="utf8"):
        p = line.split()
        if len(p) == 4 and p[0] == width:
            d[p[1]] = (p[2], int(p[3]))
    return d


cargo = None
for cand in os.environ.get("PATH", "").split(";") + [r"C:\rust\.cargo\bin"]:
    if cand and os.path.exists(os.path.join(cand, "cargo.exe")):
        cargo = os.path.join(cand, "cargo.exe")
        break
if not cargo:
    sys.exit("FAIL: cargo.exe not found on PATH")

env = dict(os.environ)
env["OPENRNG_LIB_DIR"] = lib_dir
env["PATH"] = ";".join([lib_dir, LIBM_DIR, r"C:\Windows\System32", r"C:\Windows", os.path.dirname(cargo), r"C:\Program Files\Git\cmd"])
env["CARGO_NET_OFFLINE"] = "true"
env["CARGO_BUILD_BUILD_DIR"] = os.path.join(HERE, f"target-{width}")
env["CARGO_TARGET_DIR"] = os.path.join(HERE, f"target-{width}")
feat = ["--features", "ilp64"] if width == "ilp64" else []


def run(cmd, cwd=HARNESS, timeout=TIMEOUT):
    say("$ " + " ".join(cmd) + f"    (cwd {cwd})")
    try:
        r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, errors="replace", timeout=timeout)
        out = (r.stdout or "") + (r.stderr or "")
        say(out.rstrip())
        say(f"[exit {r.returncode}]")
        return r.returncode, out
    except subprocess.TimeoutExpired as e:
        say(f"[TIMEOUT after {timeout}s]")
        return 124, ""


say(f"== OpenRNG measurement, build={width}, started {datetime.datetime.now().isoformat(timespec='seconds')}")
say(f"OPENRNG_LIB_DIR = {lib_dir}")
say(f"host: {platform.platform()} machine={platform.machine()} python={platform.python_version()}")
rc, v = run([cargo, "-V"])
rc, v = run([os.path.join(os.path.dirname(cargo), "rustc.exe"), "-vV"])
say("")

# 1. hashes of the build directory against the PM's record
say("-- hashes (build dir vs " + BUILD_HASHES + ")")
bad = False
exp = expected_hashes()
for name in ("openrng.dll", "openrng.lib", "openrng_static.lib"):
    p = os.path.join(lib_dir, name)
    got = sha(p)
    want = exp.get(name, ("MISSING", 0))[0]
    ok = got == want
    bad |= not ok
    say(f"{name:20} sha256={got} size={os.path.getsize(p)} expected={want} {'OK' if ok else 'MISMATCH'}")
libalm = os.path.join(LIBM_DIR, "libalm.dll")
say(f"{'libalm.dll':20} sha256={sha(libalm)} size={os.path.getsize(libalm)} (AOCL amd-libm install; no expected value recorded)")
if bad:
    say("FAIL: a hash differs from the build record: nothing was run.")
    sys.exit(2)

# 2. compile the harness against openrng.lib (no DLL code runs yet)
rc, _ = run([cargo, "test", "--offline", "--no-run"] + feat)
if rc != 0:
    say("FAIL: harness did not build"); sys.exit(1)

# 3. negative control: a made-up symbol must NOT link and must be named
ctl = os.path.join(HARNESS, "tests", "link_all_control.rs")
open(ctl, "w", encoding="utf8").write(
    'extern "C" { fn openrng_measure_control_does_not_exist(); }\n'
    "#[test]\nfn control() {\n    let p: unsafe extern \"C\" fn() = openrng_measure_control_does_not_exist;\n"
    "    assert_ne!(std::hint::black_box(p as usize), 0);\n}\n")
try:
    rc, out = run([cargo, "test", "--offline", "--no-run", "--test", "link_all_control"] + feat)
    neg_ok = rc != 0 and "openrng_measure_control_does_not_exist" in out
    say(f"NEGATIVE CONTROL: link failed and named the made-up symbol = {neg_ok}")
finally:
    os.remove(ctl)

# 4. run: link-all, width probe, known-answer record, loaded modules
rc, out = run([cargo, "test", "--offline", "--test", "link_all", "--test", "width_probe", "--test", "kat_record",
               "--test", "loaded_modules"] + feat + ["--", "--nocapture", "--test-threads=1"])
tests_ok = rc == 0
say(f"TESTS: exit {rc}")

# 5. hash the DLL files the process actually loaded
loaded = dict(re.findall(r"MODULE (\S+) = (.*?)\s*$", out, re.M))
say("-- loaded modules (GetModuleFileNameA) and their hashes")
for name, path in loaded.items():
    if os.path.exists(path):
        h = sha(path)
        want = exp.get(name, (None,))[0]
        note = ("OK = build record" if h == want else "MISMATCH vs build record") if want else "(no expected value: AOCL install)"
        if want and h != want:
            bad = True
        say(f"{name:14} {path} sha256={h} {note}")
    else:
        say(f"{name:14} {path}")
if "openrng.dll" not in loaded or loaded.get("openrng.dll") == "NOT LOADED":
    bad = True
    say("FAIL: openrng.dll was not reported as loaded")
if os.path.normcase(os.path.dirname(loaded.get("openrng.dll", ""))) != os.path.normcase(lib_dir):
    bad = True
    say("FAIL: openrng.dll was loaded from a different directory than OPENRNG_LIB_DIR")

say(f"== finished {datetime.datetime.now().isoformat(timespec='seconds')}; record: {raw_path}")
raw.close()
sys.exit(2 if bad else (0 if tests_ok else 1))
