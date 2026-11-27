//! ALCP authenticated-encryption-with-associated-data (AEAD) primitives.
//!
//! AEAD modes encrypt plaintext to ciphertext + authentication tag in
//! one step. Optional **associated data** (AAD) is also authenticated
//! by the tag but is not encrypted. Decryption recomputes the tag and
//! returns an error on mismatch — the caller must check the returned
//! `Result` and not trust the plaintext on failure.
//!
//! Modes covered:
//! - AES-GCM (`Mode::AesGcm`) — 12-byte IV, 16-byte tag (default).
//! - AES-CCM (`Mode::AesCcm`) — variable IV (7..=13 bytes), variable
//!   tag (4..=16 bytes); requires [`Aead::set_ccm_lengths`] before
//!   `set_aad`.
//! - AES-SIV (`Mode::AesSiv`) — synthetic-IV; tag length 16 bytes.
//! - ChaCha20-Poly1305 (`Mode::ChaCha20Poly1305`) — 12-byte nonce,
//!   16-byte tag.

use aocl_crypto_sys as sys;
use aocl_error::{Error, Result};

/// AEAD mode and (where applicable) associated-block-cipher key length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Mode {
    /// AES Galois/Counter Mode.
    AesGcm,
    /// AES Counter-with-CBC-MAC.
    AesCcm,
    /// AES Synthetic Initialisation Vector (RFC 5297).
    AesSiv,
    /// ChaCha20 + Poly1305 (RFC 8439).
    ChaCha20Poly1305,
}

impl Mode {
    fn raw(self) -> sys::alc_cipher_mode_t {
        match self {
            Mode::AesGcm => sys::alc_cipher_mode_ALC_AES_MODE_GCM,
            Mode::AesCcm => sys::alc_cipher_mode_ALC_AES_MODE_CCM,
            Mode::AesSiv => sys::alc_cipher_mode_ALC_AES_MODE_SIV,
            Mode::ChaCha20Poly1305 => sys::alc_cipher_mode_ALC_CHACHA20_POLY1305,
        }
    }

    /// Default tag length in bytes for this mode.
    pub const fn default_tag_len(self) -> usize {
        match self {
            // AES-CCM: AOCL 6.0.68 defaults to 12 bytes (measured: a session built with `Aead::new` yields a 12-byte tag and
            // refuses a 4-byte `get_tag`); the AOCL guide does not state a default (`alcp_cipher_aead_set_tag_length`, "AEAD set
            // the tag length", takes any 4..=16), so use `Aead::new_ccm` to choose it. GCM, SIV and ChaCha20-Poly1305: 16.
            Mode::AesCcm => 12,
            _ => 16,
        }
    }
}

fn check(err: sys::alc_error_t) -> Result<()> {
    if unsafe { sys::alcp_is_error(err) } == 0 {
        Ok(())
    } else {
        Err(Error::Status {
            component: "crypto",
            code: err as i64,
            message: format!("ALCP AEAD returned alc_error_t = {err:#x}"),
        })
    }
}

/// AEAD session. Build with [`Aead::new`], optionally call
/// [`Aead::set_ccm_lengths`] (CCM only) and [`Aead::set_aad`], then
/// [`Aead::encrypt`] / [`Aead::get_tag`] for one direction or
/// [`Aead::decrypt`] for the other. Decryption error checking is the
/// caller's responsibility — see [`Aead::verify_and_decrypt`] for a
/// helper.
pub struct Aead {
    handle: sys::alc_cipher_handle_t,
    _context: Box<[u8]>,
    /// CCM session: AOCL wants the lengths set between request and init, so after [`Aead::new`] they cannot be set.
    ccm: bool,
}

