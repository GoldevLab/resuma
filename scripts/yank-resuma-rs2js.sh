#!/usr/bin/env bash
# Retire the legacy resuma-rs2js crate on crates.io (code lives in resuma-macros/src/rs2js/).
# Requires: cargo login (or CRATES_IO_TOKEN in the environment).
set -euo pipefail

for ver in 0.0.0 0.1.0; do
  if cargo yank --vers "$ver" resuma-rs2js 2>/dev/null; then
    echo "yanked resuma-rs2js $ver"
  else
    echo "skip resuma-rs2js $ver (not published or already yanked)"
  fi
done

echo "Done. New projects should use resuma + resuma-macros only."
