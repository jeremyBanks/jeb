#!/bin/bash
# Check code formatting
set -euo pipefail

cargo fmt -- --check
