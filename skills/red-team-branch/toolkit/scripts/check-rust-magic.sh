#!/bin/sh
set -eu

PATTERN='send_once\(.*,[[:space:]]*0\)|default_value_t[[:space:]]*=[[:space:]]*[0-9]|default_value[[:space:]]*=[[:space:]]*"(GET|POST|query|auto|127|GET,POST)|\.take\([[:space:]]*[0-9]+\)|[0-9]{3}\.\.[0-9]{3}|rng\.below\([[:space:]]*[0-9]+\)'

matches=$(
  find src -name '*.rs' ! -name tests.rs -type f -exec grep -En "$PATTERN" {} + || true
)

if [ -n "$matches" ]; then
  echo "magic literal violation: move production literals into named constants" >&2
  echo "$matches" >&2
  exit 1
fi
