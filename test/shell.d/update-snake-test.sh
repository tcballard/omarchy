#!/bin/bash

set -euo pipefail
source "$(dirname "$0")/base-test.sh"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The drawing must complete before the update command starts writing to the
# terminal. A subsequent prompt or pacman line has the same TTY, not a pipe.
script -qefc "stty rows 30 columns 80; TERM=xterm-256color '$ROOT/bin/omarchy-update-snake' 4 'Updating system packages'; printf 'pacman output\\n'" "$tmp/terminal" >"$tmp/screen"
grep -q 'Updating system packages' "$tmp/screen" || fail "stage label is missing"
grep -q 'pacman output' "$tmp/screen" || fail "command output is missing after the animation"
grep -q '▀' "$tmp/screen" || fail "animation frames are missing"

TERM=xterm-256color "$ROOT/bin/omarchy-update-snake" 4 'Updating system packages' >"$tmp/pipe"
[[ ! -s $tmp/pipe ]] || fail "non-terminal output includes animation"

pass "terminal stages show artwork without swallowing the following command output"
