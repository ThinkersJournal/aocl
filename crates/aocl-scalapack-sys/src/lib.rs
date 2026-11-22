//! Hand-written FFI for AOCL-ScaLAPACK.
//!
//! AOCL ships no public C headers for ScaLAPACK; the declarations below
//! follow standard ScaLAPACK / BLACS conventions:
//!
//! - **BLACS C-API entry points** (`Cblacs_*`) take and return values via
//!   pointers but are exposed under their cdecl names with no trailing
//!   underscore.
//! - **ScaLAPACK Fortran routines** (`pdgesv`, `pdgemm`, etc.) follow the
//!   Fortran calling convention: trailing underscore in the symbol name,
//!   every argument by pointer, character arguments as `*const c_char`,
//!   integer arguments as `*const i32` (LP64) or `*const i64` (ILP64,
//!   not modelled here).
//! - **Distributed-matrix descriptors** are `[i32; 9]` arrays, populated
//!   by `descinit_`.
//!
//! At runtime the host program must be launched under an MPI runtime
//! (Windows: the AOCL 5.3.x `scalapack.dll` imports Intel MPI `impi.dll`, checked with
//! `dumpbin /dependents`; Linux: MPICH / OpenMPI). MPI is **not** declared as a Cargo
//! dependency — link with the MPI library on your platform via your own
//! build script if you need additional MPI symbols.
//!
//! The signatures here cover the most-used distributed solve / BLAS-3
//! entry points across all four precisions; extend as needed.

#![allow(
    non_snake_case,
    non_camel_case_types,
    dead_code,
    clippy::all,
    reason = "bindgen output mirrors the C headers: names, unused items and C-only types are not ours to change"
)]

use std::os::raw::c_char;

/// ScaLAPACK's integer: 32-bit under LP64, 64-bit with the `ilp64` feature (every integer argument, descriptor entry
/// and pivot). Under LP64 it is `i32`, so nothing changes there.
#[cfg(not(feature = "ilp64"))]
pub type Int = i32;
/// ScaLAPACK's integer: 32-bit under LP64, 64-bit with the `ilp64` feature.
#[cfg(feature = "ilp64")]
pub type Int = i64;

// =========================================================================
//   BLACS lifecycle (C-API: prefixed `Cblacs_`, no trailing underscore)
// =========================================================================

extern "C" {
    /// Get this process's BLACS rank and total process count.
    /// Output: `*mypnum` and `*nprocs` are filled. If BLACS hasn't been
    /// initialised, on entry `*mypnum` should be -1; BLACS will
    /// initialise as a side effect.
    pub fn Cblacs_pinfo(mypnum: *mut Int, nprocs: *mut Int);

    /// Get a context value for the default system. `what = 0` returns
    /// the default system context in `*val`.
    pub fn Cblacs_get(icontxt: Int, what: Int, val: *mut Int);

    /// Initialise an `nprow × npcol` BLACS process grid in `*context`.
    /// `order` is `b'R'` (row-major) or `b'C'` (column-major) for how
    /// MPI ranks are laid out across the grid.
    pub fn Cblacs_gridinit(context: *mut Int, order: *const c_char, nprow: Int, npcol: Int);

    /// Read back the grid shape and this process's coordinates.
    pub fn Cblacs_gridinfo(
        context: Int,
        nprow: *mut Int,
        npcol: *mut Int,
        myprow: *mut Int,
        mypcol: *mut Int,
    );

    /// Release the grid context.
    pub fn Cblacs_gridexit(context: Int);

    /// Tear down BLACS. Pass `continue_after = 0` for a normal shutdown.
    pub fn Cblacs_exit(continue_after: Int);

    /// Synchronization barrier across the grid. `scope` is `b'A'` (all),
    /// `b'R'` (row), or `b'C'` (column).
    pub fn Cblacs_barrier(context: Int, scope: *const c_char);
}

// =========================================================================
//   Distributed-matrix descriptor (Fortran)
// =========================================================================

// Initialise a 9-element ScaLAPACK descriptor `desc` describing an
// `m × n` block-cyclically distributed matrix with block size
// `mb × nb`, root process at `(rsrc, csrc)`, attached to the BLACS
// context `ctxt`, with local leading dimension `lld`. On exit `info`
// is non-zero on error.
extern "C" {
    pub fn descinit_(
        desc: *mut Int,
        m: *const Int,
        n: *const Int,
        mb: *const Int,
        nb: *const Int,
        rsrc: *const Int,
        csrc: *const Int,
        ictxt: *const Int,
        lld: *const Int,
        info: *mut Int,
    );
}

// Compute the local row count for a block-cyclically distributed array
// with `n` global rows split into `nb`-row blocks across `nprocs`
// processes, with this process's row coordinate `iproc` and root at
// `isrcproc`.
extern "C" {
    pub fn numroc_(
        n: *const Int,
        nb: *const Int,
        iproc: *const Int,
        isrcproc: *const Int,
        nprocs: *const Int,
    ) -> Int;
}

// =========================================================================
//   ScaLAPACK Fortran routines (trailing underscore, all-by-pointer)
// =========================================================================

// Distributed general solve `A·X = B` for double-precision `A`.
// `n`, `nrhs` are global problem sizes. `a` and `b` are local
// fragments described by `desca` and `descb`. `ipiv` holds local
// pivots. On exit `info = 0` on success.
extern "C" {
    pub fn psgesv_(
        n: *const Int,
        nrhs: *const Int,
        a: *mut f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *mut Int,
        b: *mut f32,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );

    pub fn pdgesv_(
        n: *const Int,
        nrhs: *const Int,
        a: *mut f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *mut Int,
        b: *mut f64,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );

    /// Complex (single) variant. `a` and `b` are pointers to `[f32; 2]`
    /// pairs of `(re, im)`.
    pub fn pcgesv_(
        n: *const Int,
        nrhs: *const Int,
        a: *mut [f32; 2],
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *mut Int,
        b: *mut [f32; 2],
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );

    pub fn pzgesv_(
        n: *const Int,
        nrhs: *const Int,
        a: *mut [f64; 2],
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *mut Int,
        b: *mut [f64; 2],
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );
}

