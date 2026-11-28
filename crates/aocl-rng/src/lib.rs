//! Safe wrappers for AOCL-RNG.

#![allow(
    clippy::too_many_arguments,
    reason = "wrappers mirror the C API's argument lists"
)]
#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use aocl_error::{Error, Result};
use aocl_rng_sys as sys;

/// Integer width used by AOCL-RNG for size parameters and integer-valued
/// distribution outputs. `i32` in the default LP64 build, `i64` in the
/// `ilp64` Cargo-feature build.
pub type RngInt = sys::rng_int_t;

/// `len` must hold a column-major `rows x cols` block with leading dimension `ld`: `ld >= rows` and `(cols - 1) * ld + rows` elements (the
/// library does not check any of the arrays; each too-short buffer here is an access violation, `tests/guard_lengths.rs`).
fn check_block(
    name: &str,
    what: &str,
    rows: usize,
    cols: usize,
    ld: RngInt,
    len: usize,
) -> Result<()> {
    if ld < 0 || (ld as usize) < rows {
        return Err(Error::InvalidArgument(format!(
            "{name}: {what} leading dimension {ld} is less than {rows}"
        )));
    }
    let need = if rows == 0 || cols == 0 {
        0
    } else {
        (cols - 1) * ld as usize + rows
    };
    if len < need {
        return Err(Error::InvalidArgument(format!(
            "{name}: {what} has {len} elements, {need} are needed"
        )));
    }
    Ok(())
}

fn check_len(name: &str, what: &str, len: usize, need: usize) -> Result<()> {
    if len < need {
        return Err(Error::InvalidArgument(format!(
            "{name}: {what} has {len} elements, {need} are needed"
        )));
    }
    Ok(())
}

/// Base generator algorithm. The numeric values follow the NAG-style
/// `genid` convention used by AOCL-RNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BaseGenerator {
    LinearCongruential = 1,
    WichmannHill = 2,
    MersenneTwister = 3,
    WichmannHillParallel = 4,
    Mlfg = 5,
    Mwc = 6,
}

const STATE_LEN: usize = 633;
const SEED_LEN: usize = 1;

/// The reference vector [`Rng::general_discrete`] samples from. It can only be produced by the library's own set-up routines
/// ([`DiscreteReference::binomial`] etc.), which size and fill it, so a safe call can never pass the library a vector of the wrong
/// length or origin (`drandgeneraldiscrete` does not check one: it trusts a length encoded inside it).
///
/// ```compile_fail
/// use aocl_rng::{BaseGenerator, DiscreteReference, Rng};
/// let mut rng = Rng::new(BaseGenerator::MersenneTwister, 1).unwrap();
/// let mut out = [0; 4];
/// // the data field is private: a hand-built vector is not a DiscreteReference
/// let mut fake = DiscreteReference { data: vec![0.0; 3] };
/// rng.general_discrete(&mut fake, &mut out).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct DiscreteReference {
    data: Vec<f64>,
}

impl DiscreteReference {
    /// Query the recommended length (`lref = -1`), allocate it, then let the library fill it.
    fn build(setup: impl Fn(*mut f64, *mut RngInt, *mut RngInt), what: &str) -> Result<Self> {
        let (mut lref, mut info): (RngInt, RngInt) = (-1, 0);
        setup(std::ptr::null_mut(), &mut lref, &mut info);
        // the query answers info = 1 ("length returned", measured on AOCL 6.0.68) with the recommended length in lref
        if !(0..=1).contains(&info) || lref <= 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("{what}: length query returned info={info}, lref={lref}"),
            });
        }
        let full = lref;
        info = 0;
        let mut data = vec![0.0f64; full as usize];
        setup(data.as_mut_ptr(), &mut lref, &mut info);
        if info != 0 || lref != full {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("{what}: set-up returned info={info}, lref={lref} (asked {full})"),
            });
        }
        Ok(Self { data })
    }

    /// Binomial(`m`, `p`) reference.
    pub fn binomial(m: RngInt, p: f64) -> Result<Self> {
        // SAFETY: `build` passes a null pointer only for the length query and a vector of the queried length for the fill.
        Self::build(
            |v, l, i| unsafe { sys::drandbinomialreference(m, p, v, l, i) },
            "binomial reference",
        )
    }

    /// Geometric(`p`) reference.
    pub fn geometric(p: f64) -> Result<Self> {
        // SAFETY: as in `binomial`.
        Self::build(
            |v, l, i| unsafe { sys::drandgeometricreference(p, v, l, i) },
            "geometric reference",
        )
    }

    /// Hypergeometric(`np`, `ns`, `m`) reference.
    pub fn hypergeometric(np: RngInt, ns: RngInt, m: RngInt) -> Result<Self> {
        // SAFETY: as in `binomial`.
        Self::build(
            |v, l, i| unsafe { sys::drandhypergeometricreference(np, ns, m, v, l, i) },
            "hypergeometric reference",
        )
    }

    /// Negative binomial(`m`, `p`) reference.
    pub fn negative_binomial(m: RngInt, p: f64) -> Result<Self> {
        // SAFETY: as in `binomial`.
        Self::build(
            |v, l, i| unsafe { sys::drandnegativebinomialreference(m, p, v, l, i) },
            "negative binomial reference",
        )
    }

    /// Poisson(`lambda`) reference.
    pub fn poisson(lambda: f64) -> Result<Self> {
        // SAFETY: as in `binomial`.
        Self::build(
            |v, l, i| unsafe { sys::drandpoissonreference(lambda, v, l, i) },
            "poisson reference",
        )
    }
}

/// AOCL pseudo-random number generator.
pub struct Rng {
    state: Box<[sys::rng_int_t; STATE_LEN]>,
}

impl Rng {
    /// Initialize a new generator.
    ///
    /// **Caveat:** several AOCL generators (notably the Mersenne Twister)
    /// reject `seed = 0`; use any non-zero seed.
    pub fn new(generator: BaseGenerator, seed: i32) -> Result<Self> {
        Self::with_subid(generator, 1, seed)
    }

