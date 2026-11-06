use super::*;

impl App {
    pub(super) fn chrome(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs")
            .exact_height(46.)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon::GIT_BRANCH).size(23.).color(theme::MINT));
                    ui.label(RichText::new("Ramo").size(22.).strong());
                    ui.add_space(20.);
                    if let Some(snapshot) = &self.snapshot {
                        let name = snapshot.repo.name.clone();
                        ui.add(
                            egui::Button::new(format!("{}  {name}", icon::CODE))
                                .selected(true)
                                .frame(false),
                        )
                        .on_hover_text(&snapshot.repo.path);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::button(ui, icon::FOLDER_OPEN, "Open repository").clicked() {
                            self.ask_open();
                        }
                        ui.label(theme::caption("Native preview"));
                    });
                });
            });
        egui::TopBottomPanel::top("commands").exact_height(40.).frame(egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(14,5))).show(ctx,|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon::GIT_BRANCH).color(theme::MINT));
                ui.strong(self.snapshot.as_ref().map_or("No repository",|s|s.branch.as_str()));
                ui.separator();
                if let Some(snapshot)=&self.snapshot {
                    ui.label(theme::caption(if snapshot.operation.is_some(){"Operation in progress".into()}
                        else if snapshot.files.is_empty(){format!("{} Clean",icon::CHECK)}else{format!("{} changed files",snapshot.files.len())}));
                }
                if theme::button(ui,icon::ARROWS_CLOCKWISE,"Refresh").clicked(){self.refresh();}
                ui.separator();
                let ready=self.snapshot.is_some()&&self.action_request.is_none()&&self.action.is_none()&&self.editor.is_none();
                for (glyph,label,action,detail) in [
                    (icon::ARROW_DOWN,"Fetch",Action::Fetch,"Fetch updates from configured remotes."),
                    (icon::ARROW_DOWN,"Pull",Action::Pull,"Fast-forward the current branch from its upstream. Divergence is refused."),
                    (icon::ARROW_UP,"Push",Action::Push,"Push the current branch to its configured upstream. No force push.")
                ] {
                    if ui.add_enabled(ready,egui::Button::new(format!("{glyph} {label}")).frame(false)).clicked() {
                        self.request_action(action,label,detail,label!="Fetch");
                    }
                }
                ui.separator();
                for (glyph,label) in [(icon::GIT_BRANCH,"Branch"),(icon::ARCHIVE,"Stash")] {
                    if ui.add_enabled(ready,egui::Button::new(format!("{glyph} {label}")).frame(false)).clicked(){
                        self.form=Some(actions::Form{kind:if label=="Branch"{actions::FormKind::Branch}else{actions::FormKind::Stash},text:String::new()});
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                    ui.menu_button(format!("{}  View",icon::SLIDERS_HORIZONTAL),|ui|{
                        ui.checkbox(&mut self.inspector,"Show commit inspector");
                        ui.checkbox(&mut self.compact,"Compact rows (22 px)");
                    });
                    if !self.pending.is_empty(){ui.spinner();}
                });
            });
        });
        egui::TopBottomPanel::bottom("status")
            .exact_height(27.)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 3)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(theme::caption(format!(
                        "{} commits loaded · {} rows drawn",
                        self.commits.len(),
                        self.rendered_rows
                    )));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(theme::caption("Rust / egui · 0.3 development"));
                    });
                });
            });
        if let Some(error) = self.error.clone() {
            egui::TopBottomPanel::top("error").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(icon::WARNING_CIRCLE).color(theme::AMBER));
                    ui.label(RichText::new(error).color(theme::AMBER));
                    if ui.small_button("Dismiss").clicked() {
                        self.error = None;
                    }
                });
            });
        }
    }
    pub(super) fn sidebar(&mut self, ctx: &egui::Context) {
        let Some(snapshot) = self.snapshot.clone() else {
            return;
        };
        egui::SidePanel::left("objects").resizable(true).default_width(215.).width_range(170.0..=360.0)
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(12.)).show(ctx,|ui| {
            ui.add(TextEdit::singleline(&mut self.branch_filter).hint_text("Filter branches").desired_width(f32::INFINITY));
            ui.add_space(10.);
            ScrollArea::vertical().show(ui,|ui|{
                for remote in [false,true] {
                    let branches:Vec<_>=snapshot.branches.iter().filter(|b|b.remote==remote&&b.name.to_lowercase().contains(&self.branch_filter.to_lowercase())).collect();
                    let title=if remote{"Remote branches"}else{"Local branches"};
                    egui::CollapsingHeader::new(format!("{title}   {}",branches.len())).default_open(!remote).show(ui,|ui| {
                        for branch in branches {
                            let selected=!branch.remote&&branch.name==snapshot.branch;
                            let response=ui.add(egui::Button::selectable(selected,format!("{}  {}",icon::GIT_BRANCH,branch.name)).min_size(egui::vec2(ui.available_width(),28.)).truncate());
                            if response.clicked() {
                                self.mode=Mode::History;
                                self.filter.clear();self.refilter();
                                self.scroll_target=self.commits.iter().position(|c|c.oid==branch.oid);
                                self.select(branch.oid.clone());
                            }
                            if response.double_clicked() && !remote && !selected {
                                self.request_action(Action::Switch{name:branch.name.clone()},"Checkout branch",format!("Switch to {}",branch.name),false);
                            }
                            response.context_menu(|ui|{
                                if !remote {
                                    if ui.button("Checkout").clicked(){self.request_action(Action::Switch{name:branch.name.clone()},"Checkout branch",format!("Switch to {}",branch.name),false);ui.close();}
                                    if ui.button("Merge into current branch…").clicked(){self.request_action(Action::Merge{name:branch.name.clone()},"Merge branch",format!("Merge {} into {}. Conflicts open in Changes.",branch.name,snapshot.branch),true);ui.close();}
                                    if ui.button("Rename branch…").clicked(){self.form=Some(actions::Form{kind:actions::FormKind::Rename(branch.name.clone()),text:branch.name.clone()});ui.close();}
                                    if ui.button("Delete merged branch…").clicked(){self.request_action(Action::DeleteBranch{name:branch.name.clone()},"Delete branch",format!("Delete {} only if safely merged. Worktree branches are protected.",branch.name),true);ui.close();}
                                }
                                if remote && ui.button("Create tracking branch…").clicked(){self.form=Some(actions::Form{kind:actions::FormKind::Track(branch.name.clone()),text:branch.name.split_once('/').map_or(branch.name.clone(),|(_,name)|name.to_owned())});ui.close();}
                                if ui.button("Copy branch name").clicked(){ui.ctx().copy_text(branch.name.clone());ui.close();}
                            });
                            response.on_hover_text(if remote {"Inspect remote branch"}else{"Click to inspect · double-click to checkout · right-click for actions"});
                        }
                    });ui.add_space(9.);ui.separator();
                }
                egui::CollapsingHeader::new(format!("Worktrees   {}",snapshot.worktrees.len())).default_open(true).show(ui,|ui|{
                    for worktree in &snapshot.worktrees {
                        let name=std::path::Path::new(&worktree.path).file_name().unwrap_or_default().to_string_lossy();
                        ui.label(format!("{}  {name}",icon::FOLDER_OPEN)).on_hover_text(&worktree.path);
                        ui.label(theme::caption(worktree.branch.strip_prefix("refs/heads/").unwrap_or(&worktree.branch)));
                    }
                });ui.separator();
                egui::CollapsingHeader::new(format!("Stashes   {}",snapshot.stashes.len())).show(ui,|ui|{
                    for stash in &snapshot.stashes {
                        ui.label(&stash.message).context_menu(|ui|{
                            for (label,action) in [("Apply…",Action::ApplyStash{index:stash.index,oid:stash.oid.clone()}),("Pop…",Action::PopStash{index:stash.index,oid:stash.oid.clone()}),("Drop…",Action::DropStash{index:stash.index,oid:stash.oid.clone()})] {
                                if ui.button(label).clicked(){self.request_action(action,label,format!("{} {} ({})",label,stash.reference,stash.message),true);ui.close();}
                            }
                        });
                    }
                });
            });
        });
    }
}
