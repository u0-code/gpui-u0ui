//! Overlays › Popover: the reference preview (open, bottom-left) and a
//! configurable live popover.
use crate::{controls, stand_in};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, component::input::InputState, div, rems,
};
use u0ui::components::{Align, FieldSize, Input, Popover};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let width = stand_in::input_state("100%", window, cx);
    let live_width = stand_in::input_state("100%", window, cx);
    cx.new(|_| PopoverPage {
        width,
        live_width,
        align: 4,
        caret: true,
        drop_shadow: true,
        live_open: false,
    })
    .into()
}

struct PopoverPage {
    width: Entity<InputState>,
    live_width: Entity<InputState>,
    /// Index into [`Align::ALL`].
    align: usize,
    caret: bool,
    drop_shadow: bool,
    live_open: bool,
}

/// The reference content: `cu-stack` gap 12 with a `cds--label` line and a
/// small Width input.
fn dimensions(id: &'static str, state: &Entity<InputState>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(rems(0.75))
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child("Set the layer size."),
        )
        .child(
            Input::new(id, state)
                .label("Width")
                .size(FieldSize::Sm)
                .layer(LayerLevel::Two),
        )
}

impl Render for PopoverPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut PopoverPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/Popover/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().flex().pl(rems(0.5)).pb(rems(10.)).child(
                    Popover::new(
                        "ref-popover",
                        Button::new("ref-dimensions", "Dimensions")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Md),
                    )
                    .default_open(true)
                    .align(Align::BottomLeft)
                    .child(dimensions("ref-width", &self.width)),
                ),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section(
                "Live: press the button; press outside or Escape to close",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap(tokens::SPACING_05)
                    .py(rems(12.))
                    .bg(color(ColorRole::Background))
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(
                        Popover::new(
                            "live-popover",
                            Button::new("live-dimensions", "Dimensions")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Md),
                        )
                        .open(self.live_open)
                        .on_open_change(toggle(|this, open| this.live_open = open))
                        .align(Align::ALL[self.align])
                        .caret(self.caret)
                        .drop_shadow(self.drop_shadow)
                        .child(dimensions("live-width", &self.live_width)),
                    )
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(if self.live_open { "open" } else { "closed" }),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "align",
                controls::choices(
                    "popover-align",
                    &Align::ALL.map(|a| a.name()),
                    self.align,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.align = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "popover-caret",
                "caret",
                self.caret,
                toggle(|this, on| this.caret = on),
            ))
            .child(controls::toggle(
                "popover-shadow",
                "dropShadow",
                self.drop_shadow,
                toggle(|this, on| this.drop_shadow = on),
            ))
            .child(controls::toggle(
                "popover-open",
                "open",
                self.live_open,
                toggle(|this, on| this.live_open = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(reference)
                .child(live),
            Some(config.into_any_element()),
        )
    }
}
