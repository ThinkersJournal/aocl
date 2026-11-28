//! Safe wrappers for AOCL-FFTW.
//!
//! Provides:
//! - One-shot 1-D / 2-D / 3-D complex DFTs (in-place) in both `f64`
//!   (default names) and `f32` (suffixed `_f32`).
//! - One-shot 1-D real-to-complex (`r2c`) and complex-to-real (`c2r`)
//!   transforms in both precisions.
//! - Reusable plan types: [`Plan`] (`f64`) and [`PlanF32`] (`f32`) that
//!   cache plan creation and support FFTW's "new-array execute"
//!   routines for repeated transforms over the same shape.
//!
//! Plan creation/destruction is serialized through a process-wide
//! mutex because FFTW's planner is not internally thread-safe in the
//! single-threaded build we link against. Plan **execution** is
//! thread-safe and not held under that lock. The same lock guards
//! both precisions since they share the planner state.
//!
//! # Licence condition of the library you link
//!
//! The AMD `libfftw3`/`libfftw3f` in AOCL is GPL-licensed (AOCL's `NOTICE.txt` says GPL-2.0; the shipped `COPYRIGHT` says
//! version 2 or any later version: the two disagree). This crate ships no library. A program that links it and is
//! distributed must comply with the library's licence. Whether a source-only crate of `extern` declarations is itself a
//! combined work is UNKNOWN (no FSF text found). This is not legal advice.

#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use aocl_error::{Error, Result};
use aocl_fft_sys as sys;
use std::sync::Mutex;

/// FFTW's plan-creation and plan-destruction routines mutate global state.
static PLANNER_LOCK: Mutex<()> = Mutex::new(());

/// Direction of a complex DFT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Forward transform (FFTW_FORWARD = -1).
    Forward,
    /// Backward / inverse transform (FFTW_BACKWARD = +1). FFTW does
    /// **not** divide by `n` — call sites that want a strict inverse
    /// must scale the output themselves.
    Backward,
}

impl Direction {
    fn raw(self) -> std::os::raw::c_int {
        match self {
            Direction::Forward => sys::FFTW_FORWARD,
            Direction::Backward => sys::FFTW_BACKWARD as i32,
        }
    }
}

fn check_n(name: &str, n: usize) -> Result<i32> {
    if n > i32::MAX as usize {
        return Err(Error::InvalidArgument(format!(
            "{name}: dimension {n} exceeds i32::MAX"
        )));
    }
    Ok(n as i32)
}

// =========================================================================
//   One-shot complex DFTs (1-D / 2-D / 3-D)
// =========================================================================

/// Compute a 1-D complex DFT in place. `data` is treated as `n = data.len()`
/// complex samples in `[real, imag]` order.
pub fn dft_1d_inplace(direction: Direction, data: &mut [[f64; 2]]) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    let n = check_n("dft_1d_inplace", data.len())?;
    let ptr = data.as_mut_ptr() as *mut sys::fftw_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftw_plan_dft_1d(n, ptr, ptr, direction.raw(), sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftw_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftw_destroy_plan(plan) };
    Ok(())
}

/// Compute a 2-D complex DFT in place. `data` is `n0 × n1` complex samples
/// in row-major order (length `n0 · n1`).
pub fn dft_2d_inplace(
    direction: Direction,
    n0: usize,
    n1: usize,
    data: &mut [[f64; 2]],
) -> Result<()> {
    if n0 == 0 || n1 == 0 {
        return Ok(());
    }
    let need = n0 * n1;
    if data.len() < need {
        return Err(Error::InvalidArgument(format!(
            "dft_2d_inplace: data length {} < n0·n1 = {need}",
            data.len()
        )));
    }
    let n0 = check_n("dft_2d_inplace: n0", n0)?;
    let n1 = check_n("dft_2d_inplace: n1", n1)?;
    let ptr = data.as_mut_ptr() as *mut sys::fftw_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftw_plan_dft_2d(n0, n1, ptr, ptr, direction.raw(), sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftw_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftw_destroy_plan(plan) };
    Ok(())
}

/// Compute a 3-D complex DFT in place.
pub fn dft_3d_inplace(
    direction: Direction,
    n0: usize,
    n1: usize,
    n2: usize,
    data: &mut [[f64; 2]],
) -> Result<()> {
    if n0 == 0 || n1 == 0 || n2 == 0 {
        return Ok(());
    }
    let need = n0 * n1 * n2;
    if data.len() < need {
        return Err(Error::InvalidArgument(format!(
            "dft_3d_inplace: data length {} < n0·n1·n2 = {need}",
            data.len()
        )));
    }
    let n0 = check_n("dft_3d_inplace: n0", n0)?;
    let n1 = check_n("dft_3d_inplace: n1", n1)?;
    let n2 = check_n("dft_3d_inplace: n2", n2)?;
    let ptr = data.as_mut_ptr() as *mut sys::fftw_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftw_plan_dft_3d(n0, n1, n2, ptr, ptr, direction.raw(), sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftw_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftw_destroy_plan(plan) };
    Ok(())
}

/// Convenience: forward 1-D DFT in place.
pub fn forward_inplace(data: &mut [[f64; 2]]) -> Result<()> {
    dft_1d_inplace(Direction::Forward, data)
}

/// Convenience: backward (unscaled) 1-D DFT in place.
pub fn backward_inplace(data: &mut [[f64; 2]]) -> Result<()> {
    dft_1d_inplace(Direction::Backward, data)
}

// =========================================================================
//   Real ↔ complex one-shot (1-D)
// =========================================================================

