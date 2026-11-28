//! Guard-page harness for the aocl-compression safe functions, hand-written. The library parses attacker-controlled bytes, so each case feeds a
//! valid stream that has been TRUNCATED at every length, or has ONE BIT FLIPPED at every byte, or a destination one byte short, with the
//! input / output at the end of a committed page before a PAGE_NOACCESS page: an access violation = HOLE. One process per test
//! (tools/safe-api-harness.py); each case loops over all variants and prints `RESULT <name> accepted` when none faulted. Controls (the
//! round trip) must succeed. Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_compression::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

fn at_guard<R>(src: &[u8], f: impl FnOnce(&mut [u8]) -> R) -> R {
    assert!(src.len() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(VirtualProtect((base as *mut u8).add(PAGE) as *mut c_void, PAGE, 0x01, &mut old), 0);
        let start = (base as *mut u8).add(PAGE - src.len());
        let s = core::slice::from_raw_parts_mut(start, src.len());
        s.copy_from_slice(src);
        let r = f(s);
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn data() -> Vec<u8> {
    (0..300u32).map(|i| (i % 7 + (i / 40) * 3) as u8).collect()
}
fn report(name: &str, n: usize) {
    println!("RESULT {name} accepted ({n} variants, no fault)");
}

fn variants(comp: &[u8]) -> Vec<Vec<u8>> {
    let mut v: Vec<Vec<u8>> = (0..comp.len()).map(|n| comp[..n].to_vec()).collect();
    for i in 0..comp.len() {
        for bit in [0u8, 7] {
            let mut c = comp.to_vec();
            c[i] ^= 1 << bit;
            v.push(c);
        }
    }
    v
}

fn decompress_all(codec: Codec, name: &str) {
    let d = data();
    let comp = compress(codec, &d, 5).unwrap();
    let mut n = 0;
    for v in variants(&comp) {
        for dst_len in [d.len(), d.len() - 1, 1] {
            at_guard(&v, |s| {
                let _ = decompress(codec, s, dst_len);
            });
            n += 1;
        }
    }
    report(name, n);
}
#[test]
fn decompress_lz4_corrupt() {
    decompress_all(Codec::Lz4, "decompress_lz4_corrupt");
}
#[test]
fn decompress_lzma_corrupt() {
    decompress_all(Codec::Lzma, "decompress_lzma_corrupt");
}
#[test]
fn decompress_bzip2_corrupt() {
    decompress_all(Codec::Bzip2, "decompress_bzip2_corrupt");
}
#[test]
fn decompress_snappy_corrupt() {
    decompress_all(Codec::Snappy, "decompress_snappy_corrupt");
}
#[test]
fn decompress_zlib_corrupt() {
    decompress_all(Codec::Zlib, "decompress_zlib_corrupt");
}
#[test]
fn decompress_zstd_corrupt() {
    decompress_all(Codec::Zstd, "decompress_zstd_corrupt");
}
#[test]
fn roundtrip_control() {
    let d = data();
    for c in [Codec::Lz4, Codec::Lzma, Codec::Bzip2, Codec::Snappy, Codec::Zlib, Codec::Zstd] {
        let comp = compress(c, &d, 5).unwrap();
        assert_eq!(decompress(c, &comp, d.len()).unwrap(), d, "{c:?}");
    }
}
#[test]
fn lz4_native_decompress_corrupt() {
    let d = data();
    let mut comp = vec![0u8; lz4::compress_bound(d.len()).unwrap() + 64];
    let n = lz4::compress(&d, &mut comp).unwrap();
    comp.truncate(n);
    let mut count = 0;
    for v in variants(&comp) {
        for dst_len in [d.len(), d.len() - 1] {
            let mut dst = vec![0u8; dst_len];
            at_guard(&v, |s| {
                let _ = lz4::decompress(s, &mut dst);
            });
            count += 1;
        }
    }
    report("lz4_native_decompress_corrupt", count);
}
#[test]
fn lz4_native_short_dst() {
    let d = data();
    let mut n = 0;
    for len in [0usize, 1, 10, d.len() / 2, d.len() - 1] {
        let mut dst = vec![0u8; len];
        at_guard(&d, |s| {
            let _ = lz4::compress(s, &mut dst);
            let _ = lz4::compress_hc(s, &mut dst, 9);
        });
        n += 1;
    }
    report("lz4_native_short_dst", n);
}
#[test]
fn snappy_native_short_dst() {
    let d = data();
    let mut n = 0;
    for len in [0usize, 1, 10, d.len() / 2, d.len() - 1] {
        let mut dst = vec![0u8; len];
        at_guard(&d, |s| {
            let _ = snappy::compress(s, &mut dst);
        });
        n += 1;
    }
    let mut comp = vec![0u8; snappy::max_compressed_length(d.len())];
    let c = snappy::compress(&d, &mut comp).unwrap();
    comp.truncate(c);
    for v in variants(&comp) {
        for len in [d.len(), d.len() - 1, 0] {
            let mut dst = vec![0u8; len];
            at_guard(&v, |s| {
                let _ = snappy::uncompressed_length(s);
                let _ = snappy::validate(s);
                let _ = snappy::uncompress(s, &mut dst);
            });
            n += 1;
        }
    }
    report("snappy_native_short_dst", n);
}
