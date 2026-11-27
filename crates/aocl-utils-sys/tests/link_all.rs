//! Every function the crate declares by default resolves in the installed AOCL-Utils (taking the address forces the link).
//! The 5 names left out by default (`au_get_version_*`, `au_is_error`; feature `header-only-unexported`) are exported by NO
//! installed library on AOCL 6.0.68 Windows (measured), so they are not in this test. Linux unmeasured.
#![allow(warnings)]
use aocl_utils_sys as s;

#[test]
fn declared_symbols_link() {
    let addrs: Vec<usize> = vec![
        s::GetActiveProcessorCount as usize,
        s::GetActiveProcessorGroupCount as usize,
        s::GetCurrentThread as usize,
        s::GetThreadGroupAffinity as usize,
        s::SetThreadAffinityMask as usize,
        s::SetThreadGroupAffinity as usize,
        s::au_cpuid_arch_is_x86_64v2 as usize,
        s::au_cpuid_arch_is_x86_64v3 as usize,
        s::au_cpuid_arch_is_x86_64v4 as usize,
        s::au_cpuid_arch_is_zen as usize,
        s::au_cpuid_arch_is_zen2 as usize,
        s::au_cpuid_arch_is_zen3 as usize,
        s::au_cpuid_arch_is_zen4 as usize,
        s::au_cpuid_arch_is_zen5 as usize,
        s::au_cpuid_arch_is_zen6 as usize,
        s::au_cpuid_arch_is_zen_family as usize,
        s::au_cpuid_arch_is_zenplus as usize,
        s::au_cpuid_get_core_type as usize,
        s::au_cpuid_get_vendor as usize,
        s::au_cpuid_has_flags as usize,
        s::au_cpuid_has_flags_all as usize,
        s::au_cpuid_has_flags_any as usize,
        s::au_cpuid_is_amd as usize,
        s::au_cpuid_is_error as usize,
        s::au_cpuid_is_hybrid as usize,
        s::au_pin_threads_core as usize,
        s::au_pin_threads_custom as usize,
        s::au_pin_threads_logical as usize,
        s::au_pin_threads_spread as usize,
    ];
    assert_eq!(addrs.len(), 29);
    assert!(addrs.iter().all(|a| *a != 0));
}
