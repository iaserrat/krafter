#!/bin/sh
set -eu

tuple_fields=$(
  find src -name '*.rs' ! -name tests.rs -type f -exec grep -En '\.[0-9]\b' {} + |
    grep -v 'const ' |
    grep -v '"' || true
)

tuple_signatures=$(
  find src -name '*.rs' ! -name tests.rs -type f -exec grep -En 'type .* = \(|-> \([^)]*,[^)]*\)' {} + || true
)

if [ -n "$tuple_fields$tuple_signatures" ]; then
  echo "positional contract violation: use named structs instead of tuple fields/signatures" >&2
  [ -z "$tuple_fields" ] || echo "$tuple_fields" >&2
  [ -z "$tuple_signatures" ] || echo "$tuple_signatures" >&2
  exit 1
fi
