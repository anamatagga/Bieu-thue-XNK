use egui::{Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

// ── Catppuccin Mocha (Hyprdots default) ─────────────────────────────────────
#[allow(non_snake_case)]
pub mod C {
    use egui::Color32;
    pub const BASE:     Color32 = Color32::from_rgb(30,  30,  46);
    pub const MANTLE:   Color32 = Color32::from_rgb(24,  24,  37);
    pub const CRUST:    Color32 = Color32::from_rgb(17,  17,  27);
    pub const SURFACE0: Color32 = Color32::from_rgb(49,  50,  68);
    pub const SURFACE1: Color32 = Color32::from_rgb(69,  71,  90);
    pub const SURFACE2: Color32 = Color32::from_rgb(88,  91,  112);
    pub const OVERLAY0: Color32 = Color32::from_rgb(108, 112, 134);
    pub const OVERLAY1: Color32 = Color32::from_rgb(127, 132, 156);
    pub const SUBTEXT0: Color32 = Color32::from_rgb(166, 173, 200);
    pub const SUBTEXT1: Color32 = Color32::from_rgb(186, 194, 222);
    pub const TEXT:     Color32 = Color32::from_rgb(205, 214, 244);
    pub const LAVENDER: Color32 = Color32::from_rgb(180, 190, 254);
    pub const BLUE:     Color32 = Color32::from_rgb(137, 180, 250);
    pub const SAPPHIRE: Color32 = Color32::from_rgb(116, 199, 236);
    pub const TEAL:     Color32 = Color32::from_rgb(148, 226, 213);
    pub const GREEN:    Color32 = Color32::from_rgb(166, 227, 161);
    pub const YELLOW:   Color32 = Color32::from_rgb(249, 226, 175);
    pub const PEACH:    Color32 = Color32::from_rgb(250, 179, 135);
    pub const RED:      Color32 = Color32::from_rgb(243, 139, 168);
    pub const MAUVE:    Color32 = Color32::from_rgb(203, 166, 247);
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "antigravity_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/font.ttf")),
    );
    // Put our font first for both proportional and monospace text
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, "antigravity_font".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.insert(0, "antigravity_font".to_owned());
    }
    ctx.set_fonts(fonts);

    let mut style = Style {
        text_styles: [
            (TextStyle::Small,    FontId::new(11.0, FontFamily::Proportional)),
            (TextStyle::Body,     FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Button,   FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Heading,  FontId::new(17.0, FontFamily::Proportional)),
            (TextStyle::Monospace,FontId::new(13.0, FontFamily::Monospace)),
        ].into(),
        ..Default::default()
    };

    style.spacing.item_spacing     = egui::vec2(8.0, 5.0);
    style.spacing.button_padding   = egui::vec2(14.0, 7.0);
    style.spacing.window_margin    = egui::Margin::same(12.0);
    style.spacing.scroll.bar_width = 6.0;

    style.visuals = build_visuals();
    ctx.set_style(style);
}

fn build_visuals() -> Visuals {
    let mut v = Visuals::dark();

    v.window_fill   = C::BASE;
    v.panel_fill    = C::MANTLE;
    v.window_stroke = Stroke::new(1.0, C::SURFACE1);
    v.window_rounding = Rounding::same(12.0);

    // Window shadow (egui 0.29 field names)
    v.window_shadow = egui::Shadow {
        offset: egui::Vec2::new(0.0, 6.0),
        blur:   20.0,
        spread: 2.0,
        color:  Color32::from_black_alpha(90),
    };

    // Widget states
    let mk = |fill: Color32, stroke_col: Color32, r: f32| egui::style::WidgetVisuals {
        bg_fill:   fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, stroke_col),
        rounding:  Rounding::same(r),
        fg_stroke: Stroke::new(1.5, C::TEXT),
        expansion: 0.0,
    };

    v.widgets.noninteractive = mk(C::SURFACE0, C::SURFACE1, 8.0);
    v.widgets.inactive        = mk(C::SURFACE0, C::SURFACE1, 8.0);
    v.widgets.hovered         = mk(C::SURFACE1, C::LAVENDER, 8.0);
    v.widgets.active          = mk(C::SURFACE2, C::SURFACE2, 8.0);
    v.widgets.open            = mk(C::SURFACE1, C::LAVENDER, 8.0);

    v.selection.bg_fill = C::SURFACE1;
    v.selection.stroke  = Stroke::new(1.0, C::SURFACE2);

    v.override_text_color = Some(C::TEXT);
    v.hyperlink_color     = C::BLUE;
    v.menu_rounding       = Rounding::same(10.0);

    v
}

/// Color for a tax rate string
pub fn rate_color(val: &str) -> Color32 {
    let val = val.trim();
    if val.is_empty() || val == "0" { return C::OVERLAY1; }
    if val == "*"                   { return C::SAPPHIRE; }
    // "*/5/8/10" — pick first numeric token
    let first = val.split('/').find_map(|s| {
        let s = s.trim();
        if s.is_empty() || s == "*" { None } else { s.replace(',', ".").parse::<f32>().ok() }
    });
    match first {
        None        => C::TEXT,
        Some(0.0)                => C::OVERLAY1,
        Some(n) if n <= 5.0      => C::GREEN,
        Some(n) if n <= 15.0     => C::YELLOW,
        Some(n) if n <= 30.0     => C::PEACH,
        _                        => C::RED,
    }
}
