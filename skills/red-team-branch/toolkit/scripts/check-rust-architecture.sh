#!/bin/sh
set -eu

failed=0

for file in src/cmd/*.rs; do
  [ -e "$file" ] || continue
  [ "$file" = "src/cmd/mod.rs" ] && continue
  echo "architecture violation: command must be a module directory, not $file" >&2
  failed=1
done

for file in src/config.rs src/http.rs src/util.rs; do
  [ ! -e "$file" ] && continue
  echo "architecture violation: shared module must be split into a directory, not $file" >&2
  failed=1
done

for dir in src/cmd/*; do
  [ -d "$dir" ] || continue
  [ -f "$dir/mod.rs" ] && continue
  echo "architecture violation: command directory $dir is missing mod.rs" >&2
  failed=1
done

exit "$failed"
