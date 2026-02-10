/// VM executor: runs compiled bytecode against an input string.
/// Uses recursive backtracking to support backreferences and lookaround.

use crate::ast::{ClassItem, ShorthandKind};
use crate::compiler::{Inst, Program};

/// Result of a match attempt.
pub struct MatchResult {
    /// Start position in the input.
    pub start: usize,
    /// End position in the input (exclusive).
    pub end: usize,
    /// Captured groups: (start, end) pairs. Index 0 unused (group 0 = full match).
    /// Groups are 1-indexed. Slot `2*i` is start, `2*i+1` is end for group `i`.
    pub captures: Vec<Option<usize>>,
}

/// Try to find a match anywhere in the input (like `re.search`).
pub fn search(program: &Program, input: &str) -> Option<MatchResult> {
    let chars: Vec<char> = input.chars().collect();
    let n_slots = (program.n_groups + 1) * 2;

    // Try at each starting position
    for start in 0..=chars.len() {
        let mut captures = vec![None; n_slots];
        captures[0] = Some(start);
        if exec(program, &chars, start, 0, &mut captures) {
            captures[1] = Some(captures[1].unwrap_or(start));
            let end = captures[1].unwrap();
            return Some(MatchResult {
                start,
                end,
                captures,
            });
        }
    }
    None
}

