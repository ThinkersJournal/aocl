//! Distributed solve `A x = b` through `Grid`, `BlockCyclic` and `pdgesv`, on however many MPI ranks launched it.
//!
//! `harness = false`: BLACS initialises MPI (`Cblacs_pinfo`) and tears it down (`blacs_exit`) once per process, which
//! the libtest thread-per-test harness cannot give. Run it as `cargo test -p aocl-scalapack --test pdgesv_mpi`
//! (one rank) or `mpiexec -n 2 <path to the test exe>` (two ranks, columns dealt cyclically). The data is
//! asymmetric (`A != A^T`, columns of different weight) so a transposed or mis-dealt matrix gives a wrong `x`.
//! The same process then checks `pdgemm` (`C = 2 A B + 0.5 C0` and `C = A^T B`) against a serial product, `pdgetrf` +
//! `pdgetrs` (`A x = b` and `A^T x = b`), and `pdpotrf` + `pdpotrs` for both triangles (the unused triangle holds 999,
//! so reading the wrong one gives a wrong `x`), `pdsyev` (eigenvalues ascending, trace and Frobenius invariants, and
//! unit norm, orthogonality and `A z = w z` for every eigenvector this process owns) and `pdgels` (a consistent 8 x 4 system with a known solution), and `pdgeqrf` (|R| against a serial Householder QR).

use aocl_scalapack::{
    blacs_exit, numroc, pdgels, pdgemm, pdgeqrf, pdgesv, pdgetrf, pdgetrs, pdpotrf, pdpotrs,
    pdsyev, BlockCyclic, EigCompute, Grid, Int, Trans, Uplo,
};

const N: usize = 6;
const NB: usize = 2;

fn a_global(i: usize, j: usize) -> f64 {
    if i == j {
        10.0 + i as f64
    } else {
        i as f64 - 2.0 * j as f64 + 0.5
    }
}

