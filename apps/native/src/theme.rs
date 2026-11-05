use eframe::egui::{self, Color32, FontId, RichText, TextStyle};
pub const BASE: Color32 = Color32::from_rgb(23, 27, 32);
pub const PANEL: Color32 = Color32::from_rgb(30, 37, 44);
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
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
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
        widget.bg_stroke = egui::Stroke::new(1.0, BORDER);
    }
    style.visuals.widgets.inactive.bg_fill = PANEL;
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 53, 62);
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
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(18.));
    ctx.set_style(style);
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
