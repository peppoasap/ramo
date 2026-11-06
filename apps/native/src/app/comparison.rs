//! Read-only comparison of immutable commit pairs, separate from working changes.
use super::*;
use ramo_core::compare::{Comparison, Kind};

#[derive(Default)]
pub(super) struct CompareView {
    from: String,
    to: String,
    ancestor: bool,
    labels: [Option<(String, String)>; 2],
    filters: [String; 2],
    result: Option<Comparison>,
    path: Option<String>,
    diff: String,
    error: Option<String>,
    request: Option<u64>,
    file_request: Option<u64>,
}
impl CompareView {
    pub(super) fn selection(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            ancestor: self.ancestor,
            labels: self.labels.clone(),
            filters: self.filters.clone(),
            ..Default::default()
        }
    }
    pub(super) fn has_pair(&self) -> bool {
        !self.from.is_empty() && !self.to.is_empty()
    }
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
    pub(super) fn initial_file(&self) -> Option<String> {
        if self.path.is_some() || self.error.is_some() {
            return None;
        }
        self.result.as_ref()?.files.first().map(|f| f.path.clone())
    }
    pub(super) fn owns(&self, id: u64) -> bool {
        self.request == Some(id) || self.file_request == Some(id)
    }
}
impl App {
    pub(super) fn compare_endpoint(&mut self, from: bool, oid: String) {
        if from {
            self.comparison.from = oid;
            self.comparison.labels[0] = None;
        } else {
            self.comparison.to = oid;
            self.comparison.labels[1] = None;
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
        let mut choices: Vec<Choice> = snapshot
            .branches
            .iter()
            .take(1000)
            .map(|b| Choice {
                name: b.name.clone(),
                oid: b.oid.clone(),
                branch: true,
            })
            .collect();
        choices.extend(self.commits.iter().take(1000).map(|c| Choice {
            name: c.subject.clone(),
            oid: c.oid.clone(),
            branch: false,
        }));
        let mut changed = false;
        let mut run_requested = false;
        egui::Frame::new().fill(theme::PANEL).inner_margin(12.).show(ui, |ui| {
            ui.columns(2, |columns| {
                for (index, column) in columns.iter_mut().enumerate() {
                    let verified = self.comparison.result.as_ref().is_some_and(|result| if index == 0 {result.from == self.comparison.from} else {result.to == self.comparison.to});
                    let value = if index == 0 { &mut self.comparison.from } else { &mut self.comparison.to };
                    changed |= endpoint(column, index, value, &mut self.comparison.labels[index],
                        &mut self.comparison.filters[index], &choices, verified);
                }
            });
            ui.add_space(8.);
            ui.horizontal_wrapped(|ui| {
                changed |= ui.radio_value(&mut self.comparison.ancestor, false, "Direct").on_hover_text("Compare the two selected saved file trees.").changed();
                changed |= ui.radio_value(&mut self.comparison.ancestor, true, "Since common ancestor").on_hover_text("Compare the unique common ancestor with the target. Multiple ancestors are refused.").changed();
                if theme::button(ui, icon::SWAP, "Swap").clicked() {
                    std::mem::swap(&mut self.comparison.from, &mut self.comparison.to);
                    self.comparison.labels.swap(0, 1);
                    changed = true;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let enabled = !self.comparison.from.trim().is_empty() && !self.comparison.to.trim().is_empty() && self.comparison.request.is_none();
                    if theme::primary(ui,"Compare",enabled).clicked() { run_requested = true; }
                });
            });
            ui.collapsing("Paste commit IDs", |ui| {
                ui.label(theme::caption("Full commit IDs also work outside the loaded history. These stay pinned when branches move."));
                for (index, (label, value)) in [("From", &mut self.comparison.from), ("To", &mut self.comparison.to)].into_iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(label);
                        if ui.add(TextEdit::singleline(value).desired_width(ui.available_width()).hint_text("Full commit ID")).changed() {
                            self.comparison.labels[index] = None;
                            changed = true;
                        }
                    });
                }
            });
        });
        if changed {
            self.comparison.invalidate();
        }
        if run_requested {
            self.run_comparison();
        }
        ui.add_space(6.);
        ui.horizontal_wrapped(|ui| {
            ui.label(theme::caption(
                "Saved commits only · local changes are excluded",
            ));
            if self.comparison.request.is_some() {
                ui.spinner();
                ui.label(theme::caption("Comparing…"));
            }
        });
        if let Some(error) = &self.comparison.error {
            ui.colored_label(theme::AMBER, error);
        }
        let Some(result) = &self.comparison.result else {
            ui.add_space(16.);
            ui.label(theme::emphasis("Choose two revisions to compare"));
            ui.label(theme::caption("Search branches or loaded commits above. You can also set endpoints from the History context menu or drag one commit onto another."));
            return;
        };
        let files = result.files.clone();
        ui.horizontal_wrapped(|ui| {
            ui.label(theme::emphasis(format!(
                "{} changed {}",
                files.len(),
                if files.len() == 1 { "file" } else { "files" }
            )));
            ui.label(theme::caption(if matches!(result.kind, Kind::Trees) {
                "Direct comparison"
            } else {
                "Since common ancestor"
            }));
            ui.label(theme::caption(format!(
                "{} {} {}",
                short(&result.effective_from),
                icon::ARROW_RIGHT,
                short(&result.to)
            )))
            .on_hover_text(format!(
                "Effective base: {}\nTarget: {}",
                result.effective_from, result.to
            ));
        });
        ui.separator();
        if files.is_empty() {
            ui.add_space(16.);
            ui.label(
                RichText::new(icon::CHECK_CIRCLE)
                    .size(24.)
                    .color(theme::MINT),
            );
            ui.label(theme::emphasis("These revisions have the same files"));
            ui.label(theme::caption(
                "There are no saved-file differences for this comparison.",
            ));
            return;
        }
        let mut selected = None;
        egui::SidePanel::left("comparison_files")
            .default_width(240.)
            .width_range(180.0..=360.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(12.))
            .show_inside(ui, |ui| {
                ui.label(theme::emphasis("Changed files"));
                ui.add_space(8.);
                ScrollArea::vertical()
                    .id_salt("comparison_file_list")
                    .show(ui, |ui| {
                        for file in &files {
                            if ui
                                .add_sized(
                                    [ui.available_width(), 28.],
                                    egui::Button::selectable(
                                        self.comparison.path.as_ref() == Some(&file.path),
                                        format!("{}  {}", icon::FILE_TEXT, file.path),
                                    )
                                    .right_text(&file.index)
                                    .truncate(),
                                )
                                .on_hover_text(&file.path)
                                .clicked()
                            {
                                selected = Some(file.path.clone());
                            }
                        }
                    });
            });
        if let Some(path) = selected {
            self.select_comparison_file(path);
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BASE).inner_margin(10.))
            .show_inside(ui, |ui| {
                if let Some(path) = &self.comparison.path {
                    ui.label(theme::emphasis(path));
                } else {
                    ui.label(theme::caption("Select a changed file to inspect its diff."));
                }
                ui.add_space(6.);
                if self.comparison.file_request.is_some() {
                    ui.spinner();
                    ui.label(theme::caption("Loading diff…"));
                }
                render_diff(ui, &self.comparison.diff);
            });
    }
    pub(super) fn select_comparison_file(&mut self, path: String) {
        let (Some(snapshot), Some(result)) = (&self.snapshot, &self.comparison.result) else {
            return;
        };
        let work = Work::ComparisonDiff {
            repo: snapshot.repo.id.clone(),
            from: result.effective_from.clone(),
            to: result.to.clone(),
            path: path.clone(),
        };
        self.comparison.path = Some(path);
        self.comparison.diff.clear();
        self.comparison.error = None;
        self.comparison.file_request = None;
        let id = self.send(work);
        if self.pending.contains(&id) {
            self.comparison.file_request = Some(id);
        }
    }
}

