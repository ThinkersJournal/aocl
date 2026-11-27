// Link-only. The library name (`openrng`, or `openrng_static` with `static-link`) and the 60 symbols are MEASURED to link
// against the Windows builds in docs/cleanroom/openrng/MEASUREMENT.md (both widths); nothing else about the library is
// assumed. Point OPENRNG_LIB_DIR at a directory holding the built library.
//
// WIDTH MUST MATCH: `openrng_int_t` is 4 bytes in an LP64 build and 8 in an ILP64 build, and the crate's `ilp64` feature must
// be on exactly when the library is the ILP64 build. A mismatch is SILENT at run time (measured: an LP64 library reads only the low
// half of a 64-bit `n` and accepts 0xFFFFFFFF_00000004 as 4). If OPENRNG_LIB_DIR's path names `ilp64` or `lp64`, this script
// cross-checks it against the feature and prints a build warning on a mismatch (a path heuristic, not a proof).
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
    let ilp64_feature = std::env::var_os("CARGO_FEATURE_ILP64").is_some();
    if let Some(dir) = std::env::var_os("OPENRNG_LIB_DIR") {
        let d = dir.to_string_lossy().to_ascii_lowercase();
        let says_ilp64 = d.contains("ilp64");
        let says_lp64 = !says_ilp64 && d.contains("lp64");
        if says_ilp64 && !ilp64_feature {
            println!("cargo:warning=aocl-openrng-sys: OPENRNG_LIB_DIR looks like an ILP64 build but the `ilp64` feature is OFF: openrng_int_t would be 32-bit against a 64-bit library (silent at run time)");
        }
        if says_lp64 && ilp64_feature {
            println!("cargo:warning=aocl-openrng-sys: OPENRNG_LIB_DIR looks like an LP64 build but the `ilp64` feature is ON: openrng_int_t would be 64-bit against a 32-bit library (silent at run time: the library reads only the low half of each integer argument)");
        }
    }
    println!("cargo:rustc-check-cfg=cfg(openrng_linked)");
    if std::env::var_os("OPENRNG_LIB_DIR").is_some() {
        // Tests are compiled only when a built library is available (they cannot link otherwise).
        println!("cargo:rustc-cfg=openrng_linked");
        println!(
            "cargo:rustc-link-lib={}=openrng{}",
            if static_link { "static" } else { "dylib" },
            if static_link { "_static" } else { "" }
        );
    }
}
