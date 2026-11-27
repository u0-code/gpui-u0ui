//! `.cu-pempty`: the empty-state block of the Empty states pattern, in its
//! small-space (`--sm`: Tile, side panel) and large-space (`--lg`) layouts.
//! The left-aligned page-level [`crate::components::Empty`] is the
//! `custom.css` card; this one carries the pattern's own spacing.
//!
//! - **Small**: a column, the pictogram centred, text and actions left
//!   aligned, `h4` title.
//! - **Large**: a row, the pictogram on the left, 32px gap, centred
//!   vertically, `h3` title, at least 12rem tall (`min_height(false)` drops it,
//!   as the error example does).
//! - Title 16px (`h4`) or 20px (`h3`) above `spacing-03`; body 14/20
//!   `text-secondary` up to 32rem with `spacing-05` below; then the actions.
use super::Pictogram;
use crate::{
    theme::color,
    tokens::{self, ColorRole, TokenLength, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, rems,
};

/// Which space the empty state fills.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EmptyStateSize {
    /// `cu-pempty--sm` (default).
    #[default]
    Sm,
    /// `cu-pempty--lg`.
    Lg,
}

impl EmptyStateSize {
    /// Both sizes.
    pub const ALL: [Self; 2] = [Self::Sm, Self::Lg];

    /// The modifier name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Lg => "lg",
        }
    }
}

/// The block's measurements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyStateGeometry {
    /// `.cu-pempty--sm { padding: var(--spacing-06) var(--spacing-05) }`;
    /// `--lg { padding: var(--spacing-07) var(--spacing-09) }`. (block, inline)
    pub padding: (TokenLength, TokenLength),
    /// `.cu-pempty--lg { gap: var(--spacing-07) }`; none in `--sm`.
    pub gap: Option<TokenLength>,
    /// `.cu-pempty--lg { min-height: 12rem }`, in rem.
    pub min_height_rem: Option<f32>,
    /// `.cu-pempty h3, h4 { margin: var(--spacing-05) 0 var(--spacing-03) }`;
    /// `.cu-pempty--lg h3 { margin-top: 0 }`.
    pub title_margin_top: Option<TokenLength>,
    /// The title's bottom margin (`spacing-03`).
    pub title_margin_bottom: TokenLength,
    /// `.cu-pempty p { margin: 0 0 var(--spacing-05) }`.
    pub body_margin_bottom: TokenLength,
    /// `.cu-pempty p { max-width: 32rem }`, in rem.
    pub body_max_width_rem: f32,
    /// The title type: `.cu-pempty h4 { font-size: 1rem }`, `h3 { font-size:
    /// 1.25rem }`, both `font-weight: 400`.
    pub title: TypeStyle,
}

impl EmptyStateGeometry {
    /// The geometry of `size`.
    pub fn of(size: EmptyStateSize) -> Self {
        let lg = size == EmptyStateSize::Lg;
        Self {
            padding: if lg {
                (tokens::SPACING_07, tokens::SPACING_09)
            } else {
                (tokens::SPACING_06, tokens::SPACING_05)
            },
            gap: lg.then_some(tokens::SPACING_07),
            min_height_rem: lg.then_some(12.),
            title_margin_top: (!lg).then_some(tokens::SPACING_05),
            title_margin_bottom: tokens::SPACING_03,
            body_margin_bottom: tokens::SPACING_05,
            body_max_width_rem: 32.,
            title: if lg {
                TypeStyle::Heading03
            } else {
                TypeStyle::Body02
            },
        }
    }
}

/// The block's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyStateLook {
    /// `.cu-pempty { background: var(--layer-01) }`.
    pub background: gpui_kit::Hsla,
    /// `.cu-pempty { color: var(--text-primary) }`.
    pub title: gpui_kit::Hsla,
    /// `.cu-pempty p { color: var(--text-secondary) }`.
    pub body: gpui_kit::Hsla,
}

impl EmptyStateLook {
    /// The reference look.
    pub fn current() -> Self {
        Self {
            background: color(ColorRole::Layer01),
            title: color(ColorRole::TextPrimary),
            body: color(ColorRole::TextSecondary),
        }
    }
}

