# usage: run.ps1 <LP64|ILP64|SECLP64> <case>   PATH is set for this child process only; no OpenMP dir on it.
param($build,$case)
$root='C:\Program Files\AMD\AOCL-Windows'
$dll = switch ($build) { 'LP64' {"$root\amd-rng\lib\LP64\amdsecrng.dll"} 'ILP64' {"$root\amd-rng\lib\ILP64\amdsecrng.dll"} 'SECLP64' {"$root\amd-securerng\lib\LP64\amdsecrng.dll"} }
$env:PATH = (Split-Path $dll) + ';C:\Windows\System32;C:\Windows'
& "$PSScriptRoot\probe.exe" $dll $case
Write-Output "exit_code=$LASTEXITCODE (0x$('{0:x}' -f $LASTEXITCODE))"
