use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    Component {
        component_dir: "amd-sparse",
        wrapper: "wrapper/sparse.h",
        module: "bindings",
        has_int_subdir: true,
        only_lp64: false,
        win_static: &["aoclsparse"],
        win_dynamic: &["aoclsparse"],
        unix_static: &["aoclsparse"],
        unix_dynamic: &["aoclsparse"],
        extra_includes: &[],
        ilp64_defines: &["aoclsparse_ILP64"],
    }
    .build(&root);

    fix_complex_alignment();
}

/// aoclsparse_types.h declares the complex types `alignas(2 * sizeof(T))` (16 for double, 8 for float) but defines `alignas`
/// away in pure C (`#define alignas(s)`), and bindgen parses the header as C: the generated structs are 8- and 4-aligned.
/// The library is compiled as C++ with the alignment, and the complex types are passed BY VALUE (alpha, beta, omega, val):
/// `aoclsparse_zset_value` faulted (access violation) with the unaligned binding. Re-apply the header's alignment to the
/// generated file; fail the build if the expected struct text is not there (a header change must be looked at).
fn fix_complex_alignment() {
    let out =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("bindings.rs");
    let Ok(text) = std::fs::read_to_string(&out) else {
        return;
    }; // docs.rs short-circuit writes a stub
    if !text.contains("pub struct aoclsparse_double_complex_") {
        return;
    }
    let mut fixed = text.clone();
    for (name, align) in [
        ("aoclsparse_double_complex_", 16),
        ("aoclsparse_float_complex", 8),
    ] {
        let tail = format!("#[derive(Debug, Default, Copy, Clone)]{{nl}}pub struct {name} {{");
        let mut done = false;
        for nl in ["\n", "\r\n"] {
            let old = format!("#[repr(C)]{nl}{}", tail.replace("{nl}", nl));
            let new = format!("#[repr(C, align({align}))]{nl}{}", tail.replace("{nl}", nl));
            if fixed.contains(&old) {
                fixed = fixed.replacen(&old, &new, 1);
                done = true;
                break;
            }
        }
        if !done && !fixed.contains(&format!("#[repr(C, align({align}))]")) {
            panic!("aocl-sparse-sys: cannot find the generated `{name}` struct to give it align({align}); the header or bindgen output changed");
        }
    }
    std::fs::write(&out, fixed).expect("write bindings.rs");
}