    /// Initialize with an explicit `subid`.
    pub fn with_subid(generator: BaseGenerator, subid: i32, seed: i32) -> Result<Self> {
        let mut state = Box::new([0 as sys::rng_int_t; STATE_LEN]);
        let mut seed_buf = [seed as sys::rng_int_t; SEED_LEN];
        let mut lseed: sys::rng_int_t = SEED_LEN as sys::rng_int_t;
        let mut lstate: sys::rng_int_t = STATE_LEN as sys::rng_int_t;
        let mut info: sys::rng_int_t = 0;
        let genid: sys::rng_int_t = generator as sys::rng_int_t;
        let subid_raw: sys::rng_int_t = subid as sys::rng_int_t;

        unsafe {
            sys::drandinitialize(
                genid,
                subid_raw,
                seed_buf.as_mut_ptr(),
                &mut lseed,
                state.as_mut_ptr(),
                &mut lstate,
                &mut info,
            );
        }

        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("drandinitialize returned info={info}"),
            });
        }
        Ok(Rng { state })
    }

    /// Fill `out` with samples from a uniform `[a, b)` distribution.
    pub fn uniform(&mut self, a: f64, b: f64, out: &mut [f64]) -> Result<()> {
        if !(a < b) {
            return Err(Error::InvalidArgument(format!(
                "uniform: require a < b, got a={a} b={b}"
            )));
        }
        let n = out.len();
        if n == 0 {
            return Ok(());
        }
        let n_int: sys::rng_int_t = n.try_into().map_err(|_| {
            Error::InvalidArgument(format!("uniform: n={n} exceeds rng_int_t range"))
        })?;
        let mut info: sys::rng_int_t = 0;
        unsafe {
            sys::dranduniform(
                n_int,
                a,
                b,
                self.state.as_mut_ptr(),
                out.as_mut_ptr(),
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("dranduniform returned info={info}"),
            });
        }
        Ok(())
    }

    /// Fill `out` with samples from a Gaussian / normal `N(mean, variance)`
    /// distribution. Note: AOCL takes the *variance*, not the standard
    /// deviation.
    pub fn gaussian(&mut self, mean: f64, variance: f64, out: &mut [f64]) -> Result<()> {
        if variance < 0.0 {
            return Err(Error::InvalidArgument(format!(
                "gaussian: variance must be non-negative, got {variance}"
            )));
        }
        let n = out.len();
        if n == 0 {
            return Ok(());
        }
        let n_int: sys::rng_int_t = n.try_into().map_err(|_| {
            Error::InvalidArgument(format!("gaussian: n={n} exceeds rng_int_t range"))
        })?;
        let mut info: sys::rng_int_t = 0;
        unsafe {
            sys::drandgaussian(
                n_int,
                mean,
                variance,
                self.state.as_mut_ptr(),
                out.as_mut_ptr(),
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("drandgaussian returned info={info}"),
            });
        }
        Ok(())
    }

    /// Fill `out` with samples from an exponential distribution with
    /// the given mean.
    pub fn exponential(&mut self, mean: f64, out: &mut [f64]) -> Result<()> {
        if mean <= 0.0 {
            return Err(Error::InvalidArgument(format!(
                "exponential: mean must be positive, got {mean}"
            )));
        }
        self.fill_real("exponential", out, |n, st, p, info| unsafe {
            sys::drandexponential(n, mean, st, p, info)
        })
    }

    /// Beta distribution `B(a, b)`. Both shape parameters must be positive.
    pub fn beta(&mut self, a: f64, b: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("beta", out, |n, st, p, info| unsafe {
            sys::drandbeta(n, a, b, st, p, info)
        })
    }

    /// Cauchy distribution with median `med` and scale `scale`.
    pub fn cauchy(&mut self, med: f64, scale: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("cauchy", out, |n, st, p, info| unsafe {
            sys::drandcauchy(n, med, scale, st, p, info)
        })
    }

    /// χ² distribution with `df` degrees of freedom.
    pub fn chi_squared(&mut self, df: i32, out: &mut [f64]) -> Result<()> {
        self.fill_real("chi_squared", out, |n, st, p, info| unsafe {
            sys::drandchisquared(n, df as sys::rng_int_t, st, p, info)
        })
    }

    /// F distribution with `df1`, `df2` degrees of freedom.
    pub fn f_dist(&mut self, df1: i32, df2: i32, out: &mut [f64]) -> Result<()> {
        self.fill_real("f_dist", out, |n, st, p, info| unsafe {
            sys::drandf(n, df1 as sys::rng_int_t, df2 as sys::rng_int_t, st, p, info)
        })
    }

    /// Gamma distribution with shape `a` and scale `b`.
    pub fn gamma(&mut self, a: f64, b: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("gamma", out, |n, st, p, info| unsafe {
            sys::drandgamma(n, a, b, st, p, info)
        })
    }

    /// Logistic distribution with location `a` and scale `b`.
    pub fn logistic(&mut self, a: f64, b: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("logistic", out, |n, st, p, info| unsafe {
            sys::drandlogistic(n, a, b, st, p, info)
        })
    }

    /// Lognormal with underlying-normal mean `mean_ln` and variance `var_ln`.
    pub fn lognormal(&mut self, mean_ln: f64, var_ln: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("lognormal", out, |n, st, p, info| unsafe {
            sys::drandlognormal(n, mean_ln, var_ln, st, p, info)
        })
    }

    /// Student's t distribution with `df` degrees of freedom.
    pub fn students_t(&mut self, df: i32, out: &mut [f64]) -> Result<()> {
        self.fill_real("students_t", out, |n, st, p, info| unsafe {
            sys::drandstudentst(n, df as sys::rng_int_t, st, p, info)
        })
    }

    /// Triangular distribution on `[xmin, xmax]` with mode `xmed`.
    pub fn triangular(&mut self, xmin: f64, xmed: f64, xmax: f64, out: &mut [f64]) -> Result<()> {
        if !(xmin <= xmed && xmed <= xmax) {
            return Err(Error::InvalidArgument(format!(
                "triangular: require xmin <= xmed <= xmax, got {xmin}, {xmed}, {xmax}"
            )));
        }
        self.fill_real("triangular", out, |n, st, p, info| unsafe {
            sys::drandtriangular(n, xmin, xmed, xmax, st, p, info)
        })
    }

    /// Von Mises (circular normal) distribution with concentration `kappa`.
    pub fn von_mises(&mut self, kappa: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("von_mises", out, |n, st, p, info| unsafe {
            sys::drandvonmises(n, kappa, st, p, info)
        })
    }

    /// Weibull distribution with shape `a` and scale `b`.
    pub fn weibull(&mut self, a: f64, b: f64, out: &mut [f64]) -> Result<()> {
        self.fill_real("weibull", out, |n, st, p, info| unsafe {
            sys::drandweibull(n, a, b, st, p, info)
        })
    }

    // ---- Discrete distributions (integer-valued) -----------------------

    /// Binomial(`m`, `p`).
    pub fn binomial(&mut self, m: RngInt, p: f64, out: &mut [RngInt]) -> Result<()> {
        self.fill_int("binomial", out, |n, st, ptr, info| unsafe {
            sys::drandbinomial(n, m, p, st, ptr, info)
        })
    }

    /// Geometric distribution with success probability `p`.
    pub fn geometric(&mut self, p: f64, out: &mut [RngInt]) -> Result<()> {
        self.fill_int("geometric", out, |n, st, ptr, info| unsafe {
            sys::drandgeometric(n, p, st, ptr, info)
        })
    }

    /// Negative-binomial as AOCL defines it (header `rng.h`, measured on AOCL 6.0.68): the number of **successes** before the
    /// `m`-th **failure**, each trial succeeding with probability `p` (mean `m·p/(1−p)`; `m = 4`, `p = 0.4` gives about 2.67).
    /// This is the reverse of the usual "failures before the m-th success" convention.
    pub fn negative_binomial(&mut self, m: RngInt, p: f64, out: &mut [RngInt]) -> Result<()> {
        self.fill_int("negative_binomial", out, |n, st, ptr, info| unsafe {
            sys::drandnegativebinomial(n, m, p, st, ptr, info)
        })
    }

    /// Poisson with mean `lambda`.
    pub fn poisson(&mut self, lambda: f64, out: &mut [RngInt]) -> Result<()> {
        self.fill_int("poisson", out, |n, st, ptr, info| unsafe {
            sys::drandpoisson(n, lambda, st, ptr, info)
        })
    }

    /// Discrete uniform on the inclusive integer range `[a, b]`.
    pub fn discrete_uniform(&mut self, a: RngInt, b: RngInt, out: &mut [RngInt]) -> Result<()> {
        if a > b {
            return Err(Error::InvalidArgument(format!(
                "discrete_uniform: require a <= b, got a={a} b={b}"
            )));
        }
        self.fill_int("discrete_uniform", out, |n, st, ptr, info| unsafe {
            sys::dranddiscreteuniform(n, a, b, st, ptr, info)
        })
    }

    /// Hypergeometric distribution: number of successes in `m` draws
    /// without replacement from a population of `np` items containing
    /// `ns` successes.
    pub fn hypergeometric(
        &mut self,
        np: RngInt,
        ns: RngInt,
        m: RngInt,
        out: &mut [RngInt],
    ) -> Result<()> {
        self.fill_int("hypergeometric", out, |n, st, ptr, info| unsafe {
            sys::drandhypergeometric(n, np, ns, m, st, ptr, info)
        })
    }

    /// General discrete distribution from a [`DiscreteReference`] (built only by the library's own `*reference` set-up routines, so
    /// no caller can hand the library a buffer it does not understand). `out` receives integer indices.
    pub fn general_discrete(
        &mut self,
        reference: &mut DiscreteReference,
        out: &mut [RngInt],
    ) -> Result<()> {
        // SAFETY: `reference` holds exactly the vector a `drand*reference` routine wrote (see `DiscreteReference::build`).
        unsafe { self.general_discrete_raw(&mut reference.data, out) }
    }

    /// General discrete distribution from a raw reference slice (the entry point before [`DiscreteReference`], kept for callers that
    /// hold a reference vector from the C API).
    ///
    /// # Safety
    /// `ref_dist` must be the unmodified vector that one of AOCL's `drand{binomial,geometric,hypergeometric,negativebinomial,poisson}
    /// reference` routines (header `rng.h`) wrote, as `f64`, with its full recommended length: the library trusts a length encoded
    /// inside that vector, not the slice length, so any other array can be read out of bounds.
    pub unsafe fn general_discrete_raw(
        &mut self,
        ref_dist: &mut [f64],
        out: &mut [RngInt],
    ) -> Result<()> {
        let n = out.len();
        if n == 0 {
            return Ok(());
        }
        let n_int: RngInt = n.try_into().map_err(|_| {
            Error::InvalidArgument(format!("general_discrete: n={n} exceeds rng_int_t range"))
        })?;
        let mut info: RngInt = 0;
        // SAFETY: the caller's contract (the `# Safety` section).
        unsafe {
            sys::drandgeneraldiscrete(
                n_int,
                ref_dist.as_mut_ptr(),
                self.state.as_mut_ptr(),
                out.as_mut_ptr(),
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("general_discrete returned info={info}"),
            });
        }
        Ok(())
    }

    /// Multinomial distribution: each of `n` samples is the count
    /// vector (length `k`) from `m` trials with category probabilities
    /// `p` (length `k`). `out` is laid out as `n × k` with leading
    /// dimension `ldx`.
    pub fn multinomial(
        &mut self,
        m: RngInt,
        p: &mut [f64],
        k: RngInt,
        out: &mut [RngInt],
        ldx: RngInt,
    ) -> Result<()> {
        let n_samples = if k > 0 { out.len() / k as usize } else { 0 };
        if n_samples == 0 {
            return Ok(());
        }
        let n_int: RngInt = n_samples.try_into().map_err(|_| {
            Error::InvalidArgument(format!(
                "multinomial: n={n_samples} exceeds rng_int_t range"
            ))
        })?;
        check_len("multinomial", "p", p.len(), k as usize)?;
        check_block("multinomial", "out", n_samples, k as usize, ldx, out.len())?;
        let mut info: RngInt = 0;
        unsafe {
            sys::drandmultinomial(
                n_int,
                m,
                p.as_mut_ptr(),
                k,
                self.state.as_mut_ptr(),
                out.as_mut_ptr(),
                ldx,
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("multinomial returned info={info}"),
            });
        }
        Ok(())
    }

    /// Multivariate normal: `n` samples each drawn from `N(xmu, C)`
    /// where `C` is an `m × m` covariance matrix with leading
    /// dimension `ldc`. `out` is laid out as `n × m` with leading
    /// dimension `ldx`.
    pub fn multinormal(
        &mut self,
        m: RngInt,
        xmu: &mut [f64],
        c: &mut [f64],
        ldc: RngInt,
        out: &mut [f64],
        ldx: RngInt,
    ) -> Result<()> {
        let n_samples = if m > 0 { out.len() / m as usize } else { 0 };
        if n_samples == 0 {
            return Ok(());
        }
        let n_int: RngInt = n_samples.try_into().map_err(|_| {
            Error::InvalidArgument(format!(
                "multinormal: n={n_samples} exceeds rng_int_t range"
            ))
        })?;
        check_len("multinormal", "xmu", xmu.len(), m as usize)?;
        check_block("multinormal", "c", m as usize, m as usize, ldc, c.len())?;
        check_block("multinormal", "out", n_samples, m as usize, ldx, out.len())?;
        let mut info: RngInt = 0;
        unsafe {
            sys::drandmultinormal(
                n_int,
                m,
                xmu.as_mut_ptr(),
                c.as_mut_ptr(),
                ldc,
                self.state.as_mut_ptr(),
                out.as_mut_ptr(),
                ldx,
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("multinormal returned info={info}"),
            });
        }
        Ok(())
    }

    /// Skip-ahead: advance the state as if `n_skip` samples had been
    /// drawn. Useful for splitting a single seed into independent
    /// substreams.
    pub fn skip_ahead(&mut self, n_skip: u64) -> Result<()> {
        let n_int: RngInt = n_skip.try_into().map_err(|_| {
            Error::InvalidArgument(format!("skip_ahead: n={n_skip} exceeds rng_int_t range"))
        })?;
        let mut info: RngInt = 0;
        unsafe {
            sys::drandskipahead(n_int, self.state.as_mut_ptr(), &mut info);
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("skip_ahead returned info={info}"),
            });
        }
        Ok(())
    }

    /// Leapfrog: split this stream into `n` substreams and advance
    /// this state to substream `k`, with `0 < k <= n` (`k = 0` is rejected). Each `leapfrog(n, _)` call subselects every
    /// `n`-th sample. Measured on AOCL 6.0.68 (LinearCongruential, WichmannHill, n = 3): stream `k` starts at base index
    /// `k + 1`, not at `k − 1` as the header says; only the stride, disjointness and coverage are guaranteed here.
    /// Not supported by every generator (Mersenne Twister returns an error).
    pub fn leapfrog(&mut self, n: RngInt, k: RngInt) -> Result<()> {
        let mut info: RngInt = 0;
        unsafe {
            sys::drandleapfrog(n, k, self.state.as_mut_ptr(), &mut info);
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("leapfrog returned info={info}"),
            });
        }
        Ok(())
    }

    /// Multivariate Student's-t distribution: `n` samples each drawn
    /// with `df` degrees of freedom, mean `xmu` (length `m`), and
    /// scale matrix `c` (`m × m`, leading dim `ldc`). `out` is laid
    /// out as `n × m` with leading dim `ldx`.
    pub fn multi_students_t(
        &mut self,
        m: RngInt,
        df: RngInt,
        xmu: &mut [f64],
        c: &mut [f64],
        ldc: RngInt,
        out: &mut [f64],
        ldx: RngInt,
    ) -> Result<()> {
        let n_samples = if m > 0 { out.len() / m as usize } else { 0 };
        if n_samples == 0 {
            return Ok(());
        }
        let n_int: RngInt = n_samples.try_into().map_err(|_| {
            Error::InvalidArgument(format!(
                "multi_students_t: n={n_samples} exceeds rng_int_t range"
            ))
        })?;
        check_len("multi_students_t", "xmu", xmu.len(), m as usize)?;
        check_block(
            "multi_students_t",
            "c",
            m as usize,
            m as usize,
            ldc,
            c.len(),
        )?;
        check_block(
            "multi_students_t",
            "out",
            n_samples,
            m as usize,
            ldx,
            out.len(),
        )?;
        let mut info: RngInt = 0;
        unsafe {
            sys::drandmultistudentst(
                n_int,
                m,
                df,
                xmu.as_mut_ptr(),
                c.as_mut_ptr(),
                ldc,
                self.state.as_mut_ptr(),
                out.as_mut_ptr(),
                ldx,
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("multi_students_t returned info={info}"),
            });
        }
        Ok(())
    }
}

