//! Small helpers for overlay demos that embed form controls.
use gpui_kit::{
    App, AppContext as _, Entity, IntoElement, SharedString, Window, component::input::InputState,
};
use u0ui::prelude::*;

/// A Kit input state seeded with `value`.
pub fn input_state(value: &str, window: &mut Window, cx: &mut App) -> Entity<InputState> {
    let value: SharedString = value.to_string().into();
    cx.new(|cx| InputState::new(window, cx).default_value(value))
}

/// The u0ui text input (`.cds--text-input`) with a label, on `level`.
pub fn text_field(
    id: &'static str,
    label: impl Into<SharedString>,
    state: &Entity<InputState>,
    level: LayerLevel,
) -> impl IntoElement {
    u0ui::components::Input::new(id, state)
        .label(label)
        .layer(level)
}
