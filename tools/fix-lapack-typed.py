#!/usr/bin/env python3
"""Insert the length / shape checks of the typed (d/s/z/c) free functions of aocl-lapack (group B of the safe-API audit). One spec per
function family, mirroring tools/gen-lapack-harness.py. Functions that return `Result` return `Err(InvalidArgument)`; the few that
return a plain value (`?lacpy` -> (), `?lange` -> real) PANIC with the same message (a documented contract violation, not UB, and not a
breaking signature change).
    python tools/fix-lapack-typed.py crates/aocl-lapack/src/lib.rs
"""
import re
import sys

NL = chr(10)


def M(name, a, rows, cols, ld, ln=None):
    return ("m", f'"{name}: {a}", Layout::RowMajor, {rows}, {cols}, {ld}, {ln or a}.len()')


def V(name, a, need):
    return ("v", f'"{name}: {a}", {a}.len(), {need}')


FAMILIES = {
    "trtrs": ("ds", lambda: [M("trtrs", "A", "n", "n", "lda", "a"), M("trtrs", "B", "n", "nrhs", "ldb", "b")]),
    "trtri": ("ds", lambda: [M("trtri", "A", "n", "n", "lda", "a")]),
    "potri": ("ds", lambda: [M("potri", "A", "n", "n", "lda", "a")]),
    "lacpy": ("dszc", lambda: [M("lacpy", "A", "m", "n", "lda", "a"), M("lacpy", "B", "m", "n", "ldb", "b")]),
    "lange": ("dszc", lambda: [M("lange", "A", "m", "n", "lda", "a")]),
    "sytrf": ("dszc", lambda: [M("sytrf", "A", "n", "n", "lda", "a"), V("sytrf", "ipiv", "n")]),
    "sytrs": ("dszc", lambda: [M("sytrs", "A", "n", "n", "lda", "a"), M("sytrs", "B", "n", "nrhs", "ldb", "b"), V("sytrs", "ipiv", "n")]),
    "sytri": ("dszc", lambda: [M("sytri", "A", "n", "n", "lda", "a"), V("sytri", "ipiv", "n")]),
    "hetrf": ("zc", lambda: [M("hetrf", "A", "n", "n", "lda", "a"), V("hetrf", "ipiv", "n")]),
    "hetrs": ("zc", lambda: [M("hetrs", "A", "n", "n", "lda", "a"), M("hetrs", "B", "n", "nrhs", "ldb", "b"), V("hetrs", "ipiv", "n")]),
    "hetri": ("zc", lambda: [M("hetri", "A", "n", "n", "lda", "a"), V("hetri", "ipiv", "n")]),
    "gecon": ("dszc", lambda: [M("gecon", "A", "n", "n", "lda", "a")]),
    "pocon": ("dszc", lambda: [M("pocon", "A", "n", "n", "lda", "a")]),
    "gehrd": ("dszc", lambda: [M("gehrd", "A", "n", "n", "lda", "a"), V("gehrd", "tau", "n.saturating_sub(1)")]),
    "geqp3": ("dszc", lambda: [M("geqp3", "A", "m", "n", "lda", "a"), V("geqp3", "jpvt", "n"), V("geqp3", "tau", "m.min(n)")]),
    "gelss": ("dszc", lambda: [M("gelss", "A", "m", "n", "lda", "a"), M("gelss", "B", "m.max(n)", "nrhs", "ldb", "b"), V("gelss", "s", "m.min(n)")]),
    "orgqr": ("ds", lambda: [M("orgqr", "A", "m", "n", "lda", "a"), V("orgqr", "tau", "k")]),
    "ungqr": ("zc", lambda: [M("ungqr", "A", "m", "n", "lda", "a"), V("ungqr", "tau", "k")]),
}


def main():
    p = sys.argv[1]
    b = open(p, "rb").read().decode()
    nl = "\r\n" if "\r\n" in b else NL
    s = b.replace("\r\n", NL)
    out, i, patched = [], 0, {}
    for fam, (prefixes, spec) in FAMILIES.items():
        pass
    names = {pre + fam: fam for fam, (prefixes, _) in FAMILIES.items() for pre in prefixes}
    for m in re.finditer(r"(?m)^pub fn (\w+)\(", s):
        fam = names.get(m.group(1))
        if not fam:
            continue
        k = s.index(")", s.index("(", m.start()))
        # find the `{` that opens the body: first `{` after the return type, at the end of a line
        open_brace = s.index("{" + NL, k)
        ret = s[k:open_brace]
        # parameters may contain nested parens: re-find the closing paren by balancing
        depth, j = 0, s.index("(", m.start())
        while True:
            c = s[j]
            depth += (c == "(") - (c == ")")
            if depth == 0:
                break
            j += 1
        open_brace = s.index("{" + NL, j)
        ret = s[j:open_brace]
        result_style = "Result<" in ret
        lines = []
        for kind, args in FAMILIES[fam][1]():
            if kind == "m":
                call = f"check_matrix({args})"
            else:
                call = f"check_vec({args})"
            lines.append(f"    {call}?;" if result_style else f'    if let Err(e) = {call} {{ panic!("{{e}}"); }}')
        if not result_style:
            # a plain-value function cannot return an error: document the panic just above the signature
            out.append(s[i:m.start()])
            out.append("/// # Panics" + NL + "///" + NL + "/// If a buffer is shorter than the shape and leading dimension require: LAPACKE does not check, and the call would read or write" + NL + "/// past the slice (this function returns a plain value, so it cannot report an error)." + NL)
            out.append(s[m.start():open_brace + 2])
        else:
            out.append(s[i:open_brace + 2])
        out.append(NL.join(lines) + NL)
        i = open_brace + 2
        patched[m.group(1)] = 1
    out.append(s[i:])
    open(p, "wb").write("".join(out).replace(NL, nl).encode())
    want = {pre + fam for fam, (prefixes, _) in FAMILIES.items() for pre in prefixes}
    missing = sorted(want - set(patched))
    print(f"patched {len(patched)} functions; not found: {missing}")


if __name__ == "__main__":
    main()
