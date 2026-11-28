// SPDX-License-Identifier: MIT OR Apache-2.0
//! Safe Rust wrappers for AOCL-OpenRNG (AMD's fork of Arm's OpenRNG; the VSL-compatible random number API).
//!
//! Built on [`aocl-openrng-sys`](../aocl_openrng_sys/). Nothing here was RUN by the lane that wrote it: the library is a PM-built
//! DLL that only the PM session runs (`docs/cleanroom/openrng/WRAPPER-HANDOFF.md` lists the commands and what each test records).
//! Claims below say which of them are checked in Rust, which are the library's own refusal, and which are UNMEASURED.
//!
//! # Width
//! `openrng_int_t` is 4 bytes in an LP64 library and 8 in an ILP64 one. The `ilp64` feature MUST match the library build; a
//! mismatch is silent at run time (see `aocl-openrng-sys`). Every count that goes to the library is converted with a checked
//! cast: a slice longer than `openrng_int_t::MAX` is an [`Error::InvalidArgument`], never a wrapped length.
//!
//! # Soundness rules this crate follows
//! * An output slice's length is the library's `n`; the library never gets a count larger than the slice.
//! * Arrays whose length the library derives from other arguments (Gaussian multivariate, multinomial, Poisson with a lambda per
//!   element) are length-checked in Rust first.
//! * Stream state from memory is only accepted as a [`SavedStream`] this crate produced; loading arbitrary bytes or files, and
//!   the `n`-parameter initialisers whose minimum `n` is not documented, are `unsafe`.
//! * A [`Stream`] is not `Send`/`Sync` (thread-safety of a stream is UNMEASURED).
//!
//! Out-of-domain distribution parameters (negative sigma, p outside `[0, 1]`, ...) are passed to the library, which is expected to
//! refuse with `VSL_ERROR_BADARGS`; that refusal is recorded per distribution by `tests/domain.rs` (PM-run), not assumed.

use aocl_error::{Error, Result};
use aocl_openrng_sys as sys;
use core::ffi::{c_char, c_int};
use std::ffi::CString;

/// Convert a Rust length into the library's `openrng_int_t`, refusing what does not fit.
fn int(name: &str, len: usize) -> Result<sys::openrng_int_t> {
    sys::openrng_int_t::try_from(len).map_err(|_| {
        Error::InvalidArgument(format!(
            "{name}: {len} does not fit openrng_int_t ({} bytes in this build)",
            core::mem::size_of::<sys::openrng_int_t>()
        ))
    })
}

fn describe(code: c_int) -> &'static str {
    match code {
        sys::VSL_ERROR_FEATURE_NOT_IMPLEMENTED => "feature not implemented",
        sys::VSL_ERROR_BADARGS => "bad arguments",
        sys::VSL_ERROR_MEM_FAILURE => "memory failure",
        sys::VSL_ERROR_NULL_PTR => "null pointer",
        sys::VSL_RNG_ERROR_INVALID_BRNG_INDEX => "invalid basic generator index",
        sys::VSL_RNG_ERROR_LEAPFROG_UNSUPPORTED => "leapfrog unsupported by this generator",
        sys::VSL_RNG_ERROR_SKIPAHEAD_UNSUPPORTED => "skip-ahead unsupported by this generator",
        sys::VSL_RNG_ERROR_SKIPAHEADEX_UNSUPPORTED => {
            "extended skip-ahead unsupported by this generator"
        }
        sys::VSL_RNG_ERROR_BRNGS_INCOMPATIBLE => "basic generators are incompatible",
        sys::VSL_RNG_ERROR_BAD_STREAM => "bad stream",
        sys::VSL_RNG_ERROR_BAD_MEM_FORMAT => "bad memory format",
        sys::VSL_RNG_ERROR_BRNG_NOT_SUPPORTED => "basic generator not supported",
        sys::VSL_RNG_ERROR_FILE_OPEN => "file open failed",
        sys::VSL_RNG_ERROR_FILE_WRITE => "file write failed",
        sys::VSL_RNG_ERROR_FILE_READ => "file read failed",
        sys::VSL_RNG_ERROR_FILE_CLOSE => "file close failed",
        sys::VSL_RNG_ERROR_NONDETERM_NOT_SUPPORTED => "non-deterministic source not supported",
        _ => "unrecognised status",
    }
}

/// Map a library status to a `Result`. `VSL_ERROR_OK` is the only success value; a positive status (the header documents none)
/// is reported as an error too rather than guessed to be a warning.
fn check(status: c_int) -> Result<()> {
    if status == sys::VSL_ERROR_OK {
        return Ok(());
    }
    if status == sys::VSL_ERROR_MEM_FAILURE {
        return Err(Error::AllocationFailed("openrng"));
    }
    Err(Error::Status {
        component: "openrng",
        code: i64::from(status),
        message: describe(status).to_string(),
    })
}

