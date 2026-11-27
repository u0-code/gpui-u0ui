//! `.cu-pmenu`: a menu drawn open in place, so a pattern example can show
//! its items (and which are disabled, and why) without opening an overlay.
//! A real menu is [`crate::components::DropdownMenu`].
//!
//! - **Items** (`.cu-pmenu-item`): `size-sm` rows, 16px inline padding,
//!   `layer-hover-01` on hover.
//! - **Disabled** (`.cu-pmenu-item--off`): `text-disabled`, no hover,
//!   `not-allowed`; its reason shows as a tooltip (the reference's `title`).
use crate::{
    components::{Tooltip, overlay::floating_shadow},
    theme::color,
    tokens::{self, ColorRole, TokenLength, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, rems,
};
use std::rc::Rc;

/// The menu's measurements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuGeometry {
    /// `.cu-pmenu { width: 14rem }`, in rem.
    pub width_rem: f32,
    /// `.cu-pmenu { padding: var(--spacing-02) 0 }`.
    pub padding_block: TokenLength,
    /// `.cu-pmenu-item { height: var(--size-sm) }`.
    pub item_height: TokenLength,
    /// `.cu-pmenu-item { padding: 0 var(--spacing-05) }`.
    pub item_padding: TokenLength,
}

impl MenuGeometry {
    /// The reference geometry.
    pub const REFERENCE: Self = Self {
        width_rem: 14.,
        padding_block: tokens::SPACING_02,
        item_height: tokens::SIZE_SM,
        item_padding: tokens::SPACING_05,
    };
}

/// An item's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLook {
    /// `.cu-pmenu-item { color: var(--text-primary) }`; off `text-disabled`.
    pub text: Hsla,
    /// `.cu-pmenu-item:hover { background: var(--layer-hover-01) }`; off none
    /// (`.cu-pmenu-item--off:hover { background: transparent }`).
    pub hover: Option<Hsla>,
}

impl ItemLook {
    /// The look of an item, `disabled` or not.
    pub fn of(disabled: bool) -> Self {
        if disabled {
            Self {
                text: color(ColorRole::TextDisabled),
                hover: None,
            }
        } else {
            Self {
                text: color(ColorRole::TextPrimary),
                hover: Some(color(ColorRole::LayerHover01)),
            }
        }
    }
}

/// One item: its label and, when disabled, the reason.
#[derive(Clone)]
pub struct PatternMenuItem {
    label: SharedString,
    disabled: Option<SharedString>,
}

impl PatternMenuItem {
    /// An enabled item reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            disabled: None,
        }
    }

    /// Disable it, saying why (`aria-disabled`, `title`).
    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled = Some(reason.into());
        self
    }
}

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A menu drawn in place. See the module docs.
#[derive(IntoElement)]
pub struct PatternMenu {
    id: ElementId,
    items: Vec<PatternMenuItem>,
    on_select: Option<SelectHandler>,
}

impl PatternMenu {
    /// An empty menu with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            on_select: None,
        }
    }

    /// Append an item.
    pub fn item(mut self, item: PatternMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Called with an enabled item's index when it is clicked.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PatternMenu {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let g = MenuGeometry::REFERENCE;
        let id = self.id;
        let items = self.items.into_iter().enumerate().map(|(i, item)| {
            let look = ItemLook::of(item.disabled.is_some());
            let on_select = self.on_select.clone();
            let enabled = item.disabled.is_none();
            let row = div()
                .id((id.clone(), format!("item{i}")))
                .flex()
                .items_center()
                .w_full()
                .h(g.item_height)
                .px(g.item_padding)
                .text_color(look.text)
                .overflow_hidden()
                .whitespace_nowrap()
                .when_some(look.hover, |d, hover| d.hover(move |s| s.bg(hover)))
                .cursor(if enabled {
                    CursorStyle::PointingHand
                } else {
                    CursorStyle::OperationNotAllowed
                })
                .when(enabled, |d| {
                    d.on_click(move |_, window, cx| {
                        if let Some(f) = &on_select {
                            f(i, window, cx)
                        }
                    })
                })
                .child(item.label);
            match item.disabled {
                Some(reason) => Tooltip::new((id.clone(), format!("tip{i}")), row)
                    .content(reason)
                    .into_any_element(),
                None => row.into_any_element(),
            }
        });
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w(rems(g.width_rem))
            .max_w_full()
            .py(g.padding_block)
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            // `.cu-pmenu-item { font-size: 0.875rem }`.
            .type_style(TypeStyle::BodyCompact01)
            .children(items.collect::<Vec<AnyElement>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_the_css() {
        let g = MenuGeometry::REFERENCE;
        assert_eq!(g.width_rem, 14.);
        assert_eq!(g.padding_block.px_value(), 4.);
        assert_eq!(g.item_height.px_value(), 32.);
        assert_eq!(g.item_padding.px_value(), 16.);
    }

    #[test]
    fn disabled_items_lose_hover_and_dim() {
        let on = ItemLook::of(false);
        assert_eq!(on.text, color(ColorRole::TextPrimary));
        assert_eq!(on.hover, Some(color(ColorRole::LayerHover01)));
        let off = ItemLook::of(true);
        assert_eq!(off.text, color(ColorRole::TextDisabled));
        assert_eq!(off.hover, None);
    }
}
