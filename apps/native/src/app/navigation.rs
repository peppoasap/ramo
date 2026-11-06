use super::*;

impl App {
    pub(super) fn chrome(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs")
            .exact_height(44.)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon::GIT_BRANCH).size(23.).color(theme::MINT));
                    ui.label(theme::emphasis("Ramo").size(22.));
                    ui.add_space(20.);
                    if let Some(snapshot) = &self.snapshot {
                        let name = snapshot.repo.name.clone();
                        let tab = ui.add(
                            egui::Button::new(theme::emphasis(format!("{}  {name}", icon::CODE)))
                                .fill(theme::SELECTED.gamma_multiply(0.55))
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(180., 32.)),
                        );
                        ui.painter().line_segment(
                            [tab.rect.left_bottom(), tab.rect.right_bottom()],
                            egui::Stroke::new(2., theme::MINT),
                        );
                        tab.on_hover_text(&snapshot.repo.path);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.modal_open() || self.action_request.is_some() {
                            ui.disable();
                        }
                        if theme::button(ui, icon::FOLDER_OPEN, "Open repository").clicked() {
                            self.ask_open();
                        }
                        if theme::button(ui, icon::CLOUD_ARROW_DOWN, "Clone").clicked() {
                            self.ask_clone();
                        }
                    });
                });
            });
        egui::TopBottomPanel::top("commands").exact_height(40.).frame(egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(14,5))).show(ctx,|ui| {
            ui.horizontal(|ui| {
                if self.modal_open()||self.action_request.is_some(){ui.disable();}
                ui.label(RichText::new(icon::GIT_BRANCH).color(theme::MINT));
                let branch_target=ui.label(theme::emphasis(self.snapshot.as_ref().map_or("No repository",|s|s.branch.as_str())));
                self.branch_drop_rect=Some(branch_target.rect.expand2(egui::vec2(8.,5.)));
                branch_target.on_hover_text("Drop another local branch here to review a merge, or a commit to review cherry-pick.");
                ui.separator();
                if let Some(snapshot)=&self.snapshot {
                    ui.label(theme::caption(if snapshot.operation.is_some(){"Operation in progress".into()}
                        else if snapshot.files.is_empty(){format!("{} Clean",icon::CHECK)}else{format!("{} changed files",snapshot.files.len())}));
                }
                if theme::button(ui,icon::ARROWS_CLOCKWISE,"Refresh").clicked(){self.refresh();}
                ui.separator();
                if ui.add_enabled(self.snapshot.is_some()&&!self.clone_busy(),egui::Button::new(format!("{} Remotes",icon::GIT_BRANCH)).frame(false)).clicked(){self.open_remotes();}
                let ready=self.snapshot.is_some()&&!self.clone_busy()&&self.action_request.is_none()&&self.action.is_none()&&self.editor.is_none();
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
                    if self.mode == Mode::History && self.snapshot.is_some() && self.editor.is_none() {
                        let width=(ui.available_width()-8.).clamp(90.,260.);
                        if ui.add_sized([width,28.],TextEdit::singleline(&mut self.filter).hint_text("Find commit, author or hash")).changed() {
                            self.refilter(); self.scroll_target=Some(0);
                        }
                    }
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
                        "{} commits loaded",
                        self.commits.len()
                    )));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(theme::caption("Native preview · 0.3 development"));
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
            if self.action_request.is_some()||self.modal_open(){ui.disable();}
            ui.add(TextEdit::singleline(&mut self.branch_filter).hint_text("Filter branches").desired_width(f32::INFINITY));
            ui.add_space(10.);
            ScrollArea::vertical().show(ui,|ui|{
                for remote in [false,true] {
                    let branches:Vec<_>=snapshot.branches.iter().filter(|b|b.remote==remote&&b.name.to_lowercase().contains(&self.branch_filter.to_lowercase())).collect();
                    let title=if remote{"Remote branches"}else{"Local branches"};
                    egui::CollapsingHeader::new(theme::emphasis(format!("{title}   {}",branches.len()))).default_open(!remote).show(ui,|ui| {
                        for branch in branches {
                            let selected=!branch.remote&&branch.name==snapshot.branch;
                            let response=ui.add(egui::Button::selectable(selected,format!("{}  {}  {}",icon::GIT_BRANCH,branch.name,if selected{icon::CHECK}else{""})).sense(Sense::click_and_drag()).min_size(egui::vec2(ui.available_width(),28.)).truncate());
                            if !self.modal_open() && self.action_request.is_none() && self.editor.is_none() && let Some(origin)=gestures::press_origin(ui.ctx()).filter(|p|response.rect.intersect(ui.clip_rect()).contains(*p)) {
                                let candidate=crate::gestures::Candidate{drag:crate::gestures::Drag::new(&snapshot,crate::gestures::Source::Branch{oid:branch.oid.clone(),name:branch.name.clone(),remote:branch.remote}),origin:[origin.x,origin.y]};
                                gestures::begin_drag(ui.ctx(),candidate,&mut self.drag_candidate,&mut self.drag_frame);
                            }
                            if response.clicked() && !self.drag_frame {
                                self.mode=Mode::History;
                                self.filter.clear();self.refilter();
                                self.scroll_target=self.commits.iter().position(|c|c.oid==branch.oid);
                                self.select(branch.oid.clone());
                            }
                            if response.double_clicked() && !self.drag_frame && !remote && !selected {
                                self.request_action(Action::Switch{name:branch.name.clone()},"Checkout branch",format!("Switch to {}",branch.name),false);
                            }
                            response.context_menu(|ui|{
                                if ui.button("Compare from this branch").clicked(){self.compare_endpoint(true,branch.oid.clone());ui.close();}
                                if ui.button("Compare to this branch").clicked(){self.compare_endpoint(false,branch.oid.clone());ui.close();}
                                ui.separator();
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
                egui::CollapsingHeader::new(theme::emphasis(format!("Worktrees   {}",snapshot.worktrees.len()))).default_open(true).show(ui,|ui|{
                    for worktree in &snapshot.worktrees {
                        let name=std::path::Path::new(&worktree.path).file_name().unwrap_or_default().to_string_lossy();
                        ui.label(format!("{}  {name}",icon::FOLDER_OPEN)).on_hover_text(&worktree.path);
                        ui.label(theme::caption(worktree.branch.strip_prefix("refs/heads/").unwrap_or(&worktree.branch)));
                    }
                });ui.separator();
                egui::CollapsingHeader::new(theme::emphasis(format!("Stashes   {}",snapshot.stashes.len()))).show(ui,|ui|{
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