/// Compute a 1-D real-to-complex (forward) DFT.
///
/// `out` must hold at least `n/2 + 1` complex samples (the second half of
/// the spectrum is implied by Hermitian symmetry).
pub fn r2c_1d(input: &mut [f64], output: &mut [[f64; 2]]) -> Result<()> {
    let n = input.len();
    if n == 0 {
        return Ok(());
    }
    let need_out = n / 2 + 1;
    if output.len() < need_out {
        return Err(Error::InvalidArgument(format!(
            "r2c_1d: output length {} < n/2+1 = {need_out}",
            output.len()
        )));
    }
    let n_i = check_n("r2c_1d", n)?;
    let in_p = input.as_mut_ptr();
    let out_p = output.as_mut_ptr() as *mut sys::fftw_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftw_plan_dft_r2c_1d(n_i, in_p, out_p, sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftw_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftw_destroy_plan(plan) };
    Ok(())
}

/// Compute a 1-D complex-to-real (backward) DFT.
///
/// `n` is the size of the *output* (the real time-domain signal). `input`
/// must hold at least `n/2 + 1` complex samples. Note FFTW does **not**
/// normalize — divide by `n` to recover the original time-domain signal
/// after a forward+backward round-trip.
pub fn c2r_1d(n: usize, input: &mut [[f64; 2]], output: &mut [f64]) -> Result<()> {
    if n == 0 {
        return Ok(());
    }
    let need_in = n / 2 + 1;
    if input.len() < need_in {
        return Err(Error::InvalidArgument(format!(
            "c2r_1d: input length {} < n/2+1 = {need_in}",
            input.len()
        )));
    }
    if output.len() < n {
        return Err(Error::InvalidArgument(format!(
            "c2r_1d: output length {} < n = {n}",
            output.len()
        )));
    }
    let n_i = check_n("c2r_1d", n)?;
    let in_p = input.as_mut_ptr() as *mut sys::fftw_complex;
    let out_p = output.as_mut_ptr();
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftw_plan_dft_c2r_1d(n_i, in_p, out_p, sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftw_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftw_destroy_plan(plan) };
    Ok(())
}

// =========================================================================
//   Reusable Plan
// =========================================================================

/// Kind of transform a [`Plan`] performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlanKind {
    Complex,
    R2C,
    C2R,
}

/// A reusable FFTW plan over a fixed shape and transform kind.
///
/// Construct once with [`Plan::dft_1d`] / [`Plan::dft_2d`] / [`Plan::dft_3d`]
/// / [`Plan::r2c_1d`] / [`Plan::c2r_1d`], then call
/// [`Plan::execute_dft`] / [`Plan::execute_r2c`] / [`Plan::execute_c2r`]
/// repeatedly with new buffers of the matching shape. The buffers used at
/// execute time must have the same alignment as the buffers used at plan
/// creation; plain `Vec<…>` and `[…; N]` storage on x86_64 satisfy this.
pub struct Plan {
    plan: sys::fftw_plan,
    kind: PlanKind,
    /// Total number of complex elements (for dft) or real elements (for r2c/c2r).
    n_total: usize,
}

unsafe impl Send for Plan {}

