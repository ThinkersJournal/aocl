# Changelog

## Unreleased (version: pm-assist to allocate; BREAKING for `aocl-sparse`, so 0.2.0 by the pre-1.0 rule)

- New `THIRD-PARTY-NOTICES.md`: origin and licence of everything derived from AMD or other third parties (the 11 generated bindings files and their source headers, the `lz4frame.h` guard, test data), what was verified, and 5 open licence questions for the owner (`amdlibm.h` carries an object-code-only AMD licence, `rng.h` has no licence grant, the AOCL for Windows EULA, `blis.h`/`cblas.h`/`lapack.h` carry no notice, FFTW is GPL). All 11 bindings files now start with a pointer comment to it.
- Version evidence (read on the test box 2026-11-22): the 11 component `ReleaseNotes.txt` files under `AOCL-Windows` that name a version all say 5.3.0 (blis, libflame, compression, crypto, da, fftz, libm, rng, scalapack, securerng, sparse; fftw and utils name none); the generated `AOCLSPARSE_VERSION_*` constants are 5.3.0; the installer's registry entry (`AMD AOCL Windows`) has DisplayVersion 5.3.514. Nothing read says 5.3.2, which earlier notes here used without a source, so every claim now says `5.3.x`.
- `aocl-sparse`: **breaking, and it did not compile before on AOCL 5.3.** `csr2bsr_f64` / `csr2bsr_f32` now take a `block_order: Order` argument after `descr`, because `aoclsparse_dcsr2bsr` / `scsr2bsr` in the AOCL 5.3.x headers have a `block_order` parameter (the call had 10 arguments, the header has 11: error E0061 against regenerated bindings). New test `csr2bsr_block_order_f64_and_f32` checks both orders and both precisions. The vendored docs.rs bindings for `aocl-sparse-sys` are AOCL-sparse 5.1.0 (their `AOCLSPARSE_VERSION_*` constants) and still have the 10-argument form, so docs.rs builds of `aocl-sparse` will not compile until they are regenerated (ROADMAP row 5).
- `aocl-scalapack`: `tests/pdgesv_mpi.rs` also checks `pdgemm` (`Trans::No` with alpha 2 / beta 0.5, and `Trans::T`) against a serial product on 1, 2 and 3 ranks (run by hand 2026-11-22); mutants `Trans::T`->`N` and `alpha*2` fail it on every rank count. Also `pdgetrf` + `pdgetrs` (`Trans::No` and `Trans::T`) and `pdpotrf` + `pdpotrs` (both triangles, the unused one filled with 999) on 1, 2 and 3 ranks; mutants `Uplo::Upper`->`L` and `pdgetrs` ignoring `trans` fail them. Also `pdsyev` (both triangles; ascending eigenvalues, trace and Frobenius invariants, and unit-norm, orthogonal eigenvectors with `A z = w z`) and `pdgels` (consistent 8x4 system); mutants eigen job flag `'V'`->`'N'` (first survived a residual-only check, because a zero Z satisfies `A z = w z`; the norm check catches it) and `pdgels` ignoring `trans` fail them. Still untested: `pdgeqrf`, the single-precision and complex routines.
- `aocl-compression-sys`: **did not build against AOCL 5.3.x** (bindgen: `lz4frame.h` not found; the Windows install ships `lz4.h` and `lz4hc.h` but no `lz4frame.h`). `wrapper/compression.h` now includes it only if `__has_include(<lz4frame.h>)`, so installs that ship it keep the `LZ4F_*` bindings and the safe crate (which uses none) builds on 5.3.x. `aocl-compression`: 13 tests pass; new tests check the format each `Codec` produces (zstd magic, zlib header, bzip2 `BZh`, snappy length preamble) and decode a zlib stream made by Python's `zlib`, because the round-trip tests used one mapping in both directions and could not see a wrong codec id; three mutants (Zlib->Zstd id, Snappy->LZ4 id, LZ4 source size minus one) fail them.
- `aocl`: new `tests/reexports.rs` (one test per feature, 13 with all features): each module must be the component it names. A mutant re-exporting `aocl_blas` as `math` fails to compile the test.
- Vendored docs.rs bindings regenerated from AOCL 5.3.x (bindgen 0.73.2, AMD headers; was bindgen 0.69.5 against older headers) for `aocl-sparse-sys`, `aocl-crypto-sys` and `aocl-data-analytics-sys`: required, because with the old files the three wrappers above no longer compile in `DOCS_RS=1` mode (docs.rs builds the wrapper crates against the vendored files; measured: 2, 9 and 29 errors before, 0 after). `DOCS_RS=1 cargo check` of the whole workspace passes (measured with `CARGO_BUILD_BUILD_DIR` overridden: the user cargo config redirects `build-dir`, so `CARGO_TARGET_DIR` alone reuses stale build scripts and reports a false pass). The other `*-sys` vendored files are not compared (ROADMAP row 5).
- `aocl-data-analytics`: **did not compile against AOCL 5.3.x (13 errors); now does, public API unchanged.** `KNearestNeighbours` now calls `da_nn_*` (AOCL 5.3 renamed k-NN: `da_handle_nn`, `da_nn_set_data_d` + `da_nn_set_labels_d` instead of `da_knn_set_training_data_d`, `da_nn_classifier_predict(_proba)_d` with `knn_search_mode`, `da_nn_kneighbors_d`, `da_nn_classes_d`); the new header says the training data is stored by pointer, not copied, so the struct now owns copies of it. `da_linmod_define_features_d`, `da_linmod_evaluate_model_d` (x2), `da_tree_set_training_data_d` and `da_forest_set_training_data_d` gained `ldx` / `categorical_features` (the wrapper passes `ldx = n_samples`, i.e. the column-major default it assumed before, and no categorical features); `da_covariance_matrix_*` gained `assume_centered` (wrapper passes 0, i.e. subtract the means as before). 24 tests pass with `KMP_DUPLICATE_LIB_OK=TRUE` (see ROADMAP row 3: two OpenMP runtimes); new test `covariance_matrix_subtracts_column_means`; mutants (assume_centered, linmod `ldx`, `da_nn_set_labels_d` count) fail the tests; a mutant switching predict to radius search survives, because it gives the same labels under default options.
- `aocl-crypto`: **did not compile against AOCL 5.3.x; now does** (public API unchanged). Constants `_alc_cipher_mode_*`, `_alc_digest_mode_*`, `_alc_mac_type_*`, `alc_rng_*_t_*` are now `alc_cipher_mode_*` etc.; `alcp_cipher_encrypt/decrypt` and `alcp_cipher_aead_encrypt/decrypt` take an `outlen` pointer; two `bool` arguments. **Behaviour change fixed:** in 5.3 `alcp_cipher_aead_get_tag` on decryption compares the buffer you pass with the computed tag and returns `ALC_ERROR_TAG_MISMATCH` (14), so `Aead::verify_and_decrypt` (which passed a zeroed buffer) rejected every valid GCM message (a defect in the rejecting direction: new test `aes_gcm_every_tag_bit_flip_rejected` checked that all 128 single-bit flips of a tag are still rejected with the plaintext zeroed, and the unmodified tag is accepted); it now passes the expected tag, treats 14 as the tag-mismatch error, and still compares afterwards for older versions. 22/22 tests pass; mutants (mismatch check disabled; SHA2-256 mapped to SHA2-384) fail them.
- Tests: `aocl-securerng` and `aocl-utils` tests strengthened (the old ones only called functions and discarded the result: support query vs instruction agreement, array fill to the end, `is_amd` vs the vendor code, `has_flag` truth); each has two seeded mutants that now fail them. Note: `VendorInfo::vendor_id` is a numeric EVendor code ("1" on the AMD test host), not a vendor name as its use suggests.
- Tests on real AOCL (no code change): `aocl-lapack`, `aocl-fft`, `aocl-math`, `aocl-rng`, `aocl-sparse` run on Windows with AOCL 5.3.x; see ROADMAP row 3 for counts and the seeded mutants.
- Correction to 0.1.10's note: no `homepage` field was added (new cargo flags `homepage` equal to `repository` as redundant); only `repository` changed.

