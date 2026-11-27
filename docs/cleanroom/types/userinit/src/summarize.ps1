$o = Join-Path $PSScriptRoot 'out'
$rows = foreach ($f in Get-ChildItem "$o\raw-*user*.txt" | Sort-Object Name) {
  $t = Get-Content $f.FullName
  $call = ($t | Select-String '^CALL:' | Select-Object -First 1).Line
  $m = [regex]::Match($call, 'lstate_in=(-?\d+) .*slots=(\d+) bytes=(\d+)')
  $fault = ($t | Select-String '^FAULT').Line
  $ch = ($t | Select-String '^  state\[(\d+)\].*= (0x[0-9a-f]+)' | Select-Object -First 12 | ForEach-Object { $x = [regex]::Match($_.Line, 'state\[(\d+)\].*= (0x[0-9a-f]+)'); "$($x.Groups[1].Value)=$($x.Groups[2].Value)" }) -join ' '
  $info = (($t | Select-String '^RESULT').Line -replace '^RESULT (info=\S+).*', '$1')
  $res = if ($fault) {
    $fo = [regex]::Match($fault, 'offset_from_state_start=(-?\d+) bytes \((-?\d+) slots').Groups
    $wr = [regex]::Match($fault, 'access=\d \((\w+)\)').Groups[1].Value
    $ins = if ($fault -match 'inside rng') { 'in-DLL' } else { 'HARNESS' }
    "FAULT $wr slot $($fo[2].Value) (byte $($fo[1].Value)) $ins"
  } else { "ok $info wrote: $ch" }
  "$($f.BaseName.Substring(4)) | lstate=$($m.Groups[1].Value) buf_slots=$($m.Groups[2].Value) | $res"
}
$rows | Out-File "$o\SUMMARY-user-runs.txt" -Encoding ascii
"rows=$($rows.Count) harness_faults=$(@($rows | Where-Object { $_ -match 'HARNESS' }).Count)"
