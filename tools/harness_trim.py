"""Shared by tools/gen-blas-harness.py and tools/gen-lapack-harness.py: drop the helpers a generated test file does not use, so the file
needs no crate-wide `dead_code` / `unused_imports` / `clippy::all` allow (only `unused_parens`: the generated size expressions are
parenthesised)."""
import re

NL = chr(10)


def trim(text):
    head, sep, body = text.partition("#[test]")
    body = sep + body
    if "report(" not in body:
        head = re.sub(r"fn report<.*?" + NL + "}" + NL, "", head, flags=re.S)
    for c in ("M", "N", "K", "R"):
        if not re.search(r"(?<![A-Za-z0-9_])" + c + r"(?![A-Za-z0-9_])", body):
            head = re.sub(r"const " + c + r": usize = \d+;" + NL, "", head)
    for ty in ("Complex32", "Complex64"):
        if ty not in body:
            head = head.replace(ty + ", ", "").replace(", " + ty, "")
    head = re.sub(r"use aocl_types::\{(Complex32|Complex64)?\};" + NL, "", head)
    head = re.sub(r"#!\[allow\(clippy::all, unused_imports, unused_parens, dead_code, reason = \"[^\"]*\"\)\]",
                  '#![allow(unused_parens, reason = "generated size expressions are parenthesised")]', head)
    return head + body
