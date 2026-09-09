#!/bin/bash
set -euo pipefail
source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/base-test.sh"
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/home/.config/omarchy/plugins" "$fixture/bin" "$fixture/source"
git -C "$fixture/source" init -qb main
git -C "$fixture/source" config user.name Test
git -C "$fixture/source" config user.email test@example.invalid
printf 'valid\n' > "$fixture/source/Panel.qml"
git -C "$fixture/source" add .
git -C "$fixture/source" commit -qm initial
initial=$(git -C "$fixture/source" rev-parse HEAD)
plugins="$fixture/home/.config/omarchy/plugins"
for id in direct managed; do git clone -q "$fixture/source" "$plugins/$id"; done
git clone -q "$fixture/source" "$fixture/live"
ln -s "$fixture/live" "$plugins/linked"
mkdir -p "$fixture/home/.local/state/omarchy/plugin-workbench/marketplace/receipts"
printf '{}\n' > "$fixture/home/.local/state/omarchy/plugin-workbench/marketplace/receipts/managed.json"
cat > "$fixture/bin/omarchy-plugin-validate" <<'STUB'
#!/bin/bash
! grep -q invalid "$1/Panel.qml"
STUB
printf '#!/bin/bash\nexit 0\n' > "$fixture/bin/omarchy-shell"
chmod +x "$fixture/bin/"*
printf 'invalid\n' > "$fixture/source/Panel.qml"
git -C "$fixture/source" commit -qam invalid
if HOME="$fixture/home" XDG_STATE_HOME="$fixture/home/.local/state" PATH="$fixture/bin:$PATH" "$ROOT/bin/omarchy-plugin-update" --yes; then
  fail "invalid staged update was accepted"
fi
for id in direct managed linked; do
  [[ $(git -C "$plugins/$id" rev-parse HEAD) == "$initial" ]] || fail "$id checkout changed"
done
pass "invalid updates never enter installed tree; managed and linked checkouts stay untouched"

printf 'valid again\n' > "$fixture/source/Panel.qml"
git -C "$fixture/source" commit -qam valid
reviewed=$(git -C "$fixture/source" rev-parse HEAD)
HOME="$fixture/home" XDG_STATE_HOME="$fixture/home/.local/state" PATH="$fixture/bin:$PATH" "$ROOT/bin/omarchy-plugin-update" --yes
[[ $(git -C "$plugins/direct" rev-parse HEAD) == "$reviewed" ]] || fail "valid candidate not activated"
for id in managed linked; do
  [[ $(git -C "$plugins/$id" rev-parse HEAD) == "$initial" ]] || fail "$id checkout changed"
done
pass "valid direct update activates while both external ownership boundaries remain intact"
