//! Safe wrappers for AOCL-SecureRNG.
//!
//! Wraps the AMD secure-RNG entry points around the x86 `RDRAND`
//! (already conditioned, suitable for general-purpose random) and
//! `RDSEED` (raw entropy, suitable for seeding cryptographic generators)
//! instructions.

#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use aocl_error::{Error, Result};
use aocl_securerng_sys as sys;

const SECRNG_SUCCESS: i32 = 2;
const SECRNG_SUPPORTED: i32 = 1;
const SECRNG_NOT_SUPPORTED: i32 = -1;
const SECRNG_FAILURE: i32 = -2;
const SECRNG_INVALID_INPUT: i32 = -3;

/// Default retry budget AOCL recommends for transient failures of `RDRAND` /
/// `RDSEED`.
pub const DEFAULT_RETRY_COUNT: u32 = 10;

fn check_status(component: &'static str, raw: i32) -> Result<()> {
    if raw == SECRNG_SUCCESS {
        return Ok(());
    }
    let message = match raw {
        SECRNG_NOT_SUPPORTED => "instruction not supported on this CPU",
        SECRNG_FAILURE => "RNG instruction failed after all retries",
        SECRNG_INVALID_INPUT => "invalid input",
        _ => "unknown SecureRNG status",
    }
    .to_string();
    Err(Error::Status {
        component,
        code: raw as i64,
        message,
    })
}

/// Returns `true` if the host CPU exposes the `RDRAND` instruction.
pub fn is_rdrand_supported() -> bool {
    unsafe { sys::is_RDRAND_supported() == SECRNG_SUPPORTED }
}

/// Returns `true` if the host CPU exposes the `RDSEED` instruction.
pub fn is_rdseed_supported() -> bool {
    unsafe { sys::is_RDSEED_supported() == SECRNG_SUPPORTED }
}

/// Read a single random `u16` from `RDRAND`.
pub fn rdrand_u16(retry_count: u32) -> Result<u16> {
    let mut v: u16 = 0;
    let raw = unsafe { sys::get_rdrand16u(&mut v, retry_count) };
    check_status("securerng", raw)?;
    Ok(v)
}

/// Read a single random `u32` from `RDRAND`.
pub fn rdrand_u32(retry_count: u32) -> Result<u32> {
    let mut v: u32 = 0;
    let raw = unsafe { sys::get_rdrand32u(&mut v, retry_count) };
    check_status("securerng", raw)?;
    Ok(v)
}

/// Read a single random `u64` from `RDRAND`.
pub fn rdrand_u64(retry_count: u32) -> Result<u64> {
    let mut v: u64 = 0;
    let raw = unsafe { sys::get_rdrand64u(&mut v, retry_count) };
    check_status("securerng", raw)?;
    Ok(v)
}

/// Read a single random `u16` from `RDSEED`.
pub fn rdseed_u16(retry_count: u32) -> Result<u16> {
    let mut v: u16 = 0;
    let raw = unsafe { sys::get_rdseed16u(&mut v, retry_count) };
    check_status("securerng", raw)?;
    Ok(v)
}

/// Read a single random `u32` from `RDSEED`.
pub fn rdseed_u32(retry_count: u32) -> Result<u32> {
    let mut v: u32 = 0;
    let raw = unsafe { sys::get_rdseed32u(&mut v, retry_count) };
    check_status("securerng", raw)?;
    Ok(v)
}

/// Read a single random `u64` from `RDSEED`.
pub fn rdseed_u64(retry_count: u32) -> Result<u64> {
    let mut v: u64 = 0;
    let raw = unsafe { sys::get_rdseed64u(&mut v, retry_count) };
    check_status("securerng", raw)?;
    Ok(v)
}

