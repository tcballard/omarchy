#!/bin/bash
set -euo pipefail
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/base-test.sh"
require_command python3
require_command inotifywait
python3 "$SHELL_TEST_DIR/fixtures/plugin-watch/test.py" "$ROOT/bin/omarchy-plugin-watch"
