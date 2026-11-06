//! Renderer glue for internally typed graph drag payloads.
use super::*;
use crate::gestures::{Drag, Intent};
impl App {
    pub(super) fn guard_drag(&mut self, ctx: &egui::Context) {
        let busy = self.action_request.is_some() || self.action.is_some() || self.editor.is_some();
        tick_drag(
            ctx,
            self.snapshot.as_ref(),
            busy,
            &mut self.drag_candidate,
            &mut self.drag_frame,
        );
    }
    pub(super) fn apply_drop(&mut self, drag: &Drag, target: Option<&str>) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let busy = self.action_request.is_some() || self.action.is_some() || self.editor.is_some();
        match drag.intent(snapshot, target, busy) {
            Ok(Intent::Compare { from, to }) => {
                self.compare_pair(from, to);
            }
            Ok(Intent::Review {
                action,
                title,
                detail,
            }) => self.request_action(action, title, detail, true),
            Err(error) => self.error = Some(error),
        }
    }
    pub(super) fn finish_drag(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.pointer.primary_released())
            && self.branch_drop_rect.is_some_and(|rect| {
                ctx.input(|i| i.pointer.interact_pos().is_some_and(|p| rect.contains(p)))
            })
            && let Some(drag) = egui::DragAndDrop::take_payload::<Drag>(ctx)
        {
            self.apply_drop(&drag, None);
            ctx.request_repaint();
        }
    }
    pub(super) fn drag_overlay(&mut self, ctx: &egui::Context) {
        let Some(drag) = egui::DragAndDrop::payload::<Drag>(ctx) else {
            return;
        };
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let branch = snapshot.branch.clone();
        let intent = drag.intent(snapshot, None, false);
        egui::Area::new(egui::Id::new("graph_drag_intent"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, [0., 78.])
            .show(ctx, |ui| {
                egui::Frame::popup(&ctx.style())
                    .fill(theme::PANEL)
                    .inner_margin(8.)
                    .show(ui, |ui| {
                        ui.set_max_width((ctx.content_rect().width() - 48.).clamp(240., 420.));
                        ui.add(
                            egui::Label::new(theme::caption(format!(
                                "Dragging {}",
                                drag.source.label()
                            )))
                            .truncate(),
                        )
                        .on_hover_text(drag.source.label());
                        let verb = match &drag.source {
                            crate::gestures::Source::Branch { .. } => "Drop to review merge into",
                            _ => "Drop to review cherry-pick onto",
                        };
                        let response = ui.add(egui::Button::new(format!("{verb} {branch}")).fill(
                            if intent.is_ok() {
                                theme::SELECTED
                            } else {
                                theme::PANEL
                            },
                        ));
                        if response.dnd_hover_payload::<Drag>().is_some() {
                            ui.painter().rect_stroke(
                                response.rect,
                                3.,
                                egui::Stroke::new(
                                    1.5,
                                    if intent.is_ok() {
                                        theme::MINT
                                    } else {
                                        theme::AMBER
                                    },
                                ),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if let Some(drag) = response.dnd_release_payload::<Drag>() {
                            self.apply_drop(&drag, None);
                        }
                        if let Err(error) = intent {
                            ui.label(theme::caption(error));
                        } else {
                            ui.label(theme::caption(
                                "Or drop on a commit to compare · Escape cancels",
                            ));
                        }
                    });
            });
    }
}

// Pointer press/release may arrive in a single native input batch. The raw press
// event retains the origin even after PointerState::press_origin was cleared.
pub(super) fn press_origin(ctx: &egui::Context) -> Option<egui::Pos2> {
    ctx.input(|i| {
        if !i.pointer.primary_pressed() {
            return None;
        }
        i.raw
            .events
            .iter()
            .rev()
            .find_map(|e| match e {
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    ..
                } => Some(*pos),
                _ => None,
            })
            .or_else(|| i.pointer.press_origin())
    })
}
pub(super) fn begin_drag(
    ctx: &egui::Context,
    candidate: crate::gestures::Candidate,
    pending: &mut Option<crate::gestures::Candidate>,
    active: &mut bool,
) {
    let (focused, down, released, pos) = ctx.input(|i| {
        (
            i.focused,
            i.pointer.primary_down(),
            i.pointer.primary_released(),
            i.pointer.interact_pos(),
        )
    });
    if !focused
        || ctx.input(|i| {
            i.raw
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerGone))
        })
    {
        return;
    }
    if (down || released) && pos.is_some_and(|p| candidate.moved([p.x, p.y])) {
        egui::DragAndDrop::set_payload(ctx, candidate.drag);
        *active = true;
        *pending = None;
    } else if down {
        *pending = Some(candidate);
    }
}

