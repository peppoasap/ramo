//! Native selectable unified diff; row identities always refer to the displayed Git diff.
use crate::theme;
use eframe::egui::{self, RichText, ScrollArea};
use std::collections::BTreeSet;
#[derive(Default)]
pub struct Selection {
    identity: String,
    rows: BTreeSet<usize>,
    anchor: Option<usize>,
}
pub enum Event {
    Hunk(usize),
    Lines(Vec<usize>),
}
fn changed(line: &str, in_hunk: bool) -> bool {
    in_hunk && (line.starts_with('+') || line.starts_with('-'))
}
fn numbers(lines: &[&str]) -> Vec<(Option<usize>, Option<usize>)> {
    let mut old = 0;
    let mut new = 0;
    let mut active = false;
    lines
        .iter()
        .map(|line| {
            if let Some(header) = line.strip_prefix("@@ -") {
                let parsed = header.split_once(" +").and_then(|(a, b)| {
                    Some((
                        a.split(',').next()?.parse::<usize>().ok()?,
                        b.split([',', ' ']).next()?.parse::<usize>().ok()?,
                    ))
                });
                if let Some((a, b)) = parsed {
                    old = a;
                    new = b;
                    active = true;
                } else {
                    active = false;
                }
                (None, None)
            } else if active {
                let result = match line.as_bytes().first() {
                    Some(b' ') => (Some(old), Some(new)),
                    Some(b'-') => (Some(old), None),
                    Some(b'+') => (None, Some(new)),
                    _ => (None, None),
                };
                if result.0.is_some() {
                    old += 1;
                }
                if result.1.is_some() {
                    new += 1;
                }
                result
            } else {
                (None, None)
            }
        })
        .collect()
}
impl Selection {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        diff: &str,
        staged: bool,
        operation: Option<&str>,
    ) -> Option<Event> {
        let identity = ramo_core::git::hash(diff);
        if identity != self.identity {
            self.identity = identity;
            self.rows.clear();
            self.anchor = None;
        }
        let mut event = None;
        let clipped = diff.lines().take(12001).count() > 12000
            || diff.lines().any(|l| l.chars().count() > 4096);
        let allowed = if operation.is_some() {
            Err("Use whole-file staging while a Git operation is in progress.".into())
        } else if clipped {
            Err("Partial staging requires an untruncated preview. Use whole-file staging for this large diff.".into())
        } else {
            ramo_core::git::partial_staging_available(diff)
        };
        let line_allowed = allowed.is_ok() && !diff.lines().any(|l| l.starts_with("\\ No newline"));
        let verb = if staged { "Unstage" } else { "Stage" };
        ui.horizontal(|ui| {
            ui.label(theme::caption("Select + / − rows; Shift selects a range."));
            if ui
                .add_enabled(
                    line_allowed && !self.rows.is_empty(),
                    egui::Button::new(format!("{verb} selected ({})", self.rows.len())),
                )
                .clicked()
            {
                event = Some(Event::Lines(self.rows.iter().copied().collect()));
            }
            if !self.rows.is_empty() && ui.small_button("Clear").clicked() {
                self.rows.clear();
            }
        });
        if let Err(reason) = &allowed {
            ui.label(theme::caption(reason));
        } else if !line_allowed {
            ui.label(theme::caption(
                "No final newline: stage whole hunks or the file.",
            ));
        }
        let lines: Vec<_> = diff.split_inclusive('\n').take(12001).collect();
        let first = lines
            .iter()
            .position(|l| l.starts_with("@@ -"))
            .unwrap_or(lines.len());
        let line_numbers = numbers(&lines);
        let mut hunks = Vec::with_capacity(lines.len());
        let mut hunk: usize = 0;
        for line in &lines {
            if line.starts_with("@@ -") {
                hunk += 1;
            }
            hunks.push(hunk.saturating_sub(1));
        }
        if lines.len() > 12000 {
            ui.label(theme::caption("Preview limited to 12,000 lines."));
        }
        ui.spacing_mut().item_spacing.y = 0.;
        ScrollArea::both()
            .id_salt("selectable_diff")
            .auto_shrink([false, false])
            .show_rows(ui, 22., lines.len().min(12000), |ui, range| {
                ui.spacing_mut().interact_size.y = 22.;
                ui.spacing_mut().button_padding.y = 1.;
                for index in range {
                    let line = lines[index].trim_end_matches(['\r', '\n']);
                    let content = changed(line, index > first);
                    ui.horizontal(|ui| {
                        ui.set_height(22.);
                        let (gutter, _) =
                            ui.allocate_exact_size(egui::vec2(124., 22.), egui::Sense::hover());
                        if line.starts_with("@@ -") {
                            if ui
                                .add_enabled_ui(allowed.is_ok(), |ui| {
                                    ui.put(
                                        gutter,
                                        egui::Button::new(format!(
                                            "{verb} hunk {}",
                                            hunks[index] + 1
                                        ))
                                        .small(),
                                    )
                                })
                                .inner
                                .clicked()
                            {
                                event = Some(Event::Hunk(hunks[index]));
                            }
                        } else if content {
                            let picked = self.rows.contains(&index);
                            let response = ui
                                .add_enabled_ui(line_allowed, |ui| {
                                    ui.put(
                                        egui::Rect::from_min_size(gutter.min, egui::vec2(25., 22.)),
                                        egui::Button::selectable(
                                            picked,
                                            if picked {
                                                egui_phosphor::regular::CHECK
                                            } else {
                                                line.get(..1).unwrap_or(" ")
                                            },
                                        )
                                        .min_size(egui::vec2(25., 22.)),
                                    )
                                })
                                .inner;
                            response.widget_info(|| {
                                egui::WidgetInfo::selected(
                                    egui::WidgetType::Checkbox,
                                    line_allowed && ui.is_enabled(),
                                    picked,
                                    format!("Select diff row {}", index + 1),
                                )
                            });
                            if response.clicked() {
                                if ui.input(|i| i.modifiers.shift)
                                    && let Some(anchor) = self.anchor
                                {
                                    for (n, line) in lines
                                        .iter()
                                        .enumerate()
                                        .take(anchor.max(index) + 1)
                                        .skip(anchor.min(index))
                                    {
                                        if changed(line, n > first) {
                                            self.rows.insert(n);
                                        }
                                    }
                                } else if !self.rows.remove(&index) {
                                    self.rows.insert(index);
                                }
                                self.anchor = Some(index);
                            }
                        }
                        if !line.starts_with("@@ -") {
                            for (offset, value) in
                                [(78., line_numbers[index].0), (120., line_numbers[index].1)]
                            {
                                if let Some(n) = value {
                                    ui.painter().text(
                                        egui::pos2(gutter.left() + offset, gutter.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        n,
                                        egui::FontId::monospace(11.5),
                                        theme::MUTED,
                                    );
                                }
                            }
                        }
                        let color = if content && line.starts_with('+') {
                            theme::MINT
                        } else if content {
                            theme::RED
                        } else if line.starts_with("@@") {
                            theme::LANES[1]
                        } else {
                            theme::MUTED
                        };
                        let mut text = line.chars().take(4096).collect::<String>();
                        if line.chars().count() > 4096 {
                            text.push('…');
                        }
                        ui.add(
                            egui::Label::new(
                                RichText::new(text).monospace().size(12.5).color(color),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    });
                }
            });
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gutters_follow_original_and_new_file_positions_across_hunks() {
        let lines = [
            "diff --git a/a b/a\n",
            "@@ -2,2 +2,3 @@\n",
            " same\n",
            "-old\n",
            "+new\n",
            "+added\n",
            "@@ -20 +21 @@ function\n",
            "-last\n",
            "+replacement\n",
        ];
        assert_eq!(
            numbers(&lines),
            vec![
                (None, None),
                (None, None),
                (Some(2), Some(2)),
                (Some(3), None),
                (None, Some(3)),
                (None, Some(4)),
                (None, None),
                (Some(20), None),
                (None, Some(21))
            ]
        );
    }
}
