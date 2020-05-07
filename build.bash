#!/bin/bash
# expects to run through `yarn build`.
set -vexuo pipefail;

replace-in-files --regex='"version"\s*:\s*"(.*?)"' --replacement='"version": "'"$(date --utc "+%Y.%-m.%-d")"'"' ./package.json ./src/manifest.json;
replace-in-files --regex='"node"\s*:\s*"(.*?)"' --replacement='"node": "'"$(node --version)"'"' ./package.json;

rm -rf ./docs/*;
cp -r ./src/* ./data/* ./static/* ./docs/;
cp ./src/index.html ./docs/.htm;
sort-json ./*.json ./*/*.json;
prettier --write src *.json */*.json;
tsc || (echo \"type errors! alas.\" && sleep 2);
prettier --write docs;
