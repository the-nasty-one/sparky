#!/usr/bin/env bash
set -euo pipefail

SPARK="auxidus-spark@192.168.5.118"
BINARY="target/aarch64-unknown-linux-gnu/release/spark-console"

echo "building for aarch64..."
cargo leptos build --release --target aarch64-unknown-linux-gnu

echo "deploying..."
scp "$BINARY" "$SPARK:/tmp/spark-console"
ssh "$SPARK" "sudo mv /tmp/spark-console /usr/local/bin/spark-console && sudo systemctl restart spark-console"

echo "done."
