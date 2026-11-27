//! Safe wrappers for AOCL-DA (Data Analytics).

#![allow(
    clippy::too_many_arguments,
    reason = "wrappers mirror the C API's argument lists"
)]
#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use aocl_data_analytics_sys as sys;
pub use aocl_error::{Error, Result};
use aocl_types::sealed::Sealed;
pub use aocl_types::Layout;

fn layout_raw(l: Layout) -> sys::da_order {
    match l {
        Layout::RowMajor => sys::da_order__row_major,
        Layout::ColMajor => sys::da_order__column_major,
    }
}

/// Reduction axis: per-column, per-row, or global.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Reduce each column down to a single value (output length = n_cols).
    Col,
    /// Reduce each row down to a single value (output length = n_rows).
    Row,
    /// Reduce all elements to a single value (output length = 1).
    All,
}

impl Axis {
    fn raw(self) -> sys::da_axis {
        match self {
            Axis::Col => sys::da_axis__da_axis_col,
            Axis::Row => sys::da_axis__da_axis_row,
            Axis::All => sys::da_axis__da_axis_all,
        }
    }

    fn output_len(self, n_rows: usize, n_cols: usize) -> usize {
        match self {
            Axis::Col => n_cols,
            Axis::Row => n_rows,
            Axis::All => 1,
        }
    }
}

fn check_status(component: &'static str, status: sys::da_status) -> Result<()> {
    if status == sys::da_status__da_status_success {
        return Ok(());
    }
    let message = match status {
        s if s == sys::da_status__da_status_internal_error => "internal error",
        s if s == sys::da_status__da_status_memory_error => "memory error",
        s if s == sys::da_status__da_status_invalid_pointer => "invalid pointer",
        s if s == sys::da_status__da_status_invalid_input => "invalid input",
        s if s == sys::da_status__da_status_not_implemented => "not implemented",
        s if s == sys::da_status__da_status_overflow => "overflow",
        s if s == sys::da_status__da_status_invalid_leading_dimension => {
            "invalid leading dimension"
        }
        s if s == sys::da_status__da_status_negative_data => "negative data",
        s if s == sys::da_status__da_status_invalid_array_dimension => "invalid array dimension",
        s if s == sys::da_status__da_status_no_data => "no data",
        s if s == sys::da_status__da_status_wrong_type => "wrong type",
        s if s == sys::da_status__da_status_invalid_handle_type => "invalid handle type",
        s if s == sys::da_status__da_status_handle_not_initialized => "handle not initialized",
        s if s == sys::da_status__da_status_store_not_initialized => "store not initialized",
        s if s == sys::da_status__da_status_invalid_option => "invalid option",
        s if s == sys::da_status__da_status_incompatible_options => "incompatible options",
        s if s == sys::da_status__da_status_operation_failed => "operation failed",
        s if s == sys::da_status__da_status_unknown_query => "unknown query",
        s if s == sys::da_status__da_status_incorrect_output => "incorrect output",
        s if s == sys::da_status__da_status_maxit => "max iterations reached",
        s if s == sys::da_status__da_status_maxtime => "max time reached",
        s if s == sys::da_status__da_status_numerical_difficulties => "numerical difficulties",
        s if s == sys::da_status__da_status_bad_derivatives => "bad derivatives",
        s if s == sys::da_status__da_status_optimization_usrstop => "optimisation user stop",
        _ => "unknown DA status",
    }
    .to_string();
    Err(Error::Status {
        component,
        code: status as i64,
        message,
    })
}

fn check_matrix(
    name: &str,
    layout: Layout,
    n_rows: usize,
    n_cols: usize,
    ldx: usize,
    x_len: usize,
) -> Result<()> {
    if n_rows == 0 || n_cols == 0 {
        return Ok(());
    }
    let (lead, trail) = match layout {
        Layout::RowMajor => (n_rows, n_cols),
        Layout::ColMajor => (n_cols, n_rows),
    };
    if ldx < trail {
        return Err(Error::InvalidArgument(format!(
            "{name}: ldx={ldx} < {trail}"
        )));
    }
    let needed = (lead - 1) * ldx + trail;
    if x_len < needed {
        return Err(Error::InvalidArgument(format!(
            "{name}: slice length {x_len} too small (need {needed})"
        )));
    }
    Ok(())
}

/// Method used for quantile computation. Each variant matches one of
/// the nine standard quantile types (Hyndman & Fan 1996); types 6 and 7
/// are the most commonly used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuantileType {
    Type1,
    Type2,
    Type3,
    Type4,
    Type5,
    Type6,
    Type7,
    Type8,
    Type9,
}

impl QuantileType {
    fn raw(self) -> sys::da_quantile_type {
        match self {
            QuantileType::Type1 => sys::da_quantile_type__da_quantile_type_1,
            QuantileType::Type2 => sys::da_quantile_type__da_quantile_type_2,
            QuantileType::Type3 => sys::da_quantile_type__da_quantile_type_3,
            QuantileType::Type4 => sys::da_quantile_type__da_quantile_type_4,
            QuantileType::Type5 => sys::da_quantile_type__da_quantile_type_5,
            QuantileType::Type6 => sys::da_quantile_type__da_quantile_type_6,
            QuantileType::Type7 => sys::da_quantile_type__da_quantile_type_7,
            QuantileType::Type8 => sys::da_quantile_type__da_quantile_type_8,
            QuantileType::Type9 => sys::da_quantile_type__da_quantile_type_9,
        }
    }
}

/// Scalar element type usable with the wrapped DA routines.
pub trait Scalar: Copy + Sized + Sealed {
    /// Compute means along `axis` for an `n_rows × n_cols` matrix.
    fn mean(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        out: &mut [Self],
    ) -> Result<()>;

    /// Compute means and variances along `axis`.
    ///
    /// `dof` selects the variance divisor:
    /// - `dof < 0` — divide by `n` (number of observations).
    /// - `dof == 0` — divide by `n - 1` (Bessel-corrected, unbiased).
    /// - `dof > 0` — divide by the literal value of `dof`.
    fn variance(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        dof: i64,
        mean_out: &mut [Self],
        variance_out: &mut [Self],
    ) -> Result<()>;

    /// Geometric mean. Requires non-negative data.
    fn geometric_mean(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        out: &mut [Self],
    ) -> Result<()>;

    /// Harmonic mean.
    fn harmonic_mean(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        out: &mut [Self],
    ) -> Result<()>;

    /// Mean, variance, and skewness together.
    fn skewness(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        mean_out: &mut [Self],
        variance_out: &mut [Self],
        skew_out: &mut [Self],
    ) -> Result<()>;

    /// Mean, variance, and excess kurtosis together (Fisher convention,
    /// so a normal distribution scores zero).
    fn kurtosis(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        mean_out: &mut [Self],
        variance_out: &mut [Self],
        kurtosis_out: &mut [Self],
    ) -> Result<()>;

    /// `k`-th central moment. If `mean_out` already holds means and
    /// `use_precomputed_mean` is true, those means are used; otherwise
    /// they are computed and stored.
    fn moment(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        k: i64,
        use_precomputed_mean: bool,
        mean_out: &mut [Self],
        moment_out: &mut [Self],
    ) -> Result<()>;

    /// Quantile of the data along `axis`. `q` ∈ [0, 1].
    fn quantile(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        q: Self,
        quantile_out: &mut [Self],
        qtype: QuantileType,
    ) -> Result<()>;

    /// Min, lower hinge (25th percentile), median, upper hinge
    /// (75th percentile), max — Tukey's "five-number summary".
    fn five_point_summary(
        layout: Layout,
        axis: Axis,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        min: &mut [Self],
        lower: &mut [Self],
        median: &mut [Self],
        upper: &mut [Self],
        max: &mut [Self],
    ) -> Result<()>;

    /// Compute the `n_cols × n_cols` covariance matrix (rows are
    /// observations). `dof` follows the same convention as `variance`.
    fn covariance_matrix(
        layout: Layout,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        dof: i64,
        cov: &mut [Self],
        ldcov: usize,
    ) -> Result<()>;

    /// Compute the `n_cols × n_cols` correlation matrix (rows are
    /// observations).
    fn correlation_matrix(
        layout: Layout,
        n_rows: usize,
        n_cols: usize,
        x: &[Self],
        ldx: usize,
        corr: &mut [Self],
        ldcorr: usize,
    ) -> Result<()>;
}

macro_rules! impl_scalar {
    (
        $t:ty,
        mean = $mean:ident,
        variance = $variance:ident,
        geometric_mean = $gmean:ident,
        harmonic_mean = $hmean:ident,
        skewness = $skew:ident,
        kurtosis = $kurt:ident,
        moment = $moment:ident,
        quantile = $quant:ident,
        five_point_summary = $fps:ident,
        covariance_matrix = $cov:ident,
        correlation_matrix = $corr:ident
    ) => {
        impl Scalar for $t {
            fn mean(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                out: &mut [Self],
            ) -> Result<()> {
                check_matrix("mean: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "mean: out length {} < required {need}",
                        out.len()
                    )));
                }
                let status = unsafe {
                    sys::$mean(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn variance(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                dof: i64,
                mean_out: &mut [Self],
                variance_out: &mut [Self],
            ) -> Result<()> {
                check_matrix("variance: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if mean_out.len() < need || variance_out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "variance: outputs must hold {need} elements"
                    )));
                }
                let status = unsafe {
                    sys::$variance(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        dof as sys::da_int,
                        mean_out.as_mut_ptr(),
                        variance_out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn geometric_mean(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                out: &mut [Self],
            ) -> Result<()> {
                check_matrix("geometric_mean: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "geometric_mean: out length {} < required {need}",
                        out.len()
                    )));
                }
                let status = unsafe {
                    sys::$gmean(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn harmonic_mean(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                out: &mut [Self],
            ) -> Result<()> {
                check_matrix("harmonic_mean: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "harmonic_mean: out length {} < required {need}",
                        out.len()
                    )));
                }
                let status = unsafe {
                    sys::$hmean(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn skewness(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                mean_out: &mut [Self],
                variance_out: &mut [Self],
                skew_out: &mut [Self],
            ) -> Result<()> {
                check_matrix("skewness: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if mean_out.len() < need || variance_out.len() < need || skew_out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "skewness: outputs must hold {need} elements"
                    )));
                }
                let status = unsafe {
                    sys::$skew(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        mean_out.as_mut_ptr(),
                        variance_out.as_mut_ptr(),
                        skew_out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn kurtosis(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                mean_out: &mut [Self],
                variance_out: &mut [Self],
                kurtosis_out: &mut [Self],
            ) -> Result<()> {
                check_matrix("kurtosis: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if mean_out.len() < need || variance_out.len() < need || kurtosis_out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "kurtosis: outputs must hold {need} elements"
                    )));
                }
                let status = unsafe {
                    sys::$kurt(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        mean_out.as_mut_ptr(),
                        variance_out.as_mut_ptr(),
                        kurtosis_out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn moment(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                k: i64,
                use_precomputed_mean: bool,
                mean_out: &mut [Self],
                moment_out: &mut [Self],
            ) -> Result<()> {
                check_matrix("moment: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if mean_out.len() < need || moment_out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "moment: outputs must hold {need} elements"
                    )));
                }
                let status = unsafe {
                    sys::$moment(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        k as sys::da_int,
                        if use_precomputed_mean { 1 } else { 0 } as sys::da_int,
                        mean_out.as_mut_ptr(),
                        moment_out.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn quantile(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                q: Self,
                quantile_out: &mut [Self],
                qtype: QuantileType,
            ) -> Result<()> {
                check_matrix("quantile: X", layout, n_rows, n_cols, ldx, x.len())?;
                let need = axis.output_len(n_rows, n_cols);
                if quantile_out.len() < need {
                    return Err(Error::InvalidArgument(format!(
                        "quantile: out length {} < required {need}",
                        quantile_out.len()
                    )));
                }
                let status = unsafe {
                    sys::$quant(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        &q,
                        1,
                        quantile_out.as_mut_ptr(),
                        qtype.raw(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn five_point_summary(
                layout: Layout,
                axis: Axis,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                min: &mut [Self],
                lower: &mut [Self],
                median: &mut [Self],
                upper: &mut [Self],
                max: &mut [Self],
            ) -> Result<()> {
                check_matrix(
                    "five_point_summary: X",
                    layout,
                    n_rows,
                    n_cols,
                    ldx,
                    x.len(),
                )?;
                let need = axis.output_len(n_rows, n_cols);
                if min.len() < need
                    || lower.len() < need
                    || median.len() < need
                    || upper.len() < need
                    || max.len() < need
                {
                    return Err(Error::InvalidArgument(format!(
                        "five_point_summary: each output must hold {need} elements"
                    )));
                }
                let status = unsafe {
                    sys::$fps(
                        layout_raw(layout),
                        axis.raw(),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        min.as_mut_ptr(),
                        lower.as_mut_ptr(),
                        median.as_mut_ptr(),
                        upper.as_mut_ptr(),
                        max.as_mut_ptr(),
                    )
                };
                check_status("data-analytics", status)
            }

            fn covariance_matrix(
                layout: Layout,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                dof: i64,
                cov: &mut [Self],
                ldcov: usize,
            ) -> Result<()> {
                check_matrix("covariance_matrix: X", layout, n_rows, n_cols, ldx, x.len())?;
                if ldcov < n_cols {
                    return Err(Error::InvalidArgument(format!(
                        "covariance_matrix: ldcov={ldcov} < n_cols={n_cols}"
                    )));
                }
                if cov.len() < n_cols * ldcov {
                    return Err(Error::InvalidArgument(format!(
                        "covariance_matrix: cov length {} < n_cols·ldcov = {}",
                        cov.len(),
                        n_cols * ldcov
                    )));
                }
                let status = unsafe {
                    sys::$cov(
                        layout_raw(layout),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        dof as sys::da_int,
                        cov.as_mut_ptr(),
                        ldcov as sys::da_int,
                        0, // assume_centered = false: subtract the column means, as before AOCL 5.3 added the argument
                    )
                };
                check_status("data-analytics", status)
            }

            fn correlation_matrix(
                layout: Layout,
                n_rows: usize,
                n_cols: usize,
                x: &[Self],
                ldx: usize,
                corr: &mut [Self],
                ldcorr: usize,
            ) -> Result<()> {
                check_matrix(
                    "correlation_matrix: X",
                    layout,
                    n_rows,
                    n_cols,
                    ldx,
                    x.len(),
                )?;
                if ldcorr < n_cols {
                    return Err(Error::InvalidArgument(format!(
                        "correlation_matrix: ldcorr={ldcorr} < n_cols={n_cols}"
                    )));
                }
                if corr.len() < n_cols * ldcorr {
                    return Err(Error::InvalidArgument(format!(
                        "correlation_matrix: corr length {} < n_cols·ldcorr = {}",
                        corr.len(),
                        n_cols * ldcorr
                    )));
                }
                let status = unsafe {
                    sys::$corr(
                        layout_raw(layout),
                        n_rows as sys::da_int,
                        n_cols as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        corr.as_mut_ptr(),
                        ldcorr as sys::da_int,
                    )
                };
                check_status("data-analytics", status)
            }
        }
    };
}

impl_scalar!(
    f32,
    mean = da_mean_s,
    variance = da_variance_s,
    geometric_mean = da_geometric_mean_s,
    harmonic_mean = da_harmonic_mean_s,
    skewness = da_skewness_s,
    kurtosis = da_kurtosis_s,
    moment = da_moment_s,
    quantile = da_quantile_s,
    five_point_summary = da_five_point_summary_s,
    covariance_matrix = da_covariance_matrix_s,
    correlation_matrix = da_correlation_matrix_s
);
impl_scalar!(
    f64,
    mean = da_mean_d,
    variance = da_variance_d,
    geometric_mean = da_geometric_mean_d,
    harmonic_mean = da_harmonic_mean_d,
    skewness = da_skewness_d,
    kurtosis = da_kurtosis_d,
    moment = da_moment_d,
    quantile = da_quantile_d,
    five_point_summary = da_five_point_summary_d,
    covariance_matrix = da_covariance_matrix_d,
    correlation_matrix = da_correlation_matrix_d
);

/// Compute the mean of a tightly-packed row-major matrix along `axis`.
pub fn mean<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    out: &mut [T],
) -> Result<()> {
    T::mean(Layout::RowMajor, axis, n_rows, n_cols, x, n_cols, out)
}

/// Compute mean and variance of a tightly-packed row-major matrix along `axis`.
pub fn variance<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    dof: i64,
    mean_out: &mut [T],
    variance_out: &mut [T],
) -> Result<()> {
    T::variance(
        Layout::RowMajor,
        axis,
        n_rows,
        n_cols,
        x,
        n_cols,
        dof,
        mean_out,
        variance_out,
    )
}

/// Compute the geometric mean along `axis` for a tightly-packed
/// row-major matrix. All entries must be non-negative.
pub fn geometric_mean<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    out: &mut [T],
) -> Result<()> {
    T::geometric_mean(Layout::RowMajor, axis, n_rows, n_cols, x, n_cols, out)
}

/// Compute the harmonic mean along `axis` for a tightly-packed
/// row-major matrix.
pub fn harmonic_mean<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    out: &mut [T],
) -> Result<()> {
    T::harmonic_mean(Layout::RowMajor, axis, n_rows, n_cols, x, n_cols, out)
}