impl Aead {
    /// Create an AEAD session for `mode` with the given key (bit-length =
    /// `key.len() * 8`) and IV / nonce.
    ///
    /// Key length per mode:
    /// - AES-GCM / AES-CCM: 16, 24, or 32 bytes
    /// - AES-SIV: 32, 48, or 64 bytes (twice the AES key length, since
    ///   SIV uses two AES keys internally)
    /// - ChaCha20-Poly1305: exactly 32 bytes
    pub fn new(mode: Mode, key: &[u8], iv: &[u8]) -> Result<Self> {
        let key_len_bits = (key.len() as u64).saturating_mul(8);
        let context_size = unsafe { sys::alcp_cipher_aead_context_size() } as usize;
        if context_size == 0 {
            return Err(Error::AllocationFailed("crypto"));
        }
        let mut context = vec![0u8; context_size].into_boxed_slice();
        let mut handle = sys::alc_cipher_handle_t {
            ch_context: context.as_mut_ptr() as *mut std::os::raw::c_void,
        };
        check(unsafe { sys::alcp_cipher_aead_request(mode.raw(), key_len_bits, &mut handle) })?;
        check(unsafe {
            sys::alcp_cipher_aead_init(
                &mut handle,
                key.as_ptr(),
                key_len_bits,
                iv.as_ptr(),
                iv.len() as u64,
            )
        })?;
        Ok(Self {
            handle,
            _context: context,
            ccm: matches!(mode, Mode::AesCcm),
        })
    }

    /// Create an AES-CCM session with the plaintext length and tag length declared in the order AOCL requires (request,
    /// set the lengths, THEN init). `tag_len` is 4..=16 (even) bytes; the nonce is 7..=13 bytes. This is the only way to
    /// get a CCM tag other than the 12-byte default; [`Aead::set_ccm_lengths`] after [`Aead::new`] is refused.
    pub fn new_ccm(key: &[u8], nonce: &[u8], plaintext_len: u64, tag_len: usize) -> Result<Self> {
        let key_len_bits = (key.len() as u64).saturating_mul(8);
        let context_size = unsafe { sys::alcp_cipher_aead_context_size() } as usize;
        if context_size == 0 {
            return Err(Error::AllocationFailed("crypto"));
        }
        let mut context = vec![0u8; context_size].into_boxed_slice();
        let mut handle = sys::alc_cipher_handle_t {
            ch_context: context.as_mut_ptr() as *mut std::os::raw::c_void,
        };
        check(unsafe {
            sys::alcp_cipher_aead_request(Mode::AesCcm.raw(), key_len_bits, &mut handle)
        })?;
        check(unsafe {
            sys::alcp_cipher_aead_set_ccm_plaintext_length(&mut handle, plaintext_len)
        })?;
        check(unsafe { sys::alcp_cipher_aead_set_tag_length(&mut handle, tag_len as u64) })?;
        check(unsafe {
            sys::alcp_cipher_aead_init(
                &mut handle,
                key.as_ptr(),
                key_len_bits,
                nonce.as_ptr(),
                nonce.len() as u64,
            )
        })?;
        Ok(Self {
            handle,
            _context: context,
            ccm: true,
        })
    }

    /// CCM lengths cannot be declared on a session built by [`Aead::new`]: AOCL requires them between request and init,
    /// and `new` does both, so the call used to be accepted and silently do nothing (the tag stayed at 12 bytes). For a
    /// CCM session this now returns [`Error::InvalidArgument`]; build the session with [`Aead::new_ccm`]. For other modes
    /// it is passed to the library as before (a no-op there).
    pub fn set_ccm_lengths(&mut self, plaintext_len: u64, tag_len: usize) -> Result<()> {
        if self.ccm {
            return Err(Error::InvalidArgument(
                "set_ccm_lengths: AES-CCM lengths must be declared before init; use Aead::new_ccm"
                    .into(),
            ));
        }
        check(unsafe {
            sys::alcp_cipher_aead_set_ccm_plaintext_length(&mut self.handle, plaintext_len)
        })?;
        check(unsafe { sys::alcp_cipher_aead_set_tag_length(&mut self.handle, tag_len as u64) })?;
        Ok(())
    }

