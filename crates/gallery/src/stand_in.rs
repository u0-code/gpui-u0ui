//! Small helpers for overlay demos that embed form controls. The checkbox is
//! a gallery-only stand-in until the u0ui Checkbox lands (another batch).
use gpui_kit::{
    App, AppContext as _, Entity, IntoElement, ParentElement, SharedString, Styled, Window,
    component::input::InputState, div, rems,
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

#[allow(dead_code)]
/// A Carbon checkbox (`.cds--checkbox`): 16px box, 2px radius, 1px
/// `icon-primary` border, filled `icon-primary` with a check when checked.
pub fn checkbox(
    id: impl Into<gpui_kit::ElementId>,
    label: impl Into<SharedString>,
    checked: bool,
    on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    use gpui_kit::{InteractiveElement as _, StatefulInteractiveElement as _};
    div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .gap(tokens::SPACING_03)
        .min_h(rems(1.5))
        .cursor_pointer()
        .on_click(move |_, window, cx| on_change(!checked, window, cx))
        .child(
            div()
                .size(rems(1.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(rems(0.125))
                .border_1()
                .border_color(color(ColorRole::IconPrimary))
                .when_checked(checked),
        )
        .child(
            div()
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child(label.into()),
        )
}

#[allow(dead_code)]
trait WhenChecked {
    fn when_checked(self, checked: bool) -> Self;
}

impl WhenChecked for gpui_kit::Div {
    fn when_checked(self, checked: bool) -> Self {
        if checked {
            self.bg(color(ColorRole::IconPrimary)).child(
                Icon::carbon(CarbonIcon::Checkmark)
                    .size(IconSize::S16)
                    .color(color(ColorRole::IconInverse)),
            )
        } else {
            self
        }
    }
}
