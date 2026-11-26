//! Kbd: a keyboard key cap (`.cu-kbd` / `.cu-kbd-group` in `custom.css`;
//! Carbon has no core equivalent).
//!
//! - **Appearance**: `.cu-kbd { min-width: 1.25rem; height: 1.25rem; padding:
//!   0 var(--spacing-02); border-radius: var(--border-radius-04); background:
//!   var(--layer-accent-01); box-shadow: inset 0 -1px 0 var(--border-strong-01);
//!   color: var(--text-primary); font-family: var(--font-mono); font-size:
//!   0.75rem; line-height: 1 }`. The bottom rule is an inset shadow, as in the
//!   CSS, so it takes no layout space and follows the 4px corners.
//! - `.cu-kbd-group { display: inline-flex; gap: var(--spacing-02); align-items: center }`.
//! - **Behavior**: none. Kit's `Kbd` formats a `Keystroke` into platform
//!   glyphs; here callers pass the label (`⌘`, `Ctrl`, …) as the reference does.
use crate::{
    theme::color,
    tokens::{self, BORDER_RADIUS_04, ColorRole, FontFamilyKey, SPACING_02},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, BoxShadow, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, px, rems,
};

/// Key cap geometry in px (`.cu-kbd`).
pub mod geometry {
    /// `min-width: 1.25rem; height: 1.25rem`.
    pub const SIZE: f32 = 20.;
    /// `padding: 0 var(--spacing-02)`.
    pub const PADDING_X: f32 = 4.;
    /// `border-radius: var(--border-radius-04)`.
    pub const RADIUS: f32 = 4.;
    /// `font-size: 0.75rem; line-height: 1`.
    pub const FONT_SIZE: f32 = 12.;
}

/// The key cap colors: (background, bottom rule, text).
pub fn colors() -> (Hsla, Hsla, Hsla) {
    (
        // `.cu-kbd { background: var(--layer-accent-01) }`.
        color(ColorRole::LayerAccent01),
        // `box-shadow: inset 0 -1px 0 var(--border-strong-01)`.
        color(ColorRole::BorderStrong01),
        // `color: var(--text-primary)`.
        color(ColorRole::TextPrimary),
    )
}

/// One key (`Kbd`).
#[derive(IntoElement)]
pub struct Kbd {
    label: SharedString,
}

impl Kbd {
    /// A key cap showing `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (background, rule, text) = colors();
        let size = rems(geometry::SIZE / tokens::REM_PX);
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .min_w(size)
            .h(size)
            .px(SPACING_02)
            .rounded(BORDER_RADIUS_04)
            .bg(background)
            .shadow(vec![BoxShadow::new(px(0.), px(-1.), rule).inset()])
            .text_color(text)
            .font(font(FontFamilyKey::Mono, 400))
            .text_size(rems(geometry::FONT_SIZE / tokens::REM_PX))
            .line_height(rems(geometry::FONT_SIZE / tokens::REM_PX))
            .whitespace_nowrap()
            .child(self.label)
    }
}

/// A row of keys and separators (`KbdGroup`): children are laid out inline
/// with a 4px gap, so `Kbd`, `"+"` and plain text mix as in the reference.
#[derive(IntoElement, Default)]
pub struct KbdGroup {
    children: Vec<AnyElement>,
}

impl KbdGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for KbdGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for KbdGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .gap(SPACING_02)
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn key_cap_follows_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (background, rule, text) = colors();
        // White: layer-accent-01 #e0e0e0, border-strong-01 #8d8d8d, text-primary #161616.
        assert_eq!(hex(background), "#e0e0e0");
        assert_eq!(hex(rule), "#8d8d8d");
        assert_eq!(hex(text), "#161616");
        assert_eq!(BORDER_RADIUS_04.px_value(), geometry::RADIUS);
        assert_eq!(SPACING_02.px_value(), geometry::PADDING_X);
        assert_eq!(geometry::SIZE, 20.);
    }
}
