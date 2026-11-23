# Third-party notices

This repository's own code is `MIT OR Apache-2.0` (see `LICENSE-MIT`, `LICENSE-APACHE`). It contains no AMD or other
third-party binary, no copied header, and no copied licence or documentation text. What it does contain that derives
from third parties is listed here, with each origin and what was checked. Facts below were first read from an AOCL
for Windows 5.3 install on 2026-11-22 (installer `DisplayVersion` 5.3.514). The `NOTICE` files of the six BSD/MIT
`*-sys` crates (sparse, securerng, utils, crypto, data-analytics, compression) were re-diffed on 2026-11-23 against the
headers of the installed AOCL for Windows 6.0 (installer `DisplayVersion` 6.0.68; the component `ReleaseNotes.txt`
files that name a version say 6.0.0), which is what the committed bindings are generated from, and refreshed to the
6.0 text. The other facts in this file (blas, lapack, fft, math, rng headers and NOTICE entries) are still from 5.3
and have not been re-read against 6.0; those crates are held.
Reproduce the header comparison with `python tools/check-notices.py` (exit 0 = all match; `--ref <rev>` reads the
NOTICE files from an older revision, the control). The `NOTICE.txt` files of AOCL-SecureRNG, -Utils, -Crypto and -DA list
third-party code of those libraries (documentation tooling, test frameworks, internal libraries), none of it in the
headers these crates derive from, so none is copied; AOCL-Sparse ships no `NOTICE.txt`. The bzip2 and LZ4 blocks of
`aocl-compression-sys` were compared with the 6.0.68 AOCL-Compression `NOTICE.txt` and are verbatim identical.

## 1. Generated FFI bindings (`crates/*-sys/bindings/bindings.rs`, 11 files)

Each file is the output of `bindgen` run on the public C headers of one AOCL component. What survives into the file is
the interface: type, struct, enum and function names, signatures, and numeric constant values. Measured: none of the 11
files contains a doc comment (`///` or `/**`; control: the same pattern finds 259 lines in a hand-written file of this
repo) and none contains licence or copyright text beyond the one-line pointer to this file added by this repo.
The files are used only for docs.rs builds (`DOCS_RS=1`); real builds regenerate them from the installed headers.

The headers those files were generated from carry these notices (read from the installed headers):

| Crate | Header(s) used (`wrapper/*.h`) | Notice in the header | Origin |
|---|---|---|---|
| `aocl-sparse-sys` | `aoclsparse*.h` | MIT-style, Copyright AMD | AOCL-Sparse |
| `aocl-securerng-sys` | `secrng.h` | MIT-style, Copyright 2017-2023 AMD | AOCL-SecureRNG |
| `aocl-utils-sys` | `alci/*`, `Capi/au/*` | BSD-style, Copyright AMD | AOCL-Utils |
| `aocl-crypto-sys` | `alcp/alcp.h`, `alcp/rsa.h` | BSD-style, Copyright AMD | AOCL-Cryptography |
| `aocl-data-analytics-sys` | `aoclda.h` | BSD-style, Copyright AMD | AOCL-DA |
| `aocl-compression-sys` | `aocl_compression.h`, `LzmaEnc.h`, `LzmaDec.h` | BSD-style, Copyright AMD | AOCL-Compression |
| | `lz4.h`, `lz4hc.h` | BSD-style, Copyright Yann Collet | LZ4 |
| | `snappy-c.h` | BSD-style, Copyright Martin Gieseking | Snappy C binding |
| | `bzlib.h` | copyright line only, Julian Seward | bzip2 |
| | `lz4frame.h` | not installed on the test box (see below) | LZ4 |
| `aocl-fft-sys` | `fftw3.h` | BSD-2-Clause, Copyright 2003, 2007-14 Matteo Frigo and MIT, 2020-21 AMD; the header says the licence "applies *only* to this header file, and *not* to the other files distributed with FFTW" | FFTW (AOCL-FFTW NOTICE: `fftw v3.3.10 (BSD-2-clause)` is the header; `fftw-fftw3 v3.3.8 (GPL-2.0)` is the library) |
| `aocl-lapack-sys` | `lapack.h` | NO notice in the header; AOCL-libFLAME `NOTICE.txt` lists `flame-libflame` (BSD-3-Clause) and `reference-lapack-lapack` (BSD-3-Clause) | AOCL-libFLAME |
| | `lapacke.h` | BSD-style, Copyright 2014 Intel Corp. | LAPACKE |
| `aocl-blas-sys` | `blis.h`, `cblas.h` | NO notice in the header; AOCL-BLAS `NOTICE.txt` lists `flame-blis` (BSD-3-Clause), Copyright 2017/2018 AMD and 2014 The University of Texas at Austin, full licence text in that NOTICE | AOCL-BLAS (BLIS) |
| `aocl-rng-sys` | `rng.h` | `Copyright AMD,NAG 2003-2023` only, no licence grant | AOCL-RNG |
| `aocl-math-sys` | `amdlibm.h` | AMD "Software License Agreement" (object-code licence, see 2) | AOCL-LibM |

