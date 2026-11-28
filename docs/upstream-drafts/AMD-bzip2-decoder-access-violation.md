# DRAFT, NOT SENT: AOCL-Compression bzip2 decoder crashes on corrupt streams

Status: a draft for CireSnave. Nothing has been sent to AMD (standing ruling, board 518: nothing goes to AMD without CireSnave's word; no public GitHub issue).

## Summary
`aocl_llc_decompress` with the BZIP2 codec terminates the process with an access violation (0xC0000005) inside the decoder when given certain corrupt bzip2 streams. A crafted bzip2 stream crashes the process; memory corruption is not excluded (exploitability was not measured).

## Environment
- AOCL 6.0.68, AOCL-Compression DLL as installed by the AOCL Windows installer (`C:\Program Files\AMD\AOCL-Windows`, LP64 shared build), called through `aocl_llc_setup` / `aocl_llc_decompress` / `aocl_llc_destroy` with `optLevel = 2`.
- Windows 11 Pro 10.0.26300, x86-64. Linux was not measured.

## Reproduction
1. Input: the 300 bytes `data[i] = (i % 7 + (i / 40) * 3) as u8` for `i` in `0..300`.
2. Compress with `aocl_llc_compress` (BZIP2, `level = 5`, `optLevel = 2`): the result is this 109-byte stream (hex):
   `425a683531415926535907348c3300000060007ffffff820007051a1a0000004529a4f4651a6268c1a9de0970c86f4a8c4c1c14c44c87112551` followed by `3c8f884d1283384c81f6a6c31507407513718a29b0977d13a4be826e3384e01b13a21d21da126cd8fe2ee48a70a1200e6918660`.
3. Flip exactly one bit and call `aocl_llc_decompress` (BZIP2) with `outSize = 300`: byte 22 bit 0 (`^= 0x01`), byte 23 bit 7 (`^= 0x80`) or byte 31 bit 0 (`^= 0x01`) each crash the process (exit code 0xC0000005). The unflipped stream decodes to the original 300 bytes.

## Measurement
A guard-page harness decoded every truncation (109 streams) and every single-bit flip of bits 0 and 7 of every byte (218 streams) of that stream, each with three output sizes: 3 of the 218 flips fault, no truncation does. The input buffer was also placed with 4 KiB of readable slack after it and the same flip still faulted, so it is not a read past the end of the input at the buffer boundary. The other codecs (LZ4, LZMA, Snappy, Zlib, Zstd) were run through the same truncation and bit-flip sweep with no fault.

Where to find it in our repository: `crates/aocl-compression/tests/bzip2_upstream_defect.rs` (single-process repro, one child process per flip); results `docs/safe-api-audit/results-aocl-compression.csv`.

## What we did on our side
Our Rust wrapper refuses to decode bzip2 through its safe API (`decompress_bzip2_trusted` is `unsafe` and documents the contract).