/// Fill `out` with random bytes from `RDRAND`.
pub fn rdrand_bytes(out: &mut [u8], retry_count: u32) -> Result<()> {
    if out.is_empty() {
        return Ok(());
    }
    let n: u32 = out.len().try_into().map_err(|_| {
        Error::InvalidArgument(format!(
            "rdrand_bytes: length {} exceeds u32::MAX",
            out.len()
        ))
    })?;
    let raw = unsafe { sys::get_rdrand_bytes_arr(out.as_mut_ptr(), n, retry_count) };
    check_status("securerng", raw)
}

/// Fill `out` with random bytes from `RDSEED`.
pub fn rdseed_bytes(out: &mut [u8], retry_count: u32) -> Result<()> {
    if out.is_empty() {
        return Ok(());
    }
    let n: u32 = out.len().try_into().map_err(|_| {
        Error::InvalidArgument(format!(
            "rdseed_bytes: length {} exceeds u32::MAX",
            out.len()
        ))
    })?;
    let raw = unsafe { sys::get_rdseed_bytes_arr(out.as_mut_ptr(), n, retry_count) };
    check_status("securerng", raw)
}

/// Fill `out` with random `u32`s from `RDRAND`.
pub fn rdrand_u32_array(out: &mut [u32], retry_count: u32) -> Result<()> {
    if out.is_empty() {
        return Ok(());
    }
    let n: u32 = out.len().try_into().map_err(|_| {
        Error::InvalidArgument(format!(
            "rdrand_u32_array: length {} exceeds u32::MAX",
            out.len()
        ))
    })?;
    let raw = unsafe { sys::get_rdrand32u_arr(out.as_mut_ptr(), n, retry_count) };
    check_status("securerng", raw)
}

/// Fill `out` with random `u64`s from `RDRAND`.
pub fn rdrand_u64_array(out: &mut [u64], retry_count: u32) -> Result<()> {
    if out.is_empty() {
        return Ok(());
    }
    let n: u32 = out.len().try_into().map_err(|_| {
        Error::InvalidArgument(format!(
            "rdrand_u64_array: length {} exceeds u32::MAX",
            out.len()
        ))
    })?;
    let raw = unsafe { sys::get_rdrand64u_arr(out.as_mut_ptr(), n, retry_count) };
    check_status("securerng", raw)
}