Where the notice is BSD- or MIT-style, redistribution of a derived work is allowed provided the notice is kept; this
file keeps the copyright holders' names and the licence family, and the bindings contain only interface facts. For the
BSD-style headers the licence text is the standard 3-clause one (not compared word for word for each header).
`aocl-scalapack-sys` is hand-written: AOCL ships no public ScaLAPACK headers, and the declarations follow the published
ScaLAPACK / BLACS conventions; nothing was copied.

## 2. Open questions (not decided here; they need the owner)

These are listed because the check found no licence that clearly allows what the repo does. Nothing below was resolved
by assumption.

1. **`aocl-math-sys` (`amdlibm.h`).** The header's notice is AMD's "Software License Agreement", which grants a licence
   to "the object code version of the Software and materials associated with this Agreement" on condition that
   "all copyright legends and other legal notices" are kept. It does not grant a source-form or derivative-work
   licence in the text read. The published bindings contain 118 function declarations derived from it (names and
   signatures only). Whether interface-only derivation is permitted by that text is not established.
2. **`aocl-rng-sys` (`rng.h`).** The only notice is `Copyright AMD,NAG 2003-2023` with no licence grant. Same question.
3. **AOCL for Windows `EULA.txt`** ("Evaluation and Object Code Distribution Agreement", byte-identical in all 12
   components, SHA-256 compared). Its grant is object code, for distribution inside the licensee's own products under
   an end-user agreement, and section 3 forbids modifying, creating derivative works of, or distributing "the
   Software". Section 7 says an open-source component whose licence is inconsistent with the agreement is governed
   by its own licence. The repo distributes none of the Software (no DLL, header or document), so this governs the
   user who installs AOCL, not this repository's crates; whether header-derived interface text falls under it for
   headers that carry their own BSD/MIT notice is answered by section 7 only for those headers.
4. **`aocl-blas-sys`, `aocl-lapack-sys` (`blis.h`, `cblas.h`, `lapack.h`).** No notice in the headers themselves. The
   component `NOTICE.txt` files name the licence of the project the headers belong to (BLIS: `flame-blis`
   BSD-3-Clause; libFLAME: `flame-libflame` BSD-3-Clause), so the licence exists, but the link from these header files
   to those NOTICE entries is an inference (same project, same install), not something the header states.
5. **FFTW.** The header is BSD-2-Clause and says so for itself only; the library is GPL-2.0 (AOCL-FFTW NOTICE entry
   `fftw-fftw3 v3.3.8 (GPL-2.0)`). `aocl-fft-sys` ships header-derived bindings and no library. See
   `docs/PROVENANCE-QUESTIONS.md` question 2 for the GPL question and what is guessed.

## 2b. Per-crate copies

