//! Guard-page harness for the aocl-crypto safe functions (RSA, AEAD, cipher, EC), hand-written. For every function that takes a caller buffer or
//! an input whose length the library derives from the key or mode, the case passes that argument ONE BYTE SHORT of the contractual size, at the
//! end of a committed page before a PAGE_NOACCESS page (a fault = HOLE). Each case prints `RESULT <name> refused|accepted`; controls
//! (right sizes) assert success. One process per test (tools/safe-api-harness.py). RSA key: the 1024-bit test key of the rsa.rs unit tests
//! (generated with openssl, not by AOCL). Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_crypto::aead::Aead;
use aocl_crypto::cipher::Cipher;
use aocl_crypto::digest::Mode as DigestMode;
use aocl_crypto::ec::{Curve, EcDh};
use aocl_crypto::rsa::{Padding, Rsa};
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

/// A copy of `src[..n]` ending at a committed page before a PAGE_NOACCESS page.
fn at_guard<R>(src: &[u8], n: usize, f: impl FnOnce(&mut [u8]) -> R) -> R {
    assert!(n <= src.len() && n <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(VirtualProtect((base as *mut u8).add(PAGE) as *mut c_void, PAGE, 0x01, &mut old), 0);
        let start = (base as *mut u8).add(PAGE - n);
        let s = core::slice::from_raw_parts_mut(start, n);
        s.copy_from_slice(&src[..n]);
        let r = f(s);
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn report<T>(name: &str, r: Result<T, aocl_error::Error>) {
    println!("RESULT {name} {}", if r.is_err() { "refused" } else { "accepted" });
}

const N: &str = "b7c6615dfe4ef231354f35e2edfa1df57b53810d310b713b3ec4e4bed901b762bad5e7379114f1ddaeac3ce98a82791cfdb0d97f9d1ba42958ae0053b1bde7a51c92ce7ec5128d57829f35145c166e7135a766870b05f1a58bbc81ef980e2ef114ac8e2c39c8d2a4f5fca6fcc85d4549fdf13311b805a171b75df4cbe6a539c7";
const DP: &str = "5fec14b7adf0b74d02074eddd827702daaa09545d5171dc27041e66658c88649addcc6859df6dc81874918ed3bd4cdffb5e1f8653c5c7050157e34ae1d135a45";
const DQ: &str = "46027dfda52392c76fe542359c25678c5a1ec7169d74a59f27673987b43db0217f80f0be478a1ff089a6aaea87e9dedee85ef9995838fbb511b28468b40bbe3d";
const P: &str = "d946b5d329fcc5c950314613e63ef2a3f18256e42d1912f08f727d8e429d9c4d5e5535bccff7e1b53269aa3f727317f94f51db06a2451beb8f30b1360ce9d475";
const Q: &str = "d8872a051ca5a7f0c2ff9d5b0c4966c89dac4496ecaf74ad731d2cdb453deebcfe505def0a9cb5ed1fcfec7dc666de59ec4e680c01abd14b79d87ef084769dcb";
const QINV: &str = "a01ae2bb1b144a2fe9afdb07adb092cece0e4ec9b8b392048b5275e5cf4d8b1887004da70d4240bb7332c6b2a1f6fcfc225ac33b264a273e28fe5d93230cc2b6";
const MSG: &str = "616263";
const SALT: &str = "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf";
const PT: &str = "61747461636b206174206461776e";
const LABEL: &str = "6c626c";
const SEED: &str = "303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f";
const PAD: &str = "12131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f80";
const RAW: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f";
const MH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn full_key() -> Rsa {
    let mut r = Rsa::new().unwrap();
    r.set_public_key(65537, &unhex(N)).unwrap();
    r.set_private_key(&unhex(DP), &unhex(DQ), &unhex(P), &unhex(Q), &unhex(QINV), &unhex(N)).unwrap();
    r.add_digest(DigestMode::Sha2_256).unwrap();
    r.add_mgf(DigestMode::Sha2_256).unwrap();
    r
}
fn sig_pkcs1() -> Vec<u8> {
    let mut r = full_key();
    let mut s = vec![0u8; 128];
    r.sign_pkcs1v15(&unhex(MSG), &mut s).unwrap();
    s
}
fn sig_pss() -> Vec<u8> {
    let mut r = full_key();
    let mut s = vec![0u8; 128];
    r.sign_pss(&unhex(MSG), &unhex(SALT), &mut s).unwrap();
    s
}
fn ct_raw() -> Vec<u8> {
    let mut r = full_key();
    let mut c = vec![0u8; 128];
    r.encrypt(&unhex(RAW), &mut c).unwrap();
    c
}

macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            report(stringify!($name), $body);
        }
    };
}
macro_rules! control {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            $body.unwrap();
        }
    };
}

