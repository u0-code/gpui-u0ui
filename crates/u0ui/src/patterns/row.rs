//! `.cu-prow`: a list row whose content sits at the start and whose actions
//! sit at the end, on a `border-subtle-01` rule.
use crate::{
    density::{self, Density},
    theme::color,
    tokens::{self, ColorRole, TokenLength, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder as _,
};

/// The row's box at a density.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowGeometry {
    /// `min-height: var(--density-row-lg)`.
    pub min_height: TokenLength,
    /// `padding: 0 var(--spacing-02) 0 var(--spacing-05)`: end side.
    pub padding_end: TokenLength,
    /// … start side.
    pub padding_start: TokenLength,
}

impl RowGeometry {
    /// The geometry under `density`.
    pub const fn of(density: Density) -> Self {
        Self {
            min_height: density.row_lg(),
            padding_end: tokens::SPACING_02,
            padding_start: tokens::SPACING_05,
        }
    }
}

/// A pattern list row (`.cu-prow { display: flex; align-items: center;
/// justify-content: space-between; border-bottom: 1px solid
/// var(--border-subtle-01); font-size: 0.875rem }`). Children are laid out
/// start and end; text is `body-compact-01` in `text-primary`.
#[derive(IntoElement, Default)]
pub struct PatternRow {
    children: Vec<AnyElement>,
    density: Option<Density>,
}

impl PatternRow {
    /// An empty row.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the active density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }
}

impl ParentElement for PatternRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for PatternRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let g = RowGeometry::of(self.density.unwrap_or_else(density::current));
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .min_h(g.min_height)
            .pl(g.padding_start)
            .pr(g.padding_end)
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .when(!self.children.is_empty(), |this| {
                this.children(self.children)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_is_density_row_lg_with_asymmetric_padding() {
        let g = RowGeometry::of(Density::Compact);
        assert_eq!(g.min_height.px_value(), 36.);
        assert_eq!(
            (g.padding_start.px_value(), g.padding_end.px_value()),
            (16., 4.)
        );
    }
}
