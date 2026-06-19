#!/bin/sh
set -eu

failed=0

# Commands must be module directories, not single files.
for file in src/cmd/*.rs; do
  [ -e "$file" ] || continue
  [ "$file" = "src/cmd/mod.rs" ] && continue
  echo "architecture violation: command must be a module directory, not $file" >&2
  failed=1
done

# Shared concerns must be directories, never flat god files.
for file in src/config.rs src/util.rs src/engine.rs src/git.rs; do
  [ ! -e "$file" ] && continue
  echo "architecture violation: shared module must be a directory, not $file" >&2
  failed=1
done

# Every command directory needs a mod.rs.
for dir in src/cmd/*; do
  [ -d "$dir" ] || continue
  [ -f "$dir/mod.rs" ] && continue
  echo "architecture violation: command directory $dir is missing mod.rs" >&2
  failed=1
done

exit "$failed"
