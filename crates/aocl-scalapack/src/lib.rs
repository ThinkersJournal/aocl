//! Safe Rust wrappers for AOCL-ScaLAPACK.
//!
//! Provides a [`Grid`] RAII wrapper around the BLACS process grid, a
//! [`BlockCyclic`] distributed-matrix descriptor, and safe wrappers
//! around the most-used ScaLAPACK routines: `pgesv` (distributed
//! linear solve), `pgemm` (distributed matrix-matrix multiply), and
//! `pgetrf` / `ppotrf` factorizations.
//!
//! # Runtime
//!
//! Any program calling the distributed routines or building a [`Grid`] must
//! be launched under an MPI runtime (on Windows the AOCL 5.3.x `scalapack.dll` imports Intel MPI
//! `impi.dll`, checked with `dumpbin /dependents`; MPICH / OpenMPI on Linux). MPI is **not**
//! a Cargo dependency of this crate — link against your platform's MPI
//! library at the binary level if you need additional MPI symbols.
//! [`numroc`] needs no MPI launcher or BLACS grid, but the library's DLLs
//! (including `impi.dll` on Windows) must still be loadable.
//!
//! Tests: `numroc` needs no MPI. `tests/pdgesv_mpi.rs` (its own `main`, `harness = false`) builds a [`Grid`] and
//! [`BlockCyclic`] descriptors and solves an asymmetric 6x6 system with [`pdgesv`], checks [`pdgemm`] (`Trans::No` and `Trans::T`) against a serial product, and runs [`pdgetrf`] + [`pdgetrs`] (both `Trans`) and [`pdpotrf`] + [`pdpotrs`] (both `Uplo`), [`pdsyev`] (both `Uplo`) and [`pdgels`], on however many ranks launched it:
//! one rank under `cargo test`, 1, 2 and 3 under `mpiexec` on the self-hosted runner. `tests/psgesv_mpi.rs` repeats it for the single-precision `ps*` routines, `tests/complex_mpi.rs` runs `pcgesv` and
//! `pzgesv`, and `tests/grid2d_mpi.rs` uses true 2D grids (up to 2x3), three right-hand sides and a non-zero grid
//! origin. There are no `#[ignore]`d placeholders. Not exercised: block sizes other than 2, matrix sizes other than 6 and
//! 8 and sub-matrix offsets (`ia`, `ja` other than 1). All integers are [`Int`] (`i32`, or `i64` with the `ilp64` feature); the tests
//! above pass with both.

#![allow(
    clippy::too_many_arguments,
    reason = "wrappers mirror the C API's argument lists"
)]
#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use aocl_error::{Error, Result};
use aocl_scalapack_sys as sys;
pub use aocl_types::{Layout, Trans, Uplo};

use std::os::raw::c_char;

pub use sys::Int;

fn map_info(component: &'static str, info: Int) -> Result<()> {
    use std::cmp::Ordering;
    match info.cmp(&0) {
        Ordering::Equal => Ok(()),
        Ordering::Less => Err(Error::Status {
            component,
            code: info as i64,
            message: format!("argument {} had an illegal value", -info),
        }),
        Ordering::Greater => Err(Error::Status {
            component,
            code: info as i64,
            message: format!("ScaLAPACK reported singular factor at index {info}"),
        }),
    }
}

fn trans_char(t: Trans) -> c_char {
    match t {
        Trans::No => b'N' as c_char,
        Trans::T => b'T' as c_char,
        Trans::C => b'C' as c_char,
    }
}

fn uplo_char(u: Uplo) -> c_char {
    match u {
        Uplo::Upper => b'U' as c_char,
        Uplo::Lower => b'L' as c_char,
    }
}

// =========================================================================
//   BLACS grid
// =========================================================================

/// RAII wrapper around a BLACS process grid context.
pub struct Grid {
    context: Int,
    nprow: Int,
    npcol: Int,
    myprow: Int,
    mypcol: Int,
    /// True after `Drop` (or `exit`) has been called; suppresses double-free.
    released: bool,
}

unsafe impl Send for Grid {}

