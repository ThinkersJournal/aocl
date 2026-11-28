//! ALCP RSA: public-key encryption / decryption and PKCS#1 v1.5 / PSS
//! signatures over 1024-bit and 2048-bit keys.
//!
//! The flow mirrors the underlying ALCP API:
//!
//! 1. [`Rsa::new`] requests a context.
//! 2. [`Rsa::set_public_key`] (with `(exponent, modulus)`) and/or
//!    [`Rsa::set_private_key`] (with the CRT components `(dp, dq, p, q,
//!    qInv, modulus)`) install the key material on the handle.
//! 3. Encrypt / decrypt with [`Rsa::encrypt`] / [`Rsa::decrypt`]
//!    (raw padding-NONE) or [`Rsa::encrypt_oaep`] /
//!    [`Rsa::decrypt_oaep`] for OAEP padding (paired with
//!    [`Rsa::add_digest`] to set the OAEP hash).
//! 4. Sign / verify with [`Rsa::sign_pss`] / [`Rsa::verify_pss`] (PSS)
//!    or the PKCS#1 v1.5 variants.

use aocl_crypto_sys as sys;
use aocl_error::{Error, Result};

use crate::digest::Mode as DigestMode;

/// Padding scheme for [`Rsa::encrypt`] / [`Rsa::decrypt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Padding {
    /// OAEP padding (must be paired with [`Rsa::add_digest`]).
    Oaep,
    /// No padding — input length must equal the key size in bytes.
    None,
}

impl Padding {
    fn raw(self) -> sys::alc_rsa_padding {
        match self {
            Padding::Oaep => sys::alc_rsa_padding_ALCP_RSA_PADDING_OAEP,
            Padding::None => sys::alc_rsa_padding_ALCP_RSA_PADDING_NONE,
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
            message: format!("ALCP RSA returned alc_error_t = {err:#x}"),
        })
    }
}

/// An RSA session.
pub struct Rsa {
    handle: sys::_alc_rsa_handle,
    _context: Box<[u8]>,
    /// The digest added with `add_digest` (its output length is the OAEP seed length).
    digest: Option<DigestMode>,
}

impl Rsa {
    /// Request a new RSA handle.
    pub fn new() -> Result<Self> {
        let context_size = unsafe { sys::alcp_rsa_context_size() } as usize;
        if context_size == 0 {
            return Err(Error::AllocationFailed("crypto"));
        }
        let mut context = vec![0u8; context_size].into_boxed_slice();
        let mut handle = sys::_alc_rsa_handle {
            context: context.as_mut_ptr() as *mut std::os::raw::c_void,
        };
        check(unsafe { sys::alcp_rsa_request(&mut handle) })?;
        Ok(Self {
            handle,
            _context: context,
            digest: None,
        })
    }

