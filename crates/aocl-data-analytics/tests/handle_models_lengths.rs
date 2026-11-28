//! Guard-page harness (19 of these 51 short cases were access violations before the checks) for the handle-based models of aocl-data-analytics (KMeans, Pca, KNearestNeighbours, Linmod, Dbscan, DecisionTree,
//! DecisionForest, Svm). Hand-written from a table. Each case fits the model on valid data where the call needs one, then calls ONE method
//! with ONE input or output buffer ONE ELEMENT short at the end of a committed page before a PAGE_NOACCESS page: a fault = HOLE. One process
//! per test (tools/safe-api-harness.py); a control (right sizes) must succeed. Windows only; Linux unmeasured.
#![cfg(windows)]
use aocl_data_analytics::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
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
macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let r = $body.map(|_| ());
            assert!(
                matches!(r, Err(aocl_error::Error::InvalidArgument(_))),
                "{}: {:?}",
                stringify!($name),
                r
            );
        }
    };
}
macro_rules! control {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            $body.unwrap();
        }
    };
}
const X: [f64; 12] = [
    0.0, 0.1, 0.2, 10.0, 10.1, 10.2, 0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
]; // 6 samples x 2 features, column-major
const TEST: [f64; 4] = [0.05, 9.95, 0.05, 10.05]; // 2 queries x 2 features
const YI: [i32; 6] = [0, 0, 0, 1, 1, 1];
const YF: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
const LBL: [i32; 6] = [0; 6];
const OUTF: [f64; 12] = [0.0; 12];
control!(kmeans_control, {
    let mut m = KMeans::new(2).unwrap();
    m.fit(6, 2, &X).unwrap();
    let mut l = LBL;
    m.predict(2, 2, &TEST, &mut l)
});
case!(
    kmeans_fit_short_data,
    at_guard(&X, 11, |d| {
        let mut m = KMeans::new(2).unwrap();
        m.fit(6, 2, d)
    })
);
case!(kmeans_predict_short_y, {
    let mut m = KMeans::new(2).unwrap();
    m.fit(6, 2, &X).unwrap();
    let mut l = LBL;
    at_guard(&TEST, 3, |y| m.predict(2, 2, y, &mut l))
});
case!(kmeans_predict_short_labels, {
    let mut m = KMeans::new(2).unwrap();
    m.fit(6, 2, &X).unwrap();
    at_guard(&LBL, 1, |l| m.predict(2, 2, &TEST, l))
});
control!(pca_control, {
    let mut m = Pca::new(1).unwrap();
    m.fit(6, 2, &X).unwrap();
    let mut o = OUTF;
    m.transform(2, 2, &TEST, 1, &mut o)
});
case!(
    pca_fit_short_data,
    at_guard(&X, 11, |d| {
        let mut m = Pca::new(1).unwrap();
        m.fit(6, 2, d)
    })
);
case!(pca_transform_short_x, {
    let mut m = Pca::new(1).unwrap();
    m.fit(6, 2, &X).unwrap();
    let mut o = OUTF;
    at_guard(&TEST, 3, |x| m.transform(2, 2, x, 1, &mut o))
});
case!(pca_transform_short_out, {
    let mut m = Pca::new(1).unwrap();
    m.fit(6, 2, &X).unwrap();
    at_guard(&OUTF, 1, |o| m.transform(2, 2, &TEST, 1, o))
});
control!(knn_control, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    let mut l = LBL;
    m.predict(2, 2, &TEST, &mut l)
});
case!(
    knn_fit_short_x,
    at_guard(&X, 11, |x| {
        let mut m = KNearestNeighbours::new().unwrap();
        m.fit(6, 2, x, &YI)
    })
);
case!(
    knn_fit_short_y,
    at_guard(&YI, 5, |y| {
        let mut m = KNearestNeighbours::new().unwrap();
        m.fit(6, 2, &X, y)
    })
);
case!(knn_predict_short_x, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    let mut l = LBL;
    at_guard(&TEST, 3, |x| m.predict(2, 2, x, &mut l))
});
case!(knn_predict_short_labels, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    at_guard(&LBL, 1, |l| m.predict(2, 2, &TEST, l))
});
case!(knn_kneighbors_short_x, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    let (mut i, mut d) = (LBL, OUTF);
    at_guard(&TEST, 3, |x| m.kneighbors(2, 2, x, &mut i, &mut d, 3, true))
});
case!(knn_kneighbors_short_ind, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    let mut d = OUTF;
    at_guard(&LBL, 5, |i| m.kneighbors(2, 2, &TEST, i, &mut d, 3, true))
});
case!(knn_kneighbors_short_dist, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    let mut i = LBL;
    at_guard(&OUTF, 5, |d| m.kneighbors(2, 2, &TEST, &mut i, d, 3, true))
});
case!(knn_predict_proba_short_x, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict_proba(2, 2, x, &mut p))
});
case!(knn_predict_proba_short_proba, {
    let mut m = KNearestNeighbours::new().unwrap();
    m.fit(6, 2, &X, &YI).unwrap();
    at_guard(&OUTF, 3, |p| m.predict_proba(2, 2, &TEST, p))
});
control!(linmod_control, {
    let mut m = Linmod::new_mse().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut p = OUTF;
    m.predict(2, 2, &TEST, &mut p)
});
case!(
    linmod_fit_short_x,
    at_guard(&X, 11, |x| {
        let mut m = Linmod::new_mse().unwrap();
        m.fit(6, 2, x, &YF)
    })
);
case!(
    linmod_fit_short_y,
    at_guard(&YF, 5, |y| {
        let mut m = Linmod::new_mse().unwrap();
        m.fit(6, 2, &X, y)
    })
);
case!(linmod_predict_short_x, {
    let mut m = Linmod::new_mse().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict(2, 2, x, &mut p))
});
case!(linmod_predict_short_out, {
    let mut m = Linmod::new_mse().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    at_guard(&OUTF, 1, |p| m.predict(2, 2, &TEST, p))
});
case!(linmod_evaluate_short_x, {
    let mut m = Linmod::new_mse().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.evaluate(2, 2, x, &[0.0, 1.0], &mut p))
});
case!(linmod_evaluate_short_y, {
    let mut m = Linmod::new_mse().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut p = OUTF;
    at_guard(&[0.0f64, 1.0], 1, |y| m.evaluate(2, 2, &TEST, y, &mut p))
});
case!(linmod_evaluate_short_pred, {
    let mut m = Linmod::new_mse().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    at_guard(&OUTF, 1, |p| m.evaluate(2, 2, &TEST, &[0.0, 1.0], p))
});
case!(
    dbscan_fit_short_data,
    at_guard(&X, 11, |d| {
        let mut m = Dbscan::new().unwrap();
        m.fit(6, 2, d)
    })
);
control!(dtree_control, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut l = LBL;
    m.predict(2, 2, &TEST, &mut l)
});
case!(
    dtree_fit_short_x,
    at_guard(&X, 11, |x| {
        let mut m = DecisionTree::new().unwrap();
        m.fit(6, 2, 2, x, &YI)
    })
);
case!(
    dtree_fit_short_y,
    at_guard(&YI, 5, |y| {
        let mut m = DecisionTree::new().unwrap();
        m.fit(6, 2, 2, &X, y)
    })
);
case!(dtree_predict_short_x, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut l = LBL;
    at_guard(&TEST, 3, |x| m.predict(2, 2, x, &mut l))
});
case!(dtree_predict_short_labels, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&LBL, 1, |l| m.predict(2, 2, &TEST, l))
});
case!(dtree_score_short_x, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&X, 11, |x| m.score(6, 2, x, &YI).map(|_| ()))
});
case!(dtree_score_short_y, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&YI, 5, |y| m.score(6, 2, &X, y).map(|_| ()))
});
case!(dtree_predict_proba_short_x, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict_proba(2, 2, x, &mut p, 2, 2))
});
case!(dtree_predict_proba_short_out, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&OUTF, 3, |p| m.predict_proba(2, 2, &TEST, p, 2, 2))
});
case!(dtree_predict_log_proba_short_x, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict_log_proba(2, 2, x, &mut p, 2, 2))
});
case!(dtree_predict_log_proba_short_out, {
    let mut m = DecisionTree::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&OUTF, 3, |p| m.predict_log_proba(2, 2, &TEST, p, 2, 2))
});
control!(dforest_control, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut l = LBL;
    m.predict(2, 2, &TEST, &mut l)
});
case!(
    dforest_fit_short_x,
    at_guard(&X, 11, |x| {
        let mut m = DecisionForest::new().unwrap();
        m.fit(6, 2, 2, x, &YI)
    })
);
case!(
    dforest_fit_short_y,
    at_guard(&YI, 5, |y| {
        let mut m = DecisionForest::new().unwrap();
        m.fit(6, 2, 2, &X, y)
    })
);
case!(dforest_predict_short_x, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut l = LBL;
    at_guard(&TEST, 3, |x| m.predict(2, 2, x, &mut l))
});
case!(dforest_predict_short_labels, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&LBL, 1, |l| m.predict(2, 2, &TEST, l))
});
case!(dforest_score_short_x, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&X, 11, |x| m.score(6, 2, x, &YI).map(|_| ()))
});
case!(dforest_score_short_y, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&YI, 5, |y| m.score(6, 2, &X, y).map(|_| ()))
});
case!(dforest_predict_proba_short_x, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict_proba(2, 2, x, &mut p, 2, 2))
});
case!(dforest_predict_proba_short_out, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&OUTF, 3, |p| m.predict_proba(2, 2, &TEST, p, 2, 2))
});
case!(dforest_predict_log_proba_short_x, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict_log_proba(2, 2, x, &mut p, 2, 2))
});
case!(dforest_predict_log_proba_short_out, {
    let mut m = DecisionForest::new().unwrap();
    m.fit(6, 2, 2, &X, &YI).unwrap();
    at_guard(&OUTF, 3, |p| m.predict_log_proba(2, 2, &TEST, p, 2, 2))
});
control!(svm_control, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut p = OUTF;
    m.predict(2, 2, &TEST, &mut p)
});
case!(
    svm_fit_short_x,
    at_guard(&X, 11, |x| {
        let mut m = Svm::new_svc().unwrap();
        m.fit(6, 2, x, &YF)
    })
);
case!(
    svm_fit_short_y,
    at_guard(&YF, 5, |y| {
        let mut m = Svm::new_svc().unwrap();
        m.fit(6, 2, &X, y)
    })
);
case!(svm_predict_short_x, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut p = OUTF;
    at_guard(&TEST, 3, |x| m.predict(2, 2, x, &mut p))
});
case!(svm_predict_short_out, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    at_guard(&OUTF, 1, |p| m.predict(2, 2, &TEST, p))
});
case!(svm_score_short_x, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    at_guard(&X, 11, |x| m.score(6, 2, x, &YF).map(|_| ()))
});
case!(svm_score_short_y, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    at_guard(&YF, 5, |y| m.score(6, 2, &X, y).map(|_| ()))
});
case!(svm_decision_short_x, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    let mut d = OUTF;
    at_guard(&TEST, 3, |x| {
        m.decision_function(2, 2, x, DecisionShape::Ovo, &mut d, 2)
    })
});
case!(svm_decision_short_out, {
    let mut m = Svm::new_svc().unwrap();
    m.fit(6, 2, &X, &YF).unwrap();
    at_guard(&OUTF, 1, |d| {
        m.decision_function(2, 2, &TEST, DecisionShape::Ovo, d, 2)
    })
});
