use aocl_build::{locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-changed=build.rs");

    // Link-only: the declarations are the clean-room set in src/decls_*.rs (see src/lib.rs); nothing is bindgen'd, no header is read.
    let root = locate_aocl_root();

    Component {
        component_dir: "amd-libm",
        wrapper: "",
        module: "",
        has_int_subdir: false,
        only_lp64: false,
        win_static: &["libalm-static"],
        win_dynamic: &["libalm"],
        unix_static: &["alm"],
        unix_dynamic: &["alm"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build(&root);
}
