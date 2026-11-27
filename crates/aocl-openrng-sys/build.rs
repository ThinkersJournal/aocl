// Link-only. UNMEASURED: no OpenRNG library is installed on the lane host (AOCL 6.0.68 for Windows has no amd-openrng
// directory) and none has been built yet, so the library name and location below are the README's names (`openrng`,
// `openrng_static`), not measured. Point OPENRNG_LIB_DIR at a directory holding the built library.
fn main() {
    println!("cargo:rerun-if-env-changed=OPENRNG_LIB_DIR");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("DOCS_RS").is_some() {
        return;
    }
    if let Some(dir) = std::env::var_os("OPENRNG_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
    }
    let static_link = std::env::var_os("CARGO_FEATURE_STATIC_LINK").is_some();
    println!("cargo:rustc-check-cfg=cfg(openrng_linked)");
    if std::env::var_os("OPENRNG_LIB_DIR").is_some() {
        // Tests are compiled only when a built library is available (they cannot link otherwise).
        println!("cargo:rustc-cfg=openrng_linked");
        println!("cargo:rustc-link-lib={}=openrng{}", if static_link { "static" } else { "dylib" }, if static_link { "_static" } else { "" });
    }
}
