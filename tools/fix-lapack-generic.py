#!/usr/bin/env python3
"""Insert the length / shape checks of the generic aocl-lapack trait methods (group A of the safe-API audit) into every impl of
`Scalar`, `RealScalar`, `ComplexScalar` in crates/aocl-lapack/src/lib.rs. One spec per method, mirroring the harness rules
(tools/gen-lapack-harness.py). Run once on a clean tree; it asserts every method was patched in every impl it appears in.
    python tools/fix-lapack-generic.py crates/aocl-lapack/src/lib.rs
"""
import re
import sys

NL = chr(10)
# method -> list of check expressions (Rust), using the trait parameter names
M = lambda name, a, rows, cols, ld, ln: f'check_matrix("{name}: {a}", layout, {rows}, {cols}, {ld}, {ln}.len())?;'
V = lambda name, a, need: f'check_vec("{name}: {a}", {a}.len(), {need})?;'
SPEC = {
    "gesv": lambda: [M("gesv", "A", "n", "n", "lda", "a"), M("gesv", "B", "n", "nrhs", "ldb", "b"), V("gesv", "ipiv", "n")],
    "getrf": lambda: [M("getrf", "A", "m", "n", "lda", "a"), V("getrf", "ipiv", "m.min(n)")],
    "getrs": lambda: [M("getrs", "A", "n", "n", "lda", "a"), M("getrs", "B", "n", "nrhs", "ldb", "b"), V("getrs", "ipiv", "n")],
    "getri": lambda: [M("getri", "A", "n", "n", "lda", "a"), V("getri", "ipiv", "n")],
    "posv": lambda: [M("posv", "A", "n", "n", "lda", "a"), M("posv", "B", "n", "nrhs", "ldb", "b")],
    "potrf": lambda: [M("potrf", "A", "n", "n", "lda", "a")],
    "potrs": lambda: [M("potrs", "A", "n", "n", "lda", "a"), M("potrs", "B", "n", "nrhs", "ldb", "b")],
    "potri": lambda: [M("potri", "A", "n", "n", "lda", "a")],
    "sysv": lambda: [M("sysv", "A", "n", "n", "lda", "a"), M("sysv", "B", "n", "nrhs", "ldb", "b"), V("sysv", "ipiv", "n")],
    "sytrf": lambda: [M("sytrf", "A", "n", "n", "lda", "a"), V("sytrf", "ipiv", "n")],
    "sytrs": lambda: [M("sytrs", "A", "n", "n", "lda", "a"), M("sytrs", "B", "n", "nrhs", "ldb", "b"), V("sytrs", "ipiv", "n")],
    "hesv": lambda: [M("hesv", "A", "n", "n", "lda", "a"), M("hesv", "B", "n", "nrhs", "ldb", "b"), V("hesv", "ipiv", "n")],
    "hetrf": lambda: [M("hetrf", "A", "n", "n", "lda", "a"), V("hetrf", "ipiv", "n")],
    "hetrs": lambda: [M("hetrs", "A", "n", "n", "lda", "a"), M("hetrs", "B", "n", "nrhs", "ldb", "b"), V("hetrs", "ipiv", "n")],
    "gtsv": lambda: [V("gtsv", "dl", "n.saturating_sub(1)"), V("gtsv", "d", "n"), V("gtsv", "du", "n.saturating_sub(1)"), M("gtsv", "B", "n", "nrhs", "ldb", "b")],
    "ptsv": lambda: [V("ptsv", "d", "n"), V("ptsv", "e", "n.saturating_sub(1)"), M("ptsv", "B", "n", "nrhs", "ldb", "b")],
    "gels": lambda: [M("gels", "A", "m", "n", "lda", "a"), M("gels", "B", "m.max(n)", "nrhs", "ldb", "b")],
    "geqrf": lambda: [M("geqrf", "A", "m", "n", "lda", "a"), V("geqrf", "tau", "m.min(n)")],
    "gelqf": lambda: [M("gelqf", "A", "m", "n", "lda", "a"), V("gelqf", "tau", "m.min(n)")],
    "syev": lambda: [M("syev", "A", "n", "n", "lda", "a"), V("syev", "w", "n")],
    "heev": lambda: [M("heev", "A", "n", "n", "lda", "a"), V("heev", "w", "n")],
}
HELPER = '''
/// Check that a vector argument has at least `need` elements: LAPACKE does not (a pivot, tau or eigenvalue buffer one element short is
/// read or written past its end; access violation on AOCL 6.0.68, `tests/generic_lengths.rs`).
fn check_vec(what: &str, len: usize, need: usize) -> Result<()> {
    if len < need {
        return Err(Error::InvalidArgument(format!("{what}: length {len} < {need}")));
    }
    Ok(())
}
'''


def main():
    p = sys.argv[1]
    b = open(p, "rb").read().decode()
    nl = "\r\n" if "\r\n" in b else NL
    s = b.replace("\r\n", NL)
    out, i, patched = [], 0, {}
    for m in re.finditer(r"(?m)^( +)fn (\w+)\(\n", s):
        name = m.group(2)
        if name not in SPEC:
            continue
        # a definition has `) -> Result<..> {` before the next `;` at depth 0; a trait declaration ends with `;`
        j = s.index(") -> Result<", m.end())
        k = s.index(NL, j)
        line_end = s[j:k]
        if not line_end.rstrip().endswith("{"):
            continue
        out.append(s[i:k + 1])
        ind = m.group(1) + "    "
        out.append("".join(ind + c + NL for c in SPEC[name]()))
        i = k + 1
        patched[name] = patched.get(name, 0) + 1
    out.append(s[i:])
    s = "".join(out)
    anchor = "fn check_matrix("
    k = s.index(anchor)
    s = s[:k] + HELPER.lstrip(NL) + NL + s[k:]
    open(p, "wb").write(s.replace(NL, nl).encode())
    for n in SPEC:
        print(f"{n:8} patched in {patched.get(n, 0)} impl(s)")
    missing = [n for n in SPEC if not patched.get(n)]
    if missing:
        sys.exit("NOT patched: " + ", ".join(missing))


if __name__ == "__main__":
    main()
