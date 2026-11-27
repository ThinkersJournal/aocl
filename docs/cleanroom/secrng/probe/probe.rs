// secrng probe. usage: probe <dll> <case>
// cases: info | single | arr | retry | stat | crash_<fn>_<kind>   (kind: ptrasval | swapped | null)
// Inputs: secrng-guide.txt prototypes + the DLL itself. No header, no existing binding was read.
use core::ffi::{c_int, c_uint, c_void, c_char, CStr};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
extern "system" {
    fn LoadLibraryW(p: *const u16) -> *mut c_void;
    fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void;
    fn GetLastError() -> u32;
    fn GetCurrentProcess() -> *mut c_void;
    fn K32EnumProcessModules(p: *mut c_void, m: *mut *mut c_void, cb: u32, need: *mut u32) -> i32;
    fn K32GetModuleFileNameExW(p: *mut c_void, m: *mut c_void, f: *mut u16, n: u32) -> u32;
}
type Single16 = unsafe extern "C" fn(*mut u16, c_uint) -> c_int;
type Single32 = unsafe extern "C" fn(*mut u32, c_uint) -> c_int;
type Single64 = unsafe extern "C" fn(*mut u64, c_uint) -> c_int;
type Arr32 = unsafe extern "C" fn(*mut u32, c_uint, c_uint) -> c_int;
type Arr64 = unsafe extern "C" fn(*mut u64, c_uint, c_uint) -> c_int;
type Bytes = unsafe extern "C" fn(*mut u8, c_uint, c_uint) -> c_int;
type Supp = unsafe extern "C" fn() -> c_int;
type Ver = unsafe extern "C" fn() -> *const c_char;
unsafe fn sym(h: *mut c_void, n: &str) -> *mut c_void {
    let c = format!("{}\0", n);
    let p = GetProcAddress(h, c.as_ptr());
    if p.is_null() { println!("GetProcAddress({}) FAILED err={}", n, GetLastError()); std::process::exit(3); }
    p
}
fn modules() {
    unsafe {
        let mut m = [std::ptr::null_mut::<c_void>(); 512]; let mut need = 0u32;
        K32EnumProcessModules(GetCurrentProcess(), m.as_mut_ptr(), (m.len() * 8) as u32, &mut need);
        let n = need as usize / 8; let mut omp = 0;
        for i in 0..n {
            let mut b = [0u16; 520]; let l = K32GetModuleFileNameExW(GetCurrentProcess(), m[i], b.as_mut_ptr(), 520) as usize;
            let s = String::from_utf16_lossy(&b[..l]); let lo = s.to_lowercase();
            if lo.contains("omp") || lo.contains("iomp") || lo.contains("gomp") { omp += 1; }
            println!("  module[{}] = {}", i, s);
        }
        println!("modules_loaded={} openmp_like_modules={}", n, omp);
    }
}
/// bytes that were ever != sentinel over trials with alternating sentinels 0xAA/0x55 (a random byte equals one sentinel w.p. 1/256 only)
struct Probe { w: Vec<bool> }
impl Probe {
    fn new(n: usize) -> Self { Probe { w: vec![false; n] } }
    fn note(&mut self, buf: &[u8], sent: u8) { for (i, &b) in buf.iter().enumerate() { if b != sent { self.w[i] = true; } } }
    fn count(&self) -> usize { self.w.iter().filter(|&&x| x).count() }
    fn max_off(&self) -> i64 { self.w.iter().rposition(|&x| x).map(|x| x as i64).unwrap_or(-1) }
}
const TR: usize = 32;
fn chi_bytes(b: &[u8]) -> (f64, usize, f64) {
    let mut h = [0u64; 256]; for &x in b { h[x as usize] += 1; }
    let e = b.len() as f64 / 256.0;
    let chi: f64 = h.iter().map(|&c| { let d = c as f64 - e; d * d / e }).sum();
    let ones: u64 = b.iter().map(|x| x.count_ones() as u64).sum();
    (chi, h.iter().filter(|&&c| c > 0).count(), ones as f64 / (b.len() as f64 * 8.0))
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (dll, case) = (&a[1], a[2].as_str());
    unsafe {
        let wide: Vec<u16> = OsStr::new(dll).encode_wide().chain(Some(0)).collect();
        let h = LoadLibraryW(wide.as_ptr());
        if h.is_null() { println!("LoadLibraryW failed err={}", GetLastError()); std::process::exit(2); }
        println!("dll={} case={}", dll, case);
        match case {
            "info" => {
                let v: Ver = std::mem::transmute(sym(h, "get_secrngversion"));
                println!("get_secrngversion() = {:?}", CStr::from_ptr(v()));
                for n in ["is_RDRAND_supported", "is_RDSEED_supported"] {
                    let f: Supp = std::mem::transmute(sym(h, n));
                    println!("{}() = {}", n, f());
                }
                // non-guide exports: raw bytes only (no prototype guessed)
                for n in ["aoclsecrng_version", "trdrand_status", "trdseed_status"] {
                    let p = sym(h, n) as *const u8; let b = std::slice::from_raw_parts(p, 16);
                    println!("raw first16 bytes at export {} = {:02x?}", n, b);
                }
                modules();
            }
            "single" => {
                // slot = 16 bytes; count written bytes per function
                macro_rules! one { ($name:expr, $ty:ty, $w:expr) => {{
                    let f: $ty = std::mem::transmute(sym(h, $name));
                    let mut pr = Probe::new(16); let mut rets = vec![];
                    for t in 0..TR {
                        let s = if t % 2 == 0 { 0xAAu8 } else { 0x55 };
                        let mut buf = [s; 16];
                        let r = f(buf.as_mut_ptr() as *mut _, 10);
                        rets.push(r); pr.note(&buf, s);
                    }
                    rets.sort(); rets.dedup();
                    println!("{:<18} ret={:?} bytes_written={} max_offset_written={} (expected width {})", $name, rets, pr.count(), pr.max_off(), $w);
                }}}
                one!("get_rdrand16u", Single16, 2); one!("get_rdrand32u", Single32, 4); one!("get_rdrand64u", Single64, 8);
                one!("get_rdseed16u", Single16, 2); one!("get_rdseed32u", Single32, 4); one!("get_rdseed64u", Single64, 8);
                // undocumented-by-guide l_ variants are NOT called (prototype UNKNOWN).
                // NEGATIVE CONTROL: the 32u function handed an 8-byte slot (treated as if 64-bit) and the 16u one too
                println!("-- wrong-width controls: call fn through a type that assumes a wider slot; count bytes the fn actually wrote");
                // (same functions, slot is 16 bytes already, so the count above IS the control: 16u=2, 32u=4, 64u=8 differ pairwise)
                println!("-- retry_count=0 and 0xFFFFFFFF");
                for rc in [0u32, 1, 1000, 0xFFFF_FFFF] {
                    let f: Single32 = std::mem::transmute(sym(h, "get_rdrand32u")); let mut x = 0u32;
                    let r = f(&mut x, rc); println!("get_rdrand32u retry={} -> ret={} val=0x{:08x}", rc, r, x);
                    let f: Single32 = std::mem::transmute(sym(h, "get_rdseed32u")); let mut x = 0u32;
                    let r = f(&mut x, rc); println!("get_rdseed32u retry={} -> ret={} val=0x{:08x}", rc, r, x);
                }
                modules();
            }
            "arr" => {
                let ns = [0u32, 1, 3, 13, 16, 17];
                for n in ["get_rdrand32u_arr", "get_rdseed32u_arr"] {
                    let f: Arr32 = std::mem::transmute(sym(h, n));
                    for &c in &ns {
                        let mut pr = Probe::new(256); let mut rets = vec![];
                        for t in 0..TR { let s = if t % 2 == 0 { 0xAAu8 } else { 0x55 }; let mut buf = [s; 256];
                            let r = f(buf.as_mut_ptr() as *mut u32, c, 10); rets.push(r); pr.note(&buf, s); }
                        rets.sort(); rets.dedup();
                        println!("{:<20} N={:<3} ret={:?} bytes_written={} max_off={} expect={}", n, c, rets, pr.count(), pr.max_off(), c * 4);
                    }
                }
                for n in ["get_rdrand64u_arr", "get_rdseed64u_arr"] {
                    let f: Arr64 = std::mem::transmute(sym(h, n));
                    for &c in &ns {
                        let mut pr = Probe::new(256); let mut rets = vec![];
                        for t in 0..TR { let s = if t % 2 == 0 { 0xAAu8 } else { 0x55 }; let mut buf = [s; 256];
                            let r = f(buf.as_mut_ptr() as *mut u64, c, 10); rets.push(r); pr.note(&buf, s); }
                        rets.sort(); rets.dedup();
                        println!("{:<20} N={:<3} ret={:?} bytes_written={} max_off={} expect={}", n, c, rets, pr.count(), pr.max_off(), c * 8);
                    }
                }
                for n in ["get_rdrand_bytes_arr", "get_rdseed_bytes_arr"] {
                    let f: Bytes = std::mem::transmute(sym(h, n));
                    for &c in &[0u32, 1, 3, 7, 8, 9, 13, 16, 17, 31] {
                        let mut pr = Probe::new(256); let mut rets = vec![];
                        for t in 0..TR { let s = if t % 2 == 0 { 0xAAu8 } else { 0x55 }; let mut buf = [s; 256];
                            let r = f(buf.as_mut_ptr(), c, 10); rets.push(r); pr.note(&buf, s); }
                        rets.sort(); rets.dedup();
                        println!("{:<20} N={:<3} ret={:?} bytes_written={} max_off={} expect={}", n, c, rets, pr.count(), pr.max_off(), c);
                    }
                }
                // Control: wrong element width. 32u_arr N=3 vs 64u_arr N=3 differ (12 vs 24, above). Control for N-units: N is count of
                // elements, so a caller treating N as a byte count of a u32 array would expect 3 bytes: the measured 12 refutes that.
                modules();
            }
            "retry" => {
                // failure-rate vs retry_count, tallying return values, plus slot state on failure; threads 1 and 16.
                // (a 64-bit retry_count with garbage high half is NOT attempted: could mean a 4G-element write.)
                let a32 = sym(h, "get_rdseed32u_arr") as usize; let p32 = sym(h, "get_rdseed32u") as usize; let r32 = sym(h, "get_rdrand32u") as usize;
                for &(nm, addr, is_arr) in &[("get_rdseed32u", p32, false), ("get_rdrand32u", r32, false), ("get_rdseed32u_arr(N=8)", a32, true)] {
                  for &th in &[1usize, 16] {
                    for &rc in &[0u32, 1, 10, 1000] {
                        let calls = 50000usize;
                        let mut hs = vec![];
                        for _ in 0..th { hs.push(std::thread::spawn(move || {
                            let mut tally = std::collections::BTreeMap::<i32, u64>::new(); let (mut fw, mut fu) = (0u64, 0u64);
                            for _ in 0..calls {
                                if !is_arr { let f: Single32 = std::mem::transmute(addr); let mut x = 0xAAAAAAAAu32; let r = f(&mut x, rc);
                                    *tally.entry(r).or_default() += 1; if r != 2 { if x == 0xAAAAAAAA { fu += 1 } else { fw += 1 } } }
                                else { let f: Arr32 = std::mem::transmute(addr); let mut x = [0xAAAAAAAAu32; 8]; let r = f(x.as_mut_ptr(), 8, rc);
                                    *tally.entry(r).or_default() += 1; if r != 2 { if x.iter().all(|&v| v == 0xAAAAAAAA) { fu += 1 } else { fw += 1 } } }
                            }
                            (tally, fw, fu) })); }
                        let mut tot = std::collections::BTreeMap::<i32, u64>::new(); let (mut fw, mut fu) = (0, 0);
                        for j in hs { let (t, a, b) = j.join().unwrap(); for (k, v) in t { *tot.entry(k).or_default() += v; } fw += a; fu += b; }
                        println!("{:<24} threads={:<2} retry={:<5} calls={} return-tally={:?} on_failure: slot_untouched={} slot_partly_written={}", nm, th, rc, calls * th, tot, fu, fw);
                    }
                  }
                }
                modules();
            }
            "stat" => {
                let mut all: Vec<(String, Vec<u8>)> = vec![];
                let f: Bytes = std::mem::transmute(sym(h, "get_rdrand_bytes_arr")); let mut b = vec![0u8; 65536]; let r = f(b.as_mut_ptr(), 65536, 10); println!("rdrand_bytes ret={}", r); all.push(("get_rdrand_bytes_arr(65536)".into(), b));
                let f: Bytes = std::mem::transmute(sym(h, "get_rdseed_bytes_arr")); let mut b = vec![0u8; 65536]; let r = f(b.as_mut_ptr(), 65536, 1000); println!("rdseed_bytes ret={}", r); all.push(("get_rdseed_bytes_arr(65536)".into(), b));
                let f: Arr32 = std::mem::transmute(sym(h, "get_rdrand32u_arr")); let mut b = vec![0u8; 65536]; f(b.as_mut_ptr() as *mut u32, 16384, 10); all.push(("get_rdrand32u_arr(16384)".into(), b));
                let f: Arr64 = std::mem::transmute(sym(h, "get_rdrand64u_arr")); let mut b = vec![0u8; 65536]; f(b.as_mut_ptr() as *mut u64, 8192, 10); all.push(("get_rdrand64u_arr(8192)".into(), b));
                let f: Arr32 = std::mem::transmute(sym(h, "get_rdseed32u_arr")); let mut b = vec![0u8; 65536]; f(b.as_mut_ptr() as *mut u32, 16384, 1000); all.push(("get_rdseed32u_arr(16384)".into(), b));
                let f: Arr64 = std::mem::transmute(sym(h, "get_rdseed64u_arr")); let mut b = vec![0u8; 65536]; f(b.as_mut_ptr() as *mut u64, 8192, 1000); all.push(("get_rdseed64u_arr(8192)".into(), b));
                // singles looped 4096 times; per-byte-lane distinct count
                for (nm, w) in [("get_rdrand16u", 2usize), ("get_rdrand32u", 4), ("get_rdrand64u", 8), ("get_rdseed16u", 2), ("get_rdseed32u", 4), ("get_rdseed64u", 8)] {
                    let p = sym(h, nm); let mut b = Vec::with_capacity(4096 * w); let mut fails = 0;
                    for _ in 0..4096 { let mut buf = [0u8; 8]; let r = match w {
                        2 => (std::mem::transmute::<_, Single16>(p))(buf.as_mut_ptr() as *mut u16, 1000),
                        4 => (std::mem::transmute::<_, Single32>(p))(buf.as_mut_ptr() as *mut u32, 1000),
                        _ => (std::mem::transmute::<_, Single64>(p))(buf.as_mut_ptr() as *mut u64, 1000) };
                        if r != 2 { fails += 1; } b.extend_from_slice(&buf[..w]); }
                    let mut lanes = String::new();
                    for l in 0..w { let mut seen = [false; 256]; for i in 0..4096 { seen[b[i * w + l] as usize] = true; } lanes += &format!("{} ", seen.iter().filter(|&&x| x).count()); }
                    println!("{}: 4096 calls, rets_not_2={}, distinct-values per byte lane (expect ~256 each) = {}", nm, fails, lanes);
                }
                println!("{:<30} {:>10} {:>9} {:>9}   (65536 bytes; chi2 dof=255: expect 255 +/- 22.6; ones-fraction expect 0.5 +/- 0.0007)", "source", "chi2", "distinct", "ones");
                for (n, b) in &all { let (c, d, o) = chi_bytes(b); println!("{:<30} {:>10.1} {:>9} {:>9.5}", n, c, d, o); }
                // controls: all-zero buffer, masked (bit7 cleared) random buffer, and N=0 untouched buffer
                let (c, d, o) = chi_bytes(&vec![0u8; 65536]); println!("CONTROL all-zero               {:>10.1} {:>9} {:>9.5}", c, d, o);
                let m: Vec<u8> = all[0].1.iter().map(|x| x & 0x7f).collect(); let (c, d, o) = chi_bytes(&m); println!("CONTROL rdrand_bytes & 0x7f    {:>10.1} {:>9} {:>9.5}", c, d, o);
                let f: Bytes = std::mem::transmute(sym(h, "get_rdrand_bytes_arr")); let mut b = vec![0u8; 65536]; f(b.as_mut_ptr(), 0, 10); let (c, d, o) = chi_bytes(&b); println!("CONTROL rdrand_bytes N=0       {:>10.1} {:>9} {:>9.5}", c, d, o);
                let f: Bytes = std::mem::transmute(sym(h, "get_rdrand_bytes_arr")); let mut b = vec![0u8; 65536]; f(b.as_mut_ptr(), 32768, 10); let (c, d, o) = chi_bytes(&b); println!("CONTROL rdrand_bytes N=32768 of 65536 (half buffer unwritten) {:>10.1} {:>9} {:>9.5}", c, d, o);
                let f: Bytes = std::mem::transmute(sym(h, "get_rdrand_bytes_arr")); let mut x = vec![0u8; 4096]; let mut y = vec![0u8; 4096]; f(x.as_mut_ptr(), 4096, 10); f(y.as_mut_ptr(), 4096, 10);
                println!("two successive 4096-byte calls: identical_bytes={} (expect ~16)", x.iter().zip(&y).filter(|(a, b)| a == b).count());
                modules();
            }
            c if c.starts_with("crash_") => {
                // crash_<fn>_<kind>; runs in its own process; exit code / crash is the observable
                let parts: Vec<&str> = c.splitn(3, '_').collect(); // crash, fn..., kind -> parse manually
                let rest = &c[6..]; let k = rest.rfind('_').unwrap(); let (fnname, kind) = (&rest[..k], &rest[k + 1..]);
                let _ = parts;
                let p = sym(h, fnname);
                println!("calling {} with kind={}", fnname, kind);
                use std::io::Write; std::io::stdout().flush().unwrap();
                let mut slot = [0xAAu8; 16];
                let r: c_int = match (fnname, kind) {
                    ("get_rdrand32u", "null") | ("get_rdseed32u", "null") => (std::mem::transmute::<_, Single32>(p))(std::ptr::null_mut(), 10),
                    ("get_rdrand32u", "ptrasval") => (std::mem::transmute::<_, unsafe extern "C" fn(usize, c_uint) -> c_int>(p))(0x1000, 10),
                    ("get_rdrand32u", "swapped") => (std::mem::transmute::<_, unsafe extern "C" fn(c_uint, *mut u32) -> c_int>(p))(10, slot.as_mut_ptr() as *mut u32),
                    ("get_rdrand32u_arr", "null") | ("get_rdseed32u_arr", "null") => (std::mem::transmute::<_, Arr32>(p))(std::ptr::null_mut(), 4, 10),
                    ("get_rdrand32u_arr", "null0") => (std::mem::transmute::<_, Arr32>(p))(std::ptr::null_mut(), 0, 10),
                    ("get_rdrand_bytes_arr", "null") | ("get_rdseed_bytes_arr", "null") => (std::mem::transmute::<_, Bytes>(p))(std::ptr::null_mut(), 4, 10),
                    ("get_rdrand_bytes_arr", "null0") => (std::mem::transmute::<_, Bytes>(p))(std::ptr::null_mut(), 0, 10),
                    ("get_rdrand_bytes_arr", "swapped") => (std::mem::transmute::<_, unsafe extern "C" fn(c_uint, *mut u8, c_uint) -> c_int>(p))(4, slot.as_mut_ptr(), 10),
                    ("get_rdrand64u", "null") | ("get_rdseed64u", "null") => (std::mem::transmute::<_, Single64>(p))(std::ptr::null_mut(), 10),
                    ("get_rdrand16u", "null") | ("get_rdseed16u", "null") => (std::mem::transmute::<_, Single16>(p))(std::ptr::null_mut(), 10),
                    ("get_rdrand64u_arr", "null") | ("get_rdseed64u_arr", "null") => (std::mem::transmute::<_, Arr64>(p))(std::ptr::null_mut(), 4, 10),
                    _ => { println!("bad case"); std::process::exit(4) }
                };
                println!("returned ret={} (no crash); slot={:02x?}", r, slot);
            }
            _ => { println!("bad case"); std::process::exit(4); }
        }
    }
}
