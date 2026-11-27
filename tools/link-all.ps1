# Link-all check: every declared symbol of aocl-math-sys and aocl-rng-sys (+ SecureRNG) resolves in the installed AOCL.
# A declaration that cannot link is the defect class of the yanked aocl 0.1.0 and the 20 amd_fast* of 0.2.7.
# Needs AOCL (AOCL_ROOT or %ProgramFiles%\AMD\AOCL-Windows) + oneAPI compiler runtime; fails loudly, installs nothing.
$ErrorActionPreference = 'Stop'
$root = if ($env:AOCL_ROOT) { $env:AOCL_ROOT } else { Join-Path $env:ProgramFiles 'AMD\AOCL-Windows' }
if (-not (Test-Path -LiteralPath $root)) { throw "AOCL not found at $root" }
$env:AOCL_ROOT = $root
$alldirs = Get-ChildItem $root -Recurse -Filter *.dll | ForEach-Object { $_.DirectoryName } | Sort-Object -Unique
$ilp = @($alldirs | Where-Object { $_ -match 'ILP64' })
$non = @($alldirs | Where-Object { $_ -notmatch 'ILP64' })
$oneapi = Get-ChildItem 'C:\Program Files (x86)\Intel\oneAPI\compiler' -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^\d' } | Sort-Object Name | Select-Object -Last 1
$extra = if ($oneapi) { @(Join-Path $oneapi.FullName 'bin') } else { @() }
# NOT the caller's PATH: it may list other AOCL dirs (an ILP64 rng_amd.dll ahead of the LP64 one makes the LP64 known answer fail).
$cargoDir = @((Get-Command cargo -ErrorAction SilentlyContinue | ForEach-Object { Split-Path $_.Source }), (Join-Path $env:CARGO_HOME 'bin'), (Join-Path $env:USERPROFILE '.cargo\bin'), 'C:\rust\.cargo\bin') | Where-Object { $_ -and (Test-Path (Join-Path $_ 'cargo.exe')) } | Select-Object -First 1
if (-not $cargoDir) { throw 'cargo not found' }
$tail = ';C:\Windows\System32;C:\Windows;' + $cargoDir + ';' + (Join-Path $env:ProgramFiles 'Git\cmd')
# aocl-securerng-sys (bindgen route) needs libclang: LIBCLANG_PATH if set, else the Visual Studio LLVM x64 bin.
if (-not $env:LIBCLANG_PATH) {
  $llvm = Get-ChildItem (Join-Path $env:ProgramFiles 'Microsoft Visual Studio') -Directory -ErrorAction SilentlyContinue | ForEach-Object { Get-ChildItem $_.FullName -Directory } | ForEach-Object { Join-Path $_.FullName 'VC\Tools\Llvm\x64\bin' } | Where-Object { Test-Path (Join-Path $_ 'libclang.dll') } | Select-Object -First 1
  if (-not $llvm) { throw 'libclang not found: set LIBCLANG_PATH' }
  $env:LIBCLANG_PATH = $llvm
}
foreach ($w in 'LP64', 'ILP64') {
  $p = if ($w -eq 'ILP64') { $ilp + $non } else { $non }
  $env:PATH = (($p + $extra) -join ';') + $tail
  $env:CARGO_BUILD_BUILD_DIR = "target-linkall-$w"
  $f = if ($w -eq 'ILP64') { 'ilp64' } else { '' }
  cargo test -p aocl-rng-sys --features $f --test link_all_lower --test upper_kat
  if ($LASTEXITCODE -ne 0) { throw "aocl-rng-sys link-all failed ($w)" }
}
# LibM has no integer-width split.
$env:PATH = (($non + $extra) -join ';') + $tail
$env:CARGO_BUILD_BUILD_DIR = 'target-linkall-math'
cargo test -p aocl-math-sys --test link_all
if ($LASTEXITCODE -ne 0) { throw 'aocl-math-sys link-all failed' }
# SecureRNG (separate library amdsecrng; LP64 build, the ILP64 DLL is byte-identical in its sections).
$env:CARGO_BUILD_BUILD_DIR = 'target-linkall-sec'
cargo test -p aocl-securerng-sys --test securerng
if ($LASTEXITCODE -ne 0) { throw 'aocl-securerng-sys link-all failed' }
'link-all: OK'
