//! Debounced local workspace IO, separate from Git and editor/composer persistence.
use super::*;
use crate::workspace as store;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct Persistence {
    pub loading: Option<u64>,
    pub restoring: Option<u64>,
    saving: Option<u64>,
    pub ready: bool,
    revision: Option<String>,
    saved: store::Document,
    changed: Option<Instant>,
    observed: Option<store::Document>,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub missing_notice: bool,
    initial: Option<PathBuf>,
    quit: bool,
    allow_quit: bool,
}
impl Persistence {
    pub(super) fn busy_startup(&self) -> bool {
        self.loading.is_some() || self.restoring.is_some()
    }
    pub(super) fn owns(&self, id: u64) -> bool {
        self.loading == Some(id) || self.restoring == Some(id) || self.saving == Some(id)
    }
}
impl App {
    pub(super) fn load_workspace(&mut self, initial: Option<PathBuf>) {
        self.workspace.initial = initial;
        let id = self.send(Work::WorkspaceLoad);
        self.workspace.loading = self.pending.contains(&id).then_some(id);
    }
    pub(super) fn workspace_reply(&mut self, id: u64, result: Result<Payload, String>) {
        if self.workspace.loading == Some(id) {
            self.workspace.loading = None;
            match result {
                Ok(Payload::WorkspaceLoaded(loaded)) => {
                    let document = loaded
                        .as_ref()
                        .map(|v| v.document.clone())
                        .unwrap_or_default();
                    self.workspace.revision = loaded.map(|v| v.revision);
                    self.workspace.saved = document.clone();
                    let extra = self.workspace.initial.take();
                    let request = self.send(Work::WorkspaceRestore { document, extra });
                    self.workspace.restoring = self.pending.contains(&request).then_some(request);
                }
                Err(error) => {
                    self.workspace.error = Some(error);
                    if let Some(path) = self.workspace.initial.take() {
                        self.open(Some(path));
                    }
                }
                _ => self.workspace.error = Some("Unexpected workspace load response.".into()),
            }
        } else if self.workspace.restoring == Some(id) {
            self.workspace.restoring = None;
            match result {
                Ok(Payload::WorkspaceRestored(restored)) => {
                    self.workspace.ready = true;
                    self.apply_workspace(restored);
                }
                Err(error) => self.workspace.error = Some(error),
                _ => self.workspace.error = Some("Unexpected workspace restore response.".into()),
            }
        } else if self.workspace.saving == Some(id) {
            self.workspace.saving = None;
            match result {
                Ok(Payload::WorkspaceSaved(loaded)) => {
                    self.workspace.revision = Some(loaded.revision);
                    self.workspace.saved = loaded.document;
                }
                Err(error) => self.workspace.error = Some(error),
                _ => self.workspace.error = Some("Unexpected workspace save response.".into()),
            }
        }
    }
    pub(super) fn workspace_tick(&mut self, ctx: &egui::Context) {
        let closing = ctx.input(|i| i.viewport().close_requested());
        if closing
            && !self.workspace.allow_quit
            && self.editor_exit.is_none()
            && !self.close_after_draft
            && self.action_request.is_none()
            && !self.clone_busy()
        {
            self.workspace.quit = true;
            ctx.request_repaint();
        }
        if self.workspace.busy_startup() {
            if closing {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
            return;
        }
        let document = self.workspace_document();
        if self.workspace.ready && self.workspace.observed.as_ref() != Some(&document) {
            self.workspace.observed = Some(document.clone());
            self.workspace.changed = Some(Instant::now());
        }
        let dirty = self.workspace.ready && document != self.workspace.saved;
        if !self.workspace.allow_quit
            && self.workspace.quit
            && (dirty || self.workspace.saving.is_some() || self.workspace.error.is_some())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if dirty
            && self.workspace.error.is_none()
            && self.workspace.saving.is_none()
            && self.open_request.is_none()
            && (self.workspace.quit
                || self
                    .workspace
                    .changed
                    .is_some_and(|t| t.elapsed() >= Duration::from_millis(500)))
        {
            match document.encode() {
                Ok(_) => {
                    let id = self.send(Work::WorkspaceSave {
                        document: document.clone(),
                        expected: self.workspace.revision.clone(),
                    });
                    if self.pending.contains(&id) {
                        self.workspace.saving = Some(id);
                    }
                }
                Err(error) => self.workspace.error = Some(error),
            }
        }
        if dirty && self.workspace.error.is_none() {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        if self.workspace.quit
            && self.workspace.error.is_none()
            && !dirty
            && self.workspace.saving.is_none()
        {
            self.workspace.quit = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    pub(super) fn workspace_notice(&mut self, ctx: &egui::Context) {
        if let Some(notice) = self.workspace.notice.clone() {
            egui::TopBottomPanel::top("workspace_restore_notice").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(notice);
                    if ui.button("Dismiss").clicked() {
                        self.workspace.notice = None;
                    }
                });
            });
        }
        let Some(error) = self.workspace.error.clone() else {
            return;
        };
        egui::TopBottomPanel::top("workspace_storage_error").show(ctx, |ui| {
            ui.label(RichText::new("Workspace not saved").color(theme::AMBER));
            ui.label(error);
            ui.horizontal_wrapped(|ui| {
                if self.workspace.ready && ui.button("Retry saving").clicked() {
                    self.workspace.error = None;
                    self.workspace.changed = Some(Instant::now() - Duration::from_secs(1));
                }
                if self.workspace.quit {
                    if ui.button("Keep working").clicked() {
                        self.workspace.quit = false;
                    }
                    if ui.button("Quit without saving workspace").clicked() {
                        self.workspace.allow_quit = true;
                        self.workspace.quit = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
        });
    }
}
