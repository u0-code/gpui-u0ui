//! Breadcrumb: the path to the current page → Carbon `Breadcrumb` /
//! `BreadcrumbItem` (`.cds--breadcrumb`). Place it above the page title.
//!
//! - **Behavior**: each ancestor is a Base `Button` (focus, Enter/Space,
//!   pointer) that runs the caller's navigation; the current page is plain
//!   text.
//! - **Appearance**: body-compact-01 (label-01 when `sm`) `.cds--link` items in
//!   `link-primary`, `link-primary-hover` + underline on hover, a 1px `focus`
//!   outline when focused; the current item in `text-primary` without hover;
//!   `/` separators in `text-primary` 0.5rem (sm 0.25rem) on each side; no
//!   trailing slash by default; the row wraps.
//! - **Motion**: `.cds--link { transition: color 70ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
use super::overlay::{Callback, rem};
use crate::{
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.cds--link { transition: color 70ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const LINK_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);

/// Breadcrumb size (`.cds--breadcrumb--sm`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BreadcrumbSize {
    /// `sm`: label-01, 0.25rem gaps.
    Sm,
    /// `md` (default): body-compact-01, 0.5rem gaps.
    #[default]
    Md,
}

impl BreadcrumbSize {
    /// Both sizes.
    pub const ALL: [BreadcrumbSize; 2] = [Self::Sm, Self::Md];

    /// The Carbon name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }

    /// The gap on each side of a separator (`margin-inline-end` /
    /// `::after { margin-inline-start }`).
    pub const fn gap(self) -> f32 {
        match self {
            Self::Sm => 4.,
            Self::Md => 8.,
        }
    }
}

/// The color of a breadcrumb item.
pub fn item_color(current: bool, hovered: bool) -> Hsla {
    if current {
        // `.cds--breadcrumb-item [aria-current=page] { color: var(--text-primary) }`.
        color(ColorRole::TextPrimary)
    } else if hovered {
        color(ColorRole::LinkPrimaryHover)
    } else {
        color(ColorRole::LinkPrimary)
    }
}

/// One step of the path.
pub struct BreadcrumbItem {
    label: SharedString,
    current: bool,
    on_click: Option<Callback>,
}

impl BreadcrumbItem {
    /// An item showing `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            current: false,
            on_click: None,
        }
    }

    /// The current page (`isCurrentPage`): text, not a link.
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Navigate when activated (the reference's `href`).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// A Carbon breadcrumb. See the module docs.
#[derive(IntoElement)]
pub struct Breadcrumb {
    id: ElementId,
    items: Vec<BreadcrumbItem>,
    no_trailing_slash: bool,
    size: BreadcrumbSize,
}

impl Breadcrumb {
    /// A breadcrumb with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            no_trailing_slash: true,
            size: BreadcrumbSize::Md,
        }
    }

    /// Append an item.
    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    /// Hide the slash after the last item (default true).
    pub fn no_trailing_slash(mut self, hide: bool) -> Self {
        self.no_trailing_slash = hide;
        self
    }

    /// Size (default md).
    pub fn size(mut self, size: BreadcrumbSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let gap = self.size.gap();
        let style = match self.size {
            BreadcrumbSize::Sm => TypeStyle::Label01,
            BreadcrumbSize::Md => TypeStyle::BodyCompact01,
        };
        let count = self.items.len();
        let mut row = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .type_style(style);
        for (index, item) in self.items.into_iter().enumerate() {
            let item_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("item-{index}")),
            );
            let hover = window.use_keyed_state((item_id.clone(), "hover"), cx, |_, _| false);
            let hovered = *hover.read(cx) && !item.current;
            let text = motion::animate_color(
                (item_id.clone(), "text"),
                item_color(item.current, hovered),
                LINK_MOTION,
                window,
                cx,
            );
            let focus = window
                .use_keyed_state((item_id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            let focused = focus.is_focused(window);
            let label = if item.current {
                div()
                    .whitespace_nowrap()
                    // Same 1px box as the links' focus outline, so text lines up.
                    .border_1()
                    .border_color(Hsla::transparent_black())
                    .text_color(text)
                    .child(item.label.clone())
                    .into_any_element()
            } else {
                let on_click = item.on_click.clone();
                gpui_kit::base::Button::new((item_id.clone(), "link"))
                    .track_focus(&focus)
                    .accessibility_label(item.label.clone())
                    .whitespace_nowrap()
                    .text_color(text)
                    .when(hovered || focused, |this| this.underline())
                    // `.cds--link:focus { outline: 1px solid var(--focus) }`.
                    .border_1()
                    .border_color(if focused {
                        color(ColorRole::Focus)
                    } else {
                        Hsla::transparent_black()
                    })
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |h, _, cx| {
                        hover.update(cx, |v, cx| {
                            *v = *h;
                            cx.notify();
                        })
                    })
                    .on_click(move |_, window, cx| {
                        if let Some(f) = &on_click {
                            f(window, cx)
                        }
                    })
                    .child(item.label.clone())
                    .into_any_element()
            };
            let last = index + 1 == count;
            row = row.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .when(!last, |this| this.mr(rem(gap)))
                    .child(label)
                    .when(!(last && self.no_trailing_slash), |this| {
                        this.child(
                            div()
                                .ml(rem(gap))
                                .text_color(color(ColorRole::TextPrimary))
                                .child("/"),
                        )
                    }),
            );
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn colors_and_gaps_follow_breadcrumb_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(hex(item_color(false, false)), "#0f62fe");
        assert_eq!(hex(item_color(false, true)), "#0043ce");
        assert_eq!(hex(item_color(true, true)), "#161616");
        assert_eq!(
            (BreadcrumbSize::Md.gap(), BreadcrumbSize::Sm.gap()),
            (8., 4.)
        );
        assert!(Breadcrumb::new("b").no_trailing_slash);
    }
}
