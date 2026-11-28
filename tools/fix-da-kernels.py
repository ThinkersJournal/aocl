#!/usr/bin/env python3
"""aocl-data-analytics: insert shape / length checks into the `KernelScalar` and `DistScalar` trait impls (`?_kernel`,
`pairwise_distances`): x is m x k (ldx), y (when given) n x k (ldy), d is m x n (ldd) with y, m x m without. The library does not check
(a buffer one element short is an access violation on AOCL 6.0.68, tests/kernels_lengths.rs).
    python tools/fix-da-kernels.py crates/aocl-data-analytics/src/lib.rs
"""
import re
import sys

NL = chr(10)
HELPER = '''
/// Validate a dense `rows x cols` matrix argument with leading dimension `ld` in `layout`: `ld` at least the minor extent and the buffer
/// at least `(major - 1) * ld + minor` elements. The library does not check (access violation one element short, AOCL 6.0.68).
fn check_dense(name: &str, what: &str, layout: Layout, rows: usize, cols: usize, ld: usize, len: usize) -> Result<()> {
    if rows == 0 || cols == 0 {
        return Ok(());
    }
    let (major, minor) = match layout {
        Layout::RowMajor => (rows, cols),
        Layout::ColMajor => (cols, rows),
    };
    if ld < minor {
        return Err(Error::InvalidArgument(format!("{name}: {what} has ld={ld}, the {rows}x{cols} matrix needs at least {minor}")));
    }
    let need = (major - 1) * ld + minor;
    if len < need {
        return Err(Error::InvalidArgument(format!("{name}: {what}.len()={len}, the {rows}x{cols} matrix with ld={ld} needs {need}")));
    }
    Ok(())
}
'''
NAMES = ["linear_kernel", "rbf_kernel", "polynomial_kernel", "sigmoid_kernel", "pairwise_distances"]


def lines(name):
    return [f'check_dense("{name}", "x", layout, m, k, ldx, x.len())?;',
            f'if let Some(yv) = y {{ check_dense("{name}", "y", layout, n, k, ldy, yv.len())?; }}',
            f'let d_cols = if y.is_some() {{ n }} else {{ m }};',
            f'check_dense("{name}", "d", layout, m, d_cols, ldd, d.len())?;']


def main():
    p = sys.argv[1]
    b = open(p, "rb").read().decode()
    nl = "\r\n" if "\r\n" in b else NL
    s = b.replace("\r\n", NL)
    out, i, patched = [], 0, {}
    for m in re.finditer(r"(?m)^( +)fn (\w+)\(\n", s):
        name = m.group(2)
        if name not in NAMES:
            continue
        j = s.index(") -> Result<", m.end())
        k = s.index(NL, j)
        if not s[j:k].rstrip().endswith("{"):
            continue
        out.append(s[i:k + 1])
        out.append("".join(m.group(1) + "    " + c + NL for c in lines(name)))
        i = k + 1
        patched[name] = patched.get(name, 0) + 1
    out.append(s[i:])
    s = "".join(out)
    anchor = "fn layout_raw("
    k = s.index(anchor)
    s = s[:k] + HELPER.lstrip(NL) + NL + s[k:]
    open(p, "wb").write(s.replace(NL, nl).encode())
    print("patched:", patched)
    missing = [n for n in NAMES if n not in patched]
    if missing:
        sys.exit("NOT patched: " + ", ".join(missing))


if __name__ == "__main__":
    main()
