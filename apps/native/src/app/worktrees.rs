//! Linked checkout navigation and guarded worktree operations reuse the core.
use super::*;
use ramo_core::git::WorktreeParent;

#[derive(Clone)]
pub(super) enum OpenTarget {
    Folder(Option<PathBuf>),
    Worktree { repo: String, path: String },
}
impl OpenTarget {
    pub(super) fn work(self) -> Work {
        match self {
            Self::Folder(path) => Work::Open(path),
            Self::Worktree { repo, path } => Work::OpenWorktree { repo, path },
        }
    }
}
#[derive(Default)]
pub(super) struct CreateDialog {
    parent: Option<WorktreeParent>,
    directory: String,
    branch: String,
    existing: bool,
    pub request: Option<u64>,
    error: Option<String>,
}
impl CreateDialog {
    pub(super) fn receive(&mut self, result: Result<Payload, String>) {
        self.request = None;
        match result {
            Ok(Payload::WorktreeParent(parent)) => {
                self.parent = Some(parent);
                self.error = None;
            }
            Ok(Payload::Cancelled) => {}
            Err(error) => self.error = Some(error),
            _ => self.error = Some("Unexpected folder selection response.".into()),
        }
    }
    fn action(&self) -> Option<Action> {
        if self.request.is_some()
            || self.directory.trim().is_empty()
            || self.branch.trim().is_empty()
        {
            return None;
        }
        Some(Action::CreateWorktree {
            selection_id: self.parent.as_ref()?.id.clone(),
            directory: self.directory.clone(),
            branch: self.branch.clone(),
            create_branch: !self.existing,
        })
    }
}
impl App {
    fn ask_worktree(&mut self, repo: String, path: String) {
        let target = OpenTarget::Worktree { repo, path };
        if self
            .editor
            .as_ref()
            .is_some_and(|e| e.dirty || e.draft_busy)
        {
            self.editor_exit = Some(EditorExit::Repository(target));
        } else {
            self.open_target(target);
        }
    }
    pub(super) fn worktree_list(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot) {
        let mut open = None;
        ui.horizontal(|ui| {
            ui.label(theme::emphasis(format!(
                "Worktrees   {}",
                snapshot.worktrees.len()
            )));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(
                        snapshot.head.is_some(),
                        egui::Button::new(icon::PLUS).frame(false),
                    )
                    .on_hover_text("Create worktree")
                    .clicked()
                {
                    self.worktree_dialog = Some(Default::default());
                }
            });
        });
        for tree in &snapshot.worktrees {
            let name = std::path::Path::new(&tree.path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let selected = tree.path == snapshot.repo.path;
            let branch = tree
                .branch
                .strip_prefix("refs/heads/")
                .unwrap_or(&tree.branch);
            let label = format!("{}  {name}", icon::FOLDER_OPEN);
            let response = ui.add_sized(
                [ui.available_width(), 28.],
                egui::Button::selectable(selected, label)
                    .right_text(if selected {
                        icon::CHECK
                    } else if tree.locked {
                        icon::LOCK
                    } else {
                        ""
                    })
                    .truncate(),
            );
            if response.clicked() && !selected {
                open = Some(tree.path.clone());
            }
            response.context_menu(|ui| {
                if ui.add_enabled(!selected, egui::Button::new("Open worktree")).clicked() {
                    open = Some(tree.path.clone()); ui.close();
                }
                if !tree.primary {
                    ui.separator();
                    let label = if tree.locked {"Unlock worktree…"} else {"Lock worktree…"};
                    if ui.button(label).clicked() {
                        self.request_action(Action::WorktreeLock { path: tree.path.clone(), locked: !tree.locked }, label,
                            format!("{} {}. This changes Git's worktree lock; files and branch stay in place.", if tree.locked {"Unlock"} else {"Lock"},name), true);
                        ui.close();
                    }
                    if ui.add_enabled(!selected && !tree.locked, egui::Button::new("Remove worktree…")).clicked() {
                        self.request_action(Action::RemoveWorktree { path: tree.path.clone() },"Remove worktree",
                            format!("Remove checkout {name} at {}. Its branch is kept. Modified, untracked or ignored data and active Git operations block removal.",tree.path),true);
                        ui.close();
                    }
                }
                if ui.button("Copy worktree path").clicked() {ui.ctx().copy_text(tree.path.clone());ui.close();}
            });
            response.on_hover_text(format!(
                "{}\n{}\n{}",
                tree.path,
                branch,
                if selected {
                    "Current checkout"
                } else {
                    "Click to open · right-click for actions"
                }
            ));
            ui.label(theme::caption(format!(
                "    {branch}{}",
                if tree.locked { " · Locked" } else { "" }
            )));
            ui.add_space(4.);
        }
        if let Some(path) = open {
            self.ask_worktree(snapshot.repo.id.clone(), path);
        }
    }
    pub(super) fn worktree_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.worktree_dialog.take() else {
            return;
        };
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let repo = snapshot.repo.id.clone();
        let branches: Vec<_> = snapshot
            .branches
            .iter()
            .filter(|b| {
                !b.remote
                    && !snapshot
                        .worktrees
                        .iter()
                        .any(|w| w.branch == b.name || w.branch == format!("refs/heads/{}", b.name))
            })
            .map(|b| b.name.clone())
            .collect();
        let mut pick = false;
        let mut submit = false;
        let mut cancel = false;
        let response = theme::modal(ctx, "create_worktree", "Create worktree", 480., |ui| {
            ui.label(theme::caption(
                "A separate checkout for another task. Your current changes stay here.",
            ));
            ui.add_space(12.);
            ui.label("Location");
            ui.horizontal(|ui| {
                let label = dialog
                    .parent
                    .as_ref()
                    .map_or("Choose folder…", |p| p.path.as_str());
                if ui
                    .add_enabled(
                        dialog.request.is_none(),
                        egui::Button::new(format!("{}  {label}", icon::FOLDER_OPEN))
                            .truncate()
                            .min_size(egui::vec2(440., 28.)),
                    )
                    .clicked()
                {
                    pick = true;
                }
            });
            ui.label(theme::caption(
                "Choose a parent folder outside your checkouts.",
            ));
            ui.add_space(8.);
            ui.label("New folder name");
            ui.add(
                TextEdit::singleline(&mut dialog.directory)
                    .hint_text("search-work")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.radio_value(&mut dialog.existing, false, "New branch");
                ui.radio_value(&mut dialog.existing, true, "Existing branch");
            });
            if dialog.existing {
                egui::ComboBox::from_id_salt("worktree_branch")
                    .selected_text(if branches.contains(&dialog.branch) {
                        &dialog.branch
                    } else {
                        "Choose branch"
                    })
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for name in &branches {
                            ui.selectable_value(&mut dialog.branch, name.clone(), name);
                        }
                    });
                ui.label(theme::caption("Branches already checked out are excluded."));
            } else {
                ui.add(
                    TextEdit::singleline(&mut dialog.branch)
                        .hint_text("work/search")
                        .desired_width(f32::INFINITY),
                );
                ui.label(theme::caption("The new branch starts at the current HEAD."));
            }
            if let Some(parent) = &dialog.parent {
                ui.add_space(8.);
                ui.label(theme::caption(format!(
                    "Destination: {}",
                    std::path::Path::new(&parent.path)
                        .join(&dialog.directory)
                        .display()
                )));
            }
            if dialog.request.is_some() {
                ui.spinner();
            }
            if let Some(error) = &dialog.error {
                ui.colored_label(theme::AMBER, error);
            }
            ui.add_space(16.);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                submit = theme::primary(
                    ui,
                    "Review creation",
                    dialog.action().is_some()
                        && (!dialog.existing || branches.contains(&dialog.branch)),
                )
                .clicked();
                cancel = ui
                    .add_enabled(dialog.request.is_none(), egui::Button::new("Cancel"))
                    .clicked();
            });
        });
        if pick {
            dialog.error = None;
            let id = self.send(Work::WorktreeParent { repo, path: None });
            if self.pending.contains(&id) {
                dialog.request = Some(id);
            }
        }
        if submit && let Some(action) = dialog.action() {
            let destination =
                std::path::Path::new(&dialog.parent.as_ref().unwrap().path).join(&dialog.directory);
            self.request_action(
                action,
                "Create worktree",
                format!(
                    "Create {} in {}. {} branch {}; current files and index remain here.",
                    dialog.directory,
                    destination.display(),
                    if dialog.existing {
                        "Use existing"
                    } else {
                        "Create new"
                    },
                    dialog.branch
                ),
                true,
            );
        } else if !cancel && (!response.should_close() || dialog.request.is_some()) {
            self.worktree_dialog = Some(dialog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creation_needs_selected_parent_and_idle_request() {
        let mut d = CreateDialog {
            directory: "checkout".into(),
            branch: "topic".into(),
            ..Default::default()
        };
        assert!(d.action().is_none());
        d.receive(Ok(Payload::WorktreeParent(WorktreeParent {
            id: "grant".into(),
            path: "synthetic".into(),
        })));
        assert!(
            matches!(d.action(),Some(Action::CreateWorktree{selection_id,create_branch:true,..}) if selection_id=="grant")
        );
        d.request = Some(8);
        assert!(d.action().is_none());
        d.receive(Ok(Payload::Cancelled));
        assert!(d.action().is_some());
        d.existing = true;
        assert!(matches!(
            d.action(),
            Some(Action::CreateWorktree {
                create_branch: false,
                ..
            })
        ));
    }
}
