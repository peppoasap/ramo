//! Bounded navigation intent, stored with the existing atomic/CAS local writer.
use crate::drafts::{self, Scope};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Component, Path},
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    History,
    Changes,
    Compare,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    pub from: String,
    pub to: String,
    pub ancestor: bool,
    pub labels: [Option<(String, String)>; 2],
    pub filters: [String; 2],
}
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub mode: Mode,
    pub filter: String,
    pub branches: String,
    pub selected: Option<String>,
    pub file: Option<String>,
    pub staged: bool,
    pub comparison: Option<Comparison>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    pub id: String,
    pub name: String,
    pub path: String,
    pub view: View,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u32,
    pub tabs: Vec<Tab>,
    pub active: Option<String>,
    pub compact: bool,
    pub inspector: bool,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            tabs: vec![],
            active: None,
            compact: true,
            inspector: true,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Loaded {
    pub document: Document,
    pub revision: String,
}
pub struct Restored {
    pub tabs: Vec<(Tab, Option<String>)>,
    pub active: Option<String>,
    pub compact: bool,
    pub inspector: bool,
    pub notices: Vec<String>,
}
pub fn bounded(value: &str, max: usize) -> String {
    let mut end = value.len().min(max);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}
pub fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn text(value: &str, max: usize) -> bool {
    value.len() <= max && !value.contains('\0')
}
impl Document {
    pub fn encode(&self) -> Result<String, String> {
        if self.version != 1 || self.tabs.len() > 20 {
            return Err(
                "Unsupported workspace version or tab count. Saved data was preserved.".into(),
            );
        }
        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        for tab in &self.tabs {
            let v = &tab.view;
            let file_ok = v.file.as_ref().is_none_or(|p| {
                text(p, 4096)
                    && !p.is_empty()
                    && Path::new(p)
                        .components()
                        .all(|c| matches!(c, Component::Normal(_)))
            });
            let compare_ok = v.comparison.as_ref().is_none_or(|c| {
                oid(&c.from)
                    && oid(&c.to)
                    && c.filters.iter().all(|s| text(s, 1024))
                    && c.labels
                        .iter()
                        .flatten()
                        .all(|(id, name)| oid(id) && text(name, 2048))
            });
            if tab.id != ramo_core::git::hash(tab.path.as_bytes())
                || tab.id.len() != 64
                || !oid(&tab.id)
                || !ids.insert(&tab.id)
                || !paths.insert(&tab.path)
                || !text(&tab.path, 4096)
                || !Path::new(&tab.path).is_absolute()
                || !text(&tab.name, 1024)
                || !text(&v.filter, 1024)
                || !text(&v.branches, 1024)
                || !file_ok
                || !compare_ok
                || v.selected.as_ref().is_some_and(|s| !oid(s))
            {
                return Err("Workspace contains invalid or unbounded navigation data. Saved data was preserved.".into());
            }
        }
        if self.active.as_ref().is_some_and(|id| !ids.contains(id)) {
            return Err("Workspace active tab is missing. Saved data was preserved.".into());
        }
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        if json.len() > 262_144 {
            return Err("Workspace exceeds256 KiB. Saved data was preserved.".into());
        }
        Ok(json)
    }
}
fn scope() -> Scope {
    Scope {
        repository: "native-workspace".into(),
        path: "session.json".into(),
        fingerprint: "workspace-v1".into(),
    }
}
pub fn load(state: &Path) -> Result<Option<Loaded>, String> {
    let Some(record) = drafts::load(&state.join("workspace"), &scope())? else {
        return Ok(None);
    };
    if record.record.text.len() > 262_144 {
        return Err("Workspace exceeds256 KiB. Saved data was preserved.".into());
    }
    let document: Document = serde_json::from_str(&record.record.text).map_err(
        |_| "Workspace is unreadable or has unsupported fields. Saved data was preserved.",
    )?;
    document.encode()?;
    Ok(Some(Loaded {
        document,
        revision: record.revision,
    }))
}
pub fn save(state: &Path, document: Document, expected: Option<String>) -> Result<Loaded, String> {
    // Validate an existing inner schema before the generic CAS writer can replace it.
    load(state)?;
    let text = document.encode()?;
    let record =
        drafts::save(&state.join("workspace"), scope(), text, expected).map_err(|error| {
            error
                .replace("draft", "workspace")
                .replace("Draft", "Workspace")
        })?;
    Ok(Loaded {
        document,
        revision: record.revision,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn doc(root: &Path) -> Document {
        let path = root.join("meadow").to_string_lossy().into_owned();
        let id = ramo_core::git::hash(path.as_bytes());
        Document {
            tabs: vec![Tab {
                id: id.clone(),
                name: "Meadow 日本語".into(),
                path,
                view: View {
                    filter: "Normalize".into(),
                    ..Default::default()
                },
            }],
            active: Some(id),
            ..Default::default()
        }
    }
    #[test]
    fn workspace_roundtrip_and_competing_writer_preserve_both_intents() {
        let dir = tempfile::tempdir().unwrap();
        let original = doc(dir.path());
        let first = save(dir.path(), original.clone(), None).unwrap();
        assert_eq!(load(dir.path()).unwrap().unwrap().document, original);
        let mut changed = original.clone();
        changed.inspector = false;
        let second = save(dir.path(), changed.clone(), Some(first.revision.clone())).unwrap();
        assert!(
            save(dir.path(), original.clone(), Some(first.revision))
                .unwrap_err()
                .contains("workspace")
        );
        assert_eq!(load(dir.path()).unwrap().unwrap().revision, second.revision);
        assert_eq!(original.tabs[0].view.filter, "Normalize");
        assert!(!original.encode().unwrap().contains("trusted"));
    }
    #[test]
    fn malformed_future_or_authority_bearing_files_are_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let original = doc(dir.path());
        let future = original
            .encode()
            .unwrap()
            .replacen("\"version\":1", "\"version\":2", 1);
        let stored =
            drafts::save(&dir.path().join("workspace"), scope(), future.clone(), None).unwrap();
        assert!(load(dir.path()).is_err());
        assert!(save(dir.path(), original.clone(), Some(stored.revision.clone())).is_err());
        assert_eq!(
            drafts::load(&dir.path().join("workspace"), &scope())
                .unwrap()
                .unwrap()
                .record
                .text,
            future
        );
        let corrupt = tempfile::tempdir().unwrap();
        drafts::save(
            &corrupt.path().join("workspace"),
            scope(),
            "not JSON".into(),
            None,
        )
        .unwrap();
        assert!(load(corrupt.path()).is_err());
        assert!(save(corrupt.path(), original.clone(), None).is_err());
        assert_eq!(
            drafts::load(&corrupt.path().join("workspace"), &scope())
                .unwrap()
                .unwrap()
                .record
                .text,
            "not JSON"
        );
        assert!(
            serde_json::from_str::<Document>(&original.encode().unwrap().replacen(
                "\"compact\":true",
                "\"trusted\":true,\"compact\":true",
                1
            ))
            .is_err()
        );
    }
    #[test]
    fn rejects_duplicates_unsafe_paths_oversize_and_bad_commit_ids() {
        let dir = tempfile::tempdir().unwrap();
        let original = doc(dir.path());
        let mut d = original.clone();
        d.tabs.push(d.tabs[0].clone());
        assert!(d.encode().is_err());
        let mut d = original.clone();
        d.tabs[0].view.file = Some("../outside".into());
        assert!(d.encode().is_err());
        let mut d = original.clone();
        d.tabs[0].view.selected = Some("HEAD; command".into());
        assert!(d.encode().is_err());
        let mut d = original;
        d.tabs[0].view.filter = "a".repeat(1025);
        assert!(d.encode().is_err());
    }
}
