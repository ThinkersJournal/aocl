use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_HEADER_ONLY_UNEXPORTED");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_STATIC_LINK");
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    // Measured by link-all on AOCL 6.0.68 Windows (Linux unmeasured): 10 LZMA-SDK stream helpers are declared in 7zTypes.h but
    // exported by no installed library (not statically either): left out unless `header-only-unexported`. `BZ2_bzCompressBound`
    // and `Lzma_compressBound` exist only in the static library: left out unless `static-link` (a dynamic build cannot link them).
    let header_only = std::env::var_os("CARGO_FEATURE_HEADER_ONLY_UNEXPORTED").is_some();
    let static_link = std::env::var_os("CARGO_FEATURE_STATIC_LINK").is_some();
    let mut blocklist: Vec<&str> = Vec::new();
    if !header_only {
        blocklist.extend([
            "LookInStream_LookRead",
            "LookInStream_Read",
            "LookInStream_Read2",
            "LookInStream_SeekTo",
            "LookToRead2_CreateVTable",
            "SecToLook_CreateVTable",
            "SecToRead_CreateVTable",
            "SeqInStream_Read",
            "SeqInStream_Read2",
            "SeqInStream_ReadByte",
        ]);
    }
    if !static_link {
        blocklist.extend(["BZ2_bzCompressBound", "Lzma_compressBound"]);
    }

    Component {
        component_dir: "amd-compression",
        wrapper: "wrapper/compression.h",
        module: "bindings",
        has_int_subdir: false,
        only_lp64: false,
        win_static: &["aocl_compression"],
        win_dynamic: &["aocl_compression"],
        unix_static: &["aocl_compression"],
        unix_dynamic: &["aocl_compression"],
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build_with_blocklist(&root, &blocklist);
}