impl Plan {
    /// Build a 1-D complex DFT plan over `n` samples in the given direction.
    pub fn dft_1d(n: usize, direction: Direction) -> Result<Self> {
        if n == 0 {
            return Err(Error::InvalidArgument("dft_1d: n must be positive".into()));
        }
        let n_i = check_n("dft_1d", n)?;
        // Create a scratch buffer for plan-time pointer requirements;
        // FFTW_ESTIMATE does not actually run the transform, so the
        // contents are irrelevant.
        let mut scratch = vec![[0.0_f64, 0.0_f64]; n];
        let p = scratch.as_mut_ptr() as *mut sys::fftw_complex;
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { sys::fftw_plan_dft_1d(n_i, p, p, direction.raw(), sys::FFTW_ESTIMATE) }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::Complex,
            n_total: n,
        })
    }

    /// Build a 2-D complex DFT plan over `n0 × n1` samples.
    pub fn dft_2d(n0: usize, n1: usize, direction: Direction) -> Result<Self> {
        if n0 == 0 || n1 == 0 {
            return Err(Error::InvalidArgument(
                "dft_2d: dimensions must be positive".into(),
            ));
        }
        let n0_i = check_n("dft_2d: n0", n0)?;
        let n1_i = check_n("dft_2d: n1", n1)?;
        let mut scratch = vec![[0.0_f64, 0.0_f64]; n0 * n1];
        let p = scratch.as_mut_ptr() as *mut sys::fftw_complex;
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { sys::fftw_plan_dft_2d(n0_i, n1_i, p, p, direction.raw(), sys::FFTW_ESTIMATE) }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::Complex,
            n_total: n0 * n1,
        })
    }

    /// Build a 3-D complex DFT plan.
    pub fn dft_3d(n0: usize, n1: usize, n2: usize, direction: Direction) -> Result<Self> {
        if n0 == 0 || n1 == 0 || n2 == 0 {
            return Err(Error::InvalidArgument(
                "dft_3d: dimensions must be positive".into(),
            ));
        }
        let n0_i = check_n("dft_3d: n0", n0)?;
        let n1_i = check_n("dft_3d: n1", n1)?;
        let n2_i = check_n("dft_3d: n2", n2)?;
        let mut scratch = vec![[0.0_f64, 0.0_f64]; n0 * n1 * n2];
        let p = scratch.as_mut_ptr() as *mut sys::fftw_complex;
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                sys::fftw_plan_dft_3d(n0_i, n1_i, n2_i, p, p, direction.raw(), sys::FFTW_ESTIMATE)
            }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::Complex,
            n_total: n0 * n1 * n2,
        })
    }

    /// Build a 1-D real-to-complex forward plan.
    pub fn r2c_1d(n: usize) -> Result<Self> {
        if n == 0 {
            return Err(Error::InvalidArgument("r2c_1d: n must be positive".into()));
        }
        let n_i = check_n("r2c_1d", n)?;
        let mut in_buf = vec![0.0_f64; n];
        let mut out_buf = vec![[0.0_f64, 0.0_f64]; n / 2 + 1];
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                sys::fftw_plan_dft_r2c_1d(
                    n_i,
                    in_buf.as_mut_ptr(),
                    out_buf.as_mut_ptr() as *mut sys::fftw_complex,
                    sys::FFTW_ESTIMATE,
                )
            }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::R2C,
            n_total: n,
        })
    }

    /// Build a 1-D complex-to-real backward plan.
    pub fn c2r_1d(n: usize) -> Result<Self> {
        if n == 0 {
            return Err(Error::InvalidArgument("c2r_1d: n must be positive".into()));
        }
        let n_i = check_n("c2r_1d", n)?;
        let mut in_buf = vec![[0.0_f64, 0.0_f64]; n / 2 + 1];
        let mut out_buf = vec![0.0_f64; n];
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                sys::fftw_plan_dft_c2r_1d(
                    n_i,
                    in_buf.as_mut_ptr() as *mut sys::fftw_complex,
                    out_buf.as_mut_ptr(),
                    sys::FFTW_ESTIMATE,
                )
            }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::C2R,
            n_total: n,
        })
    }

    /// Execute a complex DFT plan against new buffers of the matching shape.
    pub fn execute_dft(&self, input: &mut [[f64; 2]], output: &mut [[f64; 2]]) -> Result<()> {
        if self.kind != PlanKind::Complex {
            return Err(Error::InvalidArgument(
                "execute_dft: plan is not a complex DFT".into(),
            ));
        }
        if input.len() < self.n_total || output.len() < self.n_total {
            return Err(Error::InvalidArgument(format!(
                "execute_dft: buffers ({}, {}) smaller than plan size {}",
                input.len(),
                output.len(),
                self.n_total
            )));
        }
        unsafe {
            sys::fftw_execute_dft(
                self.plan,
                input.as_mut_ptr() as *mut sys::fftw_complex,
                output.as_mut_ptr() as *mut sys::fftw_complex,
            );
        }
        Ok(())
    }

    /// Execute a real-to-complex plan against new buffers.
    pub fn execute_r2c(&self, input: &mut [f64], output: &mut [[f64; 2]]) -> Result<()> {
        if self.kind != PlanKind::R2C {
            return Err(Error::InvalidArgument(
                "execute_r2c: plan is not an r2c plan".into(),
            ));
        }
        let need_out = self.n_total / 2 + 1;
        if input.len() < self.n_total || output.len() < need_out {
            return Err(Error::InvalidArgument(format!(
                "execute_r2c: input {} < n={}; output {} < n/2+1={}",
                input.len(),
                self.n_total,
                output.len(),
                need_out
            )));
        }
        unsafe {
            sys::fftw_execute_dft_r2c(
                self.plan,
                input.as_mut_ptr(),
                output.as_mut_ptr() as *mut sys::fftw_complex,
            );
        }
        Ok(())
    }

    /// Execute a complex-to-real plan against new buffers.
    pub fn execute_c2r(&self, input: &mut [[f64; 2]], output: &mut [f64]) -> Result<()> {
        if self.kind != PlanKind::C2R {
            return Err(Error::InvalidArgument(
                "execute_c2r: plan is not a c2r plan".into(),
            ));
        }
        let need_in = self.n_total / 2 + 1;
        if input.len() < need_in || output.len() < self.n_total {
            return Err(Error::InvalidArgument(format!(
                "execute_c2r: input {} < n/2+1={}; output {} < n={}",
                input.len(),
                need_in,
                output.len(),
                self.n_total
            )));
        }
        unsafe {
            sys::fftw_execute_dft_c2r(
                self.plan,
                input.as_mut_ptr() as *mut sys::fftw_complex,
                output.as_mut_ptr(),
            );
        }
        Ok(())
    }
}

impl Drop for Plan {
    fn drop(&mut self) {
        if !self.plan.is_null() {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { sys::fftw_destroy_plan(self.plan) };
            self.plan = std::ptr::null_mut();
        }
    }
}

impl std::fmt::Debug for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plan")
            .field("kind", &self.kind)
            .field("n_total", &self.n_total)
            .finish_non_exhaustive()
    }
}

// =========================================================================
//   Single-precision (f32) one-shot complex DFTs
// =========================================================================

/// Compute a 1-D complex DFT in place over `f32` samples.
pub fn dft_1d_inplace_f32(direction: Direction, data: &mut [[f32; 2]]) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    let n = check_n("dft_1d_inplace_f32", data.len())?;
    let ptr = data.as_mut_ptr() as *mut sys::fftwf_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftwf_plan_dft_1d(n, ptr, ptr, direction.raw(), sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftwf_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftwf_destroy_plan(plan) };
    Ok(())
}

/// Compute a 2-D complex DFT in place over `f32` samples (row-major).
pub fn dft_2d_inplace_f32(
    direction: Direction,
    n0: usize,
    n1: usize,
    data: &mut [[f32; 2]],
) -> Result<()> {
    if n0 == 0 || n1 == 0 {
        return Ok(());
    }
    let need = n0 * n1;
    if data.len() < need {
        return Err(Error::InvalidArgument(format!(
            "dft_2d_inplace_f32: data length {} < n0·n1 = {need}",
            data.len()
        )));
    }
    let n0 = check_n("dft_2d_inplace_f32: n0", n0)?;
    let n1 = check_n("dft_2d_inplace_f32: n1", n1)?;
    let ptr = data.as_mut_ptr() as *mut sys::fftwf_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftwf_plan_dft_2d(n0, n1, ptr, ptr, direction.raw(), sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftwf_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftwf_destroy_plan(plan) };
    Ok(())
}

