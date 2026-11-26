//! Gallery-only stand-ins for form controls that live in another component
//! batch (Input, Checkbox), so overlay demos can show the reference content.
//! They wrap Kit's own controls (themed by the Carbon bridge) in Carbon field
//! chrome; replace them with the u0ui components once those land.
use gpui_kit::{
    App, AppContext as _, Entity, IntoElement, ParentElement, SharedString, Styled, Window,
    component::input::{Input, InputState},
    div, rems,
};
use u0ui::prelude::*;

/// A Kit input state seeded with `value`.
pub fn input_state(value: &str, window: &mut Window, cx: &mut App) -> Entity<InputState> {
    let value: SharedString = value.to_string().into();
    cx.new(|cx| InputState::new(window, cx).default_value(value))
}

/// A Carbon text input (`.cds--text-input`): label-01 label, `field` fill at
/// `level`, 1px `border-strong` bottom rule, density height.
pub fn text_field(
    label: impl Into<SharedString>,
    state: &Entity<InputState>,
    level: LayerLevel,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child(label.into()),
        )
        .child(
            div()
                .h(density::current().field().rems())
                .px(tokens::SPACING_05)
                .flex()
                .items_center()
                .bg(color(level.field()))
                .border_b_1()
                .border_color(color(level.border_strong()))
                .type_style(TypeStyle::BodyCompact01)
                .child(Input::new(state).appearance(false).w_full()),
        )
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
