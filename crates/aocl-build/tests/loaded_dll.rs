//! Stale-DLL-on-PATH guard: for every AOCL component family, the DLLs the
//! build links must resolve (through the loader's own search, PATH included)
//! to a file under the install root. Needs a real AOCL install, as the other
//! real-AOCL tests do (PATH per HANDOFF/README).
#![cfg(windows)]

use aocl_build::loaded_dll::assert_loaded_under;
use aocl_build::locate_aocl_root;

fn check(names: &[&str]) {
    let root = locate_aocl_root();
    if let Err(e) = assert_loaded_under(&root, names) {
        panic!("{e}");
    }
}

#[test]
fn blas() {
    check(&["AOCL-LibBlis-Win-MT-dll.dll"]);
}
#[test]
fn lapack() {
    check(&["AOCL-LibFlame-Win-MT-dll.dll"]);
}
#[test]
fn compression() {
    check(&["aocl_compression.dll"]);
}
#[test]
fn crypto() {
    check(&["alcp.dll"]);
}
#[test]
fn data_analytics() {
    check(&["aocl-da.dll"]);
}
#[test]
fn fft() {
    check(&["libfftw3.dll", "libfftw3f.dll"]);
}
#[test]
fn math() {
    check(&["libalm.dll"]);
}
#[test]
fn rng() {
    check(&["rng_amd.dll"]);
}
#[test]
fn scalapack() {
    check(&["scalapack.dll"]);
}
#[test]
fn securerng() {
    check(&["amdsecrng.dll"]);
}
#[test]
fn sparse() {
    check(&["aoclsparse.dll"]);
}
#[test]
fn utils() {
    check(&["libaoclutils.dll", "au_cpuid.dll"]);
}

/// Control: a copy of a real DLL placed in a directory outside the install
/// root and first on PATH must be rejected, by name.
#[test]
fn control_wrong_dir_first_on_path_fails_by_name() {
    let root = locate_aocl_root();
    let src = root.join("amd-libm").join("lib").join("libalm.dll");
    assert!(src.exists(), "control source missing: {}", src.display());
    let dir = std::env::temp_dir().join(format!("aoclguard_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(&src, dir.join("aoclguard_ctl.dll")).unwrap();
    let old = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![dir.clone()];
    paths.extend(std::env::split_paths(&old));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    let r = assert_loaded_under(&root, &["aoclguard_ctl.dll"]);
    let err = r.expect_err("a DLL outside the install root must be rejected");
    assert!(
        err.contains("aoclguard_ctl.dll") && err.contains("not under"),
        "{err}"
    );
}