/// An empty state. Children are its actions, after the body.
#[derive(IntoElement)]
pub struct PatternEmptyState {
    size: EmptyStateSize,
    pictogram: Option<Pictogram>,
    title: SharedString,
    title_style: Option<TypeStyle>,
    body: Option<SharedString>,
    min_height: bool,
    actions: Vec<AnyElement>,
}

impl PatternEmptyState {
    /// An empty state of `size` titled `title` (say what to do, not "No data").
    pub fn new(size: EmptyStateSize, title: impl Into<SharedString>) -> Self {
        Self {
            size,
            pictogram: None,
            title: title.into(),
            title_style: None,
            body: None,
            min_height: true,
            actions: Vec::new(),
        }
    }

    /// The illustration (`Pic(icon, size)`).
    pub fn pictogram(mut self, pictogram: Pictogram) -> Self {
        self.pictogram = Some(pictogram);
        self
    }

    /// The body text.
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Override the title type (the error example sets `fontSize: 16`).
    pub fn title_style(mut self, style: TypeStyle) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Keep the large layout's 12rem minimum height (default true).
    pub fn min_height(mut self, on: bool) -> Self {
        self.min_height = on;
        self
    }
}

impl ParentElement for PatternEmptyState {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.actions.extend(elements);
    }
}

impl RenderOnce for PatternEmptyState {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let g = EmptyStateGeometry::of(self.size);
        let look = EmptyStateLook::current();
        let lg = self.size == EmptyStateSize::Lg;
        let title = div()
            .when_some(g.title_margin_top, |d, m| d.mt(m))
            .mb(g.title_margin_bottom)
            .type_style(self.title_style.unwrap_or(g.title))
            .text_color(look.title)
            .child(self.title);
        let body = self.body.map(|b| {
            div()
                .mb(g.body_margin_bottom)
                .max_w(rems(g.body_max_width_rem))
                .type_style(TypeStyle::Body01)
                .text_color(look.body)
                .child(b)
        });
        let text = div()
            .flex()
            .flex_col()
            .items_start()
            .min_w_0()
            .child(title)
            .children(body)
            .children(self.actions);
        let base = div()
            .flex()
            .w_full()
            .py(g.padding.0)
            .px(g.padding.1)
            .bg(look.background)
            .text_color(look.title);
        if lg {
            base.flex_row()
                .items_center()
                .when_some(g.gap, |d, gap| d.gap(gap))
                .when_some(g.min_height_rem.filter(|_| self.min_height), |d, h| {
                    d.min_h(rems(h))
                })
                .children(self.pictogram)
                .child(text.flex_1())
        } else {
            base.flex_col()
                .items_start()
                // `.cu-pempty--sm .cu-ppic { align-self: center }`.
                .children(self.pictogram.map(|p| div().self_center().child(p)))
                .child(text.w_full())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_and_large_geometry_match_the_css() {
        let sm = EmptyStateGeometry::of(EmptyStateSize::Sm);
        assert_eq!(sm.padding.0.px_value(), 24.);
        assert_eq!(sm.padding.1.px_value(), 16.);
        assert_eq!(sm.gap, None);
        assert_eq!(sm.min_height_rem, None);
        assert_eq!(sm.title_margin_top.map(|m| m.px_value()), Some(16.));
        assert_eq!(sm.title, TypeStyle::Body02);
        let lg = EmptyStateGeometry::of(EmptyStateSize::Lg);
        assert_eq!(lg.padding.0.px_value(), 32.);
        assert_eq!(lg.padding.1.px_value(), 48.);
        assert_eq!(lg.gap.map(|g| g.px_value()), Some(32.));
        assert_eq!(lg.min_height_rem, Some(12.));
        assert_eq!(lg.title_margin_top, None);
        assert_eq!(lg.title, TypeStyle::Heading03);
        assert_eq!(lg.title_margin_bottom.px_value(), 8.);
        assert_eq!(lg.body_margin_bottom.px_value(), 16.);
        assert_eq!(lg.body_max_width_rem, 32.);
    }

    #[test]
    fn colors_come_from_the_css_roles() {
        let look = EmptyStateLook::current();
        assert_eq!(look.background, color(ColorRole::Layer01));
        assert_eq!(look.title, color(ColorRole::TextPrimary));
        assert_eq!(look.body, color(ColorRole::TextSecondary));
    }
}
