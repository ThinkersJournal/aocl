//! Distributed solve `A x = b` through `Grid`, `BlockCyclic` and `pdgesv`, on however many MPI ranks launched it.
//!
//! `harness = false`: BLACS initialises MPI (`Cblacs_pinfo`) and tears it down (`blacs_exit`) once per process, which
//! the libtest thread-per-test harness cannot give. Run it as `cargo test -p aocl-scalapack --test pdgesv_mpi`
//! (one rank) or `mpiexec -n 2 <path to the test exe>` (two ranks, columns dealt cyclically). The data is
//! asymmetric (`A != A^T`, columns of different weight) so a transposed or mis-dealt matrix gives a wrong `x`.

use aocl_scalapack::{blacs_exit, numroc, pdgesv, BlockCyclic, Grid};

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
    let mut ipiv = vec![0_i32; N + NB];
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
    drop(grid);
    blacs_exit(false);
    println!("pdgesv_mpi: rank {mypnum} of {nprocs} ok");
}