/// Compute means, variances, and Fisher–Pearson skewness along `axis`.
pub fn skewness<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    mean_out: &mut [T],
    variance_out: &mut [T],
    skew_out: &mut [T],
) -> Result<()> {
    T::skewness(
        Layout::RowMajor,
        axis,
        n_rows,
        n_cols,
        x,
        n_cols,
        mean_out,
        variance_out,
        skew_out,
    )
}

/// Compute means, variances, and excess kurtosis (Fisher convention,
/// so a normal distribution scores zero) along `axis`.
pub fn kurtosis<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    mean_out: &mut [T],
    variance_out: &mut [T],
    kurtosis_out: &mut [T],
) -> Result<()> {
    T::kurtosis(
        Layout::RowMajor,
        axis,
        n_rows,
        n_cols,
        x,
        n_cols,
        mean_out,
        variance_out,
        kurtosis_out,
    )
}

/// Compute the `k`-th central moment along `axis`. If `mean_out` already
/// holds means and `use_precomputed_mean` is `true`, those are used
/// directly; otherwise means are computed and stored.
pub fn moment<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    k: i64,
    use_precomputed_mean: bool,
    mean_out: &mut [T],
    moment_out: &mut [T],
) -> Result<()> {
    T::moment(
        Layout::RowMajor,
        axis,
        n_rows,
        n_cols,
        x,
        n_cols,
        k,
        use_precomputed_mean,
        mean_out,
        moment_out,
    )
}

/// Compute the `q`-th quantile along `axis` (`q` ∈ [0, 1]). The
/// `qtype` selects one of the nine standard quantile formulae;
/// `QuantileType::Type7` matches numpy and R's defaults.
pub fn quantile<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    q: T,
    quantile_out: &mut [T],
    qtype: QuantileType,
) -> Result<()> {
    T::quantile(
        Layout::RowMajor,
        axis,
        n_rows,
        n_cols,
        x,
        n_cols,
        q,
        quantile_out,
        qtype,
    )
}

/// Compute Tukey's five-number summary (min, lower hinge, median,
/// upper hinge, max) along `axis`.
pub fn five_point_summary<T: Scalar>(
    axis: Axis,
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    min: &mut [T],
    lower: &mut [T],
    median: &mut [T],
    upper: &mut [T],
    max: &mut [T],
) -> Result<()> {
    T::five_point_summary(
        Layout::RowMajor,
        axis,
        n_rows,
        n_cols,
        x,
        n_cols,
        min,
        lower,
        median,
        upper,
        max,
    )
}

/// Compute the `n_cols × n_cols` covariance matrix (rows are observations).
/// `dof` follows the same convention as [`variance`].
pub fn covariance_matrix<T: Scalar>(
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    dof: i64,
    cov: &mut [T],
) -> Result<()> {
    T::covariance_matrix(
        Layout::RowMajor,
        n_rows,
        n_cols,
        x,
        n_cols,
        dof,
        cov,
        n_cols,
    )
}

/// Compute the `n_cols × n_cols` correlation matrix (rows are observations).
pub fn correlation_matrix<T: Scalar>(
    n_rows: usize,
    n_cols: usize,
    x: &[T],
    corr: &mut [T],
) -> Result<()> {
    T::correlation_matrix(Layout::RowMajor, n_rows, n_cols, x, n_cols, corr, n_cols)
}

// =========================================================================
//   Nonlinear least-squares (NLLS)
// =========================================================================

/// Type-erased residual closure: maps coefficients `x` of length `n_coef`
/// to residuals `res` of length `n_res`. Return `0` on success or non-zero
/// to signal an evaluation failure (the solver will return an error).
pub type ResFn<'a> = Box<dyn FnMut(&[f64], &mut [f64]) -> i32 + 'a>;

/// Type-erased Jacobian closure: writes the `n_res × n_coef` Jacobian
/// of the residuals at `x` into `jac`. Layout matches the AOCL
/// `"storage scheme"` option (defaults to row-major). Return `0` on
/// success.
pub type JacFn<'a> = Box<dyn FnMut(&[f64], &mut [f64]) -> i32 + 'a>;

struct NllsCallbacks<'a> {
    resfun: ResFn<'a>,
    resgrd: Option<JacFn<'a>>,
    n_coef: usize,
    n_res: usize,
}

unsafe extern "C" fn nlls_resfun_shim(
    _n_coef: sys::da_int,
    _n_res: sys::da_int,
    data: *mut std::os::raw::c_void,
    x: *const f64,
    res: *mut f64,
) -> sys::da_int {
    let cbs = unsafe { &mut *(data as *mut NllsCallbacks) };
    let x_slice = unsafe { std::slice::from_raw_parts(x, cbs.n_coef) };
    let res_slice = unsafe { std::slice::from_raw_parts_mut(res, cbs.n_res) };
    // Catch panics so we don't unwind across the FFI boundary.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (cbs.resfun)(x_slice, res_slice)
    }));
    match result {
        Ok(rc) => rc as sys::da_int,
        Err(_) => 1, // signal failure to the solver
    }
}

unsafe extern "C" fn nlls_resgrd_shim(
    _n_coef: sys::da_int,
    _n_res: sys::da_int,
    data: *mut std::os::raw::c_void,
    x: *const f64,
    jac: *mut f64,
) -> sys::da_int {
    let cbs = unsafe { &mut *(data as *mut NllsCallbacks) };
    let x_slice = unsafe { std::slice::from_raw_parts(x, cbs.n_coef) };
    let jac_slice = unsafe { std::slice::from_raw_parts_mut(jac, cbs.n_res * cbs.n_coef) };
    let f = match cbs.resgrd.as_mut() {
        Some(f) => f,
        None => return 1,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(x_slice, jac_slice)));
    match result {
        Ok(rc) => rc as sys::da_int,
        Err(_) => 1,
    }
}

/// Nonlinear least-squares solver wrapping AOCL's `RAlfit` family.
///
/// Provide a residual function (mandatory) and optionally a Jacobian.
/// If the Jacobian is omitted, AOCL will estimate derivatives by finite
/// differences. After [`Nlls::fit`], the optimised coefficients are in
/// the user's input vector and a 100-element `rinfo` of solver stats
/// can be retrieved via [`Nlls::info`].
pub struct Nlls<'a> {
    handle: Handle,
    callbacks: Box<NllsCallbacks<'a>>,
}

impl<'a> Nlls<'a> {
    /// Build a new NLLS solver with `n_coef` coefficients and `n_res`
    /// residuals. `resfun` evaluates `r(x)`; finite differences will
    /// estimate the Jacobian unless one is supplied via
    /// [`Nlls::with_jacobian`].
    pub fn new<F>(n_coef: usize, n_res: usize, resfun: F) -> Result<Self>
    where
        F: FnMut(&[f64], &mut [f64]) -> i32 + 'a,
    {
        let handle = Handle::new_double(HandleKind::Nlls)?;
        let callbacks = Box::new(NllsCallbacks {
            resfun: Box::new(resfun),
            resgrd: None,
            n_coef,
            n_res,
        });
        let mut this = Self { handle, callbacks };
        this.install()?;
        Ok(this)
    }

    /// Attach an analytic Jacobian. Layout follows the `"storage scheme"`
    /// option set on the underlying handle (default row-major).
    pub fn with_jacobian<G>(mut self, resgrd: G) -> Result<Self>
    where
        G: FnMut(&[f64], &mut [f64]) -> i32 + 'a,
    {
        self.callbacks.resgrd = Some(Box::new(resgrd));
        self.install()?;
        Ok(self)
    }

    fn install(&mut self) -> Result<()> {
        let resgrd = if self.callbacks.resgrd.is_some() {
            Some(nlls_resgrd_shim as unsafe extern "C" fn(_, _, _, _, _) -> _)
        } else {
            None
        };
        let status = unsafe {
            sys::da_nlls_define_residuals_d(
                self.handle.raw,
                self.callbacks.n_coef as sys::da_int,
                self.callbacks.n_res as sys::da_int,
                Some(nlls_resfun_shim),
                resgrd,
                None,
                None,
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("nlls define_residuals failed: {extra}"),
            });
        }
        Ok(())
    }

    /// Borrow the underlying handle to set NLLS-specific options
    /// (`"ralfit iteration limit"`, `"storage scheme"`, etc.).
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Constrain coefficients to lie within `lower[i] <= x[i] <= upper[i]`.
    /// Pass `None` for an unbounded side; the slices must have length
    /// `n_coef`.
    pub fn set_bounds(
        &mut self,
        lower: Option<&mut [f64]>,
        upper: Option<&mut [f64]>,
    ) -> Result<()> {
        let n_coef = self.callbacks.n_coef as sys::da_int;
        let l_ptr = lower
            .as_ref()
            .map_or(std::ptr::null_mut(), |s| s.as_ptr() as *mut f64);
        let u_ptr = upper
            .as_ref()
            .map_or(std::ptr::null_mut(), |s| s.as_ptr() as *mut f64);
        if let Some(l) = lower.as_ref() {
            if l.len() < self.callbacks.n_coef {
                return Err(Error::InvalidArgument(format!(
                    "set_bounds: lower length {} < n_coef = {}",
                    l.len(),
                    self.callbacks.n_coef
                )));
            }
        }
        if let Some(u) = upper.as_ref() {
            if u.len() < self.callbacks.n_coef {
                return Err(Error::InvalidArgument(format!(
                    "set_bounds: upper length {} < n_coef = {}",
                    u.len(),
                    self.callbacks.n_coef
                )));
            }
        }
        let status = unsafe { sys::da_nlls_define_bounds_d(self.handle.raw, n_coef, l_ptr, u_ptr) };
        check_status("data-analytics", status)
    }

    /// Set per-residual weights `w_i = √W_ii` (length `n_res`).
    pub fn set_weights(&mut self, weights: &mut [f64]) -> Result<()> {
        if weights.len() < self.callbacks.n_res {
            return Err(Error::InvalidArgument(format!(
                "set_weights: weights length {} < n_res = {}",
                weights.len(),
                self.callbacks.n_res
            )));
        }
        let status = unsafe {
            sys::da_nlls_define_weights_d(
                self.handle.raw,
                self.callbacks.n_res as sys::da_int,
                weights.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)
    }

    /// Run the optimiser. On entry `coef` is the initial guess; on
    /// success it holds the optimised coefficients.
    pub fn fit(&mut self, coef: &mut [f64]) -> Result<()> {
        if coef.len() < self.callbacks.n_coef {
            return Err(Error::InvalidArgument(format!(
                "nlls fit: coef length {} < n_coef = {}",
                coef.len(),
                self.callbacks.n_coef
            )));
        }
        let udata = self.callbacks.as_mut() as *mut NllsCallbacks as *mut std::os::raw::c_void;
        let status = unsafe {
            sys::da_nlls_fit_d(
                self.handle.raw,
                self.callbacks.n_coef as sys::da_int,
                coef.as_mut_ptr(),
                udata,
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("nlls fit failed: {extra}"),
            });
        }
        Ok(())
    }

    /// Retrieve the solver's information vector (`rinfo`, length 100) —
    /// see AOCL's documentation for the index meanings:
    /// `info_objective` (0), `info_grad_norm` (1), `info_iter` (2),
    /// `info_time` (3), etc.
    pub fn info(&self) -> Result<Vec<f64>> {
        // rinfo is a fixed-size 100-element vector for NLLS.
        let mut dim: sys::da_int = 100;
        let mut out = vec![0.0_f64; 100];
        let status = unsafe {
            sys::da_handle_get_result_d(
                self.handle.raw,
                sys::da_result__da_rinfo,
                &mut dim,
                out.as_mut_ptr(),
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("nlls info: {extra}"),
            });
        }
        out.truncate(dim as usize);
        Ok(out)
    }
}

impl<'a> std::fmt::Debug for Nlls<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Nlls")
            .field("n_coef", &self.callbacks.n_coef)
            .field("n_res", &self.callbacks.n_res)
            .field("handle", &self.handle)
            .finish()
    }
}

// =========================================================================
//   Datastore (CSV loading + column selection)
// =========================================================================

/// RAII wrapper around `da_datastore` — AOCL-DA's container for tabular
/// data loaded from CSV. Use it to read a file, query its dimensions,
/// label or select columns by name, and extract numeric blocks into
/// caller-provided buffers.
pub struct Datastore {
    raw: sys::da_datastore,
}

unsafe impl Send for Datastore {}

impl Datastore {
    /// Create a fresh empty datastore.
    pub fn new() -> Result<Self> {
        let mut raw: sys::da_datastore = std::ptr::null_mut();
        let status = unsafe { sys::da_datastore_init(&mut raw) };
        check_status("data-analytics", status)?;
        if raw.is_null() {
            return Err(Error::AllocationFailed("data-analytics"));
        }
        Ok(Self { raw })
    }

    /// Set an integer-valued datastore option (e.g.
    /// `set_int_option("use header row", 1)`,
    /// `set_int_option("whitespace delimiter", 1)`).
    pub fn set_int_option(&mut self, name: &str, value: i64) -> Result<()> {
        let cs = std::ffi::CString::new(name)
            .map_err(|e| Error::InvalidArgument(format!("set_int_option: {e}")))?;
        let status = unsafe {
            sys::da_datastore_options_set_int(self.raw, cs.as_ptr(), value as sys::da_int)
        };
        check_status("data-analytics", status)
    }

    /// Set a string-valued datastore option (e.g. `"comment"` to set a
    /// comment-line prefix character, `"storage order"` to choose
    /// row-major vs column-major).
    pub fn set_string_option(&mut self, name: &str, value: &str) -> Result<()> {
        let n = std::ffi::CString::new(name)
            .map_err(|e| Error::InvalidArgument(format!("set_string_option: {e}")))?;
        let v = std::ffi::CString::new(value)
            .map_err(|e| Error::InvalidArgument(format!("set_string_option: {e}")))?;
        let status =
            unsafe { sys::da_datastore_options_set_string(self.raw, n.as_ptr(), v.as_ptr()) };
        check_status("data-analytics", status)
    }

    /// Load a CSV file into the datastore.
    pub fn load_csv(&mut self, path: &str) -> Result<()> {
        let cs = std::ffi::CString::new(path)
            .map_err(|e| Error::InvalidArgument(format!("load_csv: {e}")))?;
        let status = unsafe { sys::da_data_load_from_csv(self.raw, cs.as_ptr()) };
        check_status("data-analytics", status)
    }

    /// Number of rows currently held by the store.
    pub fn n_rows(&self) -> Result<usize> {
        let mut n: sys::da_int = 0;
        let status = unsafe { sys::da_data_get_n_rows(self.raw, &mut n) };
        check_status("data-analytics", status)?;
        Ok(n as usize)
    }

    /// Number of columns currently held by the store.
    pub fn n_cols(&self) -> Result<usize> {
        let mut n: sys::da_int = 0;
        let status = unsafe { sys::da_data_get_n_cols(self.raw, &mut n) };
        check_status("data-analytics", status)?;
        Ok(n as usize)
    }

    /// `(n_rows, n_cols)` dimensions.
    pub fn dims(&self) -> Result<(usize, usize)> {
        Ok((self.n_rows()?, self.n_cols()?))
    }

