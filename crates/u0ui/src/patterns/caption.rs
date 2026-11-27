//! The caption line over a pattern example (`.cu-pcap`): `label-01` in
//! `text-secondary`, 8px above what it describes.
use crate::{
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

/// `.cu-pcap { margin-bottom: var(--spacing-03) }`.
pub const CAPTION_GAP: tokens::TokenLength = tokens::SPACING_03;

/// A pattern caption. The caller passes the text.
#[derive(IntoElement)]
pub struct Caption {
    text: SharedString,
    gap: bool,
}

impl Caption {
    /// A caption with its 8px bottom margin.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            gap: true,
        }
    }

    /// Drop the bottom margin (a caption set inline, centered on a row).
    pub fn inline(mut self) -> Self {
        self.gap = false;
        self
    }
}

impl RenderOnce for Caption {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut el = div()
            // `.cu-pcap { font-size: 0.75rem; letter-spacing: 0.32px; line-height: 1.33333 }`
            // = `label-01`.
            .type_style(TypeStyle::Label01)
            // `.cu-pcap { color: var(--text-secondary) }`.
            .text_color(color(ColorRole::TextSecondary))
            .child(self.text);
        if self.gap {
            el = el.mb(CAPTION_GAP);
        }
        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caption_gap_is_spacing_03() {
        assert_eq!(CAPTION_GAP.px_value(), 8.);
    }
}
