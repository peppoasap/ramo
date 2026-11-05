use super::*;

pub(super) struct PendingAction {
    pub action: Action,
    title: String,
    detail: String,
    review: bool,
    fingerprint: String,
    repo: String,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum FormKind {
    Branch,
    Stash,
}
impl FormKind {
    fn title(self) -> &'static str {
        match self {
            Self::Branch => "Create branch",
            Self::Stash => "Save stash",
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
        if self.action_request.is_some() || self.action.is_some() {
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
        self.action_request = Some(self.send(Work::Mutate {
            repo: action.repo.clone(),
            fingerprint: action.fingerprint.clone(),
            action: action.action.clone(),
        }));
        self.action = Some(action);
    }
    pub(super) fn action_dialogs(&mut self, ctx: &egui::Context) {
        if let Some(mut form) = self.form.take() {
            let mut submit = false;
            let mut cancel = false;
            egui::Window::new(form.kind.title())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                .show(ctx, |ui| {
                    ui.label(if form.kind == FormKind::Branch {
                        "Branch name"
                    } else {
                        "Stash message (optional)"
                    });
                    ui.add(TextEdit::singleline(&mut form.text).desired_width(360.));
                    ui.horizontal(|ui| {
                        cancel = ui.button("Cancel").clicked();
                        submit = ui
                            .add_enabled(
                                form.kind != FormKind::Branch || !form.text.trim().is_empty(),
                                egui::Button::new(form.kind.title()),
                            )
                            .clicked();
                    });
                });
            if submit {
                let (action, detail) = if form.kind == FormKind::Branch {
                    (
                        Action::CreateBranch {
                            name: form.text.clone(),
                        },
                        format!("Create and switch to {} from the current HEAD.", form.text),
                    )
                } else {
                    (
                        Action::Stash {
                            message: form.text.clone(),
                        },
                        "Save tracked and untracked changes in a stash.".into(),
                    )
                };
                self.request_action(
                    action,
                    form.kind.title(),
                    detail,
                    form.kind != FormKind::Branch,
                );
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
        egui::Window::new(title).collapsible(false).resizable(false).default_width(440.).anchor(egui::Align2::CENTER_CENTER,[0.,0.]).show(ctx,|ui|{
            if !trusted {
                ui.label("Git hooks, filters and credential helpers can run local programs. Trust lasts until closing this repository or the app.");
                if let Some(s)=&self.snapshot {ui.monospace(&s.repo.path);}
            }else{ui.label(detail);}
            ui.add_space(12.);
            ui.horizontal(|ui|{cancel=ui.button("Cancel").clicked();apply=ui.button(if trusted {"Continue"}else{"Trust repository"}).clicked();});
        });
        if cancel {
            self.action = None;
        } else if apply {
            if !trusted {
                self.action_request = Some(self.send(Work::Trust(repo)));
            } else if let Some(mut a) = self.action.take() {
                a.review = false;
                self.run_action(a);
            }
        }
    }
}
