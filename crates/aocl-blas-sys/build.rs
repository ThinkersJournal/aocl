use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

/// Declared in the headers but present ONLY in the static library (AOCL 6.0.68 Windows, measured by link-all; Linux unmeasured):
/// `bli_*` (BLIS native API), `lpgemm_*`, `lpgemv_*`, `packa_*`/`packb_*`, `bla_*`, `AOCL_FAL_*`/`aocl_*` helpers. A dynamic build
/// cannot link them (they were a link error); they are declared only with the feature `static-link`.
const STATIC_ONLY: &str = include_str!("static_only.txt");
/// Declared in the headers but exported by NO installed library, dynamic or static (the ARM/A64FX cpuid probes, internal
/// `*_ref_var*`/`*_blk_var*` kernels, `*_zen4_asm_*` micro-kernels, `xerbla_array_blis_impl`, ...): left out unless the feature
/// `header-only-unexported`, which only declares them (they do not link on Windows).
const UNEXPORTED: &str = include_str!("unexported.txt");

fn feature(name: &str) -> bool {
    std::env::var_os(format!(
        "CARGO_FEATURE_{}",
        name.to_uppercase().replace('-', "_")
    ))
    .is_some()
}

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    for f in ["HEADER_ONLY_UNEXPORTED", "LIBFLAME", "ILP64", "STATIC_LINK"] {
        println!("cargo:rerun-if-env-changed=CARGO_FEATURE_{f}");
    }
    println!("cargo:rerun-if-changed=static_only.txt");
    println!("cargo:rerun-if-changed=unexported.txt");
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    let mut blocklist: Vec<&str> = Vec::new();
    if !feature("static-link") {
        blocklist.extend(STATIC_ONLY.lines().filter(|l| !l.is_empty()));
    }
    if !feature("header-only-unexported") {
        blocklist.extend(UNEXPORTED.lines().filter(|l| !l.is_empty()));
    }
    // `xerbla_array_` is exported by libflame, not by BLIS: declared only with the feature `libflame`, which links the libflame
    // DLL (Windows names measured; dynamic builds only: the static BLIS does not have it either).
    if !feature("libflame") {
        blocklist.push("xerbla_array_");
    } else if std::env::var("CARGO_CFG_TARGET_OS")
        .map(|o| o == "windows")
        .unwrap_or(false)
    {
        let int_dir = if feature("ilp64") { "ILP64" } else { "LP64" };
        println!(
            "cargo:rustc-link-search=native={}",
            root.join("amd-libflame")
                .join("lib")
                .join(int_dir)
                .display()
        );
        println!("cargo:rustc-link-lib=dylib=AOCL-LibFlame-Win-MT-dll");
    }

    Component {
        component_dir: "amd-blis",
        wrapper: "wrapper/blas.h",
        module: "bindings",
        has_int_subdir: true,
        only_lp64: false,
        win_static: &["AOCL-LibBlis-Win-MT"],
        win_dynamic: &["AOCL-LibBlis-Win-MT-dll"],
        unix_static: &["blis-mt"],
        unix_dynamic: &["blis-mt"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build_with_blocklist(&root, &blocklist);

    // For tests/link_all.rs: EVERY function the generated bindings declare, so the list follows the feature set.
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let out_dir = std::path::Path::new(&out_dir);
        if let Ok(text) = std::fs::read_to_string(out_dir.join("bindings.rs")) {
            let mut names: Vec<&str> = text
                .lines()
                .filter_map(|l| l.trim_start().strip_prefix("pub fn "))
                .filter_map(|l| l.split('(').next())
                .collect();
            names.sort_unstable();
            names.dedup();
            let mut list = String::from(
                "vec![
",
            );
            for n in &names {
                list.push_str(&format!(
                    "    aocl_blas_sys::{n} as usize,
"
                ));
            }
            list.push_str(
                "]
",
            );
            let _ = std::fs::write(out_dir.join("link_all_list.rs"), list);
        }
    }
}
