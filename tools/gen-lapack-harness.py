#!/usr/bin/env python3
"""Generate crates/aocl-lapack/tests/generated_guard.rs: a guard-page harness for free `pub fn`s of aocl-lapack (see
tools/gen-blas-harness.py for the method: a CONTROL call with the contractual sizes must succeed, then one case per slice argument with
that slice one element short, at the end of a committed page before a PAGE_NOACCESS page; a fault = HOLE).

Row-major, tight leading dimensions (lda = columns). Dimensions: M = 3, N = 4, NRHS = 2 (all different). Data: `a` arguments are the
identity (so potrf / getrf / sytrf / heev ... succeed), every other buffer is ones; `ipiv` buffers are ones (valid 1-based pivots).
The generated tests print `RESULT <name> refused|accepted` for short cases (never assert on them) unless --assert is given (a fix PR's
regression file: short cases must be refused, controls must succeed).

    python tools/gen-lapack-harness.py [--only NAME] [--assert] > crates/aocl-lapack/tests/generated_guard.rs
Only the rows of RULES are covered; every other row of the audit table stays "read pending".
"""
import sys

NL = chr(10)
RULES = []
# kinds: "rc" = f64,f32,c64,c32 (generic fn `name::<T>`); "r" = f64,f32; "c" = c64,c32
# typed = True: the function name is `<p>name` with p in d,s,z,c (real-only: d,s; complex-only: z,c) and no turbofish.


def rule(name, kinds, slices, call, variants=None, typed=False):
    RULES.append((name, kinds, slices, call, variants or [("", {})], typed))


# slices: (arg name, size expr in M/N/R(=NRHS), mutable?, init kind) with init: "id" identity, "one" ones, "piv" ones (i32)
def S(a, size, mut=1, init="one", ty=None):
    return (a, size, mut, init, ty)


