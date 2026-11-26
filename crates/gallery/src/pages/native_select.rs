//! Forms › NativeSelect: the reference Region select with option groups and
//! a configurable live select.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::FieldSize;
use u0ui::components::native_select::NativeSelect;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| NativeSelectPage {
        size: 1,
        value: "lon".into(),
        disabled: false,
        readonly: false,
        invalid: false,
        warn: false,
    })
    .into()
}

struct NativeSelectPage {
    size: usize,
    value: SharedString,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    warn: bool,
}

fn regions(id: impl Into<gpui_kit::ElementId>) -> NativeSelect {
    NativeSelect::new(id)
        .label("Region")
        .group("Europe", [("fra", "Frankfurt"), ("lon", "London")])
        .group("Americas", [("dal", "Dallas")])
}

impl Render for NativeSelectPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut NativeSelectPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/NativeSelect/preview.html` (max-width 320).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().max_w(rems(20.)).child(
                    regions("ref-region")
                        .default_value("fra")
                        .helper_text("Data stays in this region."),
                ),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live"))
            .child(
                div().max_w(rems(20.)).child(
                    regions("live")
                        .value(self.value.clone())
                        .helper_text("Data stays in this region.")
                        .size(FieldSize::ALL[self.size])
                        .disabled(self.disabled)
                        .readonly(self.readonly)
                        .invalid(self.invalid)
                        .invalid_text("This region is unavailable.")
                        .warn(self.warn)
                        .warn_text("This region is near capacity.")
                        .on_value_change({
                            let entity = entity.clone();
                            move |value, _, cx| {
                                let value = value.clone();
                                entity.update(cx, |this, cx| {
                                    this.value = value;
                                    cx.notify();
                                })
                            }
                        }),
                ),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!("value = {}", self.value))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices("ns-size", &FieldSize::ALL.map(|s| s.name()), self.size, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.size = i;
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::toggle(
                "ns-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "ns-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "ns-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "ns-readonly",
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
                .child(live),
            Some(config.into_any_element()),
        )
    }
}