    /// Install a public key. `exponent` is typically `65537`. `modulus`
    /// is the big-endian byte representation of `n`; its length (e.g.
    /// 256 for a 2048-bit key) is also the RSA "key size" in bytes.
    pub fn set_public_key(&mut self, exponent: u64, modulus: &[u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_rsa_set_publickey(
                &mut self.handle,
                exponent,
                modulus.as_ptr(),
                modulus.len() as u64,
            )
        })
    }

    /// Install a private key in CRT form. `dp = d mod (p-1)`,
    /// `dq = d mod (q-1)`, `qinv = q⁻¹ mod p`. `modulus` is `n`.
    /// All byte slices must be the same length (key size in bytes / 2
    /// for `dp`, `dq`, `p`, `q`, `qinv`; full key size for `modulus`).
    pub fn set_private_key(
        &mut self,
        dp: &[u8],
        dq: &[u8],
        p: &[u8],
        q: &[u8],
        qinv: &[u8],
        modulus: &[u8],
    ) -> Result<()> {
        // AOCL 6.0.68 takes the size of each CRT component (half the modulus), not the modulus size its header
        // documents: passing the modulus size sets the key size to twice the modulus and every private-key
        // operation then fails (or overruns the caller's buffer). The library then reads `size` bytes from EACH of the six
        // pointers, so every slice must really be that long: checked here, before any FFI call (found in review of #46: a
        // leading-zero-trimmed `dp` shorter than `p` was an out-of-bounds read inside the call).
        let size = p.len();
        if size == 0 {
            return Err(Error::InvalidArgument(
                "set_private_key: empty CRT component".into(),
            ));
        }
        for (name, len) in [
            ("dp", dp.len()),
            ("dq", dq.len()),
            ("q", q.len()),
            ("qinv", qinv.len()),
        ] {
            if len != size {
                return Err(Error::InvalidArgument(format!(
                    "set_private_key: {name} is {len} bytes but p is {size}: every CRT component must be exactly half the modulus length"
                )));
            }
        }
        if modulus.len() != 2 * size {
            return Err(Error::InvalidArgument(format!(
                "set_private_key: modulus is {} bytes but the CRT components are {size} (expected {})",
                modulus.len(),
                2 * size
            )));
        }
        let size = size as u64;
        check(unsafe {
            sys::alcp_rsa_set_privatekey(
                &mut self.handle,
                dp.as_ptr(),
                dq.as_ptr(),
                p.as_ptr(),
                q.as_ptr(),
                qinv.as_ptr(),
                modulus.as_ptr(),
                size,
            )
        })
    }

    /// Set the digest used for OAEP padding (encrypt / decrypt) and
    /// PSS sign / verify.
    pub fn add_digest(&mut self, digest: DigestMode) -> Result<()> {
        check(unsafe { sys::alcp_rsa_add_digest(&mut self.handle, digest.raw()) })?;
        self.digest = Some(digest);
        Ok(())
    }

    /// Set the digest used by the OAEP / PSS mask-generation function.
    pub fn add_mgf(&mut self, digest: DigestMode) -> Result<()> {
        check(unsafe { sys::alcp_rsa_add_mgf(&mut self.handle, digest.raw()) })
    }

    /// Key size in bytes.
    pub fn key_size_bytes(&mut self) -> usize {
        unsafe { sys::alcp_rsa_get_key_size(&mut self.handle) as usize }
    }

    /// Encrypt with the public key. Plaintext length must equal the key
    /// size in bytes (use OAEP for shorter inputs).
    /// `buf` must hold at least one key-size block: every signature / ciphertext (and the raw plaintext of a no-padding decrypt) is
    /// written as a full block, and the library does not check the buffer (a buffer one byte short is an access violation on AOCL
    /// 6.0.68, `tests/guard_lengths.rs`).
    fn require_block(&mut self, name: &str, what: &str, len: usize) -> Result<usize> {
        let k = self.key_size_bytes();
        if k == 0 || len < k {
            return Err(Error::InvalidArgument(format!(
                "{name}: {what} is {len} bytes, the key needs {k} (set the key first)"
            )));
        }
        Ok(k)
    }

    pub fn encrypt(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<()> {
        self.require_block("encrypt", "ciphertext", ciphertext.len())?;
        check(unsafe {
            sys::alcp_rsa_publickey_encrypt(
                &mut self.handle,
                plaintext.as_ptr(),
                plaintext.len() as u64,
                ciphertext.as_mut_ptr(),
            )
        })
    }

    /// Decrypt with the private key. `padding` must match the encrypt
    /// side.
    pub fn decrypt(
        &mut self,
        padding: Padding,
        ciphertext: &[u8],
        plaintext: &mut [u8],
    ) -> Result<()> {
        self.require_block("decrypt", "plaintext", plaintext.len())?;
        check(unsafe {
            sys::alcp_rsa_privatekey_decrypt(
                &mut self.handle,
                padding.raw(),
                ciphertext.as_ptr(),
                ciphertext.len() as u64,
                plaintext.as_mut_ptr(),
            )
        })
    }

    /// Encrypt with public key + OAEP padding. `seed` must be the
    /// digest output size in bytes (e.g. 32 bytes for SHA-256). Pair
    /// with [`Rsa::add_digest`] / [`Rsa::add_mgf`].
    pub fn encrypt_oaep(
        &mut self,
        plaintext: &[u8],
        label: &[u8],
        seed: &[u8],
        ciphertext: &mut [u8],
    ) -> Result<()> {
        self.require_block("encrypt_oaep", "ciphertext", ciphertext.len())?;
        let hlen = self.digest.map_or(0, |d| d.output_len());
        if seed.len() < hlen {
            return Err(Error::InvalidArgument(format!(
                "encrypt_oaep: seed is {} bytes, the OAEP digest needs {hlen} (call add_digest first)",
                seed.len()
            )));
        }
        check(unsafe {
            sys::alcp_rsa_publickey_encrypt_oaep(
                &mut self.handle,
                plaintext.as_ptr(),
                plaintext.len() as u64,
                label.as_ptr(),
                label.len() as u64,
                seed.as_ptr(),
                ciphertext.as_mut_ptr(),
            )
        })
    }

    /// Decrypt with private key + OAEP padding. Returns the recovered
    /// plaintext length (usually shorter than the key size).
    pub fn decrypt_oaep(
        &mut self,
        ciphertext: &[u8],
        label: &[u8],
        plaintext: &mut [u8],
    ) -> Result<usize> {
        let mut text_len: u64 = plaintext.len() as u64;
        check(unsafe {
            sys::alcp_rsa_privatekey_decrypt_oaep(
                &mut self.handle,
                ciphertext.as_ptr(),
                ciphertext.len() as u64,
                label.as_ptr(),
                label.len() as u64,
                plaintext.as_mut_ptr(),
                &mut text_len,
            )
        })?;
        Ok(text_len as usize)
    }

    /// PSS sign: produce a signature over `message`. `salt` length is
    /// caller's choice (typically the digest output size).
    pub fn sign_pss(&mut self, message: &[u8], salt: &[u8], signature: &mut [u8]) -> Result<()> {
        self.require_block("sign_pss", "signature", signature.len())?;
        check(unsafe {
            sys::alcp_rsa_privatekey_sign_pss(
                &mut self.handle,
                true, // check = TRUE per ALCP convention
                message.as_ptr(),
                message.len() as u64,
                salt.as_ptr(),
                salt.len() as u64,
                signature.as_mut_ptr(),
            )
        })
    }

    /// PSS verify: returns `Ok(())` if the signature matches.
    pub fn verify_pss(&mut self, message: &[u8], signature: &[u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_rsa_publickey_verify_pss(
                &mut self.handle,
                message.as_ptr(),
                message.len() as u64,
                signature.as_ptr(),
                signature.len() as u64,
            )
        })
    }

    /// PKCS#1 v1.5 sign.
    pub fn sign_pkcs1v15(&mut self, message: &[u8], signature: &mut [u8]) -> Result<()> {
        self.require_block("sign_pkcs1v15", "signature", signature.len())?;
        check(unsafe {
            sys::alcp_rsa_privatekey_sign_pkcs1v15(
                &mut self.handle,
                true,
                message.as_ptr(),
                message.len() as u64,
                signature.as_mut_ptr(),
            )
        })
    }

    /// PKCS#1 v1.5 verify.
    pub fn verify_pkcs1v15(&mut self, message: &[u8], signature: &[u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_rsa_publickey_verify_pkcs1v15(
                &mut self.handle,
                message.as_ptr(),
                message.len() as u64,
                signature.as_ptr(),
                signature.len() as u64,
            )
        })
    }

    /// PKCS#1 v1.5 encrypt with public key. `random_pad` is a caller-
    /// provided random non-zero pad of exactly (key size − text size − 3) bytes (the `00 02 PS 00` frame around the
    /// message; AOCL reads that many bytes).
    pub fn encrypt_pkcs1v15(
        &mut self,
        plaintext: &[u8],
        random_pad: &[u8],
        ciphertext: &mut [u8],
    ) -> Result<()> {
        let k = self.require_block("encrypt_pkcs1v15", "ciphertext", ciphertext.len())?;
        // EME-PKCS1-v1_5 (RFC 8017 7.2.1): 0x00 0x02 PS 0x00 M with |PS| >= 8, so |M| <= k - 11 and the library reads k - 3 - |M| pad bytes.
        if plaintext.len() + 11 > k {
            return Err(Error::InvalidArgument(format!(
                "encrypt_pkcs1v15: plaintext is {} bytes, at most {} fit a {k}-byte key",
                plaintext.len(),
                k.saturating_sub(11)
            )));
        }
        let need = k - 3 - plaintext.len();
        if random_pad.len() < need {
            return Err(Error::InvalidArgument(format!(
                "encrypt_pkcs1v15: random_pad is {} bytes, {need} are read",
                random_pad.len()
            )));
        }
        check(unsafe {
            sys::alcp_rsa_publickey_encrypt_pkcs1v15(
                &mut self.handle,
                plaintext.as_ptr(),
                plaintext.len() as u64,
                ciphertext.as_mut_ptr(),
                random_pad.as_ptr(),
            )
        })
    }

    /// PKCS#1 v1.5 decrypt with private key. Returns the recovered
    /// plaintext length.
    pub fn decrypt_pkcs1v15(&mut self, ciphertext: &[u8], plaintext: &mut [u8]) -> Result<usize> {
        let mut text_len: u64 = plaintext.len() as u64;
        check(unsafe {
            sys::alcp_rsa_privatekey_decrypt_pkcs1v15(
                &mut self.handle,
                ciphertext.as_ptr(),
                ciphertext.len() as u64,
                plaintext.as_mut_ptr(),
                &mut text_len,
            )
        })?;
        Ok(text_len as usize)
    }

    /// Sign an already-computed digest using PSS padding.
    pub fn sign_hash_pss(&mut self, hash: &[u8], salt: &[u8], signature: &mut [u8]) -> Result<()> {
        self.require_block("sign_hash_pss", "signature", signature.len())?;
        check(unsafe {
            sys::alcp_rsa_privatekey_sign_hash_pss(
                &mut self.handle,
                hash.as_ptr(),
                hash.len() as u64,
                salt.as_ptr(),
                salt.len() as u64,
                signature.as_mut_ptr(),
            )
        })
    }

    /// Verify a PSS signature over a pre-computed digest.
    pub fn verify_hash_pss(&mut self, hash: &[u8], signature: &[u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_rsa_publickey_verify_hash_pss(
                &mut self.handle,
                hash.as_ptr(),
                hash.len() as u64,
                signature.as_ptr(),
                signature.len() as u64,
            )
        })
    }

    /// Sign an already-computed digest using PKCS#1 v1.5 padding.
    pub fn sign_hash_pkcs1v15(&mut self, hash: &[u8], signature: &mut [u8]) -> Result<()> {
        self.require_block("sign_hash_pkcs1v15", "signature", signature.len())?;
        check(unsafe {
            sys::alcp_rsa_privatekey_sign_hash_pkcs1v15(
                &mut self.handle,
                hash.as_ptr(),
                hash.len() as u64,
                signature.as_mut_ptr(),
            )
        })
    }

    /// Verify a PKCS#1 v1.5 signature over a pre-computed digest.
    pub fn verify_hash_pkcs1v15(&mut self, hash: &[u8], signature: &[u8]) -> Result<()> {
        check(unsafe {
            sys::alcp_rsa_publickey_verify_hash_pkcs1v15(
                &mut self.handle,
                hash.as_ptr(),
                hash.len() as u64,
                signature.as_ptr(),
                signature.len() as u64,
            )
        })
    }

    /// Deep-copy this RSA context into `dest`. Both handles must
    /// already be live ([`Rsa::new`]'d).
    pub fn copy_context_into(&mut self, dest: &mut Rsa) -> Result<()> {
        check(unsafe { sys::alcp_rsa_context_copy(&mut self.handle, &mut dest.handle) })
    }

    /// Borrow the raw `_alc_rsa_handle` for routines this crate
    /// doesn't yet wrap (e.g. the bignum-key setters, which take
    /// ALCP `BigNum` types).
    ///
    /// # Safety
    /// The returned pointer is only valid for the lifetime of `self`.
    pub fn as_raw(&mut self) -> *mut sys::_alc_rsa_handle {
        &mut self.handle
    }
}

