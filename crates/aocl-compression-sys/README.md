# aocl-compression-sys

Raw FFI bindings to AOCL-Compression. For the safe API see [`aocl-compression`](../aocl-compression/).

**`static-link` (Windows):** links `aocl_compression` from the install's `static/` directory (it asked for a non-existent `aocl_compression-static` before).
**Two measured gates (AOCL 6.0.68 Windows; Linux unmeasured).** (1) `BZ2_bzCompressBound` and `Lzma_compressBound` exist only in the STATIC library: they are declared only with `static-link` (a dynamic build cannot link them; before, a link error). (2) Ten LZMA-SDK stream helpers (`LookInStream_LookRead`, `LookInStream_Read`, `LookInStream_Read2`, `LookInStream_SeekTo`, `LookToRead2_CreateVTable`, `SecToLook_CreateVTable`, `SecToRead_CreateVTable`, `SeqInStream_Read`, `SeqInStream_Read2`, `SeqInStream_ReadByte`) are declared (plain functions in `7zTypes.h`, not inline or macros) but exported by NO installed library, dynamic or static: they are left out unless the feature `header-only-unexported` (default OFF) is enabled, which only declares them; it does not make them link on Windows.

Dual-licensed under MIT or Apache-2.0.
