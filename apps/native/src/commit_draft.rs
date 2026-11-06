//! Repository-scoped composer state. Storage uses the existing atomic/CAS draft IO.
use crate::drafts::{Loaded, Scope};
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct Composer {
    pub text: String,
    pub repository: Option<String>,
    pub ready: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub reload_review: bool,
    pub load_request: Option<u64>,
    pub save_request: Option<u64>,
    saved: String,
    revision: Option<String>,
    context: String,
    edited: Option<Instant>,
}
impl Composer {
    pub fn bind(&mut self, repo: String, head: String) {
        *self = Self {
            repository: Some(repo),
            context: head,
            ..Default::default()
        };
    }
    pub fn scope(&self) -> Option<Scope> {
        Some(Scope {
            repository: self.repository.clone()?,
            path: "COMMIT_MESSAGE".into(),
            fingerprint: self.context.clone(),
        })
    }
    pub fn revision(&self) -> Option<String> {
        self.revision.clone()
    }
    pub fn dirty(&self) -> bool {
        self.text != self.saved
    }
    pub fn pending(&self) -> bool {
        self.load_request.is_some() || self.save_request.is_some()
    }
    pub fn changed(&mut self) {
        self.edited = Some(Instant::now());
        self.notice = None;
    }
    pub fn due(&self) -> bool {
        self.ready
            && self.dirty()
            && !self.pending()
            && self.error.is_none()
            && self
                .edited
                .is_some_and(|t| t.elapsed() >= Duration::from_millis(350))
    }
    pub fn loaded(&mut self, result: Result<Option<Loaded>, String>) {
        self.load_request = None;
        self.ready = true;
        match result {
            Ok(record) => {
                if record.is_none() {
                    self.text.clear();
                    self.saved.clear();
                    self.revision = None;
                    self.notice = None;
                }
                if let Some(record) = record {
                    self.text = record.record.text;
                    self.saved = self.text.clone();
                    self.revision = Some(record.revision);
                    if !self.text.is_empty() {
                        self.notice=Some("Recovered local message. Review it against the current staged changes.".into());
                    }
                }
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
    pub fn saved(&mut self, result: Result<Loaded, String>) {
        self.save_request = None;
        match result {
            Ok(record) => {
                self.saved = record.record.text;
                self.revision = Some(record.revision);
                self.error = None;
                self.notice = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
    pub fn committed(&mut self) {
        self.text.clear();
        self.changed();
        self.notice = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn saved(text: &str, revision: &str) -> Loaded {
        Loaded {
            record: crate::drafts::Record {
                version: 1,
                scope: Scope {
                    repository: "repo".into(),
                    path: "COMMIT_MESSAGE".into(),
                    fingerprint: "head".into(),
                },
                text: text.into(),
            },
            revision: revision.into(),
        }
    }
    #[test]
    fn late_ack_never_erases_newer_text_and_clear_is_persisted_after_commit() {
        let mut c = Composer::default();
        c.bind("repo".into(), "head".into());
        c.loaded(Ok(None));
        c.text = "first".into();
        c.changed();
        c.save_request = Some(1);
        c.text = "newer 日本語".into();
        c.saved(Ok(saved("first", "r1")));
        assert_eq!(c.text, "newer 日本語");
        assert!(c.dirty());
        assert_eq!(c.revision().as_deref(), Some("r1"));
        c.committed();
        assert!(c.dirty());
        c.saved(Ok(saved("", "r2")));
        assert!(!c.dirty());
    }
    #[test]
    fn switching_scope_and_failed_save_do_not_silently_claim_persistence() {
        let mut c = Composer::default();
        c.bind("repo".into(), "head".into());
        c.loaded(Ok(Some(saved("recovered", "r1"))));
        assert_eq!(c.text, "recovered");
        c.text = "keep in memory".into();
        c.saved(Err("Another window changed draft".into()));
        assert!(c.dirty());
        assert!(c.error.is_some());
        assert_eq!(c.text, "keep in memory");
        c.bind("other".into(), "different".into());
        assert!(!c.ready);
        assert!(c.text.is_empty());
        assert_eq!(c.scope().unwrap().repository, "other");
    }
}
