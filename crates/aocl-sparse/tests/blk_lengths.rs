//! (7 of these 10 broken-input cases ACCESS-VIOLATED before the checks.) Guard-page harness for `blkcsrmv_f64` (the raw block-CSR arrays from `aoclsparse_csr2blkcsr`: block row pointers `brp`, block column indices
//! `bci`, values `bv`, per-row bitmasks `masks`; 4 x 8 matrix, 12 non-zeros, one block of 4 rows: `brp` 5 entries, `bci` 1, `bv` 12, `masks` 4,
//! measured). A control passes the exact lengths; each short case passes one array one element short, and each content case breaks ONE index,
//! all at the end of a committed page before a PAGE_NOACCESS page (a fault = HOLE). One process per test (tools/safe-api-harness.py).
//! Windows only; Linux unmeasured.
#![cfg(windows)]
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "test helpers mirror the long signatures of the wrappers under test"
)]
use aocl_sparse::*;
use aocl_sparse_sys as sys;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
type I = i32;

/// Run `f` with a COPY of `src[..n]` that ENDS at a committed page, the next page being PAGE_NOACCESS.
fn at_guard<T: Copy, R>(src: &[T], n: usize, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n <= src.len() && n * core::mem::size_of::<T>() <= PAGE);
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
        let s = core::slice::from_raw_parts_mut(start, n);
        s.copy_from_slice(&src[..n]);
        let r = f(s);
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn descr() -> MatDescr {
    let mut d = MatDescr::new().unwrap();
    d.set_type(MatType::General).unwrap();
    d.set_index_base(IndexBase::Zero).unwrap();
    d
}
struct Blk {
    brp: Vec<I>,
    bci: Vec<I>,
    bv: Vec<f64>,
    masks: Vec<u8>,
}
fn blk() -> Blk {
    let dense = [
        [1., 0., 2., 0., 0., 3., 0., 0.],
        [0., 4., 0., 0., 5., 0., 0., 6.],
        [7., 0., 0., 8., 0., 0., 9., 0.],
        [0., 0., 1., 0., 2., 0., 0., 3.],
    ];
    let (mut rp, mut ci, mut v) = (vec![0 as I], vec![], vec![]);
    for r in dense.iter() {
        for (j, x) in r.iter().enumerate() {
            if *x != 0.0 {
                ci.push(j as I);
                v.push(*x);
            }
        }
        rp.push(ci.len() as I);
    }
    let nnz = v.len();
    let (mut brp, mut bci, mut bv, mut masks) = (
        vec![0 as I; 64],
        vec![0 as I; 64],
        vec![0.0f64; 64],
        vec![0u8; 64],
    );
    let st = unsafe {
        sys::aoclsparse_csr2blkcsr(
            4,
            8,
            nnz as I,
            rp.as_ptr(),
            ci.as_ptr(),
            v.as_ptr(),
            brp.as_mut_ptr(),
            bci.as_mut_ptr(),
            bv.as_mut_ptr(),
            masks.as_mut_ptr(),
            4,
            sys::aoclsparse_index_base__aoclsparse_index_base_zero,
        )
    };
    assert_eq!(st, 0);
    brp.truncate(5);
    bci.truncate(1);
    bv.truncate(12);
    masks.truncate(4);
    Blk {
        brp,
        bci,
        bv,
        masks,
    }
}
const X: [f64; 8] = [1., 2., 3., 4., 5., 6., 7., 8.];
const Y: [f64; 4] = [0.5; 4];
fn run(x: &[f64], y: &mut [f64], masks: &[u8], bv: &[f64], bci: &[I], brp: &[I]) -> Result<()> {
    blkcsrmv_f64(
        Trans::No,
        1.0,
        4,
        8,
        masks,
        bv,
        bci,
        brp,
        &descr(),
        x,
        0.5,
        y,
        4,
    )
}
#[test]
fn blk_control() {
    let b = blk();
    let mut y = Y;
    run(&X, &mut y, &b.masks, &b.bv, &b.bci, &b.brp).unwrap();
}
macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let b = blk();
            let mut y = Y;
            let _ = (&b, &mut y);
            let r = $body(&b, &mut y);
            assert!(
                matches!(r, Err(Error::InvalidArgument(_))),
                "{}: {:?}",
                stringify!($name),
                r
            );
        }
    };
}
case!(blk_short_x, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &X,
    7,
    |x| run(x, y, &b.masks, &b.bv, &b.bci, &b.brp)
));
case!(blk_short_y, |b: &Blk, _y: &mut [f64; 4]| at_guard(
    &Y,
    3,
    |y| run(&X, y, &b.masks, &b.bv, &b.bci, &b.brp)
));
case!(blk_short_masks, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &b.masks,
    3,
    |m| run(&X, y, m, &b.bv, &b.bci, &b.brp)
));
case!(blk_short_bv, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &b.bv,
    11,
    |v| run(&X, y, &b.masks, v, &b.bci, &b.brp)
));
case!(blk_short_bci, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &b.bci,
    0,
    |c| run(&X, y, &b.masks, &b.bv, c, &b.brp)
));
case!(blk_short_brp, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &b.brp,
    4,
    |r| run(&X, y, &b.masks, &b.bv, &b.bci, r)
));
case!(blk_bci_past_x, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &X,
    8,
    |x| run(x, y, &b.masks, &b.bv, &[1], &b.brp)
));
case!(blk_bci_negative, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &X,
    8,
    |x| run(x, y, &b.masks, &b.bv, &[-1], &b.brp)
));
case!(blk_brp_past_nnz, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &b.bv,
    12,
    |v| run(&X, y, &b.masks, v, &b.bci, &[0, 1000, 1000, 1000, 1000])
));
case!(blk_brp_decreasing, |b: &Blk, y: &mut [f64; 4]| at_guard(
    &b.bv,
    12,
    |v| run(&X, y, &b.masks, v, &b.bci, &[0, 5, 1, 1, 1])
));

