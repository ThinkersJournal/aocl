//! True 2D block-cyclic grids and more than one right-hand side. `harness = false`; launch as
//! `mpiexec -n P <exe>` with P = 1, 2, 3, 4, 6: the grid is 1x1, 2x1, 3x1, 2x2 and 2x3, so rows AND columns are dealt
//! cyclically across processes (the other tests use a `1 x P` grid and one right-hand side). Each process checks the
//! entries it owns against the known answer; matrices are asymmetric so a transposed or mis-dealt one gives a wrong `x`.

use aocl_scalapack::{
    blacs_exit, numroc, pcgesv, pdgemm, pdgesv, pdgetrf, pdgetrs, pdpotrf, pdpotrs, pzgesv,
    BlockCyclic, Grid, Int, Trans, Uplo,
};

const N: usize = 6;
const NB: usize = 2;
const NRHS: usize = 3; // not a multiple of NB: the last column block is short

fn a_re(i: usize, j: usize) -> f64 {
    if i == j {
        10.0 + i as f64
    } else {
        i as f64 - 2.0 * j as f64 + 0.5
    }
}
fn a_im(i: usize, j: usize) -> f64 {
    if i == j {
        1.0
    } else {
        0.3 * i as f64 - 0.7 * j as f64
    }
}
fn x_re(i: usize, r: usize) -> f64 {
    (i + 1) as f64 * (1.0 + r as f64) - 2.0 * r as f64
}
fn x_im(i: usize, r: usize) -> f64 {
    -0.5 * i as f64 + 0.25 + r as f64
}
fn spd(i: usize, j: usize) -> f64 {
    if i == j {
        8.0 + i as f64
    } else {
        1.0 / (1.0 + (i as f64 - j as f64).abs())
    }
}

struct Layout2d {
    pr: usize,
    pc: usize,
    myrow: usize,
    mycol: usize,
    /// Process row / column that owns the first block (the descriptor's RSRC / CSRC).
    rsrc: usize,
    csrc: usize,
}
impl Layout2d {
    /// Local row/column count for an `m x n` global matrix.
    fn local_dims(&self, m: usize, n: usize) -> (usize, usize) {
        (
            numroc(m, NB, self.myrow, self.rsrc, self.pr),
            numroc(n, NB, self.mycol, self.csrc, self.pc),
        )
    }
    fn global_row(&self, li: usize) -> usize {
        ((li / NB) * self.pr + (self.myrow + self.pr - self.rsrc) % self.pr) * NB + li % NB
    }
    fn global_col(&self, lj: usize) -> usize {
        ((lj / NB) * self.pc + (self.mycol + self.pc - self.csrc) % self.pc) * NB + lj % NB
    }
    /// Column-major local array (leading dimension `lld`) of the entries of `f` this process owns.
    fn local<T: Copy + Default>(
        &self,
        m: usize,
        n: usize,
        f: &dyn Fn(usize, usize) -> T,
    ) -> (Vec<T>, usize) {
        let (lr, lc) = self.local_dims(m, n);
        let lld = lr.max(1);
        let mut v = vec![T::default(); lld * lc.max(1)];
        for lj in 0..lc {
            for li in 0..lr {
                v[lj * lld + li] = f(self.global_row(li), self.global_col(lj));
            }
        }
        (v, lld)
    }
}

fn shape(p: usize) -> (usize, usize) {
    match p {
        1 => (1, 1),
        2 => (2, 1),
        3 => (3, 1),
        4 => (2, 2),
        6 => (2, 3),
        _ => panic!("run with 1, 2, 3, 4 or 6 ranks"),
    }
}