fn tick_drag(
    ctx: &egui::Context,
    snapshot: Option<&Snapshot>,
    busy: bool,
    pending: &mut Option<crate::gestures::Candidate>,
    active: &mut bool,
) {
    *active = false;
    let (focused, escape, down, released, pos) = ctx.input(|i| {
        (
            i.focused,
            i.key_pressed(Key::Escape),
            i.pointer.primary_down(),
            i.pointer.primary_released(),
            i.pointer.interact_pos(),
        )
    });
    let cancelled = escape
        || ctx.input(|i| {
            i.raw
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerGone))
        });
    if let Some(candidate) = pending.take() {
        let valid =
            snapshot.is_some_and(|s| candidate.drag.can_continue(s, busy, focused)) && !cancelled;
        if valid && (down || released) {
            if pos.is_some_and(|p| candidate.moved([p.x, p.y])) {
                egui::DragAndDrop::set_payload(ctx, candidate.drag);
            } else if down {
                *pending = Some(candidate);
            }
        }
    }
    if let Some(drag) = egui::DragAndDrop::payload::<Drag>(ctx) {
        let valid = snapshot.is_some_and(|s| drag.can_continue(s, busy, focused)) && !cancelled;
        if !valid {
            egui::DragAndDrop::clear_payload(ctx);
        } else {
            *active = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gestures::{Candidate, Source};
    fn snapshot() -> Snapshot {
        Snapshot {
            repo: git::Repository {
                id: "repo".into(),
                path: "synthetic".into(),
                name: "Meadow".into(),
                trusted: false,
            },
            branch: "main".into(),
            head: Some("head".into()),
            upstream: None,
            ahead: None,
            behind: None,
            files: vec![],
            branches: vec![],
            worktrees: vec![],
            stashes: vec![],
            fingerprint: "revision".into(),
            operation: None,
        }
    }
    fn candidate(snapshot: &Snapshot) -> Candidate {
        Candidate {
            drag: Drag::new(
                snapshot,
                Source::Commit {
                    oid: "captured-source".into(),
                    parents: 1,
                    subject: "Synthetic change".into(),
                },
            ),
            origin: [10., 10.],
        }
    }
    fn escape() -> egui::Event {
        egui::Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn batched_press_and_release_preserve_source_without_drag_started_widget() {
        let ctx = egui::Context::default();
        let snapshot = snapshot();
        let input = egui::RawInput {
            focused: true,
            events: vec![
                egui::Event::PointerMoved(egui::pos2(10., 10.)),
                egui::Event::PointerButton {
                    pos: egui::pos2(10., 10.),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerMoved(egui::pos2(200., 100.)),
                egui::Event::PointerButton {
                    pos: egui::pos2(200., 100.),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            assert!(ctx.input(|i| i.pointer.press_origin().is_none()));
            assert_eq!(press_origin(ctx), Some(egui::pos2(10., 10.)));
            let mut pending = None;
            let mut active = false;
            begin_drag(ctx, candidate(&snapshot), &mut pending, &mut active);
            assert!(active);
            assert!(pending.is_none());
            let payload = egui::DragAndDrop::take_payload::<Drag>(ctx).unwrap();
            assert_eq!(payload.source.oid(), "captured-source");
        });
    }
    #[test]
    fn focus_loss_stale_snapshot_busy_and_escape_clear_active_or_pending_drag() {
        let ctx = egui::Context::default();
        let snapshot = snapshot();
        for (focused, busy, stale) in [
            (false, false, false),
            (true, true, false),
            (true, false, true),
        ] {
            let mut current = snapshot.clone();
            if stale {
                current.fingerprint = "new revision".into();
            }
            let mut pending = Some(candidate(&snapshot));
            let mut active = true;
            egui::DragAndDrop::set_payload(&ctx, candidate(&snapshot).drag);
            let _ = ctx.run(
                egui::RawInput {
                    focused,
                    ..Default::default()
                },
                |ctx| {
                    tick_drag(ctx, Some(&current), busy, &mut pending, &mut active);
                    assert!(pending.is_none());
                    assert!(!active);
                    assert!(egui::DragAndDrop::payload::<Drag>(ctx).is_none());
                },
            );
        }
        let mut pending = Some(candidate(&snapshot));
        let mut active = false;
        egui::DragAndDrop::set_payload(&ctx, candidate(&snapshot).drag);
        let _ = ctx.run(
            egui::RawInput {
                focused: true,
                events: vec![egui::Event::PointerGone],
                ..Default::default()
            },
            |ctx| {
                tick_drag(ctx, Some(&snapshot), false, &mut pending, &mut active);
                assert!(pending.is_none());
                assert!(!active);
                assert!(egui::DragAndDrop::payload::<Drag>(ctx).is_none());
            },
        );
        pending = Some(candidate(&snapshot));
        let _ = ctx.run(
            egui::RawInput {
                focused: true,
                events: vec![escape()],
                ..Default::default()
            },
            |ctx| {
                tick_drag(ctx, Some(&snapshot), false, &mut pending, &mut active);
                assert!(pending.is_none());
                assert!(!active);
            },
        );
        egui::DragAndDrop::set_payload(&ctx, candidate(&snapshot).drag);
        let _ = ctx.run(
            egui::RawInput {
                focused: true,
                events: vec![escape()],
                ..Default::default()
            },
            |ctx| {
                tick_drag(ctx, Some(&snapshot), false, &mut pending, &mut active);
                assert!(egui::DragAndDrop::payload::<Drag>(ctx).is_none());
            },
        );
    }
}