/// Index of a digest mode in ALCP's RSA digest-info table. Used with
/// the OAEP / PSS / PKCS#1 v1.5 paths to size hash inputs / outputs.
pub fn digest_info_index(digest: DigestMode) -> i32 {
    unsafe { sys::alcp_rsa_get_digest_info_index(digest.raw()) }
}

/// Size in bytes of `digest`'s DER `DigestInfo` prefix (RFC 8017 section 9.2: 19 for SHA-224/256/384/512, 15 for
/// SHA-1, 18 for MD5), not the digest's output size.
pub fn digest_info_size(digest: DigestMode) -> i32 {
    unsafe { sys::alcp_rsa_get_digest_info_size(digest.raw()) }
}

impl Drop for Rsa {
    fn drop(&mut self) {
        unsafe { sys::alcp_rsa_finish(&mut self.handle) };
    }
}

impl std::fmt::Debug for Rsa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rsa").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsa_request_and_drop() {
        // Smoke: just allocating + dropping the handle should not blow up.
        let _rsa = Rsa::new().unwrap();
    }

    // Key: a 1024-bit RSA key generated with `openssl genrsa` (not by AOCL), e = 65537. Every expected ciphertext /
    // signature below was computed independently of AOCL: pure-Python RFC 8017 arithmetic (EMSA-PKCS1-v1_5 9.2, EMSA-PSS
    // 9.1.1 with MGF1-SHA256, RSAES-OAEP 7.1.1 with SHA-256, RSAES-PKCS1-v1_5 7.2.1, big-integer `pow`) over hashlib
    // SHA-256, with the fixed seed / salt / pad given in the constants (the script is not kept; verify against any
    // RFC 8017 implementation).
    const N: &str = "b7c6615dfe4ef231354f35e2edfa1df57b53810d310b713b3ec4e4bed901b762bad5e7379114f1ddaeac3ce98a82791cfdb0d97f9d1ba42958ae0053b1bde7a51c92ce7ec5128d57829f35145c166e7135a766870b05f1a58bbc81ef980e2ef114ac8e2c39c8d2a4f5fca6fcc85d4549fdf13311b805a171b75df4cbe6a539c7";
    const DP: &str = "5fec14b7adf0b74d02074eddd827702daaa09545d5171dc27041e66658c88649addcc6859df6dc81874918ed3bd4cdffb5e1f8653c5c7050157e34ae1d135a45";
    const DQ: &str = "46027dfda52392c76fe542359c25678c5a1ec7169d74a59f27673987b43db0217f80f0be478a1ff089a6aaea87e9dedee85ef9995838fbb511b28468b40bbe3d";
    const P: &str = "d946b5d329fcc5c950314613e63ef2a3f18256e42d1912f08f727d8e429d9c4d5e5535bccff7e1b53269aa3f727317f94f51db06a2451beb8f30b1360ce9d475";
    const Q: &str = "d8872a051ca5a7f0c2ff9d5b0c4966c89dac4496ecaf74ad731d2cdb453deebcfe505def0a9cb5ed1fcfec7dc666de59ec4e680c01abd14b79d87ef084769dcb";
    const QINV: &str = "a01ae2bb1b144a2fe9afdb07adb092cece0e4ec9b8b392048b5275e5cf4d8b1887004da70d4240bb7332c6b2a1f6fcfc225ac33b264a273e28fe5d93230cc2b6";
    const MSG: &str = "616263";
    const SIG_V15: &str = "083d216adc69d051bd323bd7b5b60dc67f0a990273cd8c3895cc14adcdfaf4c9cd7065083b78cd00cd193baf8c4f3387298f80436711d9d884ddba3cc6acc80197b81dc748b84c566c86dd60b6668b939794468edcc535b76ce9dca94a0de0f6cd161eda9ed0b2d7551cdfdd1574a44b6ffc6f66872287460fbe4bb0a68380ad";
    const SALT: &str = "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf";
    const SIG_PSS: &str = "a37f4634efd5746ea8eb5b840d5b23348f8c6fefdd401cec4aac7229efa8ad7ace4777d5cb9032d85a34e4c3d2acac459b5b70331549d5840f16ff5d0e864bb5394495cbf28b9571234c6e7d95bf2d17d407679852fd36b9e6fd57703ee5aac4c1326670cce39617fb46c51887296a2a4f7ce180ff3d84ca2bfbbf9ab930b2e6";
    const PT: &str = "61747461636b206174206461776e";
    const LABEL: &str = "6c626c";
    const SEED: &str = "303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f";
    const CT_OAEP: &str = "62700edbb658c1c532a7f92b411b6c2226d08bf13c2eae1da7e84c87c13081ffb392297816f13174dd6663c23fe0c11868ac9c80c96c1edd955d300abaebbf86b7c4504881563bb5d7694e2871f2a6026807d80856a91416bd29072564443fb73c9e46ca3f953d267f0052f06727d4551cc4968cbb113999c6beaee557a1ccd6";
    const PAD: &str = "12131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f80";
    const CT_V15: &str = "35304c2dddc03723d1af15ce5dafdf999651cb4b3be91d98a3eed30ac985e34a56e4e224b19e4cd65af5381750c44365d7abf8d281ceb327741bc07bce7e5a7216fab0516582fc3e5b6b1bfe96bb4fad848e1f87437270ad047f612a3508aa4a984b4923962a7443c5fc8ca0e3e1feae57a1079fc4c11cccdcf35dd06281363b";
    const RAW: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f";
    const CT_RAW: &str = "afd745ffc0d6fe0bd1aefbeabb0382fc59cf71ed8fe66d5d53a094fe431dc1f0cf753d1d0b09b4e9e9ce515eb151d66258e8ec12933648270b9257e3b224e372c88920fab40800820bbd98f65cc76c16c13635c38c0b6b077a592bf57dfc739cafff6de0fbf673c9779a218919fc77d1ddf67a9dae49223c9dd88aea6f4bf9e0";
    const MH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const SIG_V15_NODIGESTINFO: &str = "14e1644d3b2e575122dca9290ac6a14f5ee432ae3bbb326946fd06e0fc3fd8deb6e75e9d580dce8f50fd9104278acc7151441327cbb1dda851d4cd3ed3b4566593cfc55c5940d9420f07d5a2bba29a65ecf554147bf7ffe49300cdb7ceeef1293ed2f814cf55cf82f1a221aa2762836408024bcf6136f371171a49bc62522819";
    const DIGESTINFO_SHA256: &str = "3031300d060960864801650304020105000420";

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
            .collect()
    }

    fn pub_key() -> Rsa {
        let mut r = Rsa::new().unwrap();
        r.set_public_key(65537, &unhex(N)).unwrap();
        r
    }

    /// Public + private key, SHA-256 for the digest and the MGF.
    fn full_key() -> Rsa {
        let mut r = pub_key();
        r.set_private_key(
            &unhex(DP),
            &unhex(DQ),
            &unhex(P),
            &unhex(Q),
            &unhex(QINV),
            &unhex(N),
        )
        .unwrap();
        r.add_digest(DigestMode::Sha2_256).unwrap();
        r.add_mgf(DigestMode::Sha2_256).unwrap();
        r
    }

    #[test]
    fn key_size_bytes_matches_modulus() {
        let mut r = pub_key();
        assert_eq!(r.key_size_bytes(), 128); // 1024-bit modulus
    }

    #[test]
    fn digest_info_size_is_the_der_prefix_length() {
        // RFC 8017 section 9.2 note 1: the DER DigestInfo prefix is 19 bytes for SHA-224/256/384/512 (e.g.
        // 3031300d060960864801650304020105000420 for SHA-256), 15 for SHA-1 and 18 for MD5.
        for (m, n) in [
            (DigestMode::Md5, 18),
            (DigestMode::Sha1, 15),
            (DigestMode::Sha2_224, 19),
            (DigestMode::Sha2_256, 19),
            (DigestMode::Sha2_384, 19),
            (DigestMode::Sha2_512, 19),
        ] {
            assert_eq!(digest_info_size(m), n, "{m:?}");
        }
    }

    #[test]
    fn digest_info_index_distinguishes_digests() {
        // The index is a position in ALCP's table: SHA-2 family members must have distinct, valid (non-negative) entries.
        let idx: Vec<i32> = [
            DigestMode::Sha2_224,
            DigestMode::Sha2_256,
            DigestMode::Sha2_384,
            DigestMode::Sha2_512,
        ]
        .into_iter()
        .map(digest_info_index)
        .collect();
        assert!(idx.iter().all(|&i| i >= 0), "{idx:?}");
        let mut u = idx.clone();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), 4, "{idx:?}");
    }

    #[test]
    fn raw_encrypt_known_answer_and_decrypt() {
        let mut r = full_key();
        let mut ct = vec![0u8; 128];
        r.encrypt(&unhex(RAW), &mut ct).unwrap();
        assert_eq!(ct, unhex(CT_RAW));
        let mut pt = vec![0u8; 128];
        r.decrypt(Padding::None, &ct, &mut pt).unwrap();
        assert_eq!(pt, unhex(RAW));
    }

    #[test]
    fn oaep_encrypt_known_answer_and_decrypt() {
        let mut r = full_key();
        let mut ct = vec![0u8; 128];
        r.encrypt_oaep(&unhex(PT), &unhex(LABEL), &unhex(SEED), &mut ct)
            .unwrap();
        assert_eq!(ct, unhex(CT_OAEP));
        let mut pt = vec![0u8; 128];
        let n = r.decrypt_oaep(&ct, &unhex(LABEL), &mut pt).unwrap();
        assert_eq!(&pt[..n], &unhex(PT)[..]);
    }

    #[test]
    fn oaep_wrong_label_or_corrupt_ciphertext_rejected() {
        let mut r = full_key();
        let ct = unhex(CT_OAEP);
        let mut pt = vec![0u8; 128];
        assert!(r.decrypt_oaep(&ct, b"other", &mut pt).is_err());
        let mut bad = ct.clone();
        bad[64] ^= 1;
        assert!(r.decrypt_oaep(&bad, &unhex(LABEL), &mut pt).is_err());
    }

    #[test]
    fn pkcs1v15_encrypt_known_answer_and_decrypt() {
        let mut r = full_key();
        let mut ct = vec![0u8; 128];
        r.encrypt_pkcs1v15(&unhex(PT), &unhex(PAD), &mut ct)
            .unwrap();
        assert_eq!(ct, unhex(CT_V15));
        let mut pt = vec![0u8; 128];
        let n = r.decrypt_pkcs1v15(&ct, &mut pt).unwrap();
        assert_eq!(&pt[..n], &unhex(PT)[..]);
    }

    #[test]
    fn pkcs1v15_decrypt_corrupt_ciphertext_does_not_return_plaintext() {
        let mut r = full_key();
        let mut bad = unhex(CT_V15);
        bad[10] ^= 0x40;
        let mut pt = vec![0u8; 128];
        match r.decrypt_pkcs1v15(&bad, &mut pt) {
            Err(_) => {}
            Ok(n) => assert_ne!(&pt[..n.min(128)], &unhex(PT)[..]),
        }
    }

    #[test]
    fn pkcs1v15_sign_known_answer_verify_and_tamper() {
        let mut r = full_key();
        let mut sig = vec![0u8; 128];
        r.sign_pkcs1v15(&unhex(MSG), &mut sig).unwrap();
        assert_eq!(sig, unhex(SIG_V15)); // deterministic: RFC 8017 9.2 / SHA-256 DigestInfo
        r.verify_pkcs1v15(&unhex(MSG), &sig).unwrap();
        assert!(r.verify_pkcs1v15(b"abd", &sig).is_err());
        let mut bad = sig.clone();
        bad[127] ^= 1;
        assert!(r.verify_pkcs1v15(&unhex(MSG), &bad).is_err());
        // The independently computed signature verifies with a public-only handle.
        let mut p = pub_key();
        p.add_digest(DigestMode::Sha2_256).unwrap();
        p.verify_pkcs1v15(&unhex(MSG), &unhex(SIG_V15)).unwrap();
    }

    /// `sign_hash_pkcs1v15` takes the DER DigestInfo-encoded hash (the layout `digest_info_size` describes), not the bare
    /// digest: DigestInfo || H signs to exactly the EMSA-PKCS1-v1_5 signature of the message. A bare digest is padded
    /// as `00 01 FF.. 00 || H` (no DigestInfo), which is not an RFC 8017 signature (reported as a FINDING).
    #[test]
    fn pkcs1v15_hash_sign_known_answers_and_verify() {
        let mut r = full_key();
        let mut di = unhex(DIGESTINFO_SHA256);
        assert_eq!(di.len() as i32, digest_info_size(DigestMode::Sha2_256));
        di.extend(unhex(MH));
        let mut sig = vec![0u8; 128];
        r.sign_hash_pkcs1v15(&di, &mut sig).unwrap();
        assert_eq!(sig, unhex(SIG_V15));
        r.verify_hash_pkcs1v15(&di, &sig).unwrap();
        // The signature also verifies as a message signature over "abc".
        r.verify_pkcs1v15(&unhex(MSG), &sig).unwrap();
        let mut other = di.clone();
        other[30] ^= 1;
        assert!(r.verify_hash_pkcs1v15(&other, &sig).is_err());
        // Bare digest: the library pads without DigestInfo (independently computed below), and verifies its own output.
        let mut bare = vec![0u8; 128];
        r.sign_hash_pkcs1v15(&unhex(MH), &mut bare).unwrap();
        assert_eq!(bare, unhex(SIG_V15_NODIGESTINFO));
        r.verify_hash_pkcs1v15(&unhex(MH), &bare).unwrap();
    }

    #[test]
    fn pss_sign_known_answer_verify_and_tamper() {
        let mut r = full_key();
        let mut sig = vec![0u8; 128];
        r.sign_pss(&unhex(MSG), &unhex(SALT), &mut sig).unwrap();
        assert_eq!(sig, unhex(SIG_PSS)); // fixed salt makes PSS deterministic
        r.verify_pss(&unhex(MSG), &sig).unwrap();
        // A signature produced elsewhere (our independent computation) verifies too.
        r.verify_pss(&unhex(MSG), &unhex(SIG_PSS)).unwrap();
        assert!(r.verify_pss(b"abd", &sig).is_err());
        let mut bad = sig.clone();
        bad[5] ^= 0x10;
        assert!(r.verify_pss(&unhex(MSG), &bad).is_err());
    }

    #[test]
    fn pss_hash_sign_matches_message_sign_and_verifies() {
        let mut r = full_key();
        let mut sig = vec![0u8; 128];
        r.sign_hash_pss(&unhex(MH), &unhex(SALT), &mut sig).unwrap();
        assert_eq!(sig, unhex(SIG_PSS));
        r.verify_hash_pss(&unhex(MH), &sig).unwrap();
        let mut other = unhex(MH);
        other[31] ^= 0x80;
        assert!(r.verify_hash_pss(&other, &sig).is_err());
    }

    #[test]
    fn copy_context_into_gives_a_working_copy() {
        let mut src = full_key();
        let mut dst = Rsa::new().unwrap();
        src.copy_context_into(&mut dst).unwrap();
        assert_eq!(dst.key_size_bytes(), 128);
        let mut sig = vec![0u8; 128];
        dst.sign_pkcs1v15(&unhex(MSG), &mut sig).unwrap();
        assert_eq!(sig, unhex(SIG_V15));
    }

    #[test]
    fn as_raw_points_at_the_live_handle() {
        let mut r = pub_key();
        let p = r.as_raw();
        assert!(!p.is_null());
        // SAFETY: `p` borrows `r`'s own handle, valid while `r` lives; the sys call reads the key size set above.
        assert_eq!(unsafe { sys::alcp_rsa_get_key_size(p) }, 128);
    }
    /// Found in review of #46: `set_private_key` told the library every component had `p.len()` bytes without checking the
    /// others, so a leading-zero-trimmed `dp` (63 bytes) next to a 64-byte `p` was read 64 bytes long: an out-of-bounds read
    /// inside the FFI call. Mismatched lengths are now refused before the call.
    #[test]
    fn set_private_key_refuses_mismatched_component_lengths() {
        let (dp, dq, p, q, qinv, n) = (
            unhex(DP),
            unhex(DQ),
            unhex(P),
            unhex(Q),
            unhex(QINV),
            unhex(N),
        );
        let mut r = pub_key();
        let short = |v: &Vec<u8>| v[1..].to_vec();
        let cases: [(&str, Vec<Vec<u8>>); 7] = [
            (
                "dp one byte short",
                vec![
                    short(&dp),
                    dq.clone(),
                    p.clone(),
                    q.clone(),
                    qinv.clone(),
                    n.clone(),
                ],
            ),
            (
                "dq one byte short",
                vec![
                    dp.clone(),
                    short(&dq),
                    p.clone(),
                    q.clone(),
                    qinv.clone(),
                    n.clone(),
                ],
            ),
            (
                "q one byte short",
                vec![
                    dp.clone(),
                    dq.clone(),
                    p.clone(),
                    short(&q),
                    qinv.clone(),
                    n.clone(),
                ],
            ),
            (
                "qinv one byte short",
                vec![
                    dp.clone(),
                    dq.clone(),
                    p.clone(),
                    q.clone(),
                    short(&qinv),
                    n.clone(),
                ],
            ),
            (
                "p one byte short (others agree with each other)",
                vec![
                    dp.clone(),
                    dq.clone(),
                    short(&p),
                    q.clone(),
                    qinv.clone(),
                    n.clone(),
                ],
            ),
            (
                "modulus one byte short",
                vec![
                    dp.clone(),
                    dq.clone(),
                    p.clone(),
                    q.clone(),
                    qinv.clone(),
                    short(&n),
                ],
            ),
            (
                "empty components",
                vec![vec![], vec![], vec![], vec![], vec![], vec![]],
            ),
        ];
        for (what, c) in cases {
            let e = r
                .set_private_key(&c[0], &c[1], &c[2], &c[3], &c[4], &c[5])
                .unwrap_err();
            assert!(matches!(e, Error::InvalidArgument(_)), "{what}: {e:?}");
        }
        // control: the consistent key is still accepted and signs
        r.set_private_key(&dp, &dq, &p, &q, &qinv, &n).unwrap();
    }

    /// Red-first: with the old code this call raised an access violation. `dp` (63 bytes) is placed so that its last byte is the
    /// last byte of a committed page and the next page is PAGE_NOACCESS: reading 64 bytes from it faults. (The slice itself is
    /// ordinary safe Rust; only the test's memory setup is `unsafe`.) Windows only.
    #[cfg(windows)]
    #[test]
    fn set_private_key_short_dp_at_a_guard_page_is_an_error_not_an_access_violation() {
        use core::ffi::c_void;
        extern "system" {
            fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
            fn VirtualProtect(addr: *mut c_void, size: usize, new: u32, old: *mut u32) -> i32;
            fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
        }
        let (dp, dq, p, q, qinv, n) = (
            unhex(DP),
            unhex(DQ),
            unhex(P),
            unhex(Q),
            unhex(QINV),
            unhex(N),
        );
        let short = &dp[1..]; // 63 bytes
        unsafe {
            let base = VirtualAlloc(core::ptr::null_mut(), 8192, 0x3000, 0x04);
            assert!(!base.is_null());
            let mut old = 0u32;
            assert_ne!(
                VirtualProtect(
                    (base as *mut u8).add(4096) as *mut c_void,
                    4096,
                    0x01,
                    &mut old
                ),
                0
            );
            let dst = (base as *mut u8).add(4096 - short.len());
            core::ptr::copy_nonoverlapping(short.as_ptr(), dst, short.len());
            let guarded: &[u8] = core::slice::from_raw_parts(dst, short.len());
            let mut r = pub_key();
            let e = r
                .set_private_key(guarded, &dq, &p, &q, &qinv, &n)
                .unwrap_err();
            assert!(matches!(e, Error::InvalidArgument(_)), "{e:?}");
            VirtualFree(base, 0, 0x8000);
        }
    }
}
