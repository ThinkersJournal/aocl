//! Guard-page harness for the aocl-crypto safe functions (RSA, AEAD, cipher, EC), hand-written. For every function that takes a caller buffer or
//! an input whose length the library derives from the key or mode, the case passes that argument ONE BYTE SHORT of the contractual size, at the
//! end of a committed page before a PAGE_NOACCESS page (a fault = HOLE). Each case prints `RESULT <name> refused|accepted`; controls
//! (right sizes) assert success. One process per test (tools/safe-api-harness.py). Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_crypto::ec::{Curve, EcDh};
use aocl_crypto::aead::{Aead, Mode as AeadMode};
use aocl_crypto::cipher::{Cipher, Mode as CipherMode};
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

fn report<T>(name: &str, r: Result<T, aocl_error::Error>) {
    println!("RESULT {name} {}", if r.is_err() { "refused" } else { "accepted" });
}
macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            report(stringify!($name), $body);
        }
    };
}
const KEY: [u8; 32] = [7u8; 32];
const IV: [u8; 16] = [3u8; 16];
fn cipher_ctl(m: CipherMode, key: usize, iv: usize, len: usize) -> Result<(), aocl_error::Error> {
    let mut c = Cipher::new(m, &KEY[..key], &IV[..iv])?;
    let pt = vec![1u8; len];
    let mut ct = vec![0u8; len];
    c.encrypt(&pt, &mut ct)
}
#[test]
fn cipher_controls_control() {
    for m in [CipherMode::AesCbc, CipherMode::AesCtr, CipherMode::AesOfb, CipherMode::AesCfb] {
        cipher_ctl(m, 16, 16, 32).unwrap();
    }
    cipher_ctl(CipherMode::ChaCha20, 32, 16, 32).unwrap();
}
case!(cipher_cbc_short_iv, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 15, |iv| { c = Cipher::new(CipherMode::AesCbc, &KEY[..16], iv).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_cbc_short_key, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&KEY, 15, |k| { c = Cipher::new(CipherMode::AesCbc, k, &IV).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_ctr_short_iv, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 15, |iv| { c = Cipher::new(CipherMode::AesCtr, &KEY[..16], iv).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_ctr_short_key, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&KEY, 15, |k| { c = Cipher::new(CipherMode::AesCtr, k, &IV).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_ofb_short_iv, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 15, |iv| { c = Cipher::new(CipherMode::AesOfb, &KEY[..16], iv).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_ofb_short_key, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&KEY, 15, |k| { c = Cipher::new(CipherMode::AesOfb, k, &IV).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_cfb_short_iv, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 15, |iv| { c = Cipher::new(CipherMode::AesCfb, &KEY[..16], iv).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_cfb_short_key, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&KEY, 15, |k| { c = Cipher::new(CipherMode::AesCfb, k, &IV).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_chacha20_short_iv, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 15, |iv| { c = Cipher::new(CipherMode::ChaCha20, &KEY[..32], iv).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_chacha20_short_key, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&KEY, 31, |k| { c = Cipher::new(CipherMode::ChaCha20, k, &IV).and_then(|mut c| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; c.encrypt(&pt, &mut ct) }); }); c });
case!(cipher_cbc_partial_block, { cipher_ctl_p(CipherMode::AesCbc, 15) });
case!(cipher_ecb_partial_block, { cipher_ctl_p(CipherMode::AesEcb, 15) });
fn cipher_ctl_p(m: CipherMode, len: usize) -> Result<(), aocl_error::Error> {
    let mut c = Cipher::new(m, &KEY[..16], &IV)?;
    let src = vec![1u8; len];
    at_guard(&src, len, |pt| at_guard(&vec![0u8; len], len, |ct| c.encrypt(pt, ct)))
}
// AEAD
fn aead_ctl() -> Result<(), aocl_error::Error> {
    let mut a = Aead::new(AeadMode::AesGcm, &KEY[..16], &IV[..12])?;
    let pt = vec![1u8; 32];
    let mut ct = vec![0u8; 32];
    a.encrypt(&pt, &mut ct)?;
    let mut tag = [0u8; 16];
    a.get_tag(&mut tag)
}
#[test]
fn aead_gcm_control() {
    aead_ctl().unwrap();
}
case!(aead_gcm_short_iv, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 11, |iv| { c = Aead::new(AeadMode::AesGcm, &KEY[..16], iv).and_then(|mut a| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; a.encrypt(&pt, &mut ct) }); }); c });
case!(aead_gcm_short_key, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&KEY, 15, |k| { c = Aead::new(AeadMode::AesGcm, k, &IV[..12]).and_then(|mut a| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; a.encrypt(&pt, &mut ct) }); }); c });
case!(aead_gcm_get_tag_short, { let mut a = Aead::new(AeadMode::AesGcm, &KEY[..16], &IV[..12]).unwrap(); let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; a.encrypt(&pt, &mut ct).unwrap(); at_guard(&[0u8; 16], 15, |t| a.get_tag(t)) });
case!(aead_gcm_verify_short_tag, { let mut a = Aead::new(AeadMode::AesGcm, &KEY[..16], &IV[..12]).unwrap(); let ct = vec![0u8; 32]; let mut pt = vec![0u8; 32]; at_guard(&[0u8; 16], 15, |t| a.verify_and_decrypt(&ct, &mut pt, t)) });
case!(aead_ccm_short_nonce, { let mut c = Err(aocl_error::Error::InvalidArgument(String::new())); at_guard(&IV, 6, |n| { c = Aead::new_ccm(&KEY[..16], n, 32, 16).and_then(|mut a| { let pt = vec![1u8; 32]; let mut ct = vec![0u8; 32]; a.encrypt(&pt, &mut ct) }); }); c });