impl Grid {
    /// Initialise BLACS (if not already initialised) and create a grid
    /// of size `nprow × npcol` with row-major rank layout.
    ///
    /// Must be called from inside an MPI program (typically launched
    /// via `mpiexec`).
    pub fn new(nprow: usize, npcol: usize) -> Result<Self> {
        let mut mypnum: Int = -1;
        let mut nprocs: Int = 0;
        unsafe { sys::Cblacs_pinfo(&mut mypnum, &mut nprocs) };
        if nprocs <= 0 {
            return Err(Error::Status {
                component: "scalapack",
                code: nprocs as i64,
                message: "BLACS reported zero processes — is MPI initialised?".into(),
            });
        }
        let needed = nprow * npcol;
        if (nprocs as usize) < needed {
            return Err(Error::InvalidArgument(format!(
                "Grid::new: nprow*npcol = {needed} > MPI world size {nprocs}"
            )));
        }
        let mut ctxt: Int = 0;
        unsafe { sys::Cblacs_get(-1, 0, &mut ctxt) };
        let order = c"R".as_ptr();
        unsafe { sys::Cblacs_gridinit(&mut ctxt, order, nprow as Int, npcol as Int) };

        let mut g_nprow = 0;
        let mut g_npcol = 0;
        let mut myprow = -1;
        let mut mypcol = -1;
        unsafe { sys::Cblacs_gridinfo(ctxt, &mut g_nprow, &mut g_npcol, &mut myprow, &mut mypcol) };
        Ok(Self {
            context: ctxt,
            nprow: g_nprow,
            npcol: g_npcol,
            myprow,
            mypcol,
            released: false,
        })
    }

    /// `(nprow, npcol)`.
    pub fn shape(&self) -> (usize, usize) {
        (self.nprow as usize, self.npcol as usize)
    }

    /// `(myprow, mypcol)`. Returns `(usize::MAX, usize::MAX)` for
    /// processes that aren't part of the grid (per BLACS convention,
    /// out-of-grid coords are `-1`).
    pub fn coords(&self) -> (usize, usize) {
        let to_usize = |c: Int| if c < 0 { usize::MAX } else { c as usize };
        (to_usize(self.myprow), to_usize(self.mypcol))
    }

    /// Raw BLACS context value, for routines this crate doesn't yet wrap.
    pub fn context(&self) -> Int {
        self.context
    }

    /// Synchronise across the grid (`scope = b'A'` covers the whole grid;
    /// `b'R'` row only; `b'C'` column only).
    pub fn barrier(&self, scope: u8) {
        let s = [scope as c_char, 0];
        unsafe { sys::Cblacs_barrier(self.context, s.as_ptr()) };
    }
}

impl Drop for Grid {
    fn drop(&mut self) {
        if !self.released {
            unsafe { sys::Cblacs_gridexit(self.context) };
            self.released = true;
        }
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Grid")
            .field("nprow", &self.nprow)
            .field("npcol", &self.npcol)
            .field("myprow", &self.myprow)
            .field("mypcol", &self.mypcol)
            .finish_non_exhaustive()
    }
}

/// Tear down BLACS at program exit. Call exactly once after the last
/// `Grid` is dropped.
pub fn blacs_exit(continue_after_mpi: bool) {
    unsafe { sys::Cblacs_exit(if continue_after_mpi { 1 } else { 0 }) };
}

// =========================================================================
//   Block-cyclic descriptor
// =========================================================================

/// 9-element ScaLAPACK descriptor for a block-cyclically distributed
/// matrix.
#[derive(Debug, Clone)]
pub struct BlockCyclic {
    desc: [Int; 9],
}

impl BlockCyclic {
    /// Build a descriptor for an `m × n` global matrix block-distributed
    /// in `mb × nb` blocks across `grid`, with root process at
    /// `(rsrc, csrc)` and local leading dimension `lld`.
    pub fn new(
        grid: &Grid,
        m: usize,
        n: usize,
        mb: usize,
        nb: usize,
        rsrc: usize,
        csrc: usize,
        lld: usize,
    ) -> Result<Self> {
        let mut desc = [0 as Int; 9];
        let mut info: Int = 0;
        let m_i = m as Int;
        let n_i = n as Int;
        let mb_i = mb as Int;
        let nb_i = nb as Int;
        let rsrc_i = rsrc as Int;
        let csrc_i = csrc as Int;
        let ctxt = grid.context;
        let lld_i = lld as Int;
        unsafe {
            sys::descinit_(
                desc.as_mut_ptr(),
                &m_i,
                &n_i,
                &mb_i,
                &nb_i,
                &rsrc_i,
                &csrc_i,
                &ctxt,
                &lld_i,
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "scalapack",
                code: info as i64,
                message: format!("descinit_ returned info={info}"),
            });
        }
        Ok(Self { desc })
    }

    /// Borrow the raw 9-element descriptor.
    pub fn as_raw(&self) -> &[Int; 9] {
        &self.desc
    }
}

