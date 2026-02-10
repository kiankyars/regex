# Regex Engine — Progress Tracker

## Current Status
- **Pass rate:** 97% (37/38 tests passing)
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
16. Investigate test 32 failure (escaped backslash test harness bug) — see `notes/test_harness_backslash_bug.md`
17. Add more edge case handling and hardening
18. Performance optimization

## Completed Tasks
- **2026-02-09:** Full engine implementation (parser → AST → compiler → VM). All features implemented: literals, concatenation, quantifiers (greedy/lazy), character classes, shorthand classes, anchors, alternation, groups (capturing/non-capturing), dot, escapes, bounded repetition, backreferences, lookahead/lookbehind. 97% pass rate.

## Known Issues
- Test 32 (escaped backslash): False failure due to Python oracle interpreting `\b` as backspace in non-raw string. Our engine is correct. See `notes/test_harness_backslash_bug.md`.

## Architecture Decisions
- **Backtracking VM:** We use a recursive backtracking VM (not Thompson NFA) because backreferences and lookaround assertions require backtracking.
- **Module structure:** `ast.rs` (types), `parser.rs` (pattern → AST), `compiler.rs` (AST → bytecode), `vm.rs` (bytecode execution), `main.rs` (CLI).
- **Bytecode-based:** The compiler emits instructions (Char, Split, Jump, Save, etc.) that the VM interprets. This cleanly separates parsing from execution.