// Distributed `C := α · op(A) · op(B) + β · C`. `transa`, `transb` are
// 1-byte chars (`b'N'`, `b'T'`, `b'C'`).
extern "C" {
    #[cfg_attr(windows, link_name = "psgemm")]
    pub fn psgemm_(
        transa: *const c_char,
        transb: *const c_char,
        m: *const Int,
        n: *const Int,
        k: *const Int,
        alpha: *const f32,
        a: *const f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        b: *const f32,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        beta: *const f32,
        c: *mut f32,
        ic: *const Int,
        jc: *const Int,
        descc: *const Int,
    );

    #[cfg_attr(windows, link_name = "pdgemm")]
    pub fn pdgemm_(
        transa: *const c_char,
        transb: *const c_char,
        m: *const Int,
        n: *const Int,
        k: *const Int,
        alpha: *const f64,
        a: *const f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        b: *const f64,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        beta: *const f64,
        c: *mut f64,
        ic: *const Int,
        jc: *const Int,
        descc: *const Int,
    );
}

// Distributed Cholesky factorization (positive-definite).
extern "C" {
    pub fn pspotrf_(
        uplo: *const c_char,
        n: *const Int,
        a: *mut f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        info: *mut Int,
    );

    pub fn pdpotrf_(
        uplo: *const c_char,
        n: *const Int,
        a: *mut f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        info: *mut Int,
    );
}

// Distributed LU factorization with partial pivoting.
extern "C" {
    pub fn psgetrf_(
        m: *const Int,
        n: *const Int,
        a: *mut f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *mut Int,
        info: *mut Int,
    );

    pub fn pdgetrf_(
        m: *const Int,
        n: *const Int,
        a: *mut f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *mut Int,
        info: *mut Int,
    );
}

// Distributed real symmetric eigendecomposition. `jobz` is `b'N'`
// (eigenvalues only) or `b'V'` (also eigenvectors). `uplo` is `b'U'` or
// `b'L'`. `w` receives `n` real eigenvalues. `z` (when computed) is
// the eigenvector matrix with descriptor `descz`. `work` is workspace
// of size `lwork`; pass `lwork = -1` to query the optimal size which
// is returned in `work[0]`.
extern "C" {
    pub fn pssyev_(
        jobz: *const c_char,
        uplo: *const c_char,
        n: *const Int,
        a: *mut f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        w: *mut f32,
        z: *mut f32,
        iz: *const Int,
        jz: *const Int,
        descz: *const Int,
        work: *mut f32,
        lwork: *const Int,
        info: *mut Int,
    );

    pub fn pdsyev_(
        jobz: *const c_char,
        uplo: *const c_char,
        n: *const Int,
        a: *mut f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        w: *mut f64,
        z: *mut f64,
        iz: *const Int,
        jz: *const Int,
        descz: *const Int,
        work: *mut f64,
        lwork: *const Int,
        info: *mut Int,
    );
}

// Distributed least squares (`m × n` overdetermined or underdetermined
// system). `trans = b'N'` for `min || A·X − B ||₂`, `b'T'` for the
// transposed problem. `work` is workspace of size `lwork`; pass
// `lwork = -1` for size query.
extern "C" {
    pub fn psgels_(
        trans: *const c_char,
        m: *const Int,
        n: *const Int,
        nrhs: *const Int,
        a: *mut f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        b: *mut f32,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        work: *mut f32,
        lwork: *const Int,
        info: *mut Int,
    );

    pub fn pdgels_(
        trans: *const c_char,
        m: *const Int,
        n: *const Int,
        nrhs: *const Int,
        a: *mut f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        b: *mut f64,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        work: *mut f64,
        lwork: *const Int,
        info: *mut Int,
    );
}

// Distributed back-substitution after `pdgetrf` (LU).
extern "C" {
    pub fn psgetrs_(
        trans: *const c_char,
        n: *const Int,
        nrhs: *const Int,
        a: *const f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *const Int,
        b: *mut f32,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );

    pub fn pdgetrs_(
        trans: *const c_char,
        n: *const Int,
        nrhs: *const Int,
        a: *const f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        ipiv: *const Int,
        b: *mut f64,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );
}

// Distributed back-substitution after `pdpotrf` (Cholesky).
extern "C" {
    pub fn pspotrs_(
        uplo: *const c_char,
        n: *const Int,
        nrhs: *const Int,
        a: *const f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        b: *mut f32,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );

    pub fn pdpotrs_(
        uplo: *const c_char,
        n: *const Int,
        nrhs: *const Int,
        a: *const f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        b: *mut f64,
        ib: *const Int,
        jb: *const Int,
        descb: *const Int,
        info: *mut Int,
    );
}

// Distributed QR factorization. `tau` is local; `work` workspace.
extern "C" {
    pub fn psgeqrf_(
        m: *const Int,
        n: *const Int,
        a: *mut f32,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        tau: *mut f32,
        work: *mut f32,
        lwork: *const Int,
        info: *mut Int,
    );

    pub fn pdgeqrf_(
        m: *const Int,
        n: *const Int,
        a: *mut f64,
        ia: *const Int,
        ja: *const Int,
        desca: *const Int,
        tau: *mut f64,
        work: *mut f64,
        lwork: *const Int,
        info: *mut Int,
    );
}
