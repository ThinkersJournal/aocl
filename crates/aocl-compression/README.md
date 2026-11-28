# aocl-compression

Safe Rust wrappers for AOCL-Compression (LZ4 / ZLIB / ZSTD / LZMA / BZIP2 / Snappy / LZ4HC).

Built on top of [`aocl-compression-sys`](../aocl-compression-sys/).

Dual-licensed under MIT or Apache-2.0.

## bzip2 decoding

`decompress(Codec::Bzip2, ..)` returns an error. AOCL's bzip2 decoder crashes the process on some corrupt streams (an access violation inside the decoder; measured on AOCL 6.0.68, Windows; memory corruption is not excluded). Use `unsafe fn decompress_bzip2_trusted` on a stream from a trusted source. Compression with `Codec::Bzip2` is not affected by what was measured.
