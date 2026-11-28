// The tests in `tests/` call the library, so they compile only when a built library is available (the `-sys` crate links
// it when OPENRNG_LIB_DIR is set). This script sets the same cfg for this crate's tests.
fn main() {
    println!("cargo:rerun-if-env-changed=OPENRNG_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(openrng_linked)");
    if std::env::var_os("OPENRNG_LIB_DIR").is_some() {
        println!("cargo:rustc-cfg=openrng_linked");
    }
}
