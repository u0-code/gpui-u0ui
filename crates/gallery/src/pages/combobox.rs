//! Forms › Combobox: the reference single (Framework) and multiple (Tags)
//! comboboxes and a configurable live one.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::{Combobox, FieldSize, InputState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let mut state = || cx.new(|cx| InputState::new(window, cx));
    let framework = state();
    let tags = state();
    let live_single = state();
    let live_multi = state();
    cx.new(|_| ComboboxPage {
        framework,
        tags,
        live_single,
        live_multi,
        multiple: false,
        size: 1,
        value: vec!["Vue".into()],
        invalid: false,
        warn: false,
        disabled: false,
    })
    .into()
}

struct ComboboxPage {
    framework: Entity<InputState>,
    tags: Entity<InputState>,
    live_single: Entity<InputState>,
    live_multi: Entity<InputState>,
    multiple: bool,
    size: usize,
    value: Vec<SharedString>,
    invalid: bool,
    warn: bool,
    disabled: bool,
}

const FRAMEWORKS: [&str; 5] = ["React", "Angular", "Vue", "Svelte", "Web Components"];
const TAGS: [&str; 5] = ["prod", "staging", "eu", "us", "pci"];

impl Render for ComboboxPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut ComboboxPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/Combobox/preview.html` (`cu-stack`, max-width 360).
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
                        Combobox::new("ref-framework", &self.framework)
                            .label("Framework")
                            .items(FRAMEWORKS)
                            .default_value(["React"])
                            .placeholder("Filter…")
                            .clear_label("Clear selected item"),
                    )
                    .child(
                        Combobox::new("ref-tags", &self.tags)
                            .label("Tags")
                            .multiple(true)
                            .items(TAGS)
                            .default_value(["prod", "eu"])
                            .placeholder("Filter…"),
                    ),
            );

        let (state, items): (&Entity<InputState>, &[&str]) = if self.multiple {
            (&self.live_multi, &TAGS)
        } else {
            (&self.live_single, &FRAMEWORKS)
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div().max_w(rems(22.5)).child(
                    Combobox::new(SharedString::from(format!("live-{}", self.multiple)), state)
                        .label(if self.multiple { "Tags" } else { "Framework" })
                        .multiple(self.multiple)
                        .items(items.iter().copied())
                        .value(self.value.clone())
                        .placeholder("Filter…")
                        .helper_text(if self.multiple {
                            "Start typing to narrow the list."
                        } else {
                            "Start typing to jump to a match."
                        })
                        .size(FieldSize::ALL[self.size])
                        .invalid(self.invalid)
                        .invalid_text("Pick at least one.")
                        .warn(self.warn)
                        .warn_text("Legacy frameworks are not supported.")
                        .disabled(self.disabled)
                        .clear_label("Clear selected item")
                        .on_value_change({
                            let entity = entity.clone();
                            move |value, _, cx| {
                                let value = value.to_vec();
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
                    .child(SharedString::from(format!("value = {:?}", self.value))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices("cb-size", &FieldSize::ALL.map(|s| s.name()), self.size, {
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
                "cbx-multiple",
                "multiple",
                self.multiple,
                {
                    let entity = entity.clone();
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.multiple = on;
                            this.value = if on {
                                vec!["prod".into()]
                            } else {
                                vec!["Vue".into()]
                            };
                            cx.notify();
                        })
                    }
                },
            ))
            .child(controls::toggle(
                "cbx-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "cbx-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "cbx-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ));

        Body::new()
            .playground(live)
            .hint("type to find, ↑/↓, Enter, Escape")
            .config(config)
            .example(reference)
    }
}
