//! Item: a list row or standalone entry with media, title, description and
//! actions (`.cu-item*` in `custom.css`; Carbon `ContainedList` item anatomy).
//!
//! - `.cu-item { display: flex; flex-wrap: wrap; align-items: center; gap:
//!   var(--spacing-05); min-height: var(--size-lg); padding: var(--spacing-04)
//!   var(--spacing-05); color: var(--text-primary) }`; `sm` 40px min with
//!   8px/16px padding; `xs` 32px min, 4px/12px padding, 8px gap ([`Geometry`]).
//! - Variants: `outline` (1px inset `border-subtle`), `muted` (`layer-01`).
//!   With `href`/`on_click` the item is a link: hover `layer-hover-01`, 2px
//!   inset `focus` outline (`.cu-item--link`); the CSS has no transition, so
//!   colors switch at once.
//! - Parts: `ItemMedia` (`icon`: 40px `layer-accent-01` square; `image`: 48px
//!   clipped square; icons in `icon-primary`), `ItemContent` (column, 2px gap),
//!   `ItemTitle` (14/18 semibold), `ItemDescription` (`body-01`,
//!   `text-secondary`), `ItemActions` (8px row), `ItemHeader`/`ItemFooter`
//!   (full-width rows), `ItemGroup` (column) and `ItemSeparator` (1px rule).
//! - **Behavior**: Base `Button` for link items (focus, Enter/Space, pointer).
use super::parts::part;
use crate::{
    theme::{LayerLevel, color},
    tokens::{ColorRole, FontFamilyKey, SPACING_01, SPACING_03, TypeStyle},
    typography::{TypeStyled as _, font},
};
use gpui_kit::{
    AnyElement, App, ClickEvent, CursorStyle, ElementId, Hsla, InteractiveElement as _,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// `index.d.ts` `ItemProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemVariant {
    /// Transparent.
    #[default]
    Default,
    /// 1px inset `border-subtle` (`.cu-item--outline`).
    Outline,
    /// `layer-01` fill (`.cu-item--muted`).
    Muted,
}

impl ItemVariant {
    /// Every variant.
    pub const ALL: [ItemVariant; 3] = [Self::Default, Self::Outline, Self::Muted];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
            Self::Muted => "muted",
        }
    }
}

/// `index.d.ts` `ItemProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemSize {
    /// 48px min height.
    #[default]
    Default,
    /// 40px min height.
    Sm,
    /// 32px min height.
    Xs,
}

impl ItemSize {
    /// Every size.
    pub const ALL: [ItemSize; 3] = [Self::Default, Self::Sm, Self::Xs];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Xs => "xs",
        }
    }
}

/// Item box geometry in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// `min-height`.
    pub min_height: f32,
    /// Vertical padding.
    pub padding_y: f32,
    /// Horizontal padding.
    pub padding_x: f32,
    /// Gap between media, content and actions.
    pub gap: f32,
}

impl Geometry {
    /// `.cu-item` / `--sm` / `--xs`.
    pub const fn of(size: ItemSize) -> Self {
        match size {
            ItemSize::Default => Self {
                min_height: 48.,
                padding_y: 12.,
                padding_x: 16.,
                gap: 16.,
            },
            ItemSize::Sm => Self {
                min_height: 40.,
                padding_y: 8.,
                padding_x: 16.,
                gap: 16.,
            },
            ItemSize::Xs => Self {
                min_height: 32.,
                padding_y: 4.,
                padding_x: 12.,
                gap: 8.,
            },
        }
    }
}

