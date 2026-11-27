// uprobe: contained clean-room probe of (s|d)randinitializeuser and the non-user control (s|d)randinitialize.
// parent mode:  uprobe parent <timeout_s> <child args...>   spawns itself as child (own process), enforces timeout, prints exit code
// child  mode:  uprobe child k=v ...
// Keys: build=LP64|ILP64 fn=duser|suser|dinit|sinit|selftest slots=<N>|auto|auto-1 lstate=<int>|slots lseed=<int> nseed=<n>
//       genid= subid= uini=fn|fnb|t:<hexaddr> ugen=fn|fnb|t:<hexaddr> uini_writes=0|1 fill=<hex byte> qlstate=-1 (control query)
use std::collections::HashMap;
use std::ffi::{c_void, OsStr};
use std::io::Write;
use std::os::windows::ffi::OsStrExt;

extern "system" {
    fn LoadLibraryW(p: *const u16) -> *mut c_void;
    fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void;
    fn VirtualAlloc(a: *mut c_void, sz: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(a: *mut c_void, sz: usize, prot: u32, old: *mut u32) -> i32;
    fn AddVectoredExceptionHandler(first: u32, h: extern "system" fn(*mut ExcPtrs) -> i32) -> *mut c_void;
    fn ExitProcess(c: u32) -> !;
}
#[repr(C)] struct ExcRec { code: u32, flags: u32, rec: *mut c_void, addr: *mut c_void, n: u32, _p: u32, info: [usize; 15] }
#[repr(C)] struct ExcPtrs { rec: *mut ExcRec, ctx: *mut u8 }

static mut STATE_PTR: usize = 0;
static mut DLL_BASE: usize = 0;
static mut W: usize = 4;
static mut NCALL: [u32; 2] = [0, 0];
static mut UINI_WRITES: bool = false;
static mut UINI_ENTRY: Vec<i64> = Vec::new();
static mut UINI_LSTATE_ARG: i64 = 0;
static mut ADDR_STATE: u64 = 0;
static mut UINI_N: i64 = -1;

fn out(s: &str) { println!("{s}"); let _ = std::io::stdout().flush(); }

extern "system" fn veh(p: *mut ExcPtrs) -> i32 {
    unsafe {
        let r = &*(*p).rec;
        if r.code != 0xC0000005 { return 0; }
        let rip = *((*p).ctx.add(0xF8) as *const usize);
        let a = r.info[1];
        let inside = if rip >= DLL_BASE && rip < DLL_BASE + 0x4000000 { format!("inside rng_amd.dll (RVA 0x{:x})", rip - DLL_BASE) } else { "OUTSIDE rng_amd.dll (harness or other module)".to_string() };
        out(&format!("FAULT access={} ({}) addr=0x{:x} state_ptr=0x{:x} fault_offset_from_state_start={} bytes ({} slots of {}) rip {}",
            r.info[0], if r.info[0] == 1 { "WRITE" } else if r.info[0] == 0 { "READ" } else { "DEP" }, a, STATE_PTR,
            a as i64 - STATE_PTR as i64, (a as i64 - STATE_PTR as i64) / W as i64, W, inside));
        ExitProcess(0xC0000005);
    }
}

unsafe fn rd(p: u64) -> i64 { if W == 4 { *(p as *const i32) as i64 } else { *(p as *const i64) } }
unsafe fn wr(p: u64, v: i64) { if W == 4 { *(p as *mut i32) = v as i32 } else { *(p as *mut i64) = v } }

extern "C" fn uini_a(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) { uini_common(1, a0, a1, a2, a3, a4, a5, a6) }
extern "C" fn uini_b(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) { uini_common(2, a0, a1, a2, a3, a4, a5, a6) }
#[inline(never)]
fn uini_common(id: u32, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) {
    unsafe {
        NCALL[0] += 1;
        let _ = (a1, a2, a3);
        UINI_LSTATE_ARG = rd(a5);
        UINI_ENTRY = (0..1).map(|i| rd(a4 + (i * W) as u64)).collect();
        out(&format!("UINI#{} id={} genid={} subid={} lseed={} lstate_arg={} info={} state_ptr_matches_caller={} state[0] at UINI entry={:x?}",
            NCALL[0], id, rd(a0), rd(a1), rd(a3), rd(a5), rd(a6), a4 == ADDR_STATE, UINI_ENTRY));
        if UINI_N >= 0 {
            wr(a4, UINI_N);
            let lim = UINI_N.min(if UINI_LSTATE_ARG > 0 { UINI_LSTATE_ARG } else { 1 });
            for i in 1..lim { wr(a4 + i as u64 * W as u64, 0x7000 + i); }
            out(&format!("UINI (protocol probe) wrote state[0]={} and state[1..{}]=0x7000+i", UINI_N, lim));
        }
        if UINI_WRITES { for i in 0..8u64 { wr(a4 + i * W as u64, 0x5100 + i as i64); } out("UINI wrote state[0..8]=0x5100+i"); }
    }
}
static mut GEN_KIND_S: bool = false;
extern "C" fn ugen_a(a0: u64, a1: u64, a2: u64, a3: u64) {
    unsafe {
        NCALL[1] += 1;
        let n = rd(a0);
        out(&format!("UGEN(a) called n={} a1==state:{} a2(r)={:x} a3(info)={:x}", n, a1 == ADDR_STATE, a2, a3));
        // documented 4-parameter shape: write r[i]=(i+1)/8 at the build's element width (S float, D double), *info=0
        for i in 0..n as u64 { let v = 0.125 * (i + 1) as f64; if GEN_KIND_S { *((a2 + 4 * i) as *mut f32) = v as f32 } else { *((a2 + 8 * i) as *mut f64) = v } }
        wr(a3, 0);
    }
}
extern "C" fn ugen_b(a0: u64, a1: u64, a2: u64, a3: u64) { unsafe { NCALL[1] += 1; out(&format!("UGEN(b) called a0={:x} a1={:x} a2={:x} a3={:x}", a0, a1, a2, a3)); } }

unsafe fn tramp(hint: usize, target: usize) -> usize {
    let p = VirtualAlloc(hint as *mut c_void, 0x1000, 0x3000, 0x40) as usize;
    if p == 0 { out(&format!("TRAMP alloc at 0x{:x} FAILED", hint)); return 0; }
    let mut code = vec![0x48u8, 0xB8]; code.extend_from_slice(&(target as u64).to_le_bytes()); code.extend_from_slice(&[0xFF, 0xE0]);
    // code placed at page offset 0x123 so lo32 of the address is non-zero (zero writes are invisible in a zero-filled buffer)
    std::ptr::copy_nonoverlapping(code.as_ptr(), (p + 0x123) as *mut u8, code.len());
    p + 0x123
}
unsafe fn sym(h: *mut c_void, n: &str) -> *mut c_void { let c = format!("{}\0", n); let p = GetProcAddress(h, c.as_ptr()); assert!(!p.is_null(), "no sym {n}"); p }

fn child(kv: HashMap<String, String>) {
    let g = |k: &str, d: &str| kv.get(k).cloned().unwrap_or(d.to_string());
    let build = g("build", "LP64"); let f = g("fn", "duser");
    let w: usize = if build == "ILP64" { 8 } else { 4 };
    unsafe {
        W = w;
        UINI_WRITES = g("uini_writes", "0") == "1"; UINI_N = g("uini_n", "-1").parse().unwrap();
        AddVectoredExceptionHandler(1, veh);
        let fill = u8::from_str_radix(&g("fill", "00"), 16).unwrap();
        let alloc_state = |bytes: usize| -> (usize, usize, usize) {
            // 64 MiB RESERVED (inaccessible) region; only the pages holding the state are COMMITTED read/write,
            // ending flush with the end of the state. Every byte before (1 MiB) and after (up to ~47 MiB) is uncommitted:
            // any out-of-bounds access in that window faults exactly (a one-page guard alone lets a far overrun land in other mapped memory).
            let total = 64usize << 20;
            let base = VirtualAlloc(std::ptr::null_mut(), total, 0x2000, 0x01) as usize; assert!(base != 0);
            let pages = (bytes + 4095) / 4096;
            let cs = base + (1usize << 20); let ce = cs + pages * 4096;
            assert!(!VirtualAlloc(cs as *mut c_void, pages * 4096, 0x1000, 0x04).is_null());
            std::ptr::write_bytes(cs as *mut u8, fill, ce - cs);
            (ce - bytes, cs, ce)
        };
        if f == "selftest" {
            let (sp, _cs, _ce) = alloc_state(400); STATE_PTR = sp;
            out(&format!("selftest: writing slot 99 (last in-bounds of 100) at {:x}", sp + 396));
            *((sp + 396) as *mut u32) = 1; out("selftest: in-bounds write OK");
            out("selftest: writing 1 byte one past the end (must fault)");
            *((sp + 400) as *mut u8) = 1; out("selftest: NO FAULT (guard ineffective)"); return;
        }
        let dll = format!("C:/Program Files/AMD/AOCL-Windows/amd-rng/lib/{}/rng_amd.dll", build);
        let wd: Vec<u16> = OsStr::new(&dll).encode_wide().chain(Some(0)).collect();
        let h = LoadLibraryW(wd.as_ptr()); assert!(!h.is_null()); DLL_BASE = h as usize;
        let user = f.ends_with("user");
        let name = match f.as_str() { "duser" => "drandinitializeuser", "suser" => "srandinitializeuser", "dinit" => "drandinitialize", "sinit" => "srandinitialize", _ => panic!() };
        let fp = sym(h, name);
        out(&format!("CASE {:?}", kv));
        out(&format!("dll={} int_width={} fn={} fn_addr=0x{:x} dll_base=0x{:x}", dll, w, name, fp as usize, DLL_BASE));
        let nseed: usize = g("nseed", "8").parse().unwrap();
        let mut seed = vec![0u8; (nseed.max(700)) * w];
        let setv = |b: &mut Vec<u8>, i: usize, v: i64| { if w == 4 { b[i * 4..i * 4 + 4].copy_from_slice(&(v as i32).to_le_bytes()) } else { b[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes()) } };
        let getv = |b: &Vec<u8>, i: usize| -> i64 { if w == 4 { i32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap()) as i64 } else { i64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap()) } };
        for i in 0..seed.len() / w { setv(&mut seed, i, 100 + i as i64); }
        let mut lseed = vec![0u8; 8]; let mut lstate = vec![0u8; 8]; let mut info = vec![0xAAu8; 16];
        let genid: i64 = g("genid", "5").parse().unwrap(); let subid: i64 = g("subid", "7").parse().unwrap();
        setv(&mut lseed, 0, g("lseed", "3").parse().unwrap());
        // optional control query: lstate=-1 and lseed=-1 first
        let mut slots_s = g("slots", "100");
        let call = |uini: usize, ugen: usize, statep: *mut c_void, seedp: *mut c_void, lseedp: *mut c_void, lstatep: *mut c_void, infop: *mut c_void| {
            if user {
                if w == 4 { let fnn: extern "C" fn(usize, usize, i32, i32, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(fp); fnn(uini, ugen, genid as i32, subid as i32, seedp, lseedp, statep, lstatep, infop) }
                else { let fnn: extern "C" fn(usize, usize, i64, i64, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(fp); fnn(uini, ugen, genid, subid, seedp, lseedp, statep, lstatep, infop) }
            } else {
                if w == 4 { let fnn: extern "C" fn(i32, i32, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(fp); fnn(genid as i32, subid as i32, seedp, lseedp, statep, lstatep, infop) }
                else { let fnn: extern "C" fn(i64, i64, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) = std::mem::transmute(fp); fnn(genid, subid, seedp, lseedp, statep, lstatep, infop) }
            }
        };
        let resolve = |spec: &str, a: usize, b: usize| -> usize {
            if spec == "fn" { a } else if spec == "fnb" { b } else if let Some(h) = spec.strip_prefix("t:") { let hint = usize::from_str_radix(h, 16).unwrap(); tramp(hint, a) } else { panic!() }
        };
        let (uini, ugen) = if user {
            let u = resolve(&g("uini", "fn"), uini_a as usize, uini_b as usize);
            let ug = resolve(&g("ugen", "fn"), ugen_a as usize, ugen_b as usize);
            out(&format!("uini_addr=0x{:x} (hi32=0x{:x} lo32=0x{:x})  ugen_addr=0x{:x} (hi32=0x{:x} lo32=0x{:x})", u, u >> 32, u & 0xffffffff, ug, ug >> 32, ug & 0xffffffff));
            (u, ug)
        } else { (0, 0) };
        if slots_s == "auto" || slots_s == "auto-1" {
            // guide: lstate=-1 and lseed=-1 -> both set to required values, function returns
            setv(&mut lseed, 0, -1); setv(&mut lstate, 0, -1);
            let (sp, _, _) = alloc_state(64); STATE_PTR = sp; ADDR_STATE = sp as u64;
            call(uini, ugen, sp as *mut c_void, seed.as_mut_ptr() as *mut c_void, lseed.as_mut_ptr() as *mut c_void, lstate.as_mut_ptr() as *mut c_void, info.as_mut_ptr() as *mut c_void);
            let (ls, lse, inf) = (getv(&lstate, 0), getv(&lseed, 0), getv(&info, 0));
            out(&format!("QUERY (lstate=-1,lseed=-1): lstate_out={} lseed_out={} info={}", ls, lse, inf));
            let n = if slots_s == "auto" { ls } else { ls - 1 };
            slots_s = n.to_string();
            setv(&mut lseed, 0, lse); setv(&mut lstate, 0, ls);
            for b in info.iter_mut() { *b = 0xAA; }
            NCALL = [0, 0];
        }
        let slots: usize = slots_s.parse().unwrap();
        let bytes = slots * w;
        let (sp, cs, ce) = alloc_state(bytes.max(1)); STATE_PTR = sp; ADDR_STATE = sp as u64;
        let lst = g("lstate", "slots"); let lv: i64 = if lst == "slots" { if g("slots", "").starts_with("auto") { getv(&lstate, 0) } else { slots as i64 } } else { lst.parse().unwrap() };
        setv(&mut lstate, 0, lv);
        out(&format!("CALL: genid={} subid={} lseed_in={} lstate_in={} state_buffer: ptr=0x{:x} slots={} bytes={} (uncommitted (faulting) memory starts exactly at state_ptr+{}) fill=0x{:02x}", genid, subid, getv(&lseed, 0), lv, sp, slots, bytes, bytes, fill));
        call(uini, ugen, sp as *mut c_void, seed.as_mut_ptr() as *mut c_void, lseed.as_mut_ptr() as *mut c_void, lstate.as_mut_ptr() as *mut c_void, info.as_mut_ptr() as *mut c_void);
        // scan
        let mut changed: Vec<(usize, i64)> = Vec::new();
        for i in 0..slots { let p = sp + i * w; let v = if w == 4 { *(p as *const u32) as u64 } else { *(p as *const u64) }; let pat = u64::from_ne_bytes([fill; 8]) & (if w == 4 { 0xffff_ffff } else { u64::MAX }); if v != pat { changed.push((i, v as i64)); } }
        let pre_changed = (cs..sp).filter(|&a| *(a as *const u8) != fill).count();
        let _ = ce;
        out(&format!("RESULT info={} lstate_after={} lseed_after={} UINI_calls={} UGEN_calls={} bytes_in_prepad_changed={}", getv(&info, 0) , getv(&lstate, 0), getv(&lseed, 0), NCALL[0], NCALL[1], pre_changed));
        let used = changed.last().map(|c| (c.0 + 1) * w).unwrap_or(0);
        out(&format!("CHANGED slots={} last_changed_slot={:?} last_changed_byte_end={} of buffer_bytes={} (lstate*w={})", changed.len(), changed.last().map(|c| c.0), used, bytes, lv * w as i64));
        for c in changed.iter().take(40) { out(&format!("  state[{}] (byte off 0x{:x}) = 0x{:x}", c.0, c.0 * w, if w == 4 { c.1 as u32 as u64 } else { c.1 as u64 })); }
        if changed.len() > 40 { out(&format!("  ... {} more", changed.len() - 40)); }
        if g("gen", "0") == "1" {
            GEN_KIND_S = f == "suser";
            let gname = if GEN_KIND_S { "sranduniform" } else { "dranduniform" };
            let gp = sym(h, gname);
            let n = 4usize; let xw = if GEN_KIND_S { 4 } else { 8 };
            let mut x = vec![0xAAu8; n * xw + 64];
            let mut info2 = vec![0xAAu8; 16];
            if !GEN_KIND_S {
                if w == 4 { let q: extern "C" fn(i32, f64, f64, *mut c_void, *mut f64, *mut c_void) = std::mem::transmute(gp); q(n as i32, 0.5, 2.0, sp as *mut c_void, x.as_mut_ptr() as *mut f64, info2.as_mut_ptr() as *mut c_void) }
                else { let q: extern "C" fn(i64, f64, f64, *mut c_void, *mut f64, *mut c_void) = std::mem::transmute(gp); q(n as i64, 0.5, 2.0, sp as *mut c_void, x.as_mut_ptr() as *mut f64, info2.as_mut_ptr() as *mut c_void) }
                let v: Vec<f64> = (0..n).map(|i| f64::from_le_bytes(x[8 * i..8 * i + 8].try_into().unwrap())).collect();
                out(&format!("GEN {}(n=4,a=0.5,b=2.0) NO harness patch: info={} UGEN_calls={} x={:?} expected(0.5+1.5*(i+1)/8)={:?} bytes_changed_beyond_n*8={}", gname, getv(&info2, 0), NCALL[1], v, (0..n).map(|i| 0.5 + 1.5 * 0.125 * (i + 1) as f64).collect::<Vec<_>>(), x[n * 8..].iter().filter(|&&b| b != 0xAA).count()));
            } else {
                if w == 4 { let q: extern "C" fn(i32, f32, f32, *mut c_void, *mut f32, *mut c_void) = std::mem::transmute(gp); q(n as i32, 0.5, 2.0, sp as *mut c_void, x.as_mut_ptr() as *mut f32, info2.as_mut_ptr() as *mut c_void) }
                else { let q: extern "C" fn(i64, f32, f32, *mut c_void, *mut f32, *mut c_void) = std::mem::transmute(gp); q(n as i64, 0.5, 2.0, sp as *mut c_void, x.as_mut_ptr() as *mut f32, info2.as_mut_ptr() as *mut c_void) }
                let v: Vec<f32> = (0..n).map(|i| f32::from_le_bytes(x[4 * i..4 * i + 4].try_into().unwrap())).collect();
                out(&format!("GEN {}(n=4,a=0.5,b=2.0) NO harness patch: info={} UGEN_calls={} x={:?} expected={:?} bytes_changed_beyond_n*4={}", gname, getv(&info2, 0), NCALL[1], v, (0..n).map(|i| 0.5 + 1.5 * 0.125 * (i + 1) as f32).collect::<Vec<_>>(), x[n * 4..].iter().filter(|&&b| b != 0xAA).count()));
            }
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a[1] == "child" {
        let kv: HashMap<String, String> = a[2..].iter().filter_map(|s| s.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
        child(kv); return;
    }
    let to: u64 = a[2].parse().unwrap();
    let mut c = std::process::Command::new(std::env::current_exe().unwrap()).arg("child").args(&a[3..]).spawn().unwrap();
    let t0 = std::time::Instant::now();
    loop {
        if let Some(st) = c.try_wait().unwrap() { println!("child_exit_code=0x{:x} elapsed_ms={}", st.code().unwrap_or(-1) as u32, t0.elapsed().as_millis()); break; }
        if t0.elapsed().as_secs() >= to { let _ = c.kill(); println!("child_exit_code=TIMEOUT_KILLED"); break; }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
