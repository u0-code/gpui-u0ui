//! Forms › Select: the reference Instance profile dropdown and a configurable
//! live dropdown (keyboard: ↑/↓, Enter, Escape, type-ahead).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::FieldSize;
use u0ui::components::select::{Select, SelectType};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SelectPage {
        size: 1,
        kind: 0,
        value: None,
        disabled: false,
        readonly: false,
        invalid: false,
        warn: false,
    })
    .into()
}

struct SelectPage {
    size: usize,
    kind: usize,
    value: Option<SharedString>,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    warn: bool,
}

const PROFILES: [(&str, &str); 3] = [
    ("bx2-2x8", "bx2-2x8 · 2 vCPU / 8 GB"),
    ("bx2-4x16", "bx2-4x16 · 4 vCPU / 16 GB"),
    ("cx2-8x16", "cx2-8x16 · 8 vCPU / 16 GB"),
];

const REGIONS: [&str; 8] = [
    "Dallas",
    "Frankfurt",
    "London",
    "Madrid",
    "Osaka",
    "São Paulo",
    "Sydney",
    "Tokyo",
];

impl Render for SelectPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut SelectPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let set = |f: fn(&mut SelectPage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };

        // `components/Select/preview.html` (max-width 320).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().max_w(rems(20.)).child(
                    Select::new("ref-profile")
                        .label("Instance profile")
                        .items(PROFILES)
                        .default_value("bx2-4x16")
                        .placeholder("Choose an option")
                        .helper_text("You can resize later."),
                ),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div().max_w(rems(20.)).child(
                    Select::new("live")
                        .label("Region")
                        .items_disabled_sydney()
                        .value(self.value.clone())
                        .placeholder("Choose an option")
                        .helper_text("Data stays in this region.")
                        .size(FieldSize::ALL[self.size])
                        .kind(SelectType::ALL[self.kind])
                        .disabled(self.disabled)
                        .readonly(self.readonly)
                        .invalid(self.invalid)
                        .invalid_text("Choose a region.")
                        .warn(self.warn)
                        .warn_text("This region is near capacity.")
                        .on_value_change({
                            let entity = entity.clone();
                            move |value, _, cx| {
                                let value = value.cloned();
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
                    // The reference's `value` is a string, "" when nothing is chosen.
                    .child(SharedString::from(format!(
                        "value = \"{}\"",
                        self.value.as_deref().unwrap_or_default()
                    ))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "select-size",
                    &FieldSize::ALL.map(|s| s.name()),
                    self.size,
                    set(|this, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "type",
                controls::choices(
                    "select-type",
                    &SelectType::ALL.map(|t| t.name()),
                    self.kind,
                    set(|this, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "select-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "select-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "select-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "select-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        Body::new()
            .playground(live)
            .hint("↑/↓, Enter, Escape, type a letter")
            .config(config)
            .example(reference)
    }
}

trait DisableSydney {
    fn items_disabled_sydney(self) -> Self;
}

impl DisableSydney for Select {
    /// Show a disabled option in the live demo.
    fn items_disabled_sydney(self) -> Self {
        self.items(
            REGIONS.map(|r| u0ui::components::list_box::ListItem::new(r).disabled(r == "Sydney")),
        )
    }
}
