//! Thread-pinning helpers from AOCL-Utils.
//!
//! These are very thin wrappers around the AOCL routines that take a
//! list of OS-level thread handles (`pthread_t` on Unix, `HANDLE` on
//! Windows) and pin them to physical or logical cores. Most callers
//! collect `pthread_t` / `HANDLE` values from their thread library
//! (`std::thread::current().handle()`-style accessors via `os`-specific
//! crates) before invoking these.
//!
//! The functions are `unsafe` because passing a thread handle that no
//! longer refers to a live thread is undefined behavior on the AOCL
//! side.

use aocl_utils_sys as sys;

/// OS-native thread handle.
pub type ThreadHandle = sys::pthread_t;

/// Pinning strategy for a list of thread handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PinStrategy {
    /// Pin each thread to a unique physical core (one thread per core,
    /// preferring physical core 0, 1, 2, …).
    Core,
    /// Pin each thread to a unique logical processor (SMT thread).
    Logical,
    /// Spread threads across cores to maximize per-thread cache, mapping
    /// each thread to a different L3 / NUMA region when possible.
    Spread,
}

/// Pin the given thread handles using the chosen strategy.
///
/// # Safety
///
/// Each entry of `threads` must be a valid OS-level handle to a still-live
/// thread. Passing a stale handle (e.g. for a thread that has exited or
/// been detached) is undefined behavior. The slice's length must fit in
/// a C `size_t`.
pub unsafe fn pin_threads(strategy: PinStrategy, threads: &mut [ThreadHandle]) {
    let n = threads.len();
    let p = threads.as_mut_ptr();
    match strategy {
        PinStrategy::Core => unsafe { sys::au_pin_threads_core(p, n) },
        PinStrategy::Logical => unsafe { sys::au_pin_threads_logical(p, n) },
        PinStrategy::Spread => unsafe { sys::au_pin_threads_spread(p, n) },
    }
}

/// Pin threads with a custom mapping: `affinity_vector[i]` is the logical
/// processor index thread `i` should bind to.
///
/// # Safety
///
/// Same as [`pin_threads`]; in addition, the affinity values must
/// reference logical processors that exist on the host.
pub unsafe fn pin_threads_custom(threads: &mut [ThreadHandle], affinity_vector: &[i32]) {
    let n = threads.len();
    if n == 0 || affinity_vector.is_empty() {
        return;
    }
    unsafe {
        sys::au_pin_threads_custom(
            threads.as_mut_ptr(),
            n,
            affinity_vector.as_ptr() as *mut std::os::raw::c_int,
            affinity_vector.len(),
        );
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use std::sync::mpsc;

    #[repr(C)]
    #[derive(Default)]
    struct GroupAffinity {
        mask: usize,
        group: u16,
        reserved: [u16; 3],
    }
    extern "system" {
        fn GetThreadGroupAffinity(
            thread: *mut core::ffi::c_void,
            affinity: *mut GroupAffinity,
        ) -> i32;
    }

    /// Spawn a live thread that waits, run `f` on its handle, and read back the OS affinity mask of that thread.
    fn pinned_mask(f: impl FnOnce(&mut [ThreadHandle])) -> usize {
        let (tx, rx) = mpsc::channel::<()>();
        let th = std::thread::spawn(move || {
            let _ = rx.recv();
        });
        let mut handles = [th.as_raw_handle() as ThreadHandle];
        f(&mut handles);
        let mut ga = GroupAffinity::default();
        let ok = unsafe { GetThreadGroupAffinity(handles[0] as *mut _, &mut ga) };
        assert_ne!(ok, 0, "GetThreadGroupAffinity failed");
        tx.send(()).unwrap();
        th.join().unwrap();
        ga.mask
    }

    #[test]
    fn pin_threads_custom_binds_the_thread_to_the_requested_logical_processor() {
        // Logical processor 0 -> exactly bit 0 of the affinity mask. Control: an unpinned thread is not bound to one core
        // (the default mask has more than one bit set on a multi-core host).
        let unpinned = pinned_mask(|_| {});
        let pinned = pinned_mask(|h| unsafe { pin_threads_custom(h, &[0]) });
        assert_eq!(pinned, 1, "affinity mask after pin_threads_custom(&[0])");
        if std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            > 1
        {
            assert!(
                unpinned.count_ones() > 1,
                "control: unpinned mask {unpinned:#x}"
            );
        }
    }

    #[test]
    fn pin_threads_core_and_logical_and_spread_leave_a_single_processor_mask() {
        // Core / Logical / Spread each pin one live thread to a processor of this host: the mask is non-empty, a subset of
        // the host's processors, and differs from the unpinned mask (control).
        let unpinned = pinned_mask(|_| {});
        for strategy in [PinStrategy::Core, PinStrategy::Logical, PinStrategy::Spread] {
            let m = pinned_mask(|h| unsafe { pin_threads(strategy, h) });
            assert_ne!(m, 0, "{strategy:?}");
            assert!(
                m.count_ones() >= 1 && m.count_ones() <= unpinned.count_ones(),
                "{strategy:?}: {m:#x} vs {unpinned:#x}"
            );
            if std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                > 1
            {
                assert_ne!(m, unpinned, "{strategy:?} left the thread unpinned");
            }
        }
    }
}
