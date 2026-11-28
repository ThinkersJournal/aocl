//! RED-FIRST proof for the length rule of the dense-matrix arguments of `csrmm`, `spmmd`, `sp2md`, `trsm`, `syrkd`, `syprd` (every
//! precision, both layouts, both `op`): the library does not check that the dense `B`, `C`, `X` buffers are as large as `ld` and the
//! sparse operands' shapes require, so a SAFE call with a buffer one element short read or wrote past it. Each case puts the buffer at
//! the END of a committed page followed by a PAGE_NOACCESS page (access past it = access violation, 0xc0000005) and passes it ONE
//! ELEMENT SHORT of the tight size (`ld` = the minimum leading dimension). The control cases pass the exact size and must succeed, so
//! the rule is tight from both sides: a rule that rejected valid calls would fail a control, a rule too loose would fault a short case.
//! Windows only (VirtualAlloc); Linux unmeasured. Each case is its own test (`-- --exact <name>` shows a crash by name).
#![cfg(windows)]
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "the adapters in this file mirror the wrappers under test, whose signatures are long (11 arguments) and are written once each"
)]
use aocl_sparse::complex::{self as cx, ComplexSparseMatrix};
use aocl_sparse::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

/// Run `f` with a buffer of `n` elements that ENDS at a committed page, the next page being PAGE_NOACCESS.
fn at_guard<T: Copy, R>(n: usize, fill: T, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n * core::mem::size_of::<T>() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(
            VirtualProtect(
                (base as *mut u8).add(PAGE) as *mut c_void,
                PAGE,
                0x01,
                &mut old
            ),
            0
        );
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for i in 0..n {
            start.add(i).write(fill);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn refused<T: core::fmt::Debug>(what: &str, r: Result<T>) {
    assert!(matches!(r, Err(Error::InvalidArgument(_))), "{what}: {r:?}");
}

/// Minimum leading dimension and the element count of a tight `rows x cols` dense matrix (ld = minimum).
fn tight(order: Order, rows: usize, cols: usize) -> (usize, usize) {
    match order {
        Order::ColMajor => (rows, rows * cols),
        Order::RowMajor => (cols, rows * cols),
    }
}

fn descr(ty: MatType, fill: FillMode) -> MatDescr {
    let mut d = MatDescr::new().unwrap();
    d.set_type(ty).unwrap();
    d.set_fill_mode(fill).unwrap();
    d.set_diag_type(DiagType::NonUnit).unwrap();
    d
}

// A is 3 x 4 (rectangular, so op N and op T need different shapes); S is a square 4 x 4 lower-triangular matrix for trsm; B3 is
// 3 x 3 and B4 is 4 x 3 sparse operands for spmmd / sp2md.
const RECT_RP: [i32; 4] = [0, 2, 3, 5];
const RECT_CI: [i32; 5] = [0, 2, 1, 0, 3];
const SQ_RP: [i32; 5] = [0, 1, 3, 5, 7];
const SQ_CI: [i32; 7] = [0, 0, 1, 1, 2, 2, 3];
const B4_RP: [i32; 5] = [0, 1, 2, 3, 4]; // 4 x 3: rows 0..3 have one entry
const B4_CI: [i32; 4] = [0, 1, 2, 0];
const B3_RP: [i32; 4] = [0, 1, 2, 3]; // 3 x 3 diagonal
const B3_CI: [i32; 3] = [0, 1, 2];

macro_rules! family {
    ($m:ident, $T:ty, $Mat:ty, $symty:expr, $val:expr,
     csrmm = $csrmm:expr, spmmd = $spmmd:expr, sp2md = $sp2md:expr,
     trsm = $trsm:expr, trsm_ordered = $trsm_o:expr, tr = $tr:expr, trsm_order = $eff:expr, syrkd = $syrkd:expr, syprd = $syprd:expr) => {
        mod $m {
            use super::*;
            fn v(x: f64) -> $T {
                ($val)(x)
            }
            fn csr(m: usize, n: usize, rp: &[i32], ci: &[i32]) -> $Mat {
                let vals: Vec<$T> = (0..ci.len()).map(|i| v(2.0 + i as f64)).collect();
                <$Mat>::from_csr(IndexBase::Zero, m, n, rp, ci, &vals).unwrap()
            }
            fn rect() -> $Mat {
                csr(3, 4, &RECT_RP, &RECT_CI)
            }
            fn sq() -> $Mat {
                csr(4, 4, &SQ_RP, &SQ_CI)
            }
            fn b4() -> $Mat {
                csr(4, 3, &B4_RP, &B4_CI)
            }
            fn b3() -> $Mat {
                csr(3, 3, &B3_RP, &B3_CI)
            }
            const ORDERS: [Order; 2] = [Order::ColMajor, Order::RowMajor];
            const OPS: [Trans; 2] = [Trans::No, $tr];

            /// op(A) shape for A = 3 x 4.
            fn opshape(op: Trans) -> (usize, usize) {
                if op == Trans::No {
                    (3, 4)
                } else {
                    (4, 3)
                }
            }

            // ---------------- csrmm: C(ar x n) = alpha op(A) B(ac x n) + beta C ----------------
            fn csrmm_call(op: Trans, order: Order, bl: usize, cl: usize, n: usize) -> Result<()> {
                let (ar, ac) = opshape(op);
                let (ldb, _) = tight(order, ac, n);
                let (ldc, _) = tight(order, ar, n);
                let (a, d) = (rect(), descr(MatType::General, FillMode::Lower));
                at_guard::<$T, _>(bl, v(1.0), |b| {
                    at_guard::<$T, _>(cl, v(0.0), |c| {
                        ($csrmm)(op, v(1.0), &a, &d, order, b, n, ldb, v(0.0), c, ldc)
                    })
                })
            }
            #[test]
            fn csrmm_controls_and_short_buffers() {
                for op in OPS {
                    for order in ORDERS {
                        let (ar, ac) = opshape(op);
                        let n = 3;
                        let (bl, cl) = (tight(order, ac, n).1, tight(order, ar, n).1);
                        csrmm_call(op, order, bl, cl, n)
                            .unwrap_or_else(|e| panic!("control {op:?} {order:?}: {e:?}"));
                    }
                }
            }
            macro_rules! csrmm_short {
                ($name:ident, $op:expr, $order:expr, b) => {
                    #[test]
                    fn $name() {
                        let (ar, ac) = opshape($op);
                        refused(
                            "csrmm short B",
                            csrmm_call(
                                $op,
                                $order,
                                tight($order, ac, 3).1 - 1,
                                tight($order, ar, 3).1,
                                3,
                            ),
                        );
                    }
                };
                ($name:ident, $op:expr, $order:expr, c) => {
                    #[test]
                    fn $name() {
                        let (ar, ac) = opshape($op);
                        refused(
                            "csrmm short C",
                            csrmm_call(
                                $op,
                                $order,
                                tight($order, ac, 3).1,
                                tight($order, ar, 3).1 - 1,
                                3,
                            ),
                        );
                    }
                };
            }
            csrmm_short!(csrmm_no_col_short_b, Trans::No, Order::ColMajor, b);
            csrmm_short!(csrmm_no_row_short_b, Trans::No, Order::RowMajor, b);
            csrmm_short!(csrmm_t_col_short_b, $tr, Order::ColMajor, b);
            csrmm_short!(csrmm_t_row_short_b, $tr, Order::RowMajor, b);
            csrmm_short!(csrmm_no_col_short_c, Trans::No, Order::ColMajor, c);
            csrmm_short!(csrmm_no_row_short_c, Trans::No, Order::RowMajor, c);
            csrmm_short!(csrmm_t_col_short_c, $tr, Order::ColMajor, c);
            csrmm_short!(csrmm_t_row_short_c, $tr, Order::RowMajor, c);

            // ---------------- spmmd: C(ar x b.n) = op(A) B (B sparse, ac x b.n) ----------------
            fn spmmd_call(op: Trans, order: Order, cl: usize) -> Result<()> {
                let (ar, ac) = opshape(op);
                let bm = if ac == 4 { b4() } else { b3() };
                let ncol = 3;
                let (ldc, _) = tight(order, ar, ncol);
                let a = rect();
                at_guard::<$T, _>(cl, v(0.0), |c| ($spmmd)(op, &a, &bm, order, c, ldc))
            }
            #[test]
            fn spmmd_controls() {
                for op in OPS {
                    for order in ORDERS {
                        let (ar, _) = opshape(op);
                        spmmd_call(op, order, tight(order, ar, 3).1)
                            .unwrap_or_else(|e| panic!("control {op:?} {order:?}: {e:?}"));
                    }
                }
            }
            macro_rules! spmmd_short {
                ($name:ident, $op:expr, $order:expr) => {
                    #[test]
                    fn $name() {
                        let (ar, _) = opshape($op);
                        refused(
                            "spmmd short C",
                            spmmd_call($op, $order, tight($order, ar, 3).1 - 1),
                        );
                    }
                };
            }
            spmmd_short!(spmmd_no_col_short_c, Trans::No, Order::ColMajor);
            spmmd_short!(spmmd_no_row_short_c, Trans::No, Order::RowMajor);
            spmmd_short!(spmmd_t_col_short_c, $tr, Order::ColMajor);
            spmmd_short!(spmmd_t_row_short_c, $tr, Order::RowMajor);

            // ---------------- sp2md: C(ar x bc) = alpha op(A) op(B) + beta C ----------------
            fn sp2md_call(op: Trans, order: Order, cl: usize) -> Result<()> {
                let (ar, ac) = opshape(op);
                let bm = if ac == 4 { b4() } else { b3() };
                let ncol = 3; // op_b = No: B is ac x 3
                let (ldc, _) = tight(order, ar, ncol);
                let (a, da, db) = (
                    rect(),
                    descr(MatType::General, FillMode::Lower),
                    descr(MatType::General, FillMode::Lower),
                );
                at_guard::<$T, _>(cl, v(0.0), |c| {
                    ($sp2md)(
                        op,
                        &da,
                        &a,
                        Trans::No,
                        &db,
                        &bm,
                        v(1.0),
                        v(0.0),
                        c,
                        order,
                        ldc,
                    )
                })
            }
            #[test]
            fn sp2md_controls() {
                for op in OPS {
                    for order in ORDERS {
                        let (ar, _) = opshape(op);
                        sp2md_call(op, order, tight(order, ar, 3).1)
                            .unwrap_or_else(|e| panic!("control {op:?} {order:?}: {e:?}"));
                    }
                }
            }
            macro_rules! sp2md_short {
                ($name:ident, $op:expr, $order:expr) => {
                    #[test]
                    fn $name() {
                        let (ar, _) = opshape($op);
                        refused(
                            "sp2md short C",
                            sp2md_call($op, $order, tight($order, ar, 3).1 - 1),
                        );
                    }
                };
            }
            sp2md_short!(sp2md_no_col_short_c, Trans::No, Order::ColMajor);
            sp2md_short!(sp2md_no_row_short_c, Trans::No, Order::RowMajor);
            sp2md_short!(sp2md_t_col_short_c, $tr, Order::ColMajor);
            sp2md_short!(sp2md_t_row_short_c, $tr, Order::RowMajor);

            // ---------------- trsm: A 4 x 4 lower triangular, B and X 4 x n_rhs ----------------
            fn trsm_call(order: Order, bl: usize, xl: usize) -> Result<()> {
                let order = ($eff)(order);
                let n = 3;
                let (ld, _) = tight(order, 4, n);
                let (a, d) = (sq(), descr(MatType::Triangular, FillMode::Lower));
                at_guard::<$T, _>(bl, v(1.0), |b| {
                    at_guard::<$T, _>(xl, v(0.0), |x| {
                        ($trsm_o)(&$trsm, Trans::No, v(1.0), &a, &d, order, b, n, ld, x, ld)
                    })
                })
            }
            #[test]
            fn trsm_controls() {
                for order in ORDERS {
                    let l = tight(($eff)(order), 4, 3).1;
                    trsm_call(order, l, l).unwrap_or_else(|e| panic!("control {order:?}: {e:?}"));
                }
            }
            macro_rules! trsm_short {
                ($name:ident, $order:expr, b) => {
                    #[test]
                    fn $name() {
                        let l = tight(($eff)($order), 4, 3).1;
                        refused("trsm short B", trsm_call($order, l - 1, l));
                    }
                };
                ($name:ident, $order:expr, x) => {
                    #[test]
                    fn $name() {
                        let l = tight(($eff)($order), 4, 3).1;
                        refused("trsm short X", trsm_call($order, l, l - 1));
                    }
                };
            }
            trsm_short!(trsm_col_short_b, Order::ColMajor, b);
            trsm_short!(trsm_row_short_b, Order::RowMajor, b);
            trsm_short!(trsm_col_short_x, Order::ColMajor, x);
            trsm_short!(trsm_row_short_x, Order::RowMajor, x);

            // ---------------- syrkd: C(r x r) = alpha op(A) op(A)^T + beta C, r = rows of op(A) ----------------
            fn syrkd_call(op: Trans, order: Order, cl: usize) -> Result<()> {
                let r = opshape(op).0;
                let (ld, _) = tight(order, r, r);
                let a = rect();
                at_guard::<$T, _>(cl, v(0.0), |c| {
                    ($syrkd)(op, &a, v(1.0), v(0.0), c, order, ld)
                })
            }
            #[test]
            fn syrkd_controls() {
                for op in OPS {
                    for order in ORDERS {
                        let r = opshape(op).0;
                        syrkd_call(op, order, r * r)
                            .unwrap_or_else(|e| panic!("control {op:?} {order:?}: {e:?}"));
                    }
                }
            }
            macro_rules! syrkd_short {
                ($name:ident, $op:expr, $order:expr) => {
                    #[test]
                    fn $name() {
                        let r = opshape($op).0;
                        refused("syrkd short C", syrkd_call($op, $order, r * r - 1));
                    }
                };
            }
            syrkd_short!(syrkd_no_col_short_c, Trans::No, Order::ColMajor);
            syrkd_short!(syrkd_no_row_short_c, Trans::No, Order::RowMajor);
            syrkd_short!(syrkd_t_col_short_c, $tr, Order::ColMajor);
            syrkd_short!(syrkd_t_row_short_c, $tr, Order::RowMajor);

            // ---------------- syprd: op N: B is k x k (k = 4), C is m x m (m = 3); else B is m x m, C is k x k ----------------
            fn syprd_call(op: Trans, order: Order, bl: usize, cl: usize) -> Result<()> {
                let (bd, cd) = if op == Trans::No { (4, 3) } else { (3, 4) };
                let (ldb, _) = tight(order, bd, bd);
                let (ldc, _) = tight(order, cd, cd);
                let a = rect();
                at_guard::<$T, _>(bl, v(1.0), |b| {
                    at_guard::<$T, _>(cl, v(0.0), |c| {
                        ($syprd)(op, &a, b, order, ldb, v(1.0), v(0.0), c, order, ldc)
                    })
                })
            }
            fn syprd_sizes(op: Trans) -> (usize, usize) {
                if op == Trans::No {
                    (16, 9)
                } else {
                    (9, 16)
                }
            }
            #[test]
            fn syprd_controls() {
                for op in OPS {
                    for order in ORDERS {
                        let (bl, cl) = syprd_sizes(op);
                        syprd_call(op, order, bl, cl)
                            .unwrap_or_else(|e| panic!("control {op:?} {order:?}: {e:?}"));
                    }
                }
            }
            macro_rules! syprd_short {
                ($name:ident, $op:expr, $order:expr, b) => {
                    #[test]
                    fn $name() {
                        let (bl, cl) = syprd_sizes($op);
                        refused("syprd short B", syprd_call($op, $order, bl - 1, cl));
                    }
                };
                ($name:ident, $op:expr, $order:expr, c) => {
                    #[test]
                    fn $name() {
                        let (bl, cl) = syprd_sizes($op);
                        refused("syprd short C", syprd_call($op, $order, bl, cl - 1));
                    }
                };
            }
            syprd_short!(syprd_no_col_short_b, Trans::No, Order::ColMajor, b);
            syprd_short!(syprd_no_row_short_b, Trans::No, Order::RowMajor, b);
            syprd_short!(syprd_t_col_short_b, $tr, Order::ColMajor, b);
            syprd_short!(syprd_t_row_short_b, $tr, Order::RowMajor, b);
            syprd_short!(syprd_no_col_short_c, Trans::No, Order::ColMajor, c);
            syprd_short!(syprd_no_row_short_c, Trans::No, Order::RowMajor, c);
            syprd_short!(syprd_t_col_short_c, $tr, Order::ColMajor, c);
            syprd_short!(syprd_t_row_short_c, $tr, Order::RowMajor, c);
        }
    };
}

// real precisions: trsm takes `order`; the complex trsm has a fixed row order, so its `trsm_ordered` adapter ignores `order`
// and the cases that matter for it are the RowMajor ones (the ColMajor cases are skipped for the complex families below).
fn trsm_real<T: Copy, M>(
    f: &dyn Fn(Trans, T, &M, &MatDescr, Order, &[T], usize, usize, &mut [T], usize) -> Result<()>,
    op: Trans,
    alpha: T,
    a: &M,
    d: &MatDescr,
    order: Order,
    b: &[T],
    n: usize,
    ldb: usize,
    x: &mut [T],
    ldx: usize,
) -> Result<()> {
    f(op, alpha, a, d, order, b, n, ldb, x, ldx)
}

family!(
    real_f64,
    f64,
    SparseMatrix<f64>,
    MatType::Symmetric,
    |v: f64| v,
    csrmm = csrmm::<f64>,
    spmmd = spmmd::<f64>,
    sp2md = sp2md::<f64>,
    trsm = trsm_f64,
    trsm_ordered = trsm_real,
    tr = Trans::T,
    trsm_order = |o: Order| o,
    syrkd = syrkd_f64,
    syprd = syprd_f64
);
family!(
    real_f32,
    f32,
    SparseMatrix<f32>,
    MatType::Symmetric,
    |v: f64| v as f32,
    csrmm = csrmm::<f32>,
    spmmd = spmmd::<f32>,
    sp2md = sp2md::<f32>,
    trsm = trsm_f32,
    trsm_ordered = trsm_real,
    tr = Trans::T,
    trsm_order = |o: Order| o,
    syrkd = syrkd_f32,
    syprd = syprd_f32
);

// complex: the complex trsm has no `order` argument (it is always row-major), so adapt it to the ordered signature and expect
// the ColMajor trsm cases to be meaningless: they are generated but call the row-major routine with a col-major-sized buffer; the
// tight size is the same (rows*cols) so the cases stay valid as length cases.
fn trsm_c64_ordered(
    _f: &fn(
        Trans,
        Complex64,
        &ComplexSparseMatrix<Complex64>,
        &MatDescr,
        &[Complex64],
        usize,
        usize,
        &mut [Complex64],
        usize,
    ) -> Result<()>,
    op: Trans,
    alpha: Complex64,
    a: &ComplexSparseMatrix<Complex64>,
    d: &MatDescr,
    _order: Order,
    b: &[Complex64],
    n: usize,
    ldb: usize,
    x: &mut [Complex64],
    ldx: usize,
) -> Result<()> {
    cx::trsm_c64(op, alpha, a, d, b, n, ldb, x, ldx)
}
fn trsm_c32_ordered(
    _f: &fn(
        Trans,
        Complex32,
        &ComplexSparseMatrix<Complex32>,
        &MatDescr,
        &[Complex32],
        usize,
        usize,
        &mut [Complex32],
        usize,
    ) -> Result<()>,
    op: Trans,
    alpha: Complex32,
    a: &ComplexSparseMatrix<Complex32>,
    d: &MatDescr,
    _order: Order,
    b: &[Complex32],
    n: usize,
    ldb: usize,
    x: &mut [Complex32],
    ldx: usize,
) -> Result<()> {
    cx::trsm_c32(op, alpha, a, d, b, n, ldb, x, ldx)
}

family!(
    cplx_c64,
    Complex64,
    ComplexSparseMatrix<Complex64>,
    MatType::Hermitian,
    |v: f64| Complex64::new(v, 0.0),
    csrmm = cx::csrmm::<Complex64>,
    spmmd = cx::spmmd::<Complex64>,
    sp2md = cx::sp2md::<Complex64>,
    trsm = cx::trsm_c64
        as fn(
            Trans,
            Complex64,
            &ComplexSparseMatrix<Complex64>,
            &MatDescr,
            &[Complex64],
            usize,
            usize,
            &mut [Complex64],
            usize,
        ) -> Result<()>,
    trsm_ordered = trsm_c64_ordered,
    tr = Trans::C,
    trsm_order = |_o: Order| Order::RowMajor,
    syrkd = cx::syrkd_c64,
    syprd = cx::syprd_c64
);
family!(
    cplx_c32,
    Complex32,
    ComplexSparseMatrix<Complex32>,
    MatType::Hermitian,
    |v: f64| Complex32::new(v as f32, 0.0),
    csrmm = cx::csrmm::<Complex32>,
    spmmd = cx::spmmd::<Complex32>,
    sp2md = cx::sp2md::<Complex32>,
    trsm = cx::trsm_c32
        as fn(
            Trans,
            Complex32,
            &ComplexSparseMatrix<Complex32>,
            &MatDescr,
            &[Complex32],
            usize,
            usize,
            &mut [Complex32],
            usize,
        ) -> Result<()>,
    trsm_ordered = trsm_c32_ordered,
    tr = Trans::C,
    trsm_order = |_o: Order| Order::RowMajor,
    syrkd = cx::syrkd_c32,
    syprd = cx::syprd_c32
);
