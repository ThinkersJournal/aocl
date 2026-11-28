"""Mechanical generator: include/openrng.h @ amd/openrng 6.0.0 (commit 5c05f2ee...) -> aocl-openrng-sys declarations + provenance.
No hand edits of the output. Input: the header as read through the GitHub API (sha256 ee3a48fc...)."""
import re, sys, hashlib, os, csv

HDR = sys.argv[1]
OUT = sys.argv[2]  # repo root of the worktree
raw = open(HDR, "rb").read()
SHA = hashlib.sha256(raw).hexdigest()
text = raw.decode("utf8")


def blank(m):
    return "\n" * m.group(0).count("\n")


code = re.sub(r"/\*.*?\*/", blank, text, flags=re.S)
code = re.sub(r"//[^\n]*", "", code)
lines = code.split("\n")

TAG = "6.0.0"
COMMIT = "5c05f2eeb6effc696b0df6ce8be1fcbe855b631b"

# ---- constants
consts = []
for i, l in enumerate(lines, 1):
    m = re.match(r"\s*#define\s+(\w+)\s+(-?)(0x[0-9a-fA-F]+|\d+)\s*$", l)
    if m:
        name, sign, num = m.groups()
        consts.append((name, sign + num, i))

# ---- function prototypes (join continuation lines until ';')
funcs = []
i = 0
while i < len(lines):
    l = lines[i]
    if re.match(r"(int|void) +\w+\(", l):
        start = i + 1
        buf = l
        while ";" not in buf:
            i += 1
            buf += " " + lines[i].strip()
        funcs.append((start, re.sub(r"\s+", " ", buf.strip())))
    i += 1

BASE = {
    "int": "core::ffi::c_int",
    "unsigned int": "core::ffi::c_uint",
    "float": "core::ffi::c_float",
    "double": "core::ffi::c_double",
    "long long int": "core::ffi::c_longlong",
    "int32_t": "i32",
    "openrng_int_t": "openrng_int_t",
    "openrng_uint_t": "openrng_uint_t",
    "openrng_uint64_t": "openrng_uint64_t",
    "char": "core::ffi::c_char",
    "VSLStreamStatePtr": "VSLStreamStatePtr",
    "VSLBRngProperties": "VSLBRngProperties",
    "dUpdateFuncPtr": "dUpdateFuncPtr",
    "sUpdateFuncPtr": "sUpdateFuncPtr",
    "iUpdateFuncPtr": "iUpdateFuncPtr",
}


def ctype(decl):
    """decl like 'const double *a' / 'double r[]' / 'const openrng_int_t n' -> (rust_type, name)"""
    d = decl.strip()
    is_const = d.startswith("const ")
    if is_const:
        d = d[6:]
    arr = d.endswith("[]")
    if arr:
        d = d[:-2]
    # 'VSLBRngProperties *const properties' -> const pointer (top-level const), treat as pointer
    m = re.match(r"(.+?)\s*(\*+)\s*(const\s+)?(\w+)$", d)
    if m:
        base, stars, _, name = m.groups()
        ptrs = len(stars)
    else:
        m = re.match(r"(.+?)\s+(\w+)$", d)
        base, name = m.groups()
        ptrs = 0
        stars = ""
    base = base.strip()
    if base.startswith("const "):
        base = base[6:]
        is_const = True
    rb = BASE[base]
    if arr:
        ptrs += 1
    # 'const VSLStreamStatePtr stream' is a const handle by value (pointer type itself const): by value
    t = rb
    if ptrs:
        # const applies to the pointee only when 'const' preceded the base type and there is a pointer
        for k in range(ptrs):
            t = ("*const " if (is_const and k == 0) else "*mut ") + t
    return t, name


def rust_name(n):
    return "r#" + n if n in ("ref", "type", "in", "mod", "fn") else n


decl_rows = []
out_f = []
for line_no, proto in funcs:
    m = re.match(r"(int|void) +(\w+)\((.*)\);$", proto)
    ret, name, args = m.groups()
    args = args.strip()
    params = [] if args in ("void", "") else [a.strip() for a in args.split(",")]
    ps = []
    for a in params:
        t, n = ctype(a)
        ps.append(f"{rust_name(n)}: {t}")
    r = "" if ret == "void" else " -> core::ffi::c_int"
    out_f.append(f"    // include/openrng.h:L{line_no}\n    pub fn {name}({', '.join(ps)}){r};")
    decl_rows.append((name, line_no, proto))

SPDX = """// SPDX-FileCopyrightText: Copyright 2024-2026 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-FileCopyrightText: Copyright (c) 2026 Advanced Micro Devices, Inc. (AMD fork, AOCL-OpenRNG 6.0.0)
// SPDX-License-Identifier: MIT OR Apache-2.0 WITH LLVM-exception
//
// GENERATED mechanically (no hand edits) from include/openrng.h of github.com/amd/openrng, tag %s, commit %s,
// header sha256 %s, as read through the GitHub API. Licence: the header is "MIT OR Apache-2.0 WITH LLVM-exception"
// (Arm Limited); see ../NOTICE and ../LICENSE-ARM-OPENRNG / ../LICENSE-AMD-OPENRNG. Per-declaration provenance:
// docs/cleanroom/openrng/provenance.csv at https://github.com/ThinkersJournal/aocl.
""" % (TAG, COMMIT, SHA)

crate = os.path.join(OUT, "crates", "aocl-openrng-sys")
os.makedirs(os.path.join(crate, "src"), exist_ok=True)

with open(os.path.join(crate, "src", "decls.rs"), "w", encoding="utf8", newline="\n") as f:
    f.write(SPDX + "\nuse crate::{\n    dUpdateFuncPtr, iUpdateFuncPtr, openrng_int_t, openrng_uint64_t, openrng_uint_t, sUpdateFuncPtr,\n    VSLBRngProperties, VSLStreamStatePtr,\n};\n\nextern \"C\" {\n" + "\n".join(out_f) + "\n}\n")

with open(os.path.join(crate, "src", "consts.rs"), "w", encoding="utf8", newline="\n") as f:
    f.write(SPDX + "\n")
    for name, val, ln in consts:
        f.write(f"/// include/openrng.h:L{ln}\npub const {name}: core::ffi::c_int = {val};\n")

os.makedirs(os.path.join(OUT, "docs", "cleanroom", "openrng"), exist_ok=True)
with open(os.path.join(OUT, "docs", "cleanroom", "openrng", "provenance.csv"), "w", newline="\n", encoding="utf8") as f:
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["kind", "name", "header_line", "tag", "commit", "header_sha256", "in_guide_ch10"])
    guide = set(l.split("\t")[0] for l in open(os.path.join(OUT, "docs", "cleanroom", "openrng-guide-names.txt"), encoding="utf8") if l.strip() and not l.startswith("#"))
    for name, ln, _ in decl_rows:
        w.writerow(["fn", name, ln, TAG, COMMIT, SHA, "yes" if name in guide else "no"])
    for name, val, ln in consts:
        w.writerow(["const", name, ln, TAG, COMMIT, SHA, "n/a"])
print("functions", len(decl_rows), "consts", len(consts), "sha", SHA)
print("guide names not in functions:", sorted(guide - set(n for n, _, _ in decl_rows)))
print("functions not in guide:", sorted(set(n for n, _, _ in decl_rows) - guide))
