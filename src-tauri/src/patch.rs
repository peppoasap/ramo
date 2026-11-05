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
