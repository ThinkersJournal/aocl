//! Loader-path guard (Windows only): resolve a DLL the way the loader does
//! and report which file it actually picked.
//!
//! A stale copy of an AOCL or Intel runtime DLL earlier on `PATH` is loaded
//! silently instead of the one under the install directory. This module
//! resolves each name with `LoadLibraryW` (same search order as the
//! import-table loader, `PATH` included), reads the module's full path with
//! `GetModuleFileNameW`, and checks it lies under an expected directory.

use std::ffi::{c_void, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetModuleFileNameW(module: *mut c_void, buf: *mut u16, size: u32) -> u32;
}

/// Load `dll_name` (e.g. `"libalm.dll"`) and return the full path the loader
/// resolved. The module stays loaded for the life of the process.
pub fn resolve_loaded_dll(dll_name: &str) -> Result<PathBuf, String> {
    let wide: Vec<u16> = std::ffi::OsStr::new(dll_name)
        .encode_wide()
        .chain(Some(0))
        .collect();
    // SAFETY: `wide` is NUL-terminated; the handle is only passed back to
    // GetModuleFileNameW with a buffer of the stated length.
    unsafe {
        let h = LoadLibraryW(wide.as_ptr());
        if h.is_null() {
            return Err(format!("{dll_name}: not found by the loader (PATH?)"));
        }
        let mut buf = vec![0u16; 4096];
        let n = GetModuleFileNameW(h, buf.as_mut_ptr(), buf.len() as u32) as usize;
        if n == 0 || n >= buf.len() {
            return Err(format!("{dll_name}: GetModuleFileNameW failed"));
        }
        Ok(PathBuf::from(OsString::from_wide(&buf[..n])))
    }
}

fn norm(p: &Path) -> String {
    let s = p.to_string_lossy().replace('/', "\\").to_lowercase();
    s.strip_prefix("\\\\?\\")
        .unwrap_or(&s)
        .trim_end_matches('\\')
        .to_string()
}

/// Resolve every name and fail, naming the DLL and the path it came from,
/// unless the file lies under `expected_root`. Prints each resolved path.
pub fn assert_loaded_under(expected_root: &Path, dll_names: &[&str]) -> Result<(), String> {
    let root = norm(
        &expected_root
            .canonicalize()
            .unwrap_or(expected_root.to_path_buf()),
    );
    for name in dll_names {
        let got = resolve_loaded_dll(name)?;
        println!("loaded {name} -> {}", got.display());
        let g = norm(&got.canonicalize().unwrap_or(got.clone()));
        if !g.starts_with(&format!("{root}\\")) {
            return Err(format!(
                "{name} resolved to {} which is not under {}",
                got.display(),
                expected_root.display()
            ));
        }
    }
    Ok(())
}
