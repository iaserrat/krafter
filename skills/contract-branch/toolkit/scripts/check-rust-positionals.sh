#!/bin/sh
set -eu

# Tuple field access (foo.0 / bar.1) reads as an unnamed positional contract;
# return named structs instead. The leading [)A-Za-z_] excludes float literals
# like 0.0 / 1.0 (digit before the dot), which are not tuple access.
tuple_fields=$(
  find src -name '*.rs' ! -name tests.rs -type f -exec grep -En '[)A-Za-z_]\.[0-9]+\b' {} + || true
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
