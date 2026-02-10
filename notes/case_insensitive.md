# Case-Insensitive Matching: `(?i:...)`

## Implementation

Case-insensitive matching is implemented with a depth counter (`ci_depth`) in the VM, controlled by `CaseInsensitiveOn` / `CaseInsensitiveOff` bytecode instructions.

### AST
- `AstNode::CaseInsensitive { node }` — wraps any sub-pattern

### Parser
- `(?i:...)` is parsed as a `CaseInsensitive` node containing the sub-pattern
- Standalone `(?i)` (without `:`) is NOT supported — would require significant parser refactoring to affect "rest of current group"

### Compiler
- Emits `CaseInsensitiveOn` before the sub-pattern and `CaseInsensitiveOff` after

### VM
- `ci_depth` counter is passed through the `exec()` call chain
- On backtracking (`Split`), `ci_depth` is saved and restored alongside the undo log
- Affects: `Char`, `CharClass` (including ranges), `Backref`
- Does NOT affect: `ShorthandClass` (these are already case-agnostic where appropriate)

### Scoping
The flag properly scopes — `a(?i:b)c` matches `aBc` but not `ABC`, because `a` and `c` are outside the case-insensitive region.

## Limitations
- Only ASCII case folding (`to_ascii_lowercase`), not full Unicode case folding
- `(?i)` without `:` is not supported (only `(?i:...)` syntax)
- Inline flags like `(?i)` that affect the rest of the pattern/group are not implemented

## Future Work
- Support `(?i)` as a global/group-scoped flag
- Other inline flags: `(?m)` for multiline, `(?s)` for dotall
