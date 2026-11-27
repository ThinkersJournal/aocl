use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

/// Declared in the LAPACK/LAPACKE headers but exported by NO installed AOCL 6.0.68 Windows library (measured by link-all,
/// dynamic and static, LP64 and ILP64; Linux unmeasured): left out of the bindings unless `header-only-unexported`.
/// 100 `LAPACKE_*` extended-precision routines (`*x`/`*xx` and their `_work`) and 50 Fortran `*x_`/`clagge_`-type routines.
const UNEXPORTED: &[&str] = &[
    "LAPACKE_cgbrfsx",
    "LAPACKE_cgbrfsx_work",
    "LAPACKE_cgbsvxx",
    "LAPACKE_cgbsvxx_work",
    "LAPACKE_cgerfsx",
    "LAPACKE_cgerfsx_work",
    "LAPACKE_cgesvxx",
    "LAPACKE_cgesvxx_work",
    "LAPACKE_cherfsx",
    "LAPACKE_cherfsx_work",
    "LAPACKE_chesvxx",
    "LAPACKE_chesvxx_work",
    "LAPACKE_clagge",
    "LAPACKE_clagge_work",
    "LAPACKE_claghe",
    "LAPACKE_claghe_work",
    "LAPACKE_clagsy",
    "LAPACKE_clagsy_work",
    "LAPACKE_clatms",
    "LAPACKE_clatms_work",
    "LAPACKE_cporfsx",
    "LAPACKE_cporfsx_work",
    "LAPACKE_cposvxx",
    "LAPACKE_cposvxx_work",
    "LAPACKE_csyrfsx",
    "LAPACKE_csyrfsx_work",
    "LAPACKE_csysvxx",
    "LAPACKE_csysvxx_work",
    "LAPACKE_dgbrfsx",
    "LAPACKE_dgbrfsx_work",
    "LAPACKE_dgbsvxx",
    "LAPACKE_dgbsvxx_work",
    "LAPACKE_dgerfsx",
    "LAPACKE_dgerfsx_work",
    "LAPACKE_dgesvxx",
    "LAPACKE_dgesvxx_work",
    "LAPACKE_dlagge",
    "LAPACKE_dlagge_work",
    "LAPACKE_dlagsy",
    "LAPACKE_dlagsy_work",
    "LAPACKE_dlatms",
    "LAPACKE_dlatms_work",
    "LAPACKE_dporfsx",
    "LAPACKE_dporfsx_work",
    "LAPACKE_dposvxx",
    "LAPACKE_dposvxx_work",
    "LAPACKE_dsyrfsx",
    "LAPACKE_dsyrfsx_work",
    "LAPACKE_dsysvxx",
    "LAPACKE_dsysvxx_work",
    "LAPACKE_sgbrfsx",
    "LAPACKE_sgbrfsx_work",
    "LAPACKE_sgbsvxx",
    "LAPACKE_sgbsvxx_work",
    "LAPACKE_sgerfsx",
    "LAPACKE_sgerfsx_work",
    "LAPACKE_sgesvxx",
    "LAPACKE_sgesvxx_work",
    "LAPACKE_slagge",
    "LAPACKE_slagge_work",
    "LAPACKE_slagsy",
    "LAPACKE_slagsy_work",
    "LAPACKE_slatms",
    "LAPACKE_slatms_work",
    "LAPACKE_sporfsx",
    "LAPACKE_sporfsx_work",
    "LAPACKE_sposvxx",
    "LAPACKE_sposvxx_work",
    "LAPACKE_ssyrfsx",
    "LAPACKE_ssyrfsx_work",
    "LAPACKE_ssysvxx",
    "LAPACKE_ssysvxx_work",
    "LAPACKE_zgbrfsx",
    "LAPACKE_zgbrfsx_work",
    "LAPACKE_zgbsvxx",
    "LAPACKE_zgbsvxx_work",
    "LAPACKE_zgerfsx",
    "LAPACKE_zgerfsx_work",
    "LAPACKE_zgesvxx",
    "LAPACKE_zgesvxx_work",
    "LAPACKE_zherfsx",
    "LAPACKE_zherfsx_work",
    "LAPACKE_zhesvxx",
    "LAPACKE_zhesvxx_work",
    "LAPACKE_zlagge",
    "LAPACKE_zlagge_work",
    "LAPACKE_zlaghe",
    "LAPACKE_zlaghe_work",
    "LAPACKE_zlagsy",
    "LAPACKE_zlagsy_work",
    "LAPACKE_zlatms",
    "LAPACKE_zlatms_work",
    "LAPACKE_zporfsx",
    "LAPACKE_zporfsx_work",
    "LAPACKE_zposvxx",
    "LAPACKE_zposvxx_work",
    "LAPACKE_zsyrfsx",
    "LAPACKE_zsyrfsx_work",
    "LAPACKE_zsysvxx",
    "LAPACKE_zsysvxx_work",
    "cgbrfsx_",
    "cgbsvxx_",
    "cgerfsx_",
    "cgesvxx_",
    "cherfsx_",
    "chesvxx_",
    "clagge_",
    "claghe_",
    "clagsy_",
    "clatms_",
    "cporfsx_",
    "cposvxx_",
    "csyrfsx_",
    "csysvxx_",
    "dgbrfsx_",
    "dgbsvxx_",
    "dgerfsx_",
    "dgesvxx_",
    "dlagge_",
    "dlagsy_",
    "dlatms_",
    "dporfsx_",
    "dposvxx_",
    "dsyrfsx_",
    "dsysvxx_",
    "sgbrfsx_",
    "sgbsvxx_",
    "sgerfsx_",
    "sgesvxx_",
    "slagge_",
    "slagsy_",
    "slatms_",
    "sporfsx_",
    "sposvxx_",
    "ssyrfsx_",
    "ssysvxx_",
    "zgbrfsx_",
    "zgbsvxx_",
    "zgerfsx_",
    "zgesvxx_",
    "zherfsx_",
    "zhesvxx_",
    "zlagge_",
    "zlaghe_",
    "zlagsy_",
    "zlatms_",
    "zporfsx_",
    "zposvxx_",
    "zsyrfsx_",
    "zsysvxx_",
];

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
    for f in ["HEADER_ONLY_UNEXPORTED", "BLIS", "ILP64", "STATIC_LINK"] {
        println!("cargo:rerun-if-env-changed=CARGO_FEATURE_{f}");
    }
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    // `lsame_` is exported by the BLIS library, not by libflame: declared only with the feature `blis`, which links BLIS.
    let mut blocklist: Vec<&str> = Vec::new();
    if !feature("header-only-unexported") {
        blocklist.extend(UNEXPORTED.iter().copied());
    }
    if !feature("blis") {
        blocklist.push("lsame_");
    } else if cfg!(windows)
        || std::env::var("CARGO_CFG_TARGET_OS")
            .map(|o| o == "windows")
            .unwrap_or(false)
    {
        // Windows names measured on AOCL 6.0.68 (Linux unmeasured): link the BLIS library next to libflame.
        let int_dir = if feature("ilp64") { "ILP64" } else { "LP64" };
        let blis_dir = root.join("amd-blis").join("lib").join(int_dir);
        let blis_dir = if feature("static-link") && blis_dir.join("static").exists() {
            blis_dir.join("static")
        } else {
            blis_dir
        };
        println!("cargo:rustc-link-search=native={}", blis_dir.display());
        if feature("static-link") {
            println!("cargo:rustc-link-lib=static=AOCL-LibBlis-Win-MT");
        } else {
            println!("cargo:rustc-link-lib=dylib=AOCL-LibBlis-Win-MT-dll");
        }
    }

    Component {
        component_dir: "amd-libflame",
        wrapper: "wrapper/lapack.h",
        module: "bindings",
        has_int_subdir: true,
        only_lp64: false,
        win_static: &["AOCL-LibFlame-Win-MT"],
        win_dynamic: &["AOCL-LibFlame-Win-MT-dll"],
        unix_static: &["flame"],
        unix_dynamic: &["flame"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build_with_blocklist(&root, &blocklist);

    // For tests/link_all.rs: the list of EVERY function the generated bindings declare (so the test follows the feature
    // set: the left-out names are not in it), as a `vec![ aocl_lapack_sys::name as usize, ... ]` expression in OUT_DIR.
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let bindings = std::path::Path::new(&out_dir).join("bindings.rs");
        if let Ok(text) = std::fs::read_to_string(&bindings) {
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
                    "    aocl_lapack_sys::{n} as usize,
"
                ));
            }
            list.push_str(
                "]
",
            );
            let _ = std::fs::write(
                std::path::Path::new(&out_dir).join("link_all_list.rs"),
                list,
            );
        }
    }
}
