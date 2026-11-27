use aocl_build::{locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-changed=build.rs");

    // Link-only: the declarations are the clean-room set in src/decls.rs (see src/lib.rs); nothing is bindgen'd, no header is read.
    let root = locate_aocl_root();

    Component {
        component_dir: "amd-rng",
        wrapper: "",
        module: "",
        has_int_subdir: true,
        only_lp64: false,
        win_static: &["rng_amd-static"],
        win_dynamic: &["rng_amd"],
        unix_static: &["amdrng"],
        unix_dynamic: &["amdrng"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build(&root);
}
