#!/bin/sh
set -eu

# Production code must name its bounds/counts. clap defaults, take/skip/truncate
# caps, and explicit numeric ranges belong in util/defaults.rs as constants.
PATTERN='default_value_t[[:space:]]*=[[:space:]]*[0-9]|default_value[[:space:]]*=[[:space:]]*"[0-9]|\.(take|skip|truncate|nth)\([[:space:]]*[0-9]+\)|[0-9]{3}\.\.[0-9]{3}'

matches=$(
  find src -name '*.rs' ! -name tests.rs -type f -exec grep -En "$PATTERN" {} + || true
)

if [ -n "$matches" ]; then
  echo "magic literal violation: move production literals into named constants (util/defaults.rs)" >&2
  echo "$matches" >&2
  exit 1
fi
