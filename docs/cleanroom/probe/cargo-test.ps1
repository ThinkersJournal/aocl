# usage: cargo-test.ps1 <LP64|ILP64>   (run from the repo root; links the installed AOCL 6.0.68 DLLs)
param($build)
$root='C:\Program Files\AMD\AOCL-Windows'
$dirs = Get-ChildItem $root -Recurse -Filter *.dll | ForEach-Object { $_.DirectoryName } | Sort-Object -Unique
$ilp = @($dirs | Where-Object { $_ -match '\\ILP64(\\|$)' })
$non = @($dirs | Where-Object { $_ -notmatch '\\ILP64(\\|$)' })
$oneapi='C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin'
$p = if ($build -eq 'ILP64') { $ilp + $non } else { $non }
$env:PATH = (($p + $oneapi) -join ';') + ';C:\Windows\System32;C:\Windows;C:\rust\.cargo\bin;C:\Users\cires\.cargo\bin;C:\Program Files\Git\cmd'
$env:CARGO_BUILD_BUILD_DIR = "target-$build"
$env:AOCL_ROOT = $root
$f = if ($build -eq 'ILP64') { @('--features','ilp64,securerng') } else { @('--features','securerng') }
# Build, then run the test exe directly with a per-process PATH (as run.ps1 does).
cargo test -p aocl-rng-sys @f --test width_kat --test upper_kat --test securerng --no-run
$exe = Get-ChildItem "target-$build" -Recurse -Filter '*-*.exe' | Where-Object { $_.Extension -eq '.exe' }
foreach ($e in $exe) { & $e.FullName --nocapture }
"exit=$LASTEXITCODE"

