#!/usr/bin/env python3
"""Generate crates/aocl-blas/tests/generated_guard.rs: a guard-page harness for the free `pub fn`s of aocl-blas that take explicit
dimensions (the denominator is tools/safe-api-audit.py's table; every row of the RULES table below names its table row).

Method per function and scalar type: a CONTROL call with every buffer at the size the BLAS contract requires (must succeed: it
proves the rule is not too strict), then one case per slice argument with THAT slice one element short, placed at the end of a
committed page followed by a PAGE_NOACCESS page. A library that reads or writes past the short slice faults (0xc0000005): a HOLE.
The generated tests PRINT `RESULT <test> refused|accepted` and never assert on a short case (`refused` = the wrapper returned an
Err before the FFI call, `accepted` = it called the library and nothing faulted); only the control asserts. The driver
(tools/safe-api-harness.py) runs every test in its OWN process with a timeout and records HOLE (access violation or crash), OK, or
TIMEOUT per row. Dimensions are all different (M=3, N=4, K=5) so a swapped role changes a size. Row-major, tight leading dimensions
(the free functions take none). Data is all ones, alpha = 1, beta = 0 (the library may return inf/nan for singular systems: no
status is asserted on short cases).

    python tools/gen-blas-harness.py > crates/aocl-blas/tests/generated_guard.rs
"""
import sys

M, N, K = 3, 4, 5
# name: (generic param list names for the call, [(arg name, size expr, mutable)], call template, scalar kinds)
# In the call template, {a} etc. are replaced by the buffer expression; `A1`/`B0` are alpha=1 / beta=0 in the scalar type; `Rl` is
# the real scalar type for T::Real parameters. Enum arguments are fully qualified.
SIDES = [("L", "Side::Left"), ("R", "Side::Right")]
RULES = []


def rule(name, kinds, slices, call, variants=None):
    """kinds: 'r' real only (f32,f64), 'c' complex only, 'rc' both. variants: list of (label, {placeholders...}) overriding expr values."""
    RULES.append((name, kinds, slices, call, variants or [("", {})]))


T_N = "Trans::No"
T_T = "Trans::T"
U = "Uplo::Upper"
D = "Diag::NonUnit"

rule("gemm", "rc", [("a", "M*K", 0), ("b", "K*N", 0), ("c", "M*N", 1)],
     "gemm::<T>({ta}, {tb}, M, N, K, A1, {a}, {b}, B0, {c})",
     [("nn", {"ta": T_N, "tb": T_N}), ("tt", {"ta": T_T, "tb": T_T})])
rule("gemv", "rc", [("a", "M*N", 0), ("x", "{xn}", 0), ("y", "{ym}", 1)],
     "gemv::<T>({ta}, M, N, A1, {a}, {x}, B0, {y})",
     [("n", {"ta": T_N, "xn": "N", "ym": "M"}), ("t", {"ta": T_T, "xn": "M", "ym": "N"})])
for nm in ("trmv", "trsv"):
    rule(nm, "rc", [("a", "N*N", 0), ("x", "N", 1)], nm + "::<T>(" + U + ", " + T_N + ", " + D + ", N, {a}, {x})")
rule("symv", "r", [("a", "N*N", 0), ("x", "N", 0), ("y", "N", 1)], "symv::<T>(" + U + ", N, A1, {a}, {x}, B0, {y})")
rule("syr", "r", [("x", "N", 0), ("a", "N*N", 1)], "syr::<T>(" + U + ", N, A1, {x}, {a})")
rule("syr2", "r", [("x", "N", 0), ("y", "N", 0), ("a", "N*N", 1)], "syr2::<T>(" + U + ", N, A1, {x}, {y}, {a})")
for nm in ("ger", "geru", "gerc"):
    k = "r" if nm == "ger" else "c"
    rule(nm, k, [("x", "M", 0), ("y", "N", 0), ("a", "M*N", 1)], nm + "::<T>(M, N, A1, {x}, {y}, {a})")
rule("hemv", "c", [("a", "N*N", 0), ("x", "N", 0), ("y", "N", 1)], "hemv::<T>(" + U + ", N, A1, {a}, {x}, B0, {y})")
rule("her", "c", [("x", "N", 0), ("a", "N*N", 1)], "her::<T>(" + U + ", N, Rl1, {x}, {a})")
rule("her2", "c", [("x", "N", 0), ("y", "N", 0), ("a", "N*N", 1)], "her2::<T>(" + U + ", N, A1, {x}, {y}, {a})")
for nm, k in (("symm", "rc"), ("hemm", "c")):
    rule(nm, k, [("a", "{an}", 0), ("b", "M*N", 0), ("c", "M*N", 1)], nm + "::<T>({side}, " + U + ", M, N, A1, {a}, {b}, B0, {c})",
         [("left", {"side": "Side::Left", "an": "M*M"}), ("right", {"side": "Side::Right", "an": "N*N"})])