    /// Select a contiguous range of columns into the named selection
    /// `key` (zero-based, inclusive bounds). Use [`Datastore::extract_f64`]
    /// to read the selected columns into a buffer.
    pub fn select_columns(&mut self, key: &str, lbound: usize, ubound: usize) -> Result<()> {
        let cs = std::ffi::CString::new(key)
            .map_err(|e| Error::InvalidArgument(format!("select_columns: {e}")))?;
        let status = unsafe {
            sys::da_data_select_columns(
                self.raw,
                cs.as_ptr(),
                lbound as sys::da_int,
                ubound as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }

    /// Select a contiguous range of rows into the named selection `key`.
    pub fn select_rows(&mut self, key: &str, lbound: usize, ubound: usize) -> Result<()> {
        let cs = std::ffi::CString::new(key)
            .map_err(|e| Error::InvalidArgument(format!("select_rows: {e}")))?;
        let status = unsafe {
            sys::da_data_select_rows(
                self.raw,
                cs.as_ptr(),
                lbound as sys::da_int,
                ubound as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }

    /// Extract the named selection into `data` as `f64` values, in the
    /// order specified by `layout`. `lddata` is the leading dimension
    /// of the output buffer.
    pub fn extract_f64(
        &self,
        key: &str,
        layout: Layout,
        data: &mut [f64],
        lddata: usize,
    ) -> Result<()> {
        let cs = std::ffi::CString::new(key)
            .map_err(|e| Error::InvalidArgument(format!("extract_f64: {e}")))?;
        let status = unsafe {
            sys::da_data_extract_selection_real_d(
                self.raw,
                cs.as_ptr(),
                layout_raw(layout),
                data.as_mut_ptr(),
                lddata as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }

    /// Borrow the raw `da_datastore` handle (for routines this crate
    /// doesn't yet wrap).
    pub fn as_raw(&self) -> sys::da_datastore {
        self.raw
    }
}

impl Drop for Datastore {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { sys::da_datastore_destroy(&mut self.raw) };
            self.raw = std::ptr::null_mut();
        }
    }
}

impl std::fmt::Debug for Datastore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Datastore").finish_non_exhaustive()
    }
}

// =========================================================================
//   Pairwise distances
// =========================================================================

/// Distance metric for [`pairwise_distances`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Metric {
    /// Euclidean distance (`L2`).
    Euclidean,
    /// Squared Euclidean distance (`Σ (xᵢ-yᵢ)²` without the square root).
    SqEuclidean,
    /// Minkowski distance with parameter `p` (defaults to L2 at p=2).
    Minkowski,
    /// Manhattan / `L1` / cityblock distance.
    Manhattan,
    /// Cosine distance: `1 - cos(angle)`.
    Cosine,
}

impl Metric {
    fn raw(self) -> sys::da_metric {
        match self {
            Metric::Euclidean => sys::da_metric__da_euclidean,
            Metric::SqEuclidean => sys::da_metric__da_sqeuclidean,
            Metric::Minkowski => sys::da_metric__da_minkowski,
            Metric::Manhattan => sys::da_metric__da_manhattan,
            Metric::Cosine => sys::da_metric__da_cosine,
        }
    }
}

/// Scalar element type usable with [`pairwise_distances`].
pub trait DistScalar: Scalar {
    /// Compute the pairwise distance matrix `D[i,j] = metric(X[i], Y[j])`.
    /// If `y` is `None`, distances are computed within `x` (so `D` is
    /// `m × m`); otherwise `D` is `m × n`.
    fn pairwise_distances(
        layout: Layout,
        m: usize,
        n: usize,
        k: usize,
        x: &[Self],
        ldx: usize,
        y: Option<&[Self]>,
        ldy: usize,
        d: &mut [Self],
        ldd: usize,
        p: Self,
        metric: Metric,
    ) -> Result<()>;
}

macro_rules! impl_dist_scalar {
    ($t:ty, $fn:ident) => {
        impl DistScalar for $t {
            fn pairwise_distances(
                layout: Layout,
                m: usize,
                n: usize,
                k: usize,
                x: &[Self],
                ldx: usize,
                y: Option<&[Self]>,
                ldy: usize,
                d: &mut [Self],
                ldd: usize,
                p: Self,
                metric: Metric,
            ) -> Result<()> {
                let y_ptr = y.map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
                let status = unsafe {
                    sys::$fn(
                        layout_raw(layout),
                        m as sys::da_int,
                        n as sys::da_int,
                        k as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        y_ptr,
                        ldy as sys::da_int,
                        d.as_mut_ptr(),
                        ldd as sys::da_int,
                        p,
                        metric.raw(),
                    )
                };
                check_status("data-analytics", status)
            }
        }
    };
}

impl_dist_scalar!(f32, da_pairwise_distances_s);
impl_dist_scalar!(f64, da_pairwise_distances_d);

// =========================================================================
//   Kernel functions (linear / RBF / polynomial / sigmoid)
// =========================================================================

/// Scalar element type for kernel-matrix computations.
pub trait KernelScalar: Scalar {
    /// Linear kernel: `K(x, y) = x · y`.
    fn linear_kernel(
        layout: Layout,
        m: usize,
        n: usize,
        k: usize,
        x: &[Self],
        ldx: usize,
        y: Option<&[Self]>,
        ldy: usize,
        d: &mut [Self],
        ldd: usize,
    ) -> Result<()>;

    /// RBF (Gaussian) kernel: `K(x, y) = exp(-γ · ‖x − y‖²)`.
    fn rbf_kernel(
        layout: Layout,
        m: usize,
        n: usize,
        k: usize,
        x: &[Self],
        ldx: usize,
        y: Option<&[Self]>,
        ldy: usize,
        d: &mut [Self],
        ldd: usize,
        gamma: Self,
    ) -> Result<()>;

    /// Polynomial kernel: `K(x, y) = (γ · x · y + c)^degree`.
    fn polynomial_kernel(
        layout: Layout,
        m: usize,
        n: usize,
        k: usize,
        x: &[Self],
        ldx: usize,
        y: Option<&[Self]>,
        ldy: usize,
        d: &mut [Self],
        ldd: usize,
        gamma: Self,
        degree: i64,
        coef0: Self,
    ) -> Result<()>;

    /// Sigmoid kernel: `K(x, y) = tanh(γ · x · y + c)`.
    fn sigmoid_kernel(
        layout: Layout,
        m: usize,
        n: usize,
        k: usize,
        x: &[Self],
        ldx: usize,
        y: Option<&[Self]>,
        ldy: usize,
        d: &mut [Self],
        ldd: usize,
        gamma: Self,
        coef0: Self,
    ) -> Result<()>;
}

macro_rules! impl_kernel_scalar {
    ($t:ty, $linear:ident, $rbf:ident, $poly:ident, $sigmoid:ident) => {
        impl KernelScalar for $t {
            fn linear_kernel(
                layout: Layout,
                m: usize,
                n: usize,
                k: usize,
                x: &[Self],
                ldx: usize,
                y: Option<&[Self]>,
                ldy: usize,
                d: &mut [Self],
                ldd: usize,
            ) -> Result<()> {
                let y_ptr = y.map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
                let status = unsafe {
                    sys::$linear(
                        layout_raw(layout),
                        m as sys::da_int,
                        n as sys::da_int,
                        k as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        y_ptr,
                        ldy as sys::da_int,
                        d.as_mut_ptr(),
                        ldd as sys::da_int,
                    )
                };
                check_status("data-analytics", status)
            }

            fn rbf_kernel(
                layout: Layout,
                m: usize,
                n: usize,
                k: usize,
                x: &[Self],
                ldx: usize,
                y: Option<&[Self]>,
                ldy: usize,
                d: &mut [Self],
                ldd: usize,
                gamma: Self,
            ) -> Result<()> {
                let y_ptr = y.map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
                let status = unsafe {
                    sys::$rbf(
                        layout_raw(layout),
                        m as sys::da_int,
                        n as sys::da_int,
                        k as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        y_ptr,
                        ldy as sys::da_int,
                        d.as_mut_ptr(),
                        ldd as sys::da_int,
                        gamma,
                    )
                };
                check_status("data-analytics", status)
            }

            fn polynomial_kernel(
                layout: Layout,
                m: usize,
                n: usize,
                k: usize,
                x: &[Self],
                ldx: usize,
                y: Option<&[Self]>,
                ldy: usize,
                d: &mut [Self],
                ldd: usize,
                gamma: Self,
                degree: i64,
                coef0: Self,
            ) -> Result<()> {
                let y_ptr = y.map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
                let status = unsafe {
                    sys::$poly(
                        layout_raw(layout),
                        m as sys::da_int,
                        n as sys::da_int,
                        k as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        y_ptr,
                        ldy as sys::da_int,
                        d.as_mut_ptr(),
                        ldd as sys::da_int,
                        gamma,
                        degree as sys::da_int,
                        coef0,
                    )
                };
                check_status("data-analytics", status)
            }

            fn sigmoid_kernel(
                layout: Layout,
                m: usize,
                n: usize,
                k: usize,
                x: &[Self],
                ldx: usize,
                y: Option<&[Self]>,
                ldy: usize,
                d: &mut [Self],
                ldd: usize,
                gamma: Self,
                coef0: Self,
            ) -> Result<()> {
                let y_ptr = y.map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
                let status = unsafe {
                    sys::$sigmoid(
                        layout_raw(layout),
                        m as sys::da_int,
                        n as sys::da_int,
                        k as sys::da_int,
                        x.as_ptr(),
                        ldx as sys::da_int,
                        y_ptr,
                        ldy as sys::da_int,
                        d.as_mut_ptr(),
                        ldd as sys::da_int,
                        gamma,
                        coef0,
                    )
                };
                check_status("data-analytics", status)
            }
        }
    };
}

impl_kernel_scalar!(
    f32,
    da_linear_kernel_s,
    da_rbf_kernel_s,
    da_polynomial_kernel_s,
    da_sigmoid_kernel_s
);
impl_kernel_scalar!(
    f64,
    da_linear_kernel_d,
    da_rbf_kernel_d,
    da_polynomial_kernel_d,
    da_sigmoid_kernel_d
);

/// Linear kernel `K(x, y) = x · y` between rows of row-major matrices
/// `x` (`m × k`) and optionally `y` (`n × k`). Output `d` is `m × n`
/// (or `m × m` if `y` is `None`).
pub fn linear_kernel<T: KernelScalar>(
    m: usize,
    n: usize,
    k: usize,
    x: &[T],
    y: Option<&[T]>,
    d: &mut [T],
) -> Result<()> {
    let ldd = if y.is_none() { m } else { n };
    T::linear_kernel(Layout::RowMajor, m, n, k, x, k, y, k, d, ldd)
}

/// RBF kernel `K(x, y) = exp(-γ · ‖x − y‖²)` (row-major).
pub fn rbf_kernel<T: KernelScalar>(
    m: usize,
    n: usize,
    k: usize,
    x: &[T],
    y: Option<&[T]>,
    d: &mut [T],
    gamma: T,
) -> Result<()> {
    let ldd = if y.is_none() { m } else { n };
    T::rbf_kernel(Layout::RowMajor, m, n, k, x, k, y, k, d, ldd, gamma)
}

/// Polynomial kernel `K(x, y) = (γ · x · y + c)^degree` (row-major).
pub fn polynomial_kernel<T: KernelScalar>(
    m: usize,
    n: usize,
    k: usize,
    x: &[T],
    y: Option<&[T]>,
    d: &mut [T],
    gamma: T,
    degree: i64,
    coef0: T,
) -> Result<()> {
    let ldd = if y.is_none() { m } else { n };
    T::polynomial_kernel(
        Layout::RowMajor,
        m,
        n,
        k,
        x,
        k,
        y,
        k,
        d,
        ldd,
        gamma,
        degree,
        coef0,
    )
}

/// Sigmoid kernel `K(x, y) = tanh(γ · x · y + c)` (row-major).
pub fn sigmoid_kernel<T: KernelScalar>(
    m: usize,
    n: usize,
    k: usize,
    x: &[T],
    y: Option<&[T]>,
    d: &mut [T],
    gamma: T,
    coef0: T,
) -> Result<()> {
    let ldd = if y.is_none() { m } else { n };
    T::sigmoid_kernel(Layout::RowMajor, m, n, k, x, k, y, k, d, ldd, gamma, coef0)
}

/// Compute the pairwise distance matrix between rows of `x` (and
/// optionally rows of `y`) under the given `metric`.
///
/// `x` is `m × k` row-major; `y` (optional) is `n × k` row-major. When
/// `y` is `None`, distances are computed *within* `x` and the output
/// `d` is `m × m`. Otherwise `d` is `m × n`. `p` is the Minkowski
/// exponent (ignored for non-Minkowski metrics).
pub fn pairwise_distances<T: DistScalar>(
    m: usize,
    n: usize,
    k: usize,
    x: &[T],
    y: Option<&[T]>,
    d: &mut [T],
    p: T,
    metric: Metric,
) -> Result<()> {
    let ldd = if y.is_none() { m } else { n };
    let ldy = k; // row-major: ldy = k regardless of y being None
    T::pairwise_distances(Layout::RowMajor, m, n, k, x, k, y, ldy, d, ldd, p, metric)
}

// =========================================================================
//   da_handle + k-means safe wrapper
// =========================================================================

/// Type of analytics handle. AOCL-DA uses an opaque handle to track the
/// state of an algorithm (data, options, fitted model, predictions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HandleKind {
    Linmod,
    Pca,
    Kmeans,
    Dbscan,
    DecisionTree,
    DecisionForest,
    Knn,
    Svm,
    Nlls,
}

impl HandleKind {
    fn raw(self) -> sys::da_handle_type {
        match self {
            HandleKind::Linmod => sys::da_handle_type__da_handle_linmod,
            HandleKind::Pca => sys::da_handle_type__da_handle_pca,
            HandleKind::Kmeans => sys::da_handle_type__da_handle_kmeans,
            HandleKind::Dbscan => sys::da_handle_type__da_handle_dbscan,
            HandleKind::DecisionTree => sys::da_handle_type__da_handle_decision_tree,
            HandleKind::DecisionForest => sys::da_handle_type__da_handle_decision_forest,
            // AOCL 5.3 renamed k-NN to nearest neighbours (`da_handle_nn`, `da_nn_*`).
            HandleKind::Knn => sys::da_handle_type__da_handle_nn,
            HandleKind::Svm => sys::da_handle_type__da_handle_svm,
            HandleKind::Nlls => sys::da_handle_type__da_handle_nlls,
        }
    }
}

/// Internal precision selector for [`Handle`]: f64 (`Double`) or
/// f32 (`Single`). The choice determines which underlying `da_*_d` /
/// `da_*_s` C entry points are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Precision {
    Single,
    Double,
}

/// RAII wrapper around `da_handle`. Tracks the algorithm and precision
/// chosen at construction time.
pub struct Handle {
    raw: sys::da_handle,
    kind: HandleKind,
    precision: Precision,
}

unsafe impl Send for Handle {}

impl Handle {
    /// Create an `f64` handle for the given algorithm.
    pub fn new_double(kind: HandleKind) -> Result<Self> {
        let mut raw: sys::da_handle = std::ptr::null_mut();
        let status = unsafe { sys::da_handle_init_d(&mut raw, kind.raw()) };
        check_status("data-analytics", status)?;
        if raw.is_null() {
            return Err(Error::AllocationFailed("data-analytics"));
        }
        Ok(Self {
            raw,
            kind,
            precision: Precision::Double,
        })
    }

    /// Create an `f32` handle.
    pub fn new_single(kind: HandleKind) -> Result<Self> {
        let mut raw: sys::da_handle = std::ptr::null_mut();
        let status = unsafe { sys::da_handle_init_s(&mut raw, kind.raw()) };
        check_status("data-analytics", status)?;
        if raw.is_null() {
            return Err(Error::AllocationFailed("data-analytics"));
        }
        Ok(Self {
            raw,
            kind,
            precision: Precision::Single,
        })
    }

    /// Set an integer-valued option.
    pub fn set_int_option(&mut self, name: &str, value: i64) -> Result<()> {
        let cs = std::ffi::CString::new(name)
            .map_err(|e| Error::InvalidArgument(format!("set_int_option: {e}")))?;
        let status =
            unsafe { sys::da_options_set_int(self.raw, cs.as_ptr(), value as sys::da_int) };
        check_status("data-analytics", status)
    }

    /// Set a string-valued option.
    pub fn set_string_option(&mut self, name: &str, value: &str) -> Result<()> {
        let n = std::ffi::CString::new(name)
            .map_err(|e| Error::InvalidArgument(format!("set_string_option: {e}")))?;
        let v = std::ffi::CString::new(value)
            .map_err(|e| Error::InvalidArgument(format!("set_string_option: {e}")))?;
        let status = unsafe { sys::da_options_set_string(self.raw, n.as_ptr(), v.as_ptr()) };
        check_status("data-analytics", status)
    }

    /// Borrow the raw handle for routines this crate doesn't yet wrap.
    pub fn as_raw(&self) -> sys::da_handle {
        self.raw
    }

    /// Returns `(kind, precision)`.
    pub fn info(&self) -> (HandleKind, Precision) {
        (self.kind, self.precision)
    }

    /// Fetch the most recent detailed error message AOCL-DA stored on
    /// this handle, or `None` if there isn't one. Useful for diagnosing
    /// which option or input value the C library found objectionable.
    pub fn last_error_message(&self) -> Option<String> {
        let mut p: *mut std::os::raw::c_char = std::ptr::null_mut();
        let status = unsafe { sys::da_handle_get_error_message(self.raw, &mut p) };
        if status != sys::da_status__da_status_success || p.is_null() {
            return None;
        }
        let s = unsafe { std::ffi::CStr::from_ptr(p) }
            .to_string_lossy()
            .into_owned();
        // The library returns a freshly allocated buffer that the caller
        // must free with libc::free; we don't link libc here, so skip the
        // free. The leak is bounded (one per query) and not worth pulling
        // in libc just to plug.
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { sys::da_handle_destroy(&mut self.raw) };
            self.raw = std::ptr::null_mut();
        }
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("kind", &self.kind)
            .field("precision", &self.precision)
            .finish_non_exhaustive()
    }
}

/// k-means clustering wrapper. Holds an [`Handle`] specialised for
/// `da_handle_kmeans` and exposes the typical fit/predict interface.
pub struct KMeans {
    handle: Handle,
}

impl KMeans {
    /// Build a new k-means model with `n_clusters` clusters in `f64`
    /// precision.
    pub fn new(n_clusters: usize) -> Result<Self> {
        let mut handle = Handle::new_double(HandleKind::Kmeans)?;
        handle.set_int_option("n_clusters", n_clusters as i64)?;
        Ok(Self { handle })
    }