/// Library version reported by AOCL-SecureRNG.
pub fn version() -> Option<String> {
    unsafe {
        let p = sys::get_secrngversion();
        if p.is_null() {
            return None;
        }
        Some(std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The support query and the instruction must agree: supported means the read succeeds, unsupported means it
    /// fails with a status (not a value). A flipped query fails on either kind of CPU.
    #[test]
    fn cpu_support_query_is_consistent() {
        let r = rdrand_u32(DEFAULT_RETRY_COUNT);
        assert_eq!(is_rdrand_supported(), r.is_ok(), "rdrand: {r:?}");
        let s = rdseed_u32(DEFAULT_RETRY_COUNT);
        // RDSEED can fail transiently on a supported CPU, so only the unsupported direction is exact.
        if !is_rdseed_supported() {
            assert!(s.is_err(), "rdseed unsupported but returned {s:?}");
        }
    }

    #[test]
    fn rdrand_arrays_are_filled_to_the_end() {
        if !is_rdrand_supported() {
            return;
        }
        let mut a32 = [0u32; 64];
        rdrand_u32_array(&mut a32, DEFAULT_RETRY_COUNT).unwrap();
        assert!(
            a32[32..].iter().any(|&v| v != 0),
            "second half of the u32 array untouched"
        );
        assert!(
            a32[..32].iter().any(|&v| v != 0),
            "first half of the u32 array untouched"
        );
        let mut a64 = [0u64; 32];
        rdrand_u64_array(&mut a64, DEFAULT_RETRY_COUNT).unwrap();
        assert!(
            a64[16..].iter().any(|&v| v != 0),
            "second half of the u64 array untouched"
        );
        let mut bytes = [0u8; 64];
        rdrand_bytes(&mut bytes, DEFAULT_RETRY_COUNT).unwrap();
        assert!(
            bytes[32..].iter().any(|&b| b != 0),
            "second half of the byte buffer untouched"
        );
    }

    #[test]
    fn rdrand_yields_distinct_values_when_supported() {
        if !is_rdrand_supported() {
            eprintln!("RDRAND not supported on this CPU; skipping");
            return;
        }
        let a = rdrand_u64(DEFAULT_RETRY_COUNT).unwrap();
        let b = rdrand_u64(DEFAULT_RETRY_COUNT).unwrap();
        assert_ne!(a, b, "RDRAND returned the same u64 twice");
    }

    #[test]
    fn rdrand_bytes_fills_buffer() {
        if !is_rdrand_supported() {
            return;
        }
        let mut buf = [0u8; 32];
        rdrand_bytes(&mut buf, DEFAULT_RETRY_COUNT).unwrap();
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn empty_inputs_are_ok() {
        let mut empty: [u8; 0] = [];
        rdrand_bytes(&mut empty, DEFAULT_RETRY_COUNT).unwrap();
        let mut empty32: [u32; 0] = [];
        rdrand_u32_array(&mut empty32, DEFAULT_RETRY_COUNT).unwrap();
    }

    #[test]
    fn version_string_is_non_empty() {
        if let Some(v) = version() {
            assert!(!v.is_empty());
        }
    }
    /// The 16- and 64-bit RDRAND/RDSEED reads: each returns the widths it names, and many draws are not all equal (a stuck
    /// or constant value, or a read that only fills part of the integer, fails the spread checks; 64 draws of a 16-bit
    /// value are all equal with probability 2^-960).
    #[test]
    fn rdrand_u16_and_rdseed_u16_u64_vary() {
        if is_rdrand_supported() {
            let v: Vec<u16> = (0..64)
                .map(|_| rdrand_u16(DEFAULT_RETRY_COUNT).unwrap())
                .collect();
            assert!(v.iter().any(|x| *x != v[0]), "rdrand_u16 constant: {v:?}");
            assert!(
                v.iter().any(|x| *x > 0xFF),
                "rdrand_u16 never above 8 bits: {v:?}"
            );
        }
        if is_rdseed_supported() {
            let v: Vec<u16> = (0..64).map(|_| rdseed_u16(1000).unwrap()).collect();
            assert!(v.iter().any(|x| *x != v[0]), "rdseed_u16 constant: {v:?}");
            assert!(
                v.iter().any(|x| *x > 0xFF),
                "rdseed_u16 never above 8 bits: {v:?}"
            );
            let w: Vec<u64> = (0..64).map(|_| rdseed_u64(1000).unwrap()).collect();
            assert!(w.iter().any(|x| *x != w[0]), "rdseed_u64 constant: {w:?}");
            assert!(
                w.iter().any(|x| *x > u32::MAX as u64),
                "rdseed_u64 never above 32 bits: {w:?}"
            );
        }
    }

    /// `rdseed_bytes` fills exactly the buffer it is given (odd lengths too: not rounded up) and leaves the guard bytes
    /// around it alone; an empty buffer is a no-op.
    #[test]
    fn rdseed_bytes_fills_exactly_the_buffer() {
        if !is_rdseed_supported() {
            return;
        }
        for n in [1usize, 3, 7, 9, 13, 31, 64] {
            let mut buf = vec![0xAAu8; n + 2];
            rdseed_bytes(&mut buf[1..=n], 1000).unwrap();
            assert_eq!(
                (buf[0], buf[n + 1]),
                (0xAA, 0xAA),
                "guard bytes touched for n={n}"
            );
            // 32 random bytes differ from the fill pattern somewhere with overwhelming probability; shorter ones may all
            // equal 0xAA with probability 256^-n, so only n >= 7 is asserted.
            if n >= 7 {
                assert!(
                    buf[1..=n].iter().any(|b| *b != 0xAA),
                    "n={n} not written: {buf:?}"
                );
            }
        }
        rdseed_bytes(&mut [], 1000).unwrap();
    }
}
