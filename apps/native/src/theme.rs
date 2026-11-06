use eframe::egui::{self, Color32, FontId, RichText, TextStyle};
pub const BASE: Color32 = Color32::from_rgb(23, 27, 32);
pub const PANEL: Color32 = Color32::from_rgb(28, 35, 41);
pub const HOVER: Color32 = Color32::from_rgb(38, 47, 54);
pub const STRONG: &str = "Ramo Semibold";
pub const BORDER: Color32 = Color32::from_rgb(48, 59, 69);
pub const TEXT: Color32 = Color32::from_rgb(232, 237, 243);
pub const MUTED: Color32 = Color32::from_rgb(166, 177, 192);
pub const MINT: Color32 = Color32::from_rgb(123, 216, 172);
pub const SELECTED: Color32 = Color32::from_rgb(39, 67, 55);
pub const AMBER: Color32 = Color32::from_rgb(230, 185, 104);
pub const RED: Color32 = Color32::from_rgb(239, 142, 145);
pub const LANES: [Color32; 5] = [
    MINT,
    Color32::from_rgb(113, 182, 232),
    AMBER,
    Color32::from_rgb(183, 155, 233),
    Color32::from_rgb(226, 150, 187),
];

pub fn install(ctx: &egui::Context) {
    // Pin the selected graphite workbench even when the OS prefers light mode.
    // Otherwise eframe can switch to its unstyled light palette after startup.
    ctx.set_theme(egui::Theme::Dark);
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "noto-semibold".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSans-SemiBold.ttf")).into(),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "noto".into());
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    let mut strong_fallback = vec!["noto-semibold".into()];
    strong_fallback.extend(fonts.families[&egui::FontFamily::Proportional].clone());
    fonts
        .families
        .insert(egui::FontFamily::Name(STRONG.into()), strong_fallback);
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = BASE;
    style.visuals.window_fill = PANEL;
    style.visuals.extreme_bg_color = BASE;
    style.visuals.faint_bg_color = PANEL;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.selection.bg_fill = SELECTED;
    style.visuals.selection.stroke.color = MINT;
    style.visuals.window_corner_radius = egui::CornerRadius::same(6);
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::same(3);
        widget.fg_stroke = egui::Stroke::new(1.0, TEXT);
        widget.bg_stroke = egui::Stroke::new(1.0, BORDER);
    }
    style.visuals.widgets.noninteractive.fg_stroke.color = MUTED;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.bg_fill = HOVER;
        widget.weak_bg_fill = PANEL;
    }
    style.visuals.widgets.hovered.weak_bg_fill = HOVER;
    style.visuals.widgets.active.weak_bg_fill = SELECTED;
    style.visuals.widgets.open.weak_bg_fill = SELECTED;
    style.visuals.text_cursor.stroke.color = MINT;
    style.visuals.widgets.active.bg_stroke.color = MINT;
    style.visuals.widgets.hovered.bg_stroke.color = MUTED;
    style.visuals.menu_corner_radius = egui::CornerRadius::same(4);
    style.spacing.item_spacing = egui::vec2(8., 5.);
    style.spacing.button_padding = egui::vec2(8., 5.);
    style.spacing.interact_size.y = 28.;
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(14.));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(12.));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(13.));
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(17., egui::FontFamily::Name(STRONG.into())),
    );
    ctx.set_style_of(egui::Theme::Dark, style);
}
/// Underlined workbench tabs use the same hit target without adding a button tile.
pub fn tab(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let response = ui.add(
        egui::Button::new(if selected {
            emphasis(label).color(TEXT)
        } else {
            RichText::new(label).color(MUTED)
        })
        .frame(false)
        .min_size(egui::vec2(72., 30.)),
    );
    if selected {
        ui.painter().line_segment(
            [response.rect.left_bottom(), response.rect.right_bottom()],
            egui::Stroke::new(2., MINT),
        );
    }
    response
}
pub fn emphasis(text: impl Into<String>) -> RichText {
    RichText::new(text).family(egui::FontFamily::Name(STRONG.into()))
}
pub fn button(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    ui.add(egui::Button::new(format!("{icon}  {label}")).frame(false))
}
pub fn caption(text: impl Into<String>) -> RichText {
    RichText::new(text).size(12.).color(MUTED)
}
pub fn code_job(
    text: &str,
    selected: &std::collections::BTreeSet<usize>,
    active: Option<std::ops::Range<usize>>,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    let mut offset = 0;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        let background = if selected.contains(&(i + 1)) {
            SELECTED
        } else if active
            .as_ref()
            .is_some_and(|r| r.start < offset + line.len() && r.end > offset)
        {
            Color32::from_rgb(61, 51, 33)
        } else {
            Color32::TRANSPARENT
        };
        job.append(
            line,
            0.,
            egui::TextFormat {
                font_id: FontId::monospace(13.),
                color: TEXT,
                background,
                line_height: Some(20.),
                ..Default::default()
            },
        );
        offset += line.len();
    }
    job
}

pub fn primary(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).strong().color(BASE))
            .fill(MINT)
            .stroke(egui::Stroke::NONE),
    )
}
pub fn modal<R>(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    width: f32,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::ModalResponse<R> {
    egui::Modal::new(egui::Id::new(id))
        .frame(
            egui::Frame::popup(&ctx.style())
                .fill(PANEL)
                .inner_margin(16.),
        )
        .show(ctx, |ui| {
            ui.set_width(width.min((ctx.content_rect().width() - 48.).max(240.)));
            ui.heading(title);
            ui.add_space(12.);
            body(ui)
        })
}
