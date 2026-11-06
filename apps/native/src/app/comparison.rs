//! Read-only comparison of immutable commit pairs, separate from working changes.
use super::*;
use ramo_core::compare::{Comparison, Kind};

#[derive(Default)]
pub(super) struct CompareView {
    from: String,
    to: String,
    ancestor: bool,
    result: Option<Comparison>,
    path: Option<String>,
    diff: String,
    error: Option<String>,
    request: Option<u64>,
    file_request: Option<u64>,
}
impl CompareView {
    pub(super) fn cancel_requests(&mut self) {
        self.request = None;
        self.file_request = None;
    }
    fn invalidate(&mut self) {
        self.result = None;
        self.path = None;
        self.diff.clear();
        self.error = None;
        self.request = None;
        self.file_request = None;
    }
    pub(super) fn receive(&mut self, id: u64, result: Result<Payload, String>) {
        if self.request == Some(id) {
            self.request = None;
            match result {
                Ok(Payload::Compared(Ok(value))) => self.result = Some(value),
                Ok(Payload::Compared(Err(error))) => self.error = Some(error),
                Err(error) => self.error = Some(error),
                _ => self.error = Some("Unexpected comparison response.".into()),
            }
        } else if self.file_request == Some(id) {
            self.file_request = None;
            match result {
                Ok(Payload::ComparisonDiff(Ok(diff))) => self.diff = diff,
                Ok(Payload::ComparisonDiff(Err(error))) => self.error = Some(error),
                Err(error) => self.error = Some(error),
                _ => self.error = Some("Unexpected comparison diff response.".into()),
            }
        }
    }
    pub(super) fn owns(&self, id: u64) -> bool {
        self.request == Some(id) || self.file_request == Some(id)
    }
}
impl App {
    pub(super) fn compare_endpoint(&mut self, from: bool, oid: String) {
        if from {
            self.comparison.from = oid;
        } else {
            self.comparison.to = oid;
        }
        self.comparison.invalidate();
        self.mode = Mode::Compare;
    }
    pub(super) fn compare_pair(&mut self, from: String, to: String) {
        self.compare_endpoint(true, from);
        self.compare_endpoint(false, to);
        self.comparison.ancestor = false;
        self.run_comparison();
    }
    pub(super) fn run_comparison(&mut self) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let repo = snapshot.repo.id.clone();
        self.comparison.invalidate();
        let id = self.send(Work::Compare {
            repo,
            from: self.comparison.from.trim().into(),
            to: self.comparison.to.trim().into(),
            kind: if self.comparison.ancestor {
                Kind::MergeBase
            } else {
                Kind::Trees
            },
        });
        if self.pending.contains(&id) {
            self.comparison.request = Some(id);
        }
    }
    pub(super) fn comparison(&mut self, ui: &mut egui::Ui) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let repo = snapshot.repo.id.clone();
        ui.heading("Compare revisions");
        ui.label(theme::caption(
            "Saved commits only. Your working changes and staged files stay separate.",
        ));
        let mut choices: Vec<(String, String)> = snapshot
            .branches
            .iter()
            .take(1000)
            .map(|b| (format!("{} · {}", b.name, short(&b.oid)), b.oid.clone()))
            .collect();
        choices.extend(
            self.commits
                .iter()
                .take(1000)
                .map(|c| (format!("{} · {}", short(&c.oid), c.subject), c.oid.clone())),
        );
        let mut changed = false;
        for (label, value) in [
            ("From", &mut self.comparison.from),
            ("To", &mut self.comparison.to),
        ] {
            ui.horizontal(|ui| {
                ui.label(label);
                changed |= ui
                    .add(
                        TextEdit::singleline(value)
                            .hint_text("Full commit ID or choose a revision")
                            .desired_width((ui.available_width() - 150.).max(180.)),
                    )
                    .changed();
                egui::ComboBox::from_id_salt(("compare_revision", label))
                    .selected_text("Choose revision")
                    .width(130.)
                    .show_ui(ui, |ui| {
                        // Loaded-history choices are bounded; full IDs also work outside this list.
                        for (name, oid) in choices.iter().take(1000) {
                            if ui.selectable_label(value == oid, name).clicked() {
                                value.clone_from(oid);
                                changed = true;
                                ui.close();
                            }
                        }
                    });
            });
        }
        ui.horizontal_wrapped(|ui| {
            changed |= ui
                .radio_value(&mut self.comparison.ancestor, false, "Direct comparison")
                .changed();
            changed |= ui
                .radio_value(&mut self.comparison.ancestor, true, "Since common ancestor")
                .changed();
            if theme::button(ui, icon::ARROWS_CLOCKWISE, "Swap").clicked() {
                std::mem::swap(&mut self.comparison.from, &mut self.comparison.to);
                changed = true;
            }
        });
        if changed {
            self.comparison.invalidate();
        }
        let enabled =
            !self.comparison.from.trim().is_empty() && !self.comparison.to.trim().is_empty();
        ui.horizontal(|ui| {
            if theme::primary(ui, "Compare", enabled).clicked() {
                self.run_comparison();
            }
            if self.comparison.request.is_some() {
                ui.spinner();
                ui.label("Comparing saved revisions…");
            }
        });
        if let Some(error) = &self.comparison.error {
            ui.colored_label(theme::AMBER, error);
        }
        ui.separator();
        let Some(result) = &self.comparison.result else {
            ui.label(theme::caption("Choose From and To, then Compare. Right-click a commit or branch to set an endpoint."));
            return;
        };
        ui.label(theme::caption(format!(
            "{} to {} · {} files · {}",
            short(&result.from),
            short(&result.to),
            result.files.len(),
            if matches!(result.kind, Kind::Trees) {
                "Direct"
            } else {
                "Since common ancestor"
            }
        )));
        ui.horizontal_wrapped(|ui| {
            ui.label("Effective base");
            ui.monospace(short(&result.effective_from))
                .on_hover_text(&result.effective_from);
            ui.label("Target");
            ui.monospace(short(&result.to)).on_hover_text(&result.to);
        });
        let from = result.effective_from.clone();
        let to = result.to.clone();
        let files = result.files.clone();
        if files.is_empty() {
            ui.label("No saved-file differences between these revisions.");
            return;
        }
        let mut selected = None;
        egui::SidePanel::left("comparison_files")
            .default_width(245.)
            .resizable(true)
            .show_inside(ui, |ui| {
                ui.strong("Changed files");
                ScrollArea::vertical()
                    .id_salt("comparison_file_list")
                    .show(ui, |ui| {
                        for file in &files {
                            if ui
                                .selectable_label(
                                    self.comparison.path.as_ref() == Some(&file.path),
                                    format!("{}  {}", file.index, file.path),
                                )
                                .clicked()
                            {
                                selected = Some(file.path.clone());
                            }
                        }
                    });
            });
        if let Some(path) = selected {
            self.comparison.path = Some(path.clone());
            self.comparison.diff.clear();
            self.comparison.error = None;
            self.comparison.file_request = None;
            let id = self.send(Work::ComparisonDiff {
                repo,
                from,
                to,
                path,
            });
            if self.pending.contains(&id) {
                self.comparison.file_request = Some(id);
            }
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                if let Some(path) = &self.comparison.path {
                    ui.strong(path);
                } else {
                    ui.label(theme::caption("Select a changed file to inspect its diff."));
                }
                if self.comparison.file_request.is_some() {
                    ui.spinner();
                }
                render_diff(ui, &self.comparison.diff);
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_pairs_and_file_selections_reject_late_results() {
        let mut view = CompareView {
            request: Some(7),
            ..Default::default()
        };
        view.invalidate();
        view.receive(7, Err("stale error".into()));
        assert!(view.error.is_none());
        view.file_request = Some(10);
        view.receive(9, Ok(Payload::ComparisonDiff(Ok("old file".into()))));
        assert!(view.diff.is_empty());
        view.receive(10, Ok(Payload::ComparisonDiff(Ok("selected file".into()))));
        assert_eq!(view.diff, "selected file");
        assert!(!view.owns(10));
    }
}