/// `(major, minor, patch)` of the linked library (`openRngGetVersion`; it is in the header but not in the guide).
pub fn version() -> (i32, i32, i32) {
    let (mut a, mut b, mut c) = (0i32, 0i32, 0i32);
    // SAFETY: three valid, writable i32 out-pointers.
    unsafe { sys::openRngGetVersion(&mut a, &mut b, &mut c) };
    (a, b, c)
}

/// Number of basic generators registered with the library (`vslGetNumRegBrngs`).
pub fn registered_brngs() -> Result<usize> {
    // SAFETY: no arguments.
    let n = unsafe { sys::vslGetNumRegBrngs() };
    usize::try_from(n).map_err(|_| Error::Status {
        component: "openrng",
        code: i64::from(n),
        message: "vslGetNumRegBrngs returned a negative count".into(),
    })
}

/// A basic random number generator. `Wh` and `Mt2203` come in numbered families (the id is `base + index`, MKL convention); the
/// index semantics are UNMEASURED here, so they are reachable only through [`Stream::with_brng_id`] (`unsafe`).
/// `Nondeterm`, the abstract generators and `vslRegisterBrng` stay out of the safe API (they take callbacks or hardware sources).
/// `Sobol` and `Niederr` are quasi-random: they want a dimension, which is the first parameter of [`Stream::from_params`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Brng {
    Mcg31,
    R250,
    Mrg32k3a,
    Mcg59,
    Sobol,
    Niederr,
    Mt19937,
    Sfmt19937,
    Ars5,
    Philox4x32x10,
}

impl Brng {
    /// Every variant, in declaration order.
    pub const ALL: [Brng; 10] = [
        Brng::Mcg31,
        Brng::R250,
        Brng::Mrg32k3a,
        Brng::Mcg59,
        Brng::Sobol,
        Brng::Niederr,
        Brng::Mt19937,
        Brng::Sfmt19937,
        Brng::Ars5,
        Brng::Philox4x32x10,
    ];

    /// The library's id for this generator.
    pub fn id(self) -> i32 {
        match self {
            Brng::Mcg31 => sys::VSL_BRNG_MCG31,
            Brng::R250 => sys::VSL_BRNG_R250,
            Brng::Mrg32k3a => sys::VSL_BRNG_MRG32K3A,
            Brng::Mcg59 => sys::VSL_BRNG_MCG59,
            Brng::Sobol => sys::VSL_BRNG_SOBOL,
            Brng::Niederr => sys::VSL_BRNG_NIEDERR,
            Brng::Mt19937 => sys::VSL_BRNG_MT19937,
            Brng::Sfmt19937 => sys::VSL_BRNG_SFMT19937,
            Brng::Ars5 => sys::VSL_BRNG_ARS5,
            Brng::Philox4x32x10 => sys::VSL_BRNG_PHILOX4X32X10,
        }
    }

    /// The variant with this id, if it is one of the safe ones.
    pub fn from_id(id: i32) -> Option<Brng> {
        Brng::ALL.into_iter().find(|b| b.id() == id)
    }
}

/// Method for distributions that offer a standard and an accurate variant (uniform, exponential, Rayleigh, Weibull, gamma).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Accuracy {
    #[default]
    Standard,
    Accurate,
}

/// Method for the Gaussian distributions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GaussianMethod {
    #[default]
    BoxMuller,
    BoxMuller2,
    Icdf,
}

/// Method for the lognormal distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LognormalMethod {
    #[default]
    Icdf,
    BoxMuller2,
}

/// Method for the Poisson distribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PoissonMethod {
    #[default]
    Ptpe,
    PoisNorm,
}

/// Storage of the variance-covariance (Cholesky) matrix `t` of the multivariate Gaussian.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixStorage {
    /// `dimen * dimen` elements.
    Full,
    /// `dimen * (dimen + 1) / 2` elements.
    Packed,
    /// `dimen` elements.
    Diagonal,
}

impl MatrixStorage {
    fn id(self) -> c_int {
        match self {
            MatrixStorage::Full => sys::VSL_MATRIX_STORAGE_FULL,
            MatrixStorage::Packed => sys::VSL_MATRIX_STORAGE_PACKED,
            MatrixStorage::Diagonal => sys::VSL_MATRIX_STORAGE_DIAGONAL,
        }
    }
    /// Number of elements the matrix needs for `dimen`, or `None` on overflow.
    fn len(self, dimen: usize) -> Option<usize> {
        match self {
            MatrixStorage::Full => dimen.checked_mul(dimen),
            MatrixStorage::Packed => dimen.checked_mul(dimen.checked_add(1)?).map(|v| v / 2),
            MatrixStorage::Diagonal => Some(dimen),
        }
    }
}