// =========================================================================
//   Blum-Blum-Shub cryptographic RNG (separate state from Rng)
// =========================================================================

/// Blum-Blum-Shub generator state. Use [`BlumBlumShub::new`] to
/// initialise from prime factors `p` and `q` (each must be ≡ 3 mod 4)
/// and seed `s`, then call [`BlumBlumShub::sample`] to fill a buffer.
pub struct BlumBlumShub {
    state: Box<[sys::rng_int_t; STATE_LEN]>,
}

impl BlumBlumShub {
    /// Initialise BBS from prime factors `p` and `q` (must each be
    /// `≡ 3 (mod 4)`) and a seed `s` coprime to `p · q`. `nbits`
    /// controls how many bits per call to extract (1..=15; clamped).
    ///
    /// `p` and `q` must be primes congruent to 3 mod 4. **This call may not return for large primes; no bound is known** (measured on
    /// AOCL 6.0.68 Windows: see docs/RNG-BBS-HANG-20261127.md).
    ///
    /// **Detail (measured on AOCL 6.0.68 Windows, docs/RNG-BBS-HANG-20261127.md):** for some prime pairs this call never returns
    /// (100 % CPU). Every pair with both primes <= 99 991 returned at once; above about 200 000 some pairs hang and some work
    /// (not monotonic in size; e.g. `225307/225299` hangs, `541271/541267` and `812527/812519` work; one hanging prime is
    /// enough). The seed and `nbits` are not involved. The library documents no bound and the cause is not known, so the
    /// wrapper cannot validate it; keep `p` and `q` below 100 000 unless you have tested the pair. AMD deprecates this
    /// generator in favour of SecureRNG.
    pub fn new(
        nbits: RngInt,
        p: &mut [RngInt],
        q: &mut [RngInt],
        s: &mut [RngInt],
    ) -> Result<Self> {
        let mut state = Box::new([0 as sys::rng_int_t; STATE_LEN]);
        let mut lstate: RngInt = STATE_LEN as RngInt;
        let mut info: RngInt = 0;
        unsafe {
            sys::drandinitializebbs(
                nbits,
                p.len() as RngInt,
                p.as_mut_ptr(),
                q.len() as RngInt,
                q.as_mut_ptr(),
                s.len() as RngInt,
                s.as_mut_ptr(),
                state.as_mut_ptr(),
                &mut lstate,
                &mut info,
            );
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("drandinitializebbs returned info={info}"),
            });
        }
        Ok(BlumBlumShub { state })
    }

    /// Fill `out` with BBS-generated `f64` values. Per the header the least-significant 30 bits of each double carry the
    /// output: the values are integer-valued in `[0, 2^30)` (measured), **not** in `[0, 1)`.
    pub fn sample(&mut self, out: &mut [f64]) -> Result<()> {
        let n = out.len();
        if n == 0 {
            return Ok(());
        }
        let n_int: RngInt = n.try_into().map_err(|_| {
            Error::InvalidArgument(format!("BBS sample: n={n} exceeds rng_int_t range"))
        })?;
        let mut info: RngInt = 0;
        unsafe {
            sys::drandblumblumshub(n_int, self.state.as_mut_ptr(), out.as_mut_ptr(), &mut info);
        }
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("drandblumblumshub returned info={info}"),
            });
        }
        Ok(())
    }
}

