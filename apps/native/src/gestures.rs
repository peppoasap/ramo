//! Internal drag intent only. A drop never authorizes a Git write.
use ramo_core::git::{Action, Snapshot};

#[derive(Clone, Debug)]
pub enum Source {
    Commit {
        oid: String,
        parents: usize,
        subject: String,
    },
    Branch {
        oid: String,
        name: String,
        remote: bool,
    },
}
impl Source {
    pub fn oid(&self) -> &str {
        match self {
            Self::Commit { oid, .. } | Self::Branch { oid, .. } => oid,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            Self::Commit { subject, .. } => subject,
            Self::Branch { name, .. } => name,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Drag {
    pub repository: String,
    pub fingerprint: String,
    pub source: Source,
}
/// Capture the pressed source before rapid pointer events can be coalesced.
/// Six logical pixels separate ordinary inspection clicks from a drag.
pub struct Candidate {
    pub drag: Drag,
    pub origin: [f32; 2],
}
impl Candidate {
    pub fn moved(&self, point: [f32; 2]) -> bool {
        let dx = point[0] - self.origin[0];
        let dy = point[1] - self.origin[1];
        dx * dx + dy * dy >= 36.
    }
}
pub enum Intent {
    Compare {
        from: String,
        to: String,
    },
    Review {
        action: Action,
        title: &'static str,
        detail: String,
    },
}
impl Drag {
    pub fn new(snapshot: &Snapshot, source: Source) -> Self {
        Self {
            repository: snapshot.repo.id.clone(),
            fingerprint: snapshot.fingerprint.clone(),
            source,
        }
    }
    pub fn current(&self, snapshot: &Snapshot) -> bool {
        self.repository == snapshot.repo.id && self.fingerprint == snapshot.fingerprint
    }
    pub fn can_continue(&self, snapshot: &Snapshot, busy: bool, focused: bool) -> bool {
        focused && !busy && self.current(snapshot)
    }
    pub fn intent(
        &self,
        snapshot: &Snapshot,
        target: Option<&str>,
        busy: bool,
    ) -> Result<Intent, String> {
        if busy || !self.current(snapshot) {
            return Err("Repository changed or an operation is busy. Start the drag again.".into());
        }
        if let Source::Branch { oid, name, remote } = &self.source
            && !snapshot
                .branches
                .iter()
                .any(|b| b.oid == *oid && b.name == *name && b.remote == *remote)
        {
            return Err("This branch changed. Refresh and try again.".into());
        }
        if let Some(to) = target {
            if to == self.source.oid() {
                return Err("Choose a different commit to compare.".into());
            }
            return Ok(Intent::Compare {
                from: self.source.oid().into(),
                to: to.into(),
            });
        }
        if snapshot.operation.is_some()
            || !snapshot.files.is_empty()
            || snapshot.head.is_none()
            || snapshot.branch == "Detached HEAD"
        {
            return Err("A clean working tree and an attached branch are required. Finish the current operation first.".into());
        }
        match &self.source {
            Source::Branch { name, remote, .. } => {
                if *remote || *name == snapshot.branch {
                    return Err(
                        "Choose another local branch to merge into the current branch.".into(),
                    );
                }
                Ok(Intent::Review {
                    action: Action::Merge { name: name.clone() },
                    title: "Merge local branch",
                    detail: format!(
                        "Merge {name} into {}. Fast-forward when possible; otherwise create a merge commit. Conflicts open in Changes. Push is separate.",
                        snapshot.branch
                    ),
                })
            }
            Source::Commit {
                oid,
                parents,
                subject,
            } => {
                if *parents > 1 || snapshot.head.as_deref() == Some(oid) {
                    return Err(
                        "Choose a non-merge commit other than the current HEAD to cherry-pick."
                            .into(),
                    );
                }
                Ok(Intent::Review {
                    action: Action::CherryPick {
                        target: oid.clone(),
                    },
                    title: "Cherry-pick commit",
                    detail: format!(
                        "Apply {} ({subject}) onto {} as a new local commit. Conflicts can be resolved, continued or aborted from Changes.",
                        &oid[..oid.len().min(8)],
                        snapshot.branch
                    ),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_is_captured_before_a_coalesced_release_and_clicks_stay_clicks() {
        let candidate = Candidate {
            origin: [100., 200.],
            drag: Drag {
                repository: "repo".into(),
                fingerprint: "revision".into(),
                source: Source::Commit {
                    oid: "saved-object".into(),
                    parents: 1,
                    subject: "Subject".into(),
                },
            },
        };
        assert!(!candidate.moved([103., 203.]));
        assert!(candidate.moved([106., 200.]));
        assert!(candidate.moved([600., 240.]));
        assert_eq!(candidate.drag.source.oid(), "saved-object");
    }
}
