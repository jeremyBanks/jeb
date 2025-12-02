#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x



jeb ./stackexchange-posts.xml xml-to-jsonl stdout
JEB



{"":"?xml","@text":" version=\"1.0\" encoding=\"utf-8\"","@tail":"\n","@index":0}
{"":"posts","@text":"\n  ","@tail":"\n","@index":1}
{"":"row","-":"posts","Id":"1","PostTypeId":"1","AcceptedAnswerId":"3","CreationDate":"2016-01-12T18:45:19.963","Score":"10","ViewCount":"1000","Body":"<p>What is the best way to level a 3D printer bed?</p>","OwnerUserId":"1","Title":"How to level a 3D printer bed?","Tags":"<bed-leveling><calibration>","AnswerCount":"2","CommentCount":"1","@tail":"\n  ","@index":0}
{"":"row","-":"posts","Id":"2","PostTypeId":"1","CreationDate":"2016-01-12T19:00:00.000","Score":"5","ViewCount":"500","Body":"<p>What materials work best for <strong>PLA</strong> printing?</p><ul><li>Temperature settings</li><li>Bed adhesion</li></ul>","OwnerUserId":"2","Title":"Best PLA printing settings?","Tags":"<pla><settings><temperature>","AnswerCount":"3","CommentCount":"2","@tail":"\n  ","@index":1}
{"":"row","-":"posts","Id":"3","PostTypeId":"2","ParentId":"1","CreationDate":"2016-01-12T19:30:00.000","Score":"15","Body":"<p>The best method is to use a piece of paper:</p><ol><li>Heat the bed to operating temperature</li><li>Move the nozzle to each corner</li><li>Adjust until you feel slight resistance</li></ol>","OwnerUserId":"3","@tail":"\n  ","@index":2}
{"":"row","-":"posts","Id":"4","PostTypeId":"2","ParentId":"1","CreationDate":"2016-01-12T20:00:00.000","Score":"8","Body":"<p>Consider using a <em>BLTouch</em> or similar auto-leveling probe for consistent results.</p>","OwnerUserId":"4","@tail":"\n  ","@index":3}
{"":"row","-":"posts","Id":"5","PostTypeId":"1","CreationDate":"2016-01-13T10:00:00.000","Score":"12","ViewCount":"2500","Body":"<p>My prints keep warping at the corners. What causes this and how can I fix it?</p><p>I am using ABS filament at 230C with a heated bed at 100C.</p>","OwnerUserId":"5","Title":"Why do my ABS prints warp?","Tags":"<abs><warping><troubleshooting>","AnswerCount":"4","CommentCount":"5","@tail":"\n  ","@index":4}
{"":"row","-":"posts","Id":"6","PostTypeId":"2","ParentId":"5","CreationDate":"2016-01-13T11:00:00.000","Score":"20","Body":"<p>Warping is caused by uneven cooling. Try these solutions:</p><ul><li>Use an enclosure</li><li>Apply ABS slurry to the bed</li><li>Increase bed temperature to 110C</li></ul>","OwnerUserId":"3","@tail":"\n  ","@index":5}
{"":"row","-":"posts","Id":"7","PostTypeId":"1","CreationDate":"2016-01-14T08:00:00.000","Score":"7","ViewCount":"800","Body":"<p>What is the difference between <code>Cura</code> and <code>PrusaSlicer</code>?</p>","OwnerUserId":"6","Title":"Cura vs PrusaSlicer comparison","Tags":"<slicer><software><cura><prusaslicer>","AnswerCount":"2","CommentCount":"3","@tail":"\n  ","@index":6}
{"":"row","-":"posts","Id":"8","PostTypeId":"2","ParentId":"7","CreationDate":"2016-01-14T09:00:00.000","Score":"10","Body":"<p>Both are excellent slicers. Key differences:</p><table><tr><th>Feature</th><th>Cura</th><th>PrusaSlicer</th></tr><tr><td>UI</td><td>Beginner-friendly</td><td>More technical</td></tr></table>","OwnerUserId":"7","@tail":"\n  ","@index":7}
{"":"row","-":"posts","Id":"9","PostTypeId":"1","CreationDate":"2016-01-15T12:00:00.000","Score":"3","ViewCount":"300","Body":"<p>Can I print with <strong>PETG</strong> on a non-heated bed?</p>","OwnerUserId":"8","Title":"PETG without heated bed?","Tags":"<petg><bed>","AnswerCount":"1","CommentCount":"0","@tail":"\n  ","@index":8}
{"":"row","-":"posts","Id":"10","PostTypeId":"2","ParentId":"9","CreationDate":"2016-01-15T13:00:00.000","Score":"5","Body":"<p>PETG <em>technically</em> can be printed without a heated bed, but adhesion will be poor. Recommended bed temp: 70-80C.</p>","OwnerUserId":"3","@tail":"\n","@index":9}
