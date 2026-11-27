// rng_probe <LP64|ILP64> <S|D> <genid> [lseed_or_mode...]   explore the user-generator callbacks of (s|d)randinitializeuser
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
extern "system" {
    fn LoadLibraryW(p: *const u16) -> *mut c_void;
    fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void;
    fn AddVectoredExceptionHandler(first: u32, h: unsafe extern "system" fn(*mut ExcPtrs) -> i32) -> *mut c_void;
    fn VirtualAlloc(a: *mut c_void, sz: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualQuery(a: *const c_void, mbi: *mut Mbi, len: usize) -> usize;
}
#[repr(C)] struct Mbi { base: usize, abase: usize, aprot: u32, _p1: u32, size: usize, state: u32, prot: u32, ty: u32, _p2: u32 }
fn readable(p: u64) -> bool {
    if p < 0x10000 { return false; }
    unsafe {
        let mut m: Mbi = std::mem::zeroed();
        if VirtualQuery(p as *const c_void, &mut m, std::mem::size_of::<Mbi>()) == 0 { return false; }
        m.state == 0x1000 && (m.prot & 0xff) != 0 && (m.prot & 0x101) == 0 // MEM_COMMIT, not NOACCESS/GUARD
    }
}
fn peek(p: u64) -> String {
    if !readable(p) { return "unreadable".into(); }
    let b = unsafe { std::slice::from_raw_parts(p as *const u8, 16) };
    format!("{:02x?}", b)
}
#[repr(C)] struct ExcRec { code: u32, flags: u32, next: *mut c_void, addr: u64, nparams: u32, _p: u32, info: [u64; 15] }
#[repr(C)] struct ExcPtrs { rec: *mut ExcRec, ctx: *mut u8 }
unsafe extern "system" fn veh(p: *mut ExcPtrs) -> i32 {
    let r = &*(*p).rec;
    let rip = *((*p).ctx.add(0xF8) as *const u64);
    eprintln!("EXCEPTION code={:x} at={:x} rip={:x} info0={:x} info1={:x}", r.code, r.addr, rip, r.info[0], r.info[1]);
    { let c = (*p).ctx; let names = ["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi","r8","r9","r10","r11","r12","r13","r14","r15"]; let mut o = String::new(); for (i, n) in names.iter().enumerate() { o += &format!("{}={:x} ", n, *(c.add(0x78 + 8 * i) as *const u64)); } eprintln!("regs: {}", o); }
    if STATE_P != 0 { let b = std::slice::from_raw_parts(STATE_P as *const u32, 16); eprintln!("state[0..16] u32 at exception: {:x?}", b); eprintln!("state addr={:x} fault-state={:#x}", STATE_P, r.info[1].wrapping_sub(STATE_P)); }
    0
}
static mut STATE_P: u64 = 0;
// Trampoline in the low 4 GiB: mov rax, imm64 ; jmp rax   (machine-code bytes written by us; not disassembly of the DLL)
unsafe fn sym_tramp(target: usize) -> usize { // code at 0x1_0000_0001 so that hi32 == lo32 == 1
    let p = VirtualAlloc(0x1_0000_0000usize as *mut c_void, 0x1000, 0x3000, 0x40) as usize;
    assert!(p == 0x1_0000_0000, "alloc {:x}", p);
    let code = (p + 1) as *mut u8;
    *code = 0x48; *code.add(1) = 0xB8; std::ptr::copy_nonoverlapping(target.to_le_bytes().as_ptr(), code.add(2), 8);
    *code.add(10) = 0xFF; *code.add(11) = 0xE0;
    p + 1
}
unsafe fn low_tramp(slot: usize, target: usize) -> usize {
    let mut p = 0usize;
    for k in 0..200usize { let base = std::env::var("TRAMPBASE").ok().map(|s| usize::from_str_radix(&s, 16).unwrap()).unwrap_or(0x3000_0000usize) + (slot * 300 + k) * 0x10000; p = VirtualAlloc(base as *mut c_void, 0x1000, 0x3000, 0x40) as usize; if p == base { break; } }
    assert!(p != 0, "VirtualAlloc low failed: {:x}", p);
    let code = p as *mut u8;
    *code = 0x48; *code.add(1) = 0xB8; std::ptr::copy_nonoverlapping(target.to_le_bytes().as_ptr(), code.add(2), 8);
    *code.add(10) = 0xFF; *code.add(11) = 0xE0;
    p
}
const NA: usize = 12; const PADB: u8 = 0xAA;
#[derive(Clone, Debug)] struct Call { which: &'static str, args: [u64; NA] }
static mut LOG: Vec<Call> = Vec::new();
macro_rules! cb { ($name:ident, $tag:expr) => {
    extern "C" fn $name(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64, a7: u64, a8: u64, a9: u64, a10: u64, a11: u64) {
        { let v=[a0,a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11]; eprintln!("{} called", $tag); for (i,x) in v.iter().enumerate() { eprintln!("   arg{} = {:x}  -> {}", i, x, peek(*x)); } } unsafe { (*std::ptr::addr_of_mut!(LOG)).push(Call { which: $tag, args: [a0,a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11] }); }
    } } }
static mut UFILL: i64 = i64::MIN; static mut SET5: i64 = i64::MIN; static mut W: usize = 4;
cb!(uini_rec, "UINI");
cb!(ugen_rec, "UGEN");
unsafe fn sym(h: *mut c_void, n: &str) -> *mut c_void { let c = format!("{}\0", n); let p = GetProcAddress(h, c.as_ptr()); assert!(!p.is_null(), "no sym {n}"); p }
type Cb = extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64);
struct Buf { b: Vec<u8>, w: usize }
impl Buf {
    fn new(n: usize, w: usize, pad: u8) -> Self { Buf { b: vec![if let Ok(v) = std::env::var("PAD") { v.parse::<u8>().unwrap() } else { pad }; n * w + 64], w } }
    fn set(&mut self, i: usize, v: i64) { let o = i * self.w; if self.w == 4 { self.b[o..o+4].copy_from_slice(&(v as i32).to_le_bytes()) } else { self.b[o..o+8].copy_from_slice(&v.to_le_bytes()) } }
    fn get(&self, i: usize) -> i64 { let o = i * self.w; if self.w == 4 { i32::from_le_bytes(self.b[o..o+4].try_into().unwrap()) as i64 } else { i64::from_le_bytes(self.b[o..o+8].try_into().unwrap()) } }
    fn p(&mut self) -> *mut c_void { self.b.as_mut_ptr() as *mut c_void }
    fn a(&self) -> u64 { self.b.as_ptr() as u64 }
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (build, kind, genid) = (a[1].as_str(), a[2].as_str(), a[3].parse::<i64>().unwrap());
    let lseedv: i64 = a.get(4).map(|s| s.parse().unwrap()).unwrap_or(1);
    unsafe { if let Ok(v) = std::env::var("UFILL") { UFILL = v.parse().unwrap(); } }
    unsafe { if let Some(v) = a.get(5) { SET5 = v.parse().unwrap(); } }
    let w = if build == "ILP64" { 8 } else { 4 };
    unsafe {
        let dll = format!("C:/Program Files/AMD/AOCL-Windows/amd-rng/lib/{}/rng_amd.dll", build);
        let wd: Vec<u16> = OsStr::new(&dll).encode_wide().chain(Some(0)).collect();
        let h = LoadLibraryW(wd.as_ptr()); assert!(!h.is_null());
        AddVectoredExceptionHandler(1, veh); W = w;
        eprintln!("rng_amd.dll base = {:x}", h as usize);
        let name = if kind == "S" { "srandinitializeuser" } else { "drandinitializeuser" };
        let f = sym(h, name);
        println!("dll={dll} w={w} fn={name} genid={genid}");
        let mut seed = Buf::new(8, w, 0xAA); for i in 0..8 { seed.set(i, 100 + i as i64); }
        let mut lseed = Buf::new(1, w, 0xAA); lseed.set(0, lseedv);
        let mut state = Buf::new(std::env::var("NSTATE").ok().map(|s| s.parse().unwrap()).unwrap_or(4096), w, 0xAA);
        let mut lstate = Buf::new(1, w, 0xAA); lstate.set(0, std::env::var("LSTATE").ok().map(|s| s.parse().unwrap()).unwrap_or(-1));
        let mut info = Buf::new(2, w, 0xAA);
        println!("buffers: seed={:x} lseed={:x} state={:x} lstate={:x} info={:x}", seed.a(), lseed.a(), state.a(), lstate.a(), info.a());
        STATE_P = state.a();
        (*std::ptr::addr_of_mut!(LOG)).clear();
        let (UI, UG): (Cb, Cb) = if std::env::var("SYMTRAMP").is_ok() { (uini_rec, std::mem::transmute(sym_tramp(ugen_rec as usize))) } else if std::env::var("LOWTRAMP").is_ok() { (std::mem::transmute(low_tramp(0, uini_rec as usize)), std::mem::transmute(low_tramp(1, ugen_rec as usize))) } else { (uini_rec, ugen_rec) };
        println!("uini fn addr={:x} ugen fn addr={:x}", UI as usize, UG as usize);
        let call = |genid: i64, seed: &mut Buf, lseed: &mut Buf, state: &mut Buf, lstate: &mut Buf, info: &mut Buf| {
            if w == 4 { let g: extern "C" fn(Cb, Cb, i32, i32, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(f);
                g(UI, UG, genid as i32, 1, seed.p(), lseed.p(), state.p(), lstate.p(), info.p()) }
            else { let g: extern "C" fn(Cb, Cb, i64, i64, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(f);
                g(UI, UG, genid, 1, seed.p(), lseed.p(), state.p(), lstate.p(), info.p()) }
        };
        call(genid, &mut seed, &mut lseed, &mut state, &mut lstate, &mut info);
        println!("query: info={} lstate={} info_bytes_changed={}", info.get(0), lstate.get(0), info.b[..16].iter().filter(|&&x| x != 0xAA).count());
        let ch: Vec<usize> = state.b.iter().enumerate().filter(|(_, &b)| b != 0).map(|(i, _)| i).collect();
        println!("state bytes changed by init: count={} first={:?} last={:?}", ch.len(), ch.first(), ch.last());
        { let nz: Vec<(usize, i64)> = (0..state.b.len() / w).filter(|&i| state.get(i) != 0).map(|i| (i, state.get(i))).collect(); println!("nonzero state slots (index,value) w={}: {:x?}", w, nz); }
        println!("state[0..32] after init = {:02x?}", &state.b[..32]);
        let l = &*std::ptr::addr_of!(LOG);
        println!("callback invocations during query: {}", l.len());
        (*std::ptr::addr_of_mut!(LOG)).clear();
        if std::env::var("PATCH").is_ok() { let a = UG as usize as u64; if let Ok(v) = std::env::var("PATCH0") { state.set(0, v.parse().unwrap()); } state.set(1, (a >> 32) as i64); state.set(2, (a & 0xffff_ffff) as i64); if let Ok(v) = std::env::var("PATCH3") { state.set(3, if v == "hi" { (a >> 32) as i64 } else { v.parse().unwrap() }); } println!("PATCHED state[1]=hi={:x} state[2]=lo={:x} (ugen addr {:x})", a >> 32, a & 0xffff_ffff, a); }
        if std::env::var("GEN").is_ok() {
            // generate: dranduniform / sranduniform(n, a, b, state, x, info)
            let n = 4usize;
            let gname = if kind == "S" { "sranduniform" } else { "dranduniform" };
            let g = sym(h, gname);
            let mut info2 = Buf::new(2, w, 0xAA);
            let xw = if kind == "S" { 4 } else { 8 };
            let mut x = vec![0xAAu8; n * xw + 64];
            if kind == "D" {
                if w == 4 { let f: extern "C" fn(i32, f64, f64, *mut c_void, *mut f64, *mut c_void) = std::mem::transmute(g); f(n as i32, 0.0, 1.0, state.p(), x.as_mut_ptr() as *mut f64, info2.p()) }
                else { let f: extern "C" fn(i64, f64, f64, *mut c_void, *mut f64, *mut c_void) = std::mem::transmute(g); f(n as i64, 0.0, 1.0, state.p(), x.as_mut_ptr() as *mut f64, info2.p()) }
            } else {
                if w == 4 { let f: extern "C" fn(i32, f32, f32, *mut c_void, *mut f32, *mut c_void) = std::mem::transmute(g); f(n as i32, 0.0, 1.0, state.p(), x.as_mut_ptr() as *mut f32, info2.p()) }
                else { let f: extern "C" fn(i64, f32, f32, *mut c_void, *mut f32, *mut c_void) = std::mem::transmute(g); f(n as i64, 0.0, 1.0, state.p(), x.as_mut_ptr() as *mut f32, info2.p()) }
            }
            println!("generate: x addr={:x} info2 info={} x bytes changed={} x[..32]={:02x?}", x.as_ptr() as u64, info2.get(0), x.iter().filter(|&&b| b != 0xAA).count(), &x[..32]);
            let l = &*std::ptr::addr_of!(LOG);
            println!("callback invocations during generate: {}", l.len());
        }
        let l = &*std::ptr::addr_of!(LOG);
        let _ = l;
    }
}
