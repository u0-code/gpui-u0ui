//! Shared Ghost parts (`src/ghost.css` "Fields (compact)" and `.g-iconbtn`):
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
    App, ClickEvent, CursorStyle, Div, ElementId, Entity, FocusHandle, Hsla, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window,
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
                this.on_click(move |e, window, cx| handler(e, window, cx))
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
