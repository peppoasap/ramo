mod actions;
mod comparison;
mod composer;
mod gestures;
mod history;
mod inspection;
mod navigation;

use crate::{
    drafts::Scope,
    editor::{self, Editor},
    graph, theme,
    worker::{Payload, Request, Work, Worker},
};
use eframe::egui::{
    self, Align, Color32, FontId, Key, Layout, RichText, ScrollArea, Sense, TextEdit,
};
use egui_phosphor::regular as icon;
use ramo_core::git::{self, Action, Commit, FileChange, Snapshot};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

const ROW: f32 = 22.;
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    History,
    Changes,
    Compare,
}

pub struct App {
    worker: Worker,
    #[cfg(target_os = "macos")]
    quit_bridge: Option<crate::macos_quit::Bridge>,
    action: Option<actions::PendingAction>,
    action_request: Option<u64>,
    amend_request: u64,
    form: Option<actions::Form>,
    composer: crate::commit_draft::Composer,
    comparison: comparison::CompareView,
    drag_candidate: Option<crate::gestures::Candidate>,
    drag_frame: bool,
    branch_drop_rect: Option<egui::Rect>,
    open_after_draft: Option<Option<PathBuf>>,
    close_after_draft: bool,
    epoch: u64,
    sequence: u64,
    pending: BTreeSet<u64>,
    detail_request: u64,
    diff_request: u64,
    editor_request: u64,
    draft_request: u64,
    page_request: Option<u64>,
    snapshot: Option<Snapshot>,
    commits: Vec<Commit>,
    rows: Vec<graph::Row>,
    visible: Vec<usize>,
    has_more: bool,
    selected: Option<String>,
    files: Vec<FileChange>,
    file: Option<String>,
    staged_view: bool,
    diff: String,
    patch_view: crate::patch_view::Selection,
    filter: String,
    branch_filter: String,
    mode: Mode,
    inspector: bool,
    compact: bool,
    scroll_target: Option<usize>,
    rendered_rows: usize,
    editor: Option<Editor>,
    error: Option<String>,
    close_confirm: bool,
    close_window: bool,
    open_after_discard: bool,
}
impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        state: PathBuf,
        repository: Option<PathBuf>,
    ) -> Self {
        theme::install(&cc.egui_ctx);
        let worker = Worker::start(state, cc.egui_ctx.clone());
        let mut app = Self {
            worker,
            #[cfg(target_os = "macos")]
            quit_bridge: crate::macos_quit::Bridge::install(&cc.egui_ctx),
            action: None,
            action_request: None,
            amend_request: 0,
            form: None,
            composer: Default::default(),
            comparison: Default::default(),
            drag_candidate: None,
            drag_frame: false,
            branch_drop_rect: None,
            open_after_draft: None,
            close_after_draft: false,
            epoch: 0,
            sequence: 0,
            pending: BTreeSet::new(),
            detail_request: 0,
            diff_request: 0,
            editor_request: 0,
            draft_request: 0,
            page_request: None,
            snapshot: None,
            commits: Vec::new(),
            rows: Vec::new(),
            visible: Vec::new(),
            has_more: false,
            selected: None,
            files: Vec::new(),
            file: None,
            staged_view: false,
            diff: String::new(),
            patch_view: Default::default(),
            filter: String::new(),
            branch_filter: String::new(),
            mode: Mode::History,
            inspector: true,
            compact: true,
            scroll_target: None,
            rendered_rows: 0,
            editor: None,
            error: None,
            close_confirm: false,
            close_window: false,
            open_after_discard: false,
        };
        if let Some(repository) = repository {
            app.open(Some(repository));
        }
        app
    }
    fn send(&mut self, work: Work) -> u64 {
        self.sequence += 1;
        let id = self.sequence;
        match self.worker.sender.try_send(Request {
            sequence: id,
            epoch: self.epoch,
            work,
        }) {
            Ok(()) => {
                self.pending.insert(id);
            }
            Err(_) => {
                self.error = Some(
                    "Background queue is busy. Let the current operation finish, then try again."
                        .into(),
                )
            }
        }
        id
    }
    fn open(&mut self, path: Option<PathBuf>) {
        if self.composer.pending() || self.composer.dirty() {
            self.open_after_draft = Some(path);
            self.save_composer();
            return;
        }
        if self.action_request.is_some() {
            self.error = Some("Wait for the current Git operation to finish.".into());
            return;
        }
        self.action = None;
        self.form = None;
        self.epoch += 1;
        self.comparison.cancel_requests();
        self.drag_candidate = None;
        self.pending.clear();
        self.page_request = None;
        self.send(Work::Open(path));
    }
    fn refresh(&mut self) {
        if self.action_request.is_some() {
            return;
        }
        if self.editor.as_ref().is_some_and(|e| e.dirty) {
            self.error =
                Some("Save the editor draft or return to History before refreshing.".into());
            return;
        }
        if let Some(snapshot) = &self.snapshot {
            let repo = snapshot.repo.id.clone();
            self.epoch += 1;
            self.comparison.cancel_requests();
            self.drag_candidate = None;
            self.pending.clear();
            self.page_request = None;
            self.send(Work::Refresh(repo));
        }
    }
    fn refilter(&mut self) {
        let query = self.filter.to_lowercase();
        self.visible = self
            .commits
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                query.is_empty()
                    || c.subject.to_lowercase().contains(&query)
                    || c.author.to_lowercase().contains(&query)
                    || c.oid.contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
    }
    fn select(&mut self, oid: String) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let repo = snapshot.repo.id.clone();
        self.selected = Some(oid.clone());
        self.files.clear();
        self.file = None;
        self.diff.clear();
        // Invalidate a late diff from the previously selected commit/file too.
        self.diff_request = 0;
        self.detail_request = self.send(Work::Commit { repo, oid });
    }
    fn load_more(&mut self) {
        if self.page_request.is_some() || !self.has_more {
            return;
        }
        if let Some(snapshot) = &self.snapshot {
            self.page_request = Some(self.send(Work::Page {
                repo: snapshot.repo.id.clone(),
                fingerprint: snapshot.fingerprint.clone(),
                skip: self.commits.len(),
            }));
        }
    }
    fn pump(&mut self) {
        while let Ok(reply) = self.worker.receiver.try_recv() {
            // Composer IO is scoped by its own unique request IDs. Mutations
            // invalidate Git reads via epoch changes, not an in-flight local save.
            if self.composer.load_request == Some(reply.sequence)
                || self.composer.save_request == Some(reply.sequence)
            {
                self.pending.remove(&reply.sequence);
                match reply.result {
                    Ok(Payload::CommitDraftLoaded(loaded)) => self.composer.loaded(Ok(loaded)),
                    Ok(Payload::CommitDraftSaved(loaded)) => self.composer.saved(Ok(loaded)),
                    Err(error) => {
                        self.error = Some(error.clone());
                        if self.composer.load_request == Some(reply.sequence) {
                            self.composer.loaded(Err(error));
                        } else {
                            self.composer.saved(Err(error));
                        }
                    }
                    _ => {}
                }
                continue;
            }
            if reply.epoch != self.epoch {
                continue;
            }
            self.pending.remove(&reply.sequence);
            if self.comparison.owns(reply.sequence) {
                self.comparison.receive(reply.sequence, reply.result);
                continue;
            }
            let page_failed = self.page_request == Some(reply.sequence);
            if page_failed {
                self.page_request = None;
            }
            match reply.result {
                Err(error) => {
                    if self.composer.load_request == Some(reply.sequence) {
                        self.composer.loaded(Err(error.clone()));
                    }
                    if self.composer.save_request == Some(reply.sequence) {
                        self.composer.saved(Err(error.clone()));
                    }
                    if self.action_request == Some(reply.sequence) {
                        self.action_request = None;
                        self.error = Some(error.clone());
                    }
                    if page_failed {
                        self.has_more = false;
                        self.error = Some(error.clone());
                    }
                    if reply.sequence == self.draft_request {
                        if let Some(editor) = &mut self.editor {
                            editor.draft_busy = false;
                            editor.notice = Some(error);
                        }
                    } else if reply.sequence == self.amend_request
                        || reply.sequence == self.editor_request
                        || reply.sequence == self.detail_request
                        || reply.sequence == self.diff_request
                        || reply.sequence == self.sequence
                    {
                        self.error = Some(error);
                    }
                }
                Ok(Payload::Trusted(snapshot)) => {
                    self.snapshot = Some(*snapshot);
                    self.action_request = None;
                    if let Some(action) = self.action.take() {
                        self.run_action(action);
                    }
                }
                Ok(Payload::Changed {
                    snapshot,
                    commits,
                    error,
                }) => {
                    self.action_request = None;
                    let mode = self.mode;
                    self.epoch += 1;
                    self.comparison.cancel_requests();
                    self.drag_candidate = None;
                    self.pending.clear();
                    self.page_request = None;
                    self.snapshot = Some(*snapshot);
                    self.has_more = commits.len() == 300;
                    self.commits = commits;
                    self.rows = graph::layout(&self.commits);
                    self.refilter();
                    let focused_file = self.file.clone().or_else(|| {
                        self.action.as_ref().and_then(|a| match &a.action {
                            Action::Stage { paths } | Action::Unstage { paths } => {
                                paths.first().cloned()
                            }
                            _ => None,
                        })
                    });
                    self.diff.clear();
                    self.file = None;
                    self.files.clear();
                    self.diff_request = 0;
                    self.detail_request = 0;
                    if error.is_none() {
                        if self
                            .action
                            .as_ref()
                            .is_some_and(|a| matches!(a.action, Action::Commit { .. }))
                        {
                            self.composer.committed();
                        }
                        if self
                            .action
                            .as_ref()
                            .is_some_and(|a| matches!(a.action, Action::ResolveFile { .. }))
                        {
                            self.editor = None;
                        }
                    }
                    self.action = None;
                    self.error = error;
                    self.mode = if self
                        .snapshot
                        .as_ref()
                        .is_some_and(|s| s.operation.is_some())
                    {
                        Mode::Changes
                    } else {
                        mode
                    };
                    if self.mode == Mode::History
                        && let Some(first) = self.commits.first()
                    {
                        self.select(first.oid.clone());
                    }
                    if self.mode == Mode::Changes
                        && self.editor.is_none()
                        && let (Some(path), Some(snapshot)) = (focused_file, self.snapshot.as_ref())
                        && let Some(file) = snapshot.files.iter().find(|f| f.path == path)
                    {
                        self.staged_view = !git::is_conflicted(file)
                            && file.index != "?"
                            && (file.worktree.trim().is_empty()
                                || (self.staged_view && !file.index.trim().is_empty()));
                        self.file = Some(path.clone());
                        self.diff_request = self.send(Work::Diff {
                            repo: snapshot.repo.id.clone(),
                            oid: None,
                            path,
                            staged: self.staged_view,
                        });
                    }
                }
                Ok(Payload::Compared(_) | Payload::ComparisonDiff(_)) => {}
                Ok(Payload::Cancelled) => {}
                Ok(Payload::Opened { snapshot, commits }) => {
                    if self.composer.repository.as_deref() != Some(&snapshot.repo.id) {
                        self.composer.bind(
                            snapshot.repo.id.clone(),
                            snapshot.head.clone().unwrap_or_default(),
                        );
                        self.load_composer();
                    }
                    self.error = None;
                    self.mode = Mode::History;
                    self.comparison = Default::default();
                    self.editor = None;
                    self.filter.clear();
                    self.branch_filter.clear();
                    self.has_more = commits.len() == 300;
                    self.commits = commits;
                    self.snapshot = Some(*snapshot);
                    self.rows = graph::layout(&self.commits);
                    self.refilter();
                    self.selected = None;
                    self.files.clear();
                    self.file = None;
                    self.diff.clear();
                    if let Some(first) = self.commits.first() {
                        self.select(first.oid.clone());
                    }
                    self.scroll_target = Some(0);
                }
                Ok(Payload::Page { skip, commits }) => {
                    if skip != self.commits.len() {
                        continue;
                    }
                    let count = commits.len();
                    let existing: std::collections::HashSet<_> =
                        self.commits.iter().map(|c| c.oid.clone()).collect();
                    self.commits
                        .extend(commits.into_iter().filter(|c| !existing.contains(&c.oid)));
                    self.has_more =
                        count == 300 && self.commits.len() > skip && self.commits.len() < 100_000;
                    self.rows = graph::layout(&self.commits);
                    self.refilter();
                }
                Ok(Payload::Commit { oid, files, diff }) => {
                    if self.mode != Mode::History
                        || reply.sequence != self.detail_request
                        || self.selected.as_deref() != Some(&oid)
                    {
                        continue;
                    }
                    self.file = files.first().map(|f| f.path.clone());
                    self.files = files;
                    self.diff = diff;
                }
                Ok(Payload::Diff(diff)) => {
                    if reply.sequence == self.diff_request {
                        self.diff = diff;
                    }
                }
                Ok(Payload::Conflict { path, file }) => {
                    if reply.sequence != self.editor_request {
                        continue;
                    }
                    if let Some(snapshot) = &self.snapshot {
                        let scope = Scope {
                            repository: snapshot.repo.id.clone(),
                            path,
                            fingerprint: snapshot.fingerprint.clone(),
                        };
                        self.editor = Some(Editor::new(scope.clone(), file));
                        self.draft_request = self.send(Work::DraftLoad(scope));
                    }
                }
                Ok(Payload::AmendMessage { head, message }) => {
                    if reply.sequence == self.amend_request
                        && self.form.is_none()
                        && self.snapshot.as_ref().and_then(|s| s.head.as_deref()) == Some(&head)
                    {
                        self.form = Some(actions::Form {
                            kind: actions::FormKind::Amend(head),
                            text: message,
                        });
                    }
                }
                Ok(Payload::CommitDraftLoaded(loaded)) => {
                    if self.composer.load_request == Some(reply.sequence) {
                        self.composer.loaded(Ok(loaded));
                    }
                }
                Ok(Payload::CommitDraftSaved(loaded)) => {
                    if self.composer.save_request == Some(reply.sequence) {
                        self.composer.saved(Ok(loaded));
                    }
                }
                Ok(Payload::DraftLoaded(loaded)) => {
                    if reply.sequence == self.draft_request
                        && let Some(editor) = &mut self.editor
                    {
                        editor.draft_loaded(loaded);
                    }
                }
                Ok(Payload::DraftSaved(loaded)) => {
                    if reply.sequence == self.draft_request
                        && let Some(editor) = &mut self.editor
                    {
                        editor.draft_saved(loaded);
                    }
                }
            }
        }
    }
    fn ask_open(&mut self) {
        if self.editor.as_ref().is_some_and(|e| e.dirty) {
            self.close_confirm = true;
            self.open_after_discard = true;
        } else {
            self.open(None);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "macos")]
        {
            if self.quit_bridge.is_none() {
                self.quit_bridge = crate::macos_quit::Bridge::install(ctx);
            }
            if self.quit_bridge.as_ref().is_some_and(|b| b.take_request()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.pump();
        self.composer_tick(ctx);
        self.guard_drag(ctx);
        if ctx.input(|i| i.viewport().close_requested())
            && (self.composer.pending() || self.composer.dirty())
            && !self.editor.as_ref().is_some_and(|e| e.dirty)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_after_draft = true;
            self.save_composer();
        }
        if self.action_request.is_some() && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.error = Some("Wait for the Git operation to finish before closing.".into());
        }
        if ctx.input(|i| i.viewport().close_requested())
            && self.editor.as_ref().is_some_and(|e| e.dirty)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_confirm = true;
            self.close_window = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, Key::O)) {
            self.ask_open();
        }
        self.chrome(ctx);
        self.drag_overlay(ctx);
        self.action_dialogs(ctx);
        if self.editor.is_none() {
            self.sidebar(ctx);
            self.inspector(ctx);
        }
        let mut event = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BASE).inner_margin(10.))
            .show(ctx, |ui| {
                if self.action_request.is_some() || self.action.is_some() {
                    ui.disable();
                }
                if let Some(editor) = &mut self.editor {
                    if self.action_request.is_some() || self.action.is_some() {
                        ui.disable();
                    }
                    event = editor.ui(ctx, ui);
                    return;
                }
                if self.snapshot.is_none() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.25);
                        ui.heading("Your repository, clearly.");
                        ui.label(
                            "Open a Git repository to browse its branches, history and changes.",
                        );
                        ui.add_space(12.);
                        if theme::button(ui, icon::FOLDER_OPEN, "Open repository").clicked() {
                            self.ask_open();
                        }
                        ui.add_space(14.);
                        ui.label(theme::caption(
                            "Rust-native development · Open, inspect and work with Git",
                        ));
                    });
                    return;
                }
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.mode == Mode::History, "History")
                        .clicked()
                        && self.mode != Mode::History
                    {
                        self.diff_request = 0;
                        self.mode = Mode::History;
                        if let Some(oid) = self.selected.clone() {
                            self.select(oid);
                        }
                    }
                    if ui
                        .selectable_label(self.mode == Mode::Compare, "Compare")
                        .clicked()
                    {
                        self.mode = Mode::Compare;
                    }
                    let count = self.snapshot.as_ref().map_or(0, |s| s.files.len());
                    if ui
                        .selectable_label(self.mode == Mode::Changes, format!("Changes  {count}"))
                        .clicked()
                    {
                        self.mode = Mode::Changes;
                        self.file = None;
                        self.diff.clear();
                        self.diff_request = 0;
                        self.detail_request = 0;
                    }
                });
                ui.separator();
                match self.mode {
                    Mode::History => self.history(ui),
                    Mode::Changes => self.changes(ui),
                    Mode::Compare => self.comparison(ui),
                }
            });
        self.finish_drag(ctx);
        match event {
            Some(editor::Event::Resolve) => {
                if let Some(e) = &self.editor {
                    // Use the editor's original fingerprint, not a newer refresh.
                    let action = actions::PendingAction::resolution(e);
                    if self.action.is_none() && self.action_request.is_none() {
                        self.action = Some(action);
                    }
                }
            }
            Some(editor::Event::Back) => {
                if self.editor.as_ref().is_some_and(|e| e.dirty) {
                    self.close_confirm = true;
                } else {
                    self.editor = None;
                }
            }
            Some(editor::Event::Save {
                scope,
                text,
                expected,
            }) => {
                self.draft_request = self.send(Work::DraftSave {
                    scope,
                    text,
                    expected,
                });
            }
            None => {}
        }
        if self.close_confirm {
            egui::Window::new("Keep your editor changes?").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER,[0.,0.]).show(ctx,|ui|{
                ui.label("Unsaved result edits are still in memory. Return to the editor to save a local draft.");
                ui.horizontal(|ui|{
                    if ui.button("Keep editing").clicked(){self.close_confirm=false;self.close_window=false;self.open_after_discard=false;}
                    if ui.button("Discard unsaved editor changes").clicked(){
                        self.editor=None;self.close_confirm=false;
                        if self.close_window {ctx.send_viewport_cmd(egui::ViewportCommand::Close);self.close_window=false;}
                        if self.open_after_discard {self.open_after_discard=false;self.open(None);}
                    }
                });
            });
        }
    }
}

