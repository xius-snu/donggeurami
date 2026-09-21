#!/usr/bin/env bash
#
# Boots a simulator, installs the app, lets it render, and captures a PNG at
# exactly the sizes App Store Connect asks for.
#
# Two jobs in one. It is a smoke test — if the game panics on launch, the log
# it leaves behind says so, and that is the iOS equivalent of the
# `adb logcat -d | grep RustStdoutStderr` loop in RENDERING.md. And it produces
# the store screenshots, which otherwise need a physical 6.9" iPhone and a 13"
# iPad sitting on the desk.
#
# It is NOT a substitute for running on a real device. The simulator's Metal
# is not the phone's Metal, and RENDERING.md is a long argument for trusting
# hardware over inference. Treat a clean run here as "it starts", not "it
# renders correctly".
#
# Run from the repo root, after building for iphonesimulator.

set -euo pipefail

BUNDLE_ID=town.donggeurami.app
OUT=screenshots
DWELL=${DWELL:-25}   # seconds to let Bevy compile shaders and load the glTFs

APP="$(find build/sim/Build/Products -maxdepth 2 -name 'RoundTown.app' | head -1)"
if [ -z "$APP" ]; then
    echo "error: no RoundTown.app under build/sim — build for the simulator first" >&2
    exit 1
fi
echo "app: $APP"
mkdir -p "$OUT"

# Newest installed iOS runtime, whatever the image happens to ship.
RUNTIME="$(xcrun simctl list runtimes --json | python3 -c '
import json, sys
rs = [r for r in json.load(sys.stdin)["runtimes"]
      if r.get("isAvailable") and "iOS" in r.get("name", "")]
rs.sort(key=lambda r: [int(p) for p in r["version"].split(".")])
print(rs[-1]["identifier"] if rs else "")
')"
[ -n "$RUNTIME" ] || { echo "error: no iOS simulator runtime available" >&2; exit 1; }
echo "runtime: $RUNTIME"

device_type_for() {
    xcrun simctl list devicetypes --json | python3 -c '
import json, sys
want = sys.argv[1]
hits = [d for d in json.load(sys.stdin)["devicetypes"] if want in d.get("name", "")]
print(hits[0]["identifier"] if hits else "")
' "$1"
}

# $1 label  $2 device-type name to match  $3 rotation applied to the capture
shoot() {
    local label="$1" want="$2" rotate="$3"
    local dt udid pid

    dt="$(device_type_for "$want")"
    if [ -z "$dt" ]; then
        echo "!! no simulator device type matching '$want' — skipping $label" >&2
        echo "   available:" >&2
        xcrun simctl list devicetypes | sed -n 's/^\(iP[^(]*\).*/     \1/p' >&2
        return 0
    fi

    udid="$(xcrun simctl create "shot-$label" "$dt" "$RUNTIME")"
    xcrun simctl boot "$udid"
    xcrun simctl bootstatus "$udid" -b
    xcrun simctl install "$udid" "$APP"

    # --console attaches to the app's stdout/stderr, which is where Bevy's log
    # and any Rust panic come out. It blocks, so it goes in the background.
    xcrun simctl launch --console "$udid" "$BUNDLE_ID" > "$OUT/$label.log" 2>&1 &
    pid=$!
    sleep "$DWELL"

    # simctl captures the framebuffer in the device's native portrait
    # orientation regardless of how the UI is rotated. The game is
    # landscape-only, so rotate the capture back to get the landscape
    # dimensions the store expects.
    xcrun simctl io "$udid" screenshot "$OUT/$label-raw.png"
    sips -r "$rotate" "$OUT/$label-raw.png" --out "$OUT/$label.png" >/dev/null
    rm -f "$OUT/$label-raw.png"

    echo "--- $label ---"
    sips -g pixelWidth -g pixelHeight "$OUT/$label.png" | sed 's/^/    /'
    echo "    log tail:"
    tail -n 15 "$OUT/$label.log" 2>/dev/null | sed 's/^/      /' || true

    kill "$pid" 2>/dev/null || true
    xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
    xcrun simctl delete "$udid"   >/dev/null 2>&1 || true
}

# App Store Connect now only wants the largest device in each family and scales
# the rest itself: 6.9" iPhone (2868x1320 landscape) and 13" iPad (2752x2064).
shoot "iphone-6.9" "iPhone 17 Pro Max" 90
shoot "ipad-13"    "iPad Pro 13-inch"  90

echo
echo "screenshots in ./$OUT — check the rotation direction before uploading;"
echo "if the world is upside down, change 90 to 270 in the shoot lines above."
