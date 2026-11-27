// usage: probe <dll> <argwidth 4|8> <case>   cases: width | kat1 | kat2 | kat3
use std::ffi::{c_void, CStr, OsStr};
use std::os::windows::ffi::OsStrExt;
extern "system" {
    fn LoadLibraryW(p: *const u16) -> *mut c_void;
    fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void;
    fn GetLastError() -> u32;
    fn GetCurrentProcess() -> *mut c_void;
    fn K32EnumProcessModules(p: *mut c_void, m: *mut *mut c_void, cb: u32, need: *mut u32) -> i32;
    fn K32GetModuleFileNameExW(p: *mut c_void, m: *mut c_void, f: *mut u16, n: u32) -> u32;
}
const PAD: u8 = 0xAA;
struct Slots { b: Vec<u8>, w: usize }
impl Slots {
    fn new(n: usize, w: usize) -> Self { Slots { b: vec![PAD; n * w + 32], w } }
    fn set(&mut self, i: usize, v: i64) {
        let o = i * self.w;
        if self.w == 4 { self.b[o..o + 4].copy_from_slice(&(v as i32).to_le_bytes()); }
        else { self.b[o..o + 8].copy_from_slice(&v.to_le_bytes()); }
    }
    fn get(&self, i: usize) -> i64 {
        let o = i * self.w;
        if self.w == 4 { i32::from_le_bytes(self.b[o..o + 4].try_into().unwrap()) as i64 }
        else { i64::from_le_bytes(self.b[o..o + 8].try_into().unwrap()) }
    }
    fn ptr(&mut self) -> *mut c_void { self.b.as_mut_ptr() as *mut c_void }
    fn changed(&self, nbytes: usize) -> usize { self.b[..nbytes].iter().filter(|&&x| x != PAD).count() }
}
type F4 = unsafe extern "C" fn(i32, i32, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void);
type F8 = unsafe extern "C" fn(i64, i64, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void);
type U4 = unsafe extern "C" fn(i32, f64, f64, *mut c_void, *mut f64, *mut c_void);
type U8 = unsafe extern "C" fn(i64, f64, f64, *mut c_void, *mut f64, *mut c_void);
unsafe fn sym(h: *mut c_void, n: &str) -> *mut c_void {
    let c = format!("{}\0", n);
    let p = GetProcAddress(h, c.as_ptr());
    if p.is_null() { println!("GetProcAddress({}) FAILED err={}", n, GetLastError()); std::process::exit(3); }
    p
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (dll, w, case) = (&a[1], a[2].parse::<usize>().unwrap(), a[3].as_str());
    unsafe {
        let wide: Vec<u16> = OsStr::new(dll).encode_wide().chain(Some(0)).collect();
        let h = LoadLibraryW(wide.as_ptr());
        if h.is_null() { println!("LoadLibraryW failed err={}", GetLastError()); std::process::exit(2); }
        println!("dll={} argwidth_bytes={} case={}", dll, w, case);
        let gv: unsafe extern "C" fn() -> *const i8 = std::mem::transmute(sym(h, "get_rngversion"));
        println!("get_rngversion = {:?}", CStr::from_ptr(gv()));
        let init = sym(h, "drandinitialize");
        let unif = sym(h, "dranduniform");
        let call_init = |genid: i64, subid: i64, seed: &mut Slots, lseed: &mut Slots, state: &mut Slots, lstate: &mut Slots, info: &mut Slots| {
            if w == 4 { let f: F4 = std::mem::transmute(init); f(genid as i32, subid as i32, seed.ptr(), lseed.ptr(), state.ptr(), lstate.ptr(), info.ptr()) }
            else { let f: F8 = std::mem::transmute(init); f(genid, subid, seed.ptr(), lseed.ptr(), state.ptr(), lstate.ptr(), info.ptr()) }
        };
        let (genid, seedv): (i64, Vec<i64>) = match case {
            "width" | "kat1" => (1, vec![12345]),
            "kat2" => (2, vec![1, 2, 3, 4]),
            "kat3" => (3, (1..=624).map(|i| i * 7 + 1).collect()),
            _ => panic!("case"),
        };
        let ls = seedv.len();
        let mut seed = Slots::new(ls, w);
        for (i, v) in seedv.iter().enumerate() { seed.set(i, *v); }
        let mut lseed = Slots::new(1, w); lseed.set(0, ls as i64); // slot 1 only: rest PAD
        let mut lstate = Slots::new(1, w);
        let mut info = Slots::new(2, w); // 16 bytes region (w=8) / 8 used + pad
        let mut dummy_state = Slots::new(4, w);
        // step 1: query lstate with lstate=-1
        lstate.set(0, -1);
        call_init(genid, 1, &mut seed, &mut lseed, &mut dummy_state, &mut lstate, &mut info);
        let ch = info.b[..16].iter().filter(|&&x| x != PAD).count();
        println!("query: info_bytes_changed_of_16={} info_raw={:02x?} info_as_w={} lstate_as_w={}", ch, &info.b[..16], info.get(0), lstate.get(0));
        let mut need = lstate.get(0);
        if need <= 0 || need > 100000 { println!("lstate query gave implausible {}; using 1024", need); need = 1024; }
        // step 2: init for real
        let mut state = Slots::new(need as usize + 8, w);
        let mut lstate2 = Slots::new(1, w); lstate2.set(0, need);
        let mut info2 = Slots::new(2, w);
        let mut lseed2 = Slots::new(1, w); lseed2.set(0, ls as i64);
        call_init(genid, 1, &mut seed, &mut lseed2, &mut state, &mut lstate2, &mut info2);
        let ch2 = info2.b[..16].iter().filter(|&&x| x != PAD).count();
        println!("init:  info_bytes_changed_of_16={} info_raw={:02x?} info_as_w={}", ch2, &info2.b[..16], info2.get(0));
        // step 3: uniform
        let n = 6usize;
        let mut x = vec![f64::from_bits(0xAAAAAAAAAAAAAAAA); n];
        let (av, bv) = (0.0f64, 1.0f64);
        let mut info3 = Slots::new(2, w);
        if w == 4 { let f: U4 = std::mem::transmute(unif); f(n as i32, av, bv, state.ptr(), x.as_mut_ptr(), info3.ptr()) }
        else { let f: U8 = std::mem::transmute(unif); f(n as i64, av, bv, state.ptr(), x.as_mut_ptr(), info3.ptr()) }
        let ch3 = info3.b[..16].iter().filter(|&&x| x != PAD).count();
        println!("unif:  info_bytes_changed_of_16={} info_raw={:02x?} info_as_w={}", ch3, &info3.b[..16], info3.get(0));
        for v in &x { println!("x {:016x} {}", v.to_bits(), v); }
        // modules
        let p = GetCurrentProcess();
        let mut mods = vec![std::ptr::null_mut::<c_void>(); 512]; let mut need_b = 0u32;
        K32EnumProcessModules(p, mods.as_mut_ptr(), (512 * 8) as u32, &mut need_b);
        let cnt = need_b as usize / 8;
        let mut names = vec![];
        for m in &mods[..cnt] { let mut buf = [0u16; 520]; let l = K32GetModuleFileNameExW(p, *m, buf.as_mut_ptr(), 520); names.push(String::from_utf16_lossy(&buf[..l as usize])); }
        println!("modules_loaded={}", cnt);
        for n in &names { let l = n.to_lowercase(); if !l.contains("system32") { println!("module {}", n); } }
        println!("omp_modules: {:?}", names.iter().filter(|n| { let l = n.rsplit(char::from(92)).next().unwrap().to_lowercase(); l.contains("omp") }).collect::<Vec<_>>());
    }
}