rule("syrk", "rc", [("a", "N*K", 0), ("c", "N*N", 1)], "syrk::<T>(" + U + ", " + T_N + ", N, K, A1, {a}, B0, {c})")
rule("syr2k", "rc", [("a", "N*K", 0), ("b", "N*K", 0), ("c", "N*N", 1)], "syr2k::<T>(" + U + ", " + T_N + ", N, K, A1, {a}, {b}, B0, {c})")
rule("herk", "c", [("a", "N*K", 0), ("c", "N*N", 1)], "herk::<T>(" + U + ", " + T_N + ", N, K, Rl1, {a}, Rl0, {c})")
rule("her2k", "c", [("a", "N*K", 0), ("b", "N*K", 0), ("c", "N*N", 1)], "her2k::<T>(" + U + ", " + T_N + ", N, K, A1, {a}, {b}, Rl0, {c})")
for nm in ("trmm", "trsm"):
    rule(nm, "rc", [("a", "{an}", 0), ("b", "M*N", 1)], nm + "::<T>({side}, " + U + ", " + T_N + ", " + D + ", M, N, A1, {a}, {b})",
         [("left", {"side": "Side::Left", "an": "M*M"}), ("right", {"side": "Side::Right", "an": "N*N"})])
for nm in ("tpmv", "tpsv"):
    rule(nm, "rc", [("ap", "N*(N+1)/2", 0), ("x", "N", 1)], nm + "::<T>(" + U + ", " + T_N + ", " + D + ", N, {ap}, {x})")
rule("spmv", "r", [("ap", "N*(N+1)/2", 0), ("x", "N", 0), ("y", "N", 1)], "spmv::<T>(" + U + ", N, A1, {ap}, {x}, B0, {y})")
rule("spr", "r", [("x", "N", 0), ("ap", "N*(N+1)/2", 1)], "spr::<T>(" + U + ", N, A1, {x}, {ap})")
rule("spr2", "r", [("x", "N", 0), ("y", "N", 0), ("ap", "N*(N+1)/2", 1)], "spr2::<T>(" + U + ", N, A1, {x}, {y}, {ap})")
rule("hpmv", "c", [("ap", "N*(N+1)/2", 0), ("x", "N", 0), ("y", "N", 1)], "hpmv::<T>(" + U + ", N, A1, {ap}, {x}, B0, {y})")
rule("hpr", "c", [("x", "N", 0), ("ap", "N*(N+1)/2", 1)], "hpr::<T>(" + U + ", N, Rl1, {x}, {ap})")
rule("hpr2", "c", [("x", "N", 0), ("y", "N", 0), ("ap", "N*(N+1)/2", 1)], "hpr2::<T>(" + U + ", N, A1, {x}, {y}, {ap})")
# band: row-major, kl = ku = 1, lda = 3 (>= kl+ku+1)
rule("gbmv", "rc", [("a", "M*3", 0), ("x", "N", 0), ("y", "M", 1)], "gbmv::<T>(Layout::RowMajor, " + T_N + ", M, N, 1, 1, A1, {a}, 3, {x}, B0, {y})")
for nm in ("tbmv", "tbsv"):
    rule(nm, "rc", [("a", "N*3", 0), ("x", "N", 1)], nm + "::<T>(Layout::RowMajor, " + U + ", " + T_N + ", " + D + ", N, 1, {a}, 3, {x})")
for nm, k in (("sbmv", "r"), ("hbmv", "c")):
    rule(nm, k, [("a", "N*3", 0), ("x", "N", 0), ("y", "N", 1)], nm + "::<T>(" + U + ", N, 1, A1, {a}, 3, {x}, B0, {y})")
rule("gemmt", "rc", [("a", "N*K", 0), ("b", "K*N", 0), ("c", "N*N", 1)],
     "gemmt::<T>(" + U + ", " + T_N + ", " + T_N + ", N, K, A1, {a}, K, {b}, N, B0, {c}, N)")
# level 1: n comes from the slices; two-slice functions get equal-length controls and a one-short second/first slice (mismatch)
rule("axpy", "rc", [("x", "N", 0), ("y", "N", 1)], "axpy::<T>(A1, {x}, {y})")
rule("axpby", "rc", [("x", "N", 0), ("y", "N", 1)], "axpby::<T>(A1, {x}, B0, {y})")
rule("copy", "rc", [("x", "N", 0), ("y", "N", 1)], "copy::<T>({x}, {y})")
rule("swap", "rc", [("x", "N", 1), ("y", "N", 1)], "swap::<T>({x}, {y})")
rule("dot", "rc", [("x", "N", 0), ("y", "N", 0)], "dot::<T>({x}, {y}).map(|_| ())")
rule("dotu", "c", [("x", "N", 0), ("y", "N", 0)], "dotu::<T>({x}, {y}).map(|_| ())")
rule("rot", "r", [("x", "N", 1), ("y", "N", 1)], "rot::<T>({x}, {y}, A1, A1)")

