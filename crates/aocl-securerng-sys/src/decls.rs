// AOCL-SecureRNG C declarations, from secrng-guide.txt (AMD AOCL API Guide 6.0, 68552) ONLY. No header was read.
// Types are the guide's, literally: uint16_t/uint32_t/uint64_t -> u16/u32/u64, int -> c_int, unsigned int -> c_uint,
// unsigned char -> c_uchar, const char -> c_char. No #[link]; symbols are lowercase/as-written (all 15 exported by amdsecrng.dll, LP64 and ILP64).
// UNKNOWN (the guide never states, so NOT declared): numeric values of SECRNG_SUCCESS / SECRNG_FAILURE / SECRNG_NOTSUPPORTED
//   (measured, not guide-stated: success=2, failure=-2, see SECRNG-MEASUREMENT.md; NOTSUPPORTED never observed);
//   the six non-guide exports (aoclsecrng_version, l_get_rdrand16u/32u/64u, trdrand_status, trdseed_status).
// Two array prototypes (get_rdrand64u_arr, get_rdseed64u_arr) are page-header-interleaved in the text (guide L141, L271);
//   the declarations below are the de-interleaved reading (the extra letters spell "AOCL API Guide (68552)").
extern "C" {
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L40  (9.4.1)
    pub fn get_secrngversion() -> *const core::ffi::c_char;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L52  (9.4.2)
    pub fn is_RDRAND_supported() -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L62
    pub fn get_rdrand16u(rng_val: *mut u16, retry_count: core::ffi::c_uint) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L81
    pub fn get_rdrand32u(rng_val: *mut u32, retry_count: core::ffi::c_uint) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L99
    pub fn get_rdrand64u(rng_val: *mut u64, retry_count: core::ffi::c_uint) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L121
    pub fn get_rdrand32u_arr(
        rng_arr: *mut u32,
        N: core::ffi::c_uint,
        retry_count: core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L141 (interleaved text, de-interleaved)
    pub fn get_rdrand64u_arr(
        rng_arr: *mut u64,
        N: core::ffi::c_uint,
        retry_count: core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L161
    pub fn get_rdrand_bytes_arr(
        rng_arr: *mut core::ffi::c_uchar,
        N: core::ffi::c_uint,
        retry_count: core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L182  (9.4.3)
    pub fn is_RDSEED_supported() -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L192
    pub fn get_rdseed16u(rng_val: *mut u16, retry_count: core::ffi::c_uint) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L211
    pub fn get_rdseed32u(rng_val: *mut u32, retry_count: core::ffi::c_uint) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L229
    pub fn get_rdseed64u(rng_val: *mut u64, retry_count: core::ffi::c_uint) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L251
    pub fn get_rdseed32u_arr(
        rng_arr: *mut u32,
        N: core::ffi::c_uint,
        retry_count: core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L271 (interleaved text, de-interleaved)
    pub fn get_rdseed64u_arr(
        rng_arr: *mut u64,
        N: core::ffi::c_uint,
        retry_count: core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // secrng-guide.txt (docs/cleanroom/secrng/secrng_provenance.csv) L291
    pub fn get_rdseed_bytes_arr(
        rng_arr: *mut core::ffi::c_uchar,
        N: core::ffi::c_uint,
        retry_count: core::ffi::c_uint,
    ) -> core::ffi::c_int;
}