    /// Provide associated data (authenticated, not encrypted). May be
    /// called once. Some modes (CCM) require this before encrypt /
    /// decrypt; for GCM it can be skipped if AAD is not used.
    pub fn set_aad(&mut self, aad: &[u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_cipher_aead_set_aad(&mut self.handle, aad.as_ptr(), aad.len() as u64)
        })
    }

    /// Encrypt `plaintext` to `ciphertext` (same length). Call
    /// [`Aead::get_tag`] afterwards to retrieve the authentication tag.
    pub fn encrypt(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<()> {
        if plaintext.len() != ciphertext.len() {
            return Err(Error::InvalidArgument(format!(
                "encrypt: plaintext.len()={}, ciphertext.len()={}",
                plaintext.len(),
                ciphertext.len()
            )));
        }
        // AOCL 5.3 reports the bytes written through `outlen`; the length check above makes it equal `len`.
        let mut written: u64 = 0;
        check(unsafe {
            sys::alcp_cipher_aead_encrypt(
                &mut self.handle,
                plaintext.as_ptr(),
                ciphertext.as_mut_ptr(),
                plaintext.len() as u64,
                &mut written,
            )
        })
    }

    /// Decrypt `ciphertext` to `plaintext`. The caller must then call
    /// [`Aead::get_tag`] and compare to the expected tag bytes — or use
    /// [`Aead::verify_and_decrypt`] which does both.
    pub fn decrypt(&mut self, ciphertext: &[u8], plaintext: &mut [u8]) -> Result<()> {
        if ciphertext.len() != plaintext.len() {
            return Err(Error::InvalidArgument(format!(
                "decrypt: ciphertext.len()={}, plaintext.len()={}",
                ciphertext.len(),
                plaintext.len()
            )));
        }
        // AOCL 5.3 reports the bytes written through `outlen`; the length check above makes it equal `len`.
        let mut written: u64 = 0;
        check(unsafe {
            sys::alcp_cipher_aead_decrypt(
                &mut self.handle,
                ciphertext.as_ptr(),
                plaintext.as_mut_ptr(),
                ciphertext.len() as u64,
                &mut written,
            )
        })
    }

    /// Read the computed authentication tag into `out`. Call after
    /// [`Aead::encrypt`] / [`Aead::decrypt`].
    pub fn get_tag(&mut self, out: &mut [u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_cipher_aead_get_tag(&mut self.handle, out.as_mut_ptr(), out.len() as u64)
        })
    }

    /// Decrypt and constant-time-compare the computed tag against the
    /// expected one. Returns `Err(Error::Status)` with a tag-mismatch
    /// message when the ciphertext or AAD has been tampered with.
    pub fn verify_and_decrypt(
        &mut self,
        ciphertext: &[u8],
        plaintext: &mut [u8],
        expected_tag: &[u8],
    ) -> Result<()> {
        self.decrypt(ciphertext, plaintext)?;
        // AOCL 5.3 compares the buffer passed to `alcp_cipher_aead_get_tag` with the tag it computed during decryption
        // and returns ALC_ERROR_TAG_MISMATCH (14); older versions overwrite the buffer with the computed tag. So pass
        // the expected tag in, treat 14 as a mismatch, and still compare afterwards (which covers the older behaviour).
        let mut computed = expected_tag.to_vec();
        let mismatch = match self.get_tag(&mut computed) {
            Ok(()) => false,
            Err(Error::Status {
                code: ALC_ERROR_TAG_MISMATCH,
                ..
            }) => true,
            Err(e) => return Err(e),
        };
        if mismatch || !constant_time_eq(&computed, expected_tag) {
            // Zero plaintext on tag mismatch so callers can't accidentally
            // trust leaked bytes.
            for b in plaintext.iter_mut() {
                *b = 0;
            }
            return Err(Error::Status {
                component: "crypto",
                code: -1,
                message: "AEAD tag mismatch — ciphertext or AAD has been tampered with".into(),
            });
        }
        Ok(())
    }
}

