//! `:focus-visible` input modality, tracked per window the way Chromium does.
//!
//! GPUI's [`Window::last_input_was_keyboard`] starts every window in mouse
//! modality and flips to mouse on every pointer *move*. That is right for its
//! purpose (suppressing hover while navigating by keyboard) but not for
//! `:focus-visible`, which in Chromium
//!
//! - matches programmatic focus while no pointer has been pressed yet (a dialog
//!   open at load with initial focus on a button shows the ring), and
//! - stays on a keyboard-focused element when the pointer merely moves; only a
//!   pointer press switches to pointer modality, and the next key switches back.
//!
//! [`focus_visible_mode`] answers that. u0ui feeds it from raw input: a
//! keystroke interceptor (key presses, not lone modifiers) and a window-level
//! capture-phase mouse-down listener painted by a Base [`RootPlugin`] that
//! [`crate::init`] registers, so every window opened through
//! `gpui_kit::open_window` tracks it without application code. Hover
//! suppression keeps using GPUI's modality.

use std::cell::RefCell;
use std::collections::HashSet;

use gpui_kit::base::{Root, RootPlugin};
use gpui_kit::{
    App, Context, DispatchPhase, FocusHandle, IntoElement, MouseDownEvent, Render, Styled as _,
    Window, WindowId, canvas,
};

/// The modality `:focus-visible` follows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Modality {
    /// Before any pointer press, and after a key press: rings show.
    #[default]
    Keyboard,
    /// After a pointer press: rings hide.
    Pointer,
}

/// The raw inputs that can change the modality.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// A key press (not a lone modifier).
    KeyDown,
    /// A mouse button or touch press.
    PointerDown,
    /// Pointer movement, wheel or trackpad scrolling.
    PointerMove,
}

impl Modality {
    /// The modality after `input`: keys and presses decide, movement does not.
    pub fn after(self, input: Input) -> Self {
        match input {
            Input::KeyDown => Modality::Keyboard,
            Input::PointerDown => Modality::Pointer,
            Input::PointerMove => self,
        }
    }
}

thread_local! {
    /// Windows whose last press was a pointer press. Absent = keyboard
    /// modality (a fresh window). Read without a context, like the theme.
    static POINTER: RefCell<HashSet<WindowId>> = RefCell::new(HashSet::new());
}

/// The `:focus-visible` modality of `window`.
pub fn modality(window: &Window) -> Modality {
    let id = window.window_handle().window_id();
    if POINTER.with(|p| p.borrow().contains(&id)) {
        Modality::Pointer
    } else {
        Modality::Keyboard
    }
}

/// Whether a focused element in `window` matches `:focus-visible`: true
/// before any pointer press and after a key press, false after a pointer
/// press, unchanged by pointer movement.
pub fn focus_visible_mode(window: &Window) -> bool {
    modality(window) == Modality::Keyboard
}

/// Whether `focus` shows its `:focus-visible` ring.
pub fn is_focus_visible(focus: &FocusHandle, window: &Window) -> bool {
    focus.is_focused(window) && focus_visible_mode(window)
}

/// Apply `input` to `window`'s modality, redrawing the window when it changes.
pub fn record(input: Input, window: &mut Window) {
    let id = window.window_handle().window_id();
    let before = modality(window);
    let after = before.after(input);
    if after == before {
        return;
    }
    POINTER.with(|p| {
        let mut p = p.borrow_mut();
        match after {
            Modality::Pointer => p.insert(id),
            Modality::Keyboard => p.remove(&id),
        }
    });
    window.refresh();
}

/// Whether a keystroke GPUI reports is a lone modifier (pressed and released
/// without another key), which GPUI does not count as keyboard input either.
fn lone_modifier(key: &str) -> bool {
    matches!(key, "shift" | "control" | "alt" | "platform" | "function")
}

/// Register the trackers. Called by [`crate::init`], before any window opens.
pub(crate) fn init(cx: &mut App) {
    Root::register_plugin::<PressProbe>(cx, PressProbe::new);
    cx.intercept_keystrokes(|event, window, _| {
        if !lone_modifier(&event.keystroke.key) {
            record(Input::KeyDown, window);
        }
    })
    .detach();
    cx.on_window_closed(|_, id| {
        POINTER.with(|p| p.borrow_mut().remove(&id));
    })
    .detach();
}

/// A Base root plugin that paints nothing and listens, window-wide and in the
/// capture phase (before any element's handler moves focus), for pointer
/// presses. GPUI has no app-level mouse hook, so it must be re-registered
/// every frame from paint; the root plugin is mounted in every Kit window.
struct PressProbe;

impl PressProbe {
    fn new(window: &mut Window, _: &mut Context<Self>) -> Self {
        // A new window starts in keyboard modality, even if a closed window's
        // id is reused.
        let id = window.window_handle().window_id();
        POINTER.with(|p| p.borrow_mut().remove(&id));
        PressProbe
    }
}

impl Render for PressProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            |_, _, window, _| {
                window.on_mouse_event(|_: &MouseDownEvent, phase, window, _| {
                    if phase == DispatchPhase::Capture {
                        record(Input::PointerDown, window);
                    }
                });
            },
        )
        .absolute()
        .size_0()
    }
}

impl RootPlugin for PressProbe {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_keyboard_modality() {
        assert_eq!(Modality::default(), Modality::Keyboard);
    }

    #[test]
    fn presses_decide_and_movement_does_not() {
        let m = Modality::Keyboard;
        assert_eq!(m.after(Input::PointerMove), Modality::Keyboard);
        let m = m.after(Input::PointerDown);
        assert_eq!(m, Modality::Pointer);
        assert_eq!(m.after(Input::PointerMove), Modality::Pointer);
        assert_eq!(m.after(Input::KeyDown), Modality::Keyboard);
    }

    #[test]
    fn lone_modifiers_are_not_key_presses() {
        assert!(lone_modifier("shift"));
        assert!(lone_modifier("platform"));
        assert!(!lone_modifier("tab"));
        assert!(!lone_modifier("a"));
    }
}
