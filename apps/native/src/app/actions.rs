use super::*;

pub(super) struct PendingAction {
    pub action: Action,
    title: String,
    detail: String,
    review: bool,
    fingerprint: String,
    repo: String,
}
#[derive(Clone, PartialEq)]
pub(super) enum FormKind {
    Branch,
    Stash,
    BranchAt(String),
    TagAt(String),
    Rename(String),
    Track(String),
    Amend(String),
}
impl FormKind {
    fn title(&self) -> &'static str {
        match self {
            Self::Branch => "Create branch",
            Self::Stash => "Save stash",
            Self::BranchAt(_) => "Branch from commit",
            Self::TagAt(_) => "Tag commit",
            Self::Rename(_) => "Rename branch",
            Self::Track(_) => "Track remote branch",
            Self::Amend(_) => "Amend latest commit",
        }
    }
}
pub(super) struct Form {
    pub kind: FormKind,
    pub text: String,
}
impl PendingAction {
    pub(super) fn resolution(editor: &Editor) -> Self {
        Self {
            action: Action::ResolveFile {
                path: editor.scope.path.clone(),
                content: Some(editor.result.clone()),
            },
            title: "Save and stage resolution".into(),
            detail: format!(
                "Replace {} with the result shown in the editor and stage it. The merge is completed separately in Changes.",
                editor.scope.path
            ),
            review: true,
            repo: editor.scope.repository.clone(),
            fingerprint: editor.scope.fingerprint.clone(),
        }
    }
}
impl App {
    pub(super) fn request_action(
        &mut self,
        action: Action,
        title: impl Into<String>,
        detail: impl Into<String>,
        review: bool,
    ) {
        if self.action_request.is_some() || self.action.is_some() || self.clone_busy() {
            return;
        }
        let Some(s) = &self.snapshot else {
            return;
        };
        self.action = Some(PendingAction {
            action,
            title: title.into(),
            detail: detail.into(),
            review,
            fingerprint: s.fingerprint.clone(),
            repo: s.repo.id.clone(),
        });
        if s.repo.trusted && !review {
            let action = self.action.take().unwrap();
            self.run_action(action);
        }
    }
    pub(super) fn run_action(&mut self, action: PendingAction) {
        if action.review {
            self.action = Some(action);
            return;
        }
        let id = self.send(Work::Mutate {
            repo: action.repo.clone(),
            fingerprint: action.fingerprint.clone(),
            action: action.action.clone(),
        });
        self.action_request = self.pending.contains(&id).then_some(id);
        self.action = Some(action);
    }
    pub(super) fn action_dialogs(&mut self, ctx: &egui::Context) {
        if let Some(mut form) = self.form.take() {
            let mut submit = false;
            let mut cancel = false;
            let response = theme::modal(ctx, "native_action_form", form.kind.title(), 440., |ui| {
                ui.label(match &form.kind {
                    FormKind::Stash => "Stash message (optional)",
                    FormKind::TagAt(_) => "Tag name",
                    FormKind::Amend(_) => "Commit message",
                    _ => "Local branch name",
                });
                match &form.kind {
                    FormKind::BranchAt(target)
                    | FormKind::TagAt(target)
                    | FormKind::Amend(target) => {
                        ui.monospace(target);
                    }
                    FormKind::Rename(name) | FormKind::Track(name) => {
                        ui.label(name);
                    }
                    _ => {}
                }
                if matches!(&form.kind, FormKind::Amend(_)) {
                    ui.label("Rewrite the latest eligible local commit using the staged index. Unstaged files are preserved; a recovery ref is kept.");
                    ui.add(
                        TextEdit::multiline(&mut form.text)
                            .desired_rows(5)
                            .desired_width(420.),
                    );
                } else {
                    ui.add(TextEdit::singleline(&mut form.text).desired_width(360.));
                }
                ui.add_space(12.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    submit = theme::primary(
                        ui,
                        form.kind.title(),
                        form.kind == FormKind::Stash || !form.text.trim().is_empty(),
                    )
                    .clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            cancel |= response.should_close();
            if submit {
                let (action, detail) = match &form.kind {
                    FormKind::Branch => (
                        Action::CreateBranch {
                            name: form.text.clone(),
                        },
                        format!("Create and switch to {} from current HEAD.", form.text),
                    ),
                    FormKind::Stash => (
                        Action::Stash {
                            message: form.text.clone(),
                        },
                        "Save tracked and untracked changes in a stash.".into(),
                    ),
                    FormKind::BranchAt(target) => (
                        Action::CreateBranchAt {
                            name: form.text.clone(),
                            target: target.clone(),
                        },
                        format!(
                            "Create {} at {}. Current checkout stays in place.",
                            form.text,
                            short(target)
                        ),
                    ),
                    FormKind::TagAt(target) => (
                        Action::CreateTag {
                            name: form.text.clone(),
                            target: target.clone(),
                        },
                        format!("Create local tag {} at {}.", form.text, short(target)),
                    ),
                    FormKind::Rename(name) => (
                        Action::RenameBranch {
                            name: name.clone(),
                            new_name: form.text.clone(),
                        },
                        format!("Rename {name} to {}.", form.text),
                    ),
                    FormKind::Track(remote_branch) => (
                        Action::TrackBranch {
                            name: form.text.clone(),
                            remote_branch: remote_branch.clone(),
                        },
                        format!(
                            "Create and switch to {} tracking {remote_branch}.",
                            form.text
                        ),
                    ),
                    FormKind::Amend(head) => (
                        Action::Amend {
                            head: head.clone(),
                            message: form.text.clone(),
                        },
                        format!(
                            "Rewrite {} with the staged index and the message you edited. A recovery ref retains the previous commit. Published or merge commits are refused.",
                            short(head)
                        ),
                    ),
                };
                self.request_action(
                    action,
                    form.kind.title(),
                    detail,
                    matches!(
                        &form.kind,
                        FormKind::Stash | FormKind::Track(_) | FormKind::Amend(_)
                    ),
                );
                return; // Show any resulting review modal on the next frame.
            } else if !cancel {
                self.form = Some(form);
            }
        }
        if self.action_request.is_some() {
            return;
        }
        let Some(action) = self.action.as_ref() else {
            return;
        };
        let trusted = self.snapshot.as_ref().is_some_and(|s| s.repo.trusted);
        let title = if trusted {
            action.title.clone()
        } else {
            "Trust this repository?".into()
        };
        let detail = action.detail.clone();
        let repo = action.repo.clone();
        let mut apply = false;
        let mut cancel = false;
        let response = theme::modal(ctx, "native_action_review", &title, 440., |ui| {
            if !trusted {
                ui.label("Git hooks, filters and credential helpers can run local programs. Trust lasts until closing this repository or the app.");
                if let Some(s) = &self.snapshot {
                    ui.monospace(&s.repo.path);
                }
            } else {
                ui.label(detail);
            }
            ui.add_space(16.);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                apply = theme::primary(
                    ui,
                    if trusted {
                        title.trim_end_matches('…')
                    } else {
                        "Trust repository"
                    },
                    true,
                )
                .clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        cancel |= response.should_close();
        if cancel {
            self.action = None;
        } else if apply {
            if !trusted {
                let id = self.send(Work::Trust(repo));
                self.action_request = self.pending.contains(&id).then_some(id);
            } else if let Some(mut a) = self.action.take() {
                a.review = false;
                self.run_action(a);
            }
        }
    }
}
