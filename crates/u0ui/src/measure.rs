//! Sizes measured while a frame is laid out, for layouts that depend on
//! their own width (CSS container queries, `auto-fill` tracks, fits).
//!
//! A component measures in a `canvas` prepaint and keeps the result in
//! keyed state that the next render reads. GPUI drops a `notify` made while
//! a frame is drawn (`WindowInvalidator::invalidate_view` returns `false`
//! outside `DrawPhase::None`), and `Window::refresh` is a no-op then too, so
//! a measurement that changed nothing else, such as after a live window
//! resize, stayed unused until the next input event redrew the window.
//! [`store`] asks for the next frame explicitly.
use gpui_kit::{App, Entity, Window};

/// Keeps `value`, measured in this frame's prepaint, in `state`; when it
/// changed, redraws on the next frame so the layout that reads it follows.
/// Returns whether it changed.
pub fn store<T: PartialEq + 'static>(
    state: &Entity<T>,
    value: T,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    if *state.read(cx) == value {
        return false;
    }
    state.update(cx, |cur, cx| {
        *cur = value;
        cx.notify();
    });
    redraw_next_frame(window);
    true
}

/// Redraws the whole window on the next frame: what a change found while
/// drawing (in prepaint or paint) must ask for, since a notify then is lost.
pub fn redraw_next_frame(window: &Window) {
    window.on_next_frame(|window, _| window.refresh());
}
