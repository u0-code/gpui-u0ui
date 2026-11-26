//! Forms › Label: a `.cds--label` over a bare text input; clicking the label
//! focuses the input (`htmlFor`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, Focusable as _, IntoElement, ParentElement,
    Render, Styled, Window, div, rems,
};
use u0ui::components::{Input, InputState, Label};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let reference = cx.new(|cx| InputState::new(window, cx));
    let live = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| LabelPage {
        reference,
        live,
        disabled: false,
        no_margin: false,
    })
    .into()
}

struct LabelPage {
    reference: Entity<InputState>,
    live: Entity<InputState>,
    disabled: bool,
    no_margin: bool,
}

impl Render for LabelPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut LabelPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let reference_focus = self.reference.read(cx).focus_handle(cx);
        let live_focus = self.live.read(cx).focus_handle(cx);

        // `components/Label/preview.html`: max-width 320.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .max_w(rems(20.))
                    .flex()
                    .flex_col()
                    .child(Label::new("Region").target(&reference_focus))
                    .child(
                        Input::new("ref-region", &self.reference)
                            .label("Region")
                            .hide_label(true)
                            .placeholder("us-south"),
                    ),
            );

        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            div()
                .max_w(rems(20.))
                .flex()
                .flex_col()
                .child(
                    Label::new("Billing region")
                        .disabled(self.disabled)
                        .no_margin(self.no_margin)
                        .target(&live_focus),
                )
                .child(
                    Input::new("live-region", &self.live)
                        .label("Billing region")
                        .hide_label(true)
                        .placeholder("eu-de")
                        .disabled(self.disabled),
                ),
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "label-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "label-no-margin",
                "noMargin",
                self.no_margin,
                toggle(|this, on| this.no_margin = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}
