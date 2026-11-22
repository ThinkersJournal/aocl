# Third-party notices

This repository's own code is `MIT OR Apache-2.0` (see `LICENSE-MIT`, `LICENSE-APACHE`). It contains no AMD or other
third-party binary, no copied header, and no copied licence or documentation text. What it does contain that derives
from third parties is listed here, with each origin and what was checked. Facts below were read from an AOCL for
Windows install on 2026-11-22 (release notes of the 11 components that name a version: 5.3.0; installer registry
`DisplayVersion` 5.3.514) and from this repository at the time of writing.

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
| `aocl-fft-sys` | `fftw3.h` | BSD-style, Copyright 2003, 2007-14 Matteo Frigo | FFTW (AOCL-FFTW; the library itself is GPL, see 3) |
| `aocl-lapack-sys` | `lapack.h` | NO notice in the header | AOCL-libFLAME |
| | `lapacke.h` | BSD-style, Copyright 2014 Intel Corp. | LAPACKE |
| `aocl-blas-sys` | `blis.h`, `cblas.h` | NO notice in the header | AOCL-BLAS (BLIS) |
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
4. **`aocl-blas-sys`, `aocl-lapack-sys` (`blis.h`, `cblas.h`, `lapack.h`).** No notice in the headers. The AOCL-BLAS
   `NOTICE.txt` lists `flame-blis` as BSD-3-Clause, Copyright 2017 AMD and 2014 The University of Texas at Austin, which
   is the likely origin of the BLIS headers; this is an inference from the NOTICE, not from the headers. For
   AOCL-libFLAME the head of its `NOTICE.txt` lists third-party components (e.g. CMake, BSD-3-Clause) and not
   libFLAME's own licence, so `lapack.h` is unresolved.
5. **FFTW.** `fftw3.h` carries a BSD-style notice but the AOCL-FFTW `LICENSE.txt` says the distribution "does not include
   AMD-owned licensable code" and points to a 97 KB `NOTICE.txt` of third-party licences (FFTW is GPL-2.0-or-later).
   `aocl-fft-sys` links the FFTW library dynamically on the user's machine; it does not ship it. Whether a GPL library
   linked by an `MIT OR Apache-2.0` crate needs a note beyond this one is a policy question.

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
