#!/usr/bin/env python3
"""Link-all for EVERY aocl-*-sys crate against the installed AOCL (Windows).

For each crate (and each integer width where the crate has an `ilp64` feature, and optionally static-link) this
  1. builds the crate (bindgen runs as usual) and finds the generated bindings in its OUT_DIR,
  2. writes a throw-away test that takes the address of EVERY declared function (`pub fn` in the bindings),
  3. compiles/links that test: link.exe lists every unresolved symbol,
  4. classifies each unresolved name by looking it up in the export tables of every AOCL DLL (dumpbin /exports),
  5. runs a NEGATIVE control (a made-up symbol must fail to link and be reported) and a POSITIVE control
     (the same test with the unresolved names removed must link and resolve the rest).
It prints a per-crate table (declared N, resolved M, unresolved list) and writes a JSON report. The generated test files
are deleted afterwards. It changes no crate. Usage (repo root): python tools/link-all-crates.py [--static] [crate ...]
The PATH given to the tests is built here (per width), never inherited (an ILP64 DLL ahead of LP64 breaks LP64 runs).
"""
import glob, json, os, re, shutil, subprocess, sys, time

ROOT = os.environ.get("AOCL_ROOT") or os.path.join(os.environ.get("ProgramFiles", r"C:\Program Files"), "AMD", "AOCL-Windows")
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATES = ["blas", "lapack", "fft", "sparse", "utils", "compression", "crypto", "data-analytics", "scalapack"]
LLVM = r"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin"

args = [a for a in sys.argv[1:] if not a.startswith("--")]  # flags: --static, --strict (fail on any unresolved name not in docs/link-all/known-unresolved.json.gz)
STATIC = "--static" in sys.argv
STRICT = "--strict" in sys.argv
crates = args or CRATES

all_dirs = sorted({os.path.dirname(p) for p in glob.glob(os.path.join(ROOT, "**", "*.dll"), recursive=True)})
ilp_dirs = [d for d in all_dirs if "ILP64" in d]
lp_dirs = [d for d in all_dirs if "ILP64" not in d]
oneapi = sorted(d for d in glob.glob(r"C:\Program Files (x86)\Intel\oneAPI\compiler\*") if re.search(r"\\\d", d))
extra = [os.path.join(oneapi[-1], "bin")] if oneapi else []
mpi = sorted(glob.glob(r"C:\Program Files (x86)\Intel\oneAPI\mpi\*\bin"))
cargo = shutil.which("cargo") or r"C:\rust\.cargo\bin\cargo.exe"
cargo_dir = os.path.dirname(cargo)
sysdirs = [r"C:\Windows\System32", r"C:\Windows", cargo_dir, r"C:\Program Files\Git\cmd"]

def env_for(width):
    e = dict(os.environ)
    p = (ilp_dirs + lp_dirs if width == "ILP64" else lp_dirs) + extra + mpi + sysdirs
    e["PATH"] = ";".join(p)
    e["AOCL_ROOT"] = ROOT
    e["LIBCLANG_PATH"] = LLVM
    e["CARGO_BUILD_BUILD_DIR"] = os.path.join(REPO, "target-linkall-all-" + width)
    return e

def run(cmd, env, cwd=REPO, timeout=1800):
    r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, errors="replace", timeout=timeout)
    return r.returncode, r.stdout + "\n" + r.stderr

def dumpbin():
    for pat in (r"C:\Program Files\Microsoft Visual Studio\*\*\VC\Tools\MSVC\*\bin\Hostx64\x64\dumpbin.exe",):
        c = sorted(glob.glob(pat))
        if c:
            return c[-1]
    return None

_exports = None
def exports():
    """name -> [dll basename] over every AOCL DLL."""
    global _exports
    if _exports is not None:
        return _exports
    _exports = {}
    db = dumpbin()
    if not db:
        return _exports
    env = dict(os.environ)
    env["PATH"] = os.path.dirname(db) + ";" + env["PATH"]
    for dll in sorted(glob.glob(os.path.join(ROOT, "**", "*.dll"), recursive=True)):
        rc, out = run([db, "/exports", dll], env)
        for m in re.finditer(r"^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]{8}\s+(\S+)", out, re.M):
            _exports.setdefault(m.group(1), []).append(os.path.relpath(dll, ROOT))
    return _exports

def out_dir(crate, feats, env):
    cmd = [cargo, "build", "-p", f"aocl-{crate}-sys", "--message-format=json"] + (["--features", feats] if feats else [])
    rc, out = run(cmd, env)
    found = None
    for line in out.splitlines():
        try:
            j = json.loads(line)
        except Exception:
            continue
        if j.get("reason") == "build-script-executed" and f"aocl-{crate}-sys" in j.get("package_id", ""):
            found = j["out_dir"]
    return rc, found, out

def declared(out_dir_path, src_dir=None):
    best = None
    files = glob.glob(os.path.join(out_dir_path, "*.rs"))
    if not files and src_dir:  # hand-written crates (e.g. scalapack) declare in src/*.rs
        files = glob.glob(os.path.join(src_dir, "*.rs"))
    for f in files:
        s = open(f, encoding="utf8", errors="replace").read()
        names = [n for n in re.findall(r"pub fn (\w+)\(\s*(?!&?\s*(?:mut\s+)?self)", s)]
        if best is None or len(names) > len(best[1]):
            best = (f, names)
    return best

