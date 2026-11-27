# aocl-utils-sys

Raw FFI bindings to AOCL-Utils. For the safe API see [`aocl-utils`](../aocl-utils/).

**Feature `header-only-unexported` (default OFF).** 5 functions the headers declare (`au_get_version_major`, `au_get_version_minor`, `au_get_version_patch`, `au_get_version_str`; `au_is_error`) are exported by NO library of AOCL 6.0.68 for Windows (measured by a link-all test; `au_get_version_*` carry `AUD_API_EXPORT` and are not inline): they are left out of the bindings unless the feature is on, and enabling it only declares them: it does not make them link on Windows. Linux is unmeasured. (Before, they were declared and any use was a link error.)

Dual-licensed under MIT or Apache-2.0.
