#!/bin/sh
# Build the web version of the game into dist/web. The page runs the
# Systemless browser frontend from www/ in the Systemless checkout, with the
# Hornet start page from patches/0010. The player picks a game file, and the
# page keeps it in IndexedDB for the next visits.
#
#   tools/build-web.sh                         # for https://USER.github.io/hornet/
#   PUBLIC_URL=/ tools/build-web.sh            # for a site at the domain root
#   SKIP_SETUP=1 tools/build-web.sh            # use out/systemless as it is
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
public_url="${PUBLIC_URL:-/hornet/}"
dist="${DIST:-$here/dist/web}"
trunk_version=0.21.14

if [ -z "${SKIP_SETUP:-}" ]; then
    SKIP_BUILD=1 "$here/tools/setup-systemless.sh"
fi

rustup target add wasm32-unknown-unknown >/dev/null
if [ "$(trunk --version 2>/dev/null)" != "trunk $trunk_version" ]; then
    cargo install trunk --locked --version "$trunk_version"
fi

# Trunk 0.21 rejects common NO_COLOR values like "1" for release builds.
NO_COLOR=false
# RUSTFLAGS overrides .cargo/config.toml, so keep the worker stack size here.
RUSTFLAGS="-C target-feature=+simd128 -C link-arg=-zstack-size=4194304"
export NO_COLOR RUSTFLAGS

cd "$here/out/systemless/www"
trunk build --release --locked --public-url "$public_url" --dist "$dist"
echo "built $dist"
