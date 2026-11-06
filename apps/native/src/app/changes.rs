use super::*;

impl App {
    pub(super) fn changes(&mut self, ui: &mut egui::Ui) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        let busy = self.action_request.is_some() || self.action.is_some();
        if let Some(operation) = &snapshot.operation {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(icon::GIT_MERGE).color(theme::AMBER));
                ui.label(theme::emphasis(operation));
                ui.label(theme::caption(
                    "Resolve and stage files, then complete the operation.",
                ));
            });
            ui.separator();
        }
        let mut resolve = None;
        egui::SidePanel::left("changed_files")
            .default_width(240.)
            .width_range(180.0..=340.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(12.))
            .show_inside(ui, |ui| {
                resolve = self.change_files(ui, &snapshot, busy);
            });
        egui::SidePanel::right("commit_composer")
            .default_width(300.)
            .width_range(250.0..=420.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(14.))
            .show_inside(ui, |ui| {
                self.change_composer(ui, &snapshot, busy);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BASE).inner_margin(10.))
            .show_inside(ui, |ui| self.change_diff(ui, &snapshot));
        if let Some(path) = resolve {
            self.open_conflict(path);
        }
    }
    fn change_diff(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot) {
        let Some(path) = self.file.clone() else {
            ui.add_space(16.);
            ui.label(theme::emphasis("Select a file to review its changes"));
            let hint = if snapshot.operation.is_some() {
                "Open the merge editor for conflicted files, then review the staged result."
            } else {
                "Stage a whole file with its + button, or select individual changed lines in the diff."
            };
            ui.label(theme::caption(hint));
            return;
        };
        let area = if self.staged_view {
            "Staged"
        } else {
            "Working"
        };
        ui.label(theme::caption(format!("{area} · {path}")));
        if self.diff.is_empty() {
            return;
        }
        let Some(event) = self.patch_view.ui(
            ui,
            &self.diff,
            self.staged_view,
            snapshot.operation.as_deref(),
        ) else {
            return;
        };
        let hash = git::hash(&self.diff);
        let action = match event {
            crate::patch_view::Event::Hunk(index) => Action::Hunk {
                path: path.clone(),
                index,
                unstage: self.staged_view,
                patch_hash: hash,
            },
            crate::patch_view::Event::Lines(rows) => Action::Lines {
                path: path.clone(),
                rows,
                unstage: self.staged_view,
                patch_hash: hash,
            },
        };
        self.request_action(action, "Update staged selection", path, false);
    }
    fn change_files(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &Snapshot,
        busy: bool,
    ) -> Option<String> {
        let mut resolve = None;
        ScrollArea::vertical().show(ui, |ui| {
            for staged in [false, true] {
                let count = snapshot
                    .files
                    .iter()
                    .filter(|f| {
                        if staged {
                            !f.index.trim().is_empty() && f.index != "?" && !git::is_conflicted(f)
                        } else {
                            !f.worktree.trim().is_empty() || f.index == "?" || git::is_conflicted(f)
                        }
                    })
                    .count();
                ui.label(theme::emphasis(format!(
                    "{} ({count})",
                    if staged { "Staged" } else { "Unstaged" }
                )));
                for file in &snapshot.files {
                    let conflict = git::is_conflicted(file);
                    let in_section = if staged {
                        !file.index.trim().is_empty() && file.index != "?" && !conflict
                    } else {
                        !file.worktree.trim().is_empty() || file.index == "?" || conflict
                    };
                    if !in_section {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [ui.available_width() - 34., 28.],
                                egui::Button::selectable(
                                    self.file.as_deref() == Some(&file.path)
                                        && self.staged_view == staged,
                                    format!(
                                        "{}  {}",
                                        if conflict {
                                            icon::WARNING_CIRCLE
                                        } else {
                                            icon::FILE_TEXT
                                        },
                                        file.path
                                    ),
                                )
                                .right_text(if conflict {
                                    "!"
                                } else if staged {
                                    file.index.trim()
                                } else if file.index == "?" {
                                    "U"
                                } else {
                                    file.worktree.trim()
                                })
                                .truncate(),
                            )
                            .on_hover_text(&file.path)
                            .clicked()
                        {
                            self.file = Some(file.path.clone());
                            self.staged_view = staged;
                            self.diff.clear();
                            self.diff_request = self.send(Work::Diff {
                                repo: snapshot.repo.id.clone(),
                                oid: None,
                                path: file.path.clone(),
                                staged,
                            });
                        }
                        if !conflict
                            && ui
                                .add_enabled(
                                    !busy,
                                    egui::Button::new(if staged {
                                        icon::MINUS
                                    } else {
                                        icon::PLUS
                                    })
                                    .small(),
                                )
                                .on_hover_text(if staged { "Unstage file" } else { "Stage file" })
                                .clicked()
                        {
                            let paths = vec![file.path.clone()];
                            self.request_action(
                                if staged {
                                    Action::Unstage { paths }
                                } else {
                                    Action::Stage { paths }
                                },
                                if staged { "Unstage file" } else { "Stage file" },
                                file.path.clone(),
                                false,
                            );
                        }
                    });
                    if conflict
                        && ui
                            .add_enabled(!busy, egui::Button::new("Open merge editor").small())
                            .clicked()
                    {
                        resolve = Some(file.path.clone());
                    }
                }
                ui.add_space(12.);
                ui.separator();
            }
            if snapshot.files.is_empty() {
                ui.label(theme::caption(if snapshot.operation.is_some() {
                    "No unresolved files. Complete or abort the operation."
                } else {
                    "Working tree is clean."
                }));
            }
        });
        resolve
    }
    fn change_composer(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot, busy: bool) {
        egui::TopBottomPanel::bottom("commit_actions")
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| self.change_commit_actions(ui, snapshot, busy));
        ui.heading("Commit");
        ui.label(theme::caption("Only staged changes are included."));
        ui.add_space(8.);
        if ui
            .add_enabled(
                self.composer.ready,
                TextEdit::multiline(&mut self.composer.text)
                    .hint_text("Describe your staged changes")
                    .desired_rows(8)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            self.composer.changed();
        }
        self.composer_status(ui);
    }
    fn change_commit_actions(&mut self, ui: &mut egui::Ui, snapshot: &Snapshot, busy: bool) {
        ui.vertical(|ui| {
            let conflicts = snapshot.files.iter().any(git::is_conflicted);
            let staged = snapshot.files.iter().any(|f| !f.index.trim().is_empty() && f.index != "?");
            let merge = snapshot.operation.as_deref() == Some("Merge in progress");
            let can_commit = !busy && !conflicts && (snapshot.operation.is_none() || merge)
                && (staged || merge) && !self.composer.text.trim().is_empty();
            let label = if merge { "Complete merge" } else { "Commit staged changes" };
            if theme::primary(ui, label, can_commit).clicked() {
                self.request_action(Action::Commit { message: self.composer.text.clone() },
                    "Commit staged changes", "Commit only the index using this message.", false);
            }
            if merge && ui.add_enabled(!busy, egui::Button::new("Abort merge…")).clicked() {
                self.request_action(Action::AbortMerge, "Abort merge",
                    "Abort the merge and restore the pre-merge state. Review local work first.", true);
            }
            if conflicts {
                ui.label(theme::caption("Resolve each conflicted file before committing."));
            }
            let sequence = match snapshot.operation.as_deref() {
                Some("Cherry-pick in progress") => Some(git::SequenceKind::CherryPick),
                Some("Revert in progress") => Some(git::SequenceKind::Revert),
                _ => None,
            };
            if let Some(operation) = sequence {
                let ready = !busy && !conflicts && !snapshot.files.iter().any(|f| f.index != "?" && f.worktree != " ");
                for (label, step, enabled) in [
                    ("Continue operation", git::SequenceStep::Continue, ready),
                    ("Skip current commit…", git::SequenceStep::Skip, !busy),
                    ("Abort operation…", git::SequenceStep::Abort, !busy),
                ] {
                    if ui.add_enabled(enabled, egui::Button::new(label))
                        .on_disabled_hover_text("Resolve and stage tracked changes before continuing.").clicked() {
                        let detail = match step {
                            git::SequenceStep::Continue => "Continue the reviewed sequence using the staged resolution.",
                            git::SequenceStep::Skip => "Skip the current commit and discard its tracked resolution changes; the next step can conflict.",
                            git::SequenceStep::Abort => "Abort the whole sequence and restore its starting state. Review local work first.",
                        };
                        self.request_action(Action::Sequence { operation, step }, label, detail, step != git::SequenceStep::Continue);
                    }
                }
            }
        });
    }
}
