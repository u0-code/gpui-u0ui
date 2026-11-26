//! Separator: a 1px `border-subtle` hairline (`.cu-sep` in `custom.css`;
//! Carbon has no core equivalent).
//!
//! - **Appearance**: `.cu-sep { height: 1px; width: 100%; background:
//!   var(--border-subtle); flex: none }`; vertical: `.cu-sep--v { width: 1px;
//!   height: auto; align-self: stretch }`. `border-subtle` is contextual, so the
//!   separator takes a [`LayerLevel`] (default `One`, on `background`).
//! - **Behavior**: none (`role="separator"`). Kit's `Separator` draws a dashed
//!   or labeled rule in its own colors; Carbon's is a plain token line, so this
//!   is a div.
use crate::theme::{LayerLevel, color};
use crate::tokens::ColorRole;
use gpui_kit::{Hsla, IntoElement, RenderOnce, Styled, Window, div, px};

/// `index.d.ts` `SeparatorProps.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SeparatorOrientation {
    /// A full-width 1px rule.
    #[default]
    Horizontal,
    /// A 1px rule stretched to the row height.
    Vertical,
}

impl SeparatorOrientation {
    /// Both orientations.
    pub const ALL: [SeparatorOrientation; 2] = [Self::Horizontal, Self::Vertical];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// The line color: `.cu-sep { background: var(--border-subtle) }`, where
/// `--border-subtle` is contextual: `border-subtle-00` on the page background
/// (`:root { --border-subtle: var(--border-subtle-00) }`), `-01` inside a
/// `layer-01` surface, `-02` inside `layer-02`. `level` is the level of the
/// surface the separator would paint (One = it sits on `background`).
pub fn line_color(level: LayerLevel) -> Hsla {
    color(match level {
        LayerLevel::One => ColorRole::BorderSubtle00,
        LayerLevel::Two => ColorRole::BorderSubtle01,
        LayerLevel::Three => ColorRole::BorderSubtle02,
    })
}

/// `.cu-sep` / `.cu-sep--v` thickness in px.
pub const THICKNESS: f32 = 1.;

/// A separator. See the module docs.
#[derive(IntoElement, Default)]
pub struct Separator {
    orientation: SeparatorOrientation,
    level: LayerLevel,
}

impl Separator {
    /// A horizontal separator on `background`.
    pub fn new() -> Self {
        Self::default()
    }

    /// A vertical separator.
    pub fn vertical() -> Self {
        Self::new().orientation(SeparatorOrientation::Vertical)
    }

    /// The orientation.
    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The layer the separator sits on (picks `border-subtle-0N`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for Separator {
    fn render(self, _: &mut Window, _: &mut gpui_kit::App) -> impl IntoElement {
        let line = div().flex_none().bg(line_color(self.level));
        match self.orientation {
            SeparatorOrientation::Horizontal => line.h(px(THICKNESS)).w_full(),
            SeparatorOrientation::Vertical => line.w(px(THICKNESS)).self_stretch(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn line_is_contextual_border_subtle() {
        crate::theme::tests_set(CarbonTheme::White);
        // `:root { --border-subtle: var(--border-subtle-00, #e0e0e0) }`.
        assert_eq!(
            line_color(LayerLevel::One),
            color(ColorRole::BorderSubtle00)
        );
        assert_eq!(
            line_color(LayerLevel::Two),
            color(ColorRole::BorderSubtle01)
        );
        assert_eq!(
            line_color(LayerLevel::Three),
            color(ColorRole::BorderSubtle02)
        );
        assert_eq!(THICKNESS, 1.);
        assert_eq!(SeparatorOrientation::Vertical.name(), "vertical");
    }
}