fn acc(a: Accuracy) -> sys::openrng_int_t {
    match a {
        Accuracy::Standard => 0,
        Accuracy::Accurate => 1,
    }
}

/// A random stream: a basic generator and its state. Deleted on drop.
pub struct Stream {
    ptr: sys::VSLStreamStatePtr,
}

// `ptr` is a raw pointer, so `Stream` is neither `Send` nor `Sync`; the guide's thread-safety text is not yet read or measured.

impl Stream {
    /// `vslNewStream`: a stream of generator `brng` seeded with `seed`.
    pub fn new(brng: Brng, seed: u32) -> Result<Stream> {
        // SAFETY: the id is one of the library's generator ids; seed is by value.
        unsafe { Stream::with_brng_id(brng.id(), seed) }
    }

    /// `vslNewStream` with a raw generator id (for the numbered families).
    ///
    /// # Safety
    /// Whether the library validates an arbitrary id is UNMEASURED (the guide documents `VSL_RNG_ERROR_INVALID_BRNG_INDEX`).
    pub unsafe fn with_brng_id(id: i32, seed: u32) -> Result<Stream> {
        let mut ptr: sys::VSLStreamStatePtr = core::ptr::null_mut();
        // SAFETY: ptr is a valid out-pointer.
        let st = unsafe {
            sys::vslNewStream(
                &mut ptr,
                id as sys::openrng_int_t,
                seed as sys::openrng_uint_t,
            )
        };
        check(st)?;
        Ok(Stream { ptr })
    }

    /// `vslNewStreamEx`: initialise from `params`.
    ///
    /// # Safety
    /// How many parameters each generator reads is UNMEASURED: the library may read more than `params.len()`. Pass at least as
    /// many words as the generator's initialisation in the guide lists (`tests/params_guard.rs`, PM-run, records the minimum on
    /// a guard page for each generator; until that is read, treat any shorter slice as undefined behaviour).
    pub unsafe fn from_params(brng: Brng, params: &[u32]) -> Result<Stream> {
        let n = int("params", params.len())?;
        let mut ptr: sys::VSLStreamStatePtr = core::ptr::null_mut();
        // SAFETY: ptr is a valid out-pointer; params is readable for n words; the rest is the caller's contract.
        let st = unsafe {
            sys::vslNewStreamEx(
                &mut ptr,
                brng.id() as sys::openrng_int_t,
                n,
                params.as_ptr(),
            )
        };
        check(st)?;
        Ok(Stream { ptr })
    }

    /// The raw stream pointer, for calls into `aocl-openrng-sys` that this crate does not wrap.
    pub fn as_raw(&mut self) -> sys::VSLStreamStatePtr {
        self.ptr
    }

    /// `vslGetStreamStateBrng`: the generator id of this stream.
    pub fn brng_id(&self) -> Result<i32> {
        // SAFETY: self.ptr is a live stream.
        let id = unsafe { sys::vslGetStreamStateBrng(self.ptr) };
        if id < 0 {
            return check(id).map(|_| 0);
        }
        Ok(id)
    }

    /// `vslCopyStreamState`: copy the state of `src` into this stream. The generators must be the same.
    pub fn copy_state_from(&mut self, src: &Stream) -> Result<()> {
        // SAFETY: both are live streams.
        check(unsafe { sys::vslCopyStreamState(self.ptr, src.ptr) })
    }

    /// `vslLeapfrogStream`: this stream becomes the `k`-th of `nstreams` interleaved streams (`0 <= k < nstreams`).
    pub fn leapfrog(&mut self, k: usize, nstreams: usize) -> Result<()> {
        if nstreams == 0 || k >= nstreams {
            return Err(Error::InvalidArgument(format!(
                "leapfrog: need 0 <= k < nstreams, got k={k}, nstreams={nstreams}"
            )));
        }
        let (k, n) = (int("k", k)?, int("nstreams", nstreams)?);
        // SAFETY: live stream, scalars.
        check(unsafe { sys::vslLeapfrogStream(self.ptr, k, n) })
    }

    /// `vslSkipAheadStream`: skip `nskip` outputs.
    pub fn skip_ahead(&mut self, nskip: i64) -> Result<()> {
        if nskip < 0 {
            return Err(Error::InvalidArgument(format!(
                "skip_ahead: {nskip} is negative"
            )));
        }
        // SAFETY: live stream, scalar.
        check(unsafe { sys::vslSkipAheadStream(self.ptr, nskip) })
    }

