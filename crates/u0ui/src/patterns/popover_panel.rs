//! The disclosure panel of the pattern examples: a floating surface
//! (`.cu-ppop`), its rules (`.cu-phr`) and its link rows (`.cu-plink`). The
//! Disclosures pattern draws the profile and settings menus with them; the
//! surface is drawn in place, so the caller decides when it shows.
//!
//! - **Panel** (`.cu-ppop`): `layer-01`, the floating shadow, full width up to
//!   18rem, square corners.
//! - **Rule** (`.cu-phr`): a 1px `border-subtle-01` line.
//! - **Link** (`.cu-plink`): a block row, `spacing-04` × `spacing-05` padding,
//!   `text-primary` at 0.875rem, `layer-hover-01` on hover, a 2px inset
//!   `focus` outline while focused. Enter or a click activates it.
use crate::{
    components::{form::inset_outline, overlay::floating_shadow},
    theme::color,
    tokens::{self, ColorRole, TokenLength, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// `.cu-ppop { max-width: 18rem }`, in rem.
pub const PANEL_MAX_WIDTH: f32 = 18.;

/// `.cu-plink { padding: var(--spacing-04) var(--spacing-05) }`.
pub const LINK_PADDING: (TokenLength, TokenLength) = (tokens::SPACING_04, tokens::SPACING_05);

/// A link row's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkLook {
    /// `.cu-plink { color: var(--text-primary) }`.
    pub text: Hsla,
    /// `.cu-plink:hover { background: var(--layer-hover-01) }`.
    pub hover: Hsla,
    /// `.cu-plink:focus-visible { outline: 2px solid var(--focus) }`.
    pub focus: Hsla,
}

impl LinkLook {
    /// The reference look.
    pub fn current() -> Self {
        Self {
            text: color(ColorRole::TextPrimary),
            hover: color(ColorRole::LayerHover01),
            focus: color(ColorRole::Focus),
        }
    }
}

/// `.cu-phr { border-top: 1px solid var(--border-subtle-01) }`.
pub fn rule_color() -> Hsla {
    color(ColorRole::BorderSubtle01)
}

/// The panel surface (`.cu-ppop`). Children stack top to bottom.
#[derive(IntoElement, Default)]
pub struct PatternPopover {
    children: Vec<AnyElement>,
}

impl PatternPopover {
    /// An empty panel.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for PatternPopover {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for PatternPopover {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .max_w(rems(PANEL_MAX_WIDTH))
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .text_color(color(ColorRole::TextPrimary))
            .type_style(TypeStyle::BodyCompact01)
            .children(self.children)
    }
}

/// A 1px divider across the panel (`.cu-phr`).
#[derive(IntoElement, Default)]
pub struct PatternRule;

impl PatternRule {
    /// A rule.
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for PatternRule {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().w_full().h(px(1.)).flex_none().bg(rule_color())
    }
}

type ClickHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A link row in the panel (`.cu-plink`).
#[derive(IntoElement)]
pub struct PatternLink {
    id: ElementId,
    label: SharedString,
    on_click: Option<ClickHandler>,
}

impl PatternLink {
    /// A link row with a stable `id` reading `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_click: None,
        }
    }

    /// Called on a click or Enter.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PatternLink {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let look = LinkLook::current();
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = focus.is_focused(window);
        let on_key = self.on_click.clone();
        let on_click = self.on_click;
        div()
            .id(self.id)
            .track_focus(&focus.tab_index(0).tab_stop(true))
            .relative()
            .w_full()
            .py(LINK_PADDING.0)
            .px(LINK_PADDING.1)
            .text_color(look.text)
            .type_style(TypeStyle::BodyCompact01)
            .cursor(CursorStyle::PointingHand)
            .hover(move |s| s.bg(look.hover))
            // GPUI already turns Enter/Space on a focused clickable element into a
            // click (on key up). The key handler below follows Enter itself (a
            // link ignores Space), so a keyboard click is ignored here or the
            // link would follow twice.
            .on_click(move |e, window, cx| {
                if let Some(f) = on_click.as_ref().filter(|_| !e.is_keyboard()) {
                    f(window, cx)
                }
            })
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                if e.keystroke.key == "enter" && !e.is_held {
                    if let Some(f) = &on_key {
                        f(window, cx);
                    }
                    cx.stop_propagation();
                }
            })
            .child(self.label)
            // `outline-offset: -2px`: the ring sits inside the row.
            .when(focused, |d| d.child(inset_outline(2., look.focus)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_the_css() {
        assert_eq!(PANEL_MAX_WIDTH * tokens::REM_PX, 288.);
        assert_eq!(LINK_PADDING.0.px_value(), 12.);
        assert_eq!(LINK_PADDING.1.px_value(), 16.);
    }

    #[test]
    fn colors_come_from_the_css_roles() {
        let look = LinkLook::current();
        assert_eq!(look.text, color(ColorRole::TextPrimary));
        assert_eq!(look.hover, color(ColorRole::LayerHover01));
        assert_eq!(look.focus, color(ColorRole::Focus));
        assert_eq!(rule_color(), color(ColorRole::BorderSubtle01));
    }
}
