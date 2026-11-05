//! Select complete native-generated unified hunks; never accept renderer-authored patches.
const MAX_PATCH_BYTES: usize = 256 * 1024;
fn range(value: &str) -> bool {
    let fields: Vec<_> = value.split(',').collect();
    (1..=2).contains(&fields.len())
        && fields
            .iter()
            .all(|field| !field.is_empty() && field.bytes().all(|c| c.is_ascii_digit()))
}
fn header(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("@@ -") else {
        return false;
    };
    let Some((old, rest)) = rest.split_once(" +") else {
        return false;
    };
    let Some((new, _)) = rest.split_once(" @@") else {
        return false;
    };
    range(old) && range(new)
}
pub fn select_hunk(patch: &str, index: usize) -> Result<String, String> {
    if patch.len() > MAX_PATCH_BYTES {
        return Err(
            "Hunk staging is limited to text patches up to 256 KiB. Stage the whole file instead."
                .into(),
        );
    }
    let mut prefix = String::new();
    let mut hunks: Vec<String> = Vec::new();
    let mut files = 0;
    for line in patch.split_inclusive('\n') {
        if line.starts_with("diff --git ") {
            files += 1;
        }
        if [
            "diff --cc ",
            "diff --combined ",
            "@@@",
            "GIT binary patch",
            "Binary files ",
            "old mode ",
            "new mode ",
            "new file mode ",
            "deleted file mode ",
            "rename from ",
            "rename to ",
            "copy from ",
            "copy to ",
        ]
        .iter()
        .any(|prefix| line.starts_with(prefix))
        {
            return Err("Use whole-file staging for binary, new/deleted/renamed files, links, submodules or mode changes.".into());
        }
        if header(line) {
            hunks.push(line.into());
        } else if line.starts_with("@@ ") {
            return Err("Unexpected hunk header. Refresh before staging.".into());
        } else if let Some(hunk) = hunks.last_mut() {
            hunk.push_str(line);
        } else {
            prefix.push_str(line);
        }
    }
    if files != 1
        || !prefix.starts_with("diff --git ")
        || !prefix.lines().any(|l| l.starts_with("--- "))
        || !prefix.lines().any(|l| l.starts_with("+++ "))
    {
        return Err("A single regular-file text patch is required.".into());
    }
    let hunk = hunks
        .get(index)
        .ok_or("This hunk no longer exists. Refresh and review it again.")?;
    Ok(prefix + hunk)
}

/// Derive a partial patch exclusively from Git's bounded, validated text diff.
/// Row numbers refer to the original unified diff, including its header rows.
/// For unstage, build B -> B-with-selected-changes-reverted rather than applying
/// A -> selected(A,B) backwards against a different index with other additions.
pub fn select_lines(patch: &str, rows: &[usize], unstage: bool) -> Result<String, String> {
    use std::collections::BTreeSet;
    // Reuse the strict regular-file/size checks of complete-hunk staging.
    select_hunk(patch, 0)?;
    if patch.lines().any(|l| l.starts_with("\\ No newline")) {
        return Err("Use whole-hunk staging for a file without a final newline.".into());
    }
    let selected: BTreeSet<_> = rows.iter().copied().collect();
    if selected.is_empty() || selected.len() != rows.len() {
        return Err("Select distinct added or deleted diff rows.".into());
    }
    let lines: Vec<_> = patch.split_inclusive('\n').collect();
    let first_hunk = lines
        .iter()
        .position(|line| header(line))
        .ok_or("Missing diff hunk")?;
    for &row in &selected {
        let Some(line) = lines.get(row) else {
            return Err("Selected diff row no longer exists.".into());
        };
        if !(line.starts_with('+') || line.starts_with('-')) || row <= first_hunk {
            return Err("Only added or deleted content rows may be staged.".into());
        }
    }
    let mut prefix = String::new();
    let mut result = String::new();
    let mut delta = 0isize;
    let mut position = 0;
    while position < lines.len() && !header(lines[position]) {
        if !lines[position].starts_with("index ") {
            prefix.push_str(lines[position]);
        }
        position += 1;
    }
    let mut consumed = 0;
    while position < lines.len() {
        let title = lines[position];
        if !header(title) {
            return Err("Unexpected text outside a diff hunk.".into());
        }
        let rest = title.strip_prefix("@@ -").unwrap();
        let (a, rest) = rest.split_once(" +").unwrap();
        let (b, _) = rest.split_once(" @@").unwrap();
        let parse = |s: &str| -> Result<(usize, usize), String> {
            let (start, count) = s.split_once(',').map_or((s, "1"), |(a, b)| (a, b));
            Ok((
                start.parse().map_err(|_| "Invalid hunk start")?,
                count.parse().map_err(|_| "Invalid hunk count")?,
            ))
        };
        let old = parse(a)?;
        let new = parse(b)?;
        let source = if unstage { new } else { old };
        position += 1;
        let mut body = String::new();
        let mut old_count = 0usize;
        let mut new_count = 0usize;
        let mut changes = 0;
        let mut original_a = 0;
        let mut original_b = 0;
        while position < lines.len() && !header(lines[position]) {
            let line = lines[position];
            let sign = match line.as_bytes().first() {
                Some(b' ') => ' ',
                Some(b'+') => '+',
                Some(b'-') => '-',
                _ => return Err("Unexpected diff content. Use whole-file staging.".into()),
            };
            if sign != '+' {
                original_a += 1;
            }
            if sign != '-' {
                original_b += 1;
            }
            let picked = selected.contains(&position);
            if picked {
                changes += 1;
                consumed += 1;
            }
            let oriented = if unstage {
                match sign {
                    '+' => '-',
                    '-' => '+',
                    s => s,
                }
            } else {
                sign
            };
            let output = match (oriented, picked) {
                (' ', _) => Some(' '),
                ('-', true) => Some('-'),
                ('-', false) => Some(' '),
                ('+', true) => Some('+'),
                ('+', false) => None,
                _ => unreachable!(),
            };
            if let Some(sign) = output {
                if sign != '+' {
                    old_count += 1;
                }
                if sign != '-' {
                    new_count += 1;
                }
                body.push(sign);
                body.push_str(&line[1..]);
            }
            position += 1;
        }
        if (original_a, original_b) != (old.1, new.1) {
            return Err("Diff hunk counts changed. Refresh before staging.".into());
        }
        if changes == 0 {
            continue;
        }
        if old_count != source.1 {
            return Err("Invalid partial diff source range.".into());
        }
        let base = source.0 as isize + delta;
        let target = if old_count == 0 {
            base + 1
        } else if new_count == 0 {
            base - 1
        } else {
            base
        };
        if target < 0 {
            return Err("Invalid partial diff target range.".into());
        }
        result.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            source.0, old_count, target, new_count
        ));
        result.push_str(&body);
        delta += new_count as isize - old_count as isize;
    }
    if consumed != selected.len() {
        return Err("Selected row was not in a hunk.".into());
    }
    Ok(prefix + &result)
}
