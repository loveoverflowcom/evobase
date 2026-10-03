#!/usr/bin/env bash
set -euo pipefail
repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_dir"
python3 apps/evobase-builder/generate_tokens.py --check
command -v wasm-bindgen >/dev/null || {
  echo "Install wasm-bindgen-cli matching Cargo.lock, then rerun this script." >&2
  exit 1
}
cargo build --locked -p evobase-builder --target wasm32-unknown-unknown
dist_dir="$repo_dir/apps/evobase-builder/dist"
mkdir -p "$dist_dir"
wasm-bindgen target/wasm32-unknown-unknown/debug/evobase_builder.wasm --target web --out-dir "$dist_dir" --no-typescript
cp apps/evobase-builder/index.html apps/evobase-builder/builder.css apps/evobase-builder/tokens.css "$dist_dir/"
cp design/m3-expressive/assets/OpenSans-Regular.ttf design/m3-expressive/assets/OpenSans-Semibold.ttf design/m3-expressive/assets/LICENSE-Apache-2.0.txt design/m3-expressive/assets/OpenSans-NOTICE.txt "$dist_dir/"
echo "Built local Builder. Serve apps/evobase-builder/dist with any static HTTP server."