KIND = {"f64": ("f64", "1.0", "1.0"), "f32": ("f32", "1.0f32", "1.0f32"),
        "c64": ("Complex64", "Complex64::new(1.0, 0.0)", "1.0"), "c32": ("Complex32", "Complex32::new(1.0, 0.0)", "1.0f32")}


def scal(kind, what):
    t, one, rl = KIND[kind]
    zero = {"f64": "0.0", "f32": "0.0f32", "c64": "Complex64::new(0.0, 0.0)", "c32": "Complex32::new(0.0, 0.0)"}[kind]
    rl0 = "0.0" if kind in ("c64", "f64") else "0.0f32"
    return {"A1": one, "B0": zero, "Rl1": rl, "Rl0": rl0}[what]


def expr(e, ph):
    e = e.replace("M", str(M)).replace("N", str(N)).replace("K", str(K)) if "{" not in e else e
    return e


def main():
    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    assert_mode = "--assert" in sys.argv
    out = []
    w = out.append
    w("//! GENERATED by tools/gen-blas-harness.py (do not edit): guard-page harness for the free `pub fn`s of aocl-blas, see that script.")
    w("//! Each test prints `RESULT <name> refused|accepted`; a short case that faults is a HOLE (read by tools/safe-api-harness.py, one process per test).")
    w("#![cfg(windows)]")
    w("#![allow(clippy::all, unused_imports, unused_parens, dead_code, reason = \"generated by tools/gen-blas-harness.py\")]")
    w("use aocl_blas::*;")
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
const K: usize = 5;
fn at_guard<T: Copy, R>(n: usize, fill: T, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n * core::mem::size_of::<T>() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(VirtualProtect((base as *mut u8).add(PAGE) as *mut c_void, PAGE, 0x01, &mut old), 0);
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for i in 0..n {
            start.add(i).write(fill);
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
    for name, kinds, slices, call, variants in RULES:
        if only and name != only:
            continue
        ks = {"r": ["f64", "f32"], "c": ["c64", "c32"], "rc": ["f64", "f32", "c64", "c32"]}[kinds]
        for kind in ks:
            t, one, rl = KIND[kind]
            for label, ph in variants:
                tag = f"{name}_{kind}" + (f"_{label}" if label else "")
                def size(e):
                    e = e.format(**ph) if "{" in e else e
                    return f"({e})"
                sizes = {s[0]: size(s[1]) for s in slices}
                fill = one
                def call_with(short):
                    c = call
                    c = c.replace("::<T>", f"::<{t}>")
                    for k, v in ph.items():
                        c = c.replace("{" + k + "}", v)
                    for a, szexpr, mut in slices:
                        if a == short:
                            c = c.replace("{" + a + "}", a)  # the guard slice param
                        else:
                            c = c.replace("{" + a + "}", ("&mut " if mut else "&") + "v_" + a)
                    for sc in ("A1", "B0", "Rl1", "Rl0"):
                        c = c.replace(sc, scal(kind, sc))
                    return c
                # control
                lets = "".join(f"let {'mut ' if mut else ''}v_{a} = vec![{fill}; {sizes[a]}]; " for a, _, mut in slices)
                ctl = (f"assert!(r.is_ok(), \"{tag}_control: the call with the right sizes must succeed: {{:?}}\", r.as_ref().err());" if assert_mode else f"report(\"{tag}_control\", r.map(|_| ()));")
                w(f"#[test]\nfn {tag}_control() {{ {lets}let r = {call_with(None)}; {ctl} }}")
                # short cases
                for a, szexpr, mut in slices:
                    lets = "".join(f"let {'mut ' if m2 else ''}v_{b} = vec![{fill}; {sizes[b]}]; " for b, _, m2 in slices if b != a)
                    tail = (f"assert!(r.is_err(), \"{tag}_short_{a}: a buffer one element short must be refused before the FFI call\");" if assert_mode else f"report(\"{tag}_short_{a}\", r.map(|_| ()));")
                    w("#[test]" + NL + f"fn {tag}_short_{a}() {{ {lets}let r = at_guard::<{t}, _>({sizes[a]} - 1, {fill}, |{a}| {call_with(a)}); {tail} }}")
                    count += 1
    print(NL.join(out))
    print(f"// {count} short cases", file=sys.stderr)


NL = chr(10)
if __name__ == "__main__":
    main()
