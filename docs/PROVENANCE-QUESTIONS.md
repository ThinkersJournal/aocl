# Provenance questions for the owner (self-serve, no attorney)

Written 2026-11-22 by lane aocl for the cluster PM, to become board items. I am not a lawyer and nobody here has
consulted one (the owner's 2026-11-05 rule is self-serve). Every statement is either quoted from a file I read on this
machine or from a download I read today, or it is marked **[GUESS]**. Nothing was published or posted.

> **Historical (written 2026-11-22).** The "hold publish" lines below were this lane's proposals; the later written rulings and the crates.io state are in `THIRD-PARTY-NOTICES.md` 2a and ROADMAP row 1. Nothing below is a hold in force.

## Read this first: the exposure already exists

crates.io already holds 0.1.0 of every crate in this repo. I downloaded `aocl-math-sys`, `aocl-rng-sys`,
`aocl-blas-sys`, `aocl-lapack-sys` and `aocl-fft-sys` 0.1.0: each contains `bindings/bindings.rs` (math 377 lines,
rng 2,379, fft 1,746, lapack 61,765, blas 88,571) and **no licence or notice file at all**, not even the MIT and Apache
texts the `license = "MIT OR Apache-2.0"` field promises (the workspace-root `LICENSE-*` files are not inside the
package). Control: the same listing on `blis-src` 0.2.2 shows its `upstream/LICENSE`. So "hold publish" does not undo
what is out; whether to **yank** 0.1.0 of `aocl-math-sys` and `aocl-rng-sys` (yank hides it from new dependency
resolution and does not delete it, and can be reversed) is a decision for the owner. Separate, certain defect: every
published crate needs its own copy of the two licence texts.

## Question 1. Do name-and-signature bindings of an object-code-only library need AMD's permission?

**What the AMD files say** (all read from the installed AOCL for Windows, 5.3.x):

- `amd-libm\include\amdlibm.h`, the header `aocl-math-sys` is generated from, carries AMD's "Software License Agreement".
  Section 1(a): a licence to "download, copy, use, distribute and sublicense ... the object code version of the Software
  and materials associated with this Agreement", "conditioned upon Licensee distributing the object code version of the
  Software only and under this software license agreement". Section 1(b): "Licensee does not have the right to (i)
  distribute, rent, lease, sell, sublicense, assign, or otherwise transfer the Materials, in whole or in part, to third
  parties ...; or (ii) modify, disassemble, reverse engineer, or decompile the Software". Nothing grants source-form
  redistribution.
- `amd-rng\include\rng.h`, the header for `aocl-rng-sys`: the only notice is `Copyright AMD,NAG 2003-2023`. No grant at all.
- The Windows `EULA.txt` ("Evaluation and Object Code Distribution Agreement") is byte-identical in all 12 components
  (SHA-256 compared). It grants object-code use and distribution inside your own products under an end-user agreement;
  section 3 forbids modifying, creating derivative works of, or distributing "the Software"; section 7 says an open-source
  component whose licence conflicts with the agreement is governed by its own licence.
- The other headers state BSD or MIT notices (sparse, securerng: MIT-style; utils, crypto, DA, compression: BSD-style),
  which do permit redistribution of derived text with the notice. `blis.h`, `cblas.h`, `lapack.h` have no notice; the
  component NOTICE files give BSD-3-Clause for BLIS (`flame-blis`, Copyright 2017/2018 AMD, 2014 UT Austin) and for
  libFLAME (`flame-libflame`, `reference-lapack-lapack`).

**What other crates do** (downloaded from crates.io and read today; their own files, not recollection):

| Crate | Library's licence | What the crate ships | What I read |
|---|---|---|---|
| `intel-mkl-sys` 0.8.1 | Intel Simplified Software License (permits redistribution "without modification" if the notice and terms are reproduced) | `src/mkl.rs`, 13,938 lines generated from MKL headers | `license-file = "License.txt"` is Intel's licence; README: "MKL is distributed under the Intel Simplified Software License ... Some wrapper codes are licensed by MIT". The vendor's grant exists and the crate reproduces it. |
| `opencl-sys` 0.6.5 | Apache-2.0 (Khronos) | generated bindings | `LICENSE` Apache-2.0; README "as per Khronos Group OpenCL". Grant exists. |
| `cudarc` 0.19.10 | NVIDIA CUDA (terms not read by me) | generated bindings for cuBLAS, cuDNN, cuFFT, cuSOLVER and others | `license = "MIT OR Apache-2.0"`, only `LICENSE-MIT` and `LICENSE-APACHE`; **0** files under `src/` contain the word "copyright". It publishes vendor bindings under its own licence with no vendor notice. That shows the practice exists, **not** that NVIDIA permits it: I did not read NVIDIA's terms. |
| `blis-src` 0.2.2 | BLIS BSD-3-Clause | bundles BLIS source | `upstream/LICENSE` shipped in the package (the control above). |
| `openblas-src` 0.10.16, `accelerate-src` 0.3.2 | BSD / Apple | link-only crates (10 files each, no bindings file; `accelerate-src` ships a `LICENSE.md`) | file lists only; not read in detail |

So the real pattern is: where the vendor grants redistribution (Intel, Khronos, BLIS), crates carry the vendor's text;
the one crate that ships bindings of a restrictive vendor's library (`cudarc`) does so without any vendor text.

**Lean (best-safe-guess):** do not publish bindings text derived from `amdlibm.h` or `rng.h` without AMD's answer (the
`aocl-math-sys` header's own words are about object code and "in whole or in part"). **[GUESS]** The safe shape that
avoids carrying AMD text at all is a `-sys` crate that holds only our `wrapper/*.h` and Rust, and runs bindgen at build
time against the headers the user already installed; docs.rs then cannot build it (no AOCL there), so its docs would
need to be written without generated bindings or skipped. I do not know that this is enough in law. For the BSD/MIT
crates: publish with the notice text copied into each crate. For BLIS and libFLAME: **[GUESS]** the NOTICE BSD-3 text
copied into the crate covers header-derived names, because the headers belong to those projects. I would not rely on
"the files contain 0 doc comments" as a licence: it is a fact about the notice, not a grant.

## Question 2. FFTW: GPL library, header-derived bindings

**What the sources say:**

- `fftw3.h` (AOCL install): BSD-2-Clause, Copyright 2003, 2007-14 Matteo Frigo and MIT, 2020-21 AMD, and: "The following
  statement of license applies *only* to this header file, and *not* to the other files distributed with FFTW or derived
  therefrom". The AOCL-FFTW `NOTICE.txt` lists `fftw v3.3.10 (BSD-2-clause)` (that header) and `fftw-fftw3 v3.3.8
  (GPL-2.0)` (the library). `LICENSE.txt` says the distribution "does not include AMD-owned licensable code".
- FSF GPL FAQ, fetched today from gnu.org: *#GPLStaticVsDynamic*: "Linking a GPL covered work statically or dynamically
  with other modules is making a combined work based on the GPL covered work. Thus, the terms and conditions of the GNU
  General Public License cover the whole combination." *#IfLibraryIsGPL*: "Yes, because the program actually links to the
  library. As such, the terms of the GPL apply to the entire combination. The software modules that link with the library
  may be under various GPL compatible licenses, but the work as a whole must be licensed under the GPL."
- `fftw-sys` 0.8.0 (crates.io), `LICENSE.md`: "`fftw` and `fftw-sys` crates are distributed under dual license based on
  which backends are linked": FFTW backend: "FFI codes (`fftw-sys` crate) ... are also distributed under GPLv2 or later";
  Intel MKL backend: MIT.

**Which carries the GPL?** The library (the DLL, and the AOCL-FFTW package) carries it. The FFTW authors licensed the header
BSD-2 for itself, apparently so programs can be written against it; our `aocl-fft-sys` ships header-derived bindings
and no library. The combination that the FAQ describes is made when a user builds a program that links the DLL, and the
GPL then binds that program when it is distributed. **[GUESS]** that a crate containing only extern declarations is not
itself "the combined work"; the FSF text above is about a program that links, and I found no FSF text about a source-only
bindings crate. The one comparable crate (`fftw-sys`) chose to be GPLv2+ when it targets FFTW, which is the conservative reading.

**Lean:** keep `aocl-fft-sys` and `aocl-fft` `MIT OR Apache-2.0`, include the `fftw3.h` BSD-2 notice text in the `-sys`
crate, and state in the crate README and the `lib.rs` docs, in these words: "this crate links the AOCL-FFTW library
(GPL-2.0 per its NOTICE); a program that links it and is distributed is subject to the GPL; this crate ships no FFTW
library". Hold publish until the pm decides; **[GUESS]** the stricter alternative is to follow `fftw-sys` and make
`aocl-fft-sys` GPL-2.0-or-later. The same sentence is needed for the `aocl` umbrella's `fft` feature.

**Fact recorded 2026-11-27 (read from the installed AOCL 6.0.68 `amd-fftw`):** the two AOCL files disagree on the library's
GPL version. `COPYRIGHT` lines 7-9: "either version 2 of the License, or (at your option) any later version". `NOTICE.txt`
component `fftw-fftw3 v3.3.8` is listed as GPL-2.0. **Ruling (pm-kernels, 2026-11-26):** `aocl-fft-sys` may be published
under our licence with the conditions in its README, `lib.rs` and `THIRD-PARTY-NOTICES.md`; whether a source-only extern
crate is itself a combined work stays UNKNOWN (no FSF text found).

## Question 3. The AMD question (drafted, NOT posted)

Needed from AMD for `amdlibm.h` (AOCL-LibM) and `rng.h` (AOCL-RNG): permission to publish, in Rust crates licensed
`MIT OR Apache-2.0` with AMD's copyright notice kept, bindings generated by `bindgen` from those two headers (function and
type names, signatures, constant values; no comments or prose). Where to ask is not settled: `amd/aocl-libm-ose` has 38
issues (open-source LibM; the Windows `amdlibm.h` carries the object-code licence quoted above), and I found no public AMD
repository for AOCL-RNG among the nine `amd/*` repositories I listed, so there is no known place to file that half.
Draft (a comment or e-mail, not an issue, until the owner chooses the channel):

> We publish Rust FFI crates for AOCL (`aocl-math-sys`, `aocl-rng-sys`), generated with bindgen from `amdlibm.h` (AOCL
> 5.3.x for Windows; its notice is the AMD Software License Agreement, object code) and `rng.h` (notice: "Copyright
> AMD,NAG 2003-2023", no licence text). The generated files contain only names, signatures and constant values. May we
> publish them under MIT OR Apache-2.0 with AMD's copyright notice retained? If AMD prefers another licence or wording
> for such interface files, please say which. Our crates do not redistribute any AOCL library.
> — lane: aocl

## Where I am guessing

1. That a bindings crate holding only names and signatures needs no permission beyond the header's own licence: not
   established; the `amdlibm.h` text points the other way ("in whole or in part"), and `cudarc` only shows practice.
2. That building bindings at the user's build time avoids carrying AMD text: plausible, untested in law.
3. That the BLIS and libFLAME NOTICE licences cover the header files that lack their own notice.
4. That a source-only bindings crate is not the "combined work" under the GPL.
5. I did not read NVIDIA's, Apple's, OpenBLAS's or Khronos's licence texts beyond what the crates ship.