/// Compute a 3-D complex DFT in place over `f32` samples.
pub fn dft_3d_inplace_f32(
    direction: Direction,
    n0: usize,
    n1: usize,
    n2: usize,
    data: &mut [[f32; 2]],
) -> Result<()> {
    if n0 == 0 || n1 == 0 || n2 == 0 {
        return Ok(());
    }
    let need = n0 * n1 * n2;
    if data.len() < need {
        return Err(Error::InvalidArgument(format!(
            "dft_3d_inplace_f32: data length {} < n0·n1·n2 = {need}",
            data.len()
        )));
    }
    let n0 = check_n("dft_3d_inplace_f32: n0", n0)?;
    let n1 = check_n("dft_3d_inplace_f32: n1", n1)?;
    let n2 = check_n("dft_3d_inplace_f32: n2", n2)?;
    let ptr = data.as_mut_ptr() as *mut sys::fftwf_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftwf_plan_dft_3d(n0, n1, n2, ptr, ptr, direction.raw(), sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftwf_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftwf_destroy_plan(plan) };
    Ok(())
}

/// Convenience: forward 1-D `f32` DFT in place.
pub fn forward_inplace_f32(data: &mut [[f32; 2]]) -> Result<()> {
    dft_1d_inplace_f32(Direction::Forward, data)
}

/// Convenience: backward (unscaled) 1-D `f32` DFT in place.
pub fn backward_inplace_f32(data: &mut [[f32; 2]]) -> Result<()> {
    dft_1d_inplace_f32(Direction::Backward, data)
}

// =========================================================================
//   Single-precision (f32) real ↔ complex one-shot (1-D)
// =========================================================================

/// Compute a 1-D real-to-complex (forward) `f32` DFT.
///
/// `output` must hold at least `n/2 + 1` complex samples.
pub fn r2c_1d_f32(input: &mut [f32], output: &mut [[f32; 2]]) -> Result<()> {
    let n = input.len();
    if n == 0 {
        return Ok(());
    }
    let need_out = n / 2 + 1;
    if output.len() < need_out {
        return Err(Error::InvalidArgument(format!(
            "r2c_1d_f32: output length {} < n/2+1 = {need_out}",
            output.len()
        )));
    }
    let n_i = check_n("r2c_1d_f32", n)?;
    let in_p = input.as_mut_ptr();
    let out_p = output.as_mut_ptr() as *mut sys::fftwf_complex;
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftwf_plan_dft_r2c_1d(n_i, in_p, out_p, sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftwf_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftwf_destroy_plan(plan) };
    Ok(())
}

/// Compute a 1-D complex-to-real (backward) `f32` DFT.
///
/// `n` is the size of the *output*. As with the f64 version, FFTW does
/// not normalise — divide by `n` to recover the original after a
/// forward+backward round-trip.
pub fn c2r_1d_f32(n: usize, input: &mut [[f32; 2]], output: &mut [f32]) -> Result<()> {
    if n == 0 {
        return Ok(());
    }
    let need_in = n / 2 + 1;
    if input.len() < need_in {
        return Err(Error::InvalidArgument(format!(
            "c2r_1d_f32: input length {} < n/2+1 = {need_in}",
            input.len()
        )));
    }
    if output.len() < n {
        return Err(Error::InvalidArgument(format!(
            "c2r_1d_f32: output length {} < n = {n}",
            output.len()
        )));
    }
    let n_i = check_n("c2r_1d_f32", n)?;
    let in_p = input.as_mut_ptr() as *mut sys::fftwf_complex;
    let out_p = output.as_mut_ptr();
    let plan = {
        let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { sys::fftwf_plan_dft_c2r_1d(n_i, in_p, out_p, sys::FFTW_ESTIMATE) }
    };
    if plan.is_null() {
        return Err(Error::AllocationFailed("fft"));
    }
    unsafe { sys::fftwf_execute(plan) };
    let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { sys::fftwf_destroy_plan(plan) };
    Ok(())
}

// =========================================================================
//   Reusable single-precision plan
// =========================================================================

/// A reusable single-precision FFTW plan over a fixed shape.
///
/// Mirrors [`Plan`] but for `f32` / `fftwf_complex` buffers.
pub struct PlanF32 {
    plan: sys::fftwf_plan,
    kind: PlanKind,
    n_total: usize,
}

unsafe impl Send for PlanF32 {}