fn short(oid: &str) -> &str {
    oid.get(..8).unwrap_or(oid)
}
fn date(value: &str) -> &str {
    value.get(..10).unwrap_or(value)
}
fn paint_text(
    p: &egui::Painter,
    pos: egui::Pos2,
    text: &str,
    width: f32,
    size: f32,
    color: Color32,
    mono: bool,
) {
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        if mono {
            FontId::monospace(size)
        } else {
            FontId::proportional(size)
        },
        color,
        width,
    );
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    let galley = p.layout_job(job);
    p.galley(pos, galley, color);
}
fn render_diff(ui: &mut egui::Ui, diff: &str) {
    let lines: Vec<_> = diff.split('\n').take(12_001).collect();
    let count = lines.len().min(12_000);
    if lines.len() > 12_000 {
        ui.label(theme::caption(
            "Preview limited to 12,000 lines. The file is unchanged.",
        ));
    }
    if lines.iter().any(|line| line.len() > 4096) {
        ui.label(theme::caption("Long-line preview limit: 4,096 characters."));
    }
    ui.spacing_mut().item_spacing.y = 0.;
    ScrollArea::both()
        .id_salt("diff")
        .auto_shrink([false, false])
        .show_rows(ui, 18., count, |ui, range| {
            for index in range {
                let line = lines[index];
                let color = if line.starts_with('+') && !line.starts_with("+++") {
                    theme::MINT
                } else if line.starts_with('-') && !line.starts_with("---") {
                    theme::RED
                } else if line.starts_with("@@") {
                    theme::LANES[1]
                } else {
                    theme::MUTED
                };
                let preview = if let Some((end, _)) = line.char_indices().nth(4096) {
                    format!("{}…", &line[..end])
                } else {
                    line.to_owned()
                };
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 18.),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(preview).monospace().size(12.5).color(color),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        )
                    },
                );
            }
        });
}
