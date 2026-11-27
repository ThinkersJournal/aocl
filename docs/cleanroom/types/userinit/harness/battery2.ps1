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
    foreach ($gid in 1,2,3,4) { Run "$b-$k-genid$gid-auto" $b @("fn=$k","genid=$gid","subid=1","slots=auto") }
    Run "$b-$k-genid3-auto-minus1-NEGCTL" $b @("fn=$k","genid=3","subid=0","slots=auto-1")
  }
  foreach ($k in 'duser','suser') {
    # query semantics (guide: lstate=-1 -> returns without initialising)
    Run "$b-$k-q-lstate-1-nostateinit" $b @("fn=$k","slots=100","lstate=-1","lseed=3")
    Run "$b-$k-q-lstate-1-lseed-1" $b @("fn=$k","slots=100","lstate=-1","lseed=-1")
    Run "$b-$k-q-lstate-1-uinin1" $b @("fn=$k","slots=100","lstate=-1","lseed=3","uini_n=1")
    # earlier probe's exact call, guard added; UINI writes nothing (as the earlier probe)
    Run "$b-$k-earlier-1M-slots-lstate100" $b @("fn=$k","slots=1000000","lstate=100","lseed=3","genid=5","subid=7")
    Run "$b-$k-earlier-1M-slots-lstate100-fillA5" $b @("fn=$k","slots=1000000","lstate=100","lseed=3","genid=5","subid=7","fill=a5")
    # self-consistent buffer==lstate, UINI writes nothing (state[0] stays 0)
    foreach ($n in 100, 4096, 32760, 32761, 32762, 32768, 65536) { Run "$b-$k-noprotocol-buf$n" $b @("fn=$k","slots=$n","lstate=slots") }
    Run "$b-$k-noprotocol-buf1MiB-lstate-slots" $b @("fn=$k","slots=$mib1","lstate=slots")
    Run "$b-$k-noprotocol-buf1MiB-lstate100" $b @("fn=$k","slots=$mib1","lstate=100")
    Run "$b-$k-noprotocol-buf16MiB-lstate100" $b @("fn=$k","slots=$mib16","lstate=100")
    Run "$b-$k-noprotocol-buf16MiB-lstate-slots" $b @("fn=$k","slots=$mib16","lstate=slots")
    # UINI sets state[0]=n (declared user length), library should append after it
    foreach ($n in 0, 1, 5, 97, 98, 99, 1000) { Run "$b-$k-uinin$n-buf100-lstate100" $b @("fn=$k","slots=100","lstate=100","uini_n=$n") }
    Run "$b-$k-uinin1-buf1MiB-lstateslots" $b @("fn=$k","slots=$mib1","lstate=slots","uini_n=1")
    Run "$b-$k-uinin1-buf16MiB-lstateslots" $b @("fn=$k","slots=$mib16","lstate=slots","uini_n=1")
    Run "$b-$k-uinin97-buf1MiB-lstateslots" $b @("fn=$k","slots=$mib1","lstate=slots","uini_n=97")
    Run "$b-$k-uinin1-buf4-lstate4" $b @("fn=$k","slots=4","lstate=4","uini_n=1")
    Run "$b-$k-uinin1-buf3-lstate4-NEGCTL" $b @("fn=$k","slots=3","lstate=4","uini_n=1")
    # UGEN address dependence, UINI writes nothing (n=0 aliasing path) and with n=1
    Run "$b-$k-ugen-fn-a" $b @("fn=$k","slots=$mib16","lstate=100","ugen=fn")
    Run "$b-$k-ugen-fn-b" $b @("fn=$k","slots=$mib16","lstate=100","ugen=fnb")
    foreach ($h in '10000000','100000000','500000000','2000000000','70000000000','700000000000') {
      Run "$b-$k-ugen-tramp-$h" $b @("fn=$k","slots=$mib16","lstate=100","ugen=t:$h")
      Run "$b-$k-ugen-tramp-$h-uinin1" $b @("fn=$k","slots=$mib16","lstate=100","ugen=t:$h","uini_n=1")
    }
    Run "$b-$k-uini-fnb" $b @("fn=$k","slots=$mib16","lstate=100","uini=fnb")
    Run "$b-$k-uini-tramp-500000000" $b @("fn=$k","slots=$mib16","lstate=100","uini=t:500000000")
    Run "$b-$k-uini-tramp-500000000-ugen-tramp-2000000000" $b @("fn=$k","slots=$mib16","lstate=100","uini=t:500000000","ugen=t:2000000000")
    Run "$b-$k-uini-writes-0x5100-seq" $b @("fn=$k","slots=$mib16","lstate=100","uini_writes=1")
    foreach ($l in 0,1,2,3,4,5,6) { Run "$b-$k-lstate$l-uinin1-bigbuf" $b @("fn=$k","slots=$mib1","lstate=$l","uini_n=1") }
  }
}
