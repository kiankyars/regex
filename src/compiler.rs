/// Compiler: converts AST into bytecode instructions for the VM.

use crate::ast::*;

/// VM instruction.
#[derive(Debug, Clone)]
pub enum Inst {
    /// Match a specific character.
    Char(char),
    /// Match any character (except newline).
    AnyChar,
    /// Match a character class.
    CharClass { ranges: Vec<ClassItem>, negated: bool },
    /// Match a shorthand class (\d, \w, \s, etc.)
    ShorthandClass(ShorthandKind),
    /// Successful match.
    Match,
    /// Jump to target instruction.
    Jump(usize),
    /// Try first path, fallback to second (split).
    /// If greedy, prefer `first`; if lazy, prefer `second`.
    Split(usize, usize),
    /// Save position into capture slot.
    Save(usize),
    /// Assert start of string.
    AssertStart,
    /// Assert end of string.
    AssertEnd,
    /// Assert word boundary.
    AssertWordBoundary,
    /// Assert non-word boundary.
    AssertNonWordBoundary,
    /// Backreference: match the same text as capture group N.
    Backref(usize),
    /// Positive lookahead: sub-program from `start` to `end` (exclusive).
    LookaheadPositive(usize, usize),
    /// Negative lookahead.
    LookaheadNegative(usize, usize),
    /// Positive lookbehind.
    LookbehindPositive(usize, usize),
    /// Negative lookbehind.
    LookbehindNegative(usize, usize),
    /// No-op (used as placeholder).
    Nop,
    /// Begin case-insensitive matching.
    CaseInsensitiveOn,
    /// End case-insensitive matching.
    CaseInsensitiveOff,
}

/// Compiled program.
pub struct Program {
    pub insts: Vec<Inst>,
    pub n_groups: usize,
    /// If the pattern must start with a specific literal character, store it here.
    /// Used by the VM to skip starting positions that can't possibly match.
    pub first_char: Option<char>,
    /// Whether the pattern is anchored at the start (^).
    pub anchored_start: bool,
}

/// Compile an AST into a bytecode program.
pub fn compile(ast: &AstNode, n_groups: usize) -> Program {
    let mut insts = Vec::new();
    emit(&mut insts, ast);
    insts.push(Inst::Match);
    let first_char = extract_first_char(&insts);
    let anchored_start = matches!(insts.first(), Some(Inst::AssertStart));
    Program { insts, n_groups, first_char, anchored_start }
}

/// Extract the first required literal character from the instruction stream, if any.
fn extract_first_char(insts: &[Inst]) -> Option<char> {
    match insts.first()? {
        Inst::Char(ch) => Some(*ch),
        // If the first instruction is AssertStart, check the next one
        Inst::AssertStart => {
            match insts.get(1)? {
                Inst::Char(ch) => Some(*ch),
                _ => None,
            }
        }
        _ => None,
    }
}

