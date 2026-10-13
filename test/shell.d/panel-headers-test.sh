#!/bin/bash
source "$(dirname "$0")/base-test.sh"

require_command quickshell
require_command timeout
test_dir=$(mktemp -d)
trap 'rm -rf "$test_dir"' EXIT
mkdir -p "$test_dir/config" "$test_dir/runtime"
chmod 700 "$test_dir/runtime"
ln -s "$ROOT/shell/Commons" "$test_dir/config/Commons"
ln -s "$ROOT/shell/Ui" "$test_dir/config/Ui"
cp "$SHELL_TEST_DIR/fixtures/panel-headers/shell.qml" "$test_dir/config/shell.qml"

# The actual Quickshell engine loads the actual surface and palette code,
# isolated from the desktop and its IPC sockets. No live shell is restarted.
if env -u DISPLAY -u WAYLAND_DISPLAY XDG_RUNTIME_DIR="$test_dir/runtime" \
  QT_QPA_PLATFORM=offscreen QT_QPA_PLATFORMTHEME=generic \
  QT_QUICK_CONTROLS_STYLE=Basic QT_QUICK_BACKEND=software \
  PANEL_HEADER_SCREENSHOT="$test_dir/comparison.png" \
  timeout 20s quickshell -p "$test_dir/config" >"$test_dir/log" 2>&1; then
  if ! rg -q 'PANEL_HEADER_TESTS_PASSED=12' "$test_dir/log" || rg -q 'FAIL:|Failed to load configuration' "$test_dir/log"; then
    fail "joined panel header runtime checks" "$(cat "$test_dir/log")"
  fi
else
  fail "joined panel header runtime checks" "$(cat "$test_dir/log")"
fi
[[ -s $test_dir/comparison.png ]] || fail "joined panel header renders"
pass "joined panel header opt-in, reset, overrides, geometry and rendering"
