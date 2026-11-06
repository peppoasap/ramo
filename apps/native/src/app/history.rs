use super::*;

impl App {
    pub(super) fn history(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(theme::caption(format!("{} commits", self.visible.len())));
            if self.has_more
                && ui
                    .add_enabled(
                        self.page_request.is_none(),
                        egui::Button::new("Load more").frame(false),
                    )
                    .clicked()
            {
                self.load_more();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_sized(
                        [ui.available_width().min(260.), 28.],
                        TextEdit::singleline(&mut self.filter)
                            .hint_text("Find commit, author or hash"),
                    )
                    .changed()
                {
                    self.refilter();
                    self.scroll_target = Some(0);
                }
            });
        });
        ui.add_space(3.);
        let row_height = if self.compact { ROW } else { 30. };
        let graph_width =
            (self.rows.iter().map(|r| r.lanes).max().unwrap_or(1) as f32 * 18. + 24.).max(72.);
        let width = ui
            .available_width()
            .max(150. + graph_width + 76. + 220. + 126. + 90.);
        let subject_width = width - 150. - graph_width - 76. - 126. - 90.;
        let columns = [
            0.,
            150.,
            150. + graph_width,
            226. + graph_width,
            226. + graph_width + subject_width,
            width - 90.,
        ];
        let mut choose = None;
        let mut command = None;
        let mut endpoint = None;
        let mut amend = None;
        let mut form = None;
        let clean = self
            .snapshot
            .as_ref()
            .is_some_and(|s| s.operation.is_none() && s.files.is_empty());
        let available = self
            .snapshot
            .as_ref()
            .is_some_and(|s| s.operation.is_none());
        let mut auto_more = false;
        let selected_position = self.selected.as_ref().and_then(|oid| {
            self.visible
                .iter()
                .position(|&i| &self.commits[i].oid == oid)
        });
        if !ui.ctx().wants_keyboard_input() && !self.visible.is_empty() {
            let page = (ui.available_height() / row_height).floor().max(1.) as usize;
            let next = ui.input(|i| {
                let current = selected_position.unwrap_or(0);
                if i.key_pressed(Key::ArrowDown) {
                    Some((current + 1).min(self.visible.len() - 1))
                } else if i.key_pressed(Key::ArrowUp) {
                    Some(current.saturating_sub(1))
                } else if i.key_pressed(Key::PageDown) {
                    Some((current + page).min(self.visible.len() - 1))
                } else if i.key_pressed(Key::PageUp) {
                    Some(current.saturating_sub(page))
                } else if i.key_pressed(Key::Home) {
                    Some(0)
                } else if i.key_pressed(Key::End) {
                    Some(self.visible.len() - 1)
                } else {
                    None
                }
            });
            if let Some(position) = next {
                choose = Some(self.commits[self.visible[position]].oid.clone());
                self.scroll_target = Some(position);
            }
        }
        ScrollArea::horizontal()
            .id_salt("history_horizontal")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(width);
                let (header, _) = ui.allocate_exact_size(egui::vec2(width, 28.), Sense::hover());
                for (x, label) in columns.iter().zip([
                    "BRANCH / TAG",
                    "GRAPH",
                    "COMMIT",
                    "SUBJECT",
                    "AUTHOR",
                    "DATE",
                ]) {
                    paint_text(
                        ui.painter(),
                        egui::pos2(header.left() + x + 8., header.center().y - 7.),
                        label,
                        120.,
                        12.,
                        theme::MUTED,
                        false,
                    );
                }
                ui.painter().line_segment(
                    [header.left_bottom(), header.right_bottom()],
                    egui::Stroke::new(1., theme::BORDER),
                );
                ui.spacing_mut().item_spacing.y = 0.;
                let mut scroll = ScrollArea::vertical()
                    .id_salt("history_rows")
                    .auto_shrink([false, false]);
                if let Some(index) = self.scroll_target.take() {
                    scroll = scroll.vertical_scroll_offset(index as f32 * row_height);
                }
                let branches = self
                    .snapshot
                    .as_ref()
                    .map(|s| s.branches.clone())
                    .unwrap_or_default();
                let current = self
                    .snapshot
                    .as_ref()
                    .map(|s| s.branch.clone())
                    .unwrap_or_default();
                let refs: HashMap<_, Vec<_>> =
                    branches.into_iter().fold(HashMap::new(), |mut map, b| {
                        map.entry(b.oid.clone()).or_default().push(b);
                        map
                    });
                scroll.show_rows(ui, row_height, self.visible.len(), |ui, range| {
                    self.rendered_rows = range.len();
                    if range.end + 20 >= self.visible.len() && self.filter.is_empty() {
                        auto_more = true;
                    }
                    for position in range {
                        let index = self.visible[position];
                        let commit = &self.commits[index];
                        let row = &self.rows[index];
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(width, row_height), Sense::click());
                        let selected = self.selected.as_deref() == Some(&commit.oid);
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                true,
                                selected,
                                format!(
                                    "{} · {} · {}",
                                    commit.subject,
                                    commit.author,
                                    short(&commit.oid)
                                ),
                            )
                        });
                        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
                        if selected {
                            painter.rect_filled(rect, 0., theme::SELECTED);
                        } else if response.hovered() {
                            painter.rect_filled(rect, 0., theme::PANEL);
                        }
                        painter.line_segment(
                            [rect.left_bottom(), rect.right_bottom()],
                            egui::Stroke::new(0.5, theme::BORDER.gamma_multiply(0.4)),
                        );
                        let y = rect.center().y;
                        let mut labels: Vec<_> =
                            refs.get(&commit.oid).into_iter().flatten().collect();
                        labels.sort_by_key(|b| {
                            (b.remote || b.name != current, b.remote, b.name.clone())
                        });
                        let mut names: Vec<String> =
                            labels.iter().map(|b| b.name.clone()).collect();
                        names.extend(
                            commit
                                .refs
                                .split(", ")
                                .filter_map(|r| r.strip_prefix("tag: "))
                                .map(|tag| format!("tag: {tag}")),
                        );
                        if let Some(name) = names.first() {
                            let count = if names.len() > 1 {
                                format!(" +{}", names.len() - 1)
                            } else {
                                String::new()
                            };
                            paint_text(
                                &painter,
                                egui::pos2(rect.left() + 7., y - 7.),
                                &format!("{name}{count}"),
                                138.,
                                13.,
                                theme::LANES[row.lane % 5],
                                false,
                            );
                        }
                        let (before, after) = if self.filter.is_empty() {
                            (true, true)
                        } else {
                            graph::adjacent(&self.visible, position)
                        };
                        let lane_x =
                            |lane: usize| rect.left() + columns[1] + 16. + lane as f32 * 18.;
                        if row.incoming && before {
                            painter.line_segment(
                                [
                                    egui::pos2(lane_x(row.lane), rect.top()),
                                    egui::pos2(lane_x(row.lane), y),
                                ],
                                egui::Stroke::new(2., theme::LANES[row.lane % 5]),
                            );
                        }
                        if after {
                            for edge in &row.edges {
                                let color = theme::LANES[edge.to % 5];
                                let from = egui::pos2(
                                    lane_x(edge.from),
                                    if edge.from_node { y } else { rect.top() },
                                );
                                let to = egui::pos2(lane_x(edge.to), rect.bottom());
                                if edge.from == edge.to {
                                    painter.line_segment([from, to], egui::Stroke::new(2., color));
                                } else {
                                    let mid = (from.y + to.y) / 2.;
                                    painter.add(
                                        egui::epaint::CubicBezierShape::from_points_stroke(
                                            [
                                                from,
                                                egui::pos2(from.x, mid),
                                                egui::pos2(to.x, mid),
                                                to,
                                            ],
                                            false,
                                            Color32::TRANSPARENT,
                                            egui::Stroke::new(2., color),
                                        ),
                                    );
                                }
                            }
                        }
                        painter.circle_filled(
                            egui::pos2(lane_x(row.lane), y),
                            3.8,
                            theme::LANES[row.lane % 5],
                        );
                        paint_text(
                            &painter,
                            egui::pos2(rect.left() + columns[2] + 8., y - 7.),
                            short(&commit.oid),
                            68.,
                            12.,
                            theme::MUTED,
                            true,
                        );
                        paint_text(
                            &painter,
                            egui::pos2(rect.left() + columns[3] + 8., y - 7.),
                            &commit.subject,
                            subject_width - 12.,
                            13.,
                            theme::TEXT,
                            false,
                        );
                        paint_text(
                            &painter,
                            egui::pos2(rect.left() + columns[4] + 8., y - 7.),
                            &commit.author,
                            116.,
                            12.,
                            theme::MUTED,
                            false,
                        );
                        paint_text(
                            &painter,
                            egui::pos2(rect.left() + columns[5] + 8., y - 7.),
                            date(&commit.date),
                            80.,
                            12.,
                            theme::MUTED,
                            false,
                        );
                        if response.clicked() {
                            choose = Some(commit.oid.clone());
                            response.request_focus();
                        }
                        response.context_menu(|ui| {
                            if ui
                                .button(format!("{}  Copy commit ID", icon::COPY))
                                .clicked()
                            {
                                ui.ctx().copy_text(commit.oid.clone());
                                ui.close();
                            }
                            for (label, from) in [("Compare from this commit", true), ("Compare to this commit", false)] {
                                if ui.button(label).clicked() { endpoint = Some((from, commit.oid.clone())); ui.close(); }
                            }
                            if ui.button("Copy commit subject").clicked(){ui.ctx().copy_text(commit.subject.clone());ui.close();}
                            ui.separator();
                            let is_head=self.snapshot.as_ref().and_then(|s|s.head.as_ref())==Some(&commit.oid);
                            if ui.add_enabled(available&&is_head,egui::Button::new("Amend latest commit…")).on_disabled_hover_text("Only the current local commit can be amended, with no operation in progress.").clicked(){amend=Some(commit.oid.clone());ui.close();}
                            if ui
                                .add_enabled(available, egui::Button::new("Create branch here…"))
                                .clicked()
                            {
                                form = Some(actions::Form {
                                    kind: actions::FormKind::BranchAt(commit.oid.clone()),
                                    text: String::new(),
                                });
                                ui.close();
                            }
                            if ui
                                .add_enabled(available, egui::Button::new("Create tag here…"))
                                .clicked()
                            {
                                form = Some(actions::Form {
                                    kind: actions::FormKind::TagAt(commit.oid.clone()),
                                    text: String::new(),
                                });
                                ui.close();
                            }
                            for (label, action) in [
                                (
                                    "Checkout commit…",
                                    Action::CheckoutCommit {
                                        target: commit.oid.clone(),
                                    },
                                ),
                                (
                                    "Cherry-pick commit…",
                                    Action::CherryPick {
                                        target: commit.oid.clone(),
                                    },
                                ),
                                (
                                    "Revert commit…",
                                    Action::Revert {
                                        target: commit.oid.clone(),
                                    },
                                ),
                            ] {
                                if ui
                                    .add_enabled(clean, egui::Button::new(label))
                                    .on_disabled_hover_text(
                                        "Requires a clean repository and no operation in progress.",
                                    )
                                    .clicked()
                                {
                                    command = Some((label, action, commit.oid.clone()));
                                    ui.close();
                                }
                            }
                            if !names.is_empty() {
                                ui.separator();
                                for name in &names {
                                    ui.label(format!("{} {name}", icon::GIT_BRANCH));
                                }
                            }
                        });
                        response.on_hover_text(format!(
                            "{}\n{}\n{}",
                            commit.subject, commit.oid, commit.refs
                        ));
                    }
                });
            });
        if let Some((from, oid)) = endpoint {
            self.compare_endpoint(from, oid);
        }
        if let Some(head) = amend
            && let Some(snapshot) = &self.snapshot
        {
            self.amend_request = self.send(Work::AmendMessage {
                repo: snapshot.repo.id.clone(),
                head,
            });
        }
        if let Some(form) = form {
            self.form = Some(form);
        }
        if let Some((label, action, oid)) = command {
            let detail = match &action {
                Action::CheckoutCommit { .. } => format!(
                    "Checkout {} with detached HEAD. Create a branch before adding new commits.",
                    short(&oid)
                ),
                Action::CherryPick { .. } => format!(
                    "Apply {} to the current branch as a new commit. Conflicts open in Changes.",
                    short(&oid)
                ),
                _ => format!(
                    "Create a new commit reverting {} on the current branch. Conflicts open in Changes.",
                    short(&oid)
                ),
            };
            self.request_action(action, label, detail, true);
        }
        if let Some(oid) = choose {
            self.select(oid);
        }
        if auto_more {
            self.load_more();
        }
    }
}
