use super::*;
impl App {
    pub(super) fn load_composer(&mut self) {
        if let Some(scope) = self.composer.scope() {
            let id = self.send(Work::CommitDraftLoad(scope));
            if self.pending.contains(&id) {
                self.composer.load_request = Some(id);
            } else {
                self.composer.loaded(Err(
                    "Draft queue is busy. Retry loading the saved message.".into()
                ));
            }
        }
    }
    pub(super) fn save_composer(&mut self) {
        if self.composer.pending() || !self.composer.ready || !self.composer.dirty() {
            return;
        }
        if let Some(scope) = self.composer.scope() {
            let id = self.send(Work::CommitDraftSave {
                scope,
                text: self.composer.text.clone(),
                expected: self.composer.revision(),
            });
            if self.pending.contains(&id) {
                self.composer.save_request = Some(id);
            } else {
                self.composer.error =
                    Some("Draft queue is busy. Your message is still in memory.".into());
            }
        }
    }
    pub(super) fn composer_tick(&mut self, ctx: &egui::Context) {
        if self.composer.reload_review {
            let mut load = false;
            let mut cancel = false;
            let response = theme::modal(
                ctx,
                "reload_commit_draft",
                "Load the saved message?",
                440.,
                |ui| {
                    ui.label("Loading replaces the unsaved composer text. Copy your current message first if you want to keep both versions.");
                    ui.horizontal(|ui| {
                        if ui.button("Copy current message").clicked() {
                            ui.ctx().copy_text(self.composer.text.clone());
                        }
                        cancel = ui.button("Cancel").clicked();
                        load = ui.button("Load saved copy").clicked();
                    });
                },
            );
            if cancel || response.should_close() {
                self.composer.reload_review = false;
            }
            if load {
                self.composer.reload_review = false;
                self.composer.ready = false;
                self.open_after_draft = None;
                self.close_after_draft = false;
                self.load_composer();
            }
        }

        if self.composer.due() {
            self.save_composer();
        }
        if self.composer.dirty() && self.composer.error.is_none() {
            ctx.request_repaint_after(std::time::Duration::from_millis(350));
        }
        if !self.composer.pending() && !self.composer.dirty() {
            if !self.close_after_draft
                && !self.editor.as_ref().is_some_and(|e| e.dirty)
                && let Some(path) = self.open_after_draft.take()
            {
                self.open_target(path);
            }
            if self.close_after_draft && self.action_request.is_none() {
                self.open_after_draft = None;
                self.close_after_draft = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
    pub(super) fn composer_status(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = self.composer.error.clone() {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(error).color(theme::AMBER));
                if ui.small_button("Retry save").clicked() {
                    self.composer.error = None;
                    self.save_composer();
                }
                if ui.small_button("Copy message").clicked() {
                    ui.ctx().copy_text(self.composer.text.clone());
                }
                if ui.small_button("Load saved copy…").clicked() {
                    self.composer.reload_review = true;
                }
                if (self.open_after_draft.is_some() || self.close_after_draft)
                    && ui.small_button("Keep working here").clicked()
                {
                    self.open_after_draft = None;
                    self.close_after_draft = false;
                }
            });
        } else if let Some(notice) = &self.composer.notice {
            ui.label(theme::caption(notice));
        } else {
            ui.label(theme::caption(if !self.composer.ready {
                "Loading saved message…"
            } else if self.composer.pending() || self.composer.dirty() {
                "Saving local message…"
            } else if self.composer.text.is_empty() {
                "Messages are saved locally as you type"
            } else {
                "Message saved locally"
            }));
        }
    }
}
