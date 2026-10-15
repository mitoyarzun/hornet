#!/bin/sh
# Make the launcher icons from the 16 and 32 pixel icons of the free
# F/A-18 Hornet 2.0 demo (packaging/icon/). Larger sizes are scaled up with
# nearest-neighbor sampling, so the pixels stay sharp.
#
# Needs macOS (iconutil), ffmpeg and python3. Writes:
#   packaging/macos/AppIcon.icns
#   packaging/windows/hornet.ico
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
icon="$here/packaging/icon"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

scale() { # SIZE OUTPUT
    if [ "$1" -le 16 ]; then src="$icon/hornet-16.png"; else src="$icon/hornet-32.png"; fi
    ffmpeg -loglevel error -y -i "$src" -vf "scale=$1:$1:flags=neighbor" "$2"
}

set_dir="$work/AppIcon.iconset"
mkdir -p "$set_dir"
for size in 16 32 128 256 512; do
    scale "$size" "$set_dir/icon_${size}x${size}.png"
    scale $((size * 2)) "$set_dir/icon_${size}x${size}@2x.png"
done
iconutil -c icns -o "$here/packaging/macos/AppIcon.icns" "$set_dir"

for size in 16 32 48 256; do
    scale "$size" "$work/ico-$size.png"
done
# An ICO file can hold PNG images (Windows Vista and later).
python3 -I - "$here/packaging/windows/hornet.ico" "$work"/ico-16.png \
    "$work"/ico-32.png "$work"/ico-48.png "$work"/ico-256.png <<'EOF'
import struct, sys
out, paths = sys.argv[1], sys.argv[2:]
images = [open(p, "rb").read() for p in paths]
sizes = [16, 32, 48, 256]
header = struct.pack("<HHH", 0, 1, len(images))
offset = 6 + 16 * len(images)
entries = b""
for size, data in zip(sizes, images):
    side = 0 if size == 256 else size
    entries += struct.pack("<BBBBHHII", side, side, 0, 0, 1, 32, len(data), offset)
    offset += len(data)
open(out, "wb").write(header + entries + b"".join(images))
EOF
echo "wrote packaging/macos/AppIcon.icns and packaging/windows/hornet.ico"