/// Compute the local row count for an `n`-row global array partitioned
/// into `nb`-row blocks across `nprocs` grid rows, with this process at
/// row coordinate `iproc` and root at `isrcproc`.
pub fn numroc(n: usize, nb: usize, iproc: usize, isrcproc: usize, nprocs: usize) -> usize {
    let n_i = n as Int;
    let nb_i = nb as Int;
    let iproc_i = iproc as Int;
    let isrcproc_i = isrcproc as Int;
    let nprocs_i = nprocs as Int;
    let r = unsafe { sys::numroc_(&n_i, &nb_i, &iproc_i, &isrcproc_i, &nprocs_i) };
    if r < 0 {
        0
    } else {
        r as usize
    }
}

// =========================================================================
//   Argument validation (the library trusts every pointer: a slice shorter than the descriptor's local extent is an out-of-bounds access)
// =========================================================================

/// Local rows and columns of the matrix `desc` describes on THIS process (0 x 0 off the grid).
fn local_dims(desc: &BlockCyclic) -> (usize, usize) {
    let d = desc.as_raw();
    let (mut nprow, mut npcol, mut myrow, mut mycol): (Int, Int, Int, Int) = (0, 0, -1, -1);
    unsafe { sys::Cblacs_gridinfo(d[1], &mut nprow, &mut npcol, &mut myrow, &mut mycol) };
    if myrow < 0
        || mycol < 0
        || nprow <= 0
        || npcol <= 0
        || d[2] < 0
        || d[3] < 0
        || d[4] <= 0
        || d[5] <= 0
    {
        return (0, 0);
    }
    (
        numroc(
            d[2] as usize,
            d[4] as usize,
            myrow as usize,
            d[6].max(0) as usize,
            nprow as usize,
        ),
        numroc(
            d[3] as usize,
            d[5] as usize,
            mycol as usize,
            d[7].max(0) as usize,
            npcol as usize,
        ),
    )
}

/// The `rows x cols` submatrix at (`ia`, `ja`) must lie inside the global matrix `desc` describes, and `len` must cover this process's
/// local part of it: `(local_cols - 1) * lld + local_rows` elements. (Tests: `tests/guard_lengths.rs`.)
fn check_matrix(
    what: &str,
    len: usize,
    ia: Int,
    ja: Int,
    rows: usize,
    cols: usize,
    desc: &BlockCyclic,
) -> Result<()> {
    let d = desc.as_raw();
    if ia < 1
        || ja < 1
        || (ia as i64 - 1) + rows as i64 > d[2] as i64
        || (ja as i64 - 1) + cols as i64 > d[3] as i64
    {
        return Err(Error::InvalidArgument(format!(
            "{what}: a {rows} x {cols} block at ({ia}, {ja}) does not lie inside the {} x {} matrix its descriptor describes",
            d[2], d[3]
        )));
    }
    let (lr, lc) = local_dims(desc);
    let need = if lr == 0 || lc == 0 {
        0
    } else {
        (lc - 1) * d[8].max(0) as usize + lr
    };
    if len < need {
        return Err(Error::InvalidArgument(format!(
            "{what}: {len} elements, the descriptor needs {need} locally"
        )));
    }
    Ok(())
}

fn check_len(what: &str, len: usize, need: usize) -> Result<()> {
    if len < need {
        return Err(Error::InvalidArgument(format!(
            "{what}: {len} elements, {need} are needed"
        )));
    }
    Ok(())
}

/// ipiv of an LU / solve: LOCr(M_A) + MB_A entries.
fn check_ipiv(what: &str, len: usize, desc: &BlockCyclic) -> Result<()> {
    check_len(
        what,
        len,
        local_dims(desc).0 + desc.as_raw()[4].max(0) as usize,
    )
}

/// `lwork` is the workspace size the library will use: it must fit in `work` (`-1` asks for the optimal size, written to `work[0]`).
fn check_work(what: &str, len: usize, lwork: Int) -> Result<()> {
    if lwork < -1 {
        return Err(Error::InvalidArgument(format!(
            "{what}: lwork {lwork} is invalid"
        )));
    }
    check_len(what, len, if lwork == -1 { 1 } else { lwork as usize })
}

