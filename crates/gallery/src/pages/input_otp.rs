//! Forms › InputOTP: the reference codes (6 digits in 3+3, 4 digits invalid)
//! and a configurable live code.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::components::InputState;
use u0ui::components::input_otp::InputOtp;
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let six = cx.new(|cx| InputState::new(window, cx).default_value("482"));
    let four = cx.new(|cx| InputState::new(window, cx).default_value("12"));
    let live = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| OtpPage {
        six,
        four,
        live,
        layout: 0,
        invalid: false,
        disabled: false,
        letters: false,
    })
    .into()
}

struct OtpPage {
    six: Entity<InputState>,
    four: Entity<InputState>,
    live: Entity<InputState>,
    layout: usize,
    invalid: bool,
    disabled: bool,
    letters: bool,
}

const LAYOUTS: [&str; 3] = ["6", "3+3", "4"];

impl Render for OtpPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut OtpPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/InputOTP/preview.html` (`cu-row`, gap 40).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_08)
                    .child(
                        InputOtp::new("ref-six", &self.six)
                            .length(6)
                            .groups([3, 3])
                            .label("One-time code"),
                    )
                    .child(
                        InputOtp::new("ref-four", &self.four)
                            .length(4)
                            .invalid(true)
                            .label("One-time code"),
                    ),
            );

        let (length, groups) = match self.layout {
            0 => (6, vec![6]),
            1 => (6, vec![3, 3]),
            _ => (4, vec![4]),
        };
        let mut otp = InputOtp::new(
            SharedString::from(format!("live-{}-{}", self.layout, self.letters)),
            &self.live,
        )
        .length(length)
        .groups(groups)
        .invalid(self.invalid)
        .disabled(self.disabled)
        .label("Verification code");
        if self.letters {
            otp = otp.pattern(|c| c.is_ascii_alphanumeric());
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(otp)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!(
                        "value = {:?}",
                        self.live.read(cx).value()
                    ))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "length / groups",
                controls::choices("otp-layout", &LAYOUTS, self.layout, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.layout = i;
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::toggle(
                "otp-letters",
                "pattern: letters + digits",
                self.letters,
                toggle(|this, on| this.letters = on),
            ))
            .child(controls::toggle(
                "otp-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "otp-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ));

        Body::new()
            .playground(live)
            .hint("type or paste")
            .config(config)
            .example(reference)
    }
}