impl PlanF32 {
    /// Build a 1-D complex DFT plan over `n` samples.
    pub fn dft_1d(n: usize, direction: Direction) -> Result<Self> {
        if n == 0 {
            return Err(Error::InvalidArgument("dft_1d: n must be positive".into()));
        }
        let n_i = check_n("dft_1d", n)?;
        let mut scratch = vec![[0.0_f32, 0.0_f32]; n];
        let p = scratch.as_mut_ptr() as *mut sys::fftwf_complex;
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { sys::fftwf_plan_dft_1d(n_i, p, p, direction.raw(), sys::FFTW_ESTIMATE) }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::Complex,
            n_total: n,
        })
    }

    /// Build a 2-D complex DFT plan over `n0 × n1` samples.
    pub fn dft_2d(n0: usize, n1: usize, direction: Direction) -> Result<Self> {
        if n0 == 0 || n1 == 0 {
            return Err(Error::InvalidArgument(
                "dft_2d: dimensions must be positive".into(),
            ));
        }
        let n0_i = check_n("dft_2d: n0", n0)?;
        let n1_i = check_n("dft_2d: n1", n1)?;
        let mut scratch = vec![[0.0_f32, 0.0_f32]; n0 * n1];
        let p = scratch.as_mut_ptr() as *mut sys::fftwf_complex;
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { sys::fftwf_plan_dft_2d(n0_i, n1_i, p, p, direction.raw(), sys::FFTW_ESTIMATE) }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::Complex,
            n_total: n0 * n1,
        })
    }

    /// Build a 3-D complex DFT plan.
    pub fn dft_3d(n0: usize, n1: usize, n2: usize, direction: Direction) -> Result<Self> {
        if n0 == 0 || n1 == 0 || n2 == 0 {
            return Err(Error::InvalidArgument(
                "dft_3d: dimensions must be positive".into(),
            ));
        }
        let n0_i = check_n("dft_3d: n0", n0)?;
        let n1_i = check_n("dft_3d: n1", n1)?;
        let n2_i = check_n("dft_3d: n2", n2)?;
        let mut scratch = vec![[0.0_f32, 0.0_f32]; n0 * n1 * n2];
        let p = scratch.as_mut_ptr() as *mut sys::fftwf_complex;
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                sys::fftwf_plan_dft_3d(n0_i, n1_i, n2_i, p, p, direction.raw(), sys::FFTW_ESTIMATE)
            }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::Complex,
            n_total: n0 * n1 * n2,
        })
    }

    /// Build a 1-D real-to-complex forward plan.
    pub fn r2c_1d(n: usize) -> Result<Self> {
        if n == 0 {
            return Err(Error::InvalidArgument("r2c_1d: n must be positive".into()));
        }
        let n_i = check_n("r2c_1d", n)?;
        let mut in_buf = vec![0.0_f32; n];
        let mut out_buf = vec![[0.0_f32, 0.0_f32]; n / 2 + 1];
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                sys::fftwf_plan_dft_r2c_1d(
                    n_i,
                    in_buf.as_mut_ptr(),
                    out_buf.as_mut_ptr() as *mut sys::fftwf_complex,
                    sys::FFTW_ESTIMATE,
                )
            }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::R2C,
            n_total: n,
        })
    }

    /// Build a 1-D complex-to-real backward plan.
    pub fn c2r_1d(n: usize) -> Result<Self> {
        if n == 0 {
            return Err(Error::InvalidArgument("c2r_1d: n must be positive".into()));
        }
        let n_i = check_n("c2r_1d", n)?;
        let mut in_buf = vec![[0.0_f32, 0.0_f32]; n / 2 + 1];
        let mut out_buf = vec![0.0_f32; n];
        let plan = {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe {
                sys::fftwf_plan_dft_c2r_1d(
                    n_i,
                    in_buf.as_mut_ptr() as *mut sys::fftwf_complex,
                    out_buf.as_mut_ptr(),
                    sys::FFTW_ESTIMATE,
                )
            }
        };
        if plan.is_null() {
            return Err(Error::AllocationFailed("fft"));
        }
        Ok(Self {
            plan,
            kind: PlanKind::C2R,
            n_total: n,
        })
    }

    /// Execute a complex DFT plan against new buffers.
    pub fn execute_dft(&self, input: &mut [[f32; 2]], output: &mut [[f32; 2]]) -> Result<()> {
        if self.kind != PlanKind::Complex {
            return Err(Error::InvalidArgument(
                "execute_dft: plan is not a complex DFT".into(),
            ));
        }
        if input.len() < self.n_total || output.len() < self.n_total {
            return Err(Error::InvalidArgument(format!(
                "execute_dft: buffers ({}, {}) smaller than plan size {}",
                input.len(),
                output.len(),
                self.n_total
            )));
        }
        unsafe {
            sys::fftwf_execute_dft(
                self.plan,
                input.as_mut_ptr() as *mut sys::fftwf_complex,
                output.as_mut_ptr() as *mut sys::fftwf_complex,
            );
        }
        Ok(())
    }

    /// Execute a real-to-complex plan against new buffers.
    pub fn execute_r2c(&self, input: &mut [f32], output: &mut [[f32; 2]]) -> Result<()> {
        if self.kind != PlanKind::R2C {
            return Err(Error::InvalidArgument(
                "execute_r2c: plan is not an r2c plan".into(),
            ));
        }
        let need_out = self.n_total / 2 + 1;
        if input.len() < self.n_total || output.len() < need_out {
            return Err(Error::InvalidArgument(format!(
                "execute_r2c: input {} < n={}; output {} < n/2+1={}",
                input.len(),
                self.n_total,
                output.len(),
                need_out
            )));
        }
        unsafe {
            sys::fftwf_execute_dft_r2c(
                self.plan,
                input.as_mut_ptr(),
                output.as_mut_ptr() as *mut sys::fftwf_complex,
            );
        }
        Ok(())
    }

    /// Execute a complex-to-real plan against new buffers.
    pub fn execute_c2r(&self, input: &mut [[f32; 2]], output: &mut [f32]) -> Result<()> {
        if self.kind != PlanKind::C2R {
            return Err(Error::InvalidArgument(
                "execute_c2r: plan is not a c2r plan".into(),
            ));
        }
        let need_in = self.n_total / 2 + 1;
        if input.len() < need_in || output.len() < self.n_total {
            return Err(Error::InvalidArgument(format!(
                "execute_c2r: input {} < n/2+1={}; output {} < n={}",
                input.len(),
                need_in,
                output.len(),
                self.n_total
            )));
        }
        unsafe {
            sys::fftwf_execute_dft_c2r(
                self.plan,
                input.as_mut_ptr() as *mut sys::fftwf_complex,
                output.as_mut_ptr(),
            );
        }
        Ok(())
    }
}

impl Drop for PlanF32 {
    fn drop(&mut self) {
        if !self.plan.is_null() {
            let _g = PLANNER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { sys::fftwf_destroy_plan(self.plan) };
            self.plan = std::ptr::null_mut();
        }
    }
}

