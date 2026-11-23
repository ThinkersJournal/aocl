//! Complex distributed solves `pcgesv` (single) and `pzgesv` (double) on however many MPI ranks launched this
//! (`harness = false`, same layout and launch as `pdgesv_mpi.rs`). `A` is complex and neither symmetric nor Hermitian,
//! `x` has distinct real and imaginary parts, and `b = A x` is formed with a serial complex product, so a conjugated,
//! transposed or mis-dealt matrix, or real and imaginary parts swapped, gives a wrong `x`.

use aocl_scalapack::{blacs_exit, numroc, pcgesv, pzgesv, BlockCyclic, Grid, Int};

const N: usize = 6;
const NB: usize = 2;

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
fn x_true(i: usize) -> (f64, f64) {
    ((i + 1) as f64, -0.5 * i as f64 + 0.25)
}

macro_rules! check {
    ($name:literal, $solve:ident, $t:ty, $tol:expr, $grid:expr, $mycol:expr, $p:expr, $loc_cols:expr, $rank:expr) => {{
        let (grid, mycol, p, loc_cols) = ($grid, $mycol, $p, $loc_cols);
        let mut a = vec![[0.0 as $t; 2]; N * loc_cols.max(1)];
        for j in 0..N {
            if (j / NB) % p == mycol {
                let lj = (j / NB / p) * NB + j % NB;
                for i in 0..N {
                    a[lj * N + i] = [a_re(i, j) as $t, a_im(i, j) as $t];
                }
            }
        }
        let mut b = vec![[0.0 as $t; 2]; N];
        if mycol == 0 {
            for i in 0..N {
                let (mut re, mut im) = (0.0_f64, 0.0_f64);
                for j in 0..N {
                    let (xr, xi) = x_true(j);
                    re += a_re(i, j) * xr - a_im(i, j) * xi;
                    im += a_re(i, j) * xi + a_im(i, j) * xr;
                }
                b[i] = [re as $t, im as $t];
            }
        }
        let desca = BlockCyclic::new(grid, N, N, NB, NB, 0, 0, N).expect("desca");
        let descb = BlockCyclic::new(grid, N, 1, NB, 1, 0, 0, N).expect("descb");
        let mut ipiv = vec![0 as Int; N + NB];
        $solve(N, 1, &mut a, 1, 1, &desca, &mut ipiv, &mut b, 1, 1, &descb).expect($name);
        if mycol == 0 {
            for i in 0..N {
                let (xr, xi) = x_true(i);
                assert!(
                    (b[i][0] as f64 - xr).abs() < $tol && (b[i][1] as f64 - xi).abs() < $tol,
                    "rank {}: {} x[{i}] = {:?}, want ({xr}, {xi})",
                    $rank,
                    $name,
                    b[i]
                );
            }
        }
    }};
}

fn main() {
    let (mut mypnum, mut nprocs) = (-1, 0);
    unsafe { aocl_scalapack_sys::Cblacs_pinfo(&mut mypnum, &mut nprocs) };
    let p = nprocs as usize;
    let grid = Grid::new(1, p).expect("grid");
    let (_, mycol) = grid.coords();
    let loc_cols = numroc(N, NB, mycol, 0, p);
    check!("pzgesv", pzgesv, f64, 1e-9, &grid, mycol, p, loc_cols, mypnum);
    check!("pcgesv", pcgesv, f32, 1e-3, &grid, mycol, p, loc_cols, mypnum);
    drop(grid);
    blacs_exit(false);
    println!("complex_mpi: rank {mypnum} of {nprocs} ok");
}
