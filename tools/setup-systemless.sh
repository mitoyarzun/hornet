#!/bin/sh
# Fetch Systemless at the tested commit, apply the patches in patches/, and
# build it. The checkout lives in out/systemless and is reset on every run.
#
#   tools/setup-systemless.sh            # release build (slow, fastest to run)
#   PROFILE=ci-test tools/setup-systemless.sh
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
commit=38666a8ebac3bdcfafd4bea1809c17b78157afc4
dest="$here/out/systemless"
profile="${PROFILE:-release}"

command -v cargo >/dev/null 2>&1 || {
    echo "cargo not found. Install Rust from https://rustup.rs and try again." >&2
    exit 1
}

if [ ! -d "$dest/.git" ]; then
    mkdir -p "$here/out"
    git clone https://github.com/benletchford/systemless.git "$dest"
fi

cd "$dest"
git fetch --quiet origin
git checkout --quiet --force "$commit"
git reset --quiet --hard
for patch in "$here"/patches/*.patch; do
    git apply "$patch"
    echo "applied $(basename "$patch")"
done

cargo build --locked --profile "$profile" --bin systemless
echo "built $dest/target/$profile/systemless"