/// (background, outline) of an item: `muted` paints `layer-01`, link hover
/// `layer-hover-01`; `outline` draws `border-subtle`.
pub fn look(variant: ItemVariant, hovered_link: bool, level: LayerLevel) -> (Hsla, Option<Hsla>) {
    let background = if hovered_link {
        color(ColorRole::LayerHover01)
    } else if variant == ItemVariant::Muted {
        color(ColorRole::Layer01)
    } else {
        Hsla::transparent_black()
    };
    let outline = (variant == ItemVariant::Outline).then(|| super::separator::line_color(level));
    (background, outline)
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// An item. See the module docs.
#[derive(IntoElement)]
pub struct Item {
    id: ElementId,
    variant: ItemVariant,
    size: ItemSize,
    level: LayerLevel,
    href: Option<SharedString>,
    on_click: Option<ClickHandler>,
    label: SharedString,
    children: Vec<AnyElement>,
}

impl Item {
    /// An item with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ItemVariant::Default,
            size: ItemSize::Default,
            level: LayerLevel::One,
            href: None,
            on_click: None,
            label: SharedString::default(),
            children: Vec::new(),
        }
    }

    /// The variant.
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The size.
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = size;
        self
    }

    /// The layer it sits on (the outline's `border-subtle`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Make the item a link to `href` (opened with the system handler).
    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// Make the item a link-like button.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Accessible name of a link item.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl ParentElement for Item {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Item {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = Geometry::of(self.size);
        let link = self.href.is_some() || self.on_click.is_some();
        let hovered = window.use_keyed_state((self.id.clone(), "hover"), cx, |_, _| false);
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = link && focus.is_focused(window);
        let (background, outline) = look(self.variant, link && *hovered.read(cx), self.level);
        let r = |v: f32| rems(v / 16.);
        let row = div()
            .id((self.id.clone(), "row"))
            .relative()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(r(g.gap))
            .min_h(r(g.min_height))
            .py(r(g.padding_y))
            .px(r(g.padding_x))
            .bg(background)
            .text_color(color(ColorRole::TextPrimary))
            .type_style(TypeStyle::BodyCompact01)
            .children(self.children)
            .when_some(outline, |this, outline| {
                // `outline: 1px solid var(--border-subtle); outline-offset: -1px`.
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_1()
                        .border_color(outline),
                )
            })
            .when(focused, |this| {
                // `.cu-item--link:focus-visible { outline: 2px solid var(--focus); outline-offset: -2px }`.
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            });
        if !link {
            return row.into_any_element();
        }
        let href = self.href.clone();
        let on_click = self.on_click.clone();
        row.cursor(CursorStyle::PointingHand)
            .on_hover(move |h, _, cx| {
                hovered.update(cx, |v, cx| {
                    *v = *h;
                    cx.notify();
                })
            })
            .child(
                // The Base button covers the row: focus, keyboard, pointer.
                gpui_kit::base::Button::new((self.id.clone(), "link"))
                    .track_focus(&focus)
                    .accessibility_label(self.label.clone())
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .on_click(move |event, window, cx| {
                        if let Some(handler) = &on_click {
                            handler(event, window, cx);
                        }
                        if let Some(href) = &href {
                            cx.open_url(href);
                        }
                    }),
            )
            .into_any_element()
    }
}

part!(
    /// `ItemGroup`: a column of items (`.cu-item-group`).
    ItemGroup,
    |children| div().flex().flex_col().children(children)
);

/// `ItemSeparator`: `.cu-item-sep { height: 1px; background: var(--border-subtle) }`.
#[derive(IntoElement, Default)]
pub struct ItemSeparator {
    level: LayerLevel,
}

impl ItemSeparator {
    /// A separator on `background`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The layer it sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for ItemSeparator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .h(px(1.))
            .w_full()
            .bg(super::separator::line_color(self.level))
    }
}

/// `index.d.ts` `ItemMedia.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemMediaVariant {
    /// As is (an avatar, an icon).
    #[default]
    Default,
    /// A 40px `layer-accent-01` square (`.cu-item-media--icon`).
    Icon,
    /// A 48px clipped square (`.cu-item-media--image`).
    Image,
}

