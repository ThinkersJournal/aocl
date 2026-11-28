# `LAPACKE_?lacpy`: what happens to the unreferenced triangle of B (measured 2026-11-27)

**Predicate / ref.** AOCL 6.0.68 on Windows 11 (the install `tools/wrapper-tests.py` uses, LP64 DLLs), crate `aocl-lapack` at the
branch of #42; test `crates/aocl-lapack/tests/lacpy_measure.rs`
(`cargo test -p aocl-lapack --test lacpy_measure -- --nocapture`). Linux unmeasured. M = 3, N = 4, lda = ldb = 5, A holds
`10(i+1)+j+1` with garbage (-1) in its padding, B is pre-filled with 77. Each of d, s, c, z x {row-major, col-major} x
uplo {U, L, A} is its own recorded check; no variant stops the others.

## Result (all 24 variants, identical for d, s, c, z)

| layout | uplo | referenced region copied exactly | padding beyond the block untouched | entries OUTSIDE the referenced triangle changed |
|---|---|---|---|---|
| col-major | U, L, A | yes | yes | **0** |
| row-major | A (full) | yes | yes | n/a (whole block referenced) |
| row-major | U | yes | yes | **3**: (1,0) (2,0) (2,1), the strictly lower triangle |
| row-major | L | yes | yes | **6**: (0,1) (0,2) (0,3) (1,2) (1,3) (2,3), the strictly upper triangle |

So: column-major leaves the unreferenced triangle alone (the control); row-major with `U` or `L` overwrites it. The
copied triangle is always correct. The first report said "f64 only"; it is all four types.

## Is it AOCL-specific? Reference LAPACKE 3.12.1 has the same shape (read, not inferred)

`LAPACKE/src/lapacke_dlacpy_work.c` at the Reference-LAPACK release tag `v3.12.1` (commit 6ec7f2bc4ecf4c4a93496aa2fa519575bc0e39ca;
a public BSD-licensed source, read, not copied here): for `LAPACK_ROW_MAJOR` it mallocs `a_t` and `b_t`, transposes A into
`a_t` (`LAPACKE_dge_trans`), calls `LAPACK_dlacpy` into the **uninitialised** `b_t`, then transposes the **whole** `m x n`
block of `b_t` back over B. The triangle `dlacpy` never wrote is therefore garbage copied into B. Upstream fixed it afterwards:
the file's history on the default branch has "Fix LAPACKE_?lacpy_work row-major triangular copy (#729)" (commit 3a30b715,
2026-07-06), after which the row-major path calls `LAPACK_dlacpy` on the transposed view with swapped uplo and no temporary.

What is proved and what is not: **proved** that reference LAPACKE 3.12.1 has this behaviour by construction and that AOCL
6.0.68 exhibits it (measured). **Not proved:** that AOCL's LAPACKE is derived from that source (AOCL's own source was not
read), so "AOCL inherited a reference defect later fixed upstream" is the likely reading, not a verified one. This is not an
AMD-specific defect on the evidence; **no report to AMD or upstream is made** (standing ruling: not now).

## Wrapper contract (acted on)

`aocl_lapack::{dlacpy,slacpy,zlacpy,clacpy}` document that only the referenced region of `b` is defined and the unreferenced
triangle is unspecified for row-major. The wrapper's tests assert only the referenced region and the padding (not an
`#[ignore]` test).
