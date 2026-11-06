use crate::{
    drafts::{self, Loaded, Scope},
    merge::{self, History},
    theme,
};
use eframe::egui::{self, Key, Modifiers, RichText, ScrollArea, TextEdit, TextStyle};
use egui_phosphor::regular as icon;
use ramo_core::git::ConflictFile;
use std::collections::BTreeSet;

pub enum Event {
    Save {
        scope: Scope,
        text: String,
        expected: Option<String>,
    },
    Resolve,
    Back,
}
pub struct Editor {
    pub scope: Scope,
    pub result: String,
    clean_text: String,
    pub dirty: bool,
    ours: String,
    theirs: String,
    base: Option<String>,
    history: History,
    ours_picks: BTreeSet<usize>,
    theirs_picks: BTreeSet<usize>,
    active: usize,
    cursor: usize,
    find: String,
    find_after: usize,
    find_range: Option<std::ops::Range<usize>>,
    saved: Option<Loaded>,
    pub notice: Option<String>,
    pub draft_busy: bool,
    pub draft_ready: bool,
}
impl Editor {
    pub fn new(scope: Scope, file: ConflictFile) -> Self {
        Self {
            scope,
            result: file.working.clone().unwrap_or_default(),
            clean_text: file.working.unwrap_or_default(),
            ours: file.ours.unwrap_or_default(),
            theirs: file.theirs.unwrap_or_default(),
            base: file.base,
            dirty: false,
            history: History::default(),
            ours_picks: BTreeSet::new(),
            theirs_picks: BTreeSet::new(),
            active: 0,
            cursor: 0,
            find: String::new(),
            find_after: 0,
            find_range: None,
            saved: None,
            notice: None,
            draft_busy: false,
            draft_ready: false,
        }
    }
    pub fn draft_loaded(&mut self, loaded: Option<Loaded>) {
        self.saved = loaded;
        self.draft_ready = true;
    }
    pub fn draft_saved(&mut self, loaded: Loaded) {
        // An acknowledgement may describe an older buffer than the current edits.
        self.clean_text = loaded.record.text.clone();
        self.dirty = self.result != self.clean_text;
        self.saved = Some(loaded);
        self.draft_busy = false;
        self.notice = Some("Draft saved locally. Repository file and index are unchanged.".into());
    }
    fn set_result(&mut self, next: String) {
        if next != self.result {
            self.history.record(&self.result);
            self.result = next;
            self.dirty = self.result != self.clean_text;
            self.find_range = None;
        }
    }
    fn use_lines(&mut self, ours: bool) {
        let text = if ours {
            merge::selected_lines(&self.ours, &self.ours_picks)
        } else {
            merge::selected_lines(&self.theirs, &self.theirs_picks)
        };
        if let Some(next) = merge::replace(&self.result, self.active, &text) {
            self.set_result(next);
        } else if merge::conflicts(&self.result).is_empty() {
            let offset = self
                .result
                .char_indices()
                .nth(self.cursor)
                .map_or(self.result.len(), |(i, _)| i);
            let mut next = self.result.clone();
            next.insert_str(offset, &text);
            self.set_result(next);
        }
        if ours {
            self.ours_picks.clear();
        } else {
            self.theirs_picks.clear();
        }
    }
    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<Event> {
        let mut event = None;
        let blocks = merge::conflicts(&self.result);
        self.active = self.active.min(blocks.len().saturating_sub(1));
        let mut locate = false;
        ui.horizontal(|ui| {
            if theme::button(ui, icon::ARROW_LEFT, "Back").clicked() {
                event = Some(Event::Back);
            }
            ui.separator();
            ui.label(theme::emphasis(&self.scope.path));
            ui.label(theme::caption(if blocks.is_empty() {
                "No complete conflict blocks remain".into()
            } else {
                format!("Conflict {} of {}", self.active + 1, blocks.len())
            }));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        !blocks.is_empty(),
                        egui::Button::new(format!("Next {}", icon::ARROW_RIGHT)),
                    )
                    .clicked()
                {
                    self.active = (self.active + 1) % blocks.len();
                    locate = true;
                }
                if ui
                    .add_enabled(
                        !blocks.is_empty(),
                        egui::Button::new(format!("{} Previous", icon::ARROW_LEFT)),
                    )
                    .clicked()
                {
                    self.active = (self.active + blocks.len() - 1) % blocks.len();
                    locate = true;
                }
            });
        });
        ui.separator();
        if let Some(saved) = self.saved.as_ref() {
            let differs = saved.record.text != self.result;
            let stale = saved.record.scope.fingerprint != self.scope.fingerprint;
            if differs {
                let restore = saved.record.text.clone();
                ui.horizontal(|ui| {
                    ui.label(theme::caption(if stale {
                        "Saved draft belongs to older Git state. Restore as a local copy only."
                    } else {
                        "A saved local draft is available."
                    }));
                    if ui.small_button("Restore draft").clicked() {
                        self.set_result(restore.clone());
                        self.clean_text = restore;
                        self.dirty = false;
                    }
                });
            }
        }
        if let Some(message) = self.notice.as_ref() {
            ui.label(RichText::new(message).size(12.).color(theme::AMBER));
        }
        ui.label(theme::caption(
            "Select source line numbers, then insert those lines into the result.",
        ));

        let source_height = (ui.available_height() * 0.32).clamp(120., 260.);
        ui.columns(2, |columns| {
            for (index, ours) in [(0, true), (1, false)] {
                columns[index].push_id(if ours { "ours" } else { "theirs" }, |ui| {
                    egui::Frame::new()
                        .fill(theme::PANEL)
                        .stroke(egui::Stroke::new(1., theme::BORDER))
                        .corner_radius(3.)
                        .inner_margin(8.)
                        .show(ui, |ui| {
                            ui.label(theme::emphasis(if ours {
                                "Current version"
                            } else {
                                "Incoming version"
                            }));
                            ui.separator();
                            egui::Frame::new().fill(theme::BASE).show(ui, |ui| {
                                if ours {
                                    source(
                                        ui,
                                        &self.ours,
                                        &mut self.ours_picks,
                                        source_height,
                                        "current",
                                    );
                                } else {
                                    source(
                                        ui,
                                        &self.theirs,
                                        &mut self.theirs_picks,
                                        source_height,
                                        "incoming",
                                    );
                                }
                            });
                            let count = if ours {
                                self.ours_picks.len()
                            } else {
                                self.theirs_picks.len()
                            };
                            ui.add_space(4.);
                            ui.horizontal(|ui| {
                                let label = if count > 0 {
                                    format!("{} Use selected lines ({count})", icon::ARROW_DOWN)
                                } else {
                                    format!("{} Use selected lines", icon::ARROW_DOWN)
                                };
                                if ui
                                    .add_enabled(count > 0, egui::Button::new(label))
                                    .clicked()
                                {
                                    self.use_lines(ours);
                                }
                            });
                        });
                });
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(theme::emphasis("Result · edit to resolve"));
            if theme::button(ui, icon::ARROW_COUNTER_CLOCKWISE, "Undo").clicked() {
                self.history.undo(&mut self.result);
                self.dirty = self.result != self.clean_text;
            }
            if theme::button(ui, icon::ARROW_CLOCKWISE, "Redo").clicked() {
                self.history.redo(&mut self.result);
                self.dirty = self.result != self.clean_text;
            }
            ui.separator();
            ui.add(
                TextEdit::singleline(&mut self.find)
                    .id(egui::Id::new("native_merge_find"))
                    .hint_text("Find in result")
                    .desired_width(180.),
            );
            if ui.small_button("Find next").clicked() && !self.find.is_empty() {
                let mut start = self.find_after.min(self.result.len());
                while !self.result.is_char_boundary(start) {
                    start -= 1;
                }
                let found = self.result[start..]
                    .find(&self.find)
                    .map(|n| n + start)
                    .or_else(|| self.result.find(&self.find));
                self.find_range = found.map(|start| start..start + self.find.len());
                self.find_after = self.find_range.as_ref().map_or(0, |r| r.end);
                locate = self.find_range.is_some();
            }
            ui.collapsing("Base", |ui| {
                ScrollArea::both().max_height(160.).show(ui, |ui| {
                    let mut text = self.base.as_deref().unwrap_or("No common ancestor text");
                    ui.add(TextEdit::multiline(&mut text).code_editor());
                });
            });
        });
        let editing_find = ctx.memory(|m| m.focused()) == Some(egui::Id::new("native_merge_find"));
        if ui.is_enabled()
            && !editing_find
            && ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z))
        {
            self.history.undo(&mut self.result);
            self.dirty = self.result != self.clean_text;
        }
        if ui.is_enabled()
            && !editing_find
            && ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z))
        {
            self.history.redo(&mut self.result);
            self.dirty = self.result != self.clean_text;
        }
        let save_shortcut =
            ui.is_enabled() && ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S));
        let can_save = self.draft_ready && !self.draft_busy && drafts::permitted(&self.scope.path);
        let save = egui::TopBottomPanel::bottom("editor_actions")
            .frame(egui::Frame::new().fill(theme::BASE).inner_margin(8.))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(theme::caption(
                        "Review the result, then save and stage this file.",
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::primary(ui, "Save and mark resolved…", blocks.is_empty())
                            .clicked()
                        {
                            event = Some(Event::Resolve);
                        }
                        ui.add_enabled(can_save, egui::Button::new("Save local draft"))
                            .clicked()
                    })
                    .inner
                })
                .inner
            })
            .inner;
        if can_save && (save || save_shortcut) {
            event = Some(Event::Save {
                scope: self.scope.clone(),
                text: self.result.clone(),
                expected: self.saved.as_ref().map(|s| s.revision.clone()),
            });
            self.draft_busy = true;
        }
        let active_range = self.find_range.clone().or_else(|| {
            merge::conflicts(&self.result)
                .get(self.active)
                .map(|b| b.range.clone())
        });
        let line = active_range.as_ref().map_or(0, |r| {
            self.result[..r.start]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
        });
        let before = self.result.clone();
        let mut area = ScrollArea::both()
            .id_salt("result_scroll")
            .auto_shrink([false, false]);
        if locate {
            area = area.vertical_scroll_offset(line as f32 * 20.);
        }
        egui::Frame::new()
            .fill(theme::BASE)
            .stroke(egui::Stroke::new(1., theme::BORDER))
            .inner_margin(4.)
            .show(ui, |ui| {
                area.show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.;
                        let count = self.result.bytes().filter(|b| *b == b'\n').count() + 1;
                        let (gutter, _) = ui.allocate_exact_size(
                            egui::vec2(38., count as f32 * 20.),
                            egui::Sense::hover(),
                        );
                        let start = ((ui.clip_rect().top() - gutter.top()) / 20.)
                            .floor()
                            .max(0.) as usize;
                        let end = ((ui.clip_rect().bottom() - gutter.top()) / 20.)
                            .ceil()
                            .max(0.) as usize;
                        for row in start..end.min(count) {
                            ui.painter().text(
                                gutter.min + egui::vec2(28., row as f32 * 20. + 10.),
                                egui::Align2::RIGHT_CENTER,
                                (row + 1).to_string(),
                                egui::FontId::monospace(12.),
                                theme::MUTED,
                            );
                        }
                        let active_range = active_range.clone();
                        let mut layouter =
                            |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _width: f32| {
                                ui.fonts_mut(|f| {
                                    f.layout_job(theme::code_job(
                                        buffer.as_str(),
                                        &BTreeSet::new(),
                                        active_range.clone(),
                                    ))
                                })
                            };
                        let output = TextEdit::multiline(&mut self.result)
                            .id_salt("result_buffer")
                            .font(TextStyle::Monospace)
                            .code_editor()
                            .frame(false)
                            .margin(egui::Margin::ZERO)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter)
                            .show(ui);
                        if let Some(range) = output.cursor_range {
                            self.cursor = range.primary.index;
                        }
                    });
                });
            });
        if self.result != before {
            self.history.record(&before);
            self.dirty = self.result != self.clean_text;
            self.find_range = None;
        }
        event
    }
}