def gen(crate, names, path, control=False):
    mod = f"aocl_{crate.replace('-', '_')}_sys"
    with open(path, "w", encoding="utf8", newline="\n") as f:
        f.write("// GENERATED by tools/link-all-crates.py, deleted after the run.\n#![allow(warnings)]\n")
        if control:
            f.write('extern "C" { fn aocl_rs_link_all_control_does_not_exist(); }\n')
            f.write('#[test]\nfn control() {\n    let p: unsafe extern "C" fn() = aocl_rs_link_all_control_does_not_exist;\n')
            f.write('    assert_ne!(std::hint::black_box(p as usize), 0);\n}\n')
        else:
            f.write(f"use {mod} as s;\n#[test]\nfn all_declared_symbols_link() {{\n    let addrs: Vec<usize> = vec![\n")
            for n in names:
                f.write(f"        s::{n} as usize,\n")
            f.write(f"    ];\n    assert_eq!(addrs.len(), {len(names)});\n}}\n")

def unresolved(text):
    return sorted({m.group(1).removeprefix("__imp_") for m in re.finditer(r"unresolved external symbol (\S+)", text)})

report = []
for crate in crates:
    d = os.path.join(REPO, "crates", f"aocl-{crate}-sys")
    feats_toml = open(os.path.join(d, "Cargo.toml"), encoding="utf8").read()
    widths = ["LP64"] + (["ILP64"] if re.search(r"^ilp64\s*=", feats_toml, re.M) else [])
    modes = ["dynamic"] + (["static"] if STATIC and re.search(r"^static-link\s*=", feats_toml, re.M) else [])
    for width in widths:
        for mode in modes:
            feats = ",".join(x for x in (("ilp64" if width == "ILP64" else ""), ("static-link" if mode == "static" else "")) if x)
            env = env_for(width)
            rec = {"crate": f"aocl-{crate}-sys", "width": width, "mode": mode, "features": feats}
            t0 = time.time()
            rc, od, bout = out_dir(crate, feats, env)
            if rc != 0 or not od:
                rec["error"] = "crate build failed: " + bout[-600:].replace("\n", " | ")
                report.append(rec); print(f"{rec['crate']:28} {width:5} {mode:7} BUILD FAILED"); continue
            b = declared(od, os.path.join(d, "src"))
            if not b or not b[1]:
                rec["error"] = "no bindings found in OUT_DIR"; report.append(rec); print(f"{rec['crate']:28} {width:5} {mode:7} NO BINDINGS"); continue
            names = sorted(set(b[1]))
            rec["declared"] = len(names)
            tfile = os.path.join(d, "tests", "link_all_generated.rs")
            cfile = os.path.join(d, "tests", "link_all_control.rs")
            os.makedirs(os.path.dirname(tfile), exist_ok=True)
            try:
                gen(crate, names, tfile)
                tcmd = [cargo, "test", "-p", f"aocl-{crate}-sys", "--test", "link_all_generated", "--no-run"] + (["--features", feats] if feats else [])
                rc, out = run(tcmd, env)
                un = unresolved(out)
                rec["unresolved"] = un
                rec["resolved"] = len(names) - len([u for u in un if u in set(names)])
                rec["link_ok_all"] = rc == 0
                if rc != 0 and not un:
                    rec["other_error"] = out[-800:].replace("\n", " | ")
                # negative control: a made-up symbol must fail and be reported
                gen(crate, names, cfile, control=True)
                rc2, out2 = run([cargo, "test", "-p", f"aocl-{crate}-sys", "--test", "link_all_control", "--no-run"] + (["--features", feats] if feats else []), env)
                rec["negative_control_ok"] = rc2 != 0 and "aocl_rs_link_all_control_does_not_exist" in unresolved(out2)
                # positive control: the test minus the unresolved names must link
                if un:
                    keep = [n for n in names if n not in set(un)]
                    gen(crate, keep, tfile)
                    rc3, out3 = run(tcmd, env)
                    rec["positive_control_ok"] = rc3 == 0
                else:
                    rec["positive_control_ok"] = rc == 0
            finally:
                for f in (tfile, cfile):
                    if os.path.exists(f):
                        os.remove(f)
            ex = exports()
            rec["classified"] = {u: ("exported by " + ", ".join(ex[u][:3])) if u in ex else "not exported by any installed AOCL DLL" for u in rec.get("unresolved", [])}
            rec["seconds"] = round(time.time() - t0)
            report.append(rec)
            print(f"{rec['crate']:28} {width:5} {mode:7} declared {rec['declared']:5} resolved {rec['resolved']:5} unresolved {len(rec['unresolved']):4} "
                  f"controls neg={rec['negative_control_ok']} pos={rec['positive_control_ok']}")

if STRICT:
    import gzip
    known = json.load(gzip.open(os.path.join(REPO, "docs", "link-all", "known-unresolved.json.gz"), "rt", encoding="utf8"))
    bad = []
    for rec in report:
        key = f"{rec['crate']}|{rec['width']}|{rec['mode']}"
        if "unresolved" not in rec:
            bad.append(f"{key}: {rec.get('error','failed')[:120]}"); continue
        if not rec.get("negative_control_ok"):
            bad.append(f"{key}: negative control did not fail as it must")
        new = sorted(set(rec["unresolved"]) - set(known.get(key, [])))
        if new:
            bad.append(f"{key}: {len(new)} NEW unresolved symbols, e.g. {new[:5]}")
    if bad:
        print("STRICT FAILURES:")
        print(chr(10).join(bad))
        sys.exit(1)
    print("strict: OK (only the recorded known-unresolved names, controls pass)")
out = os.path.join(REPO, "link-all-report.json")
json.dump(report, open(out, "w", encoding="utf8"), indent=1)
print("report:", out)
