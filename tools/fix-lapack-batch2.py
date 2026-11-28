#!/usr/bin/env python3
"""Group C of the aocl-lapack safe-API audit: insert length / shape checks into the remaining typed d/s/z/c free functions
(`?geev ?syevd ?heevd ?syevr ?heevr ?sygv ?hegv ?gebrd ?larfg ?orgtr ?ungtr ?orglq ?unglq ?ormqr ?unmqr ?ormlq ?unmlq ?gees ?ggev`)
and into the generic `Scalar` trait methods `gesvd gesdd gelsd gelsy`. One spec per family, mirroring the rules of
tools/gen-lapack-harness.py. A buffer that the library only touches when a flag is set (`compute_vl`, `jobz_v`, ...) is checked only
then, so valid calls that pass an empty buffer for an unused output stay valid.
    python tools/fix-lapack-batch2.py crates/aocl-lapack/src/lib.rs
"""
import re
import sys

NL = chr(10)


def M(name, a, rows, cols, ld, ln=None, cond=None):
    return ("m", f'"{name}: {a}", Layout::RowMajor, {rows}, {cols}, {ld}, {ln or a}.len()', cond)


def V(name, a, need, cond=None):
    return ("v", f'"{name}: {a}", {a}.len(), {need}', cond)


EVR_COLS = "if range == Range::Indices { (iu - il + 1).max(0) as usize } else { n }"
LARFG_X = "if n < 2 { 0 } else { (n - 2) * (incx.unsigned_abs() as usize) + 1 }"
FAMILIES = {
    # name: (prefixes, spec)
    "geev_r": ("ds", "geev", lambda: [M("geev", "A", "n", "n", "lda", "a"), V("geev", "wr", "n"), V("geev", "wi", "n"),
                                      M("geev", "VL", "n", "n", "ldvl", "vl", "compute_vl"), M("geev", "VR", "n", "n", "ldvr", "vr", "compute_vr")]),
    "geev_c": ("zc", "geev", lambda: [M("geev", "A", "n", "n", "lda", "a"), V("geev", "w", "n"),
                                      M("geev", "VL", "n", "n", "ldvl", "vl", "compute_vl"), M("geev", "VR", "n", "n", "ldvr", "vr", "compute_vr")]),
    "syevd": ("ds", "syevd", lambda: [M("syevd", "A", "n", "n", "lda", "a"), V("syevd", "w", "n")]),
    "heevd": ("zc", "heevd", lambda: [M("heevd", "A", "n", "n", "lda", "a"), V("heevd", "w", "n")]),
    "syevr": ("ds", "syevr", lambda: [M("syevr", "A", "n", "n", "lda", "a"), V("syevr", "w", "n"),
                                      M("syevr", "Z", "n", EVR_COLS, "ldz", "z", "jobz_v"), V("syevr", "isuppz", f"2 * ({EVR_COLS})", "jobz_v")]),
    "heevr": ("zc", "heevr", lambda: [M("heevr", "A", "n", "n", "lda", "a"), V("heevr", "w", "n"),
                                      M("heevr", "Z", "n", EVR_COLS, "ldz", "z", "jobz_v"), V("heevr", "isuppz", f"2 * ({EVR_COLS})", "jobz_v")]),
    "sygv": ("ds", "sygv", lambda: [M("sygv", "A", "n", "n", "lda", "a"), M("sygv", "B", "n", "n", "ldb", "b"), V("sygv", "w", "n")]),
    "hegv": ("zc", "hegv", lambda: [M("hegv", "A", "n", "n", "lda", "a"), M("hegv", "B", "n", "n", "ldb", "b"), V("hegv", "w", "n")]),
    "gebrd": ("dszc", "gebrd", lambda: [M("gebrd", "A", "m", "n", "lda", "a"), V("gebrd", "d", "m.min(n)"), V("gebrd", "e", "m.min(n).saturating_sub(1)"),
                                        V("gebrd", "tauq", "m.min(n)"), V("gebrd", "taup", "m.min(n)")]),
    "larfg": ("dszc", "larfg", lambda: [V("larfg", "x", LARFG_X)]),
    "orgtr": ("ds", "orgtr", lambda: [M("orgtr", "A", "n", "n", "lda", "a"), V("orgtr", "tau", "n.saturating_sub(1)")]),
    "ungtr": ("zc", "ungtr", lambda: [M("ungtr", "A", "n", "n", "lda", "a"), V("ungtr", "tau", "n.saturating_sub(1)")]),
    "orglq": ("ds", "orglq", lambda: [M("orglq", "A", "m", "n", "lda", "a"), V("orglq", "tau", "k")]),
    "unglq": ("zc", "unglq", lambda: [M("unglq", "A", "m", "n", "lda", "a"), V("unglq", "tau", "k")]),
    "ormqr": ("ds", "ormqr", lambda: [M("ormqr", "A", "if side_l { m } else { n }", "k", "lda", "a"), V("ormqr", "tau", "k"), M("ormqr", "C", "m", "n", "ldc", "c")]),
    "unmqr": ("zc", "unmqr", lambda: [M("unmqr", "A", "if side_l { m } else { n }", "k", "lda", "a"), V("unmqr", "tau", "k"), M("unmqr", "C", "m", "n", "ldc", "c")]),
    "ormlq": ("ds", "ormlq", lambda: [M("ormlq", "A", "k", "if side_l { m } else { n }", "lda", "a"), V("ormlq", "tau", "k"), M("ormlq", "C", "m", "n", "ldc", "c")]),
    "unmlq": ("zc", "unmlq", lambda: [M("unmlq", "A", "k", "if side_l { m } else { n }", "lda", "a"), V("unmlq", "tau", "k"), M("unmlq", "C", "m", "n", "ldc", "c")]),
    "gees": ("ds", "gees", lambda: [M("gees", "A", "n", "n", "lda", "a"), V("gees", "wr", "n"), V("gees", "wi", "n"), M("gees", "VS", "n", "n", "ldvs", "vs", "compute_vs")]),
    "ggev_r": ("ds", "ggev", lambda: [M("ggev", "A", "n", "n", "lda", "a"), M("ggev", "B", "n", "n", "ldb", "b"), V("ggev", "alphar", "n"), V("ggev", "alphai", "n"),
                                      V("ggev", "beta", "n"), M("ggev", "VL", "n", "n", "ldvl", "vl", "compute_vl"), M("ggev", "VR", "n", "n", "ldvr", "vr", "compute_vr")]),
    "ggev_c": ("zc", "ggev", lambda: [M("ggev", "A", "n", "n", "lda", "a"), M("ggev", "B", "n", "n", "ldb", "b"), V("ggev", "alpha", "n"), V("ggev", "beta", "n"),
                                      M("ggev", "VL", "n", "n", "ldvl", "vl", "compute_vl"), M("ggev", "VR", "n", "n", "ldvr", "vr", "compute_vr")]),
}
# generic trait methods (layout is a parameter): SvdJob::All needs the full factor, Singular the leading min(m, n), Overwrite / None nothing
JOBCOLS = lambda job, full, k: f"(match {job} {{ SvdJob::All => {full}, SvdJob::Singular => {k}, _ => 0 }})"
TRAIT_M = lambda name, a, rows, cols, ld, ln: f'check_matrix("{name}: {a}", layout, {rows}, {cols}, {ld}, {ln}.len())?;'
TRAIT_V = lambda name, a, need: f'check_vec("{name}: {a}", {a}.len(), {need})?;'
TRAIT = {
    "gesvd": lambda: [TRAIT_M("gesvd", "A", "m", "n", "lda", "a"), TRAIT_V("gesvd", "s", "m.min(n)"),
                      f'{{ let c = {JOBCOLS("jobu", "m", "m.min(n)")}; if c > 0 {{ check_matrix("gesvd: U", layout, m, c, ldu, u.len())?; }} }}',
                      f'{{ let r = {JOBCOLS("jobvt", "n", "m.min(n)")}; if r > 0 {{ check_matrix("gesvd: VT", layout, r, n, ldvt, vt.len())?; }} }}',
                      TRAIT_V("gesvd", "superb", "m.min(n).saturating_sub(1)")],
    "gesdd": lambda: [TRAIT_M("gesdd", "A", "m", "n", "lda", "a"), TRAIT_V("gesdd", "s", "m.min(n)"),
                      f'{{ let c = {JOBCOLS("jobz", "m", "m.min(n)")}; if c > 0 {{ check_matrix("gesdd: U", layout, m, c, ldu, u.len())?; }} }}',
                      f'{{ let r = {JOBCOLS("jobz", "n", "m.min(n)")}; if r > 0 {{ check_matrix("gesdd: VT", layout, r, n, ldvt, vt.len())?; }} }}'],
    "gelsd": lambda: [TRAIT_M("gelsd", "A", "m", "n", "lda", "a"), TRAIT_M("gelsd", "B", "m.max(n)", "nrhs", "ldb", "b"), TRAIT_V("gelsd", "s", "m.min(n)")],
    "gelsy": lambda: [TRAIT_M("gelsy", "A", "m", "n", "lda", "a"), TRAIT_M("gelsy", "B", "m.max(n)", "nrhs", "ldb", "b"), TRAIT_V("gelsy", "jpvt", "n")],
}


