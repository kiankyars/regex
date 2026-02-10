# Search Optimization: First-Char and Anchor Hints

## First-Character Optimization

When the bytecode starts with a `Char(c)` instruction (or `AssertStart` followed by `Char(c)`), we know the pattern can only match at positions where the input has character `c`. The `search()` function uses this to skip positions where `chars[start] != c`, avoiding full VM execution at those positions.

This is stored as `Program.first_char: Option<char>`, computed at compile time by `extract_first_char()`.

### When It Helps
- Patterns like `hello.*world` — only try positions starting with 'h'
- Anchored patterns like `^abc` — skip all positions after 0 (see below)

### When It Doesn't Apply
- Alternation: `a|b` — no single first char
- Quantifiers at start: `a*b` — pattern can start with 'b' too
- Dot/classes at start: `.+foo`, `\d+` — many chars possible

## Start-Anchor Optimization

When the first instruction is `AssertStart` (from `^`), only position 0 can match. `search()` skips the loop entirely and only tries position 0.

Stored as `Program.anchored_start: bool`.

## Impact

For patterns starting with a literal in long inputs, this reduces the number of VM invocations from O(n) to O(occurrences of first char). For `^`-anchored patterns, it reduces to O(1) start positions.