fn main() {
    let (mut mypnum, mut nprocs) = (-1, 0);
    unsafe { aocl_scalapack_sys::Cblacs_pinfo(&mut mypnum, &mut nprocs) };
    let p = nprocs as usize;
    // 1 x p grid: every process owns all rows; columns are dealt in NB-wide blocks, cyclically.
    let grid = Grid::new(1, p).expect("grid");
    let (_, mycol) = grid.coords();

    let x_true: Vec<f64> = (0..N)
        .map(|i| [1.0, -2.0, 3.0, 0.5, -1.5, 2.5][i])
        .collect();
    let b_global: Vec<f64> = (0..N)
        .map(|i| (0..N).map(|j| a_global(i, j) * x_true[j]).sum())
        .collect();

    // Local columns of A: global column j lives on process (j / NB) % p at local index (j / NB / p) * NB + j % NB.
    let loc_cols = numroc(N, NB, mycol, 0, p);
    let mut a = vec![0.0_f64; N * loc_cols.max(1)];
    for j in 0..N {
        if (j / NB) % p == mycol {
            let lj = (j / NB / p) * NB + j % NB;
            for i in 0..N {
                a[lj * N + i] = a_global(i, j);
            }
        }
    }
    // B is N x 1 in NB x 1 blocks: its single column lives on process column 0.
    let mut b = vec![0.0_f64; N];
    if mycol == 0 {
        b.copy_from_slice(&b_global);
    }
    let desca = BlockCyclic::new(&grid, N, N, NB, NB, 0, 0, N).expect("desca");
    let descb = BlockCyclic::new(&grid, N, 1, NB, 1, 0, 0, N).expect("descb");
    let mut ipiv = vec![0 as Int; N + NB];
    pdgesv(N, 1, &mut a, 1, 1, &desca, &mut ipiv, &mut b, 1, 1, &descb).expect("pdgesv");

    if mycol == 0 {
        for i in 0..N {
            assert!(
                (b[i] - x_true[i]).abs() < 1e-9,
                "rank {mypnum}/{nprocs}: x[{i}] = {} but expected {}",
                b[i],
                x_true[i]
            );
        }
    }
    // ---- pdgemm: N x N matrices, all dealt by columns like A above ----
    let b_global = |i: usize, j: usize| i as f64 * 0.5 - j as f64 + 1.0;
    let c0_global = |i: usize, j: usize| i as f64 + j as f64 * 0.25;
    // Global column of local column `lj` on this process.
    let global_col = |lj: usize| ((lj / NB) * p + mycol) * NB + lj % NB;
    let local = |f: &dyn Fn(usize, usize) -> f64| -> Vec<f64> {
        let mut v = vec![0.0_f64; N * loc_cols.max(1)];
        for lj in 0..loc_cols {
            for i in 0..N {
                v[lj * N + i] = f(i, global_col(lj));
            }
        }
        v
    };
    let a_loc = local(&a_global);
    let b_loc = local(&b_global);
    let desc = BlockCyclic::new(&grid, N, N, NB, NB, 0, 0, N).expect("desc");
    let serial = |trans_a: bool, alpha: f64, beta: f64, i: usize, j: usize| -> f64 {
        let ab: f64 = (0..N)
            .map(|k| {
                let a = if trans_a {
                    a_global(k, i)
                } else {
                    a_global(i, k)
                };
                a * b_global(k, j)
            })
            .sum();
        alpha * ab + beta * c0_global(i, j)
    };
    for (trans, alpha, beta) in [(Trans::No, 2.0, 0.5), (Trans::T, 1.0, 0.0)] {
        let mut c_loc = local(&c0_global);
        pdgemm(
            trans,
            Trans::No,
            N,
            N,
            N,
            alpha,
            &a_loc,
            1,
            1,
            &desc,
            &b_loc,
            1,
            1,
            &desc,
            beta,
            &mut c_loc,
            1,
            1,
            &desc,
        );
        for lj in 0..loc_cols {
            for i in 0..N {
                let want = serial(trans == Trans::T, alpha, beta, i, global_col(lj));
                let got = c_loc[lj * N + i];
                assert!(
                    (got - want).abs() < 1e-9,
                    "rank {mypnum}/{nprocs}: pdgemm {trans:?} C[{i},{}] = {got}, want {want}",
                    global_col(lj)
                );
            }
        }
    }

    // ---- pdgetrf + pdgetrs: A x = b and A^T x = b ----
    for trans in [Trans::No, Trans::T] {
        let rhs: Vec<f64> = (0..N)
            .map(|i| {
                (0..N)
                    .map(|j| {
                        let a = if trans == Trans::T {
                            a_global(j, i)
                        } else {
                            a_global(i, j)
                        };
                        a * x_true[j]
                    })
                    .sum()
            })
            .collect();
        let mut lu = local(&a_global);
        let mut piv = vec![0 as Int; N + NB];
        pdgetrf(N, N, &mut lu, 1, 1, &desca, &mut piv).expect("pdgetrf");
        let mut x = vec![0.0_f64; N];
        if mycol == 0 {
            x.copy_from_slice(&rhs);
        }
        pdgetrs(trans, N, 1, &lu, 1, 1, &desca, &piv, &mut x, 1, 1, &descb).expect("pdgetrs");
        if mycol == 0 {
            for i in 0..N {
                assert!(
                    (x[i] - x_true[i]).abs() < 1e-9,
                    "rank {mypnum}/{nprocs}: pdgetrs {trans:?} x[{i}] = {} want {}",
                    x[i],
                    x_true[i]
                );
            }
        }
    }

    // ---- pdpotrf + pdpotrs on a symmetric positive definite matrix, each triangle ----
    let spd = |i: usize, j: usize| {
        if i == j {
            8.0 + i as f64
        } else {
            1.0 / (1.0 + (i as f64 - j as f64).abs())
        }
    };
    let spd_rhs: Vec<f64> = (0..N)
        .map(|i| (0..N).map(|j| spd(i, j) * x_true[j]).sum())
        .collect();
    for uplo in [Uplo::Lower, Uplo::Upper] {
        let mut f = local(&|i, j| {
            let keep = match uplo {
                Uplo::Lower => i >= j,
                Uplo::Upper => i <= j,
            };
            if keep {
                spd(i, j)
            } else {
                999.0
            }
        });
        pdpotrf(uplo, N, &mut f, 1, 1, &desca).expect("pdpotrf");
        let mut x = vec![0.0_f64; N];
        if mycol == 0 {
            x.copy_from_slice(&spd_rhs);
        }
        pdpotrs(uplo, N, 1, &f, 1, 1, &desca, &mut x, 1, 1, &descb).expect("pdpotrs");
        if mycol == 0 {
            for i in 0..N {
                assert!(
                    (x[i] - x_true[i]).abs() < 1e-9,
                    "rank {mypnum}/{nprocs}: pdpotrs {uplo:?} x[{i}] = {} want {}",
                    x[i],
                    x_true[i]
                );
            }
        }
    }

    // ---- pdsyev: symmetric eigenproblem, each triangle (the unused one holds 999) ----
    let trace: f64 = (0..N).map(|i| spd(i, i)).sum();
    let frob2: f64 = (0..N)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .map(|(i, j)| spd(i, j).powi(2))
        .sum();
    for uplo in [Uplo::Lower, Uplo::Upper] {
        let mut m = local(&|i, j| {
            let keep = match uplo {
                Uplo::Lower => i >= j,
                Uplo::Upper => i <= j,
            };
            if keep {
                spd(i, j)
            } else {
                999.0
            }
        });
        let mut w = vec![0.0_f64; N];
        let mut z = vec![0.0_f64; N * loc_cols.max(1)];
        let mut work = vec![0.0_f64; 1];
        // Workspace query (lwork = -1), then the real call.
        pdsyev(
            EigCompute::ValuesAndVectors,
            uplo,
            N,
            &mut m,
            1,
            1,
            &desc,
            &mut w,
            &mut z,
            1,
            1,
            &desc,
            &mut work,
            -1,
        )
        .expect("pdsyev workspace query");
        let lwork = work[0] as usize + 16;
        let mut work = vec![0.0_f64; lwork];
        pdsyev(
            EigCompute::ValuesAndVectors,
            uplo,
            N,
            &mut m,
            1,
            1,
            &desc,
            &mut w,
            &mut z,
            1,
            1,
            &desc,
            &mut work,
            lwork as Int,
        )
        .expect("pdsyev");
        assert!(
            w.windows(2).all(|p| p[0] <= p[1]),
            "{uplo:?}: eigenvalues not ascending: {w:?}"
        );
        let sum: f64 = w.iter().sum();
        let sum2: f64 = w.iter().map(|v| v * v).sum();
        assert!(
            (sum - trace).abs() < 1e-9,
            "{uplo:?}: sum of eigenvalues {sum} != trace {trace}"
        );
        assert!(
            (sum2 - frob2).abs() < 1e-8,
            "{uplo:?}: sum of squares {sum2} != Frobenius^2 {frob2}"
        );
        // A zero Z satisfies A z = w z trivially, so first require unit norm, and orthogonality between this
        // process's eigenvectors (each is a whole local column, since the grid has one process row).
        for lj in 0..loc_cols {
            let norm2: f64 = (0..N).map(|k| z[lj * N + k].powi(2)).sum();
            assert!(
                (norm2 - 1.0).abs() < 1e-9,
                "{uplo:?}: |z|^2 = {norm2} for local eigenvector {lj}"
            );
            for lk in 0..lj {
                let dot: f64 = (0..N).map(|k| z[lj * N + k] * z[lk * N + k]).sum();
                assert!(
                    dot.abs() < 1e-9,
                    "{uplo:?}: local eigenvectors {lk} and {lj} not orthogonal: {dot}"
                );
            }
        }
        // Each eigenvector (a whole local column, since the grid has one process row) satisfies A z = w z.
        for lj in 0..loc_cols {
            let gj = global_col(lj);
            for i in 0..N {
                let az: f64 = (0..N).map(|k| spd(i, k) * z[lj * N + k]).sum();
                let wz = w[gj] * z[lj * N + i];
                assert!(
                    (az - wz).abs() < 1e-9,
                    "rank {mypnum}/{nprocs}: pdsyev {uplo:?} (A z - w z)[{i}] for eigenvector {gj} = {}",
                    az - wz
                );
            }
        }
    }

    // ---- pdgels: min |A x - b| for a consistent 8 x 4 system, so x must equal x_true ----
    {
        const M: usize = 8;
        const NG: usize = 4;
        let ga = |i: usize, j: usize| {
            (i as f64 + 1.0) * (j as f64 + 2.0).sqrt() + if i == j { 5.0 } else { -0.5 * j as f64 }
        };
        let loc_g = numroc(NG, NB, mycol, 0, p);
        let mut ag = vec![0.0_f64; M * loc_g.max(1)];
        for lj in 0..loc_g {
            let gj = ((lj / NB) * p + mycol) * NB + lj % NB;
            for i in 0..M {
                ag[lj * M + i] = ga(i, gj);
            }
        }
        let xg = [1.0_f64, -2.0, 0.5, 3.0];
        let mut bg = vec![0.0_f64; M];
        if mycol == 0 {
            for i in 0..M {
                bg[i] = (0..NG).map(|j| ga(i, j) * xg[j]).sum();
            }
        }
        let da = BlockCyclic::new(&grid, M, NG, NB, NB, 0, 0, M).expect("gels desca");
        let db = BlockCyclic::new(&grid, M, 1, NB, 1, 0, 0, M).expect("gels descb");
        let mut work = vec![0.0_f64; 1];
        pdgels(
            Trans::No,
            M,
            NG,
            1,
            &mut ag,
            1,
            1,
            &da,
            &mut bg,
            1,
            1,
            &db,
            &mut work,
            -1,
        )
        .expect("pdgels workspace query");
        let lwork = work[0] as usize + 16;
        let mut work = vec![0.0_f64; lwork];
        pdgels(
            Trans::No,
            M,
            NG,
            1,
            &mut ag,
            1,
            1,
            &da,
            &mut bg,
            1,
            1,
            &db,
            &mut work,
            lwork as Int,
        )
        .expect("pdgels");
        if mycol == 0 {
            for j in 0..NG {
                assert!(
                    (bg[j] - xg[j]).abs() < 1e-9,
                    "rank {mypnum}/{nprocs}: pdgels x[{j}] = {} want {}",
                    bg[j],
                    xg[j]
                );
            }
        }
    }

    // ---- pdgeqrf: |R| of an 8 x 4 matrix against a serial Householder QR (signs may differ) ----
    {
        const M: usize = 8;
        const NG: usize = 4;
        let ga = |i: usize, j: usize| {
            (i as f64 + 1.0) * (j as f64 + 2.0).sqrt() + if i == j { 5.0 } else { -0.5 * j as f64 }
        };
        // Serial QR, column-major: r[j * M + i].
        let mut r: Vec<f64> = (0..NG)
            .flat_map(|j| (0..M).map(move |i| (i, j)))
            .map(|(i, j)| ga(i, j))
            .collect();
        for k in 0..NG {
            let norm = (k..M).map(|i| r[k * M + i].powi(2)).sum::<f64>().sqrt();
            let alpha = if r[k * M + k] > 0.0 { -norm } else { norm };
            let mut v: Vec<f64> = (k..M).map(|i| r[k * M + i]).collect();
            v[0] -= alpha;
            let vnorm2: f64 = v.iter().map(|x| x * x).sum();
            for j in k..NG {
                let dot: f64 = (k..M).map(|i| v[i - k] * r[j * M + i]).sum();
                for i in k..M {
                    r[j * M + i] -= 2.0 * dot / vnorm2 * v[i - k];
                }
            }
        }
        let loc_g = numroc(NG, NB, mycol, 0, p);
        let mut ag = vec![0.0_f64; M * loc_g.max(1)];
        for lj in 0..loc_g {
            let gj = ((lj / NB) * p + mycol) * NB + lj % NB;
            for i in 0..M {
                ag[lj * M + i] = ga(i, gj);
            }
        }
        let da = BlockCyclic::new(&grid, M, NG, NB, NB, 0, 0, M).expect("qr desca");
        let mut tau = vec![0.0_f64; loc_g.max(1) + NB];
        let mut work = vec![0.0_f64; 1];
        pdgeqrf(M, NG, &mut ag, 1, 1, &da, &mut tau, &mut work, -1)
            .expect("pdgeqrf workspace query");
        let lwork = work[0] as usize + 16;
        let mut work = vec![0.0_f64; lwork];
        pdgeqrf(M, NG, &mut ag, 1, 1, &da, &mut tau, &mut work, lwork as Int).expect("pdgeqrf");
        for lj in 0..loc_g {
            let gj = ((lj / NB) * p + mycol) * NB + lj % NB;
            for i in 0..=gj {
                let (got, want) = (ag[lj * M + i].abs(), r[gj * M + i].abs());
                assert!(
                    (got - want).abs() < 1e-9,
                    "rank {mypnum}/{nprocs}: pdgeqrf |R[{i},{gj}]| = {got}, serial QR says {want}"
                );
            }
        }
    }

    drop(grid);
    blacs_exit(false);
    println!("pdgesv_mpi: rank {mypnum} of {nprocs} ok");
}
