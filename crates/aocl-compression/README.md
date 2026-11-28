# aocl-compression

**`decompress(Codec::Bzip2, ..)` returns an error for every bzip2 stream, valid ones included (from 0.5.0).** AOCL's bzip2 decoder crashes the process on some corrupt streams (an access violation inside the decoder; measured on AOCL 6.0.68, Windows; memory corruption is not excluded), so a safe decode of a stream an attacker could have modified is not possible: this is a documented functional regression until a pure-Rust decoder is available as an optional feature. Use `unsafe fn decompress_bzip2_trusted` on a stream from a trusted source; compression with `Codec::Bzip2` is not affected by what was measured.


Safe Rust wrappers for AOCL-Compression (LZ4 / ZLIB / ZSTD / LZMA / BZIP2 / Snappy / LZ4HC).

Built on top of [`aocl-compression-sys`](../aocl-compression-sys/).

Dual-licensed under MIT or Apache-2.0.


### Affected versions (safe-API memory safety)

**Measured: a crafted bzip2 stream crashes the process (an access violation inside AOCL's decoder); memory corruption is not excluded; exploitability is unmeasured.** Every published version from 0.1.0 (source read) through 0.4.0 decodes bzip2 through the safe `decompress(Codec::Bzip2, ..)`: 3 of 218 single-bit flips of a valid stream fault. From 0.5.0 the safe call refuses bzip2 (see the notice at the top of this file). The LZ4, LZMA, Snappy, Zlib and Zstd decoders and the native lz4 / snappy paths showed no fault in the same sweep. yank status as read from crates.io on 2026-11-28 (about 15:50 UTC); the yank itself is the maintainer's and the live list moves, crates.io is the authority. Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
