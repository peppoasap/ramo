//! Byte-preserving editor transformations. Selecting text never writes to Git.
use std::{collections::BTreeSet, ops::Range};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub range: Range<usize>,
    pub ours: String,
    pub theirs: String,
}

pub fn conflicts(text: &str) -> Vec<Conflict> {
    let mut lines = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        lines.push((offset, line));
        offset += line.len();
    }
    let mut found = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].1.starts_with("<<<<<<<") {
            i += 1;
            continue;
        }
        let mut base = None;
        let mut split = None;
        let mut end = None;
        for (j, (_, line)) in lines.iter().enumerate().skip(i + 1) {
            if line.starts_with("<<<<<<<") {
                break;
            }
            if line.starts_with("|||||||") {
                base = Some(j);
            }
            if line.starts_with("=======") {
                split = Some(j);
            }
            if line.starts_with(">>>>>>>") {
                end = Some(j);
                break;
            }
        }
        if let (Some(split), Some(end)) = (split, end)
            && split < end
            && base.is_none_or(|b| b < split)
        {
            found.push(Conflict {
                range: lines[i].0..lines[end].0 + lines[end].1.len(),
                ours: lines[i + 1..base.unwrap_or(split)]
                    .iter()
                    .map(|(_, l)| *l)
                    .collect(),
                theirs: lines[split + 1..end].iter().map(|(_, l)| *l).collect(),
            });
            i = end + 1;
        } else {
            i += 1;
        }
    }
    found
}

pub fn selected_lines(source: &str, picks: &BTreeSet<usize>) -> String {
    source
        .split_inclusive('\n')
        .enumerate()
        .filter(|(i, _)| picks.contains(&(i + 1)))
        .map(|(_, l)| l)
        .collect()
}

pub fn replace(text: &str, index: usize, replacement: &str) -> Option<String> {
    let block = conflicts(text).get(index)?.range.clone();
    let mut out = text.to_owned();
    let separator =
        if !replacement.is_empty() && !replacement.ends_with('\n') && block.end < text.len() {
            if text.contains("\r\n") { "\r\n" } else { "\n" }
        } else {
            ""
        };
    out.replace_range(block, &format!("{replacement}{separator}"));
    Some(out)
}

#[derive(Default)]
pub struct History {
    undo: Vec<String>,
    redo: Vec<String>,
    bytes: usize,
}
impl History {
    pub fn record(&mut self, before: &str) {
        self.redo.clear();
        self.undo.push(before.to_owned());
        self.bytes += before.len();
        // Independent transaction history for source-button edits and ordinary typing.
        while self.undo.len() > 80 || self.bytes > 8 * 1024 * 1024 {
            self.bytes -= self.undo.remove(0).len();
        }
    }
    pub fn undo(&mut self, current: &mut String) {
        if let Some(previous) = self.undo.pop() {
            self.bytes -= previous.len();
            self.redo.push(std::mem::replace(current, previous));
        }
    }
    pub fn redo(&mut self, current: &mut String) {
        if let Some(next) = self.redo.pop() {
            self.bytes += current.len();
            self.undo.push(std::mem::replace(current, next));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_preserve_order_duplicates_unicode_crlf_and_missing_eof() {
        let source = "uguale\r\nuguale\r\n日本語 🦀";
        assert_eq!(
            selected_lines(source, &BTreeSet::from([3, 1, 2, 99])),
            source
        );
        assert_eq!(selected_lines(source, &BTreeSet::from([2])), "uguale\r\n");
    }
    #[test]
    fn replaces_one_diff3_block_preserving_unrelated_bytes() {
        let text = "prima\r\n<<<<<<< HEAD\r\na\r\n||||||| base\r\nx\r\n=======\r\nb\r\n>>>>>>> side\r\ndopo";
        let blocks = conflicts(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].ours, "a\r\n");
        assert_eq!(
            replace(text, 0, "日本語"),
            Some("prima\r\n日本語\r\ndopo".into())
        );
        assert_eq!(replace(text, 1, "x"), None);
    }
    #[test]
    fn incomplete_markers_are_never_replaced() {
        assert!(conflicts("<<<<<<< HEAD\nx\n=======\ny").is_empty());
    }
    #[test]
    fn button_and_typing_edits_share_undo_redo_and_branching_history() {
        let mut history = History::default();
        let mut value = "iniziale".to_owned();
        history.record(&value);
        value = "scelta".into();
        history.record(&value);
        value.push_str(" editato");
        history.undo(&mut value);
        assert_eq!(value, "scelta");
        history.undo(&mut value);
        assert_eq!(value, "iniziale");
        history.redo(&mut value);
        assert_eq!(value, "scelta");
        history.record(&value);
        value = "nuovo".into();
        history.redo(&mut value);
        assert_eq!(value, "nuovo");
    }
}
