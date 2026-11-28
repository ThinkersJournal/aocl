//! Documents the upstream defect behind `decompress(Codec::Bzip2, ..)` returning an error, and checks the replacement API.
//!
//! AOCL's bzip2 decoder (AOCL 6.0.68, Windows; Linux unmeasured) crashes the process on some CORRUPT streams: of the 218 streams made by flipping
//! one bit of a valid 109-byte stream (300 bytes of `data()`, compressed at level 5), three fault with an access violation inside the decoder:
//! byte 22 bit 0, byte 23 bit 7 and byte 31 bit 0. A crashing decode must not take the test runner down, so each flip is decoded in a CHILD
//! process (this same test executable re-run on `child_decode`); the parent asserts the child died with STATUS_ACCESS_VIOLATION. If AMD fixes
//! the decoder this test starts failing: that is the signal to re-measure and reconsider `decompress_bzip2_trusted` (no `#[ignore]`: the
//! child process is what isolates the crash). Exactly what is claimed: a crafted bzip2 stream crashes the process (access violation inside AOCL's
//! decoder); memory corruption is not excluded; exploitability is unmeasured.
use aocl_compression::*;

const STREAM_HEX: &str = "425a683531415926535907348c3300000060007ffffff820007051a1a0000004529a4f4651a6268c1a9de0970c86f4a8c4c1c14c44c871125513c8f884d1283384c81f6a6c31507407513718a29b0977d13a4be826e3384e01b13a21d21da126cd8fe2ee48a70a1200e6918660";
/// `(byte, bit)` of the three single-bit flips that fault.
const FLIPS: [(usize, u8); 3] = [(22, 0), (23, 7), (31, 0)];

fn stream() -> Vec<u8> {
    (0..STREAM_HEX.len() / 2)
        .map(|i| u8::from_str_radix(&STREAM_HEX[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}
fn data() -> Vec<u8> {
    (0..300u32).map(|i| (i % 7 + (i / 40) * 3) as u8).collect()
}
fn flipped(byte: usize, bit: u8) -> Vec<u8> {
    let mut s = stream();
    s[byte] ^= 1 << bit;
    s
}

#[test]
fn the_embedded_stream_is_what_the_encoder_produces_and_decodes_with_the_trusted_entry() {
    // the encoder is not affected by what was measured: same bytes as the embedded stream
    assert_eq!(compress(Codec::Bzip2, &data(), 5).unwrap(), stream());
    // SAFETY: the stream was produced by `compress` just above.
    let got = unsafe { decompress_bzip2_trusted(&stream(), 300) }.unwrap();
    assert_eq!(got, data());
}

#[test]
fn safe_decompress_refuses_bzip2_even_for_a_valid_stream() {
    for s in [stream(), flipped(22, 0), flipped(23, 7), flipped(31, 0)] {
        let r = decompress(Codec::Bzip2, &s, 300);
        assert!(
            matches!(r, Err(Error::InvalidArgument(ref m)) if m.contains("decompress_bzip2_trusted")),
            "{r:?}"
        );
    }
    // the other codecs still decode through the safe call
    let z = compress(Codec::Zlib, &data(), 6).unwrap();
    assert_eq!(decompress(Codec::Zlib, &z, 300).unwrap(), data());
}

/// Run by the parent below with `BZ_FLIP=byte:bit`; a no-op otherwise.
#[test]
fn child_decode() {
    let Ok(v) = std::env::var("BZ_FLIP") else {
        return;
    };
    let (byte, bit) = v.split_once(':').unwrap();
    let s = flipped(byte.parse().unwrap(), bit.parse().unwrap());
    // SAFETY: deliberately violates the contract, in a child process, to document the crash.
    let _ = unsafe { decompress_bzip2_trusted(&s, 300) };
}

#[cfg(windows)]
#[test]
fn documents_the_upstream_defect() {
    let exe = std::env::current_exe().unwrap();
    for (byte, bit) in FLIPS {
        let st = std::process::Command::new(&exe)
            .args(["--exact", "child_decode", "--nocapture"])
            .env("BZ_FLIP", format!("{byte}:{bit}"))
            .status()
            .unwrap();
        assert_eq!(
            st.code(),
            Some(0xC0000005u32 as i32),
            "flip byte {byte} bit {bit}: expected STATUS_ACCESS_VIOLATION from AOCL's bzip2 decoder; if it no longer crashes, re-measure (tools/safe-api-versions.py) and reconsider the refusal"
        );
    }
    // control: the unflipped stream decodes in a child without crashing
    let st = std::process::Command::new(&exe)
        .args(["--exact", "child_decode", "--nocapture"])
        .env("BZ_FLIP", "0:0")
        .status()
        .unwrap();
    assert!(
        st.code() != Some(0xC0000005u32 as i32),
        "byte 0 bit 0 (a header flip) must not crash: {st:?}"
    );
}
