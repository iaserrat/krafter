#!/bin/sh
set -eu

MAX_LOC=100
find src tests -name '*.rs' -type f -exec wc -l {} + |
  awk -v max="$MAX_LOC" '
    $2 != "total" && $1 > max {
      printf "LOC violation: %s has %s lines (max %s)\n", $2, $1, max > "/dev/stderr"
      failed = 1
    }
    END { exit failed }
  '
