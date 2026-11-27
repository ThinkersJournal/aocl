// libm_probe <case>   clean-room behavioural probe of libalm.dll: long double (nexttoward) and fc32_t/fc64_t.
use std::arch::asm;
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
extern "system" {
    fn LoadLibraryW(p: *const u16) -> *mut c_void;
    fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void;
}
const DLL: &str = "C:/Program Files/AMD/AOCL-Windows/amd-libm/lib/libalm.dll";
unsafe fn sym(h: *mut c_void, n: &str) -> *mut c_void {
    let c = format!("{}\0", n);
    let p = GetProcAddress(h, c.as_ptr());
    assert!(!p.is_null(), "no symbol {}", n);
    p
}
#[repr(C)] #[derive(Clone, Copy, Debug)] struct C32 { re: f32, im: f32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct C64 { re: f64, im: f64 }
#[repr(C)] #[derive(Clone, Copy)] struct C32x3 { a: f32, b: f32, c: f32 }
fn pack(re: f32, im: f32) -> u64 { (re.to_bits() as u64) | ((im.to_bits() as u64) << 32) }
fn unpack(u: u64) -> (f32, f32) { (f32::from_bits(u as u32), f32::from_bits((u >> 32) as u32)) }

// call f(xmm0=x, xmm1=y, rdx=p) -> xmm0
unsafe fn call_x_y_p(f: *mut c_void, x: f64, y: f64, p: *const u8) -> f64 {
    let mut r = x;
    asm!("sub rsp, 32", "call {f}", "add rsp, 32", f = in(reg) f, inout("xmm0") r, in("xmm1") y, in("rdx") p, clobber_abi("win64"));
    r
}
// float variant: xmm0 low32 = x (f32)
unsafe fn call_xf_y_p(f: *mut c_void, x: f32, y: f64, p: *const u8) -> f32 {
    let mut r = x;
    asm!("sub rsp, 32", "call {f}", "add rsp, 32", f = in(reg) f, inout("xmm0") r, in("xmm1") y, in("rdx") p, clobber_abi("win64"));
    r
}
// ---- 80-bit x87 extended encoding of an f64 (finite normal values only) ----
fn ext80(v: f64) -> [u8; 16] {
    let mut b = [0u8; 16];
    let bits = v.to_bits();
    let sign = (bits >> 63) as u16;
    let e = ((bits >> 52) & 0x7ff) as i32;
    let m = bits & ((1u64 << 52) - 1);
    assert!(e != 0 && e != 0x7ff);
    let ee = (e - 1023 + 16383) as u16;
    let mant = (1u64 << 63) | (m << 11);
    b[..8].copy_from_slice(&mant.to_le_bytes());
    b[8..10].copy_from_slice(&((sign << 15) | ee).to_le_bytes());
    b
}
fn main() {
    let case = std::env::args().nth(1).unwrap();
    unsafe {
        let w: Vec<u16> = OsStr::new(DLL).encode_wide().chain(Some(0)).collect();
        let h = LoadLibraryW(w.as_ptr());
        assert!(!h.is_null());
        println!("case={} dll={}", case, DLL);
        let nt = sym(h, "amd_nexttoward"); let ntf = sym(h, "amd_nexttowardf");
        let ce = sym(h, "amd_cexp"); let cef = sym(h, "amd_cexpf");
        let cl = sym(h, "amd_clog"); let clf = sym(h, "amd_clogf");
        let cp = sym(h, "amd_cpow"); let cpf = sym(h, "amd_cpowf");
        let eps = f64::EPSILON; let epsf = f32::EPSILON;
        match case.as_str() {
        // ---------- 1. long double ----------
        "nt_f64" => { // positive hypothesis: y is a plain f64 in xmm1
            let f: extern "C" fn(f64, f64) -> f64 = std::mem::transmute(nt);
            for (x, y, name) in [(1.0, 2.0, "up"), (1.0, 0.5, "down"), (1.0, 1.0 + eps, "up_1ulp_target"), (1.0, 1.0, "equal"), (-1.0, -2.0, "neg_down"), (-1.0, 0.0, "neg_to_zero"), (1.0, f64::INFINITY, "inf")] {
                let r = f(x, y);
                println!("nexttoward({x},{y}) [{name}] = {r:e} bits={:016x} delta_from_x={:e}", r.to_bits(), r - x);
            }
            println!("expect up: 1+eps={:e}; down: 1-eps/2 -> delta {:e}", 1.0 + eps, -eps / 2.0);
            let g: extern "C" fn(f32, f64) -> f32 = std::mem::transmute(ntf);
            for (x, y, name) in [(1.0f32, 2.0, "up"), (1.0, 0.5, "down"), (1.0, 1.0 + 2.0f64.powi(-40), "up_by_tiny_f64_only"), (1.0, 1.0, "equal")] {
                let r = g(x, y);
                println!("nexttowardf({x},{y}) [{name}] = {r:e} bits={:08x} delta_from_x={:e}", r.to_bits(), r - x);
            }
            println!("expect up: delta {:e}; down: delta {:e}", epsf, -epsf / 2.0);
        }
        "nt_f32bits" => { // wrong: y passed as f32 (xmm1 low 32 bits = f32 bits, upper bits unspecified by caller -> zeroed here)
            let f: extern "C" fn(f64, f32) -> f64 = std::mem::transmute(nt);
            for (x, y) in [(1.0f64, 2.0f32), (1.0, 0.5)] { let r = f(x, y); println!("WRONG y=f32 nexttoward({x},{y}) = delta {:e}", r - x); }
            let g: extern "C" fn(f32, f32) -> f32 = std::mem::transmute(ntf);
            for (x, y) in [(1.0f32, 2.0f32), (1.0, 0.5)] { let r = g(x, y); println!("WRONG y=f32 nexttowardf({x},{y}) = delta {:e}", r - x); }
            // f32 2.0 = 0x40000000 read as low bits of an f64 -> positive denormal ~5.3e-315 -> direction DOWN
        }
        "nt_ptr80" => { // alt hypothesis: y by reference (rdx) to 80-bit extended; xmm1 set to the OPPOSITE direction. Result follows the register that is read.
            let up = ext80(2.0); let down = ext80(0.5);
            // (a) rdx -> ext(2.0) [up], xmm1 = 0.5 [down]
            let r = call_x_y_p(nt, 1.0, 0.5, up.as_ptr()); println!("rdx=ext80(2.0) xmm1=0.5 : delta {:e} (up=+{:e}, down={:e})", r - 1.0, eps, -eps / 2.0);
            // (b) rdx -> ext(0.5) [down], xmm1 = 2.0 [up]
            let r = call_x_y_p(nt, 1.0, 2.0, down.as_ptr()); println!("rdx=ext80(0.5) xmm1=2.0 : delta {:e}", r - 1.0);
            // (c) rdx -> ext(2.0), xmm1 = NaN
            let r = call_x_y_p(nt, 1.0, f64::NAN, up.as_ptr()); println!("rdx=ext80(2.0) xmm1=NaN : result {:e}", r);
            // (d) rdx = NULL, xmm1 = 2.0: no crash == rdx not dereferenced
            let r = call_x_y_p(nt, 1.0, 2.0, std::ptr::null()); println!("rdx=NULL xmm1=2.0 : delta {:e} (no crash => rdx not dereferenced)", r - 1.0);
            // float entry
            let r = call_xf_y_p(ntf, 1.0, 0.5, up.as_ptr()); println!("F rdx=ext80(2.0) xmm1=0.5 : delta {:e} (up=+{:e}, down={:e})", r - 1.0, epsf, -epsf / 2.0);
            let r = call_xf_y_p(ntf, 1.0, 2.0, down.as_ptr()); println!("F rdx=ext80(0.5) xmm1=2.0 : delta {:e}", r - 1.0);
            let r = call_xf_y_p(ntf, 1.0, 2.0, std::ptr::null()); println!("F rdx=NULL xmm1=2.0 : delta {:e} (no crash)", r - 1.0);
        }
        "nt_ptrf64" => { // alt: y by pointer to f64 in rcx slot? (second arg slot is rdx) -> same as (d) above; here try x in xmm0, y as *const f64 in rdx via fn type
            let up = 2.0f64; let down = 0.5f64;
            let f: extern "C" fn(f64, *const f64) -> f64 = std::mem::transmute(nt);
            println!("WRONG y=*const f64: up-ptr delta {:e}, down-ptr delta {:e} (xmm1 unspecified)", f(1.0, &up) - 1.0, f(1.0, &down) - 1.0);
        }
        // ---------- 2. complex ----------
        "c32" => {
            let f_struct = |p: *mut c_void, a: C32| { let f: extern "C" fn(C32) -> C32 = std::mem::transmute(p); f(a) };
            let f_u64 = |p: *mut c_void, a: u64| { let f: extern "C" fn(u64) -> u64 = std::mem::transmute(p); f(a) };
            let two = |p: *mut c_void, a: C32, b: C32| { let f: extern "C" fn(C32, C32) -> C32 = std::mem::transmute(p); f(a, b) };
            let two_u64 = |p: *mut c_void, a: u64, b: u64| { let f: extern "C" fn(u64, u64) -> u64 = std::mem::transmute(p); f(a, b) };
            let pi2 = std::f32::consts::FRAC_PI_2; let e = std::f32::consts::E;
            let un: Vec<(&str, *mut c_void, C32)> = vec![
                ("cexpf(0+0i)", cef, C32{re:0.0,im:0.0}), ("cexpf(0+i*pi/2)", cef, C32{re:0.0,im:pi2}),
                ("cexpf(1+2i) [non-symmetric; exp(1)*(cos2, sin2) = (-1.1312, 2.4717)]", cef, C32{re:1.0,im:2.0}),
                ("clogf(1+0i)", clf, C32{re:1.0,im:0.0}), ("clogf(e+0i)", clf, C32{re:e,im:0.0}),
                ("clogf(3+4i) [non-symmetric; (ln5, atan2(4,3)) = (1.6094, 0.9273)]", clf, C32{re:3.0,im:4.0})];
            for (n, p, a) in &un {
                let r = f_struct(*p, *a); let ru = f_u64(*p, pack(a.re, a.im)); let (rr, ri) = unpack(ru);
                println!("{n}: struct->({:e}, {:e}) | u64(re in low32)->({:e}, {:e}) | identical_bits={}", r.re, r.im, rr, ri, r.re.to_bits()==rr.to_bits() && r.im.to_bits()==ri.to_bits());
                // control: swapped fields (im in low32)
                let (sr, si) = unpack(f_u64(*p, pack(a.im, a.re)));
                println!("   CONTROL swapped input (im low32) -> ({:e}, {:e})  differs_from_positive={}", sr, si, (sr.to_bits(), si.to_bits()) != (r.re.to_bits(), r.im.to_bits()));
            }
            let bin: Vec<(&str, C32, C32)> = vec![("cpowf(2+0i,3+0i) [8+0i]", C32{re:2.0,im:0.0}, C32{re:3.0,im:0.0}), ("cpowf(i,2+0i) [-1+0i]", C32{re:0.0,im:1.0}, C32{re:2.0,im:0.0}), ("cpowf(1+2i,2+0i) [non-symm: -3+4i]", C32{re:1.0,im:2.0}, C32{re:2.0,im:0.0}), ("cpowf(2+0i,0+1i) [non-symm: exp(i ln2)=(0.7692,0.6390)]", C32{re:2.0,im:0.0}, C32{re:0.0,im:1.0})];
            for (n, a, b) in &bin {
                let r = two(cpf, *a, *b); let (rr, ri) = unpack(two_u64(cpf, pack(a.re, a.im), pack(b.re, b.im)));
                let (sr, si) = unpack(two_u64(cpf, pack(a.im, a.re), pack(b.im, b.re)));
                let (xr, xi) = unpack(two_u64(cpf, pack(b.re, b.im), pack(a.re, a.im)));
                println!("{n}: struct->({:e}, {:e}) | u64->({:e}, {:e}) | CONTROL swapped fields->({:e}, {:e}) | CONTROL swapped args->({:e}, {:e})", r.re, r.im, rr, ri, sr, si, xr, xi);
            }
        }
        "c32_ret_xmm0" => { // control: where is the return value? read xmm0 (f64 bits) and rax (u64)
            let a = pack(1.0, 2.0);
            let fr: extern "C" fn(u64) -> u64 = std::mem::transmute(cef);
            let fx: extern "C" fn(u64) -> f64 = std::mem::transmute(cef);
            let rax = fr(a); let x0 = fx(a).to_bits();
            println!("cexpf(1+2i): rax={:016x} -> {:?}", rax, unpack(rax));
            println!("             xmm0(low64 as f64 bits)={:016x} -> {:?}  (== rax? {})", x0, unpack(x0), x0 == rax);
            // arg in xmm0 instead of rcx (control): f(f64 with packed bits)
            let fa: extern "C" fn(f64) -> u64 = std::mem::transmute(cef);
            println!("CONTROL arg packed in xmm0 (rcx unspecified): rax={:016x} -> {:?}", fa(f64::from_bits(a)), unpack(fa(f64::from_bits(a))));
            // control: two f32 args in xmm0, xmm1
            let f2: extern "C" fn(f32, f32) -> u64 = std::mem::transmute(cef);
            println!("CONTROL args as (f32,f32) in xmm0,xmm1: rax -> {:?}", unpack(f2(1.0, 2.0)));
        }
        "c32_wrongsize" => { // control: 12-byte struct (passed by pointer by the ABI) -> rcx = pointer, read as data
            let t = C32x3 { a: 1.0, b: 2.0, c: 3.0 };
            let f: extern "C" fn(*const C32x3) -> u64 = std::mem::transmute(cef);
            println!("CONTROL pointer in rcx: -> {:?} (pointer value low32 as f32 = {:e})", unpack(f(&t)), f32::from_bits(&t as *const _ as usize as u32));
        }
        "c64" => {
            let pi2 = std::f64::consts::FRAC_PI_2; let e = std::f64::consts::E;
            let un: Vec<(&str, *mut c_void, C64)> = vec![
                ("cexp(0+0i)", ce, C64{re:0.0,im:0.0}), ("cexp(0+i*pi/2)", ce, C64{re:0.0,im:pi2}),
                ("cexp(1+2i) [non-symmetric; (-1.1312043837568135, 2.4717266720048188)]", ce, C64{re:1.0,im:2.0}),
                ("clog(1+0i)", cl, C64{re:1.0,im:0.0}), ("clog(e+0i)", cl, C64{re:e,im:0.0}),
                ("clog(3+4i) [non-symmetric; (1.6094379124341003, 0.9272952180016122)]", cl, C64{re:3.0,im:4.0})];
            for (n, p, a) in &un {
                // native Rust struct by value (MS x64: >8 bytes -> by hidden reference, result via hidden sret pointer)
                let f: extern "C" fn(C64) -> C64 = std::mem::transmute(*p); let r = f(*a);
                // explicit hidden-pointer hypothesis: rcx = ret buffer, rdx = pointer to copy of arg, rax = ret buffer
                let mut buf = [0xAAu8; 48]; let mut inp = [0xAAu8; 32];
                inp[..8].copy_from_slice(&a.re.to_le_bytes()); inp[8..16].copy_from_slice(&a.im.to_le_bytes());
                let g: extern "C" fn(*mut u8, *const u8) -> *mut u8 = std::mem::transmute(*p);
                let rax = g(buf.as_mut_ptr().add(16), inp.as_ptr());
                let written = buf.iter().enumerate().filter(|(_, &b)| b != 0xAA).map(|(i, _)| i).collect::<Vec<_>>();
                let re = f64::from_le_bytes(buf[16..24].try_into().unwrap()); let im = f64::from_le_bytes(buf[24..32].try_into().unwrap());
                let inp_changed = inp[16..].iter().any(|&b| b != 0xAA) || inp[..16].iter().zip(a.re.to_le_bytes().iter().chain(a.im.to_le_bytes().iter())).any(|(x,y)| x!=y);
                println!("{n}: struct->({:e}, {:e}) | explicit sret -> ({:e}, {:e}) rax==retptr:{} bytes_written_offsets={:?} (ret buf starts at 16; 16..32 expected) input_modified={}", r.re, r.im, re, im, rax == buf.as_mut_ptr().wrapping_add(16), if written.is_empty() { (0,0) } else { (written[0], *written.last().unwrap()) }, inp_changed);
                // control swapped
                let mut b2 = [0u8; 32]; let mut i2 = [0u8; 16]; i2[..8].copy_from_slice(&a.im.to_le_bytes()); i2[8..].copy_from_slice(&a.re.to_le_bytes());
                g(b2.as_mut_ptr(), i2.as_ptr());
                println!("   CONTROL swapped fields -> ({:e}, {:e})", f64::from_le_bytes(b2[..8].try_into().unwrap()), f64::from_le_bytes(b2[8..16].try_into().unwrap()));
            }
            let bin: Vec<(&str, C64, C64)> = vec![("cpow(2+0i,3+0i) [8+0i]", C64{re:2.0,im:0.0}, C64{re:3.0,im:0.0}), ("cpow(i,2+0i) [-1+0i]", C64{re:0.0,im:1.0}, C64{re:2.0,im:0.0}), ("cpow(1+2i,2+0i) [non-symm: -3+4i]", C64{re:1.0,im:2.0}, C64{re:2.0,im:0.0}), ("cpow(2+0i,0+1i) [non-symm: (0.7692389013639721, 0.6389612763136348)]", C64{re:2.0,im:0.0}, C64{re:0.0,im:1.0})];
            for (n, a, b) in &bin {
                let f: extern "C" fn(C64, C64) -> C64 = std::mem::transmute(cp); let r = f(*a, *b);
                let mut buf = [0xAAu8; 48]; let mut ia = [0u8; 16]; let mut ib = [0u8; 16];
                ia[..8].copy_from_slice(&a.re.to_le_bytes()); ia[8..].copy_from_slice(&a.im.to_le_bytes());
                ib[..8].copy_from_slice(&b.re.to_le_bytes()); ib[8..].copy_from_slice(&b.im.to_le_bytes());
                // explicit: rcx=ret, rdx=ptr a, r8=ptr b
                let g: extern "C" fn(*mut u8, *const u8, *const u8) -> *mut u8 = std::mem::transmute(cp);
                let rax = g(buf.as_mut_ptr().add(16), ia.as_ptr(), ib.as_ptr());
                let re = f64::from_le_bytes(buf[16..24].try_into().unwrap()); let im = f64::from_le_bytes(buf[24..32].try_into().unwrap());
                let written = buf.iter().enumerate().filter(|(_, &b)| b != 0xAA).map(|(i, _)| i).collect::<Vec<_>>();
                // swapped args control
                let mut b2 = [0u8; 32]; g(b2.as_mut_ptr(), ib.as_ptr(), ia.as_ptr());
                println!("{n}: struct->({:e}, {:e}) | explicit sret(rcx=ret,rdx=&a,r8=&b) -> ({:e}, {:e}) rax==retptr:{} written={:?}..{:?} | CONTROL swapped args -> ({:e}, {:e})", r.re, r.im, re, im, rax == buf.as_mut_ptr().wrapping_add(16), written.first(), written.last(), f64::from_le_bytes(b2[..8].try_into().unwrap()), f64::from_le_bytes(b2[8..16].try_into().unwrap()));
            }
        }
        "c64_xmm_ctl" => { // CONTROL (expected to crash if the DLL uses hidden pointers): args by value in xmm0=re, xmm1=im, rcx -> scratch buf, rdx = NULL
            let mut buf = [0xAAu8; 64];
            let (mut x0, mut x1) = (0.0f64, 1.0f64);
            asm!("sub rsp, 32", "call {f}", "add rsp, 32", f = in(reg) ce, inout("xmm0") x0, inout("xmm1") x1, in("rcx") buf.as_mut_ptr(), in("rdx") 0usize, clobber_abi("win64"));
            println!("CONTROL xmm-args survived (no crash): xmm0={x0:e} xmm1={x1:e} buf_changed={}", buf.iter().any(|&b| b != 0xAA));
        }
        "c64_fc32_ctl" => { // CONTROL: pass 8-byte packed value where the 16-byte aggregate pointer is expected
            let f: extern "C" fn(u64) -> u64 = std::mem::transmute(ce);
            println!("CONTROL fc32-sized value to amd_cexp: {:016x}", f(pack(1.0, 2.0)));
        }
        _ => panic!("case"),
        }
    }
}
