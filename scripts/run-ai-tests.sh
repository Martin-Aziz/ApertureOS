#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -x .venv/bin/python ]]; then
  .venv/bin/python -m pip install -e "./ai-service[dev]"
  .venv/bin/pytest ai-service/tests
else
  python3 -m pip install -e "./ai-service[dev]"
  python3 -m pytest ai-service/tests
fi
