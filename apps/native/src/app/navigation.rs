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
                for (glyph,label) in [(icon::ARROW_DOWN,"Fetch"),(icon::ARROW_DOWN,"Pull"),(icon::ARROW_UP,"Push")] {
                    ui.add_enabled(false,egui::Button::new(format!("{glyph} {label}")).frame(false))
                        .on_disabled_hover_text("Available in the released Ramo beta. Native write workflows are a later migration slice.");
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
                        "{} commits loaded · {} rows drawn · Read-only Git",
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
                            response.on_hover_text("Inspect this branch tip. Checkout is not enabled in this native slice.");
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
                    ui.label(theme::caption("Use the released beta to inspect or apply stashes."));
                });
            });
        });
    }
}