/// Execute the VM from a given position and instruction pointer.
/// Returns true if a match is found.
fn exec(
    program: &Program,
    chars: &[char],
    pos: usize,
    pc: usize,
    captures: &mut Vec<Option<usize>>,
) -> bool {
    let mut pos = pos;
    let mut pc = pc;

    loop {
        if pc >= program.insts.len() {
            return false;
        }
        match &program.insts[pc] {
            Inst::Match => {
                // Record end of full match
                captures[1] = Some(pos);
                return true;
            }
            Inst::Char(expected) => {
                if pos < chars.len() && chars[pos] == *expected {
                    pos += 1;
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::AnyChar => {
                if pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::CharClass { ranges, negated } => {
                if pos < chars.len() && char_class_matches(chars[pos], ranges, *negated) {
                    pos += 1;
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::ShorthandClass(kind) => {
                if pos < chars.len() && shorthand_matches(chars[pos], *kind) {
                    pos += 1;
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::Jump(target) => {
                pc = *target;
            }
            Inst::Split(first, second) => {
                let first = *first;
                let second = *second;
                // Try first branch
                let saved = captures.clone();
                if exec(program, chars, pos, first, captures) {
                    return true;
                }
                // Restore captures and try second branch
                *captures = saved;
                pc = second;
            }
            Inst::Save(slot) => {
                let slot = *slot;
                captures[slot] = Some(pos);
                pc += 1;
            }
            Inst::AssertStart => {
                if pos == 0 {
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::AssertEnd => {
                if pos == chars.len() {
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::AssertWordBoundary => {
                if is_word_boundary(chars, pos) {
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::AssertNonWordBoundary => {
                if !is_word_boundary(chars, pos) {
                    pc += 1;
                } else {
                    return false;
                }
            }
            Inst::Backref(group_idx) => {
                let group_idx = *group_idx;
                let start_slot = group_idx * 2;
                let end_slot = group_idx * 2 + 1;
                match (captures[start_slot], captures[end_slot]) {
                    (Some(gs), Some(ge)) => {
                        let group_len = ge - gs;
                        if pos + group_len <= chars.len()
                            && chars[gs..ge] == chars[pos..pos + group_len]
                        {
                            pos += group_len;
                            pc += 1;
                        } else {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            Inst::LookaheadPositive(sub_start, sub_end) => {
                let sub_start = *sub_start;
                let sub_end = *sub_end;
                // Run sub-program from current position; don't advance position
                let mut sub_captures = captures.clone();
                if exec_sub(program, chars, pos, sub_start, sub_end, &mut sub_captures) {
                    pc = sub_end; // continue after the lookahead sub-program
                } else {
                    return false;
                }
            }
            Inst::LookaheadNegative(sub_start, sub_end) => {
                let sub_start = *sub_start;
                let sub_end = *sub_end;
                let mut sub_captures = captures.clone();
                if !exec_sub(program, chars, pos, sub_start, sub_end, &mut sub_captures) {
                    pc = sub_end;
                } else {
                    return false;
                }
            }
            Inst::LookbehindPositive(sub_start, sub_end) => {
                let sub_start = *sub_start;
                let sub_end = *sub_end;
                // Try all possible lengths behind the current position
                let mut found = false;
                for lookback in 0..=pos {
                    let try_pos = pos - lookback;
                    let mut sub_captures = captures.clone();
                    if exec_sub(program, chars, try_pos, sub_start, sub_end, &mut sub_captures) {
                        // The sub-match must end exactly at `pos`
                        if sub_captures[1] == Some(pos) || find_sub_end(&sub_captures) == Some(pos) {
                            found = true;
                            break;
                        }
                    }
                }
                if found {
                    pc = sub_end;
                } else {
                    return false;
                }
            }
            Inst::LookbehindNegative(sub_start, sub_end) => {
                let sub_start = *sub_start;
                let sub_end = *sub_end;
                let mut found = false;
                for lookback in 0..=pos {
                    let try_pos = pos - lookback;
                    let mut sub_captures = captures.clone();
                    if exec_sub(program, chars, try_pos, sub_start, sub_end, &mut sub_captures) {
                        if sub_captures[1] == Some(pos) || find_sub_end(&sub_captures) == Some(pos) {
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    pc = sub_end;
                } else {
                    return false;
                }
            }
            Inst::Nop => {
                pc += 1;
            }
        }
    }
}

/// Execute a sub-program (used for lookaround).
/// The sub-program runs from `sub_start` up to (but not including) the Match at sub_end-1.
fn exec_sub(
    program: &Program,
    chars: &[char],
    pos: usize,
    sub_start: usize,
    _sub_end: usize,
    captures: &mut Vec<Option<usize>>,
) -> bool {
    // We run the sub-program starting at sub_start.
    // The sub-program ends with a Match instruction.
    // We save capture[1] to track where the sub-match ends.
    let old_cap1 = captures[1];
    captures[1] = None;
    let result = exec(program, chars, pos, sub_start, captures);
    if !result {
        captures[1] = old_cap1;
    }
    result
}

/// Helper: find end position from sub-captures.
fn find_sub_end(captures: &[Option<usize>]) -> Option<usize> {
    captures[1]
}

/// Check if a character matches a character class.
fn char_class_matches(ch: char, items: &[ClassItem], negated: bool) -> bool {
    let mut matched = false;
    for item in items {
        match item {
            ClassItem::Literal(c) => {
                if ch == *c {
                    matched = true;
                    break;
                }
            }
            ClassItem::Range(lo, hi) => {
                if ch >= *lo && ch <= *hi {
                    matched = true;
                    break;
                }
            }
            ClassItem::Shorthand(kind) => {
                if shorthand_matches(ch, *kind) {
                    matched = true;
                    break;
                }
            }
        }
    }
    if negated { !matched } else { matched }
}

/// Check if a character matches a shorthand class.
fn shorthand_matches(ch: char, kind: ShorthandKind) -> bool {
    match kind {
        ShorthandKind::Digit => ch.is_ascii_digit(),
        ShorthandKind::NonDigit => !ch.is_ascii_digit(),
        ShorthandKind::Word => ch.is_ascii_alphanumeric() || ch == '_',
        ShorthandKind::NonWord => !(ch.is_ascii_alphanumeric() || ch == '_'),
        ShorthandKind::Space => ch.is_ascii_whitespace(),
        ShorthandKind::NonSpace => !ch.is_ascii_whitespace(),
    }
}

/// Check if `pos` is at a word boundary.
fn is_word_boundary(chars: &[char], pos: usize) -> bool {
    let before = if pos > 0 {
        is_word_char(chars[pos - 1])
    } else {
        false
    };
    let after = if pos < chars.len() {
        is_word_char(chars[pos])
    } else {
        false
    };
    before != after
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
