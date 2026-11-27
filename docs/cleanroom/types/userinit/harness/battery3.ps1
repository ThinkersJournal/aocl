$here = $PSScriptRoot; $out = Join-Path $here 'out'
function Run($name, $build, $a) { $f = Join-Path $out "raw-$name.txt"; "### run.ps1 $build 40 $($a -join ' ')" | Out-File $f -Encoding ascii; & "$here\run.ps1" $build 40 @a 2>&1 | Out-File $f -Append -Encoding ascii }
foreach ($b in 'LP64','ILP64') { foreach ($k in 'duser','suser') {
  Run "$b-$k-GEN-uinin1-buf100" $b @("fn=$k","slots=100","lstate=100","uini_n=1","gen=1")
  Run "$b-$k-GEN-uinin5-buf100" $b @("fn=$k","slots=100","lstate=100","uini_n=5","gen=1")
  Run "$b-$k-GEN-uinin97-buf100" $b @("fn=$k","slots=100","lstate=100","uini_n=97","gen=1")
  Run "$b-$k-GEN-NEGCTL-noprotocol-bigbuf" $b @("fn=$k","slots=65536","lstate=100","gen=1")
} }
