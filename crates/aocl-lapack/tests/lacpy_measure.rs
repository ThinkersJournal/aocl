//! MEASUREMENT of `LAPACKE_?lacpy` (d, s, c, z) x {row-major, col-major} x uplo {U, L, A}, through the raw `-sys` entry
//! points. B is pre-filled with a sentinel (77); after the call each variant records which entries of B changed OUTSIDE the
//! documented region (the triangle for U/L; for A the whole block). The table is printed (`cargo test -p aocl-lapack --test
//! lacpy_measure -- --nocapture`) and is the evidence in docs/LACPY-MEASUREMENT.md. The test asserts only the CONTRACT:
//! the referenced region is copied exactly and the padding beyond the m x n block is untouched. What happens to the
//! unreferenced triangle is recorded, never asserted (LAPACK leaves it "not referenced"; AOCL 6.0.68 row-major clobbers it).
use aocl_lapack_sys as sys;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
struct Cx<T> {
    re: T,
    im: T,
}

const ROW: i32 = 101;
const COL: i32 = 102;
const M: usize = 3;
const N: usize = 4;
const LD: usize = 5;

struct Row {
    variant: &'static str,
    layout: &'static str,
    uplo: char,
    clobbered: Vec<(usize, usize)>,
    contract_ok: bool,
}

macro_rules! measure {
    ($rows:ident, $name:literal, $f:path, $e:ty, $mk:expr) => {{
        let mk = $mk;
        for (layout, lname) in [(ROW, "row-major"), (COL, "col-major")] {
            for uplo in ['U', 'L', 'A'] {
                // logical (i,j) -> storage index; leading dimension LD both sides
                let at = |i: usize, j: usize| if layout == ROW { i * LD + j } else { j * LD + i };
                let len = LD * if layout == ROW { M } else { N };
                let mut a = vec![mk(-1.0); len]; // padding of A holds garbage that must never be copied
                for i in 0..M {
                    for j in 0..N {
                        a[at(i, j)] = mk((10 * (i + 1) + j + 1) as f64);
                    }
                }
                let sentinel = mk(77.0);
                let mut b = vec![sentinel; len];
                let info = unsafe {
                    $f(layout, uplo as i8, M as _, N as _, a.as_ptr() as *const _, LD as _, b.as_mut_ptr() as *mut _, LD as _)
                };
                assert_eq!(info, 0, "{} {lname} {uplo}: info", $name);
                let mut contract_ok = true;
                let mut clobbered = vec![];
                let mut in_block = vec![false; len];
                for i in 0..M {
                    for j in 0..N {
                        in_block[at(i, j)] = true;
                        let referenced = match uplo {
                            'U' => j >= i,
                            'L' => j <= i,
                            _ => true,
                        };
                        if referenced {
                            contract_ok &= b[at(i, j)] == a[at(i, j)];
                        } else if b[at(i, j)] != sentinel {
                            clobbered.push((i, j));
                        }
                    }
                }
                for k in 0..len {
                    if !in_block[k] {
                        contract_ok &= b[k] == sentinel; // padding never touched
                    }
                }
                $rows.push(Row { variant: $name, layout: lname, uplo, clobbered, contract_ok });
            }
        }
    }};
}

#[test]
fn lacpy_table_and_contract() {
    let mut rows: Vec<Row> = vec![];
    measure!(rows, "dlacpy", sys::LAPACKE_dlacpy, f64, |v: f64| v);
    measure!(rows, "slacpy", sys::LAPACKE_slacpy, f32, |v: f64| v as f32);
    measure!(rows, "zlacpy", sys::LAPACKE_zlacpy, Cx<f64>, |v: f64| Cx {
        re: v,
        im: -v
    });
    measure!(rows, "clacpy", sys::LAPACKE_clacpy, Cx<f32>, |v: f64| Cx {
        re: v as f32,
        im: -(v as f32)
    });
    println!("variant  layout     uplo  referenced-region-and-padding-ok  entries-outside-the-triangle-changed");
    for r in &rows {
        println!(
            "{:7}  {:9}  {}     {:5}                             {} {:?}",
            r.variant,
            r.layout,
            r.uplo,
            r.contract_ok,
            r.clobbered.len(),
            r.clobbered
        );
    }
    // The contract (asserted): every variant copies the referenced region exactly and leaves the padding alone.
    for r in &rows {
        assert!(
            r.contract_ok,
            "{} {} {}: referenced region or padding wrong",
            r.variant, r.layout, r.uplo
        );
    }
}