    /// `vslSkipAheadStreamEx`: skip a count given as `nskip` 64-bit words (the library's own layout).
    ///
    /// # Safety
    /// The number of words each generator requires is UNMEASURED; fewer than it reads is a read past the slice.
    pub unsafe fn skip_ahead_ex(&mut self, nskip: &[u64]) -> Result<()> {
        let n = int("nskip", nskip.len())?;
        // SAFETY: nskip readable for n words; the rest is the caller's contract.
        check(unsafe { sys::vslSkipAheadStreamEx(self.ptr, n, nskip.as_ptr()) })
    }

    /// `vslSaveStreamM`: the state as an opaque value that [`SavedStream::restore`] turns back into a stream.
    pub fn save(&self) -> Result<SavedStream> {
        // SAFETY: live stream.
        let size = unsafe { sys::vslGetStreamSize(self.ptr) };
        if size < 0 {
            return check(size).map(|_| unreachable!());
        }
        let mut bytes = vec![0u8; size as usize];
        // SAFETY: the buffer is exactly the size the library reported for this stream.
        check(unsafe { sys::vslSaveStreamM(self.ptr, bytes.as_mut_ptr() as *mut c_char) })?;
        Ok(SavedStream { bytes })
    }

    /// `vslSaveStreamF`: save the state to a file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let c = path_cstring(path)?;
        // SAFETY: live stream, NUL-terminated path.
        check(unsafe { sys::vslSaveStreamF(self.ptr, c.as_ptr()) })
    }

    /// `vslLoadStreamF`: a stream from a file written by [`Stream::save_to_file`].
    ///
    /// # Safety
    /// The library trusts the file's contents (a crafted file can name a generator and sizes the library then believes). Only
    /// load files this program or a trusted one wrote.
    pub unsafe fn load_from_file(path: &std::path::Path) -> Result<Stream> {
        let c = path_cstring(path)?;
        let mut ptr: sys::VSLStreamStatePtr = core::ptr::null_mut();
        // SAFETY: out-pointer and path are valid; the contents are the caller's contract.
        check(unsafe { sys::vslLoadStreamF(&mut ptr, c.as_ptr()) })?;
        Ok(Stream { ptr })
    }

    /// `vslLoadStreamM` from raw bytes.
    ///
    /// # Safety
    /// The library reads as many bytes as the saved stream's own header says, not as many as `bytes.len()`; `bytes` must come from
    /// [`SavedStream::as_bytes`] of an untruncated value. Prefer [`SavedStream::restore`].
    pub unsafe fn load_from_memory(bytes: &[u8]) -> Result<Stream> {
        let mut ptr: sys::VSLStreamStatePtr = core::ptr::null_mut();
        // SAFETY: out-pointer is valid; the length is the caller's contract.
        check(unsafe { sys::vslLoadStreamM(&mut ptr, bytes.as_ptr() as *const c_char) })?;
        Ok(Stream { ptr })
    }

    // ---- uniform bits ----

    /// `viRngUniformBits`: fills `out` with the generator's raw 32-bit-or-wider words (see the guide: the word is generator
    /// dependent).
    pub fn uniform_bits(&mut self, out: &mut [u32]) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n words.
        check(unsafe { sys::viRngUniformBits(0, self.ptr, n, out.as_mut_ptr()) })
    }

    /// `viRngUniformBits32`: 32 random bits per element.
    pub fn uniform_bits32(&mut self, out: &mut [u32]) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n words.
        check(unsafe { sys::viRngUniformBits32(0, self.ptr, n, out.as_mut_ptr()) })
    }

    /// `viRngUniformBits64`: 64 random bits per element.
    pub fn uniform_bits64(&mut self, out: &mut [u64]) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n 64-bit words.
        check(unsafe { sys::viRngUniformBits64(0, self.ptr, n, out.as_mut_ptr()) })
    }

    // ---- integer distributions ----

    /// `viRngUniform`: integers uniform on `[a, b)`.
    pub fn uniform_int(&mut self, out: &mut [i32], a: i32, b: i32) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngUniform(0, self.ptr, n, out.as_mut_ptr(), a, b) })
    }

    /// `viRngBernoulli`.
    pub fn bernoulli(&mut self, out: &mut [i32], p: f64) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngBernoulli(0, self.ptr, n, out.as_mut_ptr(), p) })
    }

    /// `viRngBinomial`.
    pub fn binomial(&mut self, out: &mut [i32], ntrial: i32, p: f64) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngBinomial(0, self.ptr, n, out.as_mut_ptr(), ntrial, p) })
    }

    /// `viRngGeometric`.
    pub fn geometric(&mut self, out: &mut [i32], p: f64) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngGeometric(0, self.ptr, n, out.as_mut_ptr(), p) })
    }

    /// `viRngHypergeometric`: population `l`, `s` successes, `m` draws.
    pub fn hypergeometric(&mut self, out: &mut [i32], l: i32, s: i32, m: i32) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngHypergeometric(0, self.ptr, n, out.as_mut_ptr(), l, s, m) })
    }

    /// `viRngNegBinomial` (the header declares two spellings, `viRngNegBinomial` and `viRngNegbinomial`; this calls the first).
    pub fn neg_binomial(&mut self, out: &mut [i32], a: f64, p: f64) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngNegBinomial(0, self.ptr, n, out.as_mut_ptr(), a, p) })
    }

    /// `viRngNegbinomial`, the header's second spelling (recorded so a difference between the two would be seen).
    pub fn negbinomial(&mut self, out: &mut [i32], a: f64, p: f64) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n ints.
        check(unsafe { sys::viRngNegbinomial(0, self.ptr, n, out.as_mut_ptr(), a, p) })
    }

    /// `viRngPoisson`.
    pub fn poisson(&mut self, out: &mut [i32], lambda: f64, method: PoissonMethod) -> Result<()> {
        let n = int("out", out.len())?;
        let m = match method {
            PoissonMethod::Ptpe => sys::VSL_RNG_METHOD_POISSON_PTPE,
            PoissonMethod::PoisNorm => sys::VSL_RNG_METHOD_POISSON_POISNORM,
        };
        // SAFETY: out is writable for n ints.
        check(unsafe {
            sys::viRngPoisson(
                m as sys::openrng_int_t,
                self.ptr,
                n,
                out.as_mut_ptr(),
                lambda,
            )
        })
    }

    /// `viRngPoissonV`: one lambda per output element (`lambda.len() == out.len()`).
    pub fn poisson_v(&mut self, out: &mut [i32], lambda: &[f64]) -> Result<()> {
        if lambda.len() != out.len() {
            return Err(Error::InvalidArgument(format!(
                "poisson_v: {} outputs but {} lambdas",
                out.len(),
                lambda.len()
            )));
        }
        let n = int("out", out.len())?;
        // SAFETY: out is writable and lambda readable for n elements (equal lengths checked).
        check(unsafe { sys::viRngPoissonV(0, self.ptr, n, out.as_mut_ptr(), lambda.as_ptr()) })
    }

    /// `viRngMultinomial`: `out` holds `trials_count = out.len() / k` experiments of `k` counts each (`out.len()` must be a
    /// multiple of `k`, `p.len() == k`); each experiment distributes `ntrial` trials.
    pub fn multinomial(&mut self, out: &mut [i32], ntrial: i32, p: &[f64]) -> Result<()> {
        let k = p.len();
        if k == 0 || out.len() % k != 0 {
            return Err(Error::InvalidArgument(format!(
                "multinomial: out.len() {} must be a nonzero multiple of k = p.len() = {k}",
                out.len()
            )));
        }
        let n = int("experiments", out.len() / k)?;
        let kk = c_int::try_from(k)
            .map_err(|_| Error::InvalidArgument("multinomial: k does not fit int".into()))?;
        // SAFETY: out is writable for n*k ints and p readable for k doubles (checked above).
        check(unsafe {
            sys::viRngMultinomial(0, self.ptr, n, out.as_mut_ptr(), ntrial, kk, p.as_ptr())
        })
    }
}

