#!/bin/sh
# Package a patched Systemless build into a release archive in dist/.
# Run tools/setup-systemless.sh with the same TARGET first.
#
#   tools/package-release.sh TARGET VERSION
#   tools/package-release.sh source VERSION     # patched Systemless source
set -eu

[ $# -eq 2 ] || { echo "usage: $0 TARGET|source VERSION" >&2; exit 2; }
target=$1
version=$2
here=$(cd "$(dirname "$0")/.." && pwd)
src="$here/out/systemless"
dist="$here/dist"
mkdir -p "$dist"

if [ "$target" = source ]; then
    # GPL-3.0 corresponding source: Systemless at the pinned commit with the
    # patches applied, without build output or git metadata.
    name="fa18-hornet-mac-$version-systemless-source"
    tar -czf "$dist/$name.tar.gz" -C "$here/out" \
        --exclude systemless/target --exclude systemless/.git systemless
    echo "$dist/$name.tar.gz"
    exit 0
fi

name="fa18-hornet-mac-$version-$target"
stage="$here/out/stage/$name"
rm -rf "$stage"
mkdir -p "$stage"

case "$target" in
    *windows*)
        cp "$src/target/$target/release/systemless.exe" "$stage/"
        cp "$here/packaging/run-hornet.cmd" "$stage/"
        ;;
    *)
        cp "$src/target/$target/release/systemless" "$stage/"
        cp "$here/tools/run-hornet.sh" "$stage/"
        chmod +x "$stage/systemless" "$stage/run-hornet.sh"
        ;;
esac
cp "$here/LICENSE" "$stage/LICENSE.txt"
cp "$here/packaging/README.txt" "$stage/README.txt"
git -C "$src" rev-parse HEAD > "$stage/SYSTEMLESS-COMMIT.txt"

cd "$here/out/stage"
case "$target" in
    *windows*)
        if command -v 7z >/dev/null 2>&1; then
            7z a -tzip "$dist/$name.zip" "$name" >/dev/null
        else
            powershell -NoProfile -Command \
                "Compress-Archive -Path '$name' -DestinationPath '$(cygpath -w "$dist/$name.zip" 2>/dev/null || echo "$dist/$name.zip")'"
        fi
        echo "$dist/$name.zip"
        ;;
    *)
        tar -czf "$dist/$name.tar.gz" "$name"
        echo "$dist/$name.tar.gz"
        ;;
esac
