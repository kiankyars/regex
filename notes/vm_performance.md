# VM Performance Optimization

## Undo Log (replaces captures.clone())

The biggest performance issue in the original VM was `captures.clone()` on every `Split` instruction. This allocates a fresh `Vec<Option<usize>>` each time we branch, which is O(n_groups) per Split.

### The Fix: Undo Log

Instead of cloning the full captures array, we maintain an undo log — a `Vec<(usize, Option<usize>)>` — that records `(slot_index, old_value)` each time a `Save` instruction modifies a capture slot.

On `Split`:
1. Record the current undo log length as a "mark"
2. Try the first branch (which may push more entries to the undo log)
3. If first branch fails, pop entries back to the mark, restoring each capture slot
4. Try the second branch

This is O(k) per backtrack where k = number of Save instructions that fired, instead of O(n_groups) for cloning the whole array. For patterns with many groups but few saves per branch, this is much faster.

### Lookaround Still Clones

Lookaround assertions (`(?=...)`, `(?<=...)`, etc.) still use `captures.to_vec()` because they need a completely isolated captures environment — the sub-match shouldn't affect the outer captures. This is inherent to lookaround semantics.

## Recursion Depth Limit

Added `MAX_DEPTH = 10_000` to prevent stack overflow on pathological inputs (e.g., deeply nested quantifiers). The VM returns `false` (no match) when depth is exceeded.

## Future Optimization Ideas

- Convert `CharClass` items to a bitmap for ASCII characters (O(1) lookup vs O(n) scan)
- Memoization / visited-state cache to avoid re-exploring (pc, pos) pairs
- Early termination: if pattern starts with literal, skip positions where first char doesn't match
