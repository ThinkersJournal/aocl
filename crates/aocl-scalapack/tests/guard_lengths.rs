//! Guard-page harness for the aocl-scalapack safe functions (18 of these 29 short cases were access violations before the checks) (single MPI rank, 1 x 1 grid, `harness = false`; hand-written). One case per call:
//! `<exe> --list` prints the cases, `<exe> --exact <name>` runs one (tools/safe-api-harness.py runs each in its own process). A short case
//! passes one buffer ONE ELEMENT short of what the descriptor (or the lwork argument) says the library will touch, at the end of a committed
//! page before a PAGE_NOACCESS page: an access violation = HOLE. Controls (right sizes) must succeed. Windows only; Linux unmeasured.
//! Needs the Intel MPI runtime on PATH (the driver adds it).
#![cfg(windows)]
use aocl_scalapack::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
const N: usize = 4;
const NB: usize = 2;

/// `init[..n]` copied to the end of a committed page followed by a PAGE_NOACCESS page.
fn at_guard<T: Copy, R>(init: &[T], n: usize, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n <= init.len() && n * core::mem::size_of::<T>() <= PAGE);
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
        for (i, v) in init.iter().take(n).enumerate() {
            start.add(i).write(*v);
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

/// Symmetric positive definite N x N, column-major.
fn spd() -> Vec<f64> {
    (0..N * N)
        .map(|k| {
            let (i, j) = (k % N, k / N);
            if i == j {
                10.0 + i as f64
            } else {
                0.5 / (1.0 + (i as f64 - j as f64).abs())
            }
        })
        .collect()
}
fn bvec() -> Vec<f64> {
    vec![1.0, 2.0, 3.0, 4.0]
}
fn desc(g: &Grid, m: usize, n: usize) -> BlockCyclic {
    BlockCyclic::new(g, m, n, NB, NB.min(n), 0, 0, m.max(1)).unwrap()
}

type Out = Result<()>;
type Case = (&'static str, fn() -> Out);

fn lwork_of(f: impl Fn(&mut [f64], Int) -> Out) -> usize {
    let mut w = [0.0f64; 1];
    f(&mut w, -1).unwrap();
    w[0] as usize + 16
}
fn pdgesv_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a, mut b, mut ip) = (spd(), bvec(), vec![0 as Int; 6]);
    pdgesv(
        N,
        1,
        &mut a,
        1,
        1,
        &desc(&g, N, N),
        &mut ip,
        &mut b,
        1,
        1,
        &desc(&g, N, 1),
    )
}
fn pdgesv_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut b, mut ip) = (bvec(), vec![0 as Int; 6]);
    at_guard(&spd(), N * N - 1, |a| {
        pdgesv(
            N,
            1,
            a,
            1,
            1,
            &desc(&g, N, N),
            &mut ip,
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgesv_short_b() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a, mut ip) = (spd(), vec![0 as Int; 6]);
    at_guard(&bvec(), N - 1, |b| {
        pdgesv(
            N,
            1,
            &mut a,
            1,
            1,
            &desc(&g, N, N),
            &mut ip,
            b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgesv_short_ipiv() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a, mut b) = (spd(), bvec());
    at_guard(&[0 as Int; 6], 5, |ip| {
        pdgesv(
            N,
            1,
            &mut a,
            1,
            1,
            &desc(&g, N, N),
            ip,
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgesv_n_exceeds_desc() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut b = bvec();
    let mut ip = vec![0 as Int; 6];
    at_guard(&spd(), N * N, |a| {
        pdgesv(
            N + 1,
            1,
            a,
            1,
            1,
            &desc(&g, N, N),
            &mut ip,
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgesv_ia_past_desc() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut b = bvec();
    let mut ip = vec![0 as Int; 6];
    at_guard(&spd(), N * N, |a| {
        pdgesv(
            N,
            1,
            a,
            3,
            1,
            &desc(&g, N, N),
            &mut ip,
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgemm_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (a, b, mut c) = (spd(), spd(), spd());
    pdgemm(
        Trans::No,
        Trans::No,
        N,
        N,
        N,
        1.0,
        &a,
        1,
        1,
        &desc(&g, N, N),
        &b,
        1,
        1,
        &desc(&g, N, N),
        0.0,
        &mut c,
        1,
        1,
        &desc(&g, N, N),
    )
}
fn pdgemm_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (b, mut c) = (spd(), spd());
    at_guard(&spd(), N * N - 1, |a| {
        pdgemm(
            Trans::No,
            Trans::No,
            N,
            N,
            N,
            1.0,
            a,
            1,
            1,
            &desc(&g, N, N),
            &b,
            1,
            1,
            &desc(&g, N, N),
            0.0,
            &mut c,
            1,
            1,
            &desc(&g, N, N),
        )
    })
}
fn pdgemm_short_b() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (a, mut c) = (spd(), spd());
    at_guard(&spd(), N * N - 1, |b| {
        pdgemm(
            Trans::No,
            Trans::No,
            N,
            N,
            N,
            1.0,
            &a,
            1,
            1,
            &desc(&g, N, N),
            b,
            1,
            1,
            &desc(&g, N, N),
            0.0,
            &mut c,
            1,
            1,
            &desc(&g, N, N),
        )
    })
}
fn pdgemm_short_c() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (a, b) = (spd(), spd());
    at_guard(&spd(), N * N - 1, |c| {
        pdgemm(
            Trans::No,
            Trans::No,
            N,
            N,
            N,
            1.0,
            &a,
            1,
            1,
            &desc(&g, N, N),
            &b,
            1,
            1,
            &desc(&g, N, N),
            0.0,
            c,
            1,
            1,
            &desc(&g, N, N),
        )
    })
}
fn pdgemm_n_exceeds_desc() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (a, b) = (spd(), spd());
    at_guard(&spd(), N * N, |c| {
        pdgemm(
            Trans::No,
            Trans::No,
            N,
            N + 1,
            N,
            1.0,
            &a,
            1,
            1,
            &desc(&g, N, N),
            &b,
            1,
            1,
            &desc(&g, N, N),
            0.0,
            c,
            1,
            1,
            &desc(&g, N, N),
        )
    })
}
fn pdpotrf_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut a = spd();
    pdpotrf(Uplo::Lower, N, &mut a, 1, 1, &desc(&g, N, N))
}
fn pdpotrf_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    at_guard(&spd(), N * N - 1, |a| {
        pdpotrf(Uplo::Lower, N, a, 1, 1, &desc(&g, N, N))
    })
}
fn pdpotrf_n_exceeds_desc() -> Out {
    let g = Grid::new(1, 1).unwrap();
    at_guard(&spd(), N * N, |a| {
        pdpotrf(Uplo::Lower, N + 1, a, 1, 1, &desc(&g, N, N))
    })
}
fn pdgetrf_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a, mut ip) = (spd(), vec![0 as Int; 6]);
    pdgetrf(N, N, &mut a, 1, 1, &desc(&g, N, N), &mut ip)
}
fn pdgetrf_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut ip = vec![0 as Int; 6];
    at_guard(&spd(), N * N - 1, |a| {
        pdgetrf(N, N, a, 1, 1, &desc(&g, N, N), &mut ip)
    })
}
fn pdgetrf_short_ipiv() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut a = spd();
    at_guard(&[0 as Int; 6], 5, |ip| {
        pdgetrf(N, N, &mut a, 1, 1, &desc(&g, N, N), ip)
    })
}
fn pdgetrf_n_exceeds_desc() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut ip = vec![0 as Int; 6];
    at_guard(&spd(), N * N, |a| {
        pdgetrf(N, N + 1, a, 1, 1, &desc(&g, N, N), &mut ip)
    })
}
fn pdgetrs_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a, mut ip, mut b) = (spd(), vec![0 as Int; 6], bvec());
    pdgetrf(N, N, &mut a, 1, 1, &desc(&g, N, N), &mut ip).unwrap();
    pdgetrs(
        Trans::No,
        N,
        1,
        &a,
        1,
        1,
        &desc(&g, N, N),
        &ip,
        &mut b,
        1,
        1,
        &desc(&g, N, 1),
    )
}
fn pdgetrs_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a0, mut ip0) = (spd(), vec![0 as Int; 6]);
    pdgetrf(N, N, &mut a0, 1, 1, &desc(&g, N, N), &mut ip0).unwrap();
    let mut b = bvec();
    at_guard(&a0, N * N - 1, |a| {
        pdgetrs(
            Trans::No,
            N,
            1,
            a,
            1,
            1,
            &desc(&g, N, N),
            &ip0,
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgetrs_short_b() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a0, mut ip0) = (spd(), vec![0 as Int; 6]);
    pdgetrf(N, N, &mut a0, 1, 1, &desc(&g, N, N), &mut ip0).unwrap();
    at_guard(&bvec(), N - 1, |b| {
        pdgetrs(
            Trans::No,
            N,
            1,
            &a0,
            1,
            1,
            &desc(&g, N, N),
            &ip0,
            b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdgetrs_short_ipiv() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a0, mut ip0) = (spd(), vec![0 as Int; 6]);
    pdgetrf(N, N, &mut a0, 1, 1, &desc(&g, N, N), &mut ip0).unwrap();
    let mut b = bvec();
    at_guard(&ip0, 5, |ip| {
        pdgetrs(
            Trans::No,
            N,
            1,
            &a0,
            1,
            1,
            &desc(&g, N, N),
            ip,
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdpotrs_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let (mut a, mut b) = (spd(), bvec());
    pdpotrf(Uplo::Lower, N, &mut a, 1, 1, &desc(&g, N, N)).unwrap();
    pdpotrs(
        Uplo::Lower,
        N,
        1,
        &a,
        1,
        1,
        &desc(&g, N, N),
        &mut b,
        1,
        1,
        &desc(&g, N, 1),
    )
}
fn pdpotrs_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut a0 = spd();
    pdpotrf(Uplo::Lower, N, &mut a0, 1, 1, &desc(&g, N, N)).unwrap();
    let mut b = bvec();
    at_guard(&a0, N * N - 1, |a| {
        pdpotrs(
            Uplo::Lower,
            N,
            1,
            a,
            1,
            1,
            &desc(&g, N, N),
            &mut b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdpotrs_short_b() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let mut a0 = spd();
    pdpotrf(Uplo::Lower, N, &mut a0, 1, 1, &desc(&g, N, N)).unwrap();
    at_guard(&bvec(), N - 1, |b| {
        pdpotrs(
            Uplo::Lower,
            N,
            1,
            &a0,
            1,
            1,
            &desc(&g, N, N),
            b,
            1,
            1,
            &desc(&g, N, 1),
        )
    })
}
fn pdsyev_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut wv, mut z) = (spd(), vec![0.0; N], vec![0.0; N * N]);
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            w,
            l,
        )
    });
    let (mut a, mut wv, mut z, mut w) = (spd(), vec![0.0; N], vec![0.0; N * N], vec![0.0; lw]);
    pdsyev(
        EigCompute::ValuesAndVectors,
        Uplo::Lower,
        N,
        &mut a,
        1,
        1,
        &da,
        &mut wv,
        &mut z,
        1,
        1,
        &da,
        &mut w,
        lw as Int,
    )
}
fn pdsyev_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut wv, mut z) = (spd(), vec![0.0; N], vec![0.0; N * N]);
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            w,
            l,
        )
    });
    let mut wv = vec![0.0; N];
    let mut z = vec![0.0; N * N];
    let mut w = vec![0.0; lw];
    at_guard(&spd(), N * N - 1, |a| {
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            &mut w,
            lw as Int,
        )
    })
}
fn pdsyev_short_w() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut wv, mut z) = (spd(), vec![0.0; N], vec![0.0; N * N]);
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            w,
            l,
        )
    });
    let mut a = spd();
    let mut z = vec![0.0; N * N];
    let mut w = vec![0.0; lw];
    at_guard(&[0.0; N], N - 1, |x| {
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            x,
            &mut z,
            1,
            1,
            &da,
            &mut w,
            lw as Int,
        )
    })
}
fn pdsyev_short_z() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut wv, mut z) = (spd(), vec![0.0; N], vec![0.0; N * N]);
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            w,
            l,
        )
    });
    let mut a = spd();
    let mut wv = vec![0.0; N];
    let mut w = vec![0.0; lw];
    at_guard(&[0.0; N * N], N * N - 1, |x| {
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            x,
            1,
            1,
            &da,
            &mut w,
            lw as Int,
        )
    })
}
fn pdsyev_short_work() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut wv, mut z) = (spd(), vec![0.0; N], vec![0.0; N * N]);
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            w,
            l,
        )
    });
    let mut a = spd();
    let mut wv = vec![0.0; N];
    let mut z = vec![0.0; N * N];
    at_guard(&vec![0.0; lw], lw - 1, |x| {
        pdsyev(
            EigCompute::ValuesAndVectors,
            Uplo::Lower,
            N,
            &mut a,
            1,
            1,
            &da,
            &mut wv,
            &mut z,
            1,
            1,
            &da,
            x,
            lw as Int,
        )
    })
}
fn pdgels_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let db = desc(&g, N, 1);
    let lw = lwork_of(|w, l| {
        let (mut a, mut b) = (spd(), bvec());
        pdgels(
            Trans::No,
            N,
            N,
            1,
            &mut a,
            1,
            1,
            &da,
            &mut b,
            1,
            1,
            &db,
            w,
            l,
        )
    });
    let (mut a, mut b, mut w) = (spd(), bvec(), vec![0.0; lw]);
    pdgels(
        Trans::No,
        N,
        N,
        1,
        &mut a,
        1,
        1,
        &da,
        &mut b,
        1,
        1,
        &db,
        &mut w,
        lw as Int,
    )
}
fn pdgels_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let db = desc(&g, N, 1);
    let lw = lwork_of(|w, l| {
        let (mut a, mut b) = (spd(), bvec());
        pdgels(
            Trans::No,
            N,
            N,
            1,
            &mut a,
            1,
            1,
            &da,
            &mut b,
            1,
            1,
            &db,
            w,
            l,
        )
    });
    let (mut b, mut w) = (bvec(), vec![0.0; lw]);
    at_guard(&spd(), N * N - 1, |a| {
        pdgels(
            Trans::No,
            N,
            N,
            1,
            a,
            1,
            1,
            &da,
            &mut b,
            1,
            1,
            &db,
            &mut w,
            lw as Int,
        )
    })
}
fn pdgels_short_b() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let db = desc(&g, N, 1);
    let lw = lwork_of(|w, l| {
        let (mut a, mut b) = (spd(), bvec());
        pdgels(
            Trans::No,
            N,
            N,
            1,
            &mut a,
            1,
            1,
            &da,
            &mut b,
            1,
            1,
            &db,
            w,
            l,
        )
    });
    let (mut a, mut w) = (spd(), vec![0.0; lw]);
    at_guard(&bvec(), N - 1, |b| {
        pdgels(
            Trans::No,
            N,
            N,
            1,
            &mut a,
            1,
            1,
            &da,
            b,
            1,
            1,
            &db,
            &mut w,
            lw as Int,
        )
    })
}
fn pdgels_short_work() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let db = desc(&g, N, 1);
    let lw = lwork_of(|w, l| {
        let (mut a, mut b) = (spd(), bvec());
        pdgels(
            Trans::No,
            N,
            N,
            1,
            &mut a,
            1,
            1,
            &da,
            &mut b,
            1,
            1,
            &db,
            w,
            l,
        )
    });
    let (mut a, mut b) = (spd(), bvec());
    at_guard(&vec![0.0; lw], lw - 1, |w| {
        pdgels(
            Trans::No,
            N,
            N,
            1,
            &mut a,
            1,
            1,
            &da,
            &mut b,
            1,
            1,
            &db,
            w,
            lw as Int,
        )
    })
}
fn pdgeqrf_control() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut t) = (spd(), vec![0.0; N]);
        pdgeqrf(N, N, &mut a, 1, 1, &da, &mut t, w, l)
    });
    let (mut a, mut t, mut w) = (spd(), vec![0.0; N], vec![0.0; lw]);
    pdgeqrf(N, N, &mut a, 1, 1, &da, &mut t, &mut w, lw as Int)
}
fn pdgeqrf_short_a() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut t) = (spd(), vec![0.0; N]);
        pdgeqrf(N, N, &mut a, 1, 1, &da, &mut t, w, l)
    });
    let (mut t, mut w) = (vec![0.0; N], vec![0.0; lw]);
    at_guard(&spd(), N * N - 1, |a| {
        pdgeqrf(N, N, a, 1, 1, &da, &mut t, &mut w, lw as Int)
    })
}
fn pdgeqrf_short_tau() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut t) = (spd(), vec![0.0; N]);
        pdgeqrf(N, N, &mut a, 1, 1, &da, &mut t, w, l)
    });
    let (mut a, mut w) = (spd(), vec![0.0; lw]);
    at_guard(&[0.0; N], N - 1, |t| {
        pdgeqrf(N, N, &mut a, 1, 1, &da, t, &mut w, lw as Int)
    })
}
fn pdgeqrf_short_work() -> Out {
    let g = Grid::new(1, 1).unwrap();
    let da = desc(&g, N, N);
    let lw = lwork_of(|w, l| {
        let (mut a, mut t) = (spd(), vec![0.0; N]);
        pdgeqrf(N, N, &mut a, 1, 1, &da, &mut t, w, l)
    });
    let (mut a, mut t) = (spd(), vec![0.0; N]);
    at_guard(&vec![0.0; lw], lw - 1, |w| {
        pdgeqrf(N, N, &mut a, 1, 1, &da, &mut t, w, lw as Int)
    })
}

