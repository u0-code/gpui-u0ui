//! Forms › Textarea: the reference Description field with a counter and a
//! configurable live text area.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::{Textarea, TextareaState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let reference = cx.new(|cx| TextareaState::new(window, cx));
    let live = cx.new(|cx| TextareaState::new(window, cx));
    let states = [
        cx.new(|cx| TextareaState::new(window, cx).default_value("Billing service")),
        cx.new(|cx| TextareaState::new(window, cx).default_value("Billing service")),
        cx.new(|cx| TextareaState::new(window, cx)),
    ];
    cx.new(|_| TextareaPage {
        reference,
        live,
        states,
        rows: 1,
        counter: true,
        disabled: false,
        readonly: false,
        invalid: false,
        warn: false,
    })
    .into()
}

struct TextareaPage {
    reference: Entity<TextareaState>,
    live: Entity<TextareaState>,
    states: [Entity<TextareaState>; 3],
    rows: usize,
    counter: bool,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    warn: bool,
}

const ROWS: [&str; 3] = ["2", "4", "6"];

impl Render for TextareaPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut TextareaPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/Textarea/preview.html` (max-width 420).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().max_w(rems(26.25)).child(
                    Textarea::new("ref-desc", &self.reference)
                        .label("Description")
                        .placeholder("What is this service for?")
                        .helper_text("Max 200 characters.")
                        .enable_counter(true)
                        .max_count(200),
                ),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live"))
            .child(
                div().max_w(rems(26.25)).child(
                    Textarea::new("live", &self.live)
                        .label("Description")
                        .placeholder("What is this service for?")
                        .helper_text("Max 200 characters.")
                        .rows(ROWS[self.rows].parse().unwrap_or(4))
                        .enable_counter(self.counter)
                        .max_count(200)
                        .disabled(self.disabled)
                        .readonly(self.readonly)
                        .invalid(self.invalid)
                        .invalid_text("Describe the service in at least 10 characters.")
                        .warn(self.warn)
                        .warn_text("Descriptions are visible to every account member."),
                ),
            );

        let states = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("States"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_05)
                    .child(
                        div().w(rems(16.)).child(
                            Textarea::new("st-disabled", &self.states[0])
                                .label("Disabled")
                                .rows(2)
                                .disabled(true),
                        ),
                    )
                    .child(
                        div().w(rems(16.)).child(
                            Textarea::new("st-readonly", &self.states[1])
                                .label("Read-only")
                                .rows(2)
                                .readonly(true),
                        ),
                    )
                    .child(
                        div()
                            .w(rems(16.))
                            .p(tokens::SPACING_05)
                            .bg(color(ColorRole::Layer01))
                            .child(
                                Textarea::new("st-layer", &self.states[2])
                                    .label("On layer-01")
                                    .placeholder("field-02")
                                    .rows(2)
                                    .layer(LayerLevel::Two),
                            ),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "rows",
                controls::choices("ta-rows", &ROWS, self.rows, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.rows = i;
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::toggle(
                "ta-counter",
                "enableCounter",
                self.counter,
                toggle(|this, on| this.counter = on),
            ))
            .child(controls::toggle(
                "ta-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "ta-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "ta-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "ta-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(reference)
                .child(live)
                .child(states),
            Some(config.into_any_element()),
        )
    }
}
