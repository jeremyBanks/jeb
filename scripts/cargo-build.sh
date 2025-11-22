#!/bin/bash
export SOURCE_DATE_EPOCH=1608040201
export TZ=UTC
export LC_ALL=C
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$PWD=."
exec cargo build "$@"