fn path_cstring(path: &std::path::Path) -> Result<CString> {
    let s = path
        .to_str()
        .ok_or_else(|| Error::InvalidArgument("path is not valid UTF-8".into()))?;
    CString::new(s).map_err(|_| Error::InvalidArgument("path contains a NUL byte".into()))
}

impl Clone for Stream {
    /// `vslCopyStream`. Panics if the library cannot allocate the copy (use [`Stream::try_clone`] to see the status).
    fn clone(&self) -> Stream {
        self.try_clone().expect("vslCopyStream failed")
    }
}

impl Stream {
    /// `vslCopyStream` with the status returned.
    pub fn try_clone(&self) -> Result<Stream> {
        let mut ptr: sys::VSLStreamStatePtr = core::ptr::null_mut();
        // SAFETY: out-pointer valid; self.ptr is a live stream.
        check(unsafe { sys::vslCopyStream(&mut ptr, self.ptr) })?;
        Ok(Stream { ptr })
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // SAFETY: self.ptr is a stream this value owns; the library nulls it.
        let _ = unsafe { sys::vslDeleteStream(&mut self.ptr) };
    }
}

impl core::fmt::Debug for Stream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Stream")
            .field("brng_id", &self.brng_id().ok())
            .finish()
    }
}

/// A stream's state saved to memory by [`Stream::save`]. Opaque: only this crate makes one, so the library only ever reads a
/// buffer of exactly the size it wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedStream {
    bytes: Vec<u8>,
}

