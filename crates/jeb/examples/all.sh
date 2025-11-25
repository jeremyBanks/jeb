#!/bin/bash
set -euo pipefail
path="$(realpath "$0")"
pushd "$(dirname "$path")"

for example in ./*.sh; do
    if [ "$example" != "./all.sh" ]; then
        echo "$example:"
        "$example"
        echo
    fi
done