## 0.1.10 - 2026-11-22

- `aocl-scalapack-sys`: **fix link failure on Windows.** AOCL 5.3.x's `scalapack.dll` exports the PBLAS gemm routines as `pdgemm` and `psgemm` (no trailing underscore; `dumpbin /exports`), but the crate declared `pdgemm_` and `psgemm_`, so any binary that linked the `pdgemm` / `psgemm` wrappers failed with LNK2019. Windows now links by the exported names via `link_name`; other platforms are unchanged. Checked: the other 27 declared symbols are all exported.
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

- Docs only: ROADMAP rows 4 and 5 record that `aocl-blas-sys` regenerates with bindgen 0.73.2 against AOCL 5.3.x headers, and that the vendored docs.rs bindings do not match that version (265 names only in the vendored file, 568 only in the regenerated one, 30 constant values differ). No code change.

## 0.1.5

- Dependencies to latest: `thiserror` 1 to 2, `bindgen` 0.69 to 0.73. No public API change; `libc` stays on 0.2 (crates.io has only 1.0.0-alpha).

## 0.1.4

- Add `deny.toml` and a `cargo deny` CI job (advisories, licences, bans, sources); no code change.

## 0.1.3

- Add `ROADMAP.md` (docs only; no code change). Earlier history: see git log (0.1.0 published 2026-04-27; 0.1.1 and 0.1.2 not yet published).