fn main() {
    let (mut mypnum, mut nprocs) = (-1, 0);
    unsafe { aocl_scalapack_sys::Cblacs_pinfo(&mut mypnum, &mut nprocs) };
    let (pr, pc) = shape(nprocs as usize);
    let grid = Grid::new(pr, pc).expect("grid");
    let (myrow, mycol) = grid.coords();
    assert!(myrow < pr && mycol < pc, "every rank must be in the grid");
    let lay = Layout2d {
        pr,
        pc,
        myrow,
        mycol,
        rsrc: 0,
        csrc: 0,
    };
    // The same data dealt starting from process (1 % pr, 1 % pc) instead of (0, 0): exercises RSRC / CSRC.
    let shifted = Layout2d {
        pr,
        pc,
        myrow,
        mycol,
        rsrc: 1 % pr,
        csrc: 1 % pc,
    };
    let tag = format!("rank {mypnum}/{nprocs} ({pr}x{pc})");
    let desc = |m: usize, n: usize, lld: usize| {
        BlockCyclic::new(&grid, m, n, NB, NB, 0, 0, lld).expect("descriptor")
    };

    // ---- pdgesv with NRHS = 3 ----
    {
        let (mut a, lld_a) = lay.local(N, N, &a_re);
        let rhs = |i: usize, r: usize| (0..N).map(|k| a_re(i, k) * x_re(k, r)).sum::<f64>();
        let (mut b, lld_b) = lay.local(N, NRHS, &rhs);
        let (da, db) = (desc(N, N, lld_a), desc(N, NRHS, lld_b));
        let mut ipiv = vec![0 as Int; lay.local_dims(N, N).0 + NB];
        pdgesv(N, NRHS, &mut a, 1, 1, &da, &mut ipiv, &mut b, 1, 1, &db).expect("pdgesv");
        let (lr, lc) = lay.local_dims(N, NRHS);
        for lj in 0..lc {
            for li in 0..lr {
                let (i, r) = (lay.global_row(li), lay.global_col(lj));
                let (got, want) = (b[lj * lld_b + li], x_re(i, r));
                assert!(
                    (got - want).abs() < 1e-9,
                    "{tag}: pdgesv x[{i},{r}] = {got}, want {want}"
                );
            }
        }
    }

    // ---- pdgesv with the first block owned by process (1 % pr, 1 % pc), NRHS = 3 ----
    {
        let lay = &shifted;
        let (mut a, lld_a) = lay.local(N, N, &a_re);
        let rhs = |i: usize, r: usize| (0..N).map(|k| a_re(i, k) * x_re(k, r)).sum::<f64>();
        let (mut b, lld_b) = lay.local(N, NRHS, &rhs);
        let mk = |m: usize, n: usize, lld: usize| {
            BlockCyclic::new(&grid, m, n, NB, NB, lay.rsrc, lay.csrc, lld)
                .expect("shifted descriptor")
        };
        let (da, db) = (mk(N, N, lld_a), mk(N, NRHS, lld_b));
        let mut ipiv = vec![0 as Int; lay.local_dims(N, N).0 + NB];
        pdgesv(N, NRHS, &mut a, 1, 1, &da, &mut ipiv, &mut b, 1, 1, &db).expect("pdgesv shifted");
        let (lr, lc) = lay.local_dims(N, NRHS);
        for lj in 0..lc {
            for li in 0..lr {
                let (i, r) = (lay.global_row(li), lay.global_col(lj));
                let (got, want) = (b[lj * lld_b + li], x_re(i, r));
                assert!(
                    (got - want).abs() < 1e-9,
                    "{tag}: shifted pdgesv x[{i},{r}] = {got}, want {want}"
                );
            }
        }
    }

    // ---- pdgemm on a 2D grid: C = 2 A B + 0.5 C0 and C = A^T B ----
    {
        let b_f = |i: usize, j: usize| i as f64 * 0.5 - j as f64 + 1.0;
        let c0 = |i: usize, j: usize| i as f64 + j as f64 * 0.25;
        let (a, lld) = lay.local(N, N, &a_re);
        let (b, _) = lay.local(N, N, &b_f);
        let d = desc(N, N, lld);
        for (trans, alpha, beta) in [(Trans::No, 2.0, 0.5), (Trans::T, 1.0, 0.0)] {
            let (mut c, _) = lay.local(N, N, &c0);
            pdgemm(
                trans,
                Trans::No,
                N,
                N,
                N,
                alpha,
                &a,
                1,
                1,
                &d,
                &b,
                1,
                1,
                &d,
                beta,
                &mut c,
                1,
                1,
                &d,
            );
            let (lr, lc) = lay.local_dims(N, N);
            for lj in 0..lc {
                for li in 0..lr {
                    let (i, j) = (lay.global_row(li), lay.global_col(lj));
                    let ab: f64 = (0..N)
                        .map(|k| {
                            (if trans == Trans::T {
                                a_re(k, i)
                            } else {
                                a_re(i, k)
                            }) * b_f(k, j)
                        })
                        .sum();
                    let want = alpha * ab + beta * c0(i, j);
                    assert!(
                        (c[lj * lld + li] - want).abs() < 1e-9,
                        "{tag}: pdgemm {trans:?} C[{i},{j}] wrong"
                    );
                }
            }
        }
    }

    // ---- pdgetrf + pdgetrs with NRHS = 3, A x = b and A^T x = b ----
    for trans in [Trans::No, Trans::T] {
        let rhs = |i: usize, r: usize| {
            (0..N)
                .map(|k| {
                    (if trans == Trans::T {
                        a_re(k, i)
                    } else {
                        a_re(i, k)
                    }) * x_re(k, r)
                })
                .sum::<f64>()
        };
        let (mut a, lld_a) = lay.local(N, N, &a_re);
        let (mut b, lld_b) = lay.local(N, NRHS, &rhs);
        let (da, db) = (desc(N, N, lld_a), desc(N, NRHS, lld_b));
        let mut ipiv = vec![0 as Int; lay.local_dims(N, N).0 + NB];
        pdgetrf(N, N, &mut a, 1, 1, &da, &mut ipiv).expect("pdgetrf");
        pdgetrs(trans, N, NRHS, &a, 1, 1, &da, &ipiv, &mut b, 1, 1, &db).expect("pdgetrs");
        let (lr, lc) = lay.local_dims(N, NRHS);
        for lj in 0..lc {
            for li in 0..lr {
                let (i, r) = (lay.global_row(li), lay.global_col(lj));
                assert!(
                    (b[lj * lld_b + li] - x_re(i, r)).abs() < 1e-9,
                    "{tag}: pdgetrs {trans:?} x[{i},{r}] = {}, want {}",
                    b[lj * lld_b + li],
                    x_re(i, r)
                );
            }
        }
    }

    // ---- pdpotrf + pdpotrs, each triangle (unused one holds 999), NRHS = 3 ----
    for uplo in [Uplo::Lower, Uplo::Upper] {
        let keep = move |i: usize, j: usize| match uplo {
            Uplo::Lower => i >= j,
            Uplo::Upper => i <= j,
        };
        let (mut a, lld_a) = lay.local(N, N, &|i, j| if keep(i, j) { spd(i, j) } else { 999.0 });
        let rhs = |i: usize, r: usize| (0..N).map(|k| spd(i, k) * x_re(k, r)).sum::<f64>();
        let (mut b, lld_b) = lay.local(N, NRHS, &rhs);
        let (da, db) = (desc(N, N, lld_a), desc(N, NRHS, lld_b));
        pdpotrf(uplo, N, &mut a, 1, 1, &da).expect("pdpotrf");
        pdpotrs(uplo, N, NRHS, &a, 1, 1, &da, &mut b, 1, 1, &db).expect("pdpotrs");
        let (lr, lc) = lay.local_dims(N, NRHS);
        for lj in 0..lc {
            for li in 0..lr {
                let (i, r) = (lay.global_row(li), lay.global_col(lj));
                assert!(
                    (b[lj * lld_b + li] - x_re(i, r)).abs() < 1e-9,
                    "{tag}: pdpotrs {uplo:?} x[{i},{r}] wrong"
                );
            }
        }
    }

    // ---- complex solves with NRHS = 3 ----
    macro_rules! complex {
        ($name:literal, $solve:ident, $t:ty, $tol:expr) => {{
            let (mut a, lld_a) = lay.local(N, N, &|i, j| [a_re(i, j) as $t, a_im(i, j) as $t]);
            let rhs = |i: usize, r: usize| {
                let (mut re, mut im) = (0.0_f64, 0.0_f64);
                for k in 0..N {
                    re += a_re(i, k) * x_re(k, r) - a_im(i, k) * x_im(k, r);
                    im += a_re(i, k) * x_im(k, r) + a_im(i, k) * x_re(k, r);
                }
                [re as $t, im as $t]
            };
            let (mut b, lld_b) = lay.local(N, NRHS, &rhs);
            let (da, db) = (desc(N, N, lld_a), desc(N, NRHS, lld_b));
            let mut ipiv = vec![0 as Int; lay.local_dims(N, N).0 + NB];
            $solve(N, NRHS, &mut a, 1, 1, &da, &mut ipiv, &mut b, 1, 1, &db).expect($name);
            let (lr, lc) = lay.local_dims(N, NRHS);
            for lj in 0..lc {
                for li in 0..lr {
                    let (i, r) = (lay.global_row(li), lay.global_col(lj));
                    let g = b[lj * lld_b + li];
                    assert!(
                        (g[0] as f64 - x_re(i, r)).abs() < $tol
                            && (g[1] as f64 - x_im(i, r)).abs() < $tol,
                        "{tag}: {} x[{i},{r}] = {g:?}, want ({}, {})",
                        $name,
                        x_re(i, r),
                        x_im(i, r)
                    );
                }
            }
        }};
    }
    complex!("pzgesv", pzgesv, f64, 1e-9);
    complex!("pcgesv", pcgesv, f32, 1e-3);

    drop(grid);
    blacs_exit(false);
    println!("grid2d_mpi: rank {mypnum} of {nprocs} ({pr}x{pc}) ok");
}