impl std::fmt::Debug for BlumBlumShub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlumBlumShub").finish_non_exhaustive()
    }
}

impl Rng {
    // ---- internal helpers ----------------------------------------------

    fn fill_real<F>(&mut self, op: &'static str, out: &mut [f64], call: F) -> Result<()>
    where
        F: FnOnce(sys::rng_int_t, *mut sys::rng_int_t, *mut f64, *mut sys::rng_int_t),
    {
        let n = out.len();
        if n == 0 {
            return Ok(());
        }
        let n_int: sys::rng_int_t = n
            .try_into()
            .map_err(|_| Error::InvalidArgument(format!("{op}: n={n} exceeds rng_int_t range")))?;
        let mut info: sys::rng_int_t = 0;
        call(n_int, self.state.as_mut_ptr(), out.as_mut_ptr(), &mut info);
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("{op} returned info={info}"),
            });
        }
        Ok(())
    }

    fn fill_int<F>(&mut self, op: &'static str, out: &mut [RngInt], call: F) -> Result<()>
    where
        F: FnOnce(sys::rng_int_t, *mut sys::rng_int_t, *mut sys::rng_int_t, *mut sys::rng_int_t),
    {
        let n = out.len();
        if n == 0 {
            return Ok(());
        }
        let n_int: sys::rng_int_t = n
            .try_into()
            .map_err(|_| Error::InvalidArgument(format!("{op}: n={n} exceeds rng_int_t range")))?;
        let mut info: sys::rng_int_t = 0;
        call(n_int, self.state.as_mut_ptr(), out.as_mut_ptr(), &mut info);
        if info != 0 {
            return Err(Error::Status {
                component: "rng",
                code: info as i64,
                message: format!("{op} returned info={info}"),
            });
        }
        Ok(())
    }
}

