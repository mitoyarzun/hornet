#!/bin/sh
# Launch F/A-18 Hornet 2.0 in a window with the patched Systemless build.
#
#   tools/run-hornet.sh ARCHIVE [EXECUTABLE]
#
# ARCHIVE is a StuffIt archive, MacBinary file, or HFS disk image.
# EXECUTABLE is a substring of the application's name; it defaults to
# "Hornet" so the bundled docs reader is not launched instead.
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
bin="$here/out/systemless/target/${PROFILE:-release}/systemless"

[ $# -ge 1 ] || { echo "usage: $0 ARCHIVE [EXECUTABLE]" >&2; exit 2; }
[ -x "$bin" ] || { echo "no build at $bin; run tools/setup-systemless.sh first" >&2; exit 1; }

# The 68K slice is the one that runs; the PowerPC slice still stops on
# unimplemented InterfaceLib imports (see README).
SYSTEMLESS_LOAD_EXECUTABLE="${2:-Hornet}" exec "$bin" --prefer-68k "$1"
