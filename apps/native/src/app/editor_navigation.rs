//! Conflict navigation shares the same unsaved-result gate as Back/Open/Close.
use super::*;
#[derive(Clone)]
pub(super) enum EditorExit {
    Back,
    Open,
    Close,
    Conflict(String),
}
impl App {
    pub(super) fn open_conflict(&mut self, path: String) {
        if self.editor.as_ref().is_some_and(|e| e.scope.path == path) {
            return;
        }
        if self
            .editor
            .as_ref()
            .is_some_and(|e| e.dirty || e.draft_busy)
        {
            self.editor_exit = Some(EditorExit::Conflict(path));
        } else {
            self.load_conflict(path);
        }
    }
    fn load_conflict(&mut self, path: String) {
        if self.pending.contains(&self.editor_request) || self.action_request.is_some() {
            return;
        }
        if let Some(snapshot) = &self.snapshot {
            self.editor_request = self.send(Work::Conflict {
                repo: snapshot.repo.id.clone(),
                fingerprint: snapshot.fingerprint.clone(),
                path,
            });
        }
    }
    pub(super) fn conflict_sidebar(&mut self, ctx: &egui::Context) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        let current = self.editor.as_ref().map(|e| e.scope.path.clone());
        let mut open = None;
        egui::SidePanel::left("conflict_files")
            .resizable(true)
            .default_width(210.)
            .width_range(165.0..=340.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(12.))
            .show(ctx, |ui| {
                if self.modal_open()
                    || self.action_request.is_some()
                    || self.pending.contains(&self.editor_request)
                {
                    ui.disable();
                }
                let files: Vec<_> = snapshot
                    .files
                    .iter()
                    .filter(|f| git::is_conflicted(f))
                    .collect();
                ui.label(theme::emphasis(format!("Conflicts ({})", files.len())));
                ui.add_space(8.);
                ScrollArea::vertical().show(ui, |ui| {
                    for file in files {
                        let selected = current.as_deref() == Some(&file.path);
                        if ui
                            .add_sized(
                                [ui.available_width(), 30.],
                                egui::Button::selectable(
                                    selected,
                                    format!("{}  {}", icon::FILE_TEXT, file.path),
                                )
                                .right_text(if selected { icon::CHECK } else { "" })
                                .truncate(),
                            )
                            .on_hover_text(&file.path)
                            .clicked()
                        {
                            open = Some(file.path.clone());
                        }
                    }
                });
            });
        if let Some(path) = open {
            self.open_conflict(path);
        }
    }
    pub(super) fn editor_exit_dialog(&mut self, ctx: &egui::Context) {
        let Some(destination) = self.editor_exit.clone() else {
            return;
        };
        let saving = self.editor.as_ref().is_some_and(|e| e.draft_busy);
        let mut discard = false;
        let mut keep = false;
        let response = theme::modal(
            ctx,
            "editor_exit",
            "Keep your editor changes?",
            460.,
            |ui| {
                ui.label(if saving {"Wait for the local draft to finish saving before leaving this file."} else {"Unsaved result edits are still in memory. Return to the editor to save a local draft."});
                ui.add_space(12.);
                ui.horizontal_wrapped(|ui| {
                    keep = ui.button("Keep editing").clicked();
                    discard = ui
                        .add_enabled(!saving, egui::Button::new("Discard unsaved edits"))
                        .clicked();
                });
            },
        );
        if keep || response.should_close() {
            self.editor_exit = None;
        }
        if discard {
            self.editor_exit = None;
            self.editor = None;
            match destination {
                EditorExit::Back => {}
                EditorExit::Open => self.open(None),
                EditorExit::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                EditorExit::Conflict(path) => self.load_conflict(path),
            }
        }
    }
}
