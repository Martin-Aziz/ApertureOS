#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

pnpm install --no-frozen-lockfile
pnpm --filter @pixelforge/common typecheck
pnpm --filter pixelforge-frontend typecheck
pnpm --filter pixelforge-frontend test

cargo test --manifest-path backend/Cargo.toml
cargo test --manifest-path wasm/Cargo.toml

./scripts/run-ai-tests.sh