impl std::fmt::Debug for PlanF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanF32")
            .field("kind", &self.kind)
            .field("n_total", &self.n_total)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dft_inverse_recovers_input() {
        let original: [[f64; 2]; 4] = [[1.0, 0.0], [2.0, 0.0], [3.0, 0.0], [4.0, 0.0]];
        let mut buf = original;
        forward_inplace(&mut buf).unwrap();
        backward_inplace(&mut buf).unwrap();
        let n = original.len() as f64;
        for (got, orig) in buf.iter().zip(original.iter()) {
            assert!((got[0] / n - orig[0]).abs() < 1e-12);
            assert!((got[1] / n - orig[1]).abs() < 1e-12);
        }
    }

    #[test]
    fn dc_signal_concentrates_at_dc() {
        let n = 8;
        let c = 1.5_f64;
        let mut buf: Vec<[f64; 2]> = (0..n).map(|_| [c, 0.0]).collect();
        forward_inplace(&mut buf).unwrap();
        let n_f = n as f64;
        assert!((buf[0][0] - c * n_f).abs() < 1e-12);
        assert!(buf[0][1].abs() < 1e-12);
        for v in &buf[1..] {
            assert!(v[0].abs() < 1e-12);
            assert!(v[1].abs() < 1e-12);
        }
    }

    #[test]
    fn empty_input_is_ok() {
        let mut empty: [[f64; 2]; 0] = [];
        forward_inplace(&mut empty).unwrap();
    }

    #[test]
    fn dft_2d_dc() {
        // 4×4 DC signal → spectrum has all energy at (0,0).
        let n0 = 4;
        let n1 = 4;
        let mut buf: Vec<[f64; 2]> = vec![[1.0, 0.0]; n0 * n1];
        dft_2d_inplace(Direction::Forward, n0, n1, &mut buf).unwrap();
        let total = (n0 * n1) as f64;
        assert!((buf[0][0] - total).abs() < 1e-9);
        assert!(buf[0][1].abs() < 1e-9);
        for v in &buf[1..] {
            assert!(v[0].abs() < 1e-9);
            assert!(v[1].abs() < 1e-9);
        }
    }

    #[test]
    fn r2c_then_c2r_round_trip() {
        let original = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let n = original.len();
        let mut input = original;
        let mut spectrum = vec![[0.0_f64, 0.0]; n / 2 + 1];
        r2c_1d(&mut input, &mut spectrum).unwrap();
        let mut recovered = vec![0.0_f64; n];
        c2r_1d(n, &mut spectrum, &mut recovered).unwrap();
        // FFTW does not normalize; round-trip multiplies by n.
        for (got, orig) in recovered.iter().zip(original.iter()) {
            assert!((got / n as f64 - orig).abs() < 1e-10);
        }
    }

    #[test]
    fn plan_executes_repeatedly() {
        let plan = Plan::dft_1d(4, Direction::Forward).unwrap();
        let mut a: Vec<[f64; 2]> = vec![[1.0, 0.0]; 4];
        let mut out_a = vec![[0.0, 0.0]; 4];
        plan.execute_dft(&mut a, &mut out_a).unwrap();
        assert!((out_a[0][0] - 4.0).abs() < 1e-9);

        let mut b: Vec<[f64; 2]> = vec![[2.0, 0.0]; 4];
        let mut out_b = vec![[0.0, 0.0]; 4];
        plan.execute_dft(&mut b, &mut out_b).unwrap();
        assert!((out_b[0][0] - 8.0).abs() < 1e-9);
    }

    #[test]
    fn plan_kind_mismatch_is_error() {
        let plan = Plan::r2c_1d(8).unwrap();
        let mut input: Vec<[f64; 2]> = vec![[0.0, 0.0]; 8];
        let mut output: Vec<[f64; 2]> = vec![[0.0, 0.0]; 8];
        let err = plan.execute_dft(&mut input, &mut output).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));
    }

    // -------------------------------------------------------------------
    //   f32 tests
    // -------------------------------------------------------------------

    #[test]
    fn dft_inverse_recovers_input_f32() {
        let original: [[f32; 2]; 4] = [[1.0, 0.0], [2.0, 0.0], [3.0, 0.0], [4.0, 0.0]];
        let mut buf = original;
        forward_inplace_f32(&mut buf).unwrap();
        backward_inplace_f32(&mut buf).unwrap();
        let n = original.len() as f32;
        for (got, orig) in buf.iter().zip(original.iter()) {
            assert!((got[0] / n - orig[0]).abs() < 1e-5);
            assert!((got[1] / n - orig[1]).abs() < 1e-5);
        }
    }

    #[test]
    fn dc_signal_concentrates_at_dc_f32() {
        let n = 8;
        let c = 1.5_f32;
        let mut buf: Vec<[f32; 2]> = (0..n).map(|_| [c, 0.0]).collect();
        forward_inplace_f32(&mut buf).unwrap();
        let n_f = n as f32;
        assert!((buf[0][0] - c * n_f).abs() < 1e-4);
        assert!(buf[0][1].abs() < 1e-4);
        for v in &buf[1..] {
            assert!(v[0].abs() < 1e-4);
            assert!(v[1].abs() < 1e-4);
        }
    }

    #[test]
    fn r2c_then_c2r_round_trip_f32() {
        let original = [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let n = original.len();
        let mut input = original;
        let mut spectrum = vec![[0.0_f32, 0.0]; n / 2 + 1];
        r2c_1d_f32(&mut input, &mut spectrum).unwrap();
        let mut recovered = vec![0.0_f32; n];
        c2r_1d_f32(n, &mut spectrum, &mut recovered).unwrap();
        for (got, orig) in recovered.iter().zip(original.iter()) {
            assert!((got / n as f32 - orig).abs() < 1e-4);
        }
    }

    #[test]
    fn plan_f32_executes_repeatedly() {
        let plan = PlanF32::dft_1d(4, Direction::Forward).unwrap();
        let mut a: Vec<[f32; 2]> = vec![[1.0, 0.0]; 4];
        let mut out_a = vec![[0.0_f32, 0.0]; 4];
        plan.execute_dft(&mut a, &mut out_a).unwrap();
        assert!((out_a[0][0] - 4.0).abs() < 1e-4);

        let mut b: Vec<[f32; 2]> = vec![[2.0, 0.0]; 4];
        let mut out_b = vec![[0.0_f32, 0.0]; 4];
        plan.execute_dft(&mut b, &mut out_b).unwrap();
        assert!((out_b[0][0] - 8.0).abs() < 1e-4);
    }
}

