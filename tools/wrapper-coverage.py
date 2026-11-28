#!/usr/bin/env python3
"""Exposed vs tested: for every safe wrapper, the public functions/methods it exposes against the ones a test mentions.

Heuristic (stated, not hidden): a `pub fn NAME` is "tested" when the identifier NAME appears as a word in the test code of the
crate (the `#[cfg(test)]` module(s) of src/*.rs and every file under tests/). Generic names (`new`, `len`, `get`, ...) are
counted by presence, so they overstate coverage; the table prints the gaps (exposed names no test mentions) so they can be read.
A name tested is not proof that a test calls it with a checked result: `tools/wrapper-tests.py` runs the tests, and a wrong
expected value must fail (see docs/WRAPPER-TESTS-20261127.md). Usage: python tools/wrapper-coverage.py [--gaps] [crate ...]"""
import glob, os, re, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WRAPPERS = ["blas", "lapack", "fft", "utils", "compression", "sparse", "crypto", "data-analytics", "math", "rng", "securerng", "scalapack"]
SHOW = "--gaps" in sys.argv
crates = [a for a in sys.argv[1:] if not a.startswith("--")] or WRAPPERS


def split_tests(text):
    """-> (non-test text, test text): the #[cfg(test)] / #[cfg(all(test, ...))] mod body (brace matched) is the test part."""
    test, keep, i = [], [], 0
    pat = re.compile(r"#\[cfg\((?:all\(\s*)?test\b[^\]]*\]\s*(?:#\[[^\]]*\]\s*)*mod\s+\w+\s*\{")  # `cfg(test)` and `cfg(all(test, windows))`
    while True:
        m = pat.search(text, i)
        if not m:
            keep.append(text[i:])
            break
        keep.append(text[i:m.start()])
        depth, j = 1, m.end()
        while j < len(text) and depth:
            c = text[j]
            depth += (c == "{") - (c == "}")
            j += 1
        test.append(text[m.end():j])
        i = j
    return "".join(keep), "".join(test)


total_exposed = total_tested = 0
print(f"{'wrapper':22} {'#[test]':>8} {'exposed pub fns':>16} {'mentioned in tests':>19} {'gaps':>6}")
for c in crates:
    d = os.path.join(REPO, "crates", f"aocl-{c}")
    code, tests = "", ""
    n_tests = 0
    for f in glob.glob(os.path.join(d, "src", "**", "*.rs"), recursive=True):
        t = open(f, encoding="utf8", errors="replace").read()
        k, te = split_tests(t)
        code += k + "\n"
        tests += te + "\n"
    for f in glob.glob(os.path.join(d, "tests", "**", "*.rs"), recursive=True):
        tests += open(f, encoding="utf8", errors="replace").read() + "\n"
    n_tests = tests.count("#[test]")
    names = sorted(set(re.findall(r"\bpub\s+(?:unsafe\s+)?(?:const\s+)?fn\s+(\w+)", code)))
    tested = [n for n in names if re.search(r"\b" + re.escape(n) + r"\b", tests)]
    gaps = [n for n in names if n not in set(tested)]
    total_exposed += len(names)
    total_tested += len(tested)
    print(f"aocl-{c:17} {n_tests:8} {len(names):16} {len(tested):19} {len(gaps):6}")
    if SHOW and gaps:
        print("   gaps:", ", ".join(gaps))
print(f"{'TOTAL':22} {'':8} {total_exposed:16} {total_tested:19} {total_exposed - total_tested:6}")
