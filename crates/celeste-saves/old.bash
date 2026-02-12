#!/bin/bash
set -euo pipefail;

-q() {
    "$@" 1> /dev/null
}

-qq() {
    "$@" 1> /dev/null 2> /dev/null
}

cd ~/Library/"Application Support"/Celeste/Saves &> /dev/null || \
cd /mnt/d/Program\ Files/Celeste/Saves &> /dev/null || \
cd ~/.local/share/Celeste/Saves &> /dev/null; 

user="$(whoami)@$(cat /etc/hostname || hostname || echo "unknown")";

-qq git add ./*.celeste;

-qq git commit -m "⚠ $user old staged" --allow-empty-message || true;

-qq git add .;
-qq git commit . -m "⚠ $user old unstaged" --allow-empty-message || true;

-qq git fetch;

-qq git pull --ff-only || (
    echo "⚠ Sync conflict. Archiving remote data and replacing with local." &&
    git pull -s ours --no-edit);

-qq git push;

echo "✅ Synced"

echo "🍓 Celeste";

set -vx

test "ON" != "${CELESTE:-ON}" ||\
-qq open -W ~/Library/"Application Support"/itch/apps/celeste/Celeste.app || \
-qq open -W /Applications/Celeste.app || \
-qq /mnt/d/Program\ Files/Celeste/Celeste.exe || \
steam steam://rungameid/504230;

set -vx
pid="$(
    (sleep 8 && (
        pidof pv-bwrap ||
        pidof gameoverlayui ||
        false
    )) || (sleep 8 && (
        pidof pv-bwrap ||
        pidof gameoverlayui ||
        pidof steam ||
        echo ""
    ))
)";

tail --pid=$pid -f /dev/null || true;

cargo build 2> /dev/null || cargo build;

cat template.html > index.html; for n in 0 1 2; do 
    rm -f ${n}.txt || true; ~/.cargo/build/debug/celeste-saves ${n}.celeste 1> ${n}.txt 2> /dev/null || rm -f ${n}.txt;
    cat template.html > ${n}.html
    ( \
        echo -n "<pre id="${n}" style='position: relative;'>"; \
        echo -n '<div style="position: absolute; top: 0; right: 0;"><a href="'./${n}.celeste'">'${n}'.celeste</a> </div>'
        CELESTE_SAVE_COLOR=HTML ~/.cargo/build/debug/celeste-saves ${n}.celeste | head -n -1; \
        echo -n "</pre>" \
    ) | tee --append ${n}.html 1>> index.html 2> /dev/null;
done

-qq git add .;
if -qq git commit . -m "🍓 $user" --allow-empty-message; then
    -qq git push && echo "✅ Synced" || echo "⚠ Sync failed";
    set +e;
    diff="$(git diff -U5 --ws-error-highlight=none HEAD~1..HEAD 0.txt | tail -n +6 | egrep '^[\+\-]' -B 5 | grep -v '@@')";
    set -e;
    echo "$diff";
else
    echo "🆗 No changes to sync"
fi
set +vx

echo;
~/.cargo/build/debug/celeste-saves 0.celeste;
