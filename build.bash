#!/bin/bash
# expects to run through `yarn build`.
set -vexuo pipefail;

replace-in-files --regex='"version"\s*:\s*"(.*?)"' --replacement='"version": "'"$(date "+%Y.%-m.%-d")"'"' ./package.json ./src/manifest.json;

rm -rf ./docs/*;
cp -r ./src/* ./data/* ./static/* ./package.json ./docs/;
cp ./src/index.html ./docs/.htm;
sort-json ./*.json ./*/*.json;
prettier --write src *.json */*.json;
tsc || (echo \"type errors! alas.\" && sleep 2);
prettier --write docs;
