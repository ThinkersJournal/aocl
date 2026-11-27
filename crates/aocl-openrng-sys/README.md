# aocl-openrng-sys

Raw FFI declarations for AOCL-OpenRNG (AMD's fork of Arm's OpenRNG; VSL-compatible RNG: streams, 15+ base generators, continuous and discrete distributions).

**Status: skeleton, UNMEASURED.** 60 functions, 74 constants, the types and the `VSLBRngProperties` struct are generated mechanically from `include/openrng.h` of `amd/openrng` tag `6.0.0` (commit `5c05f2ee...`, header sha256 `ee3a48fc...`): see `NOTICE` (licence: header `MIT OR Apache-2.0 WITH LLVM-exception`; the repository's board-523-A legal line does NOT apply here) and `docs/cleanroom/openrng/provenance.csv` (per item: header line, tag, commit, sha256, whether guide chapter 10 names it: 59 of 60 do; `openRngGetVersion` is in the header and not in the guide).

**Not measured (needs a built library; none is installed, AOCL 6.0.68 for Windows has no OpenRNG):** the library name/location (set `OPENRNG_LIB_DIR`; `static-link` selects `openrng_static`: names are from the upstream README, unverified), the integer width (header: `openrng_int_t` is `int32_t`, or `long long` when `OPENRNG_ILP64`/`INTEGER64` is defined: the `ilp64` feature picks the latter), and all behaviour. `tests/link_all.rs` and `tests/stream.rs` are written and compile, and are built only when `OPENRNG_LIB_DIR` is set; **they have not been run**. No safe wrapper exists (design: `docs/cleanroom/openrng/WRAPPER-DESIGN.md`; no code until the PM reads this crate).

Licence: `MIT OR Apache-2.0 WITH LLVM-exception`; texts in `LICENSE-AMD-OPENRNG` and `LICENSE-ARM-OPENRNG`.
