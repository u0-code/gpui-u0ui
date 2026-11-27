//! `Pic` / `.cu-ppic`: a system icon drawn large on an accent square, the
//! illustration of empty states (the pattern cards draw these instead of IBM
//! pictograms).
use crate::{
    icons::Icon,
    theme::color,
    tokens::{ColorRole, REM_PX},
};
use gpui_kit::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, rems};

/// The default square side (`Pic(icon, size)`: `size || 80`).
pub const DEFAULT_SIZE: f32 = 80.;

/// The icon side for a square of `size` px: `Math.round(size * 0.5)`.
pub fn icon_size(size: f32) -> f32 {
    (size * 0.5).round()
}

/// A pictogram: `icon` centred on a `size`-px square of `layer-accent-01`,
/// drawn in `icon-secondary` at half the side (`.cu-ppic { background:
/// var(--layer-accent-01); color: var(--icon-secondary) }`). Square corners;
/// decorative, so it takes no focus.
#[derive(IntoElement)]
pub struct Pictogram {
    icon: Icon,
    size: f32,
}

impl Pictogram {
    /// A pictogram of `icon`, 80px square.
    pub fn new(icon: impl Into<Icon>) -> Self {
        Self {
            icon: icon.into(),
            size: DEFAULT_SIZE,
        }
    }

    /// The square side in px at 100% zoom (`Pic(icon, size)`).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Pictogram {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let side = rems(self.size / REM_PX);
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(side)
            .bg(color(ColorRole::LayerAccent01))
            .child(
                self.icon
                    .size_length(rems(icon_size(self.size) / REM_PX))
                    .color(color(ColorRole::IconSecondary)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_half_the_square_rounded() {
        assert_eq!(icon_size(DEFAULT_SIZE), 40.);
        assert_eq!(icon_size(64.), 32.);
        assert_eq!(icon_size(120.), 60.);
        assert_eq!(icon_size(45.), 23.);
    }
}