// =========================================================================
//   Distributed solve / GEMM / factorization
// =========================================================================

/// Distributed double-precision general solve `A · X = B`.
/// All matrices use 1-based row/column origins per Fortran convention
/// (`ia`, `ja`, `ib`, `jb` are typically 1).
pub fn pdgesv(
    n: usize,
    nrhs: usize,
    a: &mut [f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &mut [Int],
    b: &mut [f64],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("pdgesv: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("pdgesv: B", b.len(), ib, jb, n, nrhs, descb)?;
    check_ipiv("pdgesv: ipiv", ipiv.len(), desca)?;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdgesv_(
            &n_i,
            &nrhs_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_mut_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Distributed double-precision `C := α · op(A) · op(B) + β · C`.
pub fn pdgemm(
    trans_a: Trans,
    trans_b: Trans,
    m: usize,
    n: usize,
    k: usize,
    alpha: f64,
    a: &[f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    b: &[f64],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
    beta: f64,
    c: &mut [f64],
    ic: Int,
    jc: Int,
    descc: &BlockCyclic,
) -> Result<()> {
    let (ar, ac) = if trans_a == Trans::No { (m, k) } else { (k, m) };
    let (br, bc) = if trans_b == Trans::No { (k, n) } else { (n, k) };
    check_matrix("pdgemm: A", a.len(), ia, ja, ar, ac, desca)?;
    check_matrix("pdgemm: B", b.len(), ib, jb, br, bc, descb)?;
    check_matrix("pdgemm: C", c.len(), ic, jc, m, n, descc)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let k_i = k as Int;
    let ta = trans_char(trans_a);
    let tb = trans_char(trans_b);
    unsafe {
        sys::pdgemm_(
            &ta,
            &tb,
            &m_i,
            &n_i,
            &k_i,
            &alpha,
            a.as_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            b.as_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &beta,
            c.as_mut_ptr(),
            &ic,
            &jc,
            descc.as_raw().as_ptr(),
        );
    }
    Ok(())
}

/// Distributed double-precision Cholesky factorization.
pub fn pdpotrf(
    uplo: Uplo,
    n: usize,
    a: &mut [f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
) -> Result<()> {
    check_matrix("pdpotrf: A", a.len(), ia, ja, n, n, desca)?;
    let n_i = n as Int;
    let u = uplo_char(uplo);
    let mut info: Int = 0;
    unsafe {
        sys::pdpotrf_(
            &u,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Distributed double-precision LU factorization with partial pivoting.
pub fn pdgetrf(
    m: usize,
    n: usize,
    a: &mut [f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &mut [Int],
) -> Result<()> {
    check_matrix("pdgetrf: A", a.len(), ia, ja, m, n, desca)?;
    check_ipiv("pdgetrf: ipiv", ipiv.len(), desca)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdgetrf_(
            &m_i,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_mut_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Distributed back-substitution after [`pdgetrf`].
pub fn pdgetrs(
    trans: Trans,
    n: usize,
    nrhs: usize,
    a: &[f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &[Int],
    b: &mut [f64],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("pdgetrs: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("pdgetrs: B", b.len(), ib, jb, n, nrhs, descb)?;
    check_ipiv("pdgetrs: ipiv", ipiv.len(), desca)?;
    let t = trans_char(trans);
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdgetrs_(
            &t,
            &n_i,
            &nrhs_i,
            a.as_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Distributed back-substitution after [`pdpotrf`].
pub fn pdpotrs(
    uplo: Uplo,
    n: usize,
    nrhs: usize,
    a: &[f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    b: &mut [f64],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("pdpotrs: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("pdpotrs: B", b.len(), ib, jb, n, nrhs, descb)?;
    let u = uplo_char(uplo);
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdpotrs_(
            &u,
            &n_i,
            &nrhs_i,
            a.as_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Whether `pdsyev` should compute eigenvectors as well as eigenvalues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EigCompute {
    ValuesOnly,
    ValuesAndVectors,
}

impl EigCompute {
    fn job_char(self) -> c_char {
        match self {
            EigCompute::ValuesOnly => b'N' as c_char,
            EigCompute::ValuesAndVectors => b'V' as c_char,
        }
    }
}

/// Distributed real symmetric eigendecomposition. Provide `work` of
/// size `lwork`; pass `lwork = -1` for a workspace-size query (the
/// optimum is written into `work[0]`).
pub fn pdsyev(
    compute: EigCompute,
    uplo: Uplo,
    n: usize,
    a: &mut [f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    w: &mut [f64],
    z: &mut [f64],
    iz: Int,
    jz: Int,
    descz: &BlockCyclic,
    work: &mut [f64],
    lwork: Int,
) -> Result<()> {
    check_matrix("pdsyev: A", a.len(), ia, ja, n, n, desca)?;
    check_len("pdsyev: w", w.len(), n)?;
    if compute == EigCompute::ValuesAndVectors {
        check_matrix("pdsyev: Z", z.len(), iz, jz, n, n, descz)?;
    }
    check_work("pdsyev: work", work.len(), lwork)?;
    let j = compute.job_char();
    let u = uplo_char(uplo);
    let n_i = n as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdsyev_(
            &j,
            &u,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            w.as_mut_ptr(),
            z.as_mut_ptr(),
            &iz,
            &jz,
            descz.as_raw().as_ptr(),
            work.as_mut_ptr(),
            &lwork,
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Distributed least squares `min || A·X − B ||₂`. `trans = Trans::No`
/// for the standard problem; `Trans::T` for the transposed.
pub fn pdgels(
    trans: Trans,
    m: usize,
    n: usize,
    nrhs: usize,
    a: &mut [f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    b: &mut [f64],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
    work: &mut [f64],
    lwork: Int,
) -> Result<()> {
    check_matrix("pdgels: A", a.len(), ia, ja, m, n, desca)?;
    check_matrix("pdgels: B", b.len(), ib, jb, m.max(n), nrhs, descb)?;
    check_work("pdgels: work", work.len(), lwork)?;
    let t = trans_char(trans);
    let m_i = m as Int;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdgels_(
            &t,
            &m_i,
            &n_i,
            &nrhs_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            work.as_mut_ptr(),
            &lwork,
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Distributed QR factorization.
pub fn pdgeqrf(
    m: usize,
    n: usize,
    a: &mut [f64],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    tau: &mut [f64],
    work: &mut [f64],
    lwork: Int,
) -> Result<()> {
    check_matrix("pdgeqrf: A", a.len(), ia, ja, m, n, desca)?;
    check_len("pdgeqrf: tau", tau.len(), local_dims(desca).1)?;
    check_work("pdgeqrf: work", work.len(), lwork)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pdgeqrf_(
            &m_i,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            tau.as_mut_ptr(),
            work.as_mut_ptr(),
            &lwork,
            &mut info,
        );
    }
    map_info("scalapack", info)
}

// ===========================================================================
//   Single-precision real (ps*) wrappers
// ===========================================================================

/// Single-precision distributed `A·X = B` solve. See [`pdgesv`].
pub fn psgesv(
    n: usize,
    nrhs: usize,
    a: &mut [f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &mut [Int],
    b: &mut [f32],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("psgesv: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("psgesv: B", b.len(), ib, jb, n, nrhs, descb)?;
    check_ipiv("psgesv: ipiv", ipiv.len(), desca)?;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::psgesv_(
            &n_i,
            &nrhs_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_mut_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision distributed `C := α · op(A) · op(B) + β · C`. See [`pdgemm`].
pub fn psgemm(
    trans_a: Trans,
    trans_b: Trans,
    m: usize,
    n: usize,
    k: usize,
    alpha: f32,
    a: &[f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    b: &[f32],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
    beta: f32,
    c: &mut [f32],
    ic: Int,
    jc: Int,
    descc: &BlockCyclic,
) -> Result<()> {
    let (ar, ac) = if trans_a == Trans::No { (m, k) } else { (k, m) };
    let (br, bc) = if trans_b == Trans::No { (k, n) } else { (n, k) };
    check_matrix("psgemm: A", a.len(), ia, ja, ar, ac, desca)?;
    check_matrix("psgemm: B", b.len(), ib, jb, br, bc, descb)?;
    check_matrix("psgemm: C", c.len(), ic, jc, m, n, descc)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let k_i = k as Int;
    let ta = trans_char(trans_a);
    let tb = trans_char(trans_b);
    unsafe {
        sys::psgemm_(
            &ta,
            &tb,
            &m_i,
            &n_i,
            &k_i,
            &alpha,
            a.as_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            b.as_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &beta,
            c.as_mut_ptr(),
            &ic,
            &jc,
            descc.as_raw().as_ptr(),
        );
    }
    Ok(())
}

/// Single-precision distributed Cholesky factorisation. See [`pdpotrf`].
pub fn pspotrf(
    uplo: Uplo,
    n: usize,
    a: &mut [f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
) -> Result<()> {
    check_matrix("pspotrf: A", a.len(), ia, ja, n, n, desca)?;
    let n_i = n as Int;
    let u = uplo_char(uplo);
    let mut info: Int = 0;
    unsafe {
        sys::pspotrf_(
            &u,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision distributed LU factorisation. See [`pdgetrf`].
pub fn psgetrf(
    m: usize,
    n: usize,
    a: &mut [f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &mut [Int],
) -> Result<()> {
    check_matrix("psgetrf: A", a.len(), ia, ja, m, n, desca)?;
    check_ipiv("psgetrf: ipiv", ipiv.len(), desca)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let mut info: Int = 0;
    unsafe {
        sys::psgetrf_(
            &m_i,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_mut_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision distributed back-substitution after [`psgetrf`]. See
/// [`pdgetrs`].
pub fn psgetrs(
    trans: Trans,
    n: usize,
    nrhs: usize,
    a: &[f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &[Int],
    b: &mut [f32],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("psgetrs: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("psgetrs: B", b.len(), ib, jb, n, nrhs, descb)?;
    check_ipiv("psgetrs: ipiv", ipiv.len(), desca)?;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let t = trans_char(trans);
    let mut info: Int = 0;
    unsafe {
        sys::psgetrs_(
            &t,
            &n_i,
            &nrhs_i,
            a.as_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision Cholesky back-substitution. See [`pdpotrs`].
pub fn pspotrs(
    uplo: Uplo,
    n: usize,
    nrhs: usize,
    a: &[f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    b: &mut [f32],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("pspotrs: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("pspotrs: B", b.len(), ib, jb, n, nrhs, descb)?;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let u = uplo_char(uplo);
    let mut info: Int = 0;
    unsafe {
        sys::pspotrs_(
            &u,
            &n_i,
            &nrhs_i,
            a.as_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision symmetric eigendecomposition. See [`pdsyev`].
pub fn pssyev(
    compute: EigCompute,
    uplo: Uplo,
    n: usize,
    a: &mut [f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    w: &mut [f32],
    z: &mut [f32],
    iz: Int,
    jz: Int,
    descz: &BlockCyclic,
    work: &mut [f32],
    lwork: Int,
) -> Result<()> {
    check_matrix("pssyev: A", a.len(), ia, ja, n, n, desca)?;
    check_len("pssyev: w", w.len(), n)?;
    if compute == EigCompute::ValuesAndVectors {
        check_matrix("pssyev: Z", z.len(), iz, jz, n, n, descz)?;
    }
    check_work("pssyev: work", work.len(), lwork)?;
    let n_i = n as Int;
    let jobz = match compute {
        EigCompute::ValuesOnly => b'N' as c_char,
        EigCompute::ValuesAndVectors => b'V' as c_char,
    };
    let u = uplo_char(uplo);
    let lwork = lwork as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pssyev_(
            &jobz,
            &u,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            w.as_mut_ptr(),
            z.as_mut_ptr(),
            &iz,
            &jz,
            descz.as_raw().as_ptr(),
            work.as_mut_ptr(),
            &lwork,
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision distributed least-squares solve. See [`pdgels`].
pub fn psgels(
    trans: Trans,
    m: usize,
    n: usize,
    nrhs: usize,
    a: &mut [f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    b: &mut [f32],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
    work: &mut [f32],
    lwork: Int,
) -> Result<()> {
    check_matrix("psgels: A", a.len(), ia, ja, m, n, desca)?;
    check_matrix("psgels: B", b.len(), ib, jb, m.max(n), nrhs, descb)?;
    check_work("psgels: work", work.len(), lwork)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let t = trans_char(trans);
    let lwork = lwork as Int;
    let mut info: Int = 0;
    unsafe {
        sys::psgels_(
            &t,
            &m_i,
            &n_i,
            &nrhs_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            work.as_mut_ptr(),
            &lwork,
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Single-precision distributed QR. See [`pdgeqrf`].
pub fn psgeqrf(
    m: usize,
    n: usize,
    a: &mut [f32],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    tau: &mut [f32],
    work: &mut [f32],
    lwork: Int,
) -> Result<()> {
    check_matrix("psgeqrf: A", a.len(), ia, ja, m, n, desca)?;
    check_len("psgeqrf: tau", tau.len(), local_dims(desca).1)?;
    check_work("psgeqrf: work", work.len(), lwork)?;
    let m_i = m as Int;
    let n_i = n as Int;
    let lwork = lwork as Int;
    let mut info: Int = 0;
    unsafe {
        sys::psgeqrf_(
            &m_i,
            &n_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            tau.as_mut_ptr(),
            work.as_mut_ptr(),
            &lwork,
            &mut info,
        );
    }
    map_info("scalapack", info)
}

// ===========================================================================
//   Complex distributed solves: pcgesv (c32) and pzgesv (c64)
// ===========================================================================

/// Single-precision complex distributed solve `A·X = B`. Elements are
/// passed as `[f32; 2]` `(re, im)` pairs (ABI-compatible with
/// `Complex32`).
pub fn pcgesv(
    n: usize,
    nrhs: usize,
    a: &mut [[f32; 2]],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &mut [Int],
    b: &mut [[f32; 2]],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("pcgesv: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("pcgesv: B", b.len(), ib, jb, n, nrhs, descb)?;
    check_ipiv("pcgesv: ipiv", ipiv.len(), desca)?;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pcgesv_(
            &n_i,
            &nrhs_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_mut_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

/// Double-precision complex distributed solve `A·X = B`. Elements are
/// passed as `[f64; 2]` `(re, im)` pairs (ABI-compatible with
/// `Complex64`).
pub fn pzgesv(
    n: usize,
    nrhs: usize,
    a: &mut [[f64; 2]],
    ia: Int,
    ja: Int,
    desca: &BlockCyclic,
    ipiv: &mut [Int],
    b: &mut [[f64; 2]],
    ib: Int,
    jb: Int,
    descb: &BlockCyclic,
) -> Result<()> {
    check_matrix("pzgesv: A", a.len(), ia, ja, n, n, desca)?;
    check_matrix("pzgesv: B", b.len(), ib, jb, n, nrhs, descb)?;
    check_ipiv("pzgesv: ipiv", ipiv.len(), desca)?;
    let n_i = n as Int;
    let nrhs_i = nrhs as Int;
    let mut info: Int = 0;
    unsafe {
        sys::pzgesv_(
            &n_i,
            &nrhs_i,
            a.as_mut_ptr(),
            &ia,
            &ja,
            desca.as_raw().as_ptr(),
            ipiv.as_mut_ptr(),
            b.as_mut_ptr(),
            &ib,
            &jb,
            descb.as_raw().as_ptr(),
            &mut info,
        );
    }
    map_info("scalapack", info)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `numroc` is a pure tool routine (no BLACS grid, no MPI), so it runs
    /// anywhere the library links. Expected values follow from the
    /// block-cyclic definition: 10 rows in blocks of 2 over 3 process rows
    /// deal blocks 0,3 / 1,4 / 2 to rows 0 / 1 / 2. The values are worked
    /// by hand from the definition, cross-checked against nothing external
    /// except this test passing against AOCL 5.3.x's `scalapack.dll` in a
    /// single process (2026-11-19, no `mpiexec`); the multi-rank run is `tests/pdgesv_mpi.rs`.
    /// been done.
    #[test]
    fn numroc_block_cyclic_split() {
        let got: Vec<usize> = (0..3).map(|p| numroc(10, 2, p, 0, 3)).collect();
        assert_eq!(got, [4, 4, 2]);
        // Every global row is owned exactly once.
        assert_eq!(got.iter().sum::<usize>(), 10);
        // Source process 1 takes block 0, so owners rotate: rows 1,2,0.
        let rot: Vec<usize> = (0..3).map(|p| numroc(10, 2, p, 1, 3)).collect();
        assert_eq!(rot, [2, 4, 4]);
        // One process owns everything; an empty array owns nothing.
        assert_eq!(numroc(7, 3, 0, 0, 1), 7);
        assert_eq!(numroc(0, 2, 0, 0, 3), 0);
    }
}