/// Positive controls: valid block-CSR matrices of other shapes (m not a multiple of the block height, block columns not multiples of 8,
/// several block rows) are NOT refused and give the dense product.
#[test]
fn blk_valid_shapes_are_accepted_and_correct() {
    for (m, n) in [(4usize, 8usize), (8, 16), (5, 11), (12, 20)] {
        let dense: Vec<Vec<f64>> = (0..m)
            .map(|r| {
                (0..n)
                    .map(|j| {
                        if (r * 7 + j * 3) % 5 == 0 {
                            1.0 + (r * n + j) as f64
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect();
        let (mut rp, mut ci, mut v) = (vec![0 as I], vec![], vec![]);
        for r in &dense {
            for (j, x) in r.iter().enumerate() {
                if *x != 0.0 {
                    ci.push(j as I);
                    v.push(*x);
                }
            }
            rp.push(ci.len() as I);
        }
        let (mut brp, mut bci, mut bv, mut masks) = (
            vec![0 as I; 256],
            vec![0 as I; 256],
            vec![0.0f64; 256],
            vec![0u8; 256],
        );
        let st = unsafe {
            sys::aoclsparse_csr2blkcsr(
                m as I,
                n as I,
                v.len() as I,
                rp.as_ptr(),
                ci.as_ptr(),
                v.as_ptr(),
                brp.as_mut_ptr(),
                bci.as_mut_ptr(),
                bv.as_mut_ptr(),
                masks.as_mut_ptr(),
                4,
                sys::aoclsparse_index_base__aoclsparse_index_base_zero,
            )
        };
        assert_eq!(st, 0);
        let nb = brp[m] as usize;
        let x: Vec<f64> = (0..n).map(|j| 1.0 + j as f64 * 0.5).collect();
        let mut y = vec![0.25f64; m];
        blkcsrmv_f64(
            Trans::No,
            2.0,
            m,
            n,
            &masks[..nb * 4],
            &bv[..v.len()],
            &bci[..nb],
            &brp[..=m],
            &descr(),
            &x,
            0.5,
            &mut y,
            4,
        )
        .unwrap_or_else(|e| panic!("{m}x{n}: {e:?}"));
        for r in 0..m {
            let want = 2.0 * (0..n).map(|j| dense[r][j] * x[j]).sum::<f64>() + 0.5 * 0.25;
            assert!(
                (y[r] - want).abs() < 1e-9,
                "{m}x{n} row {r}: {} vs {want}",
                y[r]
            );
        }
    }
}
