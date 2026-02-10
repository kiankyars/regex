#!/bin/bash
# test.sh — regex engine test harness
# Usage: ./test.sh [--fast]

set -e

FAST=false
SAMPLE_PERCENT=10
BINARY="./target/release/regex-engine"

if [ "$1" = "--fast" ]; then
    FAST=true
    # Deterministic per-agent, random across agents
    SEED="${AGENT_ID:-0}"
fi

set -euo pipefail
cargo build --release 2>&1 | tail -5
if [ $? -ne 0 ]; then
    echo "ERROR: Build failed"
    exit 1
fi

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY"
    exit 1
fi

PASS=0
FAIL=0
SKIP=0
TOTAL=0
ERRORS=""

run_test() {
    local pattern="$1"
    local input="$2"
    local expected="$3"
    local description="$4"
    TOTAL=$((TOTAL + 1))

    # In fast mode, only run ~10% of tests, deterministic per seed
    if [ "$FAST" = true ]; then
        HASH=$(echo "${SEED}:${TOTAL}" | md5sum | head -c 4)
        HASH_DEC=$((16#$HASH))
        if [ $((HASH_DEC % 100)) -ge $SAMPLE_PERCENT ]; then
            SKIP=$((SKIP + 1))
            return
        fi
    fi

    # Compare against Python's re module as oracle
    # Pass pattern/input via argv to avoid Python string escape interpretation
    EXPECTED_OUTPUT=$(python3 - "$pattern" "$input" <<'PYEOF'
import re, sys
pattern = sys.argv[1]
text = sys.argv[2]
try:
    m = re.search(pattern, text)
    if m:
        print('MATCH:' + m.group(0))
        for i, g in enumerate(m.groups(), 1):
            print(f'GROUP {i}:{g if g is not None else ""}')
    else:
        print('NO_MATCH')
except Exception as e:
    print('ERROR:' + str(e))
PYEOF
)

    ACTUAL_OUTPUT=$($BINARY "$pattern" "$input" 2>&1) || true

    if [ "$EXPECTED_OUTPUT" = "$ACTUAL_OUTPUT" ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        ERRORS="${ERRORS}\nFAIL test ${TOTAL}: pattern='${pattern}' input='${input}' desc='${description}'\n  expected: ${EXPECTED_OUTPUT}\n  actual:   ${ACTUAL_OUTPUT}"
    fi
}

echo "Running tests..."

# === BASIC LITERALS ===
run_test "hello" "hello world" "MATCH:hello" "basic literal"
run_test "xyz" "hello world" "NO_MATCH" "literal no match"
run_test "abc" "xxxabcxxx" "MATCH:abc" "literal in middle"

# === QUANTIFIERS ===
run_test "a*" "aaa" "MATCH:aaa" "star greedy"
run_test "a+" "aaa" "MATCH:aaa" "plus greedy"
run_test "a?" "aaa" "MATCH:a" "question mark"
run_test "a{3}" "aaaa" "MATCH:aaa" "exact repeat"
run_test "a{2,4}" "aaaaa" "MATCH:aaaa" "range repeat"
run_test "a*?" "aaa" "MATCH:" "star lazy"
run_test "a+?" "aaa" "MATCH:a" "plus lazy"

# === CHARACTER CLASSES ===
run_test "[abc]" "b" "MATCH:b" "char class"
run_test "[a-z]" "m" "MATCH:m" "char range"
run_test "[^abc]" "d" "MATCH:d" "negated class"
run_test "\\d" "5" "MATCH:5" "digit shorthand"
run_test "\\w+" "hello_123" "MATCH:hello_123" "word shorthand"
run_test "\\s" " " "MATCH: " "space shorthand"

# === ANCHORS ===
run_test "^hello" "hello world" "MATCH:hello" "start anchor"
run_test "world$" "hello world" "MATCH:world" "end anchor"
run_test "^hello$" "hello" "MATCH:hello" "both anchors"
run_test "^hello$" "hello world" "NO_MATCH" "anchors no match"

# === ALTERNATION ===
run_test "cat|dog" "I have a dog" "MATCH:dog" "alternation"
run_test "cat|dog" "I have a cat" "MATCH:cat" "alternation first"
run_test "cat|dog" "I have a fish" "NO_MATCH" "alternation no match"

# === GROUPS ===
run_test "(ab)+" "ababab" "MATCH:ababab" "group repeat"
run_test "(a)(b)(c)" "abc" "MATCH:abc" "capturing groups"
run_test "(?:ab)+" "ababab" "MATCH:ababab" "non-capturing group"

# === DOT ===
run_test "a.b" "axb" "MATCH:axb" "dot match"
run_test "a.b" "a\nb" "NO_MATCH" "dot no newline"
run_test ".*" "hello" "MATCH:hello" "dot star"

# === ESCAPES ===
run_test "\\." "a.b" "MATCH:." "escaped dot"
run_test "\\*" "a*b" "MATCH:*" "escaped star"
run_test "\\\\" "a\\b" "MATCH:\\" "escaped backslash"

# === BACKREFERENCES ===
run_test "(a+)\\1" "aaaa" "MATCH:aaaa" "backreference"
run_test "(\\w+) \\1" "hello hello" "MATCH:hello hello" "word backreference"

# === LOOKAHEAD/LOOKBEHIND ===
run_test "a(?=b)" "ab" "MATCH:a" "positive lookahead"
run_test "a(?!b)" "ac" "MATCH:a" "negative lookahead"
run_test "(?<=a)b" "ab" "MATCH:b" "positive lookbehind"
run_test "(?<!a)b" "cb" "MATCH:b" "negative lookbehind"

# === WORD BOUNDARIES ===
run_test "\\bfoo\\b" "a foo b" "MATCH:foo" "word boundary both sides"
run_test "\\bhello" "hello world" "MATCH:hello" "word boundary start"
run_test "\\Boo" "foobar" "MATCH:oo" "non-word boundary"

# === NEGATED SHORTHANDS ===
run_test "\\D+" "123abc456" "MATCH:abc" "non-digit shorthand"
run_test "\\W" "hello world" "MATCH: " "non-word shorthand"
run_test "\\S+" "  hello  " "MATCH:hello" "non-space shorthand"

# === UNBOUNDED REPETITION ===
run_test "a{2,}" "aaaa" "MATCH:aaaa" "at-least repetition"
run_test "a{2,4}?" "aaaa" "MATCH:aa" "lazy range repetition"

# === NESTED GROUPS ===
run_test "((a)(b))" "ab" "MATCH:ab" "nested capturing groups"
run_test "(cat|dog)s" "dogs" "MATCH:dogs" "alternation in group"

# === GREEDY VS LAZY ===
run_test "a(.*)b" "aXbYb" "MATCH:aXbYb" "greedy dot star"
run_test "a(.*?)b" "aXbYb" "MATCH:aXb" "lazy dot star"

# === CHAR CLASS EDGE CASES ===
run_test "[-abc]" "-" "MATCH:-" "hyphen in char class"
run_test "[a-z0-9]" "5" "MATCH:5" "multiple ranges in class"

# === LOOKAROUND COMBOS ===
run_test "(?=a)a" "a" "MATCH:a" "lookahead then match"
run_test "(?=.*b)a.b" "axb" "MATCH:axb" "lookahead with dot star"

# === REPORT ===
echo ""
echo "================================"
echo "RESULTS: ${PASS} passed, ${FAIL} failed, ${SKIP} skipped out of ${TOTAL} total"
RATE=0
TESTED=$((PASS + FAIL))
if [ $TESTED -gt 0 ]; then
    RATE=$((PASS * 100 / TESTED))
fi
echo "PASS RATE: ${RATE}%"
echo "================================"

if [ $FAIL -gt 0 ]; then
    echo ""
    echo "FAILURES:"
    echo -e "$ERRORS"
    # Log to file for agent inspection
    echo -e "$ERRORS" > test_failures.log
    echo ""
    echo "Failure details written to test_failures.log"
fi

exit $FAIL
