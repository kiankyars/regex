# Test Harness Backslash Bug

## The Issue

Test 32 (`escaped backslash`) fails because of a discrepancy between what Python and our binary receive.

The test line:
```bash
run_test "\\\\" "a\\b" "MATCH:\\" "escaped backslash"
```

After bash expansion:
- pattern = `\\` (two chars: backslash backslash)
- input = `a\b` (three chars: a, backslash, b)

Our binary receives these exact bytes via argv. It correctly interprets `\\` as "match a literal backslash" and finds `\` in `a\b`, returning `MATCH:\`.

But the Python oracle receives the input via `'''a\b'''` (a non-raw Python string), which Python interprets `\b` as the backspace character (`\x08`). So Python searches for `\\` in `a<backspace>`, finds no match, and returns `NO_MATCH`.

## Our Engine Is Correct

Our engine correctly matches a literal backslash. The test harness has a subtle bug: it uses non-raw Python strings for the input, causing Python escape sequences (like `\b`, `\n`, `\t`) to be interpreted differently than what the binary receives via shell argv.

## Impact

Only affects test 32. All other tests pass.

## Fix Applied

Fixed by passing pattern and input to the Python oracle via `sys.argv` instead of string interpolation. This ensures Python receives the exact same bytes as the binary, avoiding any Python string escape interpretation. The Python script now uses a heredoc (`<<'PYEOF'`) to avoid both bash and Python escape issues.
