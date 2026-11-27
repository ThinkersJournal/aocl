use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_HEADER_ONLY_UNEXPORTED");
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    // Declared in the headers (`au_get_version_*` carry AUD_API_EXPORT) but exported by NO installed library on AOCL 6.0.68
    // Windows (measured by link-all; Linux unmeasured): left out unless the feature `header-only-unexported` is enabled.
    let header_only = std::env::var_os("CARGO_FEATURE_HEADER_ONLY_UNEXPORTED").is_some();
    let blocklist: &[&str] = if header_only {
        &[]
    } else {
        &[
            "au_get_version_major",
            "au_get_version_minor",
            "au_get_version_patch",
            "au_get_version_str",
            "au_is_error",
        ]
    };

    Component {
        component_dir: "amd-utils",
        wrapper: "wrapper/utils.h",
        module: "bindings",
        has_int_subdir: false,
        only_lp64: false,
        win_static: &["libaoclutils_static", "au_cpuid_static"],
        win_dynamic: &["libaoclutils", "au_cpuid"],
        unix_static: &["aoclutils", "au_cpuid"],
        unix_dynamic: &["aoclutils", "au_cpuid"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build_with_blocklist(&root, blocklist);
}
