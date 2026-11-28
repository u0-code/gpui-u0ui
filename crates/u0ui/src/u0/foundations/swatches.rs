//! TokenSwatches: a grid of color tokens with their resolved values
//! (`TokenSwatches` in `src/ghost/foundations.jsx`), used by the agent,
//! syntax and ANSI specimens.
//!
//! `.g-swatches` (8px row / 12px column gaps) of `.g-swatch` cells: a 28px
//! chip with a 1px `border-subtle-01` inset rule (filled with the color, or
//! showing `sample` text in it: 14px mono 600), then the token name (11px
//! mono `text-primary`) over its value in the active theme (11px mono
//! `text-secondary`), both truncated.
use crate::{
    theme::{color, rgba},
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    App, BoxShadow, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    Rgba, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative, rems,
};

/// A color as CSS prints a resolved custom property: `#rrggbb`, or
/// `rgba(r, g, b, a)` when translucent.
pub fn css_value(c: Rgba) -> String {
    let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    if c.a >= 0.999 {
        format!("#{:02x}{:02x}{:02x}", byte(c.r), byte(c.g), byte(c.b))
    } else {
        let a = (c.a * 100.).round() / 100.;
        format!("rgba({}, {}, {}, {a})", byte(c.r), byte(c.g), byte(c.b))
    }
}

/// A token swatch grid. See the module docs.
#[derive(IntoElement)]
pub struct TokenSwatches {
    id: ElementId,
    names: Vec<ColorRole>,
    label: Option<SharedString>,
    columns: usize,
    sample: Option<SharedString>,
}

impl TokenSwatches {
    /// Swatches for `names` (4 columns).
    pub fn new(id: impl Into<ElementId>, names: impl IntoIterator<Item = ColorRole>) -> Self {
        Self {
            id: id.into(),
            names: names.into_iter().collect(),
            label: None,
            columns: 4,
            sample: None,
        }
    }

    /// The list's accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Columns (`repeat(columns, minmax(0, 1fr))`).
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Show this text in the token's color instead of a filled chip (`sample`).
    pub fn sample(mut self, sample: impl Into<SharedString>) -> Self {
        self.sample = Some(sample.into());
        self
    }
}

impl RenderOnce for TokenSwatches {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mono = font(FontFamilyKey::Mono, 400);
        let rule = vec![BoxShadow {
            color: color(ColorRole::BorderSubtle01),
            offset: gpui_kit::point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(1.),
            inset: true,
        }];
        let cols = self.columns;
        let cells: Vec<_> = self
            .names
            .iter()
            .map(|&role| {
                let chip = div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rems(1.75))
                    .shadow(rule.clone())
                    .map(|d| match &self.sample {
                        Some(s) => d
                            .font(font(FontFamilyKey::Mono, 600))
                            .text_size(rems(0.875))
                            .text_color(color(role))
                            .child(s.clone()),
                        None => d.bg(color(role)),
                    });
                let text = |t: String, role: ColorRole| {
                    div()
                        .font(mono.clone())
                        .text_size(rems(0.75))
                        .line_height(rems(1.))
                        .text_color(color(role))
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(t)
                };
                div()
                    // `minmax(0, 1fr)` columns.
                    .w(relative(1. / cols as f32))
                    .pr(rems(0.75))
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap(rems(0.5))
                    .child(chip)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(text(role.name().to_string(), ColorRole::TextPrimary))
                            .child(text(css_value(rgba(role)), ColorRole::TextSecondary)),
                    )
            })
            .collect();
        let mut grid = div()
            .id(self.id.clone())
            .role(gpui_kit::Role::List)
            .flex()
            .flex_wrap()
            .gap_y(rems(0.5))
            // The 12px column gap is the cell's end padding, so the last
            // column's chip edge lines up with `1fr`.
            .mr(rems(-0.75));
        if let Some(l) = self.label.clone() {
            grid = grid.aria_label(l);
        }
        grid.children(cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_print_like_computed_style() {
        let c = Rgba {
            r: 15. / 255.,
            g: 98. / 255.,
            b: 254. / 255.,
            a: 1.,
        };
        assert_eq!(css_value(c), "#0f62fe");
        let t = Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.5,
        };
        assert_eq!(css_value(t), "rgba(0, 0, 0, 0.5)");
    }
}
