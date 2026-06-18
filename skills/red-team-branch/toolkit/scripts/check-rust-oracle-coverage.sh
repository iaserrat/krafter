#!/bin/sh
# Oracle-coverage gate. Every detection probe must prove its oracle CLEARS a
# safe input (a negative control), not just that it flags a vulnerable one.
# The absence of that safe-clear test is exactly how five probes once shipped
# green while silently crying wolf. Scans each probe's integration test
# (tests/<p>.rs) and unit tests (src/cmd/<p>/tests.rs) together.
set -eu

DETECTION="sweep bopla matrix race timing cors gql headers discover params smuggle jwt"

# A recognizable "safe input was cleared / no finding / inconclusive / error" assertion.
NEG='safe|cleared|no candidate|not observed|no .* observed|inconclusive|no race|no function|HEADERS ERROR|not_flag|does_not|do_not|must not|must_not|is_empty|is_null|== ?false|!= ?true|BLOCKED|denied|not .*contains|!issues|no broad'

failed=0
for p in $DETECTION; do
  files=""
  [ -f "tests/$p.rs" ] && files="$files tests/$p.rs"
  [ -f "src/cmd/$p/tests.rs" ] && files="$files src/cmd/$p/tests.rs"
  if [ -z "$files" ]; then
    echo "oracle-coverage: '$p' has no test file (tests/$p.rs or src/cmd/$p/tests.rs)" >&2
    failed=1
    continue
  fi
  # shellcheck disable=SC2086
  if ! grep -hqE '#\[test\]' $files; then
    echo "oracle-coverage: '$p' has no #[test]" >&2
    failed=1
    continue
  fi
  # shellcheck disable=SC2086
  if ! grep -hiqE "$NEG" $files; then
    echo "oracle-coverage: '$p' has no safe-clear / negative-control assertion (a detection probe must prove it CLEARS a safe input, not only that it flags a vulnerable one)" >&2
    failed=1
  fi
done

exit "$failed"