Each published crate must carry its own notice text. Done on this branch: `LICENSE-MIT` and `LICENSE-APACHE` in all 29
crate directories (the published 0.1.0 packages carry neither), and a `NOTICE` file in `aocl-sparse-sys`,
`aocl-securerng-sys`, `aocl-utils-sys`, `aocl-crypto-sys`, `aocl-data-analytics-sys`, `aocl-compression-sys` and
`aocl-fft-sys` holding the verbatim leading notice blocks of every header in the include closure of that crate's
`wrapper/*.h` (plus, for compression, the bzip2 and LZ4 texts from the AOCL-Compression NOTICE). `aocl-blas-sys` and
`aocl-lapack-sys` carry a `NOTICE` with the BLIS and libFLAME/LAPACK entries from the component NOTICE files.
`aocl-math-sys` and `aocl-rng-sys` have no `NOTICE` because their publication is held (below).
Checked with `cargo package --list` (`aocl-sparse-sys` lists `NOTICE`, `LICENSE-MIT`, `LICENSE-APACHE`; control:
`aocl-math-sys` lists the two licences and no `NOTICE`).

## 2a. Publication status (CireSnave's cluster PM, 2026-11-22)

Publish is **held** for every crate whose bindings derive from a header with no licence grant: `aocl-math-sys`
(`amdlibm.h`), `aocl-rng-sys` (`rng.h`), and `aocl-blas-sys` / `aocl-lapack-sys` until the NOTICE terms above are
accepted as covering the headers; and for `aocl-fft-sys` until the GPL question is answered. Crates whose headers state
a BSD/MIT notice (`sparse`, `securerng`, `utils`, `crypto`, `data-analytics`, `compression` AMD/lz4/snappy parts) may
publish once each published crate contains its own copy of the notice text.
Already published: crates.io holds 0.1.0 of the `*-sys` crates with their `bindings/bindings.rs` and **no licence or
notice file at all** (checked by downloading `aocl-math-sys`, `-rng-sys`, `-blas-sys`, `-lapack-sys`, `-fft-sys`
0.1.0; `aocl-math-sys` 0.1.0 contains `bindings/bindings.rs`, 377 lines from `amdlibm.h`, and no `LICENSE*`).

## 3. `lz4frame.h` guard (`crates/aocl-compression-sys/wrapper/compression.h`)

The wrapper includes `<lz4frame.h>` only if `__has_include` finds it, because the Windows AOCL install ships `lz4.h`
and `lz4hc.h` but not `lz4frame.h` (listed: `ls amd-compression/include`). The `LZ4F_*` declarations in the committed
`aocl-compression-sys` bindings come from an earlier install that had that header; their origin is the LZ4 project
(BSD-2-Clause, Yann Collet), and it was not verified which install produced them.

## 4. Test data

| Where | What | Origin |
|---|---|---|
| `crates/aocl-crypto/src/digest.rs` | SHA-256, SHA-512, SHA3-256 digests of `"abc"` (and similar) | public standard example values (NIST FIPS 180-4 / FIPS 202); not copied from AOCL |
| `crates/aocl-crypto/src/mac.rs` | HMAC-SHA256 known answer | published HMAC test vector (RFC 4231 family); the exact source was not recorded in the test and was not re-checked here |
| `crates/aocl-compression/src/lib.rs` | zlib stream of `"hello hello hello hello"` | produced for this repo with Python's `zlib.compress(..., 6)` |
| same | zstd `28 B5 2F FD`, zlib CMF/FLG rule, bzip2 `BZh`, snappy length preamble | file-format constants from the formats' public specifications |
| `crates/aocl-scalapack/tests/pdgesv_mpi.rs`, other tests | small matrices and known solutions | written for this repo |

## 5. Not in this repository

AOCL libraries and headers, Intel oneAPI runtimes (`libiomp5md.dll` and friends), the Microsoft OpenMP runtime
(`libomp140`), and MPI. They are installed by the user, who accepts their licences; the crates only link to them.
CI actions are referenced by pinned commit, not copied.

— lane: aocl
