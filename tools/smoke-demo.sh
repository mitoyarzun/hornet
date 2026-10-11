#!/bin/sh
# Download Graphsim's freely redistributable Hornet 2.0 demo, verify it, and
# fly it headlessly: title, briefing, cockpit, engine start, takeoff roll and
# external view. Screenshots and report.json land in out/smoke/.
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
bin="$here/out/systemless/target/${PROFILE:-release}/systemless"
url=https://classicmacdemos.com/download/fa-18-hornet-20/
sha=86ccdaf2b83c84804b2bdbbac0db7d445bff106d67f8f5515f9a38d21bbb2f85
demo="$here/out/FA-18-2.0-Demo-68k.sit"
run="$here/out/smoke"

[ -x "$bin" ] || { echo "no build at $bin; run tools/setup-systemless.sh first" >&2; exit 1; }

mkdir -p "$here/out"
[ -f "$demo" ] || curl -fsSL -o "$demo" "$url"

if command -v sha256sum >/dev/null 2>&1; then
    got=$(sha256sum "$demo" | cut -d' ' -f1)
else
    got=$(shasum -a 256 "$demo" | cut -d' ' -f1)
fi
[ "$got" = "$sha" ] || { echo "checksum mismatch for $demo: $got" >&2; exit 1; }

rm -rf "$run"
if SYSTEMLESS_LOAD_EXECUTABLE="Hornet" "$bin" "$demo" --headless --prefer-68k \
    --play-script "$here/play/fly-demo.json" --play-output "$run" \
    > "$here/out/smoke.log" 2>&1; then
    echo "passed: screenshots in $run"
else
    echo "failed: see $here/out/smoke.log and $run/failure.png" >&2
    exit 1
fi
