//! Empty: the Carbon empty-state pattern (`.cu-empty*` in `custom.css`).
//!
//! - `.cu-empty { display: flex; flex-direction: column; align-items: flex-start;
//!   gap: var(--spacing-06); max-width: 26rem; padding: var(--spacing-07) 0 }`:
//!   left-aligned, 24px between the header and the actions.
//! - `EmptyHeader` stacks media, title and description 8px apart;
//!   `EmptyMedia` is `icon-secondary` with 8px below, or with `variant = icon`
//!   a 5rem `layer-01` square holding the icon in `icon-primary`;
//!   `EmptyTitle` is `heading-03` (20/28) in `text-primary`; `EmptyDescription`
//!   `body-01` in `text-secondary`; `EmptyContent` a wrapping row of actions 8px apart.
//! - **Behavior**: the title and description are selectable (Base's
//!   `SelectableText`), so what an error state says can be copied. Kit's
//!   `Empty` centers its content; Carbon's pattern is left-aligned, so it
//!   is drawn here.
use super::parts::part;
use crate::{
    theme::color,
    tokens::{ColorRole, SPACING_03, SPACING_06, SPACING_07, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::base::SelectableText;
use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, rems,
};

/// `.cu-empty { max-width: 26rem }` in px.
pub const MAX_WIDTH: f32 = 416.;
/// `.cu-empty-media--icon { width: 5rem; height: 5rem }` in px.
pub const ICON_BOX: f32 = 80.;

part!(
    /// `Empty`: the left-aligned state container.
    Empty,
    |children| div()
        .flex()
        .flex_col()
        .items_start()
        .gap(SPACING_06)
        .max_w(rems(MAX_WIDTH / 16.))
        .py(SPACING_07)
        .children(children)
);

part!(
    /// `EmptyHeader`: media, title and description, 8px apart.
    EmptyHeader,
    |children| div().flex().flex_col().gap(SPACING_03).children(children)
);

part!(
    /// `EmptyContent`: the actions row (`.cu-empty-content { display: flex;
    /// gap: var(--spacing-03); flex-wrap: wrap }`).
    EmptyContent,
    |children| div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(SPACING_03)
        .children(children)
);

/// `index.d.ts` `EmptyMedia.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EmptyMediaVariant {
    /// The illustration or icon as is, in `icon-secondary`.
    #[default]
    Default,
    /// A 5rem `layer-01` square with the icon in `icon-primary`.
    Icon,
}

impl EmptyMediaVariant {
    /// Both variants.
    pub const ALL: [EmptyMediaVariant; 2] = [Self::Default, Self::Icon];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Icon => "icon",
        }
    }

    /// The media color: `.cu-empty-media { color: var(--icon-secondary) }`,
    /// `--icon { color: var(--icon-primary) }`.
    pub fn icon_color(self) -> gpui_kit::Hsla {
        color(match self {
            Self::Default => ColorRole::IconSecondary,
            Self::Icon => ColorRole::IconPrimary,
        })
    }
}

/// `EmptyMedia`. Give icons [`EmptyMedia::icon_color`] (the CSS sets `fill:
/// currentColor`, which GPUI icons do not inherit).
#[derive(IntoElement, Default)]
pub struct EmptyMedia {
    variant: EmptyMediaVariant,
    children: Vec<AnyElement>,
}

impl EmptyMedia {
    /// Default media.
    pub fn new() -> Self {
        Self::default()
    }

    /// The variant.
    pub fn variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The color the media's icon should take.
    pub fn icon_color(&self) -> gpui_kit::Hsla {
        self.variant.icon_color()
    }
}

impl ParentElement for EmptyMedia {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for EmptyMedia {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.cu-empty-media { margin-bottom: var(--spacing-03) }`.
        let media = div().mb(SPACING_03).text_color(self.variant.icon_color());
        match self.variant {
            EmptyMediaVariant::Default => media.flex().children(self.children),
            // `.cu-empty-media--icon { display: flex; width: 5rem; height: 5rem;
            // align-items: center; justify-content: center; background: var(--layer-01) }`.
            EmptyMediaVariant::Icon => media
                .flex()
                .items_center()
                .justify_center()
                .size(rems(ICON_BOX / 16.))
                .bg(color(ColorRole::Layer01))
                .children(self.children),
        }
    }
}

/// `EmptyTitle`: `.cu-empty-title { font-size: 1.25rem; line-height: 1.4;
/// font-weight: 400; color: var(--text-primary) }`.
#[derive(IntoElement)]
pub struct EmptyTitle(SharedString);

impl EmptyTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for EmptyTitle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Heading03)
            .text_color(color(ColorRole::TextPrimary))
            .child(SelectableText::new(
                ElementId::Name(format!("empty-title:{}", self.0).into()),
                self.0,
            ))
    }
}

/// `EmptyDescription`: `.cu-empty-desc { font-size: 0.875rem; line-height:
/// 1.42857; color: var(--text-secondary) }`.
#[derive(IntoElement)]
pub struct EmptyDescription(SharedString);

impl EmptyDescription {
    /// A description.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for EmptyDescription {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Body01)
            .text_color(color(ColorRole::TextSecondary))
            .child(SelectableText::new(
                ElementId::Name(format!("empty-desc:{}", self.0).into()),
                self.0,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn empty_follows_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!((MAX_WIDTH, ICON_BOX), (416., 80.));
        assert_eq!((SPACING_06.px_value(), SPACING_07.px_value()), (24., 32.));
        assert_eq!(
            EmptyMediaVariant::Default.icon_color(),
            color(ColorRole::IconSecondary)
        );
        assert_eq!(
            EmptyMediaVariant::Icon.icon_color(),
            color(ColorRole::IconPrimary)
        );
    }
}