impl std::fmt::Debug for Rng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rng").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean(xs: &[f64]) -> f64 {
        xs.iter().sum::<f64>() / xs.len() as f64
    }

    fn variance(xs: &[f64], mu: f64) -> f64 {
        xs.iter().map(|&x| (x - mu).powi(2)).sum::<f64>() / xs.len() as f64
    }

    #[test]
    fn uniform_in_range() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 12345).unwrap();
        let mut out = vec![0.0_f64; 10_000];
        rng.uniform(0.0, 1.0, &mut out).unwrap();
        assert!(out.iter().all(|&x| (0.0..=1.0).contains(&x)));
        let mu = mean(&out);
        assert!((mu - 0.5).abs() < 0.05, "mean={mu}");
    }

    #[test]
    fn gaussian_stats_close() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 67890).unwrap();
        let mut out = vec![0.0_f64; 50_000];
        rng.gaussian(2.0, 4.0, &mut out).unwrap();
        let mu = mean(&out);
        let var = variance(&out, mu);
        assert!((mu - 2.0).abs() < 0.1, "mean={mu}");
        assert!((var - 4.0).abs() < 0.2, "variance={var}");
    }

    #[test]
    fn reproducible_with_same_seed() {
        let mut a = Rng::new(BaseGenerator::MersenneTwister, 1).unwrap();
        let mut b = Rng::new(BaseGenerator::MersenneTwister, 1).unwrap();
        let mut va = vec![0.0_f64; 100];
        let mut vb = vec![0.0_f64; 100];
        a.uniform(0.0, 1.0, &mut va).unwrap();
        b.uniform(0.0, 1.0, &mut vb).unwrap();
        assert_eq!(va, vb);
    }

    #[test]
    fn beta_in_unit_interval() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 13).unwrap();
        let mut out = vec![0.0_f64; 5_000];
        rng.beta(2.0, 5.0, &mut out).unwrap();
        assert!(out.iter().all(|&x| (0.0..=1.0).contains(&x)));
        // Beta(2,5) mean = 2/(2+5) ≈ 0.2857
        let mu = mean(&out);
        assert!((mu - 2.0 / 7.0).abs() < 0.05, "mean={mu}");
    }

    #[test]
    fn gamma_positive_and_mean() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 17).unwrap();
        let mut out = vec![0.0_f64; 5_000];
        rng.gamma(3.0, 2.0, &mut out).unwrap();
        // Gamma(shape=3, scale=2) is positive, mean = 6.
        assert!(out.iter().all(|&x| x > 0.0));
        let mu = mean(&out);
        assert!((mu - 6.0).abs() < 0.5, "mean={mu}");
    }

    #[test]
    fn weibull_positive() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 23).unwrap();
        let mut out = vec![0.0_f64; 1_000];
        rng.weibull(1.5, 1.0, &mut out).unwrap();
        assert!(out.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn poisson_non_negative_with_mean() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 29).unwrap();
        let mut out = vec![0 as RngInt; 5_000];
        rng.poisson(4.5, &mut out).unwrap();
        assert!(out.iter().all(|&x| x >= 0));
        let mu = out.iter().map(|&x| x as f64).sum::<f64>() / out.len() as f64;
        assert!((mu - 4.5).abs() < 0.2, "mean={mu}");
    }

    #[test]
    fn discrete_uniform_in_range() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 31).unwrap();
        let mut out = vec![0 as RngInt; 1_000];
        rng.discrete_uniform(10, 20, &mut out).unwrap();
        assert!(out.iter().all(|&x| (10..=20).contains(&x)));
    }

    #[test]
    fn binomial_in_range() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 41).unwrap();
        let mut out = vec![0 as RngInt; 2_000];
        rng.binomial(10, 0.3, &mut out).unwrap();
        assert!(out.iter().all(|&x| (0..=10).contains(&x)));
        // Mean = 10·0.3 = 3
        let mu = out.iter().map(|&x| x as f64).sum::<f64>() / out.len() as f64;
        assert!((mu - 3.0).abs() < 0.2, "mean={mu}");
    }

    #[test]
    fn triangular_in_range() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 47).unwrap();
        let mut out = vec![0.0_f64; 2_000];
        rng.triangular(-1.0, 0.0, 1.0, &mut out).unwrap();
        assert!(out.iter().all(|&x| (-1.0..=1.0).contains(&x)));
    }

    #[test]
    fn invalid_uniform_range_is_error() {
        let mut rng = Rng::new(BaseGenerator::MersenneTwister, 42).unwrap();
        let mut out = [0.0_f64; 4];
        let err = rng.uniform(1.0, 0.5, &mut out).unwrap_err();
        matches!(err, Error::InvalidArgument(_));
    }
}

/// Statistical and stream-consistency tests for the distribution wrappers. Every expected value is a closed form
/// (moments, CDFs, support) or the base stream itself; every random test uses a fixed seed and a 5-sigma bound.
#[cfg(test)]
mod distribution_tests {
    use super::*;

    const N: usize = 20_000;
    const MT: BaseGenerator = BaseGenerator::MersenneTwister;

