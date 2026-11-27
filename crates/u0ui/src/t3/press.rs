//! Small shared pieces for the composer and timeline cards: hover and
//! keyboard-focus state for the reference's bare `<button>`s (`.t3-chip-x`,
//! `.t3-cctl`, `.t3-wrow`, …), which Kit's unstyled [`gpui_kit::base::Button`]
//! drives, and the `:focus-visible` ring they all share.
use crate::{
    components::form::inset_outline,
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{App, Div, ElementId, Entity, FocusHandle, Hsla, Rems, Window};

/// px → rems at Carbon's 16px rem.
pub(crate) fn rem(px_value: f32) -> Rems {
    gpui_kit::rems(px_value / tokens::REM_PX)
}

/// A Carbon icon `size` px square in `ink`.
pub(crate) fn icon(name: CarbonIcon, size: f32, ink: Hsla) -> Icon {
    Icon::carbon(name).size_length(rem(size)).color(ink)
}

/// Keyed hover flag: (hovered now, the store to update from `on_hover`).
pub(crate) fn hover(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> (bool, Entity<bool>) {
    let store = window.use_keyed_state(id, cx, |_, _| false);
    (*store.read(cx), store)
}

/// The `on_hover` handler that keeps a [`hover`] store current.
pub(crate) fn track_hover(store: Entity<bool>) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
    move |over, _, cx| {
        store.update(cx, |h, cx| {
            if *h != *over {
                *h = *over;
                cx.notify();
            }
        })
    }
}

/// A keyed focus handle and whether it shows `:focus-visible` (focused by
/// the keyboard).
pub(crate) fn focus(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> (FocusHandle, bool) {
    let handle = window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let visible = handle.is_focused(window) && crate::focus::focus_visible_mode(window);
    (handle, visible)
}

/// A click handler without the event.
pub(crate) type Handler = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

/// `inset 0 0 0 1px <c>` (`box-shadow`), the reference's hairline frames.
pub(crate) fn inset_frame(c: Hsla) -> Vec<gpui_kit::BoxShadow> {
    vec![
        gpui_kit::BoxShadow::new(gpui_kit::px(0.), gpui_kit::px(0.), c)
            .spread_radius(gpui_kit::px(1.))
            .inset(),
    ]
}

/// `:focus-visible { outline: 2px solid var(--focus); outline-offset: -2px }`
/// (`t3.css` "review fixes"): an inset ring over a `relative` parent.
pub(crate) fn ring() -> Div {
    inset_outline(2., color(ColorRole::Focus))
}
