# usage: run-all.ps1  (writes ../out/raw-<build>-<case>.txt)
$out = Join-Path $PSScriptRoot '..\out'
$crash = 'crash_get_rdrand32u_null','crash_get_rdrand32u_ptrasval','crash_get_rdrand32u_swapped','crash_get_rdseed32u_null','crash_get_rdrand16u_null','crash_get_rdrand64u_null','crash_get_rdseed64u_null','crash_get_rdseed16u_null','crash_get_rdrand32u_arr_null','crash_get_rdrand32u_arr_null0','crash_get_rdseed32u_arr_null','crash_get_rdrand64u_arr_null','crash_get_rdseed64u_arr_null','crash_get_rdrand_bytes_arr_null','crash_get_rdrand_bytes_arr_null0','crash_get_rdrand_bytes_arr_swapped','crash_get_rdseed_bytes_arr_null'
foreach ($b in 'LP64','ILP64','SECLP64') {
  foreach ($c in (@('info','single','arr','retry','stat') + $crash)) {
    & "$PSScriptRoot\run.ps1" $b $c 2>&1 | Out-File -Encoding utf8 "$out\raw-$b-$c.txt"
  }
}
