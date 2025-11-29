#!/bin/bash
# Run clippy linter
set -euo pipefail

cargo clippy -- -D warnings
