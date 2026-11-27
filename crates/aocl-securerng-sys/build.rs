use aocl_build::{locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-changed=build.rs");

    // Link-only: the declarations are the clean-room set in src/decls.rs; nothing is bindgen'd, no header is read.
    let root = locate_aocl_root();

    Component {
        component_dir: "amd-securerng",
        wrapper: "",
        module: "",
        has_int_subdir: true,
        only_lp64: true,
        win_static: &["amdsecrng-static"],
        win_dynamic: &["amdsecrng"],
        unix_static: &["secrng"],
        unix_dynamic: &["secrng"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build(&root);
}
