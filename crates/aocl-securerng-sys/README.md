# aocl-securerng-sys

Raw FFI declarations for AOCL-SecureRNG (`amdsecrng`: RDRAND / RDSEED). For the safe API see [`aocl-securerng`](../aocl-securerng/).

**Clean-room.** The 15 declarations are written from AMD's AOCL API Guide 6.0 (doc 68552) by a session that was given only the
guide text, with no header read (this lane read the headers earlier and did not hand-edit them). They are declared from the guide
and link-tested against AOCL 6.0.68 on Windows; all 15 were probed against the DLLs (LP64 and ILP64: identical behaviour) in
`docs/cleanroom/secrng/SECRNG-MEASUREMENT.md`, which lists the controls that were vacuous. Measured codes (not guide-stated): success 2,
failure -2, NULL output -3; the guide's `SECRNG_*` names have no values in the guide, so none are exported. Linux is unmeasured.

Dual-licensed under MIT or Apache-2.0. See NOTICE for provenance and AMD's terms.