impl ItemMediaVariant {
    /// Every variant.
    pub const ALL: [ItemMediaVariant; 3] = [Self::Default, Self::Icon, Self::Image];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Icon => "icon",
            Self::Image => "image",
        }
    }
}

/// `ItemMedia`. Icons inside should take [`ItemMedia::icon_color`] (`icon-primary`).
#[derive(IntoElement, Default)]
pub struct ItemMedia {
    variant: ItemMediaVariant,
    children: Vec<AnyElement>,
}

impl ItemMedia {
    /// Default media.
    pub fn new() -> Self {
        Self::default()
    }

    /// The variant.
    pub fn variant(mut self, variant: ItemMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `.cu-item-media { color: var(--icon-primary) }`.
    pub fn icon_color() -> Hsla {
        color(ColorRole::IconPrimary)
    }
}

impl ParentElement for ItemMedia {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ItemMedia {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let media = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .text_color(Self::icon_color());
        match self.variant {
            ItemMediaVariant::Default => media,
            ItemMediaVariant::Icon => media.size(rems(2.5)).bg(color(ColorRole::LayerAccent01)),
            ItemMediaVariant::Image => media.size(rems(3.)).overflow_hidden(),
        }
        .children(self.children)
    }
}

part!(
    /// `ItemContent`: `.cu-item-content { display: flex; flex: 1; flex-direction:
    /// column; gap: var(--spacing-01); min-width: 0 }`.
    ItemContent,
    |children| div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap(SPACING_01)
        .children(children)
);

part!(
    /// `ItemActions`: `.cu-item-actions { display: flex; gap: var(--spacing-03); align-items: center }`.
    ItemActions,
    |children| div()
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .gap(SPACING_03)
        .children(children)
);

part!(
    /// `ItemHeader`: a full-width row (`flex-basis: 100%; justify-content: space-between`).
    ItemHeader,
    |children| div()
        .flex()
        .flex_row()
        .w_full()
        .justify_between()
        .items_center()
        .gap(SPACING_03)
        .children(children)
);

part!(
    /// `ItemFooter`: a full-width row (`flex-basis: 100%; justify-content: space-between`).
    ItemFooter,
    |children| div()
        .flex()
        .flex_row()
        .w_full()
        .justify_between()
        .items_center()
        .gap(SPACING_03)
        .children(children)
);

/// `ItemTitle`: `.cu-item-title { font-size: 0.875rem; font-weight: 600; line-height: 1.28572 }`.
#[derive(IntoElement)]
pub struct ItemTitle(SharedString);

impl ItemTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for ItemTitle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .font(font(FontFamilyKey::Sans, 600))
            .text_size(rems(0.875))
            .line_height(rems(1.125))
            .child(self.0)
    }
}

/// `ItemDescription`: `.cu-item-desc { font-size: 0.875rem; line-height: 1.42857;
/// color: var(--text-secondary) }`.
#[derive(IntoElement)]
pub struct ItemDescription(SharedString);

impl ItemDescription {
    /// A description.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for ItemDescription {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Body01)
            .text_color(color(ColorRole::TextSecondary))
            .child(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn geometry_and_colors_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(Geometry::of(ItemSize::Default).min_height, 48.);
        assert_eq!(Geometry::of(ItemSize::Sm).padding_y, 8.);
        let xs = Geometry::of(ItemSize::Xs);
        assert_eq!(
            (xs.min_height, xs.padding_y, xs.padding_x, xs.gap),
            (32., 4., 12., 8.)
        );
        let (bg, outline) = look(ItemVariant::Muted, false, LayerLevel::One);
        assert_eq!(bg, color(ColorRole::Layer01));
        assert!(outline.is_none());
        let (bg, _) = look(ItemVariant::Default, true, LayerLevel::One);
        assert_eq!(bg, color(ColorRole::LayerHover01));
        let (_, outline) = look(ItemVariant::Outline, false, LayerLevel::One);
        assert_eq!(outline, Some(color(ColorRole::BorderSubtle00)));
    }
}
