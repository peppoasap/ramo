use super::*;
use crate::connection_work::{Job, Reply};
use ramo_core::remotes::Remote;
pub(super) struct RemoteDialog {
    pub repo: String,
    pub request: Option<u64>,
    rows: Vec<Remote>,
    name: String,
    url: String,
    existing: bool,
    error: Option<String>,
}
impl RemoteDialog {
    pub(super) fn receive(&mut self, result: Result<Reply, String>) {
        self.request = None;
        match result {
            Ok(Reply::Remotes(rows)) => {
                self.rows = rows;
                if !self.existing
                    && self.url.is_empty()
                    && self.rows.iter().any(|r| r.name == self.name)
                {
                    self.name = if !self.rows.iter().any(|r| r.name == "upstream") {
                        "upstream".into()
                    } else {
                        (2..)
                            .map(|i| format!("remote-{i}"))
                            .find(|name| !self.rows.iter().any(|r| r.name == *name))
                            .unwrap()
                    };
                }
            }
            Err(error) => self.error = Some(error),
            _ => self.error = Some("Unexpected remote response.".into()),
        }
    }
}
impl App {
    pub(super) fn open_remotes(&mut self) {
        if self.clone_busy() || self.action_request.is_some() {
            return;
        }
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let repo = snapshot.repo.id.clone();
        let id = self.send(Work::Connection(Job::Remotes(repo.clone())));
        self.remote_dialog = Some(RemoteDialog {
            repo,
            request: self.pending.contains(&id).then_some(id),
            rows: vec![],
            name: "origin".into(),
            url: String::new(),
            existing: false,
            error: None,
        });
    }
    pub(super) fn remote_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut state) = self.remote_dialog.take() else {
            return;
        };
        if self
            .snapshot
            .as_ref()
            .is_none_or(|s| s.repo.id != state.repo)
        {
            return;
        }
        let busy = state.request.is_some();
        let mut close = false;
        let mut save = false;
        let mut reload = false;
        let response = theme::modal(ctx, "repository_remotes", "Remotes", 640., |ui| {
            if let Some(error) = &state.error {
                ui.colored_label(theme::AMBER, error);
            }
            ui.label(theme::caption("Inspect where this repository fetches and pushes. Saving a URL changes local configuration only."));
            if busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading remotes…");
                });
            }
            ScrollArea::vertical().id_salt("configured_remotes").max_height(220.).show(ui,|ui|{
                for remote in &state.rows {
                    ui.horizontal(|ui|{
                        ui.strong(&remote.name);
                        if ui.add_enabled(!busy&&remote.editable,egui::Button::new("Edit").small()).clicked(){state.name=remote.name.clone();state.url=remote.fetch[0].clone();state.existing=true;state.error=None;}
                    });
                    for url in &remote.fetch {ui.label(theme::caption(format!("Fetch · {url}")));}
                    if remote.push!=remote.fetch {for url in &remote.push {ui.label(theme::caption(format!("Push · {url}")));}}
                    if !remote.editable {ui.label(theme::caption("Protected configuration: edit multiple/separate push URLs or local/custom transports outside Ramo."));}
                    ui.separator();
                }
                if !busy&&state.rows.is_empty(){ui.label("No remotes configured yet.");}
            });
            ui.add_space(10.);
            ui.add_enabled_ui(!busy, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(if state.existing {
                        "Edit remote"
                    } else {
                        "Add remote"
                    });
                    if state.existing && ui.small_button("Add another").clicked() {
                        state.name.clear();
                        state.url.clear();
                        state.existing = false;
                    }
                });
                ui.label("Name");
                ui.add_enabled(
                    !state.existing,
                    TextEdit::singleline(&mut state.name).desired_width(f32::INFINITY),
                );
                ui.label("Repository URL");
                ui.add(
                    TextEdit::singleline(&mut state.url)
                        .hint_text("HTTPS or SSH")
                        .desired_width(f32::INFINITY),
                );
            });
            let name = state.name.trim();
            let name_valid = !name.is_empty()
                && name.len() <= 100
                && !name.starts_with('-')
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
            let valid = ramo_core::cloning::validate_url(state.url.trim()).is_ok() && name_valid;
            let unchanged = state.existing
                && state.rows.iter().any(|r| {
                    r.name == state.name
                        && r.fetch.first().is_some_and(|url| url == state.url.trim())
                });
            if !state.url.is_empty() && !valid {
                ui.label(theme::caption("Use an ordinary HTTPS/SSH URL and existing Git/SSH authentication, with no token in the URL."));
            }
            ui.add_space(14.);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                save = theme::primary(ui, "Review changes", !busy && valid && !unchanged).clicked();
                close = ui.button("Close").clicked();
                reload = ui
                    .add_enabled(!busy, egui::Button::new("Refresh"))
                    .clicked();
            });
        });
        if save {
            self.request_action(
                Action::ConfigureRemote {
                    name: state.name.trim().into(),
                    url: state.url.trim().into(),
                    existing: state.existing,
                },
                "Save remote URL",
                format!(
                    "{} remote {} to {}. This updates local configuration; no fetch or push runs.",
                    if state.existing { "Change" } else { "Add" },
                    state.name.trim(),
                    state.url.trim()
                ),
                true,
            );
            return;
        }
        if reload {
            if self.editor.as_ref().is_some_and(|e| e.dirty) {
                state.error =
                    Some("Save your merge draft before refreshing repository state.".into());
                self.remote_dialog = Some(state);
                return;
            }
            self.refresh();
            self.open_remotes();
            return;
        }
        if !(close || response.should_close()) {
            self.remote_dialog = Some(state);
        }
    }
}
