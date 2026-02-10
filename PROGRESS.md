# Regex Engine — Progress Tracker

## Current Status
- **Pass rate:** 100% (54/54 tests passing)
- **Last updated:** 2026-02-09

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

## Completed Tasks
- **2026-02-09:** Full engine implementation (parser → AST → compiler → VM). All features implemented: literals, concatenation, quantifiers (greedy/lazy), character classes, shorthand classes, anchors, alternation, groups (capturing/non-capturing), dot, escapes, bounded repetition, backreferences, lookahead/lookbehind. 97% pass rate.
- **2026-02-09:** Fixed test harness bug (test 32): Python oracle was interpreting escape sequences in input via string interpolation. Fixed by passing pattern/input via sys.argv instead. Also added 16 new edge case tests. 100% pass rate (54/54).
- **2026-02-09:** Performance optimization of VM. Replaced `captures.clone()` in Split with undo log (save/restore only changed slots). Added recursion depth limit (10,000) to prevent stack overflow. See `notes/vm_performance.md`. 100% pass rate (54/54).

## Known Issues
- ~~Test 32 (escaped backslash)~~: **FIXED.** The Python oracle was using string interpolation which caused Python escape interpretation. Fixed by passing values via `sys.argv`.

## Architecture Decisions
- **Backtracking VM:** We use a recursive backtracking VM (not Thompson NFA) because backreferences and lookaround assertions require backtracking.
- **Module structure:** `ast.rs` (types), `parser.rs` (pattern → AST), `compiler.rs` (AST → bytecode), `vm.rs` (bytecode execution), `main.rs` (CLI).
- **Bytecode-based:** The compiler emits instructions (Char, Split, Jump, Save, etc.) that the VM interprets. This cleanly separates parsing from execution.
