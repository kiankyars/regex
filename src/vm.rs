/// VM executor: runs compiled bytecode against an input string.
/// Uses recursive backtracking to support backreferences and lookaround.
///
/// Performance optimizations:
/// - Undo log instead of full captures.clone() on Split (save/restore only changed slots)
/// - Recursion depth limit to prevent stack overflow on pathological inputs

use crate::ast::{ClassItem, ShorthandKind};
use crate::compiler::{Inst, Program};

/// Maximum recursion depth for the backtracking VM.
const MAX_DEPTH: usize = 10_000;

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

/// An entry in the undo log: (slot_index, old_value).
type UndoEntry = (usize, Option<usize>);

/// Try to find a match anywhere in the input (like `re.search`).
pub fn search(program: &Program, input: &str) -> Option<MatchResult> {
    let chars: Vec<char> = input.chars().collect();
    let n_slots = (program.n_groups + 1) * 2;

    // If anchored at start, only try position 0
    if program.anchored_start {
        let mut captures = vec![None; n_slots];
        captures[0] = Some(0);
        let mut undo_log = Vec::new();
        if exec(program, &chars, 0, 0, &mut captures, &mut undo_log, 0) {
            captures[1] = Some(captures[1].unwrap_or(0));
            let end = captures[1].unwrap();
            return Some(MatchResult {
                start: 0,
                end,
                captures,
            });
        }
        return None;
    }

    // Try at each starting position
    for start in 0..=chars.len() {
        // First-char optimization: skip positions where the first required char doesn't match
        if let Some(fc) = program.first_char {
            if start < chars.len() {
                if chars[start] != fc {
                    continue;
                }
            } else {
                // At end of input, a required first char can't match
                continue;
            }
        }

        let mut captures = vec![None; n_slots];
        captures[0] = Some(start);
        let mut undo_log = Vec::new();
        if exec(program, &chars, start, 0, &mut captures, &mut undo_log, 0) {
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
///
/// Uses an undo log to efficiently save/restore capture slots on backtracking,
/// avoiding full Vec clones on every Split instruction.
fn exec(
    program: &Program,
    chars: &[char],
    pos: usize,
    pc: usize,
    captures: &mut [Option<usize>],
    undo_log: &mut Vec<UndoEntry>,
    depth: usize,
) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }

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
                // Save undo log position before trying first branch
                let undo_mark = undo_log.len();
                if exec(program, chars, pos, first, captures, undo_log, depth + 1) {
                    return true;
                }
                // Restore captures from undo log
                while undo_log.len() > undo_mark {
                    let (slot, old_val) = undo_log.pop().unwrap();
                    captures[slot] = old_val;
                }
                // Try second branch (tail call — continue loop)
                pc = second;
            }
            Inst::Save(slot) => {
                let slot = *slot;
                // Record old value in undo log before overwriting
                undo_log.push((slot, captures[slot]));
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
                // Run sub-program from current position; don't advance position.
                // Use a separate captures copy so failure doesn't corrupt captures.
                // On success, propagate capture groups back (positive lookahead
                // captures are visible to the rest of the pattern).
                let mut sub_captures: Vec<Option<usize>> = captures.to_vec();
                let mut sub_undo = Vec::new();
                if exec_sub(program, chars, pos, sub_start, sub_end, &mut sub_captures, &mut sub_undo, depth + 1) {
                    // Propagate capture groups (skip slots 0,1 which are full match bounds)
                    for i in 2..captures.len() {
                        if sub_captures[i] != captures[i] {
                            undo_log.push((i, captures[i]));
                            captures[i] = sub_captures[i];
                        }
                    }
                    pc = sub_end; // continue after the lookahead sub-program
                } else {
                    return false;
                }
            }
            Inst::LookaheadNegative(sub_start, sub_end) => {
                let sub_start = *sub_start;
                let sub_end = *sub_end;
                let mut sub_captures: Vec<Option<usize>> = captures.to_vec();
                let mut sub_undo = Vec::new();
                if !exec_sub(program, chars, pos, sub_start, sub_end, &mut sub_captures, &mut sub_undo, depth + 1) {
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
                    let mut sub_captures: Vec<Option<usize>> = captures.to_vec();
                    let mut sub_undo = Vec::new();
                    if exec_sub(program, chars, try_pos, sub_start, sub_end, &mut sub_captures, &mut sub_undo, depth + 1) {
                        // The sub-match must end exactly at `pos`
                        if sub_captures[1] == Some(pos) {
                            // Propagate capture groups back (skip slots 0,1)
                            for i in 2..captures.len() {
                                if sub_captures[i] != captures[i] {
                                    undo_log.push((i, captures[i]));
                                    captures[i] = sub_captures[i];
                                }
                            }
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
                    let mut sub_captures: Vec<Option<usize>> = captures.to_vec();
                    let mut sub_undo = Vec::new();
                    if exec_sub(program, chars, try_pos, sub_start, sub_end, &mut sub_captures, &mut sub_undo, depth + 1) {
                        if sub_captures[1] == Some(pos) {
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
    captures: &mut [Option<usize>],
    undo_log: &mut Vec<UndoEntry>,
    depth: usize,
) -> bool {
    // We run the sub-program starting at sub_start.
    // The sub-program ends with a Match instruction.
    // We save capture[1] to track where the sub-match ends.
    let old_cap1 = captures[1];
    captures[1] = None;
    let result = exec(program, chars, pos, sub_start, captures, undo_log, depth);
    if !result {
        captures[1] = old_cap1;
    }
    result
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
