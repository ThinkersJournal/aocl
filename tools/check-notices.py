#!/usr/bin/env python3
"""Check (or regenerate) the NOTICE files of the six BSD/MIT -sys crates against installed AOCL headers.

Usage:  python tools/check-notices.py [--aocl DIR] [--ref REV] [--apply]
  default   exit 0 iff every header in the include closure of each crate's wrapper/*.h has its leading notice comment
            (whitespace-normalised) inside the NOTICE block that lists it, and no listed header is missing from the closure.
  --ref REV read the NOTICE files from git revision REV instead of the working tree (control: --ref 857ef5e reports
            DIFF for aocl-sparse-sys, e.g. aoclsparse.h 2020-2024 in the NOTICE vs 2020-2025 installed).
  --apply   rewrite the NOTICE blocks from the installed headers (grouped per distinct verbatim text; blocks whose
            title is not "Source header(s): ..." are kept verbatim).
AOCL default: C:/Program Files/AMD/AOCL-Windows. Headers are read with BOM stripped; sets are enumerated from the
wrapper include closure, not by walking the install."""
import re, os, sys, subprocess
args = sys.argv[1:]
def opt(name, default=None):
    return args[args.index(name) + 1] if name in args else default
AOCL = opt("--aocl", r"C:\Program Files\AMD\AOCL-Windows"); REF = opt("--ref"); APPLY = "--apply" in args
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# securerng-sys is clean-room since 2026-11-27 (no wrapper/, no header-derived bindings): not checked here.
COMP = {"sparse": "amd-sparse", "utils": "amd-utils", "crypto": "amd-crypto",
        "data-analytics": "amd-da", "compression": "amd-compression"}
WRAP = {"data-analytics": "data_analytics.h"}
def norm(s): return re.sub(r"\s+", " ", s).strip()
def read(p): return open(p, encoding="utf-8-sig", errors="replace").read().replace("\r\n", "\n")
def lead(path):
    t = read(path); b = os.path.basename(path)
    if b in ("LzmaDec.h", "LzmaEnc.h", "7zTypes.h"):       # two consecutive leading comments
        m = re.match(r"\s*((?:/\*.*?\*/\s*){2})", t, re.S); return m.group(1).rstrip() if m else None
    if b == "bzlib.h":                                       # the comment holding the bzip2 notice
        c = [x for x in re.findall(r"/\*.*?\*/", t[:4000], re.S) if "Julian" in x]; return c[0] if c else None
    m = re.match(r"\s*(/\*.*?\*/)", t, re.S); return m.group(1) if m else None
def roots(c):
    inc = os.path.join(AOCL, COMP[c], "include")
    return [inc] + [os.path.join(inc, d) for d in sorted(os.listdir(inc)) if os.path.isdir(os.path.join(inc, d))]
def closure(c):
    rs = roots(c); top = os.path.join(AOCL, COMP[c])
    todo = [os.path.join(REPO, "crates", f"aocl-{c}-sys", "wrapper", WRAP.get(c, c + ".h"))]; seen = {}
    while todo:
        f = todo.pop()
        for m in re.finditer(r'#\s*include\s*[<"]([^>"]+)[>"]', read(f)):
            for base in [os.path.dirname(f)] + rs:
                p = os.path.normpath(os.path.join(base, m.group(1)))
                if os.path.isfile(p):
                    if p not in seen and os.path.commonpath([p, top]) == top: seen[p] = 1; todo.append(p)
                    break
    inc = os.path.join(top, "include")
    return {os.path.relpath(p, inc).replace("\\", "/"): p for p in seen}
def notice_text(c):
    rel = f"crates/aocl-{c}-sys/NOTICE"
    if REF: return subprocess.check_output(["git", "-C", REPO, "show", f"{REF}:{rel}"]).decode("utf-8").replace("\r\n", "\n")
    return read(os.path.join(REPO, rel))
def m_(k, r): return k.lower() == r.lower() or r.lower().endswith("/" + k.lower()) or k.lower().endswith("/" + r.lower())
HEAD = re.compile(r"(={20,}\n.*?\n={20,}\n\n)", re.S)
bad = 0
for c in COMP:
    cl = closure(c); t = notice_text(c); parts = HEAD.split(t)
    print(f"=== aocl-{c}-sys ({len(cl)} headers in closure)")
    listed = {}
    for i in range(1, len(parts), 2):
        title = parts[i].split("\n")[1]
        if title.startswith("Source header(s):"):
            for n in title[17:].split(","): listed[n.strip()] = norm(parts[i + 1])
    nmatch = 0
    for rel, p in sorted(cl.items()):
        L = lead(p); k = next((k for k in listed if m_(k, rel)), None)
        if L is None: print(f"  NO-NOTICE-COMMENT {rel}"); continue
        if k is None: print(f"  NOT-LISTED {rel}"); bad += 1
        elif norm(L) in listed[k]: nmatch += 1
        else: print(f"  DIFF {rel}"); bad += 1
    for k in listed:
        if not any(m_(k, r) for r in cl): print(f"  LISTED-NOT-IN-CLOSURE {k}"); bad += 1
    print(f"  MATCH {nmatch}")
    if APPLY:
        out = [parts[0]]; assigned = set()
        for i in range(1, len(parts), 2):
            head, body = parts[i], parts[i + 1]; title = head.split("\n")[1]
            if not title.startswith("Source header(s):"): out.append(head + body); continue
            groups = []
            for n in [x.strip() for x in title[17:].split(",")]:
                r = next((r for r in cl if m_(n, r)), None)
                if r is None: continue
                assigned.add(r); L = lead(cl[r])
                for g in groups:
                    if g[1] == L: g[0].append(r); break
                else: groups.append([[r], L])
            out.append(groups)
        extra = []
        for r in sorted(cl):
            if r in assigned or not lead(cl[r]): continue
            L = lead(cl[r])
            for g in extra:
                if g[1] == L: g[0].append(r); break
            else: extra.append([[r], L])
        last = max((i for i, o in enumerate(out) if isinstance(o, list)), default=0); out.insert(last + 1, extra)
        eq = "=" * 100 + "\n"; s = ""
        for o in out:
            if isinstance(o, str): s += o
            else:
                for names, L in o: s += eq + "Source header(s): " + ", ".join(names) + "\n" + eq + "\n" + L + "\n\n"
        open(os.path.join(REPO, "crates", f"aocl-{c}-sys", "NOTICE"), "w", encoding="utf-8", newline="\n").write(s.rstrip("\n") + "\n")
print("problems:", bad)
sys.exit(1 if bad and not APPLY else 0)
