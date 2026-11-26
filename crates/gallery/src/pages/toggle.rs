//! Actions › Toggle: the reference toggles (B, I, Pin, Pinned, Disabled) and
//! a configurable live toggle.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, FontWeight, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::components::toggle::{Toggle, ToggleSize, ToggleVariant};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TogglePage {
        variant: 0,
        size: 1,
        disabled: false,
        with_icon: false,
        pressed: false,
    })
    .into()
}

struct TogglePage {
    variant: usize,
    size: usize,
    disabled: bool,
    with_icon: bool,
    pressed: bool,
}

fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
}

impl Render for TogglePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut TogglePage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let set = |f: fn(&mut TogglePage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };
        let variant = ToggleVariant::ALL[self.variant];
        let size = ToggleSize::ALL[self.size];

        // `components/Toggle/preview.html` (`cu-row`).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                row()
                    .child(
                        Toggle::new("ref-bold")
                            .aria_label("Bold")
                            .default_pressed(true)
                            .child(div().font_weight(FontWeight::BOLD).child("B")),
                    )
                    .child(
                        Toggle::new("ref-italic")
                            .aria_label("Italic")
                            .child(div().italic().child("I")),
                    )
                    .child(
                        Toggle::new("ref-pin")
                            .variant(ToggleVariant::Outline)
                            .aria_label("Pin")
                            .child("Pin"),
                    )
                    .child(
                        Toggle::new("ref-pinned")
                            .variant(ToggleVariant::Outline)
                            .default_pressed(true)
                            .child("Pinned"),
                    )
                    .child(Toggle::new("ref-disabled").disabled(true).child("Disabled")),
            );

        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            row()
                .p(tokens::SPACING_05)
                .child(
                    Toggle::new("live")
                        .variant(variant)
                        .size(size)
                        .disabled(self.disabled)
                        .pressed(self.pressed)
                        .aria_label("Pin")
                        .on_pressed_change({
                            let entity = entity.clone();
                            move |on, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.pressed = on;
                                    cx.notify();
                                })
                            }
                        })
                        .when_icon(self.with_icon)
                        .child("Pin"),
                )
                .child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(SharedString::from(format!("aria-pressed={}", self.pressed))),
                ),
        );

        // Every variant × size, rest and pressed.
        let matrix = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Variants × sizes"))
            .children(ToggleVariant::ALL.iter().map(|v| {
                row().children(ToggleSize::ALL.iter().flat_map(|s| {
                    [false, true].map(|on| {
                        Toggle::new(SharedString::from(format!(
                            "m-{}-{}-{on}",
                            v.name(),
                            s.name()
                        )))
                        .variant(*v)
                        .size(*s)
                        .default_pressed(on)
                        .child(Icon::carbon(CarbonIcon::Pin).size(IconSize::S16))
                        .child(SharedString::from(format!("{} {}", v.name(), s.name())))
                    })
                }))
            }));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "toggle-variant",
                    &ToggleVariant::ALL.map(|v| v.name()),
                    self.variant,
                    set(|this, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "toggle-size",
                    &ToggleSize::ALL.map(|s| s.name()),
                    self.size,
                    set(|this, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "toggle-pressed",
                "pressed",
                self.pressed,
                toggle(|this, on| this.pressed = on),
            ))
            .child(controls::toggle(
                "toggle-icon",
                "icon",
                self.with_icon,
                toggle(|this, on| this.with_icon = on),
            ))
            .child(controls::toggle(
                "toggle-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
            .example(matrix)
    }
}

trait WhenIcon {
    fn when_icon(self, on: bool) -> Self;
}

impl WhenIcon for Toggle {
    fn when_icon(self, on: bool) -> Self {
        if on {
            self.child(Icon::carbon(CarbonIcon::Pin).size(IconSize::S16))
        } else {
            self
        }
    }
}
