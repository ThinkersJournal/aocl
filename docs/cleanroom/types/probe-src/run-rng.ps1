# usage: run-rng.ps1 <LP64|ILP64> <args...>   PATH per child process (ILP64 dirs first for ILP64), as cargo-test.ps1
param($build, [Parameter(ValueFromRemainingArguments=$true)]$rest)
$root='C:\Program Files\AMD\AOCL-Windows'
$dirs = Get-ChildItem $root -Recurse -Filter *.dll | ForEach-Object { $_.DirectoryName } | Sort-Object -Unique
$ilp = @($dirs | Where-Object { $_ -match '\\ILP64(\\|$)' })
$non = @($dirs | Where-Object { $_ -notmatch '\\ILP64(\\|$)' })
$oneapi='C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin'
$p = if ($build -eq 'ILP64') { $ilp + $non } else { $non }
$env:PATH = (($p + $oneapi) -join ';') + ';C:\Windows\System32;C:\Windows'
$exe = if ($env:PROBE_EXE) { $env:PROBE_EXE } else { "$PSScriptRoot\bin\rng_probe.exe" }
& $exe $build @rest
Write-Output "exit_code=$LASTEXITCODE (0x$('{0:x}' -f $LASTEXITCODE))"