/// Wrapper tests against naive O(N^2) DFT references (computed in f64, independent of the library).
#[cfg(test)]
mod reference_tests {
    use super::*;

    /// Deterministic pseudo-random signal in [-1, 1) (LCG, fixed seed).
    fn signal(len: usize, seed: u64) -> Vec<[f64; 2]> {
        let mut s = seed;
        let mut next = || {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((s >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
        };
        (0..len).map(|_| [next(), next()]).collect()
    }

    /// Naive N-d DFT over row-major `shape`; sign -1 = forward, +1 = backward (unnormalised).
    fn naive(shape: &[usize], x: &[[f64; 2]], sign: f64) -> Vec<[f64; 2]> {
        let total: usize = shape.iter().product();
        let idx = |mut flat: usize| -> Vec<usize> {
            let mut v = vec![0; shape.len()];
            for d in (0..shape.len()).rev() {
                v[d] = flat % shape[d];
                flat /= shape[d];
            }
            v
        };
        (0..total)
            .map(|k| {
                let kk = idx(k);
                let (mut re, mut im) = (0.0, 0.0);
                for (j, xj) in x.iter().enumerate() {
                    let jj = idx(j);
                    let mut phase = 0.0;
                    for d in 0..shape.len() {
                        phase += (kk[d] * jj[d]) as f64 / shape[d] as f64;
                    }
                    let a = sign * 2.0 * std::f64::consts::PI * phase;
                    re += xj[0] * a.cos() - xj[1] * a.sin();
                    im += xj[0] * a.sin() + xj[1] * a.cos();
                }
                [re, im]
            })
            .collect()
    }

    /// Naive real-to-complex DFT, first n/2+1 bins.
    fn naive_r2c(x: &[f64]) -> Vec<[f64; 2]> {
        let cx: Vec<[f64; 2]> = x.iter().map(|&v| [v, 0.0]).collect();
        naive(&[x.len()], &cx, -1.0)[..x.len() / 2 + 1].to_vec()
    }

    macro_rules! fft_ref_tests {
        ($m:ident, $T:ty, $tol:expr, $Plan:ident, $d1:ident, $d2:ident, $d3:ident,
         $r2c:ident, $c2r:ident) => {
            mod $m {
                use super::*;
                const TOL: f64 = $tol;

                fn to_t(v: &[[f64; 2]]) -> Vec<[$T; 2]> {
                    v.iter().map(|c| [c[0] as $T, c[1] as $T]).collect()
                }
                fn check(got: &[[$T; 2]], want: &[[f64; 2]], what: &str) {
                    assert_eq!(got.len(), want.len());
                    for (i, (g, w)) in got.iter().zip(want).enumerate() {
                        assert!(
                            (g[0] as f64 - w[0]).abs() < TOL && (g[1] as f64 - w[1]).abs() < TOL,
                            "{what}[{i}]: got {g:?} want {w:?}"
                        );
                    }
                }
                // Reference computed from the f32/f64-rounded input so only the transform differs.
                fn rounded(v: &[[f64; 2]]) -> Vec<[f64; 2]> {
                    to_t(v).iter().map(|c| [c[0] as f64, c[1] as f64]).collect()
                }

                #[test]
                fn one_d_inplace_matches_naive_dft() {
                    for &n in &[1usize, 2, 7, 12, 16] {
                        for (dir, sign) in [(Direction::Forward, -1.0), (Direction::Backward, 1.0)]
                        {
                            let x = rounded(&signal(n, 11 + n as u64));
                            let mut buf = to_t(&x);
                            $d1(dir, &mut buf).unwrap();
                            check(&buf, &naive(&[n], &x, sign), "dft_1d_inplace");
                        }
                    }
                }

                #[test]
                fn one_d_inplace_impulse_and_empty() {
                    // delta at index 1 -> X[k] = exp(-2 pi i k / n)
                    let n = 8;
                    let mut buf = vec![[0 as $T, 0 as $T]; n];
                    buf[1] = [1 as $T, 0 as $T];
                    $d1(Direction::Forward, &mut buf).unwrap();
                    let want: Vec<[f64; 2]> = (0..n)
                        .map(|k| {
                            let a = -2.0 * std::f64::consts::PI * k as f64 / n as f64;
                            [a.cos(), a.sin()]
                        })
                        .collect();
                    check(&buf, &want, "impulse");
                    let mut empty: [[$T; 2]; 0] = [];
                    $d1(Direction::Forward, &mut empty).unwrap();
                }

                #[test]
                fn two_d_inplace_matches_naive_and_rejects_short_buffer() {
                    for &(n0, n1) in &[(3usize, 5usize), (4, 4), (1, 6)] {
                        let x = rounded(&signal(n0 * n1, 21));
                        let mut buf = to_t(&x);
                        $d2(Direction::Forward, n0, n1, &mut buf).unwrap();
                        check(&buf, &naive(&[n0, n1], &x, -1.0), "dft_2d_inplace");
                    }
                    let mut short = vec![[0 as $T, 0 as $T]; 5];
                    assert!(matches!(
                        $d2(Direction::Forward, 3, 2, &mut short),
                        Err(Error::InvalidArgument(_))
                    ));
                }

                #[test]
                fn three_d_inplace_matches_naive_and_rejects_short_buffer() {
                    for &(n0, n1, n2) in &[(2usize, 3usize, 4usize), (3, 3, 3)] {
                        for (dir, sign) in [(Direction::Forward, -1.0), (Direction::Backward, 1.0)]
                        {
                            let x = rounded(&signal(n0 * n1 * n2, 31));
                            let mut buf = to_t(&x);
                            $d3(dir, n0, n1, n2, &mut buf).unwrap();
                            check(&buf, &naive(&[n0, n1, n2], &x, sign), "dft_3d_inplace");
                        }
                    }
                    let mut short = vec![[0 as $T, 0 as $T]; 7];
                    assert!(matches!(
                        $d3(Direction::Forward, 2, 2, 2, &mut short),
                        Err(Error::InvalidArgument(_))
                    ));
                }

                #[test]
                fn plan_dft_2d_and_3d_match_naive() {
                    let x = rounded(&signal(12, 41));
                    let plan = $Plan::dft_2d(3, 4, Direction::Forward).unwrap();
                    let mut input = to_t(&x);
                    let mut out = vec![[0 as $T, 0 as $T]; 12];
                    plan.execute_dft(&mut input, &mut out).unwrap();
                    check(&out, &naive(&[3, 4], &x, -1.0), "Plan::dft_2d");

                    let x = rounded(&signal(24, 42));
                    let plan = $Plan::dft_3d(2, 3, 4, Direction::Backward).unwrap();
                    let mut input = to_t(&x);
                    let mut out = vec![[0 as $T, 0 as $T]; 24];
                    plan.execute_dft(&mut input, &mut out).unwrap();
                    check(&out, &naive(&[2, 3, 4], &x, 1.0), "Plan::dft_3d");
                }

                #[test]
                fn plan_r2c_matches_naive_and_c2r_inverts_it() {
                    for &n in &[8usize, 9] {
                        let real: Vec<f64> = signal(n, 51).iter().map(|c| c[0]).collect();
                        let real_t: Vec<$T> = real.iter().map(|&v| v as $T).collect();
                        let real_r: Vec<f64> = real_t.iter().map(|&v| v as f64).collect();
                        let fwd = $Plan::r2c_1d(n).unwrap();
                        let mut input = real_t.clone();
                        let mut spec = vec![[0 as $T, 0 as $T]; n / 2 + 1];
                        fwd.execute_r2c(&mut input, &mut spec).unwrap();
                        check(&spec, &naive_r2c(&real_r), "execute_r2c");

                        // c2r of the reference spectrum gives n * x (unnormalised).
                        let bwd = $Plan::c2r_1d(n).unwrap();
                        let mut cin = to_t(&naive_r2c(&real_r));
                        let mut out = vec![0 as $T; n];
                        bwd.execute_c2r(&mut cin, &mut out).unwrap();
                        for (g, w) in out.iter().zip(&real_r) {
                            assert!(
                                (*g as f64 / n as f64 - w).abs() < TOL,
                                "execute_c2r {g} vs {w}"
                            );
                        }
                    }
                }

                #[test]
                fn r2c_c2r_execute_reject_wrong_kind_or_short_buffers() {
                    let c2c = $Plan::dft_1d(8, Direction::Forward).unwrap();
                    let mut r = vec![0 as $T; 8];
                    let mut c = vec![[0 as $T, 0 as $T]; 8];
                    assert!(matches!(
                        c2c.execute_r2c(&mut r, &mut c),
                        Err(Error::InvalidArgument(_))
                    ));
                    assert!(matches!(
                        c2c.execute_c2r(&mut c, &mut r),
                        Err(Error::InvalidArgument(_))
                    ));
                    let p = $Plan::r2c_1d(8).unwrap();
                    let mut short = vec![[0 as $T, 0 as $T]; 4]; // needs 5
                    assert!(matches!(
                        p.execute_r2c(&mut r, &mut short),
                        Err(Error::InvalidArgument(_))
                    ));
                    let q = $Plan::c2r_1d(8).unwrap();
                    let mut sin = vec![[0 as $T, 0 as $T]; 4];
                    assert!(matches!(
                        q.execute_c2r(&mut sin, &mut r),
                        Err(Error::InvalidArgument(_))
                    ));
                }

                #[test]
                fn parseval_holds_for_3d_forward() {
                    // sum |X|^2 = N * sum |x|^2
                    let (n0, n1, n2) = (3, 4, 2);
                    let x = rounded(&signal(n0 * n1 * n2, 61));
                    let mut buf = to_t(&x);
                    $d3(Direction::Forward, n0, n1, n2, &mut buf).unwrap();
                    let e_t: f64 = buf
                        .iter()
                        .map(|c| (c[0] as f64).powi(2) + (c[1] as f64).powi(2))
                        .sum();
                    let e_x: f64 = x.iter().map(|c| c[0] * c[0] + c[1] * c[1]).sum();
                    assert!((e_t - 24.0 * e_x).abs() < TOL * 24.0 * e_x.max(1.0) * 10.0);
                }

                #[test]
                fn r2c_one_shot_matches_naive() {
                    let real: Vec<f64> = signal(10, 71).iter().map(|c| c[1]).collect();
                    let mut input: Vec<$T> = real.iter().map(|&v| v as $T).collect();
                    let r: Vec<f64> = input.iter().map(|&v| v as f64).collect();
                    let mut spec = vec![[0 as $T, 0 as $T]; 6];
                    $r2c(&mut input, &mut spec).unwrap();
                    check(&spec, &naive_r2c(&r), "r2c_1d");
                    let mut back = vec![0 as $T; 10];
                    $c2r(10, &mut spec, &mut back).unwrap();
                    for (g, w) in back.iter().zip(&r) {
                        assert!((*g as f64 / 10.0 - w).abs() < TOL);
                    }
                }
            }
        };
    }

    fft_ref_tests!(
        f64_ref,
        f64,
        1e-10,
        Plan,
        dft_1d_inplace,
        dft_2d_inplace,
        dft_3d_inplace,
        r2c_1d,
        c2r_1d
    );
    fft_ref_tests!(
        f32_ref,
        f32,
        2e-4,
        PlanF32,
        dft_1d_inplace_f32,
        dft_2d_inplace_f32,
        dft_3d_inplace_f32,
        r2c_1d_f32,
        c2r_1d_f32
    );
}
