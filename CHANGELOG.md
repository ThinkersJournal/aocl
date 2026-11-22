# Changelog

## 0.1.10 - 2026-11-22

- `aocl-scalapack-sys`: **fix link failure on Windows.** AOCL 5.3.2's `scalapack.dll` exports the PBLAS gemm routines as `pdgemm` and `psgemm` (no trailing underscore; `dumpbin /exports`), but the crate declared `pdgemm_` and `psgemm_`, so any binary that linked the `pdgemm` / `psgemm` wrappers failed with LNK2019. Windows now links by the exported names via `link_name`; other platforms are unchanged. Checked: the other 27 declared symbols are all exported.
- `aocl-scalapack`: add `tests/pdgesv_mpi.rs`, a multi-rank test of `Grid`, `BlockCyclic` and `pdgesv` (asymmetric data; a transposed-deal mutant fails it on 2 ranks). Run by hand on 1, 2 and 3 ranks, 2026-11-21.
- CI: `selfhosted.yml` now picks only real x64 oneAPI files (not 0-byte symlinks, 32-bit or debug copies) and prints each resolved path; adds a `mpiexec -n 1/2/3` step. Dispatch-only; still not run on a runner.
- Repository URL is now `ThinkersJournal/aocl` (`repository`, `homepage`).

## 0.1.9 (CI only: self-hosted scalapack workflow; no code change; allocated by pm-assist 2026-11-21)

- CI: `.github/workflows/selfhosted.yml`, a self-hosted Windows job (`[self-hosted, Windows, X64, aocl]`) that runs the `aocl-scalapack` tests against installed AOCL and Intel MPI. Dispatch-only until the runner is registered; NOT run (no runner exists). No `pull_request` trigger. ROADMAP row 7 updated. No code change.

## 0.1.8 (docs + real numroc test; public API unchanged; allocated by pm-assist 2026-11-19)

- `aocl-scalapack`: replace the no-op link test and the empty `#[ignore]`d MPI sketch with a real MPI-free test of `numroc`; README and crate docs now say what is written and what is untested (the Windows DLL needs Intel MPI, not MS-MPI). Public API unchanged.

## 0.1.7

- Docs only: ROADMAP row 7 (`aocl-scalapack`) is scheduled for when an MPI consumer exists, per PM ruling 2026-11-01. No code change.

## 0.1.6

- Docs only: ROADMAP rows 4 and 5 record that `aocl-blas-sys` regenerates with bindgen 0.73.2 against AOCL 5.3.2 headers, and that the vendored docs.rs bindings do not match that version (265 names only in the vendored file, 568 only in the regenerated one, 30 constant values differ). No code change.

## 0.1.5

- Dependencies to latest: `thiserror` 1 to 2, `bindgen` 0.69 to 0.73. No public API change; `libc` stays on 0.2 (crates.io has only 1.0.0-alpha).

## 0.1.4

- Add `deny.toml` and a `cargo deny` CI job (advisories, licences, bans, sources); no code change.

## 0.1.3

- Add `ROADMAP.md` (docs only; no code change). Earlier history: see git log (0.1.0 published 2026-04-27; 0.1.1 and 0.1.2 not yet published).
