//! Shared u0 parts (`src/ghost.css` "Fields (compact)" and `.g-iconbtn`):
//! the compact field column, its label/helper/error lines, the 32px ghost
//! icon button and the chrome-less Kit text editor the primitives draw on.
use crate::{
    components::form,
    density,
    icons::{Icon, IconSize},
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, ClickEvent, CursorStyle, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window,
    component::{
        Sizable as _, Size,
        input::{Input as KitInput, InputState},
    },
    div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// A px length in rems (16px rem), so zoom reflows.
pub(crate) fn rem(px: f32) -> gpui_kit::Rems {
    gpui_kit::rems(px / tokens::REM_PX)
}

/// `.g-field { display: flex; flex-direction: column; gap: var(--spacing-02); min-width: 0 }`.
pub fn field() -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_02)
        .min_w_0()
}

/// `.cds--label.g-label`: `label-01` in `text-secondary`, no margin
/// (`.g-label { margin: 0 }`; the field's gap spaces it).
pub fn label(text: impl Into<SharedString>) -> Div {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// `.g-helper { font-size: 0.75rem; line-height: 1.33333; color: var(--text-helper) }`.
pub fn helper(text: impl Into<SharedString>) -> Div {
    div()
        .type_style(TypeStyle::HelperText01)
        .text_color(color(ColorRole::TextHelper))
        .child(text.into())
}

/// `.g-error { … color: var(--text-error) }`.
pub fn error(text: impl Into<SharedString>) -> Div {
    div()
        .type_style(TypeStyle::HelperText01)
        .text_color(color(ColorRole::TextError))
        .child(text.into())
}

/// The 2px inset focus ring (`outline: 2px solid var(--focus); outline-offset: -2px`).
pub fn focus_ring() -> Div {
    form::inset_outline(2., color(ColorRole::Focus))
}

/// Whether `focus` shows a `:focus-visible` ring (focused by keyboard).
pub fn focus_visible(focus: &FocusHandle, window: &Window) -> bool {
    focus.is_focused(window) && window.last_input_was_keyboard()
}

/// A focus handle kept under `id` across frames.
pub fn keyed_focus(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

/// A zero-size, non-tab-stop focus target that keeps the sequential focus
/// navigation starting point where a focused element blurred or vanished.
///
/// Chromium keeps that starting point at an element that loses focus through
/// `blur()` or removal, so the next Tab goes to the stop after it and
/// Shift+Tab to the one before. GPUI's Tab starts from the focused handle, and
/// from the top of the window when nothing (or a handle no longer painted) is
/// focused. GPUI orders every tracked handle, tab stop or not, so focusing
/// this anchor instead of blurring keeps the place, with no visible focus.
///
/// Put it right after the element (it is absolutely positioned, so it takes
/// no room or flex gap). When that element is still there (an input blurred
/// by Enter), pass it as `after`: Shift+Tab then moves back from it, so focus
/// lands on the stop before it, not on it.
pub fn nav_anchor(
    id: impl Into<ElementId>,
    handle: &FocusHandle,
    after: Option<FocusHandle>,
    cx: &App,
) -> gpui_kit::Stateful<Div> {
    let anchor = div()
        .id(id)
        .track_focus(&handle.clone().tab_stop(false))
        .absolute()
        .size_0();
    // Kit's root binds Shift+Tab to `root::TabPrev`, which is not exported.
    match (after, cx.build_action("root::TabPrev", None)) {
        (Some(after), Ok(tab_prev)) => anchor.on_boxed_action(&*tab_prev, move |a, window, cx| {
            after.focus(window, cx);
            after.dispatch_action(a, window, cx);
            cx.stop_propagation();
        }),
        _ => anchor,
    }
}

/// Where a list keeps its [`nav_anchor`]: the anchor's handle and the index
/// of the item that was removed while focus was in it.
pub struct FocusSlot {
    /// The anchor's focus handle.
    pub handle: FocusHandle,
    /// The removed item's index; the anchor sits before the item now there.
    pub at: usize,
}

/// A [`FocusSlot`] kept under `id` across frames.
pub fn focus_slot(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<FocusSlot> {
    window.use_keyed_state(id, cx, |_, cx| FocusSlot {
        handle: cx.focus_handle(),
        at: usize::MAX,
    })
}

/// Item `at` of a list is about to be removed. When focus is on one of its
/// `parts` (its own button or its close button), focus rests on the slot's
/// anchor at that index instead of vanishing with it, so Tab continues with
/// the stop after the item and Shift+Tab with the one before (Chromium's
/// sequential focus navigation starting point). Render the anchor with
/// [`insert_slot_anchor`].
pub fn vacate_slot(
    slot: &Entity<FocusSlot>,
    at: usize,
    parts: &[FocusHandle],
    window: &mut Window,
    cx: &mut App,
) {
    if !parts.iter().any(|h| h.is_focused(window)) {
        return;
    }
    let handle = slot.update(cx, |s, _| {
        s.at = at;
        s.handle.clone()
    });
    handle.focus(window, cx);
}

/// Put the slot's [`nav_anchor`] among a list's `children`, before the item
/// at its index (at the end when that index is past the last item).
pub fn insert_slot_anchor(
    children: &mut Vec<gpui_kit::AnyElement>,
    id: impl Into<ElementId>,
    slot: &Entity<FocusSlot>,
    cx: &App,
) {
    let s = slot.read(cx);
    let at = s.at.min(children.len());
    let anchor = nav_anchor(id, &s.handle, None, cx).into_any_element();
    children.insert(at, anchor);
}

/// The focus point of a window that a pressed control may take with it: the
/// focused handle and the stops before and after it in that frame.
struct Vacated {
    window: gpui_kit::AnyWindowHandle,
    here: FocusHandle,
    prev: Option<FocusHandle>,
    next: Option<FocusHandle>,
}

#[derive(Default)]
struct VacatedFocus {
    point: Option<Vacated>,
    _intercept: Option<gpui_kit::Subscription>,
}

impl gpui_kit::Global for VacatedFocus {}

/// A control inside `within` is being pressed and may disappear with it,
/// because its caller stops rendering it (a dismissed tag, a removed
/// attachment, a dialog button closing its dialog), so it cannot hold a
/// [`nav_anchor`]. Call it before running the press handler.
///
/// When the focused element is gone by the next Tab, that Tab continues from
/// where it was, as Chromium keeps the sequential focus navigation starting
/// point at a removed element: Tab goes to the stop that followed it,
/// Shift+Tab to the one before, and nothing shows focus meanwhile. When it is
/// still there (or focus moved elsewhere), Tab is untouched.
///
/// GPUI does not expose its tab order, so the stops around the element are
/// read from the current frame by stepping focus forward and back and then
/// returning it; focus listeners only see the final state, which is unchanged.
pub fn vacate_focus(within: &FocusHandle, window: &mut Window, cx: &mut App) {
    if !(within.is_focused(window) || within.contains_focused(window, cx)) {
        return;
    }
    let Some(here) = window.focused(cx) else {
        return;
    };
    window.focus_next(cx);
    let next = window.focused(cx).filter(|h| *h != here);
    window.focus(&here, cx);
    window.focus_prev(cx);
    let prev = window.focused(cx).filter(|h| *h != here);
    window.focus(&here, cx);
    if !cx.has_global::<VacatedFocus>() {
        let intercept = cx.intercept_keystrokes(|event, window, cx| {
            let k = &event.keystroke;
            let m = k.modifiers;
            if k.key != "tab" || m.control || m.alt || m.platform || m.function {
                return;
            }
            match cx.global::<VacatedFocus>().point.as_ref() {
                Some(p) if p.window == window.window_handle() => {}
                _ => return,
            }
            // Any Tab in that window ends the point.
            let Some(p) = cx.global_mut::<VacatedFocus>().point.take() else {
                return;
            };
            let gone = window.focused(cx).as_ref() == Some(&p.here) && !p.here.is_rendered(window);
            if !gone {
                return;
            }
            let to = if m.shift { p.prev } else { p.next };
            if let Some(to) = to.filter(|h| h.is_rendered(window)) {
                to.focus(window, cx);
                cx.stop_propagation();
            }
        });
        cx.set_global(VacatedFocus {
            point: None,
            _intercept: Some(intercept),
        });
    }
    cx.global_mut::<VacatedFocus>().point = Some(Vacated {
        window: window.window_handle(),
        here,
        prev,
        next,
    });
}

/// The Kit single-line editor without chrome, in the primitives' 14px body
/// type (`font: inherit; font-size: 0.875rem; border: 0; background: none;
/// outline: 0`). The caller draws the field around it.
pub fn editor(state: &Entity<InputState>) -> KitInput {
    KitInput::new(state)
        .appearance(false)
        .with_size(Size::XSmall)
        .p_0()
        .h_full()
        .flex_1()
        .min_w_0()
        .text_size(rem(14.))
        .line_height(rem(18.))
}

/// The same editor in `.g-mono` (IBM Plex Mono 12px).
pub fn mono_editor(state: &Entity<InputState>) -> KitInput {
    editor(state)
        .font(crate::typography::font(tokens::FontFamilyKey::Mono, 400))
        .text_size(rem(12.))
        .line_height(rem(16.))
}

/// `.g-iconbtn` colors: `icon-primary`, `--on` `interactive`, disabled
/// `icon-disabled`; hover `layer-hover-01` (none when disabled).
pub fn icon_button_colors(on: bool, hovered: bool, disabled: bool) -> (Hsla, Hsla) {
    let fg = if disabled {
        ColorRole::IconDisabled
    } else if on {
        ColorRole::Interactive
    } else {
        ColorRole::IconPrimary
    };
    let bg = if hovered && !disabled {
        color(ColorRole::LayerHover01)
    } else {
        Hsla::transparent_black()
    };
    (color(fg), bg)
}

/// `useFlash(ms)`: a keyed "on for `ms` after [`flash`]" flag (CopyField's
/// checkmark). Returns the state; read it with [`flash_on`].
pub fn flash_state(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Option<std::time::Instant>> {
    window.use_keyed_state(id, cx, |_, _| None)
}

/// Whether a [`flash_state`] is still on, `ms` after it was set.
pub fn flash_on(state: &Entity<Option<std::time::Instant>>, ms: u64, cx: &App) -> bool {
    state
        .read(cx)
        .is_some_and(|at| at.elapsed() < std::time::Duration::from_millis(ms))
}

/// Turn a [`flash_state`] on now and repaint when it turns off `ms` later
/// (a second flash restarts it: `clearTimeout(t.current)`).
pub fn flash(state: &Entity<Option<std::time::Instant>>, ms: u64, cx: &mut App) {
    state.update(cx, |s, cx| {
        *s = Some(std::time::Instant::now());
        cx.notify();
    });
    let state = state.downgrade();
    let delay = std::time::Duration::from_millis(ms);
    cx.spawn(async move |cx| {
        cx.background_executor().timer(delay).await;
        let _ = state.update(cx, |_, cx| cx.notify());
    })
    .detach();
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// `.g-iconbtn`: a square ghost icon button `density-field-md` wide (32px
/// compact), 16px icon, `layer-hover-01` on hover, a 2px inset `focus` ring
/// on keyboard focus. The label is its accessible name (callers pass it).
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: Icon,
    label: SharedString,
    on: bool,
    disabled: bool,
    tab_stop: bool,
    size: Option<f32>,
    icon_size: Option<f32>,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    /// A button with a stable `id`, an icon and its accessible `label`.
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<Icon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            on: false,
            disabled: false,
            tab_stop: true,
            size: None,
            icon_size: None,
            on_click: None,
        }
    }

    /// The pressed/active look (`.g-iconbtn--on`: `interactive`).
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Disabled (`icon-disabled`, `cursor: not-allowed`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Leave it out of the Tab order (`tabIndex={-1}`).
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    /// Override the square size in px (default `density-field-md`).
    pub fn size_px(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// Override the icon size in px (default 16).
    pub fn icon_px(mut self, size: f32) -> Self {
        self.icon_size = Some(size);
        self
    }

    /// Activation handler (pointer, Enter, Space).
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let (fg, bg) = icon_button_colors(self.on, *hovered.read(cx), self.disabled);
        let size = self.size.unwrap_or(density::current().field().px_value());
        let ring = focus_visible(&focus, window);
        let icon = self.icon.size(IconSize::S16).color(fg);
        let icon = match self.icon_size {
            Some(px) => icon.size_length(gpui_kit::px(px)),
            None => icon,
        };
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .tab_stop(self.tab_stop)
            .disabled(self.disabled)
            .accessibility_label(self.label)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(rem(size))
            .bg(bg)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover(move |h, _, cx| {
                hovered.update(cx, |v, cx| {
                    if *v != *h {
                        *v = *h;
                        cx.notify();
                    }
                })
            })
            .child(icon)
            .when(ring, |this| this.child(focus_ring()))
            .when_some(self.on_click, |this, handler| {
                // A press that removes the button keeps the Tab starting point.
                let focus = focus.clone();
                this.on_click(move |e, window, cx| {
                    vacate_focus(&focus, window, cx);
                    handler(e, window, cx)
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn icon_button_follows_g_iconbtn() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        // `.g-iconbtn { color: var(--icon-primary) }`, `:hover { background: layer-hover-01 }`.
        assert_eq!(
            icon_button_colors(false, true, false),
            (c(ColorRole::IconPrimary), c(ColorRole::LayerHover01))
        );
        // `.g-iconbtn--on { color: var(--interactive) }`.
        assert_eq!(
            icon_button_colors(true, false, false).0,
            c(ColorRole::Interactive)
        );
        // `.g-iconbtn:disabled { color: icon-disabled; background: transparent }`.
        assert_eq!(
            icon_button_colors(false, true, true),
            (c(ColorRole::IconDisabled), Hsla::transparent_black())
        );
    }
}
