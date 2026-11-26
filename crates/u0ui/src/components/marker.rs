//! Marker: an inline status, system note or labelled divider in a
//! conversation (`.cu-marker*` in `custom.css`). Layout only; nothing from Kit.
//!
//! - `.cu-marker { display: flex; align-items: center; gap: spacing-03;
//!   font-size: 0.75rem; line-height: 1.33333; color: text-secondary }`
//!   (`label-01`).
//! - `border`: `padding: spacing-03 spacing-04` with a 1px inset
//!   `border-subtle-01` outline.
//! - `separator`: a 1px `border-subtle-01` rule growing on each side of the
//!   text (`::before, ::after { flex: 1; height: 1px }`).
//! - [`MarkerIcon`] holds an icon or spinner in `currentColor`
//!   (`.cu-marker-icon svg { fill: currentColor }`); [`MarkerContent`] the text.
//! - `role="status"` (a streaming status) is [`Marker::status`]: the row gets
//!   the accessible name of its text.
use crate::{
    theme::color,
    tokens::{ColorRole, SPACING_03, SPACING_04, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};

/// `index.d.ts` `Marker.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MarkerVariant {
    /// Plain row.
    #[default]
    Default,
    /// Padded, with a 1px outline.
    Border,
    /// Rules on both sides of the text.
    Separator,
}

impl MarkerVariant {
    /// Every variant.
    pub const ALL: [MarkerVariant; 3] = [Self::Default, Self::Border, Self::Separator];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Border => "border",
            Self::Separator => "separator",
        }
    }
}

/// Text color (`.cu-marker { color: text-secondary }`).
pub fn text_color() -> Hsla {
    color(ColorRole::TextSecondary)
}

/// Outline and rule color (`border-subtle-01`).
pub fn line_color() -> Hsla {
    color(ColorRole::BorderSubtle01)
}

/// A marker row. See the module docs.
#[derive(IntoElement, Default)]
pub struct Marker {
    variant: MarkerVariant,
    status: Option<(ElementId, SharedString)>,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl Marker {
    /// A default marker.
    pub fn new() -> Self {
        Self::default()
    }

    /// The variant.
    pub fn variant(mut self, variant: MarkerVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Mark it as a live status (`role="status"`), announced as `label`.
    pub fn status(mut self, id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        self.status = Some((id.into(), label.into()));
        self
    }
}

impl Styled for Marker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Marker {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Marker {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let rule = || div().flex_1().h(px(1.)).bg(line_color());
        let separator = self.variant == MarkerVariant::Separator;
        let mut row = div()
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(SPACING_03)
            .type_style(TypeStyle::Label01)
            .text_color(text_color())
            .when(self.variant == MarkerVariant::Border, |this| {
                this.py(SPACING_03).px(SPACING_04).child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_1()
                        .border_color(line_color()),
                )
            })
            .when(separator, |this| this.child(rule()))
            .children(self.children)
            .when(separator, |this| this.child(rule()));
        gpui_kit::Refineable::refine(row.style(), &self.style);
        match self.status {
            Some((id, label)) => div().id(id).aria_label(label).child(row).into_any_element(),
            None => row.into_any_element(),
        }
    }
}

/// The icon slot.
#[derive(IntoElement, Default)]
pub struct MarkerIcon {
    children: Vec<AnyElement>,
}

impl MarkerIcon {
    /// An empty slot; add an icon (it takes the marker's text color unless
    /// given one) or a small spinner.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for MarkerIcon {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MarkerIcon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_none()
            .items_center()
            .children(self.children)
    }
}

/// The text slot.
#[derive(IntoElement, Default)]
pub struct MarkerContent {
    children: Vec<AnyElement>,
}

impl MarkerContent {
    /// An empty slot.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for MarkerContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MarkerContent {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_row().children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn marker_follows_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(text_color(), color(ColorRole::TextSecondary));
        assert_eq!(line_color(), color(ColorRole::BorderSubtle01));
        // `.cu-marker { gap: spacing-03 }`, `--border { padding: spacing-03 spacing-04 }`.
        assert_eq!((SPACING_03.px_value(), SPACING_04.px_value()), (8., 12.));
        let label = TypeStyle::Label01.spec();
        assert_eq!(label.font_size_px(), 12.);
    }
}
