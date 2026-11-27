//! StateRow: a row showing the three selection states (`StateRow` in
//! `src/ghost/foundations.jsx`, the FocusStates specimen).
//!
//! `.g-staterow`: a `density-row-md` button on `layer-01`, 12px padding,
//! 16px icon and 14px `text-secondary` label 8px apart; hover
//! `layer-hover-01` + `text-primary`; current, selected and focus per
//! [`Selection`] (current/selected text `text-primary`). `focus_ring` forces
//! the focus outline for the specimen; real keyboard focus draws it too.
use crate::{
    density,
    icons::{CarbonIcon, Icon, IconSize},
    theme::{Selection, color},
    tokens::{self, ColorRole},
    u0::primitives::parts,
};
use gpui_kit::{
    App, BoxShadow, ClickEvent, CursorStyle, ElementId, FontWeight, Hsla, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// (background, text) of a state row: `.g-staterow`, `:hover`, then the
/// current/selected rules that follow it in the cascade.
pub fn colors(current: bool, selected: bool, hovered: bool) -> (Hsla, Hsla) {
    let sel = Selection::of(current, selected);
    let mut bg = color(ColorRole::Layer01);
    let mut fg = color(ColorRole::TextSecondary);
    if hovered {
        bg = color(ColorRole::LayerHover01);
        fg = color(ColorRole::TextPrimary);
    }
    if let Some(b) = sel.background {
        bg = b;
        fg = color(ColorRole::TextPrimary);
    }
    (bg, fg)
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A selection-state row. See the module docs.
#[derive(IntoElement)]
pub struct StateRow {
    id: ElementId,
    label: SharedString,
    icon: CarbonIcon,
    current: bool,
    selected: bool,
    focus_ring: bool,
    on_click: Option<ClickHandler>,
}

impl StateRow {
    /// A row with a stable `id` and a `label` (Chat icon).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: CarbonIcon::Chat,
            current: false,
            selected: false,
            focus_ring: false,
            on_click: None,
        }
    }

    /// The leading icon (default Chat).
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = icon;
        self
    }

    /// The open item (`aria-current="page"`).
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Part of a multi-selection (`aria-pressed`).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Always show the focus outline (specimens).
    pub fn focus_ring(mut self, on: bool) -> Self {
        self.focus_ring = on;
        self
    }

    /// Press handler.
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for StateRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = parts::keyed_focus((id.clone(), "focus"), window, cx);
        let hover = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let (bg, fg) = colors(self.current, self.selected, *hover.read(cx));
        let sel = Selection::of(self.current, self.selected);
        let ring = self.focus_ring || parts::focus_visible(&focus, window);
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .accessibility_label(self.label.clone())
            .selected(self.selected)
            .relative()
            .flex()
            .items_center()
            .justify_start()
            .gap(tokens::SPACING_03)
            .w_full()
            .h(density::current().row())
            .px(tokens::SPACING_04)
            .bg(bg)
            .text_color(fg)
            .text_size(rems(0.875))
            .line_height(rems(1.125))
            .when(sel.semibold, |b| b.font_weight(FontWeight::SEMIBOLD))
            .when_some(sel.indicator, |b, c| {
                b.shadow(vec![BoxShadow {
                    color: c,
                    offset: gpui_kit::point(px(3.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(0.),
                    inset: true,
                }])
            })
            .cursor(CursorStyle::PointingHand)
            .on_hover(move |h, _, cx| {
                hover.update(cx, |v, cx| {
                    *v = *h;
                    cx.notify();
                })
            })
            .child(Icon::carbon(self.icon).size(IconSize::S16).color(fg))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.label),
            )
            .when(ring, |b| {
                // `outline: 2px solid var(--state-focus); outline-offset: -2px`.
                b.child(crate::components::form::inset_outline(2., sel.focus))
            })
            .when_some(self.on_click, |b, f| {
                b.on_click(move |e, w, cx| f(e, w, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn states_stack_as_in_ghost_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            colors(false, false, false),
            (c(ColorRole::Layer01), c(ColorRole::TextSecondary))
        );
        assert_eq!(colors(false, false, true).0, c(ColorRole::LayerHover01));
        assert_eq!(
            colors(true, false, true),
            (
                c(ColorRole::StateCurrentBackground),
                c(ColorRole::TextPrimary)
            )
        );
        // Current + selected: the selected fill, the current bar and weight.
        assert_eq!(
            colors(true, true, false).0,
            c(ColorRole::StateSelectedBackground)
        );
        let both = Selection::of(true, true);
        assert_eq!(both.indicator, Some(c(ColorRole::StateCurrentIndicator)));
        assert!(both.semibold);
        assert_eq!(Selection::of(false, true).indicator, None);
        assert_eq!(both.focus, c(ColorRole::StateFocus));
    }
}
