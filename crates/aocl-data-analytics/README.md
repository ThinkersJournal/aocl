# aocl-data-analytics

Safe Rust wrappers for AOCL-DA (data analytics, statistics, ML primitives).

Built on top of [`aocl-data-analytics-sys`](../aocl-data-analytics-sys/). Uses the shared [`aocl-types::Layout`](../aocl-types/) for matrix-orientation enums.

Currently exposes basic-statistics primitives (`mean`, `variance` along row / column / global axes). Linear models, k-means, PCA, decision forests, k-NN, SVM will follow.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4 and 0.2.5 are yanked: `KNearestNeighbours::classes` returned garbage or faulted on an unfitted handle; `Handle::last_error_message` could read uninitialised memory; concurrent `Nlls` fits crashed the process. Fixed in 0.2.28.
