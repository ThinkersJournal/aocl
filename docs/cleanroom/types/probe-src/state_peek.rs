use std::ffi::{c_void, OsStr}; use std::os::windows::ffi::OsStrExt;
extern "system" { fn LoadLibraryW(p: *const u16) -> *mut c_void; fn GetProcAddress(h: *mut c_void, n: *const u8) -> *mut c_void; }
fn main() { unsafe {
    let dll = "C:/Program Files/AMD/AOCL-Windows/amd-rng/lib/LP64/rng_amd.dll";
    let wd: Vec<u16> = OsStr::new(dll).encode_wide().chain(Some(0)).collect();
    let h = LoadLibraryW(wd.as_ptr());
    let f: extern "C" fn(i32, i32, *mut i32, *mut i32, *mut i32, *mut i32, *mut i32) = std::mem::transmute(GetProcAddress(h, b"drandinitialize\0".as_ptr()));
    let mut seed = [12345i32; 4]; let mut ls = 1i32; let mut st = vec![0i32; 100000]; let mut lst = 100i32; let mut info = 0i32;
    f(1, 1, seed.as_mut_ptr(), &mut ls, st.as_mut_ptr(), &mut lst, &mut info);
    println!("genid1 info={info} lstate={lst} state[0..12]={:x?}", &st[..12]);
    for g in [2, 3] { let mut st = vec![0i32; 100000]; let mut lst = 700i32; let mut info = 0i32; f(g, 1, seed.as_mut_ptr(), &mut ls, st.as_mut_ptr(), &mut lst, &mut info); println!("genid{g} info={info} lstate={lst} state[0..12]={:x?}", &st[..12]); }
}}
