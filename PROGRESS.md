# Regex Engine — Progress Tracker

## Current Status
- **Pass rate:** 100% (159/159 tests passing)
- **Last updated:** 2026-02-10

## Priority Tasks (High → Low)
1. ~~Initialize Rust project with basic structure (parser, AST, engine modules)~~ **DONE**
2. ~~Implement literal matching~~ **DONE**
3. ~~Implement concatenation~~ **DONE**
4. ~~Implement quantifiers: `*`, `+`, `?`~~ **DONE**
5. ~~Implement character classes: `[abc]`, `[a-z]`, `[^abc]`~~ **DONE**
6. ~~Implement shorthand classes: `\d`, `\w`, `\s`~~ **DONE**
7. ~~Implement anchors: `^`, `$`~~ **DONE**
8. ~~Implement alternation: `|`~~ **DONE**
9. ~~Implement grouping and capturing: `(...)`, `(?:...)`~~ **DONE**
10. ~~Implement dot `.` matching~~ **DONE**
11. ~~Implement escape sequences~~ **DONE**
12. ~~Implement greedy vs lazy quantifiers~~ **DONE**
13. ~~Implement bounded repetition: `{n}`, `{n,m}`~~ **DONE**
14. ~~Implement backreferences: `\1`~~ **DONE**
15. ~~Implement lookahead/lookbehind~~ **DONE**
16. ~~Investigate test 32 failure (escaped backslash test harness bug)~~ **DONE** — fixed by passing args via sys.argv
17. ~~Add more edge case handling and hardening~~ **DONE** — added 16 new tests (word boundaries, negated shorthands, unbounded repetition, nested groups, greedy/lazy, char class edges, lookaround combos)
18. ~~Performance optimization~~ **DONE** — undo log for Split backtracking, recursion depth limit
19. ~~Search optimization~~ **DONE** — first-char skip and anchored-start optimization
20. ~~Expand test coverage~~ **DONE** — 15 new tests (quantifier edges, multi-backreference, real-world patterns, advanced lookaround, nested groups, boundary cases)
21. ~~Multi-digit backreferences~~ **DONE** — parser now handles `\10`, `\12`, etc. (was limited to `\1`-`\9`)
22. ~~Case-insensitive flag `(?i:...)`~~ **DONE** — new `CaseInsensitive` AST node, `CaseInsensitiveOn/Off` VM instructions, applies to literals, char classes, and backreferences
23. ~~Comprehensive hardening~~ **DONE** — fixed `a{5,3}` crash (min>max validation), 84 new edge case tests

## Completed Tasks
- **2026-02-09:** Full engine implementation (parser → AST → compiler → VM). All features implemented: literals, concatenation, quantifiers (greedy/lazy), character classes, shorthand classes, anchors, alternation, groups (capturing/non-capturing), dot, escapes, bounded repetition, backreferences, lookahead/lookbehind. 97% pass rate.
- **2026-02-09:** Fixed test harness bug (test 32): Python oracle was interpreting escape sequences in input via string interpolation. Fixed by passing pattern/input via sys.argv instead. Also added 16 new edge case tests. 100% pass rate (54/54).
- **2026-02-09:** Performance optimization of VM. Replaced `captures.clone()` in Split with undo log (save/restore only changed slots). Added recursion depth limit (10,000) to prevent stack overflow. See `notes/vm_performance.md`. 100% pass rate (54/54).
- **2026-02-10:** Search optimization: first-char skip (skip starting positions where first required char doesn't match) and anchored-start optimization (only try position 0 for `^`-anchored patterns). See `notes/search_optimization.md`. Added 15 new edge case tests covering quantifier edges, multi-backreference, real-world patterns (IP addresses, email-like, hex), advanced lookaround, nested groups, and boundary cases. 100% pass rate (69/69).
- **2026-02-10:** Multi-digit backreferences (`\10`, `\12`, etc.) and case-insensitive matching (`(?i:...)`). Parser now consumes all consecutive digits for backreference numbers. New `CaseInsensitive` AST node with `CaseInsensitiveOn/Off` bytecode instructions. CI mode applies to `Char`, `CharClass` (including ranges), and `Backref` instructions. Uses a depth counter for proper nesting. 7 new tests added. 100% pass rate (76/76).
- **2026-02-10:** Comprehensive hardening: fixed `a{5,3}` crash (integer underflow when min>max), added 84 new edge case tests covering empty patterns, empty alternation branches, zero-count quantifiers, quantified backreferences, lazy quantifier edges, complex lookaround, nested quantifiers, character class corners, brace literals, real-world patterns (dates, phone numbers, CSV, HTML tags), and combined assertions. See `notes/hardening_edge_cases.md`. 100% pass rate (159/159).

## Known Issues
- ~~Test 32 (escaped backslash)~~: **FIXED.** The Python oracle was using string interpolation which caused Python escape interpretation. Fixed by passing values via `sys.argv`.
- **Quantifiers on zero-width assertions:** Python 3.11+ errors on `\b+`, `^*`, `$+` ("nothing to repeat"). Our engine silently accepts them and matches empty. Low priority — these patterns are meaningless and rarely used.
- **Variable-length lookbehind:** Python errors on patterns like `(?<=a|ab)c`. Our engine accepts them but may produce different results. Low priority.

## Architecture Decisions
- **Backtracking VM:** We use a recursive backtracking VM (not Thompson NFA) because backreferences and lookaround assertions require backtracking.
- **Module structure:** `ast.rs` (types), `parser.rs` (pattern → AST), `compiler.rs` (AST → bytecode), `vm.rs` (bytecode execution), `main.rs` (CLI).
- **Bytecode-based:** The compiler emits instructions (Char, Split, Jump, Save, etc.) that the VM interprets. This cleanly separates parsing from execution.
