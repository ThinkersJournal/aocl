$here = $PSScriptRoot
$out = Join-Path $here 'out'
function Run($name, $build, $a) {
  $f = Join-Path $out "raw-$name.txt"
  "### run.ps1 $build 40 $($a -join ' ')" | Out-File $f -Encoding ascii
  & "$here\run.ps1" $build 40 @a 2>&1 | Out-File $f -Append -Encoding ascii
}
Run 'selftest' 'LP64' @('fn=selftest')
foreach ($b in 'LP64','ILP64') {
  $w = if ($b -eq 'ILP64') { 8 } else { 4 }
  $mib1 = 1048576 / $w; $mib16 = 16777216 / $w
  # ---- CONTROL: non-user init, guide-stated length via lstate=-1/lseed=-1 ----
  foreach ($k in 'dinit','sinit') {
    foreach ($gid in 1,2,3,4) {
      Run "$b-$k-genid$gid-auto" $b @("fn=$k","genid=$gid","subid=1","slots=auto")
    }
    Run "$b-$k-genid3-auto-minus1-NEGCTL" $b @("fn=$k","genid=3","subid=0","slots=auto-1")
  }
  foreach ($k in 'duser','suser') {
    # ---- query semantics ----
    Run "$b-$k-q-lstate-1" $b @("fn=$k","slots=100","lstate=-1","lseed=3")
    Run "$b-$k-q-lstate-1-lseed-1" $b @("fn=$k","slots=100","lstate=-1","lseed=-1")
    # ---- the earlier probe's exact call (1,000,000 slots, lstate=100, lseed=3, genid=5, subid=7), now with end guard ----
    Run "$b-$k-earlier-1M-slots-lstate100" $b @("fn=$k","slots=1000000","lstate=100","lseed=3","genid=5","subid=7")
    Run "$b-$k-earlier-1M-slots-lstate100-fill00" $b @("fn=$k","slots=1000000","lstate=100","lseed=3","genid=5","subid=7","fill=00")
    # ---- buffer == lstate (self-consistent call), then bigger ----
    foreach ($n in 100, 4096, 32760, 32761, 32762, 32768, 65536) {
      Run "$b-$k-buf$n-lstate$n" $b @("fn=$k","slots=$n","lstate=slots")
    }
    Run "$b-$k-buf1MiB-lstate-slots" $b @("fn=$k","slots=$mib1","lstate=slots")
    Run "$b-$k-buf1MiB-lstate100" $b @("fn=$k","slots=$mib1","lstate=100")
    Run "$b-$k-buf16MiB-lstate-slots" $b @("fn=$k","slots=$mib16","lstate=slots")
    # ---- UGEN address dependence (state = 16 MiB with end guard) ----
    Run "$b-$k-ugen-fn-a" $b @("fn=$k","slots=$mib16","lstate=100","ugen=fn")
    Run "$b-$k-ugen-fn-b" $b @("fn=$k","slots=$mib16","lstate=100","ugen=fnb")
    foreach ($h in '10000000','100000000','500000000','2000000000','70000000000','700000000000') {
      Run "$b-$k-ugen-tramp-$h" $b @("fn=$k","slots=$mib16","lstate=100","ugen=t:$h")
    }
    # ---- UINI address dependence (UGEN fixed) ----
    Run "$b-$k-uini-fnb" $b @("fn=$k","slots=$mib16","lstate=100","uini=fnb")
    Run "$b-$k-uini-tramp-500000000" $b @("fn=$k","slots=$mib16","lstate=100","uini=t:500000000")
    Run "$b-$k-uini-tramp-500000000-ugen-tramp-2000000000" $b @("fn=$k","slots=$mib16","lstate=100","uini=t:500000000","ugen=t:2000000000")
    # ---- UINI state writes: who wins ----
    Run "$b-$k-uini-writes" $b @("fn=$k","slots=$mib16","lstate=100","uini_writes=1")
    # ---- lstate threshold ----
    foreach ($l in 0,1,2,3,4,5,6) { Run "$b-$k-lstate$l-bigbuf" $b @("fn=$k","slots=$mib1","lstate=$l") }
  }
}
