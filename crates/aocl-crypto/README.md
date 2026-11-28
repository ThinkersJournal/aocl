# aocl-crypto

Safe Rust wrappers for AOCL-Cryptography (ALCP).

Built on top of [`aocl-crypto-sys`](../aocl-crypto-sys/).

Currently exposes the digest API (MD5, SHA-1, SHA-2 224/256/384/512, SHA-3 224/256/384/512). Cipher / AEAD / MAC / RSA / EC will follow.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4 and 0.2.5 are yanked: `Rsa::set_private_key` passed the wrong component size and did not check the component lengths (out-of-bounds read, failed or overrunning private-key operations); `Aead::set_ccm_lengths` after `Aead::new` silently did nothing; `Mode::default_tag_len` said 16 for AES-CCM (12). Fixed in 0.2.28.

### Affected versions (safe-API memory safety)

Measured per published version with a guard-page harness built against each `.crate`: the RSA functions `encrypt`, `decrypt`, `encrypt_oaep`, `sign_pss`, `sign_pkcs1v15`, `encrypt_pkcs1v15`, `sign_hash_pss`, `sign_hash_pkcs1v15` wrote past the caller's output buffer or read past the seed / pad in 0.1.0 (source read) through 0.2.54 (7 of the 10 faulting cases measurable in 0.2.2 to 0.2.5); fixed in 0.4.0. The cipher, AEAD and EC functions were measured clean. NOT MEASURED: digest, mac (read by source only). Which of these versions are yanked is shown by crates.io, which is the authority (yanking and advisories are the maintainer's, not this file's). Per-family first / last affected version, fixed-in version, the harness, what was NOT MEASURED and the result files: [`docs/safe-api-audit/AFFECTED-VERSIONS.md`](../../docs/safe-api-audit/AFFECTED-VERSIONS.md).
