use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    Component {
        component_dir: "amd-rng",
        wrapper: "wrapper/rng.h",
        module: "bindings",
        has_int_subdir: true,
        only_lp64: false,
        win_static: &["rng_amd-static"],
        win_dynamic: &["rng_amd"],
        unix_static: &["amdrng"],
        unix_dynamic: &["amdrng"],
        extra_includes: &[],
        ilp64_defines: if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
            &["INTEGER64", "WIN64"]
        } else {
            &["INTEGER64"]
        },
    }
    .build(&root);
}
