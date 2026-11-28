"""Runs `cargo test -p aocl-openrng-sys` (the crate's OWN tests: link_all, stream known answers, width_match, width_probe, kat_record) against ONE built OpenRNG.

READS : the repo worktree C:/Projects/aocl-wt-orng (branch lane/openrng-skeleton: the crate under test; `git rev-parse HEAD` is printed), the build dir in OPENRNG_LIB_DIR
        (hashed against C:/openrng-build/build-sha256.txt BEFORE anything runs), AOCL amd-libm libalm.dll.
LOADS : openrng.dll of that build dir, libalm.dll, the CRT. PATH is set per child process only: [build lib dir; amd-libm lib; System32; Windows; cargo dir; Git cmd].
WRITES: only under C:/openrng-measure/target-crate-<width>/ (cargo output) and C:/openrng-measure/out/CRATE-TESTS-<width>-<stamp>.txt (the record). Nothing under C:/openrng-build or the repo.
NETWORK: none (cargo --offline). Timeouts on every child. No admin.
USAGE : set OPENRNG_LIB_DIR=C:\\openrng-build\\src\\build\\rel-llvm-lp64\\install\\lib   & python -I C:\\openrng-measure\\run_crate_tests.py
        set OPENRNG_LIB_DIR=C:\\openrng-build\\src\\build\\rel-llvm-ilp64\\install\\lib  & python -I C:\\openrng-measure\\run_crate_tests.py
EXPECT: exit 0 on both. The ILP64 run enables `--features ilp64` itself. A width mismatch (feature vs library) FAILS tests/width_match.rs: that is the guard working.
"""
import datetime, hashlib, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = r"C:\Projects\aocl-wt-orng"
HASHES = r"C:\openrng-build\build-sha256.txt"
LIBM = r"C:\Program Files\AMD\AOCL-Windows\amd-libm\lib"

lib_dir = os.environ.get("OPENRNG_LIB_DIR", "")
norm = os.path.normcase(os.path.normpath(lib_dir)) if lib_dir else ""
m = re.search(r"rel-llvm-(lp64|ilp64)[\\]install[\\]lib$", norm)
PREFIX = os.path.normcase("C:" + chr(92) + "openrng-build" + chr(92) + "src" + chr(92) + "build") + os.sep
if not m or not norm.startswith(PREFIX):
    sys.exit("FAIL: OPENRNG_LIB_DIR must be C:/openrng-build/src/build/rel-llvm-<lp64|ilp64>/install/lib")
width = m.group(1)
lib_dir = os.path.normpath(lib_dir)

cargo = None
for cand in os.environ.get("PATH", "").split(";") + [r"C:\rust\.cargo\bin"]:
    if cand and os.path.exists(os.path.join(cand, "cargo.exe")):
        cargo = os.path.join(cand, "cargo.exe")
        break
if not cargo:
    sys.exit("FAIL: cargo.exe not found")

stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
os.makedirs(os.path.join(HERE, "out"), exist_ok=True)
raw = open(os.path.join(HERE, "out", f"CRATE-TESTS-{width}-{stamp}.txt"), "w", encoding="utf8")


def say(s=""):
    print(s, flush=True)
    raw.write(s + "\n")
    raw.flush()


def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for c in iter(lambda: f.read(1 << 20), b""):
            h.update(c)
    return h.hexdigest()


env = dict(os.environ)
env["OPENRNG_LIB_DIR"] = lib_dir
env["PATH"] = ";".join([lib_dir, LIBM, r"C:\Windows\System32", r"C:\Windows", os.path.dirname(cargo), r"C:\Program Files\Git\cmd"])
env["CARGO_NET_OFFLINE"] = "true"
env["CARGO_TARGET_DIR"] = os.path.join(HERE, f"target-crate-{width}")
env["CARGO_BUILD_BUILD_DIR"] = env["CARGO_TARGET_DIR"]


def run(cmd, cwd, timeout=1800):
    say("$ " + " ".join(cmd) + f"    (cwd {cwd})")
    try:
        r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, errors="replace", timeout=timeout)
        out = (r.stdout or "") + (r.stderr or "")
        say(out.rstrip())
        say(f"[exit {r.returncode}]")
        return r.returncode
    except subprocess.TimeoutExpired:
        say(f"[TIMEOUT {timeout}s]")
        return 124


say(f"== crate tests, build={width}, {datetime.datetime.now().isoformat(timespec='seconds')}")
run(["git", "rev-parse", "HEAD"], REPO)
run(["git", "status", "--short", "--untracked-files=no"], REPO)
exp = {}
for line in open(HASHES, encoding="utf8"):
    p = line.split()
    if len(p) == 4 and p[0] == width:
        exp[p[1]] = p[2]
bad = False
for name in ("openrng.dll", "openrng.lib"):
    got = sha(os.path.join(lib_dir, name))
    ok = got == exp.get(name)
    bad |= not ok
    say(f"{name:14} sha256={got} {'OK = build record' if ok else 'MISMATCH'}")
if bad:
    say("FAIL: hash mismatch, nothing run")
    sys.exit(2)
feat = ["--features", "ilp64"] if width == "ilp64" else []
rc = run([cargo, "test", "--offline", "-p", "aocl-openrng-sys"] + feat + ["--", "--nocapture", "--test-threads=1"], REPO)
say(f"== finished; exit {rc}")
raw.close()
sys.exit(0 if rc == 0 else 1)