# ---- linear systems (generic names) ----
rule("gesv", "rc", [S("a", "N*N", 1, "id"), S("ipiv", "N", 1, "piv", "i32"), S("b", "N*R")], "gesv::<T>(N, {a}, {ipiv}, {b})")
rule("getrf", "rc", [S("a", "M*N", 1, "id"), S("ipiv", "M", 1, "piv", "i32")], "getrf::<T>(M, N, {a}, {ipiv})")
rule("getrs", "rc", [S("a", "N*N", 0, "id"), S("ipiv", "N", 0, "piv", "i32"), S("b", "N*R")], "getrs::<T>(Trans::No, N, {a}, {ipiv}, {b})")
rule("getri", "rc", [S("a", "N*N", 1, "id"), S("ipiv", "N", 0, "piv", "i32")], "getri::<T>(N, {a}, {ipiv})")
rule("posv", "rc", [S("a", "N*N", 1, "id"), S("b", "N*R")], "posv::<T>(Uplo::Upper, N, {a}, {b})")
rule("potrf", "rc", [S("a", "N*N", 1, "id")], "potrf::<T>(Uplo::Upper, N, {a})")
rule("potrs", "rc", [S("a", "N*N", 0, "id"), S("b", "N*R")], "potrs::<T>(Uplo::Upper, N, {a}, {b})")
rule("sysv", "rc", [S("a", "N*N", 1, "id"), S("ipiv", "N", 1, "piv", "i32"), S("b", "N*R")], "sysv::<T>(Uplo::Upper, N, {a}, {ipiv}, {b})")
rule("hesv", "c", [S("a", "N*N", 1, "id"), S("ipiv", "N", 1, "piv", "i32"), S("b", "N*R")], "hesv::<T>(Uplo::Upper, N, {a}, {ipiv}, {b})")
rule("gtsv", "rc", [S("dl", "(N-1)"), S("d", "N"), S("du", "(N-1)"), S("b", "N*R")], "gtsv::<T>(N, {dl}, {d}, {du}, {b})")
rule("geqrf", "rc", [S("a", "M*N", 1, "id"), S("tau", "M")], "geqrf::<T>(M, N, {a}, {tau})")
rule("gelqf", "rc", [S("a", "M*N", 1, "id"), S("tau", "M")], "gelqf::<T>(M, N, {a}, {tau})")
rule("syev", "r", [S("a", "N*N", 1, "id"), S("w", "N")], "syev::<T>(Compute::ValuesAndVectors, Uplo::Upper, N, {a}, {w})")
# ---- typed (explicit lda) ----
rule("{p}trtrs", "r", [S("a", "N*N", 0, "id"), S("b", "N*R")], "{p}trtrs(Uplo::Upper, Trans::No, aocl_types::Diag::NonUnit, N, R, {a}, N, {b}, R)", typed=True)
rule("{p}trtri", "r", [S("a", "N*N", 1, "id")], "{p}trtri(Uplo::Upper, aocl_types::Diag::NonUnit, N, {a}, N)", typed=True)
rule("{p}potri", "r", [S("a", "N*N", 1, "id")], "{p}potri(Uplo::Upper, N, {a}, N)", typed=True)
rule("{p}lacpy", "rc", [S("a", "M*N", 0, "id"), S("b", "M*N")], "NR:{p}lacpy(Uplo::Upper, M, N, {a}, N, {b}, N)", typed=True)
rule("{p}lange", "rc", [S("a", "M*N", 0, "id")], "NR:{p}lange(Norm::One, M, N, {a}, N)", typed=True)
rule("{p}sytrf", "rc", [S("a", "N*N", 1, "id"), S("ipiv", "N", 1, "piv", "i32")], "{p}sytrf(Uplo::Upper, N, {a}, N, {ipiv})", typed=True)
rule("{p}sytrs", "rc", [S("a", "N*N", 0, "id"), S("ipiv", "N", 0, "piv", "i32"), S("b", "N*R")], "{p}sytrs(Uplo::Upper, N, R, {a}, N, {ipiv}, {b}, R)", typed=True)
rule("{p}sytri", "rc", [S("a", "N*N", 1, "id"), S("ipiv", "N", 0, "piv", "i32")], "{p}sytri(Uplo::Upper, N, {a}, N, {ipiv})", typed=True)
rule("{p}hetrf", "c", [S("a", "N*N", 1, "id"), S("ipiv", "N", 1, "piv", "i32")], "{p}hetrf(Uplo::Upper, N, {a}, N, {ipiv})", typed=True)
rule("{p}hetrs", "c", [S("a", "N*N", 0, "id"), S("ipiv", "N", 0, "piv", "i32"), S("b", "N*R")], "{p}hetrs(Uplo::Upper, N, R, {a}, N, {ipiv}, {b}, R)", typed=True)
rule("{p}hetri", "c", [S("a", "N*N", 1, "id"), S("ipiv", "N", 0, "piv", "i32")], "{p}hetri(Uplo::Upper, N, {a}, N, {ipiv})", typed=True)
rule("{p}gecon", "rc", [S("a", "N*N", 0, "id")], "{p}gecon(Norm::One, N, {a}, N, ONE_RL).map(|_| ())", typed=True)
rule("{p}pocon", "rc", [S("a", "N*N", 0, "id")], "{p}pocon(Uplo::Upper, N, {a}, N, ONE_RL).map(|_| ())", typed=True)
rule("{p}gehrd", "rc", [S("a", "N*N", 1, "id"), S("tau", "N")], "{p}gehrd(N, 1, N as i32, {a}, N, {tau})", typed=True)
rule("{p}geqp3", "rc", [S("a", "M*N", 1, "id"), S("jpvt", "N", 1, "piv", "i32"), S("tau", "M")], "{p}geqp3(M, N, {a}, N, {jpvt}, {tau})", typed=True)
rule("{p}gelss", "rc", [S("a", "M*N", 1, "id"), S("b", "N*R"), S("s", "M", 1, "one", "real")], "{p}gelss(M, N, R, {a}, N, {b}, R, {s}, ONE_RL).map(|_| ())", typed=True)
rule("{p}orgqr", "r", [S("a", "M*N", 1, "id"), S("tau", "M", 0)], "{p}orgqr(M, M, M, {a}, N, {tau})", typed=True)
rule("{p}ungqr", "c", [S("a", "M*N", 1, "id"), S("tau", "M", 0)], "{p}ungqr(M, M, M, {a}, N, {tau})", typed=True)

KIND = {
    "f64": ("f64", "d", "1.0", "0.0"), "f32": ("f32", "s", "1.0f32", "0.0f32"),
    "c64": ("Complex64", "z", "Complex64::new(1.0, 0.0)", "Complex64::new(0.0, 0.0)"),
    "c32": ("Complex32", "c", "Complex32::new(1.0, 0.0)", "Complex32::new(0.0, 0.0)"),
}


