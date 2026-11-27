# usage (from the repo root): pwsh docs/cleanroom/probe/cargo-test-simd.ps1
# NIGHTLY ONLY: by-value SIMD LibM functions (feature simd_ffi); links the installed AOCL 6.0.68 libalm (LP64).
$root = 'C:\Program Files\AMD\AOCL-Windows'
$dirs = Get-ChildItem $root -Recurse -Filter *.dll | ForEach-Object { $_.DirectoryName } | Sort-Object -Unique | Where-Object { $_ -notmatch '[\\/]ILP64([\\/]|$)' }
$oneapi = 'C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin'
$env:PATH = (($dirs + $oneapi) -join ';') + ';C:\Windows\System32;C:\Windows;C:\rust\.cargo\bin;C:\Users\cires\.cargo\bin;C:\Program Files\Git\cmd'
$env:CARGO_BUILD_BUILD_DIR = 'target-simd'
$env:AOCL_ROOT = $root
rustc +nightly -V
cargo +nightly test -p aocl-math-sys --features simd_ffi --test simd -- --nocapture
"exit=$LASTEXITCODE"
