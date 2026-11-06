//! Read-only unified diff coordinates. Never used to build staging patches.
pub struct Line<'a> {
    pub text: &'a str,
    pub old: Option<usize>,
    pub new: Option<usize>,
    pub kind: char,
}
pub fn lines(diff: &str) -> Vec<Line<'_>> {
    let all: Vec<_> = diff.lines().take(12_001).collect();
    let start = all
        .iter()
        .position(|line| line.starts_with("@@ "))
        .unwrap_or(0);
    let visible = &all[start..];
    let coordinates = crate::patch_view::numbers(visible);
    visible
        .iter()
        .zip(coordinates)
        .map(|(&text, (old, new))| Line {
            text,
            old,
            new,
            kind: if text.starts_with("@@ ") {
                '@'
            } else {
                text.chars().next().unwrap_or(' ')
            },
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinates_follow_each_hunk_without_counting_metadata() {
        let rows = lines(
            "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -4,2 +4,3 @@\n keep\n-old\n+new\n+extra\n\\ No newline at end of file\n@@ -20 +21 @@\n-last\n+end\n",
        );
        assert_eq!(rows.len(), 9);
        assert_eq!((rows[1].old, rows[1].new), (Some(4), Some(4)));
        assert_eq!((rows[2].old, rows[2].new), (Some(5), None));
        assert_eq!((rows[4].old, rows[4].new), (None, Some(6)));
        assert_eq!((rows[7].old, rows[8].new), (Some(20), Some(21)));
    }
}
