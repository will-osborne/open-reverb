use egui::{Color32, Context, FontFamily, FontId, RichText, Stroke, TextStyle, Visuals};

// Color scheme
pub const ACCENT_COLOR: Color32 = Color32::from_rgb(88, 101, 242); // Discord-like blue
pub const ACCENT_HOVER_COLOR: Color32 = Color32::from_rgb(71, 82, 196);
pub const BACKGROUND_COLOR: Color32 = Color32::from_rgb(54, 57, 63);
pub const SECONDARY_BACKGROUND: Color32 = Color32::from_rgb(47, 49, 54);
pub const TEXT_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
pub const SECONDARY_TEXT_COLOR: Color32 = Color32::from_rgb(185, 187, 190);
pub const ERROR_COLOR: Color32 = Color32::from_rgb(237, 66, 69);
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(59, 165, 93);
pub const AWAY_COLOR: Color32 = Color32::from_rgb(250, 166, 26);
pub const DND_COLOR: Color32 = Color32::from_rgb(237, 66, 69);
pub const OFFLINE_COLOR: Color32 = Color32::from_rgb(116, 127, 141);

// Status colors
pub fn status_color(status: open_reverb_common::models::UserStatus) -> Color32 {
    match status {
        open_reverb_common::models::UserStatus::Online => SUCCESS_COLOR,
        open_reverb_common::models::UserStatus::Away => AWAY_COLOR,
        open_reverb_common::models::UserStatus::DoNotDisturb => DND_COLOR,
        open_reverb_common::models::UserStatus::Offline => OFFLINE_COLOR,
    }
}

// Apply the OpenReverb theme to the UI context
pub fn setup_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    
    // Configure text styles
    style.text_styles = [
        (TextStyle::Heading, FontId::new(26.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
    ]
    .into();
    
    // Set up dark theme
    let mut visuals = Visuals::dark();
    
    // Customize colors
    visuals.widgets.noninteractive.bg_fill = BACKGROUND_COLOR;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_COLOR);
    
    visuals.widgets.inactive.bg_fill = SECONDARY_BACKGROUND;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, SECONDARY_TEXT_COLOR);
    
    visuals.widgets.active.bg_fill = ACCENT_COLOR;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_COLOR);
    
    visuals.widgets.hovered.bg_fill = ACCENT_HOVER_COLOR;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_COLOR);
    
    // Window colors
    visuals.window_fill = BACKGROUND_COLOR;
    visuals.panel_fill = BACKGROUND_COLOR;
    
    // Misc
    visuals.window_shadow.extrusion = 8.0;
    visuals.popup_shadow.extrusion = 8.0;
    
    style.visuals = visuals;
    
    ctx.set_style(style);
}

// Helper functions for text styling
pub fn heading(text: &str) -> RichText {
    RichText::new(text).color(TEXT_COLOR).size(24.0).strong()
}

pub fn subheading(text: &str) -> RichText {
    RichText::new(text).color(TEXT_COLOR).size(18.0).strong()
}

pub fn body_text(text: &str) -> RichText {
    RichText::new(text).color(TEXT_COLOR).size(16.0)
}

pub fn secondary_text(text: &str) -> RichText {
    RichText::new(text).color(SECONDARY_TEXT_COLOR).size(16.0)
}

pub fn error_text(text: &str) -> RichText {
    RichText::new(text).color(ERROR_COLOR).size(16.0)
}

pub fn success_text(text: &str) -> RichText {
    RichText::new(text).color(SUCCESS_COLOR).size(16.0)
}