# aocl-crypto-sys

Raw FFI bindings to AOCL-Cryptography (ALCP). For the safe API see [`aocl-crypto`](../aocl-crypto/).

**`static-link` needs more on the link line (Windows, AOCL 6.0.68, measured; this crate does not link it).** The static `alcp_static` library references the Windows `bcrypt` library (`BCryptGenRandom`) and OpenSSL's libcrypto (`BN_*`, `EVP_*`, `ERR_*`, `OSSL_PARAM_*`): add `bcrypt.lib` and an OpenSSL 3 libcrypto import/static library; neither is in the AOCL install. All 89 declared functions resolve in the dynamic build.

Dual-licensed under MIT or Apache-2.0.
