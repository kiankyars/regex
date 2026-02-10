/// AST types for the regex engine.

/// A single node in the regex AST.
#[derive(Debug, Clone)]
pub enum AstNode {
    /// Matches a single literal character.
    Literal(char),
    /// Matches any character (except newline by default).
    Dot,
    /// Concatenation of nodes (implicit in `ab`).
    Concat(Vec<AstNode>),
    /// Alternation (`a|b`).
    Alternation(Vec<AstNode>),
    /// Quantifier applied to a sub-expression.
    Quantifier {
        node: Box<AstNode>,
        kind: QuantifierKind,
        greedy: bool,
    },
    /// Character class like `[abc]`, `[a-z]`, `[^abc]`.
    CharClass {
        ranges: Vec<ClassItem>,
        negated: bool,
    },
    /// Shorthand class: `\d`, `\w`, `\s` and their negations.
    ShorthandClass(ShorthandKind),
    /// Anchor: `^`, `$`, `\b`.
    Anchor(AnchorKind),
    /// Capturing group `(...)` with a group index.
    Group {
        index: usize,
        node: Box<AstNode>,
    },
    /// Non-capturing group `(?:...)`.
    NonCapturingGroup {
        node: Box<AstNode>,
    },
    /// Backreference `\1`, `\2`, etc.
    Backreference(usize),
    /// Lookahead `(?=...)` or `(?!...)`.
    Lookahead {
        node: Box<AstNode>,
        positive: bool,
    },
    /// Lookbehind `(?<=...)` or `(?<!...)`.
    Lookbehind {
        node: Box<AstNode>,
        positive: bool,
    },
    /// Inline flags wrapper `(?flags:...)` — contents match with the given flags active.
    /// Flags may include: i (case-insensitive), s (dotall), m (multiline).
    InlineFlags {
        node: Box<AstNode>,
        flags: RegexFlags,
    },
}

/// Kind of quantifier.
#[derive(Debug, Clone)]
pub enum QuantifierKind {
    /// `*` — zero or more.
    Star,
    /// `+` — one or more.
    Plus,
    /// `?` — zero or one.
    Question,
    /// `{n}` — exactly n.
    Exact(usize),
    /// `{n,}` — at least n.
    AtLeast(usize),
    /// `{n,m}` — between n and m inclusive.
    Range(usize, usize),
}

/// Item within a character class.
#[derive(Debug, Clone)]
pub enum ClassItem {
    /// Single character.
    Literal(char),
    /// Character range `a-z`.
    Range(char, char),
    /// Shorthand within a class, e.g. `[\d]`.
    Shorthand(ShorthandKind),
}

/// Shorthand character class kind.
#[derive(Debug, Clone, Copy)]
pub enum ShorthandKind {
    /// `\d` — digits.
    Digit,
    /// `\D` — non-digits.
    NonDigit,
    /// `\w` — word characters.
    Word,
    /// `\W` — non-word characters.
    NonWord,
    /// `\s` — whitespace.
    Space,
    /// `\S` — non-whitespace.
    NonSpace,
}

/// Inline regex flags (i, s, m).
#[derive(Debug, Clone, Copy, Default)]
pub struct RegexFlags {
    /// Case-insensitive matching.
    pub case_insensitive: bool,
    /// Dotall mode: `.` matches newline.
    pub dotall: bool,
    /// Multiline mode: `^` and `$` match at line boundaries.
    pub multiline: bool,
}

/// Anchor kind.
#[derive(Debug, Clone, Copy)]
pub enum AnchorKind {
    /// `^` — start of string.
    Start,
    /// `$` — end of string.
    End,
    /// `\b` — word boundary.
    WordBoundary,
    /// `\B` — non-word boundary.
    NonWordBoundary,
}
