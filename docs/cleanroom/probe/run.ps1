# usage: run.ps1 <LP64|ILP64> <argwidth> <case>
param($build,$w,$case)
$root='C:\Program Files\AMD\AOCL-Windows'
$dirs = Get-ChildItem $root -Recurse -Filter *.dll | ForEach-Object { $_.DirectoryName } | Sort-Object -Unique
$ilp = @($dirs | Where-Object { $_ -match '\\ILP64(\\|$)' })
$non = @($dirs | Where-Object { $_ -notmatch '\\ILP64(\\|$)' })
$oneapi='C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin'
if ($build -eq 'ILP64') { $p = $ilp + $non } else { $p = $non }
$env:PATH = (($p + $oneapi) -join ';') + ';C:\Windows\System32;C:\Windows'
if ($env:KMP_DUPLICATE_LIB_OK) { Write-Output "KMP_DUPLICATE_LIB_OK set!" }
if ($case -eq 'path') { $env:PATH -split ';'; return }
$dll = "$root\amd-rng\lib\$build\rng_amd.dll"
& "$PSScriptRoot\probe.exe" $dll $w $case
Write-Output "exit_code=$LASTEXITCODE (0x$('{0:x}' -f $LASTEXITCODE))"