// ---------------- RSA (1024-bit key: every signature / ciphertext / output buffer is 128 bytes) ----------------
control!(rsa_controls, (|| -> Result<(), aocl_error::Error> {
    let mut r = full_key();
    let mut c = vec![0u8; 128];
    r.encrypt(&unhex(RAW), &mut c)?;
    let mut p = vec![0u8; 128];
    r.decrypt(Padding::None, &c, &mut p)?;
    r.encrypt_oaep(&unhex(PT), &unhex(LABEL), &unhex(SEED), &mut c)?;
    r.decrypt_oaep(&c, &unhex(LABEL), &mut p)?;
    let s1 = sig_pkcs1();
    r.verify_pkcs1v15(&unhex(MSG), &s1)?;
    let s2 = sig_pss();
    r.verify_pss(&unhex(MSG), &s2)?;
    let mut s = vec![0u8; 128];
    r.sign_hash_pss(&unhex(MH), &unhex(SALT), &mut s)?;
    r.verify_hash_pss(&unhex(MH), &s)?;
    r.sign_hash_pkcs1v15(&unhex(MH), &mut s)?;
    r.verify_hash_pkcs1v15(&unhex(MH), &s)?;
    r.encrypt_pkcs1v15(&unhex(PT), &unhex(PAD), &mut c)?;
    r.decrypt_pkcs1v15(&c, &mut p)?;
    Ok(())
})());
case!(rsa_encrypt_short_output, { let mut r = full_key(); let pt = unhex(RAW); at_guard(&[0u8; 128], 127, |c| r.encrypt(&pt, c)) });
case!(rsa_encrypt_short_plaintext, { let mut r = full_key(); let mut c = vec![0u8; 128]; at_guard(&unhex(RAW), 127, |p| r.encrypt(p, &mut c)) });
case!(rsa_decrypt_short_ciphertext, { let mut r = full_key(); let mut p = vec![0u8; 128]; at_guard(&ct_raw(), 127, |c| r.decrypt(Padding::None, c, &mut p)) });
case!(rsa_decrypt_short_output, { let mut r = full_key(); let c = ct_raw(); at_guard(&[0u8; 128], 127, |p| r.decrypt(Padding::None, &c, p)) });
case!(rsa_encrypt_oaep_short_output, { let mut r = full_key(); at_guard(&[0u8; 128], 127, |c| r.encrypt_oaep(&unhex(PT), &unhex(LABEL), &unhex(SEED), c)) });
case!(rsa_encrypt_oaep_short_seed, { let mut r = full_key(); let mut c = vec![0u8; 128]; at_guard(&unhex(SEED), 31, |s| r.encrypt_oaep(&unhex(PT), &unhex(LABEL), s, &mut c)) });
case!(rsa_decrypt_oaep_short_ciphertext, { let mut r = full_key(); let mut c = vec![0u8; 128]; r.encrypt_oaep(&unhex(PT), &unhex(LABEL), &unhex(SEED), &mut c).unwrap(); let mut p = vec![0u8; 128]; at_guard(&c, 127, |cc| r.decrypt_oaep(cc, &unhex(LABEL), &mut p).map(|_| ())) });
case!(rsa_decrypt_oaep_short_output, { let mut r = full_key(); let mut c = vec![0u8; 128]; r.encrypt_oaep(&unhex(PT), &unhex(LABEL), &unhex(SEED), &mut c).unwrap(); at_guard(&[0u8; 128], 127, |p| r.decrypt_oaep(&c, &unhex(LABEL), p).map(|_| ())) });
case!(rsa_sign_pss_short_signature, { let mut r = full_key(); at_guard(&[0u8; 128], 127, |s| r.sign_pss(&unhex(MSG), &unhex(SALT), s)) });
case!(rsa_verify_pss_short_signature, { let mut r = full_key(); at_guard(&sig_pss(), 127, |s| r.verify_pss(&unhex(MSG), s)) });
case!(rsa_sign_pkcs1v15_short_signature, { let mut r = full_key(); at_guard(&[0u8; 128], 127, |s| r.sign_pkcs1v15(&unhex(MSG), s)) });
case!(rsa_verify_pkcs1v15_short_signature, { let mut r = full_key(); at_guard(&sig_pkcs1(), 127, |s| r.verify_pkcs1v15(&unhex(MSG), s)) });
case!(rsa_encrypt_pkcs1v15_short_output, { let mut r = full_key(); at_guard(&[0u8; 128], 127, |c| r.encrypt_pkcs1v15(&unhex(PT), &unhex(PAD), c)) });
case!(rsa_encrypt_pkcs1v15_short_pad, { let mut r = full_key(); let mut c = vec![0u8; 128]; let pad = unhex(PAD); let need = 128 - 3 - unhex(PT).len(); at_guard(&pad, need.min(pad.len()) - 1, |p| r.encrypt_pkcs1v15(&unhex(PT), p, &mut c)) });
case!(rsa_decrypt_pkcs1v15_short_ciphertext, { let mut r = full_key(); let mut c = vec![0u8; 128]; r.encrypt_pkcs1v15(&unhex(PT), &unhex(PAD), &mut c).unwrap(); let mut p = vec![0u8; 128]; at_guard(&c, 127, |cc| r.decrypt_pkcs1v15(cc, &mut p).map(|_| ())) });
case!(rsa_decrypt_pkcs1v15_short_output, { let mut r = full_key(); let mut c = vec![0u8; 128]; r.encrypt_pkcs1v15(&unhex(PT), &unhex(PAD), &mut c).unwrap(); at_guard(&[0u8; 128], 127, |p| r.decrypt_pkcs1v15(&c, p).map(|_| ())) });
case!(rsa_sign_hash_pss_short_hash, { let mut r = full_key(); let mut s = vec![0u8; 128]; at_guard(&unhex(MH), 31, |h| r.sign_hash_pss(h, &unhex(SALT), &mut s)) });
case!(rsa_sign_hash_pss_short_signature, { let mut r = full_key(); at_guard(&[0u8; 128], 127, |s| r.sign_hash_pss(&unhex(MH), &unhex(SALT), s)) });
case!(rsa_verify_hash_pss_short_hash, { let mut r = full_key(); let mut s = vec![0u8; 128]; r.sign_hash_pss(&unhex(MH), &unhex(SALT), &mut s).unwrap(); at_guard(&unhex(MH), 31, |h| r.verify_hash_pss(h, &s)) });
case!(rsa_verify_hash_pss_short_signature, { let mut r = full_key(); let mut s = vec![0u8; 128]; r.sign_hash_pss(&unhex(MH), &unhex(SALT), &mut s).unwrap(); at_guard(&s, 127, |ss| r.verify_hash_pss(&unhex(MH), ss)) });
case!(rsa_sign_hash_pkcs1v15_short_hash, { let mut r = full_key(); let mut s = vec![0u8; 128]; at_guard(&unhex(MH), 31, |h| r.sign_hash_pkcs1v15(h, &mut s)) });
case!(rsa_sign_hash_pkcs1v15_short_signature, { let mut r = full_key(); at_guard(&[0u8; 128], 127, |s| r.sign_hash_pkcs1v15(&unhex(MH), s)) });
case!(rsa_verify_hash_pkcs1v15_short_hash, { let mut r = full_key(); let mut s = vec![0u8; 128]; r.sign_hash_pkcs1v15(&unhex(MH), &mut s).unwrap(); at_guard(&unhex(MH), 31, |h| r.verify_hash_pkcs1v15(h, &s)) });
case!(rsa_verify_hash_pkcs1v15_short_signature, { let mut r = full_key(); let mut s = vec![0u8; 128]; r.sign_hash_pkcs1v15(&unhex(MH), &mut s).unwrap(); at_guard(&s, 127, |ss| r.verify_hash_pkcs1v15(&unhex(MH), ss)) });