def main():
    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    assert_mode = "--assert" in sys.argv
    out = []
    w = out.append
    w("//! GENERATED by tools/gen-lapack-harness.py (do not edit): guard-page harness for free `pub fn`s of aocl-lapack, see that script.")
    w("#![cfg(windows)]")
    w('#![allow(clippy::all, unused_imports, unused_parens, dead_code, reason = "generated by tools/gen-lapack-harness.py")]')
    w("use aocl_lapack::*;")
    w("use aocl_types::{Complex32, Complex64};")
    w("use core::ffi::c_void;")
    w("""extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
const M: usize = 3;
const N: usize = 4;
const R: usize = 2;
fn at_guard<T: Copy, X>(n: usize, init: impl Fn(usize) -> T, f: impl FnOnce(&mut [T]) -> X) -> X {
    assert!(n * core::mem::size_of::<T>() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(VirtualProtect((base as *mut u8).add(PAGE) as *mut c_void, PAGE, 0x01, &mut old), 0);
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for i in 0..n {
            start.add(i).write(init(i));
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}
fn report<T: core::fmt::Debug>(name: &str, r: core::result::Result<T, aocl_error::Error>) {
    println!("RESULT {name} {}", if r.is_err() { "refused" } else { "accepted" });
}
""")
    count = 0
    for name, kinds, slices, call, variants, typed in RULES:
        if only and name.replace("{p}", "") != only and name != only:
            continue
        ks = {"r": ["f64", "f32"], "c": ["c64", "c32"], "rc": ["f64", "f32", "c64", "c32"]}[kinds]
        for kind in ks:
            t, p, one, zero = KIND[kind]
            one_rl = "1.0" if kind in ("f64", "c64") else "1.0f32"
            for label, ph in variants:
                fn = name.replace("{p}", p)
                tag = f"{fn}_{kind}" + (f"_{label}" if label else "")

                def ty_of(s):
                    if s[4] == "real":
                        return "f64" if kind in ("f64", "c64") else "f32"
                    return s[4] or t

                def initexpr(s, zero_one):
                    a, size, mut, init, ty = s
                    if ty == "i32":
                        return "|_| 1i32"
                    if ty == "real":
                        return "|_| " + ("1.0" if kind in ("f64", "c64") else "1.0f32")
                    if init == "id":
                        return f"|i| if i % ({size.split('*')[1] if '*' in size else 'N'} + 1) == 0 {{ {one} }} else {{ {zero} }}"
                    return f"|_| {one}"

                def call_with(short):
                    c = call.replace("{p}", p).replace("::<T>", f"::<{t}>").replace("ONE_RL", one_rl)
                    for a, size, mut, init, ty in slices:
                        c = c.replace("{" + a + "}", a if a == short else ("&mut " if mut else "&") + "v_" + a)
                    if c.startswith("NR:"):
                        c = "{ let _ = " + c[3:] + "; Ok::<(), aocl_error::Error>(()) }"
                    return c

                lets = lambda skip: "".join(
                    f"let {'mut ' if s[2] else ''}v_{s[0]}: Vec<{ty_of(s)}> = (0..({s[1]})).map({initexpr(s, 1)}).collect(); " for s in slices if s[0] != skip)
                ctl = (f'assert!(r.is_ok(), "{tag}_control: {{:?}}", r.as_ref().err());' if assert_mode else f'report("{tag}_control", r.map(|_| ()));')
                w("#[test]" + NL + f"fn {tag}_control() {{ {lets(None)}let r = {call_with(None)}; {ctl} }}")
                for s in slices:
                    a = s[0]
                    tail = (f'assert!(r.is_err(), "{tag}_short_{a}: a buffer one element short must be refused before the FFI call");' if assert_mode
                            else f'report("{tag}_short_{a}", r.map(|_| ()));')
                    w("#[test]" + NL + f"fn {tag}_short_{a}() {{ {lets(a)}let r = at_guard::<{ty_of(s)}, _>(({s[1]}) - 1, {initexpr(s, 1)}, |{a}| {call_with(a)}); {tail} }}")
                    count += 1
    print(NL.join(out))
    print(f"// {count} short cases", file=sys.stderr)


if __name__ == "__main__":
    main()
