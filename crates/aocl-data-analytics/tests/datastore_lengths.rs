//! (3 of these 4 short cases were access violations before the checks.) Guard-page harness for `Datastore::extract_f64`: the library writes the selection's rows x columns with leading dimension `lddata` and
//! checks nothing about the destination. One process per test (tools/safe-api-harness.py); a fault = HOLE. Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_data_analytics::*;
use core::ffi::c_void;
use std::io::Write;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
fn at_guard<R>(n: usize, f: impl FnOnce(&mut [f64]) -> R) -> R {
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        let mut old = 0u32;
        assert_ne!(
            VirtualProtect(
                (base as *mut u8).add(PAGE) as *mut c_void,
                PAGE,
                0x01,
                &mut old
            ),
            0
        );
        let start = (base as *mut u8).add(PAGE - n * 8) as *mut f64;
        for i in 0..n {
            start.add(i).write(0.0);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}
fn store(tag: &str) -> Datastore {
    let mut path = std::env::temp_dir();
    path.push(format!("aocl_da_dsl_{tag}_{}.csv", std::process::id()));
    let mut f = std::fs::File::create(&path).unwrap();
    for r in 0..4 {
        writeln!(f, "{}.0,{}.5,{}.25", r, r, r).unwrap();
    }
    drop(f);
    let mut s = Datastore::new().unwrap();
    s.load_csv(path.to_str().unwrap()).unwrap();
    let _ = std::fs::remove_file(&path);
    s
}
#[test]
fn extract_control() {
    let mut s = store("c");
    s.select_columns("all", 0, 2).unwrap();
    at_guard(12, |b| s.extract_f64("all", Layout::ColMajor, b, 4)).unwrap();
}
#[test]
fn extract_short_data() {
    let mut s = store("a");
    s.select_columns("all", 0, 2).unwrap();
    let r = at_guard(11, |b| s.extract_f64("all", Layout::ColMajor, b, 4));
    assert!(
        matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
        "{r:?}"
    );
}
#[test]
fn extract_rows_short_data() {
    let mut s = store("b");
    s.select_rows("mid", 1, 2).unwrap();
    let r = at_guard(5, |b| s.extract_f64("mid", Layout::RowMajor, b, 3));
    assert!(
        matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
        "{r:?}"
    );
}
#[test]
fn extract_oversized_lddata() {
    let mut s = store("d");
    s.select_columns("all", 0, 2).unwrap();
    let r = at_guard(12, |b| s.extract_f64("all", Layout::ColMajor, b, 8));
    assert!(
        matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
        "{r:?}"
    );
}
#[test]
fn extract_unknown_key() {
    let s = store("e");
    let r = at_guard(12, |b| {
        s.extract_f64("never_selected", Layout::ColMajor, b, 4)
    });
    assert!(
        matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
        "{r:?}"
    );
}
