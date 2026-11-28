//! Out-of-range CPU numbers for the aocl-utils cpuid functions (the library takes a CPU index; `Cpu::Specific(n)` is caller-chosen). One process
//! per test (tools/safe-api-harness.py); a fault = HOLE. Hand-written, Windows only, Linux unmeasured.
use aocl_utils::cpuid::*;

macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let r = $body;
            println!("RESULT {} accepted ({r:?})", stringify!($name));
        }
    };
}
#[test]
fn current_control() {
    let _ = vendor_info(Cpu::Current);
    let _ = has_flag(Cpu::Current, "avx2");
}
case!(has_flag_cpu_1e6, has_flag(Cpu::Specific(1_000_000), "avx2"));
case!(has_flag_cpu_max_minus_1, has_flag(Cpu::Specific(u32::MAX - 1), "avx2"));
case!(has_flags_all_cpu_1e6, has_flags_all(Cpu::Specific(1_000_000), &["avx2", "sse2"]));
case!(has_flags_any_cpu_1e6, has_flags_any(Cpu::Specific(1_000_000), &["avx2", "sse2"]));
case!(vendor_info_cpu_1e6, format!("{:?}", vendor_info(Cpu::Specific(1_000_000))));
case!(vendor_info_cpu_max_minus_1, format!("{:?}", vendor_info(Cpu::Specific(u32::MAX - 1))));
