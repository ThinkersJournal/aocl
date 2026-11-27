// rng_user_probe <LP64|ILP64> <S|D> <case>
// Clean-room behavioural probe of the user-generator callbacks of (s|d)randinitializeuser.
// cases: init     - UINI argument shapes (distinct genid/subid/lseed/lstate values)
//        lquery   - lstate = -1 with a UINI that writes *lstate
//        gen      - UGEN argument shapes + bytes written (natural element width)
//        gen_wrongw - negative control: UGEN writes the OTHER element width
//        gen_nopatch - negative control: no state patch (UGEN must NOT be reached)
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
extern "system" {
    fn LoadLibraryW(p: *const u16) -> *mut c_void;
    fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void;
    fn VirtualQuery(a: *const c_void, mbi: *mut Mbi, len: usize) -> usize;
}
#[repr(C)] struct Mbi { base: usize, abase: usize, aprot: u32, _p1: u32, size: usize, state: u32, prot: u32, ty: u32, _p2: u32 }
fn readable(p: u64) -> bool {
    if p < 0x10000 { return false; }
    unsafe {
        let mut m: Mbi = std::mem::zeroed();
        if VirtualQuery(p as *const c_void, &mut m, std::mem::size_of::<Mbi>()) == 0 { return false; }
        m.state == 0x1000 && (m.prot & 0xff) != 0 && (m.prot & 0x101) == 0
    }
}
fn peek(p: u64, n: usize) -> String {
    if !readable(p) { return "unreadable".into(); }
    format!("{:02x?}", unsafe { std::slice::from_raw_parts(p as *const u8, n) })
}
const NA: usize = 12; // callback records 12 raw 64-bit argument slots (rcx,rdx,r8,r9 + 8 stack slots)
// shared harness state (single thread)
static mut W: usize = 4;           // rng_int_t width of the build under test
static mut KIND_S: bool = false;
static mut MODE: u8 = 0;           // 0 record only; 1 natural width writes; 2 wrong width writes; 3 UINI writes *lstate
static mut ADDR: [u64; 8] = [0; 8]; // seed, lseed, state, lstate, info, x, info2
static mut NCALL: [u32; 2] = [0, 0];
unsafe fn rd(p: u64) -> i64 { if W == 4 { *(p as *const i32) as i64 } else { *(p as *const i64) } }
unsafe fn wr(p: u64, v: i64) { if W == 4 { *(p as *mut i32) = v as i32 } else { *(p as *mut i64) = v } }
extern "C" fn uini_cb(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64, a7: u64, a8: u64, a9: u64, a10: u64, a11: u64) {
    unsafe {
        NCALL[0] += 1;
        let v = [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11];
        println!("UINI call #{}", NCALL[0]);
        for (i, x) in v.iter().enumerate() {
            let tag = if *x == ADDR[0] { " ==seed" } else if *x == ADDR[1] { " ==lseed" } else if *x == ADDR[2] { " ==state" } else if *x == ADDR[3] { " ==lstate(caller's buffer)" } else if *x == ADDR[4] { " ==info" } else { "" };
            println!("  arg{i} = {:x}{tag} -> {}", x, peek(*x, 16));
        }
        // the 7 documented pointer args, dereferenced at rng_int_t width
        println!("  deref as rng_int_t (width {}): genid={} subid={} seed[0..4]={},{},{},{} lseed={} lstate_arg={} info={}",
            W, rd(a0), rd(a1), rd(a2), rd(a2 + W as u64), rd(a2 + 2 * W as u64), rd(a2 + 3 * W as u64), rd(a3), rd(a5), rd(a6));
        println!("  stride arg1-arg0 = {} bytes, arg3-arg2 irrelevant; arg5 (lstate arg) is {} the caller's lstate buffer", a1 as i64 - a0 as i64, if a5 == ADDR[3] { "THE SAME AS" } else { "NOT" });
        if MODE == 3 { wr(a5, 55); println!("  UINI wrote *lstate = 55"); }
    }
}
extern "C" fn ugen_cb(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64, a7: u64, a8: u64, a9: u64, a10: u64, a11: u64) {
    unsafe {
        NCALL[1] += 1;
        let v = [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11];
        println!("UGEN call #{}", NCALL[1]);
        for (i, x) in v.iter().enumerate() {
            let tag = if *x == ADDR[2] { " ==state" } else if *x == ADDR[5] { " ==x(output buffer)" } else if *x == ADDR[6] { " ==info(caller's buffer)" } else { "" };
            println!("  arg{i} = {:x}{tag} -> {}", x, peek(*x, 16));
        }
        let n = rd(a0);
        println!("  *arg0 as rng_int_t = {n}; arg4-arg0 = {} bytes", a4 as i64 - a0 as i64);
        if MODE == 1 || MODE == 2 {
            // natural element width: S float(4), D double(8); wrong = the other
            let wide = if MODE == 1 { !KIND_S } else { KIND_S };
            for i in 0..n as u64 {
                let val = 0.125 * (i + 1) as f64;
                if wide { *((a2 + 8 * i) as *mut f64) = val } else { *((a2 + 4 * i) as *mut f32) = val as f32 }
            }
            wr(a3, 0); // info = 0 at rng_int_t width
        }
    }
}
type Cb = extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64);
struct Buf { b: Vec<u8>, w: usize }
impl Buf {
    fn new(n: usize, w: usize, pad: u8) -> Self { Buf { b: vec![pad; n * w + 64], w } }
    fn set(&mut self, i: usize, v: i64) { let o = i * self.w; if self.w == 4 { self.b[o..o + 4].copy_from_slice(&(v as i32).to_le_bytes()) } else { self.b[o..o + 8].copy_from_slice(&v.to_le_bytes()) } }
    fn get(&self, i: usize) -> i64 { let o = i * self.w; if self.w == 4 { i32::from_le_bytes(self.b[o..o + 4].try_into().unwrap()) as i64 } else { i64::from_le_bytes(self.b[o..o + 8].try_into().unwrap()) } }
    fn p(&mut self) -> *mut c_void { self.b.as_mut_ptr() as *mut c_void }
    fn a(&self) -> u64 { self.b.as_ptr() as u64 }
}
unsafe fn sym(h: *mut c_void, n: &str) -> *mut c_void { let c = format!("{}\0", n); let p = GetProcAddress(h, c.as_ptr()); assert!(!p.is_null(), "no sym {n}"); p }
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (build, kind, case) = (a[1].as_str(), a[2].as_str(), a[3].as_str());
    let w = if build == "ILP64" { 8 } else { 4 };
    unsafe {
        W = w; KIND_S = kind == "S";
        MODE = match case { "gen" => 1, "gen_wrongw" => 2, "lquery" => 3, _ => 0 };
        let dll = format!("C:/Program Files/AMD/AOCL-Windows/amd-rng/lib/{}/rng_amd.dll", build);
        let wd: Vec<u16> = OsStr::new(&dll).encode_wide().chain(Some(0)).collect();
        let h = LoadLibraryW(wd.as_ptr()); assert!(!h.is_null());
        let name = if kind == "S" { "srandinitializeuser" } else { "drandinitializeuser" };
        let f = sym(h, name);
        println!("case={case} dll={dll} int_width={w} fn={name}");
        let mut seed = Buf::new(8, w, 0); for i in 0..8 { seed.set(i, 100 + i as i64); }
        let mut lseed = Buf::new(1, w, 0); lseed.set(0, 3);
        // state: the DLL writes at state[0x7ff9]-ish for a code address in 0x7ff6..: allocate 1M slots (see report), zero filled
        let mut state = Buf::new(1_000_000, w, 0);
        let mut lstate = Buf::new(1, w, 0); lstate.set(0, if case == "lquery" { -1 } else { 100 });
        let mut info = Buf::new(2, w, 0xAA);
        ADDR[0] = seed.a(); ADDR[1] = lseed.a(); ADDR[2] = state.a(); ADDR[3] = lstate.a(); ADDR[4] = info.a();
        let (genid, subid) = (5i64, 7i64);
        if w == 4 { let g: extern "C" fn(Cb, Cb, i32, i32, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(f);
            g(uini_cb, ugen_cb, genid as i32, subid as i32, seed.p(), lseed.p(), state.p(), lstate.p(), info.p()) }
        else { let g: extern "C" fn(Cb, Cb, i64, i64, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(f);
            g(uini_cb, ugen_cb, genid, subid, seed.p(), lseed.p(), state.p(), lstate.p(), info.p()) }
        println!("after init: info={} info_bytes_changed_of_16={} lstate={} UINI_calls={} UGEN_calls={}", info.get(0), info.b[..16].iter().filter(|&&x| x != 0xAA).count(), lstate.get(0), NCALL[0], NCALL[1]);
        let nz: Vec<(usize, i64)> = (0..state.b.len() / w).filter(|&i| state.get(i) != 0).map(|i| (i, state.get(i))).collect();
        println!("nonzero state slots (index,value) after init: {:x?}", nz);
        if case == "gen" || case == "gen_wrongw" || case == "gen_nopatch" {
            let ugen_addr = ugen_cb as usize as u64;
            if case != "gen_nopatch" {
                // HARNESS WORKAROUND for a DLL defect (see report): make the stored callback address and the self-check consistent.
                state.set(0, 1); state.set(1, (ugen_addr >> 32) as i64); state.set(2, (ugen_addr & 0xffff_ffff) as i64); state.set(3, (ugen_addr >> 32) as i64);
                println!("harness patch: state[0..4]=1,hi,lo,hi  (ugen {:x})", ugen_addr);
            }
            let n = 4usize; let (av, bv) = (0.5f64, 2.0f64);
            let gname = if kind == "S" { "sranduniform" } else { "dranduniform" };
            let g = sym(h, gname);
            let xw = if kind == "S" { 4 } else { 8 };
            let mut x = vec![0xAAu8; n * xw + 64];
            let mut info2 = Buf::new(2, w, 0xAA);
            ADDR[5] = x.as_ptr() as u64; ADDR[6] = info2.a();
            if kind == "D" {
                if w == 4 { let f: extern "C" fn(i32, f64, f64, *mut c_void, *mut f64, *mut c_void) = std::mem::transmute(g); f(n as i32, av, bv, state.p(), x.as_mut_ptr() as *mut f64, info2.p()) }
                else { let f: extern "C" fn(i64, f64, f64, *mut c_void, *mut f64, *mut c_void) = std::mem::transmute(g); f(n as i64, av, bv, state.p(), x.as_mut_ptr() as *mut f64, info2.p()) }
            } else {
                if w == 4 { let f: extern "C" fn(i32, f32, f32, *mut c_void, *mut f32, *mut c_void) = std::mem::transmute(g); f(n as i32, av as f32, bv as f32, state.p(), x.as_mut_ptr() as *mut f32, info2.p()) }
                else { let f: extern "C" fn(i64, f32, f32, *mut c_void, *mut f32, *mut c_void) = std::mem::transmute(g); f(n as i64, av as f32, bv as f32, state.p(), x.as_mut_ptr() as *mut f32, info2.p()) }
            }
            println!("after {gname}(n=4,a=0.5,b=2.0): info={} UGEN_calls={}", info2.get(0), NCALL[1]);
            let changed: Vec<usize> = x.iter().enumerate().filter(|(_, &b)| b != 0xAA).map(|(i, _)| i).collect();
            println!("x element width {xw}: bytes changed in x = {} (offsets {:?}..{:?}); guard bytes beyond n*{xw}={} changed: {}", changed.len(), changed.first(), changed.last(), n * xw, changed.iter().any(|&i| i >= n * xw));
            if kind == "D" { let v: Vec<f64> = (0..n).map(|i| f64::from_le_bytes(x[8 * i..8 * i + 8].try_into().unwrap())).collect(); println!("x as f64 = {:?}", v); println!("expected if callback r=(i+1)/8 and DLL scales a+(b-a)r: {:?}", (0..n).map(|i| 0.5 + 1.5 * 0.125 * (i + 1) as f64).collect::<Vec<_>>()); }
            else { let v: Vec<f32> = (0..n).map(|i| f32::from_le_bytes(x[4 * i..4 * i + 4].try_into().unwrap())).collect(); println!("x as f32 = {:?}", v); println!("expected if callback r=(i+1)/8 and DLL scales a+(b-a)r: {:?}", (0..n).map(|i| 0.5 + 1.5 * 0.125 * (i + 1) as f32).collect::<Vec<_>>()); }
        }
    }
}
