#!/bin/sh
# Package a patched Systemless build and the launcher into a release archive
# in dist/. First run tools/setup-systemless.sh with the same TARGET, and
# build launcher/ with --target TARGET.
#
#   tools/package-release.sh TARGET VERSION
#   tools/package-release.sh source VERSION     # patched Systemless source
set -eu

[ $# -eq 2 ] || { echo "usage: $0 TARGET|source VERSION" >&2; exit 2; }
target=$1
version=$2
here=$(cd "$(dirname "$0")/.." && pwd)
src="$here/out/systemless"
launcher="$here/launcher/target/$target/release"
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
cp "$here/LICENSE" "$stage/LICENSE.txt"
cp "$here/packaging/README.txt" "$stage/README.txt"
git -C "$src" rev-parse HEAD > "$stage/SYSTEMLESS-COMMIT.txt"

case "$target" in
    *windows*)
        cp "$launcher/hornet-launcher.exe" "$stage/F-A-18 Hornet.exe"
        cp "$src/target/$target/release/systemless.exe" "$stage/"
        (cd "$here/out/stage" && 7z a -tzip "$dist/$name.zip" "$name" >/dev/null)
        echo "$dist/$name.zip"
        ;;
    *apple*)
        app="$stage/F-A-18 Hornet.app"
        mkdir -p "$app/Contents/MacOS"
        sed "s/VERSION/${version#v}/g" "$here/packaging/macos/Info.plist" \
            > "$app/Contents/Info.plist"
        cp "$launcher/hornet-launcher" "$src/target/$target/release/systemless" \
            "$app/Contents/MacOS/"
        # An ad-hoc signature lets macOS offer "Open Anyway" for the bundle.
        codesign --force --deep --sign - "$app"
        ditto -c -k --norsrc --noextattr --keepParent "$stage" "$dist/$name.zip"
        echo "$dist/$name.zip"
        ;;
    *)
        echo "unsupported target: $target" >&2
        exit 2
        ;;
esac
