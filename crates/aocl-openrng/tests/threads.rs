//! Separate streams used from several threads at once (each thread makes its own: `Stream` is `!Send`). The wrapper serialises every
//! library call behind one lock; this test records that the results equal the single-threaded ones, so a future change that drops the
//! lock has a test to answer to. Compiled only with OPENRNG_LIB_DIR; PM-run. It does NOT measure whether the library alone is thread-safe.
#![cfg(openrng_linked)]
use aocl_openrng::*;

fn run(seed: u32) -> Vec<u32> {
    let mut s = Stream::new(Brng::Philox4x32x10, seed).unwrap();
    let mut out = Vec::new();
    for _ in 0..200 {
        let mut w = [0u32; 16];
        s.uniform_bits32(&mut w).unwrap();
        out.extend_from_slice(&w);
        let mut g = [0f64; 16];
        let _ = s.gaussian(&mut g, 0.0, 1.0, GaussianMethod::BoxMuller2);
    }
    out
}

#[test]
fn concurrent_separate_streams_match_sequential() {
    let seq: Vec<Vec<u32>> = (0..8).map(run).collect();
    let par: Vec<Vec<u32>> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..8u32).map(|i| sc.spawn(move || run(i))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(seq, par);
}
