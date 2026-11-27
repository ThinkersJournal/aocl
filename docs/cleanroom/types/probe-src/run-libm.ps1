# usage: run-libm.ps1 <case>   PATH set per child process only
param($case)
$root='C:\Program Files\AMD\AOCL-Windows'
$env:PATH = "$root\amd-libm\lib;C:\Program Files (x86)\Intel\oneAPI\compiler\2026.1\bin;C:\Windows\System32;C:\Windows"
& "$PSScriptRoot\bin\libm_probe.exe" $case
Write-Output "exit_code=$LASTEXITCODE (0x$('{0:x}' -f $LASTEXITCODE))"