const CASES: &[Case] = &[
    ("pdgesv_control", pdgesv_control),
    ("pdgesv_short_a", pdgesv_short_a),
    ("pdgesv_short_b", pdgesv_short_b),
    ("pdgesv_short_ipiv", pdgesv_short_ipiv),
    ("pdgesv_n_exceeds_desc", pdgesv_n_exceeds_desc),
    ("pdgesv_ia_past_desc", pdgesv_ia_past_desc),
    ("pdgemm_control", pdgemm_control),
    ("pdgemm_short_a", pdgemm_short_a),
    ("pdgemm_short_b", pdgemm_short_b),
    ("pdgemm_short_c", pdgemm_short_c),
    ("pdgemm_n_exceeds_desc", pdgemm_n_exceeds_desc),
    ("pdpotrf_control", pdpotrf_control),
    ("pdpotrf_short_a", pdpotrf_short_a),
    ("pdpotrf_n_exceeds_desc", pdpotrf_n_exceeds_desc),
    ("pdgetrf_control", pdgetrf_control),
    ("pdgetrf_short_a", pdgetrf_short_a),
    ("pdgetrf_short_ipiv", pdgetrf_short_ipiv),
    ("pdgetrf_n_exceeds_desc", pdgetrf_n_exceeds_desc),
    ("pdgetrs_control", pdgetrs_control),
    ("pdgetrs_short_a", pdgetrs_short_a),
    ("pdgetrs_short_b", pdgetrs_short_b),
    ("pdgetrs_short_ipiv", pdgetrs_short_ipiv),
    ("pdpotrs_control", pdpotrs_control),
    ("pdpotrs_short_a", pdpotrs_short_a),
    ("pdpotrs_short_b", pdpotrs_short_b),
    ("pdsyev_control", pdsyev_control),
    ("pdsyev_short_a", pdsyev_short_a),
    ("pdsyev_short_w", pdsyev_short_w),
    ("pdsyev_short_z", pdsyev_short_z),
    ("pdsyev_short_work", pdsyev_short_work),
    ("pdgels_control", pdgels_control),
    ("pdgels_short_a", pdgels_short_a),
    ("pdgels_short_b", pdgels_short_b),
    ("pdgels_short_work", pdgels_short_work),
    ("pdgeqrf_control", pdgeqrf_control),
    ("pdgeqrf_short_a", pdgeqrf_short_a),
    ("pdgeqrf_short_tau", pdgeqrf_short_tau),
    ("pdgeqrf_short_work", pdgeqrf_short_work),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--list") {
        for (n, _) in CASES {
            println!("{n}: test");
        }
        return;
    }
    let pos = args
        .iter()
        .position(|a| a == "--exact")
        .expect("--exact <name>");
    let name = &args[pos + 1];
    let (_, f) = CASES.iter().find(|(n, _)| n == name).expect("unknown case");
    let r = f();
    if name.ends_with("_control") {
        r.unwrap();
    } else {
        // every non-control case passes an argument the descriptor / lwork says is too small (or outside the matrix): refused before the call
        assert!(matches!(r, Err(Error::InvalidArgument(_))), "{name}: {r:?}");
    }
    blacs_exit(false);
}
