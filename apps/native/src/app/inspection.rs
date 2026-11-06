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
        let busy = self.action_request.is_some() || self.action.is_some();
        ui.horizontal(|ui| {
            ui.heading(if snapshot.operation.is_some() {
                "Operation in progress"
            } else {
                "Changes"
            });
            if let Some(operation) = &snapshot.operation {
                ui.label(theme::caption(operation));
            }
        });
        ui.separator();
        let mut resolve = None;
        egui::SidePanel::left("changed_files")
            .default_width(280.)
            .resizable(true)
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for staged in [false, true] {
                        ui.strong(if staged {
                            "Staged changes"
                        } else {
                            "Working changes"
                        });
                        for file in &snapshot.files {
                            let conflict = git::is_conflicted(file);
                            let in_section = if staged {
                                !file.index.trim().is_empty() && file.index != "?" && !conflict
                            } else {
                                !file.worktree.trim().is_empty() || file.index == "?" || conflict
                            };
                            if !in_section {
                                continue;
                            }
                            ui.horizontal(|ui| {
                                if ui
                                    .selectable_label(
                                        self.file.as_deref() == Some(&file.path)
                                            && self.staged_view == staged,
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
                                    self.staged_view = staged;
                                    self.diff.clear();
                                    self.diff_request = self.send(Work::Diff {
                                        repo: snapshot.repo.id.clone(),
                                        oid: None,
                                        path: file.path.clone(),
                                        staged,
                                    });
                                }
                                if !conflict
                                    && ui
                                        .add_enabled(
                                            !busy,
                                            egui::Button::new(if staged { "−" } else { "+" })
                                                .small(),
                                        )
                                        .on_hover_text(if staged {
                                            "Unstage file"
                                        } else {
                                            "Stage file"
                                        })
                                        .clicked()
                                {
                                    let paths = vec![file.path.clone()];
                                    self.request_action(
                                        if staged {
                                            Action::Unstage { paths }
                                        } else {
                                            Action::Stage { paths }
                                        },
                                        if staged { "Unstage file" } else { "Stage file" },
                                        file.path.clone(),
                                        false,
                                    );
                                }
                            });
                            if conflict && ui.small_button("Resolve in editor").clicked() {
                                resolve = Some(file.path.clone());
                            }
                        }
                        ui.add_space(12.);
                        ui.separator();
                    }
                    if snapshot.files.is_empty() {
                        ui.label(theme::caption(if snapshot.operation.is_some() {
                            "No unresolved files. Complete or abort the operation."
                        } else {
                            "Working tree is clean."
                        }));
                    }
                });
            });
        egui::CentralPanel::default().frame(egui::Frame::NONE).show_inside(ui,|ui|{
            egui::TopBottomPanel::bottom("commit_composer").resizable(true).default_height(155.).show_inside(ui,|ui|{
                ui.strong("Commit message");
                if ui.add_enabled(self.composer.ready,TextEdit::multiline(&mut self.composer.text).hint_text("Describe your staged changes").desired_rows(3).desired_width(f32::INFINITY)).changed(){self.composer.changed();}
                self.composer_status(ui);
                ui.horizontal(|ui|{
                    let conflicts=snapshot.files.iter().any(git::is_conflicted);
                    let staged=snapshot.files.iter().any(|f|!f.index.trim().is_empty()&&f.index!="?");
                    let merge=snapshot.operation.as_deref()==Some("Merge in progress");
                    if theme::primary(ui,if merge {"Complete merge"}else{"Commit staged changes"},!busy&&!conflicts&&(snapshot.operation.is_none()||merge)&&(staged||merge)&&!self.composer.text.trim().is_empty()).clicked(){
                        self.request_action(Action::Commit{message:self.composer.text.clone()},"Commit staged changes","Commit only the index using this message.",false);
                    }
                    if merge&&ui.add_enabled(!busy,egui::Button::new("Abort merge…")).clicked(){self.request_action(Action::AbortMerge,"Abort merge","Abort the merge and restore the pre-merge state. Review local work first.",true);}
                    if conflicts {ui.label(theme::caption("Resolve each conflicted file before committing."));}
                    let sequence=match snapshot.operation.as_deref(){Some("Cherry-pick in progress")=>Some(git::SequenceKind::CherryPick),Some("Revert in progress")=>Some(git::SequenceKind::Revert),_=>None};
                    if let Some(operation)=sequence {
                        let ready=!busy&&!conflicts&&!snapshot.files.iter().any(|f|f.index!="?"&&f.worktree!=" ");
                        for (label,step,enabled) in [("Continue operation",git::SequenceStep::Continue,ready),("Skip current commit…",git::SequenceStep::Skip,!busy),("Abort operation…",git::SequenceStep::Abort,!busy)] {
                            if ui.add_enabled(enabled,egui::Button::new(label)).on_disabled_hover_text("Resolve and stage tracked changes before continuing.").clicked(){
                                self.request_action(Action::Sequence{operation,step},label,match step {git::SequenceStep::Continue=>"Continue the reviewed sequence using the staged resolution.",git::SequenceStep::Skip=>"Skip the current commit and discard its tracked resolution changes; the next step can conflict.",git::SequenceStep::Abort=>"Abort the whole sequence and restore its starting state. Review local work first."},step!=git::SequenceStep::Continue);
                            }
                        }
                    }
                });
            });
            if let Some(path)=&self.file {ui.label(theme::caption(format!("{} · {path}",if self.staged_view {"Staged"}else{"Working"})));}
            if !self.diff.is_empty() && let Some(path)=self.file.clone()
                && let Some(event)=self.patch_view.ui(ui,&self.diff,self.staged_view,snapshot.operation.as_deref()) {
                    let hash=git::hash(&self.diff);
                    let action=match event {
                        crate::patch_view::Event::Hunk(index)=>Action::Hunk{path:path.clone(),index,unstage:self.staged_view,patch_hash:hash},
                        crate::patch_view::Event::Lines(rows)=>Action::Lines{path:path.clone(),rows,unstage:self.staged_view,patch_hash:hash},
                    };
                    self.request_action(action,"Update staged selection",path,false);
            }
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