fn source(ui: &mut egui::Ui, text: &str, picks: &mut BTreeSet<usize>, height: f32, side: &str) {
    ScrollArea::both()
        .max_height(height)
        .min_scrolled_height(height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.;
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(2., 0.);
                    ui.spacing_mut().interact_size.y = 20.;
                    let count = text.split_inclusive('\n').count();
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(42., count as f32 * 20.),
                        egui::Sense::hover(),
                    );
                    let start =
                        ((ui.clip_rect().top() - rect.top()) / 20.).floor().max(0.) as usize;
                    let end = ((ui.clip_rect().bottom() - rect.top()) / 20.)
                        .ceil()
                        .max(0.) as usize;
                    for row in start..end.min(count) {
                        let number = row + 1;
                        let row_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(0., row as f32 * 20.),
                            egui::vec2(42., 20.),
                        );
                        let response = ui
                            .push_id((side, number), |ui| {
                                ui.put(
                                    row_rect,
                                    egui::Button::selectable(
                                        picks.contains(&number),
                                        number.to_string(),
                                    )
                                    .frame_when_inactive(picks.contains(&number)),
                                )
                            })
                            .inner;
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                true,
                                picks.contains(&number),
                                format!("Select {side} line {number}"),
                            )
                        });
                        if response.clicked() && !picks.remove(&number) {
                            picks.insert(number);
                        }
                        response.on_hover_text(format!("Select {side} line {number}"));
                    }
                });
                let mut view = text;
                let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _width: f32| {
                    ui.fonts_mut(|f| f.layout_job(theme::code_job(buffer.as_str(), picks, None)))
                };
                ui.add(
                    TextEdit::multiline(&mut view)
                        .code_editor()
                        .frame(false)
                        .margin(egui::Margin::ZERO)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn editor() -> Editor {
        Editor::new(
            Scope {
                repository: "synthetic".into(),
                path: "src/search.rs".into(),
                fingerprint: "now".into(),
            },
            ConflictFile {
                base: Some("base\n".into()),
                ours: Some("current\n".into()),
                theirs: Some("incoming\n".into()),
                working: Some("initial buffer\n".into()),
            },
        )
    }
    #[test]
    fn late_draft_acknowledgement_preserves_newer_unsaved_edits() {
        let mut editor = editor();
        editor.set_result("submitted\r\n".into());
        let loaded = Loaded {
            record: drafts::Record {
                version: 1,
                scope: editor.scope.clone(),
                text: editor.result.clone(),
            },
            revision: "saved revision".into(),
        };
        editor.set_result("newer 日本語 without EOF".into());
        editor.draft_saved(loaded);
        assert_eq!(editor.result, "newer 日本語 without EOF");
        assert!(editor.dirty);
        editor.set_result("submitted\r\n".into());
        assert!(!editor.dirty);
    }
    #[test]
    fn loading_an_older_draft_requires_explicit_restore_and_never_replaces_the_buffer() {
        let mut editor = editor();
        let mut old = editor.scope.clone();
        old.fingerprint = "old state".into();
        editor.draft_loaded(Some(Loaded {
            record: drafts::Record {
                version: 1,
                scope: old,
                text: "old resolution".into(),
            },
            revision: "old revision".into(),
        }));
        assert_eq!(editor.result, "initial buffer\n");
        assert!(!editor.dirty);
        assert!(editor.draft_ready);
    }
}