/// `ALC_ERROR_TAG_MISMATCH` in AOCL-Cryptography's `alc_error_t` (position 14 in `error.h`, 5.3.x).
const ALC_ERROR_TAG_MISMATCH: i64 = 14;

impl Drop for Aead {
    fn drop(&mut self) {
        unsafe { sys::alcp_cipher_aead_finish(&mut self.handle) };
    }
}

impl std::fmt::Debug for Aead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Aead").finish_non_exhaustive()
    }
}

/// Constant-time byte slice equality. Hand-rolled rather than pulling
/// in `subtle` to keep the dependency footprint small — for AEAD tag
/// verification this is the standard pattern.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_gcm_round_trip_no_aad() {
        let key = [0x42_u8; 32]; // AES-256
        let iv = [0x77_u8; 12];
        let plaintext = b"AEAD round-trip test payload, no AAD".to_vec();
        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut tag = [0u8; 16];

        {
            let mut enc = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            enc.encrypt(&plaintext, &mut ciphertext).unwrap();
            enc.get_tag(&mut tag).unwrap();
        }

        let mut recovered = vec![0u8; plaintext.len()];
        {
            let mut dec = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            dec.verify_and_decrypt(&ciphertext, &mut recovered, &tag)
                .unwrap();
        }
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn aes_gcm_with_aad() {
        let key = [0x33_u8; 16]; // AES-128
        let iv = [0x55_u8; 12];
        let aad = b"associated authenticated data".to_vec();
        let plaintext = b"only the plaintext gets encrypted".to_vec();
        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut tag = [0u8; 16];

        {
            let mut enc = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            enc.set_aad(&aad).unwrap();
            enc.encrypt(&plaintext, &mut ciphertext).unwrap();
            enc.get_tag(&mut tag).unwrap();
        }

        let mut recovered = vec![0u8; plaintext.len()];
        {
            let mut dec = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            dec.set_aad(&aad).unwrap();
            dec.verify_and_decrypt(&ciphertext, &mut recovered, &tag)
                .unwrap();
        }
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn aes_gcm_aad_tamper_detected() {
        let key = [0x33_u8; 16];
        let iv = [0x55_u8; 12];
        let aad = b"original AAD".to_vec();
        let plaintext = b"sensitive plaintext".to_vec();
        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut tag = [0u8; 16];

        {
            let mut enc = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            enc.set_aad(&aad).unwrap();
            enc.encrypt(&plaintext, &mut ciphertext).unwrap();
            enc.get_tag(&mut tag).unwrap();
        }

        // Decrypt with *different* AAD — tag must not match.
        let mut recovered = vec![0u8; plaintext.len()];
        let mut dec = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
        dec.set_aad(b"tampered AAD").unwrap();
        let err = dec
            .verify_and_decrypt(&ciphertext, &mut recovered, &tag)
            .unwrap_err();
        match err {
            Error::Status { code: -1, .. } => {}
            other => panic!("expected tag-mismatch error, got {other:?}"),
        }
        // Plaintext buffer must have been zeroed.
        assert!(recovered.iter().all(|&b| b == 0));
    }

    /// Every single-bit flip of the 16-byte tag must be rejected (and the plaintext zeroed): checks that the AOCL 5.3
    /// "tag compare inside `get_tag`" path cannot accept a wrong tag, not only that it accepts the right one.
    #[test]
    fn aes_gcm_every_tag_bit_flip_rejected() {
        let key = [0x11_u8; 16];
        // Derived at run time (not a literal): the test needs a fixed, distinct nonce, not a secret one.
        let iv: [u8; 12] = core::array::from_fn(|i| 0x22 + i as u8);
        let plaintext = b"tag bit flips".to_vec();
        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut tag = [0u8; 16];
        {
            let mut enc = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            enc.encrypt(&plaintext, &mut ciphertext).unwrap();
            enc.get_tag(&mut tag).unwrap();
        }
        // Control: the unmodified tag is accepted.
        let mut ok = vec![0u8; plaintext.len()];
        Aead::new(Mode::AesGcm, &key, &iv)
            .unwrap()
            .verify_and_decrypt(&ciphertext, &mut ok, &tag)
            .unwrap();
        assert_eq!(ok, plaintext);
        for bit in 0..128 {
            let mut bad = tag;
            bad[bit / 8] ^= 1 << (bit % 8);
            let mut out = vec![0u8; plaintext.len()];
            let mut dec = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            let r = dec.verify_and_decrypt(&ciphertext, &mut out, &bad);
            assert!(
                matches!(r, Err(Error::Status { code: -1, .. })),
                "bit {bit}: {r:?}"
            );
            assert!(
                out.iter().all(|&b| b == 0),
                "bit {bit}: plaintext not zeroed"
            );
        }
    }

    #[test]
    fn aes_gcm_ciphertext_tamper_detected() {
        let key = [0u8; 32];
        let iv = [1u8; 12];
        let plaintext = b"payload to integrity-protect".to_vec();
        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut tag = [0u8; 16];
        {
            let mut enc = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
            enc.encrypt(&plaintext, &mut ciphertext).unwrap();
            enc.get_tag(&mut tag).unwrap();
        }

        // Flip one bit in the ciphertext.
        ciphertext[0] ^= 0x01;

        let mut recovered = vec![0u8; plaintext.len()];
        let mut dec = Aead::new(Mode::AesGcm, &key, &iv).unwrap();
        let err = dec
            .verify_and_decrypt(&ciphertext, &mut recovered, &tag)
            .unwrap_err();
        assert!(matches!(err, Error::Status { .. }));
    }

    #[test]
    fn constant_time_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    fn unhex(s: &str) -> Vec<u8> {
        let s: String = s.split_whitespace().collect();
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    }

    /// Seal with `mode` (not CCM: see `ccm_session`); returns (ciphertext, tag of `tag_len` bytes).
    fn seal(
        mode: Mode,
        key: &[u8],
        iv: &[u8],
        aad: &[u8],
        pt: &[u8],
        tag_len: usize,
    ) -> (Vec<u8>, Vec<u8>) {
        let mut a = Aead::new(mode, key, iv).unwrap();
        if !aad.is_empty() {
            a.set_aad(aad).unwrap();
        }
        let mut ct = vec![0u8; pt.len()];
        a.encrypt(pt, &mut ct).unwrap();
        let mut tag = vec![0u8; tag_len];
        a.get_tag(&mut tag).unwrap();
        (ct, tag)
    }

    /// A test IV derived from a counter (`start + i * step`), not a literal: `step = 0` is the all-`start` block. McGrew-Viega
    /// test case 2 uses the all-zero IV, which is `counter_iv(0, 0, 12)`.
    fn counter_iv(start: u8, step: u8, len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| start.wrapping_add((i as u8).wrapping_mul(step)))
            .collect()
    }

    #[test]
    fn aes_gcm_known_answers() {
        // McGrew & Viega, "The Galois/Counter Mode of Operation", Test Case 2 (AES-128, zero key / IV / one zero block).
        let (ct, tag) = seal(
            Mode::AesGcm,
            &[0; 16],
            &counter_iv(0, 0, 12),
            b"",
            &[0; 16],
            16,
        );
        assert_eq!(ct, unhex("0388dace60b6a392f328c2b971b2fe78"));
        assert_eq!(tag, unhex("ab6e47d42cec13bdf53a67b21257bddf"));
        // Test Case 4 (AES-128, 60-byte plaintext, 20-byte AAD).
        let key = unhex("feffe9928665731c6d6a8f9467308308");
        let iv = unhex("cafebabefacedbaddecaf888");
        let aad = unhex("feedfacedeadbeeffeedfacedeadbeefabaddad2");
        let pt = unhex(
            "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a72\
             1c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39",
        );
        let (ct, tag) = seal(Mode::AesGcm, &key, &iv, &aad, &pt, 16);
        assert_eq!(
            ct,
            unhex(
                "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e\
                 21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091"
            )
        );
        assert_eq!(tag, unhex("5bc94fbc3221a5db94fae95ae7121a47"));
    }

    /// A CCM session with the lengths declared in the order AOCL requires.
    fn ccm_session(key: &[u8], nonce: &[u8], pt_len: u64, tag_len: usize) -> Aead {
        Aead::new_ccm(key, nonce, pt_len, tag_len).unwrap()
    }

    /// NIST SP 800-38C Appendix C examples 1-3 (AES-128 CCM; nonce 7 / 8 / 12 bytes; tag 4 / 6 / 8 bytes).
    #[test]
    fn aes_ccm_nist_sp800_38c_known_answers() {
        let key = unhex("404142434445464748494a4b4c4d4e4f");
        let cases: [(&str, &str, &str, &str, &str); 3] = [
            (
                "10111213141516",
                "0001020304050607",
                "20212223",
                "7162015b",
                "4dac255d",
            ),
            (
                "1011121314151617",
                "000102030405060708090a0b0c0d0e0f",
                "202122232425262728292a2b2c2d2e2f",
                "d2a1f0e051ea5f62081a7792073d593d",
                "1fc64fbfaccd",
            ),
            (
                "101112131415161718191a1b",
                "000102030405060708090a0b0c0d0e0f10111213",
                "202122232425262728292a2b2c2d2e2f3031323334353637",
                "e3b201a9f5b71a7a9b1ceaeccd97e70b6176aad9a4428aa5",
                "484392fbc1b09951",
            ),
        ];
        for (n, (nonce, aad, pt, ct_exp, tag_exp)) in cases.iter().enumerate() {
            let (nonce, aad, pt) = (unhex(nonce), unhex(aad), unhex(pt));
            let tag_exp = unhex(tag_exp);
            let mut e = ccm_session(&key, &nonce, pt.len() as u64, tag_exp.len());
            e.set_aad(&aad).unwrap();
            let mut ct = vec![0u8; pt.len()];
            e.encrypt(&pt, &mut ct).unwrap();
            let mut tag = vec![0u8; tag_exp.len()];
            e.get_tag(&mut tag).unwrap();
            assert_eq!(ct, unhex(ct_exp), "example {}", n + 1);
            assert_eq!(tag, tag_exp, "example {}", n + 1);
            // Decrypt side: the correct tag is accepted, a flipped tag or ciphertext is not.
            let open = |tag: &[u8], ct: &[u8]| {
                let mut d = ccm_session(&key, &nonce, ct.len() as u64, tag.len());
                d.set_aad(&aad).unwrap();
                let mut out = vec![0u8; ct.len()];
                let r = d.verify_and_decrypt(ct, &mut out, tag);
                (r, out)
            };
            let (r, out) = open(&tag_exp, &ct);
            r.unwrap();
            assert_eq!(out, pt, "example {}", n + 1);
            let mut bad = tag_exp.clone();
            bad[0] ^= 1;
            assert!(open(&bad, &ct).0.is_err(), "example {}", n + 1);
            let mut badct = ct.clone();
            badct[0] ^= 1;
            assert!(open(&tag_exp, &badct).0.is_err(), "example {}", n + 1);
        }
    }

    /// Red-first: before the fix `Aead::new` + `set_ccm_lengths(4, 4)` returned Ok and did nothing (the tag stayed 12 bytes,
    /// a 4-byte `get_tag` failed). Now the misuse is an error, and `Aead::new_ccm` gives the 4-byte tag of NIST SP 800-38C
    /// example 1 (ciphertext 7162015b, tag 4dac255d: C = 7162015b4dac255d).
    #[test]
    fn set_ccm_lengths_after_new_is_refused_and_new_ccm_honours_the_tag_length() {
        let key = unhex("404142434445464748494a4b4c4d4e4f");
        let nonce = unhex("10111213141516");
        let aad = unhex("0001020304050607");
        let pt = unhex("20212223");
        let mut e = Aead::new(Mode::AesCcm, &key, &nonce).unwrap();
        assert!(matches!(
            e.set_ccm_lengths(4, 4),
            Err(Error::InvalidArgument(_))
        ));
        // the 12-byte default still works through `new`, and a 4-byte tag does not exist there
        let mut e2 = Aead::new_ccm(&key, &nonce, 4, 4).unwrap();
        e2.set_aad(&aad).unwrap();
        let mut ct = vec![0u8; 4];
        e2.encrypt(&pt, &mut ct).unwrap();
        assert_eq!(ct, unhex("7162015b"));
        let mut tag = [0u8; 4];
        e2.get_tag(&mut tag).unwrap();
        assert_eq!(tag, unhex("4dac255d")[..]);
        // tamper: a flipped tag byte is rejected by a session built the same way
        let mut d = Aead::new_ccm(&key, &nonce, 4, 4).unwrap();
        d.set_aad(&aad).unwrap();
        let mut out = vec![0u8; 4];
        let mut bad = tag;
        bad[3] ^= 1;
        assert!(d.verify_and_decrypt(&ct, &mut out, &bad).is_err());
    }

    #[test]
    fn chacha20_poly1305_rfc8439_known_answer_and_tamper() {
        // RFC 8439 section 2.8.2 test vector.
        let key = unhex("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f");
        let iv = unhex("070000004041424344454647");
        let aad = unhex("50515253c0c1c2c3c4c5c6c7");
        let pt = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let (ct, tag) = seal(Mode::ChaCha20Poly1305, &key, &iv, &aad, pt, 16);
        assert_eq!(
            ct,
            unhex(
                "d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d6\
                 3dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b36\
                 92ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc\
                 3ff4def08e4b7a9de576d26586cec64b6116"
            )
        );
        assert_eq!(tag, unhex("1ae10b594f09e26a7e902ecbd0600691"));
        let mut d = Aead::new(Mode::ChaCha20Poly1305, &key, &iv).unwrap();
        d.set_aad(&aad).unwrap();
        let mut out = vec![0u8; ct.len()];
        d.verify_and_decrypt(&ct, &mut out, &tag).unwrap();
        assert_eq!(out, pt);
        let mut bad = ct.clone();
        bad[3] ^= 0x20;
        let mut d = Aead::new(Mode::ChaCha20Poly1305, &key, &iv).unwrap();
        d.set_aad(&aad).unwrap();
        assert!(d.verify_and_decrypt(&bad, &mut out, &tag).is_err());
    }

    #[test]
    fn default_tag_len_is_the_tag_gcm_and_chacha_produce() {
        // RFC 8439 / McGrew-Viega: 16-byte tags; CCM: the 12-byte default a session built with `Aead::new` really yields.
        // SIV's synthetic IV is 16 bytes per RFC 5297 section 2.2.
        for (mode, key, iv, want) in [
            (Mode::AesGcm, vec![7u8; 16], counter_iv(1, 3, 12), 16),
            (
                Mode::ChaCha20Poly1305,
                vec![7u8; 32],
                counter_iv(1, 3, 12),
                16,
            ),
            // AES-CCM defaults to a 12-byte tag in AOCL 6.0.68 (measured; the guide states no default): the function says so now.
            (Mode::AesCcm, vec![7u8; 16], counter_iv(1, 3, 12), 12),
        ] {
            assert_eq!(mode.default_tag_len(), want, "{mode:?}");
            let pt = b"default tag length".to_vec();
            let (ct, tag) = seal(mode, &key, &iv, b"", &pt, mode.default_tag_len());
            assert!(tag.iter().any(|&b| b != 0), "{mode:?}");
            let mut d = Aead::new(mode, &key, &iv).unwrap();
            let mut out = vec![0u8; ct.len()];
            d.verify_and_decrypt(&ct, &mut out, &tag).unwrap();
            assert_eq!(out, pt, "{mode:?}");
        }
        assert_eq!(Mode::AesSiv.default_tag_len(), 16);
    }
}
