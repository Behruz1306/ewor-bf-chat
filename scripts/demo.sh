#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release --quiet
cargo run --release --bin bf-gen --quiet

echo "open two terminals:"
echo "  make run-server"
echo "  make run-client"