struct Choice {
    name: String,
    oid: String,
    branch: bool,
}
fn endpoint_name<'a>(
    value: &str,
    label: &'a Option<(String, String)>,
    choices: &'a [Choice],
) -> &'a str {
    if value.is_empty() {
        return "Choose branch or commit";
    }
    if let Some((oid, name)) = label
        && oid == value
    {
        return name;
    }
    choices
        .iter()
        .find(|c| c.oid == value)
        .map_or("Custom commit", |c| c.name.as_str())
}
fn matches_choice(choice: &Choice, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    choice.name.to_lowercase().contains(&query) || choice.oid.to_lowercase().contains(&query)
}
fn endpoint(
    ui: &mut egui::Ui,
    index: usize,
    value: &mut String,
    label: &mut Option<(String, String)>,
    filter: &mut String,
    choices: &[Choice],
    verified: bool,
) -> bool {
    let mut changed = false;
    ui.label(theme::caption(if index == 0 { "From" } else { "To" }));
    let name = endpoint_name(value, label, choices).to_owned();
    let response = egui::ComboBox::from_id_salt(("native_endpoint", index))
        .selected_text(theme::emphasis(name))
        .width(ui.available_width())
        .height(300.)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .truncate()
        .show_ui(ui, |ui| {
            ui.set_min_width(300.);
            ui.add(
                TextEdit::singleline(filter)
                    .hint_text("Find branch, commit or hash")
                    .desired_width(f32::INFINITY),
            );
            ui.label(theme::caption(
                "Branches and loaded commits · up to 100 matches",
            ));
            ui.separator();
            let mut count = 0;
            for choice in choices
                .iter()
                .filter(|c| matches_choice(c, filter))
                .take(100)
            {
                count += 1;
                let glyph = if choice.branch {
                    icon::GIT_BRANCH
                } else {
                    icon::GIT_COMMIT
                };
                if ui
                    .selectable_label(
                        *value == choice.oid,
                        format!("{glyph}  {} · {}", choice.name, short(&choice.oid)),
                    )
                    .clicked()
                {
                    *value = choice.oid.clone();
                    *label = Some((choice.oid.clone(), choice.name.clone()));
                    changed = true;
                    ui.close();
                }
            }
            if count == 0 {
                ui.label(theme::caption(
                    "No loaded revision matches. Paste a full commit ID below the selectors.",
                ));
            }
        })
        .response;
    let loaded = verified
        || choices.iter().any(|c| c.oid == *value)
        || label.as_ref().is_some_and(|(oid, _)| oid == value);
    response.on_hover_text(if value.is_empty() {
        "Choose a saved commit".into()
    } else if loaded {
        format!("Pinned commit: {value}")
    } else {
        format!("Unvalidated commit ID: {value}")
    });
    ui.label(theme::caption(if value.is_empty() {
        "Nothing selected".into()
    } else if loaded {
        format!("Pinned commit {}", short(value))
    } else {
        "Custom ID · compare to validate".into()
    }));
    changed
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
    #[test]
    fn chosen_names_stay_bound_to_the_selected_commit_and_search_handles_names_and_ids() {
        let picked = Some(("old-object".into(), "feat/search".into()));
        let choices = vec![Choice {
            name: "feat/search".into(),
            oid: "new-object".into(),
            branch: true,
        }];
        assert_eq!(
            endpoint_name("old-object", &picked, &choices),
            "feat/search"
        );
        assert_eq!(
            endpoint_name("unlisted-object", &picked, &choices),
            "Custom commit"
        );
        assert!(matches_choice(&choices[0], "  SEARCH  "));
        assert!(matches_choice(&choices[0], "new-obj"));
        assert!(!matches_choice(&choices[0], "old-object"));
    }
    #[test]
    fn only_a_current_successful_comparison_can_offer_an_initial_file() {
        let mut view = CompareView {
            request: Some(4),
            ..Default::default()
        };
        let value = || Comparison {
            from: "left".into(),
            to: "right".into(),
            effective_from: "left".into(),
            kind: Kind::Trees,
            files: vec![FileChange {
                original: None,
                path: "src/search.rs".into(),
                index: "M".into(),
                worktree: " ".into(),
            }],
        };
        view.receive(4, Ok(Payload::Compared(Ok(value()))));
        assert_eq!(view.initial_file().as_deref(), Some("src/search.rs"));
        view.path = Some("src/search.rs".into());
        assert!(view.initial_file().is_none());
        view.invalidate();
        view.receive(4, Ok(Payload::Compared(Ok(value()))));
        assert!(view.result.is_none());
        assert!(view.initial_file().is_none());
        view.request = Some(5);
        view.receive(5, Ok(Payload::Compared(Err("invalid pair".into()))));
        assert!(view.initial_file().is_none());
    }
}