    /// Fit on `n_samples × n_features` data matrix in **column-major**
    /// layout (one column per feature; `lda = n_samples`). This matches
    /// AOCL-DA's default storage convention.
    pub fn fit(&mut self, n_samples: usize, n_features: usize, data: &[f64]) -> Result<()> {
        if data.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "fit: data length {} < n_samples·n_features = {}",
                data.len(),
                n_samples * n_features
            )));
        }
        let lda = n_samples;
        let status = unsafe {
            sys::da_kmeans_set_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                data.as_ptr(),
                lda as sys::da_int,
            )
        };
        check_status("data-analytics", status)?;
        let status = unsafe { sys::da_kmeans_compute_d(self.handle.raw) };
        check_status("data-analytics", status)
    }

    /// Predict cluster labels for new samples in column-major layout.
    pub fn predict(
        &mut self,
        k_samples: usize,
        k_features: usize,
        y: &[f64],
        labels: &mut [i32],
    ) -> Result<()> {
        if labels.len() < k_samples {
            return Err(Error::InvalidArgument(format!(
                "predict: labels length {} < k_samples = {k_samples}",
                labels.len()
            )));
        }
        let mut out = IntsOut::new(k_samples);
        let status = unsafe {
            sys::da_kmeans_predict_d(
                self.handle.raw,
                k_samples as sys::da_int,
                k_features as sys::da_int,
                y.as_ptr(),
                k_samples as sys::da_int,
                out.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        out.write_back(labels, k_samples)
    }
}

impl std::fmt::Debug for KMeans {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KMeans")
            .field("handle", &self.handle)
            .finish()
    }
}

/// Principal Component Analysis. Reduces an `n_samples × n_features`
/// matrix to a lower-dimensional representation.
pub struct Pca {
    handle: Handle,
}

impl Pca {
    /// Build a new PCA handle that retains `n_components` principal
    /// directions, in `f64` precision.
    pub fn new(n_components: usize) -> Result<Self> {
        let mut handle = Handle::new_double(HandleKind::Pca)?;
        handle.set_int_option("n_components", n_components as i64)?;
        Ok(Self { handle })
    }

    /// Fit on column-major `n_samples × n_features` data.
    pub fn fit(&mut self, n_samples: usize, n_features: usize, data: &[f64]) -> Result<()> {
        if data.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "PCA fit: data length {} < n_samples·n_features = {}",
                data.len(),
                n_samples * n_features
            )));
        }
        let lda = n_samples;
        let status = unsafe {
            sys::da_pca_set_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                data.as_ptr(),
                lda as sys::da_int,
            )
        };
        check_status("data-analytics", status)?;
        let status = unsafe { sys::da_pca_compute_d(self.handle.raw) };
        check_status("data-analytics", status)
    }

    /// Project `m_samples × m_features` new data into the fitted
    /// principal-component space. `out` receives the transformed
    /// `m_samples × n_components` matrix in column-major layout
    /// (`ldx_transform = m_samples`).
    pub fn transform(
        &mut self,
        m_samples: usize,
        m_features: usize,
        x: &[f64],
        n_components: usize,
        out: &mut [f64],
    ) -> Result<()> {
        if x.len() < m_samples * m_features {
            return Err(Error::InvalidArgument(format!(
                "PCA transform: x length {} < m_samples·m_features = {}",
                x.len(),
                m_samples * m_features
            )));
        }
        if out.len() < m_samples * n_components {
            return Err(Error::InvalidArgument(format!(
                "PCA transform: out length {} < m_samples·n_components = {}",
                out.len(),
                m_samples * n_components
            )));
        }
        let status = unsafe {
            sys::da_pca_transform_d(
                self.handle.raw,
                m_samples as sys::da_int,
                m_features as sys::da_int,
                x.as_ptr(),
                m_samples as sys::da_int,
                out.as_mut_ptr(),
                m_samples as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }
}

impl std::fmt::Debug for Pca {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pca").field("handle", &self.handle).finish()
    }
}

// ---------------------------------------------------------------------------
//   Width-correct integer buffers
// ---------------------------------------------------------------------------
//
// The safe API takes and returns `i32` labels, indices and class ids, but the library's `da_int` is 32-bit under LP64
// and 64-bit under ILP64. Handing it a pointer cast from an `i32` slice is only right for LP64 (under ILP64 it reads
// and writes 8 bytes per element: measured, an access violation), so every integer array goes through a `da_int`
// buffer and is narrowed back with a range check.

/// Copy `i32` values into a `da_int` buffer.
fn ints_in(x: &[i32]) -> Vec<sys::da_int> {
    x.iter().map(|&v| v as sys::da_int).collect()
}

/// A `da_int` output buffer that is narrowed back into the caller's `&mut [i32]`.
struct IntsOut {
    buf: Vec<sys::da_int>,
}

impl IntsOut {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0; len.max(1)],
        }
    }
    fn as_mut_ptr(&mut self) -> *mut sys::da_int {
        self.buf.as_mut_ptr()
    }
    /// Copy the first `n` results into `out`, failing if a value does not fit in `i32`.
    fn write_back(&self, out: &mut [i32], n: usize) -> Result<()> {
        for (o, &v) in out.iter_mut().zip(self.buf.iter()).take(n) {
            *o = i32::try_from(v).map_err(|_| {
                Error::InvalidArgument(format!(
                    "the library returned {v}, which does not fit in i32"
                ))
            })?;
        }
        Ok(())
    }
}

/// k-nearest-neighbours classifier.
pub struct KNearestNeighbours {
    handle: Handle,
    // `da_nn_set_data_d` / `da_nn_set_labels_d` store pointers, not copies, so the training data is owned here.
    train_x: Vec<f64>,
    train_y: Vec<sys::da_int>,
}

impl KNearestNeighbours {
    /// Build a new k-NN classifier in `f64` precision. The number of
    /// neighbours `k` defaults to AOCL-DA's built-in default (commonly
    /// 5); call [`KNearestNeighbours::handle_mut`] and
    /// `set_int_option` to override it (the option name is AOCL-version
    /// specific — try `"number of neighbours"`).
    pub fn new() -> Result<Self> {
        let handle = Handle::new_double(HandleKind::Knn)?;
        Ok(Self {
            handle,
            train_x: Vec::new(),
            train_y: Vec::new(),
        })
    }

    /// Borrow the underlying handle to set algorithm-specific options.
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Fit with `n_samples × n_features` training data (column-major)
    /// and a `n_samples` integer label vector.
    pub fn fit(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_train: &[f64],
        y_train: &[i32],
    ) -> Result<()> {
        if x_train.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "knn fit: x_train length {} < n_samples·n_features = {}",
                x_train.len(),
                n_samples * n_features
            )));
        }
        if y_train.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "knn fit: y_train length {} < n_samples = {n_samples}",
                y_train.len()
            )));
        }
        self.train_x = x_train[..n_samples * n_features].to_vec();
        self.train_y = ints_in(&y_train[..n_samples]);
        let status = unsafe {
            sys::da_nn_set_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                self.train_x.as_ptr(),
                n_samples as sys::da_int,
            )
        };
        check_status("data-analytics", status)?;
        let status = unsafe {
            sys::da_nn_set_labels_d(
                self.handle.raw,
                n_samples as sys::da_int,
                self.train_y.as_ptr(),
            )
        };
        check_status("data-analytics", status)
    }

    /// Predict labels for `n_queries × n_features` samples (column-major).
    pub fn predict(
        &mut self,
        n_queries: usize,
        n_features: usize,
        x_test: &[f64],
        labels: &mut [i32],
    ) -> Result<()> {
        if x_test.len() < n_queries * n_features {
            return Err(Error::InvalidArgument(format!(
                "knn predict: x_test length {} < n_queries·n_features = {}",
                x_test.len(),
                n_queries * n_features
            )));
        }
        if labels.len() < n_queries {
            return Err(Error::InvalidArgument(format!(
                "knn predict: labels length {} < n_queries = {n_queries}",
                labels.len()
            )));
        }
        let mut out = IntsOut::new(n_queries);
        let status = unsafe {
            sys::da_nn_classifier_predict_d(
                self.handle.raw,
                n_queries as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_queries as sys::da_int,
                out.as_mut_ptr(),
                sys::da_nn_search_mode__knn_search_mode,
            )
        };
        check_status("data-analytics", status)?;
        out.write_back(labels, n_queries)
    }

    /// For each of `n_queries` test rows, write into `n_ind` (length
    /// `n_queries · k`) the column-major indices of its `k` nearest
    /// training-set neighbours. If `return_distance` is true, also fill
    /// `n_dist` (same shape) with the distances.
    pub fn kneighbors(
        &mut self,
        n_queries: usize,
        n_features: usize,
        x_test: &[f64],
        n_ind: &mut [i32],
        n_dist: &mut [f64],
        k: usize,
        return_distance: bool,
    ) -> Result<()> {
        let mut out = IntsOut::new(n_ind.len());
        let status = unsafe {
            sys::da_nn_kneighbors_d(
                self.handle.raw,
                n_queries as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_queries as sys::da_int,
                out.as_mut_ptr(),
                n_dist.as_mut_ptr(),
                k as sys::da_int,
                if return_distance { 1 } else { 0 } as sys::da_int,
            )
        };
        check_status("data-analytics", status)?;
        out.write_back(n_ind, n_queries * k)
    }

    /// Retrieve the unique class labels seen during training, ascending,
    /// into `classes` (at most `classes.len()` of them). Returns the
    /// number of classes, which may exceed `classes.len()`.
    pub fn classes(&self, classes: &mut [i32]) -> Result<usize> {
        // `n_classes` is in/out: a value > 0 on entry is taken as the number of classes (the library then writes that
        // many entries, past its own array when the caller's buffer is merely large), so ask for the count with 0 first.
        let mut n_classes: sys::da_int = 0;
        let status =
            unsafe { sys::da_nn_classes_d(self.handle.raw, &mut n_classes, std::ptr::null_mut()) };
        check_status("data-analytics", status)?;
        let total = n_classes.max(0) as usize;
        if total == 0 {
            return Ok(0);
        }
        let mut out = IntsOut::new(total);
        let mut n_again: sys::da_int = total as sys::da_int;
        let status =
            unsafe { sys::da_nn_classes_d(self.handle.raw, &mut n_again, out.as_mut_ptr()) };
        check_status("data-analytics", status)?;
        out.write_back(classes, total.min(classes.len()))?;
        Ok(total)
    }

    /// Probability estimates: `n_queries × n_classes` column-major matrix
    /// at `proba` (element (query, class) at `class * n_queries + query`). Use [`KNearestNeighbours::classes`] to discover
    /// which class each column corresponds to.
    pub fn predict_proba(
        &mut self,
        n_queries: usize,
        n_features: usize,
        x_test: &[f64],
        proba: &mut [f64],
    ) -> Result<()> {
        let status = unsafe {
            sys::da_nn_classifier_predict_proba_d(
                self.handle.raw,
                n_queries as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_queries as sys::da_int,
                proba.as_mut_ptr(),
                sys::da_nn_search_mode__knn_search_mode,
            )
        };
        check_status("data-analytics", status)
    }
}

impl std::fmt::Debug for KNearestNeighbours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KNearestNeighbours")
            .field("handle", &self.handle)
            .finish()
    }
}

// =========================================================================
//   Linear models (linmod)
// =========================================================================

/// Family of linear model handled by [`Linmod`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LinmodKind {
    /// L2 (least-squares) linear regression.
    Mse,
    /// Logistic regression for classification.
    Logistic,
}

impl LinmodKind {
    fn raw(self) -> sys::linmod_model {
        match self {
            LinmodKind::Mse => sys::linmod_model__linmod_model_mse,
            LinmodKind::Logistic => sys::linmod_model__linmod_model_logistic,
        }
    }
}

/// Linear-model wrapper covering least-squares (MSE) regression and
/// logistic regression. Use [`Linmod::fit`] to fit the model on training
/// data, then [`Linmod::coefficients`] to inspect the fitted weights or
/// [`Linmod::predict`] / [`Linmod::evaluate`] to apply the model.
///
/// Data is laid out **column-major** by default (one column per feature,
/// `lda = n_samples`). To pass row-major data, call
/// [`Linmod::handle_mut`] and `set_string_option("storage order",
/// "row-major")` before fitting.
pub struct Linmod {
    handle: Handle,
    kind: LinmodKind,
    n_features_at_fit: Option<usize>,
}

impl Linmod {
    /// Build a new MSE (L2) regression model in `f64` precision.
    pub fn new_mse() -> Result<Self> {
        Self::new_with(LinmodKind::Mse)
    }

    /// Build a new logistic regression model in `f64` precision.
    pub fn new_logistic() -> Result<Self> {
        Self::new_with(LinmodKind::Logistic)
    }

    fn new_with(kind: LinmodKind) -> Result<Self> {
        let handle = Handle::new_double(HandleKind::Linmod)?;
        let status = unsafe { sys::da_linmod_select_model_d(handle.raw, kind.raw()) };
        check_status("data-analytics", status)?;
        Ok(Self {
            handle,
            kind,
            n_features_at_fit: None,
        })
    }

    /// Borrow the underlying handle to set algorithm-specific options
    /// (regularisation, solver choice, storage order, …).
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Family of linear model that this handle was constructed for.
    pub fn kind(&self) -> LinmodKind {
        self.kind
    }

    /// Fit the model on `n_samples × n_features` training data (column-major
    /// by default) with a length-`n_samples` response vector `y`.
    pub fn fit(&mut self, n_samples: usize, n_features: usize, x: &[f64], y: &[f64]) -> Result<()> {
        if x.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "linmod fit: x length {} < n_samples·n_features = {}",
                x.len(),
                n_samples * n_features
            )));
        }
        if y.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "linmod fit: y length {} < n_samples = {n_samples}",
                y.len()
            )));
        }
        let status = unsafe {
            sys::da_linmod_define_features_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x.as_ptr(),
                n_samples as sys::da_int, // ldx: column-major, the default storage order
                y.as_ptr(),
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("linmod define_features failed: {extra}"),
            });
        }
        let status = unsafe { sys::da_linmod_fit_d(self.handle.raw) };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("linmod fit failed: {extra}"),
            });
        }
        self.n_features_at_fit = Some(n_features);
        Ok(())
    }

    /// Retrieve the fitted model coefficients. For MSE regression with no
    /// intercept this is a vector of length `n_features`; with an intercept
    /// or for logistic regression it may be longer (the library decides
    /// the layout).
    pub fn coefficients(&self) -> Result<Vec<f64>> {
        // AOCL's size-discovery pattern: call with a buffer that may be
        // too small. On `invalid_array_dimension`, `dim` is updated with
        // the required size; resize and call again.
        let mut dim: sys::da_int = 1;
        let mut tmp = [0.0_f64; 1];
        let probe = unsafe {
            sys::da_handle_get_result_d(
                self.handle.raw,
                sys::da_result__da_linmod_coef,
                &mut dim,
                tmp.as_mut_ptr(),
            )
        };
        if probe == sys::da_status__da_status_success {
            return Ok(tmp[..dim as usize].to_vec());
        }
        if probe != sys::da_status__da_status_invalid_array_dimension {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: probe as i64,
                message: format!("linmod coefficients (probe): {extra}"),
            });
        }
        if dim <= 0 {
            return Err(Error::Status {
                component: "data-analytics",
                code: probe as i64,
                message: "linmod coefficients: solver did not report a size".into(),
            });
        }
        let mut out = vec![0.0_f64; dim as usize];
        let status = unsafe {
            sys::da_handle_get_result_d(
                self.handle.raw,
                sys::da_result__da_linmod_coef,
                &mut dim,
                out.as_mut_ptr(),
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("linmod coefficients: {extra}"),
            });
        }
        out.truncate(dim as usize);
        Ok(out)
    }

    /// Apply the model to `n_samples × n_features` new data, writing
    /// `n_samples` predicted values into `predictions`. For MSE regression
    /// these are continuous responses; for logistic regression they are
    /// integer class indices stored as floats.
    pub fn predict(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x: &[f64],
        predictions: &mut [f64],
    ) -> Result<()> {
        if let Some(expected) = self.n_features_at_fit {
            if expected != n_features {
                return Err(Error::InvalidArgument(format!(
                    "linmod predict: n_features {n_features} != fit-time {expected}"
                )));
            }
        }
        if x.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "linmod predict: x length {} < n_samples·n_features = {}",
                x.len(),
                n_samples * n_features
            )));
        }
        if predictions.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "linmod predict: predictions length {} < n_samples = {n_samples}",
                predictions.len()
            )));
        }
        let status = unsafe {
            sys::da_linmod_evaluate_model_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x.as_ptr(),
                n_samples as sys::da_int, // ldx: column-major, the default storage order
                predictions.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        check_status("data-analytics", status)
    }

    /// Apply the model and also compute the model loss against the
    /// supplied observations `y`. Returns the scalar loss.
    pub fn evaluate(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x: &[f64],
        y: &[f64],
        predictions: &mut [f64],
    ) -> Result<f64> {
        if y.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "linmod evaluate: y length {} < n_samples = {n_samples}",
                y.len()
            )));
        }
        if x.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "linmod evaluate: x length {} < n_samples·n_features = {}",
                x.len(),
                n_samples * n_features
            )));
        }
        if predictions.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "linmod evaluate: predictions length {} < n_samples = {n_samples}",
                predictions.len()
            )));
        }
        // The C API takes `observations` as *mut despite logically being
        // input; copy `y` into a local buffer to keep the callsite safe.
        let mut y_buf = y.to_vec();
        let mut loss = 0.0_f64;
        let status = unsafe {
            sys::da_linmod_evaluate_model_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x.as_ptr(),
                n_samples as sys::da_int, // ldx: column-major, the default storage order
                predictions.as_mut_ptr(),
                y_buf.as_mut_ptr(),
                &mut loss,
            )
        };
        check_status("data-analytics", status)?;
        Ok(loss)
    }
}

