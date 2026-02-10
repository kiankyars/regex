# Comprehensive Hardening: Edge Cases and Findings

## Bugs Fixed

### 1. `a{5,3}` — Integer Underflow Crash (Critical)
Pattern `a{5,3}` (min > max in range quantifier) caused an integer underflow in the compiler at `Range(n, m)` where `for _ in 0..(*m - *n)` with usize types would wrap around, causing a massive allocation and OOM kill.

**Fix:** Added validation in `parse_brace_quantifier` to return an error when min > max. The error is raised after successful brace syntax parsing (so invalid brace syntax still falls back to literal), but before construction of the quantifier node.

### 2. Multi-Digit Backreferences
Parser now greedily consumes consecutive digits for backreferences, supporting `\10`, `\11`, etc.

## Patterns Tested and Confirmed Working
- Empty pattern, empty alternation branches (`a|`, `|b`, `(a|)`)
- Zero-count quantifiers (`a{0}`, `a{0,0}`, `a{0,5}`)
- Multi-digit backreferences (`\10`, `\11`)
- Quantified backreferences (`(a)\1+`, `(\w)\1{2}`)
- Nested quantifiers (`(a+)+`, `(a*)*`, `(a+)*`)
- Lazy quantifier edge cases (`a??b`, `a{1,3}?`, `a{2,}?`)
- Complex lookaround (`(?<=a)(?=b)`, `(?=a+?)\\1`, double lookahead)
- Alternation with anchors (`^a|b`, `^a|^b`)
- Character class edge cases (`[]a]`, `[^]]`, `[\^\]]`)
- Brace literals (`{`, `}`, `{abc}`)
- Real-world patterns (dates, phone numbers, CSV, HTML tags, identifiers)

## Known Limitations (Not Fixed)
1. **Quantifiers on zero-width assertions**: Python 3.11+ errors on `\b+`, `^*`, `$+` ("nothing to repeat"). Our engine silently accepts them. This would require the parser to track which AST nodes are zero-width, which is a larger change.
2. **Variable-length lookbehind**: Python errors on `(?<=a|ab)c` ("look-behind requires fixed-width pattern"). Our engine accepts it but may produce different results.
3. **Error message format**: Our error messages don't include position information like Python's. Tests that compare error messages would fail.
