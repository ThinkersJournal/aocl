use aocl_build::{locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_SECURERNG");
    println!("cargo:rerun-if-changed=build.rs");

    // Link-only: the declarations are the clean-room set in src/decls.rs (see src/lib.rs); nothing is bindgen'd, no header is read.
    let root = locate_aocl_root();

    // `securerng`: also link amdsecrng (same lib dir, per-width). Windows names only: the Linux library name was not measured.
    let secure = std::env::var_os("CARGO_FEATURE_SECURERNG").is_some();
    let win_static: &[&str] = if secure { &["rng_amd-static", "amdsecrng-static"] } else { &["rng_amd-static"] };
    let win_dynamic: &[&str] = if secure { &["rng_amd", "amdsecrng"] } else { &["rng_amd"] };

    Component {
        component_dir: "amd-rng",
        wrapper: "",
        module: "",
        has_int_subdir: true,
        only_lp64: false,
        win_static,
        win_dynamic,
        unix_static: &["amdrng"],
        unix_dynamic: &["amdrng"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build(&root);
}
