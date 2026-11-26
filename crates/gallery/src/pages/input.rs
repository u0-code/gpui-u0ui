//! Forms › Input: the reference preview (text, invalid, password), a
//! configurable live input, and every size and state.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::{FieldSize, Input, InputState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let state = |value: &'static str, window: &mut Window, cx: &mut App| {
        cx.new(|cx| InputState::new(window, cx).default_value(value))
    };
    let reference = [
        state("", window, cx),
        state("name@company", window, cx),
        state("secret-key", window, cx),
    ];
    let live = state("", window, cx);
    let matrix = [(); 9].map(|_| state("", window, cx));
    let layer = [
        state("", window, cx),
        state("prod-us-east", window, cx),
        state("prod-us-east", window, cx),
    ];
    cx.new(|_| InputPage {
        reference,
        live,
        matrix,
        layer,
        size: 1,
        disabled: false,
        readonly: false,
        invalid: false,
        warn: false,
        password: false,
        hide_label: false,
        helper: true,
    })
    .into()
}

struct InputPage {
    reference: [Entity<InputState>; 3],
    live: Entity<InputState>,
    matrix: [Entity<InputState>; 9],
    layer: [Entity<InputState>; 3],
    size: usize,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    warn: bool,
    password: bool,
    hide_label: bool,
    helper: bool,
}

impl Render for InputPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut InputPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let size = FieldSize::ALL[self.size];

        // `components/Input/preview.html` (`cu-stack`, max-width 360).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .max_w(rems(22.5))
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_06)
                    .child(
                        Input::new("ref-name", &self.reference[0])
                            .label("Cluster name")
                            .placeholder("e.g. prod-us-east")
                            .helper_text("Lowercase letters, numbers and hyphens."),
                    )
                    .child(
                        Input::new("ref-email", &self.reference[1])
                            .label("Email")
                            .invalid(true)
                            .invalid_text("Enter a valid email address."),
                    )
                    .child(
                        Input::new("ref-key", &self.reference[2])
                            .label("API key")
                            .password(true)
                            .password_labels("Show password", "Hide password"),
                    ),
            );

        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            div().max_w(rems(22.5)).child(
                Input::new("live", &self.live)
                    .label("Cluster name")
                    .hide_label(self.hide_label)
                    .placeholder("e.g. prod-us-east")
                    .when_helper(self.helper)
                    .size(size)
                    .disabled(self.disabled)
                    .readonly(self.readonly)
                    .invalid(self.invalid)
                    .invalid_text("Enter a valid cluster name.")
                    .warn(self.warn)
                    .warn_text("Names over 30 characters are truncated in the console.")
                    .password(self.password)
                    .password_labels("Show password", "Hide password"),
            ),
        );

        // Sizes × states, plus the field on layer-01 (field-02).
        let states: [State; 3] = [
            ("rest", |i| i),
            ("invalid", |i| i.invalid(true).invalid_text("Required.")),
            ("warn", |i| i.warn(true).warn_text("Check this value.")),
        ];
        let matrix = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Sizes × states"))
            .children(FieldSize::ALL.iter().enumerate().map(|(row, size)| {
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_05)
                    .children(states.iter().enumerate().map(|(col, (name, apply))| {
                        div().w(rems(13.)).child(apply(
                            Input::new(
                                gpui_kit::SharedString::from(format!("m-{row}-{col}")),
                                &self.matrix[row * 3 + col],
                            )
                            .label(format!("{} · {}", size.name(), name))
                            .placeholder("Placeholder")
                            .size(*size),
                        ))
                    }))
            }))
            .child(
                div()
                    .p(tokens::SPACING_05)
                    .bg(color(ColorRole::Layer01))
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_05)
                    .child(
                        div().w(rems(13.)).child(
                            Input::new("layer-two", &self.layer[0])
                                .label("On layer-01 (field-02)")
                                .placeholder("e.g. prod-us-east")
                                .layer(LayerLevel::Two),
                        ),
                    )
                    .child(
                        div().w(rems(13.)).child(
                            Input::new("layer-disabled", &self.layer[1])
                                .label("Disabled")
                                .layer(LayerLevel::Two)
                                .disabled(true)
                                .helper_text("Helper text dims too."),
                        ),
                    )
                    .child(
                        div().w(rems(13.)).child(
                            Input::new("layer-readonly", &self.layer[2])
                                .label("Read-only")
                                .layer(LayerLevel::Two)
                                .readonly(true),
                        ),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "input-size",
                    &FieldSize::ALL.map(|s| s.name()),
                    self.size,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.size = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "input-helper",
                "helperText",
                self.helper,
                toggle(|this, on| this.helper = on),
            ))
            .child(controls::toggle(
                "input-hide-label",
                "hideLabel",
                self.hide_label,
                toggle(|this, on| this.hide_label = on),
            ))
            .child(controls::toggle(
                "input-password",
                "type=password",
                self.password,
                toggle(|this, on| this.password = on),
            ))
            .child(controls::toggle(
                "input-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "input-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "input-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "input-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
            .example(matrix)
    }
}

/// A named way to dress an input for the state matrix.
type State = (&'static str, fn(Input) -> Input);

trait WhenHelper {
    fn when_helper(self, on: bool) -> Self;
}

impl WhenHelper for Input {
    fn when_helper(self, on: bool) -> Self {
        if on {
            self.helper_text("Lowercase letters, numbers and hyphens.")
        } else {
            self
        }
    }
}