impl SavedStream {
    /// The saved bytes (for storage; to use them again hand them back through [`SavedStream::from_bytes`]'s contract).
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Rebuild a `SavedStream` from bytes previously taken from [`SavedStream::as_bytes`].
    ///
    /// # Safety
    /// The bytes must be exactly those of an earlier `as_bytes` of an untruncated, unmodified value.
    pub unsafe fn from_bytes(bytes: Vec<u8>) -> SavedStream {
        SavedStream { bytes }
    }

    /// `vslLoadStreamM`: a stream continuing from the saved state.
    pub fn restore(&self) -> Result<Stream> {
        // SAFETY: `bytes` came from the library's own vslSaveStreamM at its own reported size (see `save`).
        unsafe { Stream::load_from_memory(&self.bytes) }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Continuous distributions. Each generates a method for f64 (`name`) and for f32 (`name_f32`), `n = out.len()`.
// ---------------------------------------------------------------------------------------------------------------------

macro_rules! dist {
    ($(#[$m:meta])* $name:ident, $name32:ident, $d:ident, $s:ident; $meth:ty => |$mv:ident| $conv:expr; ($($arg:ident),*)) => {
        impl Stream {
            $(#[$m])*
            pub fn $name(&mut self, out: &mut [f64], $($arg: f64,)* $mv: $meth) -> Result<()> {
                let n = int("out", out.len())?;
                let method = $conv;
                // SAFETY: out is writable for n doubles.
                check(unsafe { sys::$d(method as sys::openrng_int_t, self.ptr, n, out.as_mut_ptr(), $($arg),*) })
            }
            $(#[$m])*
            pub fn $name32(&mut self, out: &mut [f32], $($arg: f32,)* $mv: $meth) -> Result<()> {
                let n = int("out", out.len())?;
                let method = $conv;
                // SAFETY: out is writable for n floats.
                check(unsafe { sys::$s(method as sys::openrng_int_t, self.ptr, n, out.as_mut_ptr(), $($arg),*) })
            }
        }
    };
}

dist!(/// `vdRngUniform` / `vsRngUniform`: uniform on `[a, b)`.
    uniform, uniform_f32, vdRngUniform, vsRngUniform; Accuracy => |m| acc(m); (a, b));
dist!(/// `vdRngGaussian` / `vsRngGaussian`.
    gaussian, gaussian_f32, vdRngGaussian, vsRngGaussian; GaussianMethod => |m| match m {
        GaussianMethod::BoxMuller => sys::VSL_RNG_METHOD_GAUSSIAN_BOXMULLER,
        GaussianMethod::BoxMuller2 => sys::VSL_RNG_METHOD_GAUSSIAN_BOXMULLER2,
        GaussianMethod::Icdf => sys::VSL_RNG_METHOD_GAUSSIAN_ICDF,
    }; (a, sigma));
dist!(/// `vdRngLognormal` / `vsRngLognormal`.
    lognormal, lognormal_f32, vdRngLognormal, vsRngLognormal; LognormalMethod => |m| match m {
        LognormalMethod::Icdf => sys::VSL_RNG_METHOD_LOGNORMAL_ICDF,
        LognormalMethod::BoxMuller2 => sys::VSL_RNG_METHOD_LOGNORMAL_BOXMULLER2,
    }; (alpha, sigma, b, beta));
dist!(/// `vdRngExponential` / `vsRngExponential`.
    exponential, exponential_f32, vdRngExponential, vsRngExponential; Accuracy => |m| acc(m); (a, beta));
dist!(/// `vdRngRayleigh` / `vsRngRayleigh`.
    rayleigh, rayleigh_f32, vdRngRayleigh, vsRngRayleigh; Accuracy => |m| acc(m); (a, beta));
dist!(/// `vdRngWeibull` / `vsRngWeibull`.
    weibull, weibull_f32, vdRngWeibull, vsRngWeibull; Accuracy => |m| acc(m); (alpha, a, beta));
dist!(/// `vdRngGamma` / `vsRngGamma`.
    gamma, gamma_f32, vdRngGamma, vsRngGamma; Accuracy => |m| acc(m); (alpha, a, beta));

// Distributions with a single method (id 0): the method argument is omitted.
macro_rules! dist0 {
    ($(#[$m:meta])* $name:ident, $name32:ident, $d:ident, $s:ident; ($($arg:ident),*)) => {
        impl Stream {
            $(#[$m])*
            pub fn $name(&mut self, out: &mut [f64], $($arg: f64),*) -> Result<()> {
                let n = int("out", out.len())?;
                // SAFETY: out is writable for n doubles.
                check(unsafe { sys::$d(0, self.ptr, n, out.as_mut_ptr(), $($arg),*) })
            }
            $(#[$m])*
            pub fn $name32(&mut self, out: &mut [f32], $($arg: f32),*) -> Result<()> {
                let n = int("out", out.len())?;
                // SAFETY: out is writable for n floats.
                check(unsafe { sys::$s(0, self.ptr, n, out.as_mut_ptr(), $($arg),*) })
            }
        }
    };
}

dist0!(/// `vdRngCauchy` / `vsRngCauchy`.
    cauchy, cauchy_f32, vdRngCauchy, vsRngCauchy; (a, beta));
dist0!(/// `vdRngGumbel` / `vsRngGumbel`.
    gumbel, gumbel_f32, vdRngGumbel, vsRngGumbel; (a, beta));
dist0!(/// `vdRngLaplace` / `vsRngLaplace`.
    laplace, laplace_f32, vdRngLaplace, vsRngLaplace; (a, beta));
dist0!(/// `vdRngBeta` / `vsRngBeta`.
    beta, beta_f32, vdRngBeta, vsRngBeta; (p, q, a, beta));

impl Stream {
    /// `vdRngChiSquare`: `v` degrees of freedom.
    pub fn chi_square(&mut self, out: &mut [f64], v: i32) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n doubles.
        check(unsafe { sys::vdRngChiSquare(0, self.ptr, n, out.as_mut_ptr(), v) })
    }

    /// `vsRngChiSquare`.
    pub fn chi_square_f32(&mut self, out: &mut [f32], v: i32) -> Result<()> {
        let n = int("out", out.len())?;
        // SAFETY: out is writable for n floats.
        check(unsafe { sys::vsRngChiSquare(0, self.ptr, n, out.as_mut_ptr(), v) })
    }

    /// `vdRngGaussianMV`: `out.len() / dimen` vectors of dimension `dimen`; `mean` has `dimen` elements; `chol` is the
    /// variance-covariance matrix's Cholesky factor in `storage` (the exact element counts are checked here).
    pub fn gaussian_mv(
        &mut self,
        out: &mut [f64],
        dimen: usize,
        storage: MatrixStorage,
        mean: &[f64],
        chol: &[f64],
        method: GaussianMethod,
    ) -> Result<()> {
        let (n, d, m) = mv_check(
            "gaussian_mv",
            out.len(),
            dimen,
            storage,
            mean.len(),
            chol.len(),
        )?;
        let method = gaussian_mv_method(method);
        // SAFETY: out holds n*dimen doubles, mean dimen, chol the storage's count (all checked above).
        check(unsafe {
            sys::vdRngGaussianMV(
                method,
                self.ptr,
                n,
                out.as_mut_ptr(),
                d,
                m,
                mean.as_ptr(),
                chol.as_ptr(),
            )
        })
    }

    /// `vsRngGaussianMV`.
    pub fn gaussian_mv_f32(
        &mut self,
        out: &mut [f32],
        dimen: usize,
        storage: MatrixStorage,
        mean: &[f32],
        chol: &[f32],
        method: GaussianMethod,
    ) -> Result<()> {
        let (n, d, m) = mv_check(
            "gaussian_mv_f32",
            out.len(),
            dimen,
            storage,
            mean.len(),
            chol.len(),
        )?;
        let method = gaussian_mv_method(method);
        // SAFETY: as `gaussian_mv`.
        check(unsafe {
            sys::vsRngGaussianMV(
                method,
                self.ptr,
                n,
                out.as_mut_ptr(),
                d,
                m,
                mean.as_ptr(),
                chol.as_ptr(),
            )
        })
    }
}

fn gaussian_mv_method(m: GaussianMethod) -> sys::openrng_int_t {
    (match m {
        GaussianMethod::BoxMuller => sys::VSL_RNG_METHOD_GAUSSIANMV_BOXMULLER,
        GaussianMethod::BoxMuller2 => sys::VSL_RNG_METHOD_GAUSSIANMV_BOXMULLER2,
        GaussianMethod::Icdf => sys::VSL_RNG_METHOD_GAUSSIANMV_ICDF,
    }) as sys::openrng_int_t
}

/// Validate the shapes of a multivariate Gaussian call; returns `(n vectors, dimen, storage id)` in the library's types.
fn mv_check(
    name: &str,
    out_len: usize,
    dimen: usize,
    storage: MatrixStorage,
    mean_len: usize,
    chol_len: usize,
) -> Result<(sys::openrng_int_t, sys::openrng_int_t, sys::openrng_int_t)> {
    if dimen == 0 || out_len % dimen != 0 {
        return Err(Error::InvalidArgument(format!(
            "{name}: out.len() {out_len} must be a nonzero multiple of dimen {dimen}"
        )));
    }
    if mean_len != dimen {
        return Err(Error::InvalidArgument(format!(
            "{name}: mean has {mean_len} elements, dimen is {dimen}"
        )));
    }
    let need = storage.len(dimen).ok_or_else(|| {
        Error::InvalidArgument(format!("{name}: matrix size overflows for dimen {dimen}"))
    })?;
    if chol_len != need {
        return Err(Error::InvalidArgument(format!(
            "{name}: {storage:?} storage of dimen {dimen} needs {need} matrix elements, got {chol_len}"
        )));
    }
    Ok((
        int("vectors", out_len / dimen)?,
        int("dimen", dimen)?,
        storage.id() as sys::openrng_int_t,
    ))
}

#[cfg(test)]
mod tests {
    //! Pure-Rust checks that touch no library function (the library tests are in `tests/`, run by the PM).
    use super::*;

    #[test]
    fn matrix_storage_element_counts() {
        assert_eq!(MatrixStorage::Full.len(3), Some(9));
        assert_eq!(MatrixStorage::Packed.len(3), Some(6));
        assert_eq!(MatrixStorage::Diagonal.len(3), Some(3));
        assert_eq!(MatrixStorage::Full.len(usize::MAX), None);
        assert_eq!(MatrixStorage::Packed.len(usize::MAX), None);
    }

    #[test]
    fn mv_check_rejects_every_wrong_shape() {
        let ok = mv_check("t", 6, 3, MatrixStorage::Packed, 3, 6);
        assert!(ok.is_ok(), "{ok:?}");
        for (out, d, st, m, c) in [
            (7, 3, MatrixStorage::Packed, 3, 6), // out not a multiple of dimen
            (6, 0, MatrixStorage::Packed, 0, 0), // dimen 0
            (6, 3, MatrixStorage::Packed, 2, 6), // mean too short
            (6, 3, MatrixStorage::Packed, 4, 6), // mean too long
            (6, 3, MatrixStorage::Packed, 3, 5), // matrix too short
            (6, 3, MatrixStorage::Full, 3, 6),   // matrix wrong for storage
            (6, 3, MatrixStorage::Diagonal, 3, 6),
        ] {
            assert!(
                matches!(
                    mv_check("t", out, d, st, m, c),
                    Err(Error::InvalidArgument(_))
                ),
                "{out} {d} {st:?} {m} {c}"
            );
        }
    }

    #[test]
    fn status_mapping() {
        assert!(check(sys::VSL_ERROR_OK).is_ok());
        assert!(matches!(
            check(sys::VSL_ERROR_MEM_FAILURE),
            Err(Error::AllocationFailed("openrng"))
        ));
        for code in [
            sys::VSL_ERROR_BADARGS,
            sys::VSL_RNG_ERROR_BAD_STREAM,
            -0x2000,
            7,
        ] {
            match check(code) {
                Err(Error::Status {
                    component: "openrng",
                    code: c,
                    ..
                }) => assert_eq!(c, i64::from(code)),
                other => panic!("{code}: {other:?}"),
            }
        }
        assert_eq!(describe(sys::VSL_ERROR_BADARGS), "bad arguments");
    }

    #[test]
    fn brng_ids_round_trip_and_are_distinct() {
        for b in Brng::ALL {
            assert_eq!(Brng::from_id(b.id()), Some(b));
        }
        let mut ids: Vec<i32> = Brng::ALL.iter().map(|b| b.id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), Brng::ALL.len());
        assert_eq!(Brng::from_id(sys::VSL_BRNG_NONDETERM), None);
    }

    #[test]
    fn int_refuses_what_does_not_fit() {
        assert!(int("t", 4).is_ok());
        if core::mem::size_of::<sys::openrng_int_t>() == 4 {
            assert!(matches!(
                int("t", (i32::MAX as usize) + 1),
                Err(Error::InvalidArgument(_))
            ));
        }
    }
}
