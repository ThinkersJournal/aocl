# aocl-crypto

Safe Rust wrappers for AOCL-Cryptography (ALCP).

Built on top of [`aocl-crypto-sys`](../aocl-crypto-sys/).

Currently exposes the digest API (MD5, SHA-1, SHA-2 224/256/384/512, SHA-3 224/256/384/512). Cipher / AEAD / MAC / RSA / EC will follow.

Dual-licensed under MIT or Apache-2.0.

**Yanked versions:** 0.1.0, 0.2.2, 0.2.4 and 0.2.5 are yanked: `Rsa::set_private_key` passed the wrong component size and did not check the component lengths (out-of-bounds read, failed or overrunning private-key operations); `Aead::set_ccm_lengths` after `Aead::new` silently did nothing; `Mode::default_tag_len` said 16 for AES-CCM (12). Fixed in 0.2.28.