impl std::fmt::Debug for Linmod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Linmod")
            .field("kind", &self.kind)
            .field("handle", &self.handle)
            .finish()
    }
}

// =========================================================================
//   DBSCAN clustering
// =========================================================================

/// DBSCAN density-based clustering. Unlike k-means it does not require
/// the number of clusters in advance and can mark points as noise
/// (label `-1`).
///
/// Configure with options like `"eps"` and `"min_samples"` via
/// [`Dbscan::handle_mut`] before [`Dbscan::fit`]; defaults follow
/// scikit-learn's conventions.
pub struct Dbscan {
    handle: Handle,
    n_samples_at_fit: Option<usize>,
}

impl Dbscan {
    /// Build a new DBSCAN handle in `f64` precision.
    pub fn new() -> Result<Self> {
        let handle = Handle::new_double(HandleKind::Dbscan)?;
        Ok(Self {
            handle,
            n_samples_at_fit: None,
        })
    }

    /// Borrow the underlying handle to set DBSCAN-specific options:
    /// e.g. `set_string_option("metric", "euclidean")`,
    /// `set_int_option("min samples", 5)`, or for the real-valued
    /// `eps` use `da_options_set_real_d` directly via [`Handle::as_raw`].
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Fit on column-major `n_samples × n_features` data matrix.
    pub fn fit(&mut self, n_samples: usize, n_features: usize, data: &[f64]) -> Result<()> {
        if data.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "dbscan fit: data length {} < n_samples·n_features = {}",
                data.len(),
                n_samples * n_features
            )));
        }
        let lda = n_samples;
        let status = unsafe {
            sys::da_dbscan_set_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                data.as_ptr(),
                lda as sys::da_int,
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("dbscan set_data failed: {extra}"),
            });
        }
        let status = unsafe { sys::da_dbscan_compute_d(self.handle.raw) };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("dbscan compute failed: {extra}"),
            });
        }
        self.n_samples_at_fit = Some(n_samples);
        Ok(())
    }

    /// Number of clusters discovered (excludes noise).
    pub fn n_clusters(&self) -> Result<usize> {
        let mut dim: sys::da_int = 1;
        let mut buf = [0_i64 as sys::da_int; 1];
        let status = unsafe {
            sys::da_handle_get_result_int(
                self.handle.raw,
                sys::da_result__da_dbscan_n_clusters,
                &mut dim,
                buf.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        Ok(buf[0] as usize)
    }

    /// Number of *core* samples (points that have at least `min_samples`
    /// neighbours within distance `eps`).
    pub fn n_core_samples(&self) -> Result<usize> {
        let mut dim: sys::da_int = 1;
        let mut buf = [0_i64 as sys::da_int; 1];
        let status = unsafe {
            sys::da_handle_get_result_int(
                self.handle.raw,
                sys::da_result__da_dbscan_n_core_samples,
                &mut dim,
                buf.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        Ok(buf[0] as usize)
    }

    /// Labels assigned to each input sample. `-1` indicates noise; other
    /// values index a cluster `0 .. n_clusters()`.
    pub fn labels(&self) -> Result<Vec<i32>> {
        let n = self.n_samples_at_fit.ok_or_else(|| {
            Error::InvalidArgument("dbscan labels: model has not been fit yet".into())
        })?;
        let mut dim: sys::da_int = n as sys::da_int;
        let mut out: Vec<sys::da_int> = vec![0; n];
        let status = unsafe {
            sys::da_handle_get_result_int(
                self.handle.raw,
                sys::da_result__da_dbscan_labels,
                &mut dim,
                out.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        out.truncate(dim as usize);
        Ok(out.into_iter().map(|v| v as i32).collect())
    }

    /// Indices of the core samples (length = `n_core_samples()`).
    pub fn core_sample_indices(&self) -> Result<Vec<i32>> {
        let n_core = self.n_core_samples()?;
        let mut dim: sys::da_int = n_core as sys::da_int;
        let mut out: Vec<sys::da_int> = vec![0; n_core];
        if n_core == 0 {
            return Ok(Vec::new());
        }
        let status = unsafe {
            sys::da_handle_get_result_int(
                self.handle.raw,
                sys::da_result__da_dbscan_core_sample_indices,
                &mut dim,
                out.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        out.truncate(dim as usize);
        Ok(out.into_iter().map(|v| v as i32).collect())
    }
}

impl std::fmt::Debug for Dbscan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dbscan")
            .field("handle", &self.handle)
            .finish()
    }
}

// =========================================================================
//   Decision tree / forest
// =========================================================================

/// A single decision tree classifier. Configure tree-specific options
/// such as `"maximum depth"` or `"minimum samples per split"` via
/// [`DecisionTree::handle_mut`] before calling [`DecisionTree::fit`].
pub struct DecisionTree {
    handle: Handle,
}

impl DecisionTree {
    /// Build a new decision-tree handle in `f64` precision.
    pub fn new() -> Result<Self> {
        let handle = Handle::new_double(HandleKind::DecisionTree)?;
        Ok(Self { handle })
    }

    /// Borrow the underlying handle to set tree options.
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Fit on column-major `n_samples × n_features` features `x` with
    /// integer class labels `y`. `n_class = 0` lets AOCL infer it from
    /// the label distribution.
    pub fn fit(
        &mut self,
        n_samples: usize,
        n_features: usize,
        n_class: usize,
        x: &[f64],
        y: &[i32],
    ) -> Result<()> {
        if x.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "tree fit: x length {} < n_samples·n_features = {}",
                x.len(),
                n_samples * n_features
            )));
        }
        if y.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "tree fit: y length {} < n_samples = {n_samples}",
                y.len()
            )));
        }
        let lda = n_samples;
        let y_da = ints_in(&y[..n_samples]);
        let status = unsafe {
            sys::da_tree_set_training_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                n_class as sys::da_int,
                x.as_ptr(),
                lda as sys::da_int,
                y_da.as_ptr(),
                std::ptr::null(), // categorical_features: none (argument added in AOCL 5.3)
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("tree set_training_data failed: {extra}"),
            });
        }
        let status = unsafe { sys::da_tree_fit_d(self.handle.raw) };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("tree fit failed: {extra}"),
            });
        }
        Ok(())
    }

    /// Predict class labels for `n_samples × n_features` test data.
    pub fn predict(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        labels: &mut [i32],
    ) -> Result<()> {
        if x_test.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "tree predict: x_test length {} < n_samples·n_features = {}",
                x_test.len(),
                n_samples * n_features
            )));
        }
        if labels.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "tree predict: labels length {} < n_samples = {n_samples}",
                labels.len()
            )));
        }
        let mut out = IntsOut::new(n_samples);
        let status = unsafe {
            sys::da_tree_predict_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                out.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        out.write_back(labels, n_samples)
    }

    /// Compute prediction accuracy on a labelled test set: returns the
    /// fraction of correctly classified samples.
    pub fn score(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_test: &[i32],
    ) -> Result<f64> {
        if y_test.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "score: y_test length {} < n_samples = {n_samples}",
                y_test.len()
            )));
        }
        let y_da = ints_in(&y_test[..n_samples]);
        let mut acc = 0.0_f64;
        let status = unsafe {
            sys::da_tree_score_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_da.as_ptr(),
                &mut acc,
            )
        };
        check_status("data-analytics", status)?;
        Ok(acc)
    }

    /// Predict class probabilities for `n_samples × n_features` test
    /// data. `y_proba` is `n_samples × n_class` in the handle's storage
    /// order (column-major, as for the features): element (sample `i`,
    /// class `c`) is at `c * ldy + i` with `ldy` ≥ `n_samples`.
    pub fn predict_proba(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_proba: &mut [f64],
        n_class: usize,
        ldy: usize,
    ) -> Result<()> {
        let status = unsafe {
            sys::da_tree_predict_proba_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_proba.as_mut_ptr(),
                n_class as sys::da_int,
                ldy as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }

    /// Predict log class probabilities. See [`DecisionTree::predict_proba`].
    pub fn predict_log_proba(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_log_proba: &mut [f64],
        n_class: usize,
        ldy: usize,
    ) -> Result<()> {
        let status = unsafe {
            sys::da_tree_predict_log_proba_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_log_proba.as_mut_ptr(),
                n_class as sys::da_int,
                ldy as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }
}

impl std::fmt::Debug for DecisionTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecisionTree")
            .field("handle", &self.handle)
            .finish()
    }
}

/// A random-forest classifier. Configure forest options such as
/// `"number of trees"` or `"bootstrap"` via [`DecisionForest::handle_mut`]
/// before calling [`DecisionForest::fit`].
pub struct DecisionForest {
    handle: Handle,
}

impl DecisionForest {
    /// Build a new decision-forest handle in `f64` precision.
    pub fn new() -> Result<Self> {
        let handle = Handle::new_double(HandleKind::DecisionForest)?;
        Ok(Self { handle })
    }

    /// Borrow the underlying handle to set forest options.
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Fit on column-major `n_samples × n_features` features `x` with
    /// integer class labels `y`. Pass `n_class = 0` to let AOCL infer it.
    pub fn fit(
        &mut self,
        n_samples: usize,
        n_features: usize,
        n_class: usize,
        x: &[f64],
        y: &[i32],
    ) -> Result<()> {
        if x.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "forest fit: x length {} < n_samples·n_features = {}",
                x.len(),
                n_samples * n_features
            )));
        }
        if y.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "forest fit: y length {} < n_samples = {n_samples}",
                y.len()
            )));
        }
        let lda = n_samples;
        let y_da = ints_in(&y[..n_samples]);
        let status = unsafe {
            sys::da_forest_set_training_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                n_class as sys::da_int,
                x.as_ptr(),
                lda as sys::da_int,
                y_da.as_ptr(),
                std::ptr::null(), // categorical_features: none (argument added in AOCL 5.3)
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("forest set_training_data failed: {extra}"),
            });
        }
        let status = unsafe { sys::da_forest_fit_d(self.handle.raw) };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("forest fit failed: {extra}"),
            });
        }
        Ok(())
    }

    /// Predict class labels for `n_samples × n_features` test data.
    pub fn predict(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        labels: &mut [i32],
    ) -> Result<()> {
        if x_test.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "forest predict: x_test length {} < n_samples·n_features = {}",
                x_test.len(),
                n_samples * n_features
            )));
        }
        if labels.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "forest predict: labels length {} < n_samples = {n_samples}",
                labels.len()
            )));
        }
        let mut out = IntsOut::new(n_samples);
        let status = unsafe {
            sys::da_forest_predict_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                out.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)?;
        out.write_back(labels, n_samples)
    }

    /// Compute prediction accuracy on a labelled test set.
    pub fn score(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_test: &[i32],
    ) -> Result<f64> {
        if y_test.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "score: y_test length {} < n_samples = {n_samples}",
                y_test.len()
            )));
        }
        let y_da = ints_in(&y_test[..n_samples]);
        let mut acc = 0.0_f64;
        let status = unsafe {
            sys::da_forest_score_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_da.as_ptr(),
                &mut acc,
            )
        };
        check_status("data-analytics", status)?;
        Ok(acc)
    }

    /// Predict class probabilities for the random forest. See
    /// [`DecisionTree::predict_proba`].
    pub fn predict_proba(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_proba: &mut [f64],
        n_class: usize,
        ldy: usize,
    ) -> Result<()> {
        let status = unsafe {
            sys::da_forest_predict_proba_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_proba.as_mut_ptr(),
                n_class as sys::da_int,
                ldy as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }

    /// Predict log class probabilities for the random forest.
    pub fn predict_log_proba(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_log_proba: &mut [f64],
        n_class: usize,
        ldy: usize,
    ) -> Result<()> {
        let status = unsafe {
            sys::da_forest_predict_log_proba_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_log_proba.as_mut_ptr(),
                n_class as sys::da_int,
                ldy as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }
}

impl std::fmt::Debug for DecisionForest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecisionForest")
            .field("handle", &self.handle)
            .finish()
    }
}

// =========================================================================
//   Support Vector Machines (SVM)
// =========================================================================

/// Family of SVM model. Classification variants train against integer
/// class labels (passed as `f64`); regression variants train against a
/// real-valued response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SvmKind {
    /// C-regularised classification (`SVC`).
    Svc,
    /// Nu-regularised classification (`NuSVC`).
    NuSvc,
    /// Epsilon-regularised regression (`SVR`).
    Svr,
    /// Nu-regularised regression (`NuSVR`).
    NuSvr,
}

impl SvmKind {
    fn raw(self) -> sys::da_svm_model {
        match self {
            SvmKind::Svc => sys::da_svm_model__svc,
            SvmKind::NuSvc => sys::da_svm_model__nusvc,
            SvmKind::Svr => sys::da_svm_model__svr,
            SvmKind::NuSvr => sys::da_svm_model__nusvr,
        }
    }
}

/// Decision-function shape used for multi-class classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionShape {
    /// One-vs-Rest.
    Ovr,
    /// One-vs-One.
    Ovo,
}

impl DecisionShape {
    fn raw(self) -> sys::da_svm_decision_function_shape {
        match self {
            DecisionShape::Ovr => sys::da_svm_decision_function_shape__ovr,
            DecisionShape::Ovo => sys::da_svm_decision_function_shape__ovo,
        }
    }
}

/// Support vector machine wrapper covering both classification (`Svc`,
/// `NuSvc`) and regression (`Svr`, `NuSvr`) modes. Configure
/// SVM-specific options (kernel, C, epsilon, gamma, …) via
/// [`Svm::handle_mut`] before calling [`Svm::fit`].
pub struct Svm {
    handle: Handle,
    kind: SvmKind,
}

impl Svm {
    /// Build a C-SVC classifier in `f64` precision.
    pub fn new_svc() -> Result<Self> {
        Self::new_with(SvmKind::Svc)
    }
    /// Build a Nu-SVC classifier in `f64` precision.
    pub fn new_nu_svc() -> Result<Self> {
        Self::new_with(SvmKind::NuSvc)
    }
    /// Build an epsilon-SVR regressor in `f64` precision.
    pub fn new_svr() -> Result<Self> {
        Self::new_with(SvmKind::Svr)
    }
    /// Build a Nu-SVR regressor in `f64` precision.
    pub fn new_nu_svr() -> Result<Self> {
        Self::new_with(SvmKind::NuSvr)
    }

    fn new_with(kind: SvmKind) -> Result<Self> {
        let handle = Handle::new_double(HandleKind::Svm)?;
        let status = unsafe { sys::da_svm_select_model_d(handle.raw, kind.raw()) };
        check_status("data-analytics", status)?;
        Ok(Self { handle, kind })
    }

    /// Borrow the underlying handle to set SVM-specific options.
    pub fn handle_mut(&mut self) -> &mut Handle {
        &mut self.handle
    }

    /// Family of SVM model that this handle was constructed for.
    pub fn kind(&self) -> SvmKind {
        self.kind
    }

    /// Fit on column-major `n_samples × n_features` features `x` and a
    /// length-`n_samples` response vector `y`. For classification, `y`
    /// holds class indices `0..n_class-1` (as floats); for regression
    /// it holds the real target.
    pub fn fit(&mut self, n_samples: usize, n_features: usize, x: &[f64], y: &[f64]) -> Result<()> {
        if x.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "svm fit: x length {} < n_samples·n_features = {}",
                x.len(),
                n_samples * n_features
            )));
        }
        if y.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "svm fit: y length {} < n_samples = {n_samples}",
                y.len()
            )));
        }
        let lda = n_samples;
        let status = unsafe {
            sys::da_svm_set_data_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x.as_ptr(),
                lda as sys::da_int,
                y.as_ptr(),
            )
        };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("svm set_data failed: {extra}"),
            });
        }
        let status = unsafe { sys::da_svm_compute_d(self.handle.raw) };
        if status != sys::da_status__da_status_success {
            let extra = self.handle.last_error_message().unwrap_or_default();
            return Err(Error::Status {
                component: "data-analytics",
                code: status as i64,
                message: format!("svm compute failed: {extra}"),
            });
        }
        Ok(())
    }

    /// Predict labels (classification) or values (regression) for new
    /// data. `predictions` receives `n_samples` outputs.
    pub fn predict(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        predictions: &mut [f64],
    ) -> Result<()> {
        if x_test.len() < n_samples * n_features {
            return Err(Error::InvalidArgument(format!(
                "svm predict: x_test length {} < n_samples·n_features = {}",
                x_test.len(),
                n_samples * n_features
            )));
        }
        if predictions.len() < n_samples {
            return Err(Error::InvalidArgument(format!(
                "svm predict: predictions length {} < n_samples = {n_samples}",
                predictions.len()
            )));
        }
        let status = unsafe {
            sys::da_svm_predict_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                predictions.as_mut_ptr(),
            )
        };
        check_status("data-analytics", status)
    }

    /// Compute the model's score: accuracy for classification or `R²`
    /// for regression.
    pub fn score(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        y_test: &[f64],
    ) -> Result<f64> {
        let mut s = 0.0_f64;
        let status = unsafe {
            sys::da_svm_score_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                y_test.as_ptr(),
                &mut s,
            )
        };
        check_status("data-analytics", status)?;
        Ok(s)
    }

    /// Compute the SVM decision function on test data (classification
    /// only; for regression call [`Svm::predict`] directly). Output
    /// layout depends on `shape` and `n_class`; see AOCL docs for the
    /// exact size requirements.
    pub fn decision_function(
        &mut self,
        n_samples: usize,
        n_features: usize,
        x_test: &[f64],
        shape: DecisionShape,
        decision_values: &mut [f64],
        ldd: usize,
    ) -> Result<()> {
        let status = unsafe {
            sys::da_svm_decision_function_d(
                self.handle.raw,
                n_samples as sys::da_int,
                n_features as sys::da_int,
                x_test.as_ptr(),
                n_samples as sys::da_int,
                shape.raw(),
                decision_values.as_mut_ptr(),
                ldd as sys::da_int,
            )
        };
        check_status("data-analytics", status)
    }
}

