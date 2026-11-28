# aocl-data-analytics

Safe Rust wrappers for AOCL-DA (data analytics, statistics, ML primitives).

Built on top of [`aocl-data-analytics-sys`](../aocl-data-analytics-sys/). Uses the shared [`aocl-types::Layout`](../aocl-types/) for matrix-orientation enums.

Currently exposes basic-statistics primitives (`mean`, `variance` along row / column / global axes). Linear models, k-means, PCA, decision forests, k-NN, SVM will follow.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4 and 0.2.5 are yanked: `KNearestNeighbours::classes` returned garbage or faulted on an unfitted handle; `Handle::last_error_message` could read uninitialised memory; concurrent `Nlls` fits crashed the process. Fixed in 0.2.28.

### Affected versions (safe-API memory safety)

Measured per published version with a guard-page harness built against each `.crate`: the five kernel / distance functions (`linear_kernel`, `rbf_kernel`, `polynomial_kernel`, `sigmoid_kernel`, `pairwise_distances`) faulted in 0.1.0 (source read) through 0.2.49, fixed in 0.2.54; the handle models (decision tree / forest `predict_proba`, `predict_log_proba`, `score`; kNN `kneighbors`, `predict_proba`; `Svm::score`, `decision_function`; `KMeans::predict`: 19 faulting cases and one write past the `kneighbors` output buffer) in 0.1.0 through 0.5.0, fixed in 0.5.1; `Datastore::extract_f64` (a write past the caller's buffer) in 0.1.0 through 0.5.1, fixed in 0.5.2. The 11 free statistics functions were measured clean in every version. NOT MEASURED by a harness: `Handle` options and the `Nlls` callbacks (classified by source read). yank status as read from crates.io on 2026-11-28 (about 15:50 UTC); the yank itself is the maintainer's and the live list moves, crates.io is the authority. Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
