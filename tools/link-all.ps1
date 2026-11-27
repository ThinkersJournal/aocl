# Link-all check: every declared symbol of aocl-math-sys (the RNG crates join in their own PR) resolves in the installed AOCL.
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
# LibM has no integer-width split.
$env:PATH = (($non + $extra) -join ';') + $tail
$env:CARGO_BUILD_BUILD_DIR = 'target-linkall-math'
cargo test -p aocl-math-sys --test link_all
if ($LASTEXITCODE -ne 0) { throw 'aocl-math-sys link-all failed' }
'link-all: OK'
