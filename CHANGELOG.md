# Changelog

## Unreleased (version: pm-assist to allocate; BREAKING for `aocl-sparse`, so 0.2.0 by the pre-1.0 rule)

- `aocl-sparse`: **breaking, and it did not compile before on AOCL 5.3.** `csr2bsr_f64` / `csr2bsr_f32` now take a `block_order: Order` argument after `descr`, because `aoclsparse_dcsr2bsr` / `scsr2bsr` in the AOCL 5.3.2 headers have a `block_order` parameter (the call had 10 arguments, the header has 11: error E0061 against regenerated bindings). New test `csr2bsr_block_order_f64_and_f32` checks both orders and both precisions. The vendored docs.rs bindings for `aocl-sparse-sys` are AOCL-sparse 5.1.0 (their `AOCLSPARSE_VERSION_*` constants) and still have the 10-argument form, so docs.rs builds of `aocl-sparse` will not compile until they are regenerated (ROADMAP row 5).
- Vendored docs.rs bindings regenerated from AOCL 5.3.2 (bindgen 0.73.2, AMD headers; was bindgen 0.69.5 against older headers) for `aocl-sparse-sys`, `aocl-crypto-sys` and `aocl-data-analytics-sys`: required, because with the old files the three wrappers above no longer compile in `DOCS_RS=1` mode (docs.rs builds the wrapper crates against the vendored files; measured: 2, 9 and 29 errors before, 0 after). `DOCS_RS=1 cargo check` of the whole workspace passes. The other `*-sys` vendored files are not compared (ROADMAP row 5).
- `aocl-data-analytics`: **did not compile against AOCL 5.3.2 (13 errors); now does, public API unchanged.** `KNearestNeighbours` now calls `da_nn_*` (AOCL 5.3 renamed k-NN: `da_handle_nn`, `da_nn_set_data_d` + `da_nn_set_labels_d` instead of `da_knn_set_training_data_d`, `da_nn_classifier_predict(_proba)_d` with `knn_search_mode`, `da_nn_kneighbors_d`, `da_nn_classes_d`); the new header says the training data is stored by pointer, not copied, so the struct now owns copies of it. `da_linmod_define_features_d`, `da_linmod_evaluate_model_d` (x2), `da_tree_set_training_data_d` and `da_forest_set_training_data_d` gained `ldx` / `categorical_features` (the wrapper passes `ldx = n_samples`, i.e. the column-major default it assumed before, and no categorical features); `da_covariance_matrix_*` gained `assume_centered` (wrapper passes 0, i.e. subtract the means as before). 24 tests pass with `KMP_DUPLICATE_LIB_OK=TRUE` (see ROADMAP row 3: two OpenMP runtimes); new test `covariance_matrix_subtracts_column_means`; mutants (assume_centered, linmod `ldx`, `da_nn_set_labels_d` count) fail the tests; a mutant switching predict to radius search survives, because it gives the same labels under default options.
- `aocl-crypto`: **did not compile against AOCL 5.3.2; now does** (public API unchanged). Constants `_alc_cipher_mode_*`, `_alc_digest_mode_*`, `_alc_mac_type_*`, `alc_rng_*_t_*` are now `alc_cipher_mode_*` etc.; `alcp_cipher_encrypt/decrypt` and `alcp_cipher_aead_encrypt/decrypt` take an `outlen` pointer; two `bool` arguments. **Behaviour change fixed:** in 5.3 `alcp_cipher_aead_get_tag` on decryption compares the buffer you pass with the computed tag and returns `ALC_ERROR_TAG_MISMATCH` (14), so `Aead::verify_and_decrypt` (which passed a zeroed buffer) rejected every valid GCM message (a defect in the rejecting direction: new test `aes_gcm_every_tag_bit_flip_rejected` checked that all 128 single-bit flips of a tag are still rejected with the plaintext zeroed, and the unmodified tag is accepted); it now passes the expected tag, treats 14 as the tag-mismatch error, and still compares afterwards for older versions. 22/22 tests pass; mutants (mismatch check disabled; SHA2-256 mapped to SHA2-384) fail them.
- Tests: `aocl-securerng` and `aocl-utils` tests strengthened (the old ones only called functions and discarded the result: support query vs instruction agreement, array fill to the end, `is_amd` vs the vendor code, `has_flag` truth); each has two seeded mutants that now fail them. Note: `VendorInfo::vendor_id` is a numeric EVendor code ("1" on the AMD test host), not a vendor name as its use suggests.
- Tests on real AOCL (no code change): `aocl-lapack`, `aocl-fft`, `aocl-math`, `aocl-rng`, `aocl-sparse` run on Windows with AOCL 5.3.2; see ROADMAP row 3 for counts and the seeded mutants.
- Correction to 0.1.10's note: no `homepage` field was added (new cargo flags `homepage` equal to `repository` as redundant); only `repository` changed.

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