fn emit(insts: &mut Vec<Inst>, node: &AstNode) {
    match node {
        AstNode::Literal(ch) => {
            insts.push(Inst::Char(*ch));
        }
        AstNode::Dot => {
            insts.push(Inst::AnyChar);
        }
        AstNode::Concat(nodes) => {
            for n in nodes {
                emit(insts, n);
            }
        }
        AstNode::Alternation(branches) => {
            // a|b|c compiles to:
            //   split L1, L2
            //   L1: <a> jump END
            //   L2: split L3, L4
            //   L3: <b> jump END
            //   L4: <c>
            //   END:
            let n = branches.len();
            if n == 0 {
                return;
            }
            if n == 1 {
                emit(insts, &branches[0]);
                return;
            }
            let mut fixup_jumps = Vec::new();
            for i in 0..n - 1 {
                let split_pc = insts.len();
                insts.push(Inst::Nop); // placeholder for split
                let branch_start = insts.len();
                emit(insts, &branches[i]);
                let jump_pc = insts.len();
                insts.push(Inst::Nop); // placeholder for jump to end
                fixup_jumps.push(jump_pc);
                let next_branch = insts.len();
                insts[split_pc] = Inst::Split(branch_start, next_branch);
            }
            // Last branch
            emit(insts, &branches[n - 1]);
            let end = insts.len();
            for jpc in fixup_jumps {
                insts[jpc] = Inst::Jump(end);
            }
        }
        AstNode::Quantifier { node: sub, kind, greedy } => {
            emit_quantifier(insts, sub, kind, *greedy);
        }
        AstNode::CharClass { ranges, negated } => {
            insts.push(Inst::CharClass {
                ranges: ranges.clone(),
                negated: *negated,
            });
        }
        AstNode::ShorthandClass(kind) => {
            insts.push(Inst::ShorthandClass(*kind));
        }
        AstNode::Anchor(AnchorKind::Start) => {
            insts.push(Inst::AssertStart);
        }
        AstNode::Anchor(AnchorKind::End) => {
            insts.push(Inst::AssertEnd);
        }
        AstNode::Anchor(AnchorKind::WordBoundary) => {
            insts.push(Inst::AssertWordBoundary);
        }
        AstNode::Anchor(AnchorKind::NonWordBoundary) => {
            insts.push(Inst::AssertNonWordBoundary);
        }
        AstNode::Group { index, node: sub } => {
            // Save start
            insts.push(Inst::Save(*index * 2));
            emit(insts, sub);
            // Save end
            insts.push(Inst::Save(*index * 2 + 1));
        }
        AstNode::NonCapturingGroup { node: sub } => {
            emit(insts, sub);
        }
        AstNode::Backreference(idx) => {
            insts.push(Inst::Backref(*idx));
        }
        AstNode::Lookahead { node: sub, positive } => {
            // Emit sub-program inline, wrap with lookahead marker
            let sub_start = insts.len() + 1; // after the lookahead instruction
            // We'll compile the sub-pattern as a separate sub-program
            // Reserve the lookahead instruction
            let la_pc = insts.len();
            insts.push(Inst::Nop);
            emit(insts, sub);
            insts.push(Inst::Match); // end of sub-program
            let sub_end = insts.len();
            if *positive {
                insts[la_pc] = Inst::LookaheadPositive(sub_start, sub_end);
            } else {
                insts[la_pc] = Inst::LookaheadNegative(sub_start, sub_end);
            }
        }
        AstNode::Lookbehind { node: sub, positive } => {
            let lb_pc = insts.len();
            insts.push(Inst::Nop);
            let sub_start = insts.len();
            emit(insts, sub);
            insts.push(Inst::Match);
            let sub_end = insts.len();
            if *positive {
                insts[lb_pc] = Inst::LookbehindPositive(sub_start, sub_end);
            } else {
                insts[lb_pc] = Inst::LookbehindNegative(sub_start, sub_end);
            }
        }
        AstNode::CaseInsensitive { node: sub } => {
            insts.push(Inst::CaseInsensitiveOn);
            emit(insts, sub);
            insts.push(Inst::CaseInsensitiveOff);
        }
    }
}

fn emit_quantifier(insts: &mut Vec<Inst>, sub: &AstNode, kind: &QuantifierKind, greedy: bool) {
    match kind {
        QuantifierKind::Star => {
            // L1: split L2, L3  (greedy: prefer L2)
            // L2: <sub> jump L1
            // L3:
            let l1 = insts.len();
            insts.push(Inst::Nop); // placeholder
            let l2 = insts.len();
            emit(insts, sub);
            insts.push(Inst::Jump(l1));
            let l3 = insts.len();
            if greedy {
                insts[l1] = Inst::Split(l2, l3);
            } else {
                insts[l1] = Inst::Split(l3, l2);
            }
        }
        QuantifierKind::Plus => {
            // L1: <sub>
            //     split L1, L2  (greedy: prefer L1)
            // L2:
            let l1 = insts.len();
            emit(insts, sub);
            let l2 = insts.len() + 1;
            if greedy {
                insts.push(Inst::Split(l1, l2));
            } else {
                insts.push(Inst::Split(l2, l1));
            }
        }
        QuantifierKind::Question => {
            // split L1, L2 (greedy: prefer L1)
            // L1: <sub>
            // L2:
            let split_pc = insts.len();
            insts.push(Inst::Nop);
            let l1 = insts.len();
            emit(insts, sub);
            let l2 = insts.len();
            if greedy {
                insts[split_pc] = Inst::Split(l1, l2);
            } else {
                insts[split_pc] = Inst::Split(l2, l1);
            }
        }
        QuantifierKind::Exact(n) => {
            for _ in 0..*n {
                emit(insts, sub);
            }
        }
        QuantifierKind::AtLeast(n) => {
            for _ in 0..*n {
                emit(insts, sub);
            }
            // Then star
            emit_quantifier(insts, sub, &QuantifierKind::Star, greedy);
        }
        QuantifierKind::Range(n, m) => {
            // First n required
            for _ in 0..*n {
                emit(insts, sub);
            }
            // Then up to (m - n) optional
            for _ in 0..(*m - *n) {
                emit_quantifier(insts, sub, &QuantifierKind::Question, greedy);
            }
        }
    }
}
