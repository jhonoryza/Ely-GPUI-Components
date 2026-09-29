#!/bin/sh
# Pre-review gate: build rules plus house rules.
set -eu
cargo fmt --check
cargo clippy --all-targets --features test-support -- -D warnings
cargo test --lib --features test-support --quiet
python3 scripts/stories.py --check
fail=0
files=$(git ls-files -co --exclude-standard '*.rs')
multi=$(awk 'FNR==1{prev=0} /^[[:space:]]*\/\// {if(prev) print FILENAME":"FNR; prev=1; next} {prev=0}' $files)
[ -z "$multi" ] || { echo "multi-line comments:"; echo "$multi"; fail=1; }
for f in $files AGENTS.md README.md TASKS.md tasks/*.md; do
  n=$(wc -l < "$f")
  [ "$n" -le 500 ] || { echo "$f has $n lines"; fail=1; }
done
components=$(find src -name '*.rs' ! -path 'src/theme/*' ! -path 'src/motion/*' ! -path '*/tests/*' ! -name 'tests.rs')
raw=$(awk '/#\[cfg\(test\)\]/{nextfile} /(^|[^.[:alnum:]_])px\(/{print FILENAME":"FNR": "$0}' $components)
[ -z "$raw" ] || { echo "raw px in components:"; echo "$raw"; fail=1; }
deferred=$(grep -rn 'deferred(' src --include='*.rs' | grep -v '^src/primitives/layer.rs' || true)
[ -z "$deferred" ] || { echo "gpui deferred outside primitives::raise:"; echo "$deferred"; fail=1; }
instant=$(grep -rnE '(^|[^_[:alnum:]])time::(Instant|\{[^}]*Instant)' src examples || true)
[ -z "$instant" ] || { echo "std Instant panics in the browser, use web_time::Instant:"; echo "$instant"; fail=1; }
exit $fail
