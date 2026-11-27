//! AOCL-SecureRNG (feature `securerng`): output widths measured by sentinel bytes, plus the version string.
//! Measurement: docs/cleanroom/secrng/SECRNG-MEASUREMENT.md. Success code 2 / failure -2 are measured, not guide-stated.
#![cfg(feature = "securerng")]
use aocl_rng_sys::*;

fn written(buf: &[u8; 16], sentinel: u8) -> usize {
    buf.iter().filter(|b| **b != sentinel).count()
}

#[test]
fn width_by_sentinel() {
    unsafe {
        assert_eq!(is_RDRAND_supported(), 1);
        let v = std::ffi::CStr::from_ptr(get_secrngversion())
            .to_str()
            .unwrap();
        assert!(v.starts_with("AOCL-SecureRNG 6."), "{v}");
        // Positive: widths 2/4/8 (distinct); a fixed sentinel byte can equal a random byte, so use 32 trials and union.
        let mut w = [[false; 16]; 3];
        for t in 0..32 {
            let s = if t % 2 == 0 { 0xAAu8 } else { 0x55 };
            let mut b = [[s; 16]; 3];
            assert_eq!(get_rdrand16u(b[0].as_mut_ptr() as *mut u16, 0), 2);
            assert_eq!(get_rdrand32u(b[1].as_mut_ptr() as *mut u32, 0), 2);
            assert_eq!(get_rdrand64u(b[2].as_mut_ptr() as *mut u64, 0), 2);
            for k in 0..3 {
                for i in 0..16 {
                    if b[k][i] != s {
                        w[k][i] = true;
                    }
                }
            }
        }
        let n: Vec<usize> = w.iter().map(|r| r.iter().filter(|x| **x).count()).collect();
        assert_eq!(n, vec![2, 4, 8]);
        // Negative control: NULL output -> -3, no crash; N = 0 -> -2.
        assert_eq!(get_rdrand32u(core::ptr::null_mut(), 0), -3);
        let mut a = [0u32; 1];
        assert_eq!(get_rdrand32u_arr(a.as_mut_ptr(), 0, 0), -2);
    }
    let _ = written;
}

#[test]
fn all_15_secrng_symbols_link() {
    let addrs: Vec<usize> = vec![
        get_secrngversion as usize,
        is_RDRAND_supported as usize,
        get_rdrand16u as usize,
        get_rdrand32u as usize,
        get_rdrand64u as usize,
        get_rdrand32u_arr as usize,
        get_rdrand64u_arr as usize,
        get_rdrand_bytes_arr as usize,
        is_RDSEED_supported as usize,
        get_rdseed16u as usize,
        get_rdseed32u as usize,
        get_rdseed64u as usize,
        get_rdseed32u_arr as usize,
        get_rdseed64u_arr as usize,
        get_rdseed_bytes_arr as usize,
    ];
    assert_eq!(addrs.len(), 15);
    assert!(addrs.iter().all(|a| *a != 0));
}
