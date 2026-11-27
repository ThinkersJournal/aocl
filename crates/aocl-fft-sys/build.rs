use aocl_build::{ensure_libclang_path, locate_aocl_root, Component};

fn main() {
    println!("cargo:rerun-if-env-changed=AOCL_ROOT");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    for f in ["LONG_DOUBLE", "THREADS_OMP", "MPI"] {
        println!("cargo:rerun-if-env-changed=CARGO_FEATURE_{f}");
    }
    println!("cargo:rerun-if-changed=wrapper");
    println!("cargo:rerun-if-changed=build.rs");

    ensure_libclang_path();
    let root = locate_aocl_root();

    // Optional precision/threading libraries (default OFF). Names are the installed AOCL 6.0.68 import-library names.
    let feat = |f: &str| {
        std::env::var_os(format!(
            "CARGO_FEATURE_{}",
            f.to_uppercase().replace('-', "_")
        ))
        .is_some()
    };
    let long_double = feat("long-double");
    let mut win: Vec<&'static str> = vec!["libfftw3", "libfftw3f"];
    let mut unix: Vec<&'static str> = vec!["fftw3", "fftw3f"];
    if long_double {
        win.push("libfftw3l");
        unix.push("fftw3l");
    }
    if feat("threads-omp") {
        win.extend(["libfftw3_omp", "libfftw3f_omp"]);
        unix.extend(["fftw3_omp", "fftw3f_omp"]);
        if long_double {
            win.push("libfftw3l_omp");
            unix.push("fftw3l_omp");
        }
    }
    if feat("mpi") {
        win.extend(["libfftw3_mpi", "libfftw3f_mpi"]);
        unix.extend(["fftw3_mpi", "fftw3f_mpi"]);
        if long_double {
            win.push("libfftw3l_mpi");
            unix.push("fftw3l_mpi");
        }
    }

    Component {
        component_dir: "amd-fftw",
        wrapper: "wrapper/fft.h",
        module: "bindings",
        has_int_subdir: false,
        only_lp64: false,
        // FFTW filenames vary by precision: libfftw3 is double, libfftw3f
        // is single, libfftw3l is long double. We link both single and
        // double; long-double is not yet exposed by the safe wrapper.
        win_static: &win,
        win_dynamic: &win,
        unix_static: &unix,
        unix_dynamic: &unix,
        extra_includes: &[],
        ilp64_defines: &[],
    }
    .build(&root);
}
