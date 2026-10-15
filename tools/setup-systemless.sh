#!/bin/sh
# Fetch Systemless at the tested commit, apply the patches in patches/, and
# build it. The checkout lives in out/systemless and is reset on every run.
#
#   tools/setup-systemless.sh            # release build (slow, fastest to run)
#   PROFILE=ci-test tools/setup-systemless.sh
#   TARGET=x86_64-apple-darwin tools/setup-systemless.sh
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
commit=38666a8ebac3bdcfafd4bea1809c17b78157afc4
dest="$here/out/systemless"
profile="${PROFILE:-release}"
target="${TARGET:-}"

command -v cargo >/dev/null 2>&1 || {
    echo "cargo not found. Install Rust from https://rustup.rs and try again." >&2
    exit 1
}

if [ ! -d "$dest/.git" ]; then
    mkdir -p "$here/out"
    # Keep LF line endings on Windows, or the patches do not apply.
    git clone -c core.autocrlf=false https://github.com/benletchford/systemless.git "$dest"
fi

cd "$dest"
git fetch --quiet origin
git checkout --quiet --force "$commit"
git reset --quiet --hard
for patch in "$here"/patches/*.patch; do
    git apply "$patch"
    echo "applied $(basename "$patch")"
done

if [ -n "$target" ]; then
    cargo build --locked --profile "$profile" --target "$target" --bin systemless
    echo "built $dest/target/$target/$profile/systemless"
else
    cargo build --locked --profile "$profile" --bin systemless
    echo "built $dest/target/$profile/systemless"
fi
