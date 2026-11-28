#![cfg(windows)]
use aocl_sparse_sys as sys;
type I = i32;
fn run(m: usize, n: usize, blk: i32) {
    let (mut rp, mut ci, mut v) = (vec![0 as I], vec![], vec![]);
    for r in 0..m {
        for j in 0..n {
            if (r * 7 + j * 3) % 5 == 0 {
                ci.push(j as I);
                v.push(1.0 + (r * n + j) as f64);
            }
        }
        rp.push(ci.len() as I);
    }
    let nnz = v.len();
    let (mut brp, mut bci, mut bv, mut masks) = (vec![-7 as I; 256], vec![-7 as I; 256], vec![-7.0f64; 256], vec![255u8; 256]);
    let st = unsafe { sys::aoclsparse_csr2blkcsr(m as I, n as I, nnz as I, rp.as_ptr(), ci.as_ptr(), v.as_ptr(), brp.as_mut_ptr(), bci.as_mut_ptr(), bv.as_mut_ptr(), masks.as_mut_ptr(), blk, sys::aoclsparse_index_base__aoclsparse_index_base_zero) };
    let nbr = brp.iter().position(|&x| x == -7).unwrap();
    let nb = bci.iter().position(|&x| x == -7).unwrap();
    let nm = masks.iter().position(|&x| x == 255 && false).unwrap_or(0);
    let pop: u32 = masks.iter().take(64).map(|b| b.count_ones()).sum();
    println!("m={m} n={n} blk={blk} st={st} nnz={nnz} brp_written={nbr} {:?} bci_written={nb} {:?} popcount(first64 masks incl untouched)={pop} {:?}", &brp[..nbr.min(12)], &bci[..nb.min(12)], &masks[..12.min(masks.len())]);
}
#[test]
fn probe() {
    run(4, 8, 4);
    run(8, 16, 4);
    run(8, 16, 8);
    run(5, 11, 4);
    run(12, 20, 4);
}
