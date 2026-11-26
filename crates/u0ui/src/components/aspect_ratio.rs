//! AspectRatio: a box of fixed width : height (`.cu-aspect` in `custom.css`).
//!
//! - **Appearance**: `.cu-aspect { position: relative; width: 100%; overflow:
//!   hidden; background: var(--layer-01); aspect-ratio: <ratio> }`. Children
//!   fill the box (`.cu-aspect > img, video { width: 100%; height: 100%;
//!   object-fit: cover }`): they sit in an absolutely positioned layer covering
//!   it, so an `img().size_full().object_fit(Cover)` behaves like the CSS.
//! - **Layout** is Taffy's `aspect-ratio`, the same algorithm as CSS: the width
//!   comes from the parent, the height is `width / ratio`.
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

/// Carbon's recommended ratios (README): 16:9, 9:16, 2:1, 1:2, 4:3, 3:4, 3:2, 2:3, 1:1.
pub const CARBON_RATIOS: [(u32, u32); 9] = [
    (16, 9),
    (9, 16),
    (2, 1),
    (1, 2),
    (4, 3),
    (3, 4),
    (3, 2),
    (2, 3),
    (1, 1),
];

/// The height of a box `width` wide at `ratio` (CSS `aspect-ratio`).
pub fn height_for(width: f32, ratio: f32) -> f32 {
    if ratio > 0. { width / ratio } else { width }
}

/// A fixed-ratio box. See the module docs.
#[derive(IntoElement)]
pub struct AspectRatio {
    ratio: f32,
    children: Vec<AnyElement>,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::new(16. / 9.)
    }
}

impl AspectRatio {
    /// A box at `ratio` = width / height (`index.jsx` default `16 / 9`).
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: if ratio > 0. { ratio } else { 16. / 9. },
            children: Vec::new(),
        }
    }

    /// The ratio (width / height).
    pub fn ratio(&self) -> f32 {
        self.ratio
    }
}

impl ParentElement for AspectRatio {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AspectRatio {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .relative()
            .w_full()
            .aspect_ratio(self.ratio)
            .overflow_hidden()
            .bg(color(ColorRole::Layer01))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .flex()
                    .flex_col()
                    .children(self.children),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_sets_height() {
        // The preview: 280px wide at 16:9 → 157.5px; 140px at 1:1 → 140px.
        assert_eq!(height_for(280., 16. / 9.), 157.5);
        assert_eq!(height_for(140., 1.), 140.);
        assert_eq!(AspectRatio::new(0.).ratio(), 16. / 9.);
        assert_eq!(AspectRatio::default().ratio(), 16. / 9.);
        assert!(CARBON_RATIOS.contains(&(4, 3)));
    }
}