GROUPS = {
    "eigen": ({"geev_r", "geev_c", "syevd", "heevd", "syevr", "heevr", "sygv", "hegv", "gees", "ggev_r", "ggev_c"}, set()),
    "rest": ({"gebrd", "larfg", "orgtr", "ungtr", "orglq", "unglq", "ormqr", "unmqr", "ormlq", "unmlq"}, {"gesvd", "gesdd", "gelsd", "gelsy"}),
}


def main():
    p = sys.argv[1]
    group = sys.argv[sys.argv.index("--group") + 1] if "--group" in sys.argv else None
    fams, trait_names = GROUPS[group] if group else (set(FAMILIES), set(TRAIT))
    b = open(p, "rb").read().decode()
    nl = "\r\n" if "\r\n" in b else NL
    s = b.replace("\r\n", NL)
    names = {}
    for key, (prefixes, base, spec) in FAMILIES.items():
        if key not in fams:
            continue
        for pre in prefixes:
            names[pre + base] = key
    out, i, patched = [], 0, {}
    for m in re.finditer(r"(?m)^pub fn (\w+)\(", s):
        key = names.get(m.group(1))
        if not key:
            continue
        depth, j = 0, s.index("(", m.start())
        while True:
            c = s[j]
            depth += (c == "(") - (c == ")")
            if depth == 0:
                break
            j += 1
        open_brace = s.index("{" + NL, j)
        assert "Result<" in s[j:open_brace], m.group(1)
        lines = []
        for kind, args, cond in FAMILIES[key][2]():
            call = f"check_matrix({args})?;" if kind == "m" else f"check_vec({args})?;"
            lines.append(f"    if {cond} {{ {call} }}" if cond else f"    {call}")
        out.append(s[i:open_brace + 2])
        out.append(NL.join(lines) + NL)
        i = open_brace + 2
        patched[m.group(1)] = 1
    # trait methods
    out.append(s[i:])
    s = "".join(out)
    out, i, tp = [], 0, {}
    for m in re.finditer(r"(?m)^( +)fn (\w+)\(\n", s):
        name = m.group(2)
        if name not in TRAIT or name not in trait_names:
            continue
        jj = s.index(") -> Result<", m.end())
        k = s.index(NL, jj)
        if not s[jj:k].rstrip().endswith("{"):
            continue
        out.append(s[i:k + 1])
        out.append("".join(m.group(1) + "    " + c + NL for c in TRAIT[name]()))
        i = k + 1
        tp[name] = tp.get(name, 0) + 1
    out.append(s[i:])
    open(p, "wb").write("".join(out).replace(NL, nl).encode())
    want = {pre + base for k, (prefixes, base, _) in FAMILIES.items() if k in fams for pre in prefixes}
    print(f"typed patched {len(patched)} of {len(want)}; missing: {sorted(want - set(patched))}")
    print("trait methods patched:", tp, "missing:", sorted(trait_names - set(tp)))


if __name__ == "__main__":
    main()
