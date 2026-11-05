use super::*;

impl App {
    pub(super) fn inspector(&mut self, ctx: &egui::Context) {
        if !self.inspector || self.mode != Mode::History || self.snapshot.is_none() {
            return;
        }
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(340.)
            .width_range(260.0..=600.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(14.))
            .show(ctx, |ui| {
                let commit = self
                    .selected
                    .as_ref()
                    .and_then(|oid| self.commits.iter().find(|c| &c.oid == oid));
                ui.heading(commit.map_or("Commit details", |c| c.subject.as_str()));
                if let Some(commit) = commit {
                    ui.add_space(8.);
                    ui.label(theme::caption(format!(
                        "{} · {}",
                        short(&commit.oid),
                        commit.author
                    )));
                    ui.label(theme::caption(&commit.date));
                } else if let Some(oid) = &self.selected {
                    ui.monospace(short(oid));
                }
                ui.add_space(10.);
                ui.separator();
                ui.strong(format!("Changed files ({})", self.files.len()));
                let files = self.files.clone();
                ScrollArea::vertical()
                    .id_salt("commit_files")
                    .max_height(160.)
                    .show(ui, |ui| {
                        for file in files {
                            if ui
                                .selectable_label(
                                    self.file.as_deref() == Some(&file.path),
                                    format!("{}  {}", icon::FILE_TEXT, file.path),
                                )
                                .clicked()
                                && let Some(snapshot) = &self.snapshot
                            {
                                self.file = Some(file.path.clone());
                                self.diff.clear();
                                self.diff_request = self.send(Work::Diff {
                                    repo: snapshot.repo.id.clone(),
                                    oid: self.selected.clone(),
                                    path: file.path,
                                    staged: false,
                                });
                            }
                        }
                    });
                ui.separator();
                if let Some(path) = &self.file {
                    ui.strong(path);
                }
                if self.files.is_empty() && !self.pending.contains(&self.detail_request) {
                    ui.label(theme::caption(
                        "No first-parent file changes in this commit.",
                    ));
                }
                render_diff(ui, &self.diff);
            });
    }
    pub(super) fn changes(&mut self, ui: &mut egui::Ui) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        ui.label(theme::caption("Browse working changes. Git staging and commits remain in the released beta during this migration slice."));
        ui.separator();
        let mut resolve = None;
        egui::SidePanel::left("changed_files")
            .default_width(260.)
            .resizable(true)
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for file in &snapshot.files {
                        let conflict = git::is_conflicted(file);
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    self.file.as_deref() == Some(&file.path),
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
                                .clicked()
                            {
                                self.file = Some(file.path.clone());
                                self.diff.clear();
                                self.diff_request = self.send(Work::Diff {
                                    repo: snapshot.repo.id.clone(),
                                    oid: None,
                                    path: file.path.clone(),
                                    staged: !conflict
                                        && file.worktree.trim().is_empty()
                                        && !file.index.trim().is_empty(),
                                });
                            }
                        });
                        if conflict && ui.small_button("Open merge editor").clicked() {
                            resolve = Some(file.path.clone());
                        }
                    }
                    if snapshot.files.is_empty() {
                        ui.label(theme::caption("Working tree is clean."));
                    }
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                render_diff(ui, &self.diff);
            });
        if let Some(path) = resolve {
            self.editor_request = self.send(Work::Conflict {
                repo: snapshot.repo.id,
                fingerprint: snapshot.fingerprint,
                path,
            });
        }
    }
}
