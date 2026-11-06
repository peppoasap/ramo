use super::*;

impl App {
    pub(super) fn inspector(&mut self, ctx: &egui::Context) {
        if !self.inspector || self.mode != Mode::History || self.snapshot.is_none() {
            return;
        }
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(420.)
            .width_range(260.0..=600.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(14.))
            .show(ctx, |ui| {
                if self.modal_open() || self.action_request.is_some() {
                    ui.disable();
                }
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
                    ui.label(theme::caption(commit.date.replace('T', "  ")))
                        .on_hover_text(&commit.date);
                } else if let Some(oid) = &self.selected {
                    ui.monospace(short(oid));
                }
                ui.add_space(10.);
                ui.separator();
                ui.label(theme::emphasis(format!(
                    "Changed files ({})",
                    self.files.len()
                )));
                let files = self.files.clone();
                ScrollArea::vertical()
                    .id_salt("commit_files")
                    .max_height(160.)
                    .show(ui, |ui| {
                        for file in files {
                            if ui
                                .add(
                                    egui::Button::selectable(
                                        self.file.as_deref() == Some(&file.path),
                                        format!("{}  {}", icon::FILE_TEXT, file.path),
                                    )
                                    .min_size(egui::vec2(ui.available_width(), 28.))
                                    .truncate(),
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
}