    fn rng(seed: i32) -> Rng {
        Rng::new(MT, seed).unwrap()
    }
    fn mean(xs: &[f64]) -> f64 {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
    fn ints(xs: &[RngInt]) -> Vec<f64> {
        xs.iter().map(|&x| x as f64).collect()
    }
    fn uniform_stream(r: &mut Rng, n: usize) -> Vec<f64> {
        let mut v = vec![0.0; n];
        r.uniform(0.0, 1.0, &mut v).unwrap();
        v
    }

    /// Sample mean within 5 standard errors of `mu` (true variance `var`), and sample variance within 5 standard errors
    /// of `var` (the s^2 standard error is var * sqrt((excess_kurtosis + 2) / n)).
    fn check_moments(what: &str, xs: &[f64], mu: f64, var: f64, excess_kurtosis: f64) {
        let n = xs.len() as f64;
        let m = mean(xs);
        assert!(
            (m - mu).abs() < 5.0 * (var / n).sqrt(),
            "{what}: mean {m} vs {mu}"
        );
        let s2 = xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
        let se = var * ((excess_kurtosis + 2.0) / n).sqrt();
        assert!(
            (s2 - var).abs() < 5.0 * se,
            "{what}: variance {s2} vs {var}"
        );
    }

    /// Empirical CDF at each point `x` within 5 binomial standard errors of `cdf(x)`.
    fn check_cdf(what: &str, xs: &[f64], points: &[f64], cdf: impl Fn(f64) -> f64) {
        let n = xs.len() as f64;
        for &x in points {
            let f = cdf(x);
            let emp = xs.iter().filter(|&&v| v <= x).count() as f64 / n;
            assert!(
                (emp - f).abs() < 5.0 * (f * (1.0 - f) / n).sqrt(),
                "{what}: F({x}) emp {emp} vs {f}"
            );
        }
    }

    fn fill(f: impl FnOnce(&mut Rng, &mut [f64]) -> Result<()>, seed: i32) -> Vec<f64> {
        let mut r = rng(seed);
        let mut v = vec![0.0; N];
        f(&mut r, &mut v).unwrap();
        v
    }
    fn fill_int(f: impl FnOnce(&mut Rng, &mut [RngInt]) -> Result<()>, seed: i32) -> Vec<RngInt> {
        let mut r = rng(seed);
        let mut v = vec![0 as RngInt; N];
        f(&mut r, &mut v).unwrap();
        v
    }

    #[test]
    fn exponential_cdf_and_moments() {
        let xs = fill(|r, o| r.exponential(2.5, o), 101);
        assert!(xs.iter().all(|&x| x >= 0.0));
        check_moments("exponential", &xs, 2.5, 6.25, 6.0);
        check_cdf("exponential", &xs, &[0.5, 1.0, 2.5, 5.0, 8.0], |x| {
            1.0 - (-x / 2.5).exp()
        });
        let mut r = rng(1);
        assert!(matches!(
            r.exponential(0.0, &mut [0.0; 4]),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn cauchy_cdf() {
        let xs = fill(|r, o| r.cauchy(1.0, 2.0, o), 102);
        check_cdf("cauchy", &xs, &[-8.0, -2.0, 0.0, 1.0, 3.0, 10.0], |x| {
            0.5 + ((x - 1.0) / 2.0).atan() / std::f64::consts::PI
        });
    }

    #[test]
    fn logistic_cdf_and_moments() {
        let (a, b) = (1.0_f64, 0.5_f64);
        let xs = fill(|r, o| r.logistic(a, b, o), 103);
        check_moments(
            "logistic",
            &xs,
            a,
            std::f64::consts::PI.powi(2) * b * b / 3.0,
            1.2,
        );
        check_cdf("logistic", &xs, &[-0.5, 0.5, 1.0, 1.5, 2.5], |x| {
            1.0 / (1.0 + (-(x - a) / b).exp())
        });
    }

    #[test]
    fn chi_squared_moments() {
        let df = 6;
        let xs = fill(|r, o| r.chi_squared(df, o), 104);
        assert!(xs.iter().all(|&x| x >= 0.0));
        check_moments("chi_squared", &xs, 6.0, 12.0, 12.0 / 6.0);
    }

    #[test]
    fn f_dist_mean_and_support() {
        let (d1, d2) = (8, 30);
        let xs = fill(|r, o| r.f_dist(d1, d2, o), 105);
        assert!(xs.iter().all(|&x| x > 0.0));
        // mean d2/(d2-2); var 2 d2^2 (d1+d2-2) / (d1 (d2-2)^2 (d2-4))
        let (a, b) = (d1 as f64, d2 as f64);
        let var = 2.0 * b * b * (a + b - 2.0) / (a * (b - 2.0).powi(2) * (b - 4.0));
        let m = mean(&xs);
        assert!(
            (m - b / (b - 2.0)).abs() < 5.0 * (var / N as f64).sqrt(),
            "mean {m}"
        );
    }

    #[test]
    fn lognormal_moments_and_median() {
        let (mu, s2) = (0.3_f64, 0.25_f64);
        let xs = fill(|r, o| r.lognormal(mu, s2, o), 106);
        assert!(xs.iter().all(|&x| x > 0.0));
        let mean_t = (mu + s2 / 2.0).exp();
        let var_t = (s2.exp() - 1.0) * (2.0 * mu + s2).exp();
        let exk = (4.0 * s2).exp() + 2.0 * (3.0 * s2).exp() + 3.0 * (2.0 * s2).exp() - 6.0;
        check_moments("lognormal", &xs, mean_t, var_t, exk);
        // median exp(mu): F = 0.5
        check_cdf("lognormal", &xs, &[mu.exp()], |_| 0.5);
    }

    #[test]
    fn students_t_moments_and_symmetry() {
        let df = 10;
        let xs = fill(|r, o| r.students_t(df, o), 107);
        check_moments("students_t", &xs, 0.0, 10.0 / 8.0, 6.0 / 6.0);
        check_cdf("students_t", &xs, &[0.0], |_| 0.5);
    }

    /// Modified Bessel I_nu(k) for nu = 0, 1, 2 by power series.
    fn bessel_i(nu: i32, k: f64) -> f64 {
        let mut term = (k / 2.0).powi(nu) / (1..=nu).map(|i| i as f64).product::<f64>().max(1.0);
        let mut sum = term;
        for j in 1..60 {
            term *= (k / 2.0).powi(2) / (j as f64 * (j + nu) as f64);
            sum += term;
        }
        sum
    }

    #[test]
    fn von_mises_support_and_circular_moment() {
        let kappa = 2.0;
        let xs = fill(|r, o| r.von_mises(kappa, o), 108);
        let pi = std::f64::consts::PI;
        assert!(xs.iter().all(|&x| (-pi..=pi).contains(&x)));
        // E cos(x) = I1/I0, E cos^2(x) = (1 + I2/I0) / 2
        let (i0, i1, i2) = (bessel_i(0, kappa), bessel_i(1, kappa), bessel_i(2, kappa));
        let ec = i1 / i0;
        let var = (1.0 + i2 / i0) / 2.0 - ec * ec;
        let cs: Vec<f64> = xs.iter().map(|x| x.cos()).collect();
        assert!(
            (mean(&cs) - ec).abs() < 5.0 * (var / N as f64).sqrt(),
            "E cos = {} vs {ec}",
            mean(&cs)
        );
        // symmetric about 0
        check_cdf("von_mises", &xs, &[0.0], |_| 0.5);
        let mut r = rng(1);
        assert!(r.von_mises(-1.0, &mut [0.0; 4]).is_err());
    }

    #[test]
    fn geometric_moments_and_support() {
        // failures before the first success: P(X=k) = (1-p)^k p, mean (1-p)/p, var (1-p)/p^2
        let p = 0.3;
        let xs = ints(&fill_int(|r, o| r.geometric(p, o), 109));
        assert!(xs.iter().all(|&x| x >= 0.0));
        check_moments("geometric", &xs, 0.7 / 0.3, 0.7 / 0.09, 6.0 + p * p / 0.7);
        let mut r = rng(1);
        assert!(r.geometric(0.0, &mut [0 as RngInt; 4]).is_err());
    }

    #[test]
    fn negative_binomial_moments() {
        // AOCL header (drandnegativebinomial): m = number of failures that stops the count, p = probability of
        // success; X counts the successes before the m-th failure: mean m p/(1-p), var m p/(1-p)^2. (The wrapper's
        // doc comment states the roles the other way round: see the FINDING in the lane report.)
        let (m, p) = (4 as RngInt, 0.4);
        let xs = ints(&fill_int(|r, o| r.negative_binomial(m, p, o), 110));
        assert!(xs.iter().all(|&x| x >= 0.0));
        let (q, r) = (1.0 - p, 4.0); // q = per-trial stopping probability
        let exk = 6.0 / r + q * q / (r * (1.0 - q));
        check_moments("negative_binomial", &xs, r * p / q, r * p / (q * q), exk);
    }

    #[test]
    fn hypergeometric_moments_and_support() {
        // population 50, 20 specified, 10 drawn: mean 10*20/50, var n p (1-p) (N-n)/(N-1)
        let (np, ns, m) = (50 as RngInt, 20 as RngInt, 10 as RngInt);
        let v = fill_int(|r, o| r.hypergeometric(np, ns, m, o), 111);
        assert!(v.iter().all(|&x| (0..=10).contains(&x)));
        let var = 10.0 * 0.4 * 0.6 * 40.0 / 49.0;
        let xs = ints(&v);
        let mu = mean(&xs);
        assert!(
            (mu - 4.0).abs() < 5.0 * (var / N as f64).sqrt(),
            "mean {mu}"
        );
        let s2 = xs.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / N as f64;
        assert!((s2 - var).abs() < 0.15, "variance {s2} vs {var}");
    }

    /// Reference vector for `general_discrete`, built by the library's own `*reference` set-up routine: `setup`
    /// is called first with lref = -1 (query the recommended length), then with the allocated vector.
    fn reference(setup: impl Fn(*mut f64, *mut RngInt, *mut RngInt)) -> Vec<f64> {
        let (mut lref, mut info): (RngInt, RngInt) = (-1, 0);
        setup(std::ptr::null_mut(), &mut lref, &mut info);
        assert!(lref > 0, "recommended lref {lref}, info {info}");
        let mut r = vec![0.0; lref as usize];
        setup(r.as_mut_ptr(), &mut lref, &mut info);
        assert_eq!(info, 0);
        r
    }

    #[test]
    fn general_discrete_with_binomial_and_geometric_references() {
        // The AOCL header requires `ref` to come from a drand*reference routine (the wrapper's "cumulative
        // probability vector" doc is rejected with info=-2: see the FINDING in the lane report).
        let (m, p) = (12 as RngInt, 0.35);
        let mut r = DiscreteReference::binomial(m, p).unwrap();
        let v = fill_int(|g, o| g.general_discrete(&mut r, o), 112);
        assert!(v.iter().all(|&x| (0..=12).contains(&x)));
        let var = 12.0 * 0.35 * 0.65;
        check_moments(
            "general_discrete(binomial)",
            &ints(&v),
            12.0 * 0.35,
            var,
            (1.0 - 6.0 * 0.35 * 0.65) / var,
        );

        let pg = 0.4;
        let mut r = DiscreteReference::geometric(pg).unwrap();
        let v = fill_int(|g, o| g.general_discrete(&mut r, o), 113);
        assert!(v.iter().all(|&x| x >= 0));
        // geometric (failures before the first success), same convention as Rng::geometric
        check_moments(
            "general_discrete(geometric)",
            &ints(&v),
            0.6 / 0.4,
            0.6 / 0.16,
            6.0 + pg * pg / 0.6,
        );
    }

    #[test]
    fn discrete_reference_constructors_validate_and_all_five_sample() {
        // the library's own info code is surfaced for an invalid parameter instead of a garbage vector
        assert!(DiscreteReference::binomial(10, 1.5).is_err());
        assert!(DiscreteReference::geometric(0.0).is_err());
        assert!(DiscreteReference::poisson(-1.0).is_err());
        for mut r in [
            DiscreteReference::hypergeometric(20, 8, 5).unwrap(),
            DiscreteReference::negative_binomial(3, 0.4).unwrap(),
            DiscreteReference::poisson(2.5).unwrap(),
        ] {
            let v = fill_int(|g, o| g.general_discrete(&mut r, o), 5);
            assert!(v.iter().all(|&x| x >= 0));
        }
    }

    #[test]
    fn general_discrete_raw_matches_the_typed_path() {
        let mut typed = DiscreteReference::binomial(12, 0.35).unwrap();
        let mut raw =
            reference(|v, l, i| unsafe { sys::drandbinomialreference(12, 0.35, v, l, i) });
        let a = fill_int(|g, o| g.general_discrete(&mut typed, o), 7);
        let b = fill_int(|g, o| unsafe { g.general_discrete_raw(&mut raw, o) }, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn multinomial_rows_sum_to_trials_and_match_probabilities() {
        let (m, k) = (10 as RngInt, 3 as RngInt);
        let mut p = vec![0.2, 0.3, 0.5];
        let mut out = vec![0 as RngInt; N * 3];
        rng(113)
            .multinomial(m, &mut p, k, &mut out, N as RngInt)
            .unwrap();
        // x(ldx, k) column-major: column j holds category j
        for i in 0..N {
            let s: RngInt = (0..3).map(|j| out[i + j * N]).sum();
            assert_eq!(s, 10, "row {i}");
        }
        for (j, &pj) in [0.2, 0.3, 0.5].iter().enumerate() {
            let col = ints(&out[j * N..(j + 1) * N]);
            let var = 10.0 * pj * (1.0 - pj);
            assert!(
                (mean(&col) - 10.0 * pj).abs() < 5.0 * (var / N as f64).sqrt(),
                "category {j}"
            );
        }
        // fewer than 2 outcomes is rejected
        let mut one = vec![1.0];
        assert!(rng(1)
            .multinomial(5, &mut one, 1, &mut out, N as RngInt)
            .is_err());
    }

    /// Sample mean of column `j` and sample covariance of columns `a`, `b` of a column-major `n x m` block.
    fn col_stats(out: &[f64], n: usize, a: usize, b: usize) -> (f64, f64, f64) {
        let ca = &out[a * n..(a + 1) * n];
        let cb = &out[b * n..(b + 1) * n];
        let (ma, mb) = (mean(ca), mean(cb));
        let cov = ca
            .iter()
            .zip(cb)
            .map(|(x, y)| (x - ma) * (y - mb))
            .sum::<f64>()
            / n as f64;
        (ma, mb, cov)
    }

    #[test]
    fn multinormal_mean_and_covariance() {
        // C = [[4, 1.2], [1.2, 1]] column-major, mean (1, -2)
        let mut c = vec![4.0, 1.2, 1.2, 1.0];
        let mut mu = vec![1.0, -2.0];
        let mut out = vec![0.0; N * 2];
        rng(114)
            .multinormal(2, &mut mu, &mut c, 2, &mut out, N as RngInt)
            .unwrap();
        let (m0, m1, c01) = col_stats(&out, N, 0, 1);
        let (_, _, c00) = col_stats(&out, N, 0, 0);
        let (_, _, c11) = col_stats(&out, N, 1, 1);
        let n = N as f64;
        assert!((m0 - 1.0).abs() < 5.0 * (4.0 / n).sqrt(), "m0={m0}");
        assert!((m1 + 2.0).abs() < 5.0 * (1.0 / n).sqrt(), "m1={m1}");
        // Gaussian: var(s_ij) = (s_ii s_jj + s_ij^2) / n
        assert!((c00 - 4.0).abs() < 5.0 * ((32.0) / n).sqrt(), "c00={c00}");
        assert!((c11 - 1.0).abs() < 5.0 * (2.0 / n).sqrt(), "c11={c11}");
        assert!(
            (c01 - 1.2).abs() < 5.0 * ((4.0 + 1.44) / n).sqrt(),
            "c01={c01}"
        );
        // covariance matrix with ldc < m is rejected
        assert!(rng(1)
            .multinormal(2, &mut mu, &mut c, 1, &mut out, N as RngInt)
            .is_err());
    }

    #[test]
    fn multi_students_t_mean_and_correlation() {
        // The correlation of a multivariate t equals the correlation of its scale matrix (here 0.5).
        let mut c = vec![2.0, 1.0, 1.0, 2.0];
        let mut mu = vec![3.0, -1.0];
        let mut out = vec![0.0; N * 2];
        rng(115)
            .multi_students_t(2, 8, &mut mu, &mut c, 2, &mut out, N as RngInt)
            .unwrap();
        let (m0, m1, c01) = col_stats(&out, N, 0, 1);
        let (_, _, c00) = col_stats(&out, N, 0, 0);
        let (_, _, c11) = col_stats(&out, N, 1, 1);
        let n = N as f64;
        // tolerance from the sample variance (df/(df-2) scaling is the library's convention, not asserted)
        assert!((m0 - 3.0).abs() < 5.0 * (c00 / n).sqrt(), "m0={m0}");
        assert!((m1 + 1.0).abs() < 5.0 * (c11 / n).sqrt(), "m1={m1}");
        let corr = c01 / (c00 * c11).sqrt();
        assert!((corr - 0.5).abs() < 0.05, "corr={corr}");
        // df < 2 is rejected
        assert!(rng(1)
            .multi_students_t(2, 1, &mut mu, &mut c, 2, &mut out, N as RngInt)
            .is_err());
    }

    /// Fixed-seed streams of `n` uniforms from a fresh generator.
    fn base_stream(g: BaseGenerator, n: usize) -> Vec<f64> {
        uniform_stream(&mut Rng::new(g, 77).unwrap(), n)
    }

    const STREAMS: [BaseGenerator; 2] = [
        BaseGenerator::LinearCongruential,
        BaseGenerator::WichmannHill,
    ];

    #[test]
    fn skip_ahead_matches_base_stream() {
        for g in STREAMS {
            let base = base_stream(g, 200);
            for skip in [0usize, 1, 17, 100] {
                let mut r = Rng::new(g, 77).unwrap();
                r.skip_ahead(skip as u64).unwrap();
                assert_eq!(
                    uniform_stream(&mut r, 100),
                    base[skip..skip + 100],
                    "{g:?} skip {skip}"
                );
            }
        }
        // a generator that cannot skip ahead reports it (header: info = -2)
        assert!(rng(77).skip_ahead(10).is_err());
        assert!(rng(77).skip_ahead(u64::MAX).is_err());
    }

    #[test]
    fn leapfrog_streams_partition_the_base_stream() {
        // Each leapfrog stream is every n-th value of the base stream, and the n streams k = 1..=n (1-based, per the
        // AOCL header) are disjoint and together tile it. The header also says stream k starts at base[k-1]; the
        // installed library starts it elsewhere (see the FINDING in the lane report), so only the offset-free
        // properties are asserted here.
        for g in STREAMS {
            let (n, len) = (3usize, 20usize);
            let base = base_stream(g, 200);
            let mut starts = Vec::new();
            for k in 1..=n {
                let mut r = Rng::new(g, 77).unwrap();
                r.leapfrog(n as RngInt, k as RngInt).unwrap();
                let got = uniform_stream(&mut r, len);
                let c = base
                    .iter()
                    .position(|&b| b == got[0])
                    .expect("first value is in the base stream");
                for (i, v) in got.iter().enumerate() {
                    assert_eq!(*v, base[c + i * n], "{g:?} stream {k}/{n} element {i}");
                }
                starts.push(c % n);
            }
            starts.sort_unstable();
            assert_eq!(
                starts,
                vec![0, 1, 2],
                "{g:?}: streams must hit each residue once"
            );
        }
    }

    #[test]
    fn leapfrog_rejects_bad_arguments() {
        // header: 0 < k <= n, and generators that cannot be leapfrogged return info = -3
        for (k, n) in [(0, 3), (4, 3)] {
            let mut r = Rng::new(BaseGenerator::LinearCongruential, 77).unwrap();
            assert!(r.leapfrog(n, k).is_err(), "k={k} n={n}");
        }
        assert!(rng(77).leapfrog(3, 1).is_err());
    }

    #[test]
    fn with_subid_selects_wichmann_hill_substream() {
        let g = BaseGenerator::WichmannHill;
        let a = uniform_stream(&mut Rng::with_subid(g, 1, 5).unwrap(), 50);
        // new() is subid 1
        assert_eq!(a, uniform_stream(&mut Rng::new(g, 5).unwrap(), 50));
        let b = uniform_stream(&mut Rng::with_subid(g, 2, 5).unwrap(), 50);
        assert_ne!(a, b);
        assert_eq!(
            b,
            uniform_stream(&mut Rng::with_subid(g, 2, 5).unwrap(), 50)
        );
        // valid range is 1..=273
        assert!(Rng::with_subid(g, 0, 5).is_err());
        assert!(Rng::with_subid(g, 274, 5).is_err());
        assert!(Rng::with_subid(g, 273, 5).is_ok());
        // subid is not referenced by other generators
        assert_eq!(
            uniform_stream(&mut Rng::with_subid(MT, 9, 5).unwrap(), 20),
            uniform_stream(&mut Rng::new(MT, 5).unwrap(), 20)
        );
    }

    fn bbs(s0: RngInt) -> Result<BlumBlumShub> {
        // primes 30011 and 40031, both = 3 (mod 4), one base-2^30 digit each. (A single-digit prime near 1e6, e.g.
        // 999983, makes drandinitializebbs spin forever on AOCL 6.0.68: see the FINDING in the lane report.)
        let mut p = [30_011 as RngInt];
        let mut q = [40_031 as RngInt];
        let mut s = [s0];
        BlumBlumShub::new(8, &mut p, &mut q, &mut s)
    }

    #[test]
    fn blum_blum_shub_sample_is_reproducible_and_in_30_bit_range() {
        // AOCL header: the least-significant 30 bits of each returned double are BBS output, i.e. non-negative
        // integers below 2^30 (not the [0,1) the wrapper doc claims).
        let draw = |s0| {
            let mut v = vec![0.0; 2_000];
            bbs(s0).unwrap().sample(&mut v).unwrap();
            v
        };
        let a = draw(12_345);
        assert!(a
            .iter()
            .all(|&x| x >= 0.0 && x < (1u64 << 30) as f64 && x.fract() == 0.0));
        assert!(a.iter().any(|&x| x > 1.0));
        assert_eq!(a, draw(12_345));
        assert_ne!(a, draw(12_347));
        // an empty buffer is a no-op
        bbs(12_345).unwrap().sample(&mut []).unwrap();
    }
}