impl std::fmt::Debug for Svm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Svm")
            .field("kind", &self.kind)
            .field("handle", &self.handle)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RALFit (the NLLS solver in `aocl-da.dll`) is not safe to run from several threads at once (measured on AOCL
    /// 6.0.68: concurrent solves crash with an access violation), so every test that fits an `Nlls` holds this lock.
    static NLLS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn nlls_guard() -> std::sync::MutexGuard<'static, ()> {
        NLLS_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn column_means_2x3() {
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut out = [0.0_f64; 3];
        mean(Axis::Col, 2, 3, &x, &mut out).unwrap();
        assert!((out[0] - 2.5).abs() < 1e-12);
        assert!((out[1] - 3.5).abs() < 1e-12);
        assert!((out[2] - 4.5).abs() < 1e-12);
    }

    #[test]
    fn row_means_2x3() {
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut out = [0.0_f64; 2];
        mean(Axis::Row, 2, 3, &x, &mut out).unwrap();
        assert!((out[0] - 2.0).abs() < 1e-12);
        assert!((out[1] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn variance_dof_conventions() {
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let mut mu = [0.0_f64; 1];
        let mut var = [0.0_f64; 1];

        variance(Axis::All, 1, 5, &x, -1, &mut mu, &mut var).unwrap();
        assert!((mu[0] - 3.0).abs() < 1e-12);
        assert!((var[0] - 2.0).abs() < 1e-12, "n-divisor var = {}", var[0]);

        variance(Axis::All, 1, 5, &x, 0, &mut mu, &mut var).unwrap();
        assert!(
            (var[0] - 2.5).abs() < 1e-12,
            "(n-1)-divisor var = {}",
            var[0]
        );

        variance(Axis::All, 1, 5, &x, 2, &mut mu, &mut var).unwrap();
        assert!(
            (var[0] - 5.0).abs() < 1e-12,
            "literal-divisor var = {}",
            var[0]
        );
    }

    #[test]
    fn dim_mismatch_is_error() {
        let x = [1.0_f64; 4];
        let mut out = [0.0_f64; 3];
        let err = mean(Axis::Col, 2, 3, &x, &mut out).unwrap_err();
        matches!(err, Error::InvalidArgument(_));
    }

    #[test]
    fn handle_init_destroy() {
        // Smoke test: handle creation + drop on each algorithm kind.
        let _h1 = Handle::new_double(HandleKind::Kmeans).unwrap();
        let _h2 = Handle::new_double(HandleKind::Pca).unwrap();
        let _h3 = Handle::new_single(HandleKind::Linmod).unwrap();
    }

    #[test]
    fn pca_two_components() {
        // 4 points in 3-D forming a clear 2-D plane: x and y vary,
        // z is small noise.
        // Stored column-major (column = feature).
        let data: Vec<f64> = vec![
            // feature 0 (x): values 1, 2, 3, 4
            1.0, 2.0, 3.0, 4.0, // feature 1 (y): values 4, 3, 2, 1
            4.0, 3.0, 2.0, 1.0, // feature 2 (z): tiny noise
            0.01, 0.02, 0.01, 0.02,
        ];
        let mut pca = Pca::new(2).unwrap();
        pca.fit(4, 3, &data).unwrap();

        let mut transformed = vec![0.0_f64; 4 * 2];
        pca.transform(4, 3, &data, 2, &mut transformed).unwrap();
        // First two PCs should capture nearly all the variance — the
        // transformed values should be non-trivial (not all zero).
        let nonzero = transformed.iter().any(|&v| v.abs() > 1e-6);
        assert!(nonzero, "PCA-transformed coordinates were all zero");
    }

    #[test]
    fn knn_two_class_separation() {
        // Two clusters in 2-D (column-major): label 0 around (0,0),
        // label 1 around (10,10).
        let x_train: Vec<f64> = vec![
            // feature 0
            0.0, 0.1, 0.2, 10.0, 10.1, 10.2, // feature 1
            0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
        ];
        let y_train = vec![0_i32, 0, 0, 1, 1, 1];
        let mut knn = KNearestNeighbours::new().unwrap();
        knn.fit(6, 2, &x_train, &y_train).unwrap();

        // A query point near (0, 0) → label 0; near (10, 10) → label 1.
        let x_test = vec![
            0.05, 9.95, // feature 0
            0.05, 10.05, // feature 1
        ];
        let mut labels = vec![0_i32; 2];
        knn.predict(2, 2, &x_test, &mut labels).unwrap();
        assert_eq!(labels[0], 0);
        assert_eq!(labels[1], 1);
    }

    #[test]
    fn decision_tree_two_class_separation() {
        // Same separable two-class setup as the KNN test.
        let x_train: Vec<f64> = vec![
            // feature 0
            0.0, 0.1, 0.2, 10.0, 10.1, 10.2, // feature 1
            0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
        ];
        let y_train = vec![0_i32, 0, 0, 1, 1, 1];
        let mut tree = DecisionTree::new().unwrap();
        tree.fit(6, 2, 2, &x_train, &y_train).unwrap();

        let x_test = vec![0.05, 9.95, 0.05, 10.05];
        let mut labels = vec![0_i32; 2];
        tree.predict(2, 2, &x_test, &mut labels).unwrap();
        assert_eq!(labels[0], 0);
        assert_eq!(labels[1], 1);

        // Score on the training set should be 1.0 for this separable problem.
        let acc = tree.score(6, 2, &x_train, &y_train).unwrap();
        assert!((acc - 1.0).abs() < 1e-9, "score = {acc}");
    }

    #[test]
    fn decision_forest_two_class_separation() {
        let x_train: Vec<f64> = vec![
            0.0, 0.1, 0.2, 10.0, 10.1, 10.2, 0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
        ];
        let y_train = vec![0_i32, 0, 0, 1, 1, 1];
        let mut forest = DecisionForest::new().unwrap();
        forest.fit(6, 2, 2, &x_train, &y_train).unwrap();

        let x_test = vec![0.05, 9.95, 0.05, 10.05];
        let mut labels = vec![0_i32; 2];
        forest.predict(2, 2, &x_test, &mut labels).unwrap();
        assert_eq!(labels[0], 0);
        assert_eq!(labels[1], 1);

        let acc = forest.score(6, 2, &x_train, &y_train).unwrap();
        assert!((acc - 1.0).abs() < 1e-9, "score = {acc}");
    }

    #[test]
    fn datastore_init_destroy_and_options() {
        let mut store = Datastore::new().unwrap();
        // Setting a few standard options should succeed even on an
        // empty store.
        store.set_int_option("use header row", 1).unwrap();
        store.set_int_option("whitespace delimiter", 1).unwrap();
        store.set_string_option("comment", "#").unwrap();
        // dims() should report (0, 0) before any data is loaded.
        let (r, c) = store.dims().unwrap();
        assert_eq!((r, c), (0, 0));
    }

    #[test]
    fn datastore_round_trip_through_temp_csv() {
        // Write a minimal numeric CSV to a tempfile, load it via the
        // datastore, select all columns, and extract.
        use std::io::Write;
        let mut path = std::env::temp_dir();
        path.push(format!("aocl_da_csv_{}.csv", std::process::id()));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            // 3 rows × 2 cols; no header.
            writeln!(f, "1.0,2.0").unwrap();
            writeln!(f, "3.0,4.0").unwrap();
            writeln!(f, "5.0,6.0").unwrap();
        }

        let mut store = Datastore::new().unwrap();
        store.load_csv(path.to_str().unwrap()).unwrap();
        let (r, c) = store.dims().unwrap();
        assert_eq!((r, c), (3, 2));

        store.select_columns("all", 0, 1).unwrap();
        let mut buf = vec![0.0_f64; 6];
        // Column-major layout, lddata = n_rows.
        store
            .extract_f64("all", Layout::ColMajor, &mut buf, 3)
            .unwrap();
        // Column 0 = [1, 3, 5]; column 1 = [2, 4, 6].
        assert!((buf[0] - 1.0).abs() < 1e-12);
        assert!((buf[1] - 3.0).abs() < 1e-12);
        assert!((buf[2] - 5.0).abs() < 1e-12);
        assert!((buf[3] - 2.0).abs() < 1e-12);
        assert!((buf[4] - 4.0).abs() < 1e-12);
        assert!((buf[5] - 6.0).abs() < 1e-12);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn nlls_fits_exponential_decay() {
        let _lock = nlls_guard();
        // Fit y(t) = a · exp(-b · t) to noiseless data with a=2, b=0.3.
        // 5 sample points; residuals = predictions - observed.
        let t: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let true_a = 2.0_f64;
        let true_b = 0.3_f64;
        let y: Vec<f64> = t.iter().map(|&ti| true_a * (-true_b * ti).exp()).collect();

        // Closure captures t and y by reference. resfun computes
        // r_i(x) = a · exp(-b · t_i) - y_i, where x = [a, b].
        let n_coef = 2;
        let n_res = t.len();
        let resfun = |x: &[f64], res: &mut [f64]| -> i32 {
            let a = x[0];
            let b = x[1];
            for i in 0..res.len() {
                res[i] = a * (-b * t[i]).exp() - y[i];
            }
            0
        };

        let mut model = Nlls::new(n_coef, n_res, resfun).unwrap();
        // Start far from the truth to give the optimiser real work.
        let mut coef = vec![1.0_f64, 1.0_f64];
        model.fit(&mut coef).unwrap();

        // Should recover the true parameters within a tight tolerance.
        assert!((coef[0] - true_a).abs() < 1e-4, "a = {}", coef[0]);
        assert!((coef[1] - true_b).abs() < 1e-4, "b = {}", coef[1]);

        // Solver info should report nonzero iterations.
        let rinfo = model.info().unwrap();
        assert!(!rinfo.is_empty());
        // info[2] is iter count per the C header; should be > 0.
        assert!(rinfo[2] > 0.0, "iterations = {}", rinfo[2]);
    }

    #[test]
    fn linear_kernel_self_gram() {
        // X = [[1, 0], [0, 1], [1, 1]] row-major; Gram = X·Xᵀ
        // = [[1, 0, 1], [0, 1, 1], [1, 1, 2]]
        let x = [1.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
        let mut d = [0.0_f64; 9];
        linear_kernel::<f64>(3, 0, 2, &x, None, &mut d).unwrap();
        let want = [1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 2.0];
        for (got, w) in d.iter().zip(want.iter()) {
            assert!((got - w).abs() < 1e-12, "got {got}, want {w}");
        }
    }

    #[test]
    fn rbf_kernel_diagonal_is_one() {
        // RBF kernel of any vector with itself is exp(0) = 1.
        // Two-point set; check the diagonal entries.
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2 rows × 3 cols
        let mut d = [0.0_f64; 4];
        rbf_kernel::<f64>(2, 0, 3, &x, None, &mut d, 0.5).unwrap();
        // d[0] (= K(x0, x0)) and d[3] (= K(x1, x1)) should be 1.
        assert!((d[0] - 1.0).abs() < 1e-12, "K(x0,x0) = {}", d[0]);
        assert!((d[3] - 1.0).abs() < 1e-12, "K(x1,x1) = {}", d[3]);
        // Off-diagonal should be < 1.
        assert!(d[1] < 1.0);
        assert!(d[2] < 1.0);
    }

    #[test]
    fn pairwise_distances_within_set() {
        // Three 2-D points on the corners of a 3-4-5 right triangle:
        // p0 = (0, 0); p1 = (3, 0); p2 = (0, 4)
        // Row-major 3×2 matrix.
        let x = [0.0_f64, 0.0, 3.0, 0.0, 0.0, 4.0];
        let mut d = [0.0_f64; 9];
        pairwise_distances::<f64>(3, 0, 2, &x, None, &mut d, 2.0, Metric::Euclidean).unwrap();
        // d is 3×3 row-major; diagonal = 0, off-diagonals are 3, 4, 5.
        assert!(d[0].abs() < 1e-12);
        assert!((d[1] - 3.0).abs() < 1e-9, "d[0,1] = {}", d[1]);
        assert!((d[2] - 4.0).abs() < 1e-9, "d[0,2] = {}", d[2]);
        assert!((d[3] - 3.0).abs() < 1e-9);
        assert!(d[4].abs() < 1e-12);
        assert!((d[5] - 5.0).abs() < 1e-9, "d[1,2] = {}", d[5]);
    }

    #[test]
    fn quantile_median_of_1_to_5() {
        // Row-major 1×5 row vector; median (q=0.5) should be 3.
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let mut q = [0.0_f64; 1];
        quantile(Axis::All, 1, 5, &x, 0.5, &mut q, QuantileType::Type7).unwrap();
        assert!((q[0] - 3.0).abs() < 1e-12, "median = {}", q[0]);
    }

    #[test]
    fn five_point_summary_works() {
        // 1×5 row vector — five-number summary should give 1, ?, 3, ?, 5.
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let mut min = [0.0_f64; 1];
        let mut lower = [0.0_f64; 1];
        let mut med = [0.0_f64; 1];
        let mut upper = [0.0_f64; 1];
        let mut max = [0.0_f64; 1];
        five_point_summary(
            Axis::All,
            1,
            5,
            &x,
            &mut min,
            &mut lower,
            &mut med,
            &mut upper,
            &mut max,
        )
        .unwrap();
        assert!((min[0] - 1.0).abs() < 1e-12);
        assert!((max[0] - 5.0).abs() < 1e-12);
        assert!((med[0] - 3.0).abs() < 1e-12);
        // Hinges should sit between min and median, median and max.
        assert!(lower[0] >= min[0] && lower[0] <= med[0]);
        assert!(upper[0] >= med[0] && upper[0] <= max[0]);
    }

    /// Rows (1,2), (2,1), (3,4), (4,3): both column means are 2.5, so with the (n-1) divisor (`dof = 0`) both variances
    /// are 5/3 and the covariance is 1. If the data were wrongly treated as already centred (AOCL 5.3's
    /// `assume_centered = 1`) the diagonal would be 10 and the off-diagonal 11/3 instead.
    #[test]
    fn covariance_matrix_subtracts_column_means() {
        let x = [1.0_f64, 2.0, 2.0, 1.0, 3.0, 4.0, 4.0, 3.0];
        let mut cov = [0.0_f64; 4];
        covariance_matrix(4, 2, &x, 0, &mut cov).unwrap();
        let want = [5.0 / 3.0, 1.0, 1.0, 5.0 / 3.0];
        for (g, w) in cov.iter().zip(want) {
            assert!((g - w).abs() < 1e-12, "cov = {cov:?}, want {want:?}");
        }
    }

    #[test]
    fn correlation_matrix_identity_for_perfectly_correlated() {
        // Two perfectly correlated columns: y = 2*x + 1. Correlation
        // matrix is the 2x2 all-ones matrix.
        // Row-major: 4 rows, 2 columns.
        let x = [1.0_f64, 3.0, 2.0, 5.0, 3.0, 7.0, 4.0, 9.0];
        let mut corr = [0.0_f64; 4];
        correlation_matrix(4, 2, &x, &mut corr).unwrap();
        for v in &corr {
            assert!((v - 1.0).abs() < 1e-9, "off-diagonal correlation = {}", v);
        }
    }

    #[test]
    fn skewness_kurtosis_normal_data_near_zero() {
        // Symmetric data → skewness ≈ 0; mesokurtic ≈ 0 (Fisher).
        let x = [1.0_f64, 2.0, 3.0, 4.0, 5.0]; // 1×5 row vector
        let mut mu = [0.0_f64; 1];
        let mut var = [0.0_f64; 1];
        let mut skew = [0.0_f64; 1];
        skewness(Axis::All, 1, 5, &x, &mut mu, &mut var, &mut skew).unwrap();
        assert!((mu[0] - 3.0).abs() < 1e-12);
        assert!(skew[0].abs() < 1e-9, "skewness = {}", skew[0]);

        let mut kurt = [0.0_f64; 1];
        kurtosis(Axis::All, 1, 5, &x, &mut mu, &mut var, &mut kurt).unwrap();
        // Uniform discrete sample like 1..5 has Fisher kurtosis = -1.3.
        assert!(kurt[0] < 0.0, "kurtosis = {}", kurt[0]);
    }

    #[test]
    fn svm_svc_two_class_separation() {
        // Same separable two-class setup; SVM uses f64 labels (class
        // indices stored as floats).
        let x_train: Vec<f64> = vec![
            0.0, 0.1, 0.2, 10.0, 10.1, 10.2, 0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
        ];
        let y_train = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
        let mut svm = Svm::new_svc().unwrap();
        svm.fit(6, 2, &x_train, &y_train).unwrap();

        let x_test = vec![0.05, 9.95, 0.05, 10.05];
        let mut predictions = vec![0.0_f64; 2];
        svm.predict(2, 2, &x_test, &mut predictions).unwrap();
        assert!(
            (predictions[0] - 0.0).abs() < 1e-9,
            "pred[0] = {}",
            predictions[0]
        );
        assert!(
            (predictions[1] - 1.0).abs() < 1e-9,
            "pred[1] = {}",
            predictions[1]
        );

        let acc = svm.score(6, 2, &x_train, &y_train).unwrap();
        assert!((acc - 1.0).abs() < 1e-9, "score = {acc}");
    }

    #[test]
    fn dbscan_two_clusters() {
        // Two well-separated clusters; should yield n_clusters = 2 and
        // each cluster's samples sharing a positive label.
        let data: Vec<f64> = vec![
            // feature 0 (column 0): cluster A near 0, cluster B near 10
            0.0, 0.1, 0.2, 0.0, 0.1, 10.0, 10.1, 10.0, 10.2, 10.1, // feature 1 (column 1)
            0.0, 0.1, 0.0, 0.2, 0.1, 10.0, 10.1, 10.2, 10.0, 10.1,
        ];
        let mut db = Dbscan::new().unwrap();
        // Set min_samples small so a 5-point cluster qualifies; eps the
        // default (which is ~0.5 in AOCL) is not enough so widen it.
        // We use the underlying real-valued setter directly.
        let cs = std::ffi::CString::new("eps").unwrap();
        let status =
            unsafe { sys::da_options_set_real_d(db.handle_mut().as_raw(), cs.as_ptr(), 1.0) };
        assert_eq!(status, sys::da_status__da_status_success);
        db.handle_mut().set_int_option("min samples", 3).unwrap();
        db.fit(10, 2, &data).unwrap();
        assert_eq!(db.n_clusters().unwrap(), 2);
        let labels = db.labels().unwrap();
        // First five samples share a label, last five share another.
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert_eq!(labels[5], labels[6]);
        assert_eq!(labels[6], labels[9]);
        assert_ne!(labels[0], labels[5]);
        assert!(db.n_core_samples().unwrap() > 0);
    }

    #[test]
    fn linmod_mse_recovers_linear_relationship() {
        // y = 2 * x0 + 3 * x1 + small intercept; one feature varies with
        // each axis. Column-major: feature 0 first, then feature 1.
        // Use 5 samples to give the solver something to work with.
        let x: Vec<f64> = vec![
            // feature 0
            1.0, 2.0, 3.0, 4.0, 5.0, // feature 1
            5.0, 4.0, 3.0, 2.0, 1.0,
        ];
        // y = 2·x0 + 3·x1 (no noise)
        let y: Vec<f64> = vec![17.0, 16.0, 15.0, 14.0, 13.0];
        let mut model = Linmod::new_mse().unwrap();
        model.fit(5, 2, &x, &y).unwrap();
        let coefs = model.coefficients().unwrap();
        assert!(!coefs.is_empty(), "fitted coefficients vector was empty");
        // The first two entries should approximate the true coefficients.
        // AOCL may report them with small floating-point noise.
        assert!((coefs[0] - 2.0).abs() < 1e-6, "coef[0] = {}", coefs[0]);
        assert!((coefs[1] - 3.0).abs() < 1e-6, "coef[1] = {}", coefs[1]);

        // Predict on the same samples — should reproduce y nearly exactly.
        let mut pred = vec![0.0_f64; 5];
        model.predict(5, 2, &x, &mut pred).unwrap();
        for (got, want) in pred.iter().zip(y.iter()) {
            assert!((got - want).abs() < 1e-6, "predict {} vs {}", got, want);
        }

        // evaluate() produces a loss; the loss for an exact fit is small
        // (zero up to floating-point noise).
        let mut pred2 = vec![0.0_f64; 5];
        let loss = model.evaluate(5, 2, &x, &y, &mut pred2).unwrap();
        assert!(loss.abs() < 1e-6, "exact-fit loss = {}", loss);
    }

    #[test]
    fn kmeans_two_clusters() {
        // Two well-separated clusters: rows 0..=2 near (0, 0); rows
        // 3..=5 near (10, 10). Stored column-major: feature 0 first,
        // then feature 1.
        let data: Vec<f64> = vec![
            // feature 0 (column 0)
            0.0, 0.1, 0.2, 10.0, 10.1, 10.2, // feature 1 (column 1)
            0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
        ];
        let mut km = KMeans::new(2).unwrap();
        km.fit(6, 2, &data).unwrap();

        // Predict on the same six samples; rows 0..=2 should share a
        // label, rows 3..=5 the other.
        let mut labels = vec![0_i32; 6];
        km.predict(6, 2, &data, &mut labels).unwrap();
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);
        assert_eq!(labels[3], labels[4]);
        assert_eq!(labels[4], labels[5]);
        assert_ne!(labels[0], labels[3]);
    }

    // ---- reference helpers: naive formulas, independent of AOCL ----

    fn near(a: f64, b: f64, tol: f64, what: &str) {
        assert!(
            (a - b).abs() <= tol * (1.0 + b.abs()),
            "{what}: got {a}, want {b}"
        );
    }

    /// Column `j` (axis = Col) or row `i` (axis = Row) of a row-major `r x c` matrix.
    fn slices(x: &[f64], r: usize, c: usize, axis: Axis) -> Vec<Vec<f64>> {
        match axis {
            Axis::Col => (0..c)
                .map(|j| (0..r).map(|i| x[i * c + j]).collect())
                .collect(),
            Axis::Row => (0..r).map(|i| x[i * c..(i + 1) * c].to_vec()).collect(),
            _ => unreachable!(),
        }
    }

    const X23: [f64; 6] = [1.0, 2.0, 4.0, 4.0, 8.0, 16.0]; // row-major 2 x 3, all positive

    #[test]
    fn geometric_mean_matches_exp_mean_log_f64_and_f32() {
        for axis in [Axis::Col, Axis::Row] {
            let want: Vec<f64> = slices(&X23, 2, 3, axis)
                .iter()
                .map(|v| (v.iter().map(|x| x.ln()).sum::<f64>() / v.len() as f64).exp())
                .collect();
            let mut out = vec![0.0_f64; want.len()];
            geometric_mean(axis, 2, 3, &X23, &mut out).unwrap();
            for (g, w) in out.iter().zip(&want) {
                near(*g, *w, 1e-12, "f64 geometric mean");
            }
            let x32: Vec<f32> = X23.iter().map(|&v| v as f32).collect();
            let mut out32 = vec![0.0_f32; want.len()];
            geometric_mean(axis, 2, 3, &x32, &mut out32).unwrap();
            for (g, w) in out32.iter().zip(&want) {
                near(*g as f64, *w, 1e-5, "f32 geometric mean");
            }
        }
        // Closed form: columns of [[1,2,4],[4,8,16]] have geometric means sqrt(4)=2, sqrt(16)=4, sqrt(64)=8.
        let mut out = [0.0_f64; 3];
        geometric_mean(Axis::Col, 2, 3, &X23, &mut out).unwrap();
        for (g, w) in out.iter().zip([2.0, 4.0, 8.0]) {
            near(*g, w, 1e-12, "closed-form geometric mean");
        }
    }

    #[test]
    fn harmonic_mean_matches_n_over_sum_of_reciprocals_f64_and_f32() {
        for axis in [Axis::Col, Axis::Row] {
            let want: Vec<f64> = slices(&X23, 2, 3, axis)
                .iter()
                .map(|v| v.len() as f64 / v.iter().map(|x| 1.0 / x).sum::<f64>())
                .collect();
            let mut out = vec![0.0_f64; want.len()];
            harmonic_mean(axis, 2, 3, &X23, &mut out).unwrap();
            for (g, w) in out.iter().zip(&want) {
                near(*g, *w, 1e-12, "f64 harmonic mean");
            }
            let x32: Vec<f32> = X23.iter().map(|&v| v as f32).collect();
            let mut out32 = vec![0.0_f32; want.len()];
            harmonic_mean(axis, 2, 3, &x32, &mut out32).unwrap();
            for (g, w) in out32.iter().zip(&want) {
                near(*g as f64, *w, 1e-5, "f32 harmonic mean");
            }
        }
        // Closed form: harmonic mean of {1, 4} = 2 / (1 + 1/4) = 1.6.
        let mut out = [0.0_f64; 3];
        harmonic_mean(Axis::Col, 2, 3, &X23, &mut out).unwrap();
        near(out[0], 1.6, 1e-12, "harmonic mean of column 0");
    }

    #[test]
    fn moment_matches_central_moment_definition_f64_and_f32() {
        let x = [
            1.0_f64, 2.0, 4.0, 8.0, 3.0, 5.0, 7.0, 11.0, 6.0, 9.0, 10.0, 14.0,
        ]; // 4 x 3
        for axis in [Axis::Col, Axis::Row] {
            let sl = slices(&x, 4, 3, axis);
            for k in [2_i64, 3] {
                let means: Vec<f64> = sl
                    .iter()
                    .map(|v| v.iter().sum::<f64>() / v.len() as f64)
                    .collect();
                let want: Vec<f64> = sl
                    .iter()
                    .zip(&means)
                    .map(|(v, m)| {
                        v.iter().map(|x| (x - m).powi(k as i32)).sum::<f64>() / v.len() as f64
                    })
                    .collect();
                let mut mean_out = vec![0.0_f64; sl.len()];
                let mut mom = vec![0.0_f64; sl.len()];
                moment(axis, 4, 3, &x, k, false, &mut mean_out, &mut mom).unwrap();
                for j in 0..sl.len() {
                    near(mean_out[j], means[j], 1e-12, "moment: mean output");
                    near(mom[j], want[j], 1e-10, "f64 central moment");
                }
                // Precomputed means supplied: same result.
                let mut mom2 = vec![0.0_f64; sl.len()];
                let mut means_in = means.clone();
                moment(axis, 4, 3, &x, k, true, &mut means_in, &mut mom2).unwrap();
                for j in 0..sl.len() {
                    near(
                        mom2[j],
                        want[j],
                        1e-10,
                        "f64 central moment, precomputed mean",
                    );
                }
                let x32: Vec<f32> = x.iter().map(|&v| v as f32).collect();
                let mut m32 = vec![0.0_f32; sl.len()];
                let mut mom32 = vec![0.0_f32; sl.len()];
                moment(axis, 4, 3, &x32, k, false, &mut m32, &mut mom32).unwrap();
                for j in 0..sl.len() {
                    near(mom32[j] as f64, want[j], 1e-4, "f32 central moment");
                }
            }
        }
    }

    #[test]
    fn moment_rejects_short_output() {
        let mut m = [0.0_f64; 1];
        let mut o = [0.0_f64; 1];
        assert!(moment(Axis::Col, 2, 3, &X23, 2, false, &mut m, &mut o).is_err());
    }

    fn dot(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    #[test]
    fn polynomial_and_sigmoid_kernels_match_definitions_f64_and_f32() {
        let x = [1.0_f64, 0.5, -1.0, 2.0, 0.0, 1.5]; // 3 x 2 row-major
        let y = [0.5_f64, -0.5, 1.0, 1.0]; // 2 x 2
        let (gamma, coef0, degree) = (0.5_f64, 1.0_f64, 3_i64);
        let rows = |m: &[f64], n: usize| -> Vec<Vec<f64>> {
            (0..n).map(|i| m[i * 2..i * 2 + 2].to_vec()).collect()
        };
        let (xr, yr) = (rows(&x, 3), rows(&y, 2));
        // Cross kernel (3 x 2) and self kernel (3 x 3).
        for with_y in [true, false] {
            let (n, other): (usize, &Vec<Vec<f64>>) = if with_y { (2, &yr) } else { (3, &xr) };
            let ys: Option<&[f64]> = if with_y { Some(&y) } else { None };
            let mut pd = vec![0.0_f64; 3 * n];
            polynomial_kernel(3, n, 2, &x, ys, &mut pd, gamma, degree, coef0).unwrap();
            let mut sg = vec![0.0_f64; 3 * n];
            sigmoid_kernel(3, n, 2, &x, ys, &mut sg, gamma, coef0).unwrap();
            let x32: Vec<f32> = x.iter().map(|&v| v as f32).collect();
            let y32: Vec<f32> = y.iter().map(|&v| v as f32).collect();
            let ys32: Option<&[f32]> = if with_y { Some(&y32) } else { None };
            let mut pd32 = vec![0.0_f32; 3 * n];
            polynomial_kernel(
                3,
                n,
                2,
                &x32,
                ys32,
                &mut pd32,
                gamma as f32,
                degree,
                coef0 as f32,
            )
            .unwrap();
            let mut sg32 = vec![0.0_f32; 3 * n];
            sigmoid_kernel(3, n, 2, &x32, ys32, &mut sg32, gamma as f32, coef0 as f32).unwrap();
            for i in 0..3 {
                for j in 0..n {
                    let d = dot(&xr[i], &other[j]);
                    let wp = (gamma * d + coef0).powi(degree as i32);
                    let ws = (gamma * d + coef0).tanh();
                    near(pd[i * n + j], wp, 1e-12, "polynomial kernel f64");
                    near(sg[i * n + j], ws, 1e-12, "sigmoid kernel f64");
                    near(pd32[i * n + j] as f64, wp, 1e-4, "polynomial kernel f32");
                    near(sg32[i * n + j] as f64, ws, 1e-5, "sigmoid kernel f32");
                }
            }
        }
    }

    #[test]
    fn datastore_n_cols_and_select_rows() {
        use std::io::Write;
        let mut path = std::env::temp_dir();
        path.push(format!("aocl_da_rows_{}.csv", std::process::id()));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "1.0,2.0,3.0").unwrap();
            writeln!(f, "4.0,5.0,6.0").unwrap();
            writeln!(f, "7.0,8.0,9.0").unwrap();
            writeln!(f, "10.0,11.0,12.0").unwrap();
        }
        let mut store = Datastore::new().unwrap();
        assert_eq!(store.n_cols().unwrap(), 0); // empty store (control)
        store.load_csv(path.to_str().unwrap()).unwrap();
        assert_eq!(store.n_cols().unwrap(), 3);
        assert_eq!(store.n_rows().unwrap(), 4);
        // Rows 1..=2 (zero-based, inclusive): [[4,5,6],[7,8,9]].
        store.select_rows("mid", 1, 2).unwrap();
        let mut buf = vec![0.0_f64; 6];
        store
            .extract_f64("mid", Layout::RowMajor, &mut buf, 3)
            .unwrap();
        assert_eq!(buf, [4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        // Out-of-range selection is an error.
        assert!(store.select_rows("bad", 0, 10).is_err());
        let _ = std::fs::remove_file(&path);
    }

    fn decay_data() -> (Vec<f64>, Vec<f64>) {
        let t: Vec<f64> = (0..8).map(|i| i as f64 * 0.5).collect();
        let y: Vec<f64> = t.iter().map(|&ti| 2.0 * (-0.3 * ti).exp()).collect();
        (t, y)
    }

    #[test]
    fn nlls_with_analytic_jacobian_recovers_parameters() {
        let _lock = nlls_guard();
        let (t, y) = decay_data();
        let (t2, y2) = (t.clone(), y.clone());
        let model = Nlls::new(2, t.len(), move |x: &[f64], r: &mut [f64]| {
            for i in 0..r.len() {
                r[i] = x[0] * (-x[1] * t2[i]).exp() - y2[i];
            }
            0
        })
        .unwrap();
        let t3 = t.clone();
        let jac_calls = std::cell::Cell::new(0usize);
        let mut model = model
            .with_jacobian(|x: &[f64], j: &mut [f64]| {
                jac_calls.set(jac_calls.get() + 1);
                // Row-major n_res x n_coef: d r_i / d a and d r_i / d b.
                for i in 0..t3.len() {
                    let e = (-x[1] * t3[i]).exp();
                    j[i * 2] = e;
                    j[i * 2 + 1] = -x[0] * t3[i] * e;
                }
                0
            })
            .unwrap();
        let mut coef = vec![1.0_f64, 1.0];
        model.fit(&mut coef).unwrap();
        near(coef[0], 2.0, 1e-5, "a");
        near(coef[1], 0.3, 1e-5, "b");
        drop(model);
        assert!(
            jac_calls.get() > 0,
            "the analytic Jacobian was never called"
        );
    }

    #[test]
    fn nlls_set_bounds_are_respected_and_wide_bounds_do_not_bind() {
        let _lock = nlls_guard();
        let (t, y) = decay_data();
        let make = || {
            let (t, y) = (t.clone(), y.clone());
            Nlls::new(2, 8, move |x: &[f64], r: &mut [f64]| {
                for i in 0..r.len() {
                    r[i] = x[0] * (-x[1] * t[i]).exp() - y[i];
                }
                0
            })
            .unwrap()
        };
        // Wide bounds around the truth (a = 2, b = 0.3): the unconstrained answer is recovered.
        let mut m = make();
        m.set_bounds(Some(&mut [0.0, 0.0]), Some(&mut [10.0, 5.0]))
            .unwrap();
        let mut c = vec![1.0_f64, 1.0];
        m.fit(&mut c).unwrap();
        near(c[0], 2.0, 1e-4, "a (wide bounds)");
        near(c[1], 0.3, 1e-4, "b (wide bounds)");
        // Upper bound a <= 1.5 excludes the true a = 2: the fit must stay feasible, sit on the bound, and leave a
        // strictly positive residual (the data are exactly generated by a = 2).
        let mut m = make();
        m.set_bounds(None, Some(&mut [1.5, 5.0])).unwrap();
        let mut c = vec![1.0_f64, 1.0];
        m.fit(&mut c).unwrap();
        assert!(c[0] <= 1.5 + 1e-9, "a = {} violates a <= 1.5", c[0]);
        near(c[0], 1.5, 1e-4, "a sits on its upper bound");
        let obj: f64 = (0..8)
            .map(|i| (c[0] * (-c[1] * t[i]).exp() - y[i]).powi(2))
            .sum();
        assert!(obj > 1e-4, "constrained objective {obj} should exceed 0");
        // Lower bound b >= 0.5 excludes the true b = 0.3.
        let mut m = make();
        m.set_bounds(Some(&mut [0.0, 0.5]), None).unwrap();
        let mut c = vec![1.0_f64, 1.0];
        m.fit(&mut c).unwrap();
        assert!(c[1] >= 0.5 - 1e-9, "b = {} violates b >= 0.5", c[1]);
        // Too-short bound slices are rejected.
        let mut m = make();
        assert!(m.set_bounds(Some(&mut [0.0]), None).is_err());
    }

    #[test]
    fn nlls_set_weights_tiny_weight_suppresses_an_outlier() {
        let _lock = nlls_guard();
        let (t, mut y) = decay_data();
        y[7] += 5.0; // corrupt the last observation
        let make = || {
            let (t, y) = (t.clone(), y.clone());
            Nlls::new(2, 8, move |x: &[f64], r: &mut [f64]| {
                for i in 0..r.len() {
                    r[i] = x[0] * (-x[1] * t[i]).exp() - y[i];
                }
                0
            })
            .unwrap()
        };
        // Unweighted: the outlier drags the fit away from the truth (control for the weighted case).
        let mut m = make();
        let mut c = vec![1.0_f64, 1.0];
        m.fit(&mut c).unwrap();
        assert!(
            (c[0] - 2.0).abs() > 1e-2 || (c[1] - 0.3).abs() > 1e-2,
            "outlier had no effect: {c:?}"
        );
        // Weights w_i = sqrt(W_ii): AOCL rejects zero ("must be sufficiently positive"), so the outlier gets 1e-4, which
        // shrinks its pull by 10^4 and recovers the true parameters to ~1e-3.
        let mut m = make();
        let mut w = vec![1.0_f64; 8];
        w[7] = 1e-4;
        m.set_weights(&mut w).unwrap();
        let mut c = vec![1.0_f64, 1.0];
        m.fit(&mut c).unwrap();
        near(c[0], 2.0, 3e-3, "a (weighted)");
        near(c[1], 0.3, 3e-3, "b (weighted)");
        assert!(m.set_weights(&mut [1.0; 3]).is_err());
    }

    /// (A fresh handle's `last_error_message` is NOT asserted `None`: on AOCL 6.0.68 the library returns success with an
    /// uninitialised pointer when no error is stored, so the wrapper can return garbage: FINDING.)
    #[test]
    fn last_error_message_names_the_rejected_option() {
        let mut h = Handle::new_double(HandleKind::Linmod).unwrap();
        assert!(h.set_int_option("no such option", 1).is_err());
        let msg = h
            .last_error_message()
            .expect("an error message after a rejected option");
        assert!(!msg.is_empty());
        assert!(
            msg.contains("no such option"),
            "message does not name the option: {msg}"
        );
    }

    // Two separable clusters in 2-D, column-major (feature 0 then feature 1): class 0 near (0,0), class 1 near (10,10).
    const CL_X: [f64; 12] = [
        0.0, 0.1, 0.2, 10.0, 10.1, 10.2, 0.1, 0.0, 0.1, 10.1, 10.0, 10.1,
    ];
    const CL_Y: [i32; 6] = [0, 0, 0, 1, 1, 1];
    const CL_TEST: [f64; 4] = [0.05, 9.95, 0.05, 10.05]; // (0.05,0.05) and (9.95,10.05)

    #[test]
    fn knn_classes_lists_distinct_labels_whatever_the_buffer_size() {
        let mut knn = KNearestNeighbours::new().unwrap();
        knn.fit(6, 2, &CL_X, &[7, 7, 7, 3, 3, 3]).unwrap();
        // Ascending distinct labels; the return value is the number of classes, not the buffer length.
        let mut big = [0_i32; 5];
        assert_eq!(knn.classes(&mut big).unwrap(), 2);
        assert_eq!(&big[..2], &[3, 7]);
        let mut exact = [0_i32; 2];
        assert_eq!(knn.classes(&mut exact).unwrap(), 2);
        assert_eq!(exact, [3, 7]);
    }

    #[test]
    fn knn_kneighbors_match_brute_force_ranking() {
        let mut knn = KNearestNeighbours::new().unwrap();
        knn.fit(6, 2, &CL_X, &CL_Y).unwrap();
        // Queries (0.04, 0.02) and (10.15, 10.0): no distance ties among the training points.
        let q = [0.04, 10.15, 0.02, 10.0];
        let k = 3;
        let mut ind = vec![0_i32; 2 * k];
        let mut dist = vec![0.0_f64; 2 * k];
        knn.kneighbors(2, 2, &q, &mut ind, &mut dist, k, true)
            .unwrap();
        for qi in 0..2 {
            let mut ranked: Vec<(f64, i32)> = (0..6)
                .map(|i| {
                    (
                        ((CL_X[i] - q[qi]).powi(2) + (CL_X[6 + i] - q[2 + qi]).powi(2)).sqrt(),
                        i as i32,
                    )
                })
                .collect();
            ranked.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for j in 0..k {
                // Column-major (n_queries x k): element (query, rank) at rank * n_queries + query.
                assert_eq!(ind[j * 2 + qi], ranked[j].1, "query {qi} rank {j}");
                near(dist[j * 2 + qi], ranked[j].0, 1e-9, "neighbour distance");
            }
        }
        // Without distances the indices are the same.
        let mut ind2 = vec![0_i32; 2 * k];
        let mut dist2 = vec![0.0_f64; 2 * k];
        knn.kneighbors(2, 2, &q, &mut ind2, &mut dist2, k, false)
            .unwrap();
        assert_eq!(ind, ind2);
    }

    #[test]
    fn knn_predict_proba_counts_neighbour_classes() {
        // Default k = 5 with 3 training points per class: a query always has all 3 of its own cluster plus the 2
        // nearest of the other, so P(own class) = 3/5 and P(other) = 2/5. Queries 0 and 1 sit in cluster 0, query 2 in
        // cluster 1. The 3 x 2 result is column-major (ldp = n_queries): element (query, class) at class * 3 + query.
        let mut knn = KNearestNeighbours::new().unwrap();
        knn.fit(6, 2, &CL_X, &CL_Y).unwrap();
        let q = [0.05, 0.1, 9.95, 0.05, 0.0, 10.05]; // 3 queries: (0.05,0.05) (0.1,0.0) (9.95,10.05)
        let mut proba = vec![0.0_f64; 6];
        knn.predict_proba(3, 2, &q, &mut proba).unwrap();
        for (g, w) in proba.iter().zip([0.6, 0.6, 0.4, 0.4, 0.4, 0.6]) {
            near(*g, w, 1e-9, "knn predict_proba");
        }
    }

    #[test]
    fn dbscan_core_sample_indices_exclude_noise() {
        // Five points in a tight cluster plus one far outlier: with min samples = 3 and eps = 1 the cluster points
        // (indices 0..=4) are core samples, the outlier (index 5) is noise (label -1) and not core.
        let data: Vec<f64> = vec![
            0.0, 0.1, 0.2, 0.0, 0.1, 50.0, // feature 0
            0.0, 0.1, 0.0, 0.2, 0.1, 50.0, // feature 1
        ];
        let mut db = Dbscan::new().unwrap();
        let cs = std::ffi::CString::new("eps").unwrap();
        let st = unsafe { sys::da_options_set_real_d(db.handle_mut().as_raw(), cs.as_ptr(), 1.0) };
        assert_eq!(st, sys::da_status__da_status_success);
        db.handle_mut().set_int_option("min samples", 3).unwrap();
        db.fit(6, 2, &data).unwrap();
        let mut core = db.core_sample_indices().unwrap();
        core.sort();
        assert_eq!(core, vec![0, 1, 2, 3, 4]);
        assert_eq!(core.len(), db.n_core_samples().unwrap());
        let labels = db.labels().unwrap();
        assert_eq!(labels[5], -1);
        assert!(labels[..5].iter().all(|&l| l == labels[0] && l >= 0));
    }

    // Four queries (column-major, feature 0 then feature 1): classes 0, 1, 0, 1.
    const Q4: [f64; 8] = [0.05, 9.95, 0.2, 10.2, 0.05, 10.05, 0.0, 10.1];

    /// Probability outputs are laid out in the handle's storage order (column-major, `ldy >= n_samples`): element
    /// (sample `i`, class `c`) is at `c * ldy + i`. (The wrapper's docs say row-major / `ldy >= n_class`: FINDING.)
    #[test]
    fn tree_and_forest_predict_proba_and_log_proba_on_separable_data() {
        let mut tree = DecisionTree::new().unwrap();
        tree.fit(6, 2, 2, &CL_X, &CL_Y).unwrap();
        let mut forest = DecisionForest::new().unwrap();
        forest.fit(6, 2, 2, &CL_X, &CL_Y).unwrap();
        let truth = [0usize, 1, 0, 1];
        let at = |v: &[f64], i: usize, c: usize| v[c * 4 + i];

        let mut tp = vec![0.0_f64; 8];
        tree.predict_proba(4, 2, &Q4, &mut tp, 2, 4).unwrap();
        for i in 0..4 {
            for c in 0..2 {
                near(
                    at(&tp, i, c),
                    if c == truth[i] { 1.0 } else { 0.0 },
                    1e-12,
                    "tree proba (pure leaves)",
                );
            }
        }
        let mut tl = vec![0.0_f64; 8];
        tree.predict_log_proba(4, 2, &Q4, &mut tl, 2, 4).unwrap();
        for i in 0..4 {
            near(
                at(&tl, i, truth[i]),
                0.0,
                1e-12,
                "tree log P(true class) = ln 1",
            );
            assert!(
                at(&tl, i, 1 - truth[i]) < -10.0,
                "ln 0 must be very negative: {tl:?}"
            );
        }

        let mut fp = vec![0.0_f64; 8];
        forest.predict_proba(4, 2, &Q4, &mut fp, 2, 4).unwrap();
        for i in 0..4 {
            near(
                at(&fp, i, 0) + at(&fp, i, 1),
                1.0,
                1e-9,
                "forest probabilities sum to 1",
            );
            assert!(at(&fp, i, truth[i]) > 0.8, "forest proba {fp:?}");
        }
        let mut fl = vec![0.0_f64; 8];
        forest.predict_log_proba(4, 2, &Q4, &mut fl, 2, 4).unwrap();
        // exp(log P) reproduces P wherever P > 0 (the invariant tying the two outputs together).
        for (l, p) in fl.iter().zip(&fp) {
            if *p > 1e-12 {
                near(l.exp(), *p, 1e-9, "forest exp(log_proba) == proba");
            }
        }
    }

    #[test]
    fn linmod_logistic_separates_two_classes() {
        // 1-D, linearly separable at 0: class 0 for x < 0, class 1 for x > 0.
        let x = [-2.0_f64, -1.0, -0.5, 0.5, 1.0, 2.0];
        let y = [0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
        let mut m = Linmod::new_logistic().unwrap();
        assert_eq!(m.kind(), LinmodKind::Logistic);
        m.fit(6, 1, &x, &y).unwrap();
        let coefs = m.coefficients().unwrap();
        assert!(
            coefs[0] > 0.0,
            "slope of a positively separated problem must be > 0: {coefs:?}"
        );
        let mut pred = vec![0.0_f64; 6];
        m.predict(6, 1, &x, &mut pred).unwrap();
        assert_eq!(pred, y);
        // Unseen points on either side of the boundary.
        let mut p2 = [0.0_f64; 2];
        m.predict(2, 1, &[-3.0, 3.0], &mut p2).unwrap();
        assert_eq!(p2, [0.0, 1.0]);
    }

    fn set_real(svm: &mut Svm, name: &str, v: f64) {
        let cs = std::ffi::CString::new(name).unwrap();
        let st = unsafe { sys::da_options_set_real_d(svm.handle_mut().as_raw(), cs.as_ptr(), v) };
        assert_eq!(st, sys::da_status__da_status_success, "option {name}");
    }

    #[test]
    fn svm_nu_svc_separates_two_classes() {
        let y = [0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
        let mut svm = Svm::new_nu_svc().unwrap();
        assert_eq!(svm.kind(), SvmKind::NuSvc);
        svm.fit(6, 2, &CL_X, &y).unwrap();
        let mut p = [0.0_f64; 2];
        svm.predict(2, 2, &CL_TEST, &mut p).unwrap();
        assert_eq!(p, [0.0, 1.0]);
        near(
            svm.score(6, 2, &CL_X, &y).unwrap(),
            1.0,
            1e-12,
            "nu-svc training accuracy",
        );
    }

    #[test]
    fn svm_decision_function_sign_separates_classes() {
        let y = [0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
        let mut svm = Svm::new_svc().unwrap();
        svm.fit(6, 2, &CL_X, &y).unwrap();
        for shape in [DecisionShape::Ovo, DecisionShape::Ovr] {
            let mut dv = vec![0.0_f64; 2];
            svm.decision_function(2, 2, &CL_TEST, shape, &mut dv, 2)
                .unwrap();
            // Binary problem (scikit-learn convention): positive value <=> the second class.
            assert!(dv[0] < 0.0 && dv[1] > 0.0, "{shape:?}: {dv:?}");
        }
        // On the training set every class-1 sample scores above every class-0 sample.
        let mut dv = vec![0.0_f64; 6];
        svm.decision_function(6, 2, &CL_X, DecisionShape::Ovo, &mut dv, 6)
            .unwrap();
        let max0 = dv[..3].iter().cloned().fold(f64::MIN, f64::max);
        let min1 = dv[3..].iter().cloned().fold(f64::MAX, f64::min);
        assert!(max0 < min1, "{dv:?}");
    }

    /// y = 2x + 1 sampled at x = 0..=9: a linear-kernel SVR with a large C must reproduce it within its epsilon tube.
    fn regression_data() -> (Vec<f64>, Vec<f64>) {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| 2.0 * v + 1.0).collect();
        (x, y)
    }

    #[test]
    fn svm_svr_fits_a_line() {
        let (x, y) = regression_data();
        let mut svm = Svm::new_svr().unwrap();
        assert_eq!(svm.kind(), SvmKind::Svr);
        svm.handle_mut()
            .set_string_option("kernel", "linear")
            .unwrap();
        set_real(&mut svm, "C", 1000.0);
        set_real(&mut svm, "epsilon", 0.1);
        svm.fit(10, 1, &x, &y).unwrap();
        let mut p = vec![0.0_f64; 10];
        svm.predict(10, 1, &x, &mut p).unwrap();
        for (g, w) in p.iter().zip(&y) {
            // Epsilon-insensitive loss: training points lie within epsilon (+ solver slack) of the truth.
            assert!((g - w).abs() < 0.2, "svr pred {g} vs {w}");
        }
        assert!(svm.score(10, 1, &x, &y).unwrap() > 0.99);
    }

    #[test]
    fn svm_nu_svr_fits_a_line() {
        let (x, y) = regression_data();
        let mut svm = Svm::new_nu_svr().unwrap();
        assert_eq!(svm.kind(), SvmKind::NuSvr);
        svm.handle_mut()
            .set_string_option("kernel", "linear")
            .unwrap();
        set_real(&mut svm, "C", 1000.0);
        svm.fit(10, 1, &x, &y).unwrap();
        let mut p = vec![0.0_f64; 10];
        svm.predict(10, 1, &x, &mut p).unwrap();
        for (g, w) in p.iter().zip(&y) {
            assert!((g - w).abs() < 0.5, "nu-svr pred {g} vs {w}");
        }
        assert!(svm.score(10, 1, &x, &y).unwrap() > 0.99);
        // Extrapolation follows the line (slope 2): f(12) ~ 25.
        let mut q = [0.0_f64; 1];
        svm.predict(1, 1, &[12.0], &mut q).unwrap();
        assert!((q[0] - 25.0).abs() < 1.0, "f(12) = {}", q[0]);
    }
}
