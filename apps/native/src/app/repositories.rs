//! Session tabs keep navigation intent, never cached Git snapshots or trust.
use super::*;
use ramo_core::git::Repository;

struct View {
    mode: Mode,
    filter: String,
    branches: String,
    selected: Option<String>,
    file: Option<String>,
    staged: bool,
    comparison: comparison::CompareView,
}
#[derive(Clone)]
pub(super) struct Tab {
    id: String,
    name: String,
    path: String,
}
#[derive(Default)]
pub(super) struct Tabs {
    pub entries: Vec<Tab>,
    views: HashMap<String, View>,
    pub recent: Vec<String>,
    prompted: BTreeSet<String>,
    pub prompt: Option<String>,
    pub trust_request: Option<u64>,
}
impl Tabs {
    fn opened(&mut self, repo: &Repository) {
        if self.prompted.insert(repo.id.clone()) && !repo.trusted {
            self.prompt = Some(repo.id.clone());
        }
        let tab = Tab {
            id: repo.id.clone(),
            name: repo.name.clone(),
            path: repo.path.clone(),
        };
        if let Some(current) = self.entries.iter_mut().find(|t| t.id == repo.id) {
            *current = tab;
        } else {
            self.entries.push(tab);
        }
    }
    fn visible(&self, active: Option<&str>, count: usize) -> Vec<Tab> {
        let mut visible: Vec<_> = self.entries.iter().take(count.max(1)).cloned().collect();
        if let Some(tab) = self.entries.iter().find(|t| Some(t.id.as_str()) == active)
            && !visible.iter().any(|t| t.id == tab.id)
        {
            visible.pop();
            visible.push(tab.clone());
        }
        visible
    }
    fn remove(&mut self, id: &str) -> Option<String> {
        let index = self.entries.iter().position(|t| t.id == id)?;
        self.entries.remove(index);
        self.views.remove(id);
        self.prompted.remove(id);
        self.entries
            .get(index.saturating_sub(1))
            .or_else(|| self.entries.first())
            .map(|t| t.id.clone())
    }
}
impl App {
    pub(super) fn repository_trust_dialog(&mut self, ctx: &egui::Context) {
        let Some(repo) = self.repositories.prompt.clone() else {
            return;
        };
        let Some(snapshot) = self.snapshot.as_ref().filter(|s| s.repo.id == repo) else {
            return;
        };
        let path = snapshot.repo.path.clone();
        let busy = self.repositories.trust_request.is_some();
        let mut trust = false;
        let mut readonly = false;
        let response = theme::modal(
            ctx,
            "repository_open_trust",
            "Trust this repository?",
            460.,
            |ui| {
                ui.label("Git hooks, filters and credential helpers can run local programs. Enable Git writes only for repositories you trust.");
                ui.add_space(8.);
                ui.label(theme::caption(&path));
                ui.label(theme::caption("Trust lasts until this tab closes or the app quits. Browsing works in read-only mode."));
                ui.add_space(16.);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    trust = theme::primary(ui, "Trust repository", !busy).clicked();
                    readonly = ui
                        .add_enabled(!busy, egui::Button::new("Keep read-only"))
                        .clicked();
                    if busy {
                        ui.spinner();
                    }
                });
            },
        );
        if !busy && (readonly || response.should_close()) {
            self.repositories.prompt = None;
        }
        if trust {
            let id = self.send(Work::Trust(repo));
            self.repositories.trust_request = self.pending.contains(&id).then_some(id);
        }
    }
    pub(super) fn load_recent(&mut self) {
        let id = self.send(Work::RecentList);
        self.recent_request = self.pending.contains(&id).then_some(id);
    }
    pub(super) fn ask_repository(&mut self, target: OpenTarget) {
        if self.modal_open() || self.action_request.is_some() {
            return;
        }
        let inactive = matches!(&target,OpenTarget::Close(id) if self.snapshot.as_ref().is_none_or(|s|&s.repo.id != id));
        if !inactive
            && self
                .editor
                .as_ref()
                .is_some_and(|e| e.dirty || e.draft_busy)
        {
            self.editor_exit = Some(EditorExit::Repository(target));
        } else {
            self.open_target(target);
        }
    }
    fn capture_view(&self) -> View {
        View {
            mode: if self.editor.is_some() {
                Mode::Changes
            } else {
                self.mode
            },
            filter: self.filter.clone(),
            branches: self.branch_filter.clone(),
            selected: self.selected.clone(),
            file: self
                .editor
                .as_ref()
                .map(|e| e.scope.path.clone())
                .or_else(|| self.file.clone()),
            staged: self.staged_view,
            comparison: self.comparison.selection(),
        }
    }
    pub(super) fn install_repository(&mut self, snapshot: Snapshot, commits: Vec<Commit>) {
        let changed = self
            .snapshot
            .as_ref()
            .is_none_or(|s| s.repo.id != snapshot.repo.id);
        if changed && let Some(current) = &self.snapshot {
            self.repositories
                .views
                .insert(current.repo.id.clone(), self.capture_view());
        }
        self.repositories.opened(&snapshot.repo);
        let view = if changed {
            self.repositories.views.remove(&snapshot.repo.id)
        } else {
            Some(self.capture_view())
        };
        let retain_comparison = !changed && self.mode == Mode::Compare;
        if self.composer.repository.as_deref() != Some(&snapshot.repo.id) {
            self.composer.bind(
                snapshot.repo.id.clone(),
                snapshot.head.clone().unwrap_or_default(),
            );
            self.load_composer();
        }
        self.error = None;
        self.editor = None;
        if !retain_comparison {
            self.mode = Mode::History;
            self.comparison = Default::default();
        }
        self.filter.clear();
        self.branch_filter.clear();
        self.has_more = commits.len() == 300;
        self.commits = commits;
        self.snapshot = Some(snapshot);
        self.rows = graph::layout(&self.commits);
        self.selected = None;
        self.files.clear();
        self.file = None;
        self.diff.clear();
        self.patch_view = Default::default();
        if let Some(view) = view {
            self.mode = view.mode;
            self.filter = view.filter;
            self.branch_filter = view.branches;
            if !retain_comparison {
                self.comparison = view.comparison;
            }
            self.selected = view.selected;
            self.file = view.file;
            self.staged_view = view.staged;
        }
        self.refilter();
        self.scroll_target = Some(0);
        if self.mode == Mode::History {
            let oid = self
                .selected
                .as_ref()
                .filter(|oid| self.commits.iter().any(|c| &c.oid == *oid))
                .cloned()
                .or_else(|| self.commits.first().map(|c| c.oid.clone()));
            if let Some(oid) = oid {
                self.scroll_target = self.commits.iter().position(|c| c.oid == oid);
                self.select(oid);
            }
        } else if self.mode == Mode::Compare && changed && self.comparison.has_pair() {
            self.run_comparison();
        } else if self.mode == Mode::Changes
            && let (Some(path), Some(snapshot)) = (self.file.clone(), self.snapshot.as_ref())
        {
            if let Some(file) = snapshot.files.iter().find(|f| f.path == path) {
                if git::is_conflicted(file) {
                    self.open_conflict(path);
                } else {
                    let staged =
                        self.staged_view && !file.index.trim().is_empty() && file.index != "?";
                    self.staged_view = staged;
                    self.diff_request = self.send(Work::Diff {
                        repo: snapshot.repo.id.clone(),
                        oid: None,
                        path,
                        staged,
                    });
                }
            } else {
                self.file = None;
            }
        }
        self.load_recent();
    }
    pub(super) fn closed_repository(&mut self, id: &str) {
        let active = self.snapshot.as_ref().is_some_and(|s| s.repo.id == id);
        let next = self.repositories.remove(id);
        if !active {
            return;
        }
        self.snapshot = None;
        self.commits.clear();
        self.rows.clear();
        self.visible.clear();
        self.selected = None;
        self.files.clear();
        self.file = None;
        self.diff.clear();
        self.editor = None;
        self.composer = Default::default();
        self.comparison = Default::default();
        self.filter.clear();
        self.branch_filter.clear();
        self.mode = Mode::History;
        if let Some(id) = next {
            self.open_target(OpenTarget::Activate(id));
        }
    }
    pub(super) fn repository_tabs(&mut self, ui: &mut egui::Ui) {
        let active = self.snapshot.as_ref().map(|s| s.repo.id.clone());
        let width = (ui.available_width() - 245.).max(120.);
        let count = ((width - 44.) / 212.).floor().max(1.) as usize;
        let tabs = self.repositories.visible(active.as_deref(), count);
        let mut target = None;
        ui.allocate_ui_with_layout(
            egui::vec2(width, 32.),
            Layout::left_to_right(Align::Center),
            |ui| {
                if self.modal_open() || self.action_request.is_some() {
                    ui.disable();
                }
                for tab in &tabs {
                    let selected = active.as_deref() == Some(tab.id.as_str());
                    let response = ui.add_sized(
                        [180., 32.],
                        egui::Button::new(theme::emphasis(format!("{}  {}", icon::CODE, tab.name)))
                            .fill(if selected {
                                theme::SELECTED.gamma_multiply(0.55)
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(egui::Stroke::NONE)
                            .truncate(),
                    );
                    if selected {
                        ui.painter().line_segment(
                            [response.rect.left_bottom(), response.rect.right_bottom()],
                            egui::Stroke::new(2., theme::MINT),
                        );
                    }
                    if response.clicked() && !selected {
                        target = Some(OpenTarget::Activate(tab.id.clone()));
                    }
                    response.context_menu(|ui| {
                        if ui.button("Close repository").clicked() {
                            target = Some(OpenTarget::Close(tab.id.clone()));
                            ui.close();
                        }
                        if ui.button("Copy repository path").clicked() {
                            ui.ctx().copy_text(tab.path.clone());
                            ui.close();
                        }
                    });
                    response.on_hover_text(&tab.path);
                    let close = ui.add_sized([20., 28.], egui::Button::new(icon::X).frame(false));
                    close.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            ui.is_enabled(),
                            format!("Close {}", tab.name),
                        )
                    });
                    if close.on_hover_text(format!("Close {}", tab.name)).clicked() {
                        target = Some(OpenTarget::Close(tab.id.clone()));
                    }
                }
                if tabs.len() < self.repositories.entries.len() {
                    let all = self.repositories.entries.clone();
                    ui.menu_button(format!("+{}", all.len() - tabs.len()), |ui| {
                        for tab in all {
                            if ui
                                .selectable_label(active.as_deref() == Some(&tab.id), &tab.name)
                                .on_hover_text(&tab.path)
                                .clicked()
                            {
                                target = Some(OpenTarget::Activate(tab.id));
                                ui.close();
                            }
                        }
                    });
                }
            },
        );
        if let Some(target) = target {
            self.ask_repository(target);
        }
    }
    pub(super) fn repository_menu(&mut self, ui: &mut egui::Ui) {
        let mut target = None;
        let recent = self.repositories.recent.clone();
        ui.menu_button(format!("{}  Open", icon::FOLDER_OPEN), |ui| {
            if ui.button("Open repository…").clicked() {
                target = Some(OpenTarget::Folder(None));
                ui.close();
            }
            ui.separator();
            ui.label(theme::caption("Recent repositories"));
            if recent.is_empty() {
                ui.label(theme::caption("Repositories you open will appear here."));
            }
            for path in recent {
                let name = PathBuf::from(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                if ui.button(name).on_hover_text(&path).clicked() {
                    target = Some(
                        self.repositories
                            .entries
                            .iter()
                            .find(|t| t.path == path)
                            .map_or(OpenTarget::Recent(path), |t| {
                                OpenTarget::Activate(t.id.clone())
                            }),
                    );
                    ui.close();
                }
            }
        });
        if let Some(target) = target {
            self.ask_repository(target);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_identity_updates_one_tab_and_overflow_keeps_active_visible() {
        let mut tabs = Tabs::default();
        for n in 0..6 {
            tabs.opened(&Repository {
                id: n.to_string(),
                name: format!("repo{n}"),
                path: format!("synthetic/{n}"),
                trusted: false,
            });
        }
        tabs.opened(&Repository {
            id: "5".into(),
            name: "renamed".into(),
            path: "synthetic/5".into(),
            trusted: true,
        });
        assert_eq!(tabs.entries.len(), 6);
        tabs.prompt = None;
        tabs.opened(&Repository {
            id: "5".into(),
            name: "renamed".into(),
            path: "synthetic/5".into(),
            trusted: false,
        });
        assert!(tabs.prompt.is_none());
        assert_eq!(tabs.visible(Some("5"), 2).len(), 2);
        assert_eq!(tabs.visible(Some("5"), 2)[1].name, "renamed");
        assert_eq!(tabs.remove("5"), Some("4".into()));
        assert_eq!(tabs.entries.len(), 5);
        assert!(tabs.entries.iter().all(|t| t.id != "5"));
        tabs.opened(&Repository {
            id: "5".into(),
            name: "reopened".into(),
            path: "synthetic/5".into(),
            trusted: false,
        });
        assert_eq!(tabs.prompt.as_deref(), Some("5"));
    }
}
