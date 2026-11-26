//! Navigation › Tabs: the reference preview (line and contained) and a
//! configurable live set.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::{Tabs, TabsActivation, TabsTrigger, TabsVariant};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TabsPage {
        variant: 0,
        activation: 0,
        full_width: false,
        disabled: true,
        icons: false,
        value: "ov".into(),
    })
    .into()
}

struct TabsPage {
    variant: usize,
    activation: usize,
    full_width: bool,
    disabled: bool,
    icons: bool,
    value: SharedString,
}

impl Render for TabsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut TabsPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/Tabs/preview.html` (`.cu-stack { gap: var(--spacing-06) }`).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::section("Reference preview"))
            .child(
                Tabs::new("ref-tabs")
                    .default_value("ov")
                    .trigger(TabsTrigger::new("ov", "Overview"))
                    .trigger(TabsTrigger::new("met", "Metrics"))
                    .trigger(TabsTrigger::new("log", "Logs"))
                    .trigger(TabsTrigger::new("set", "Settings").disabled(true))
                    .content("ov", "3 of 3 replicas ready.")
                    .content("met", "CPU 34%")
                    .content("log", "No errors."),
            )
            .child(
                Tabs::new("ref-contained")
                    .default_value("a")
                    .variant(TabsVariant::Contained)
                    .trigger(TabsTrigger::new("a", "Contained"))
                    .trigger(TabsTrigger::new("b", "Second"))
                    .content("a", "Contained tabs.")
                    .content("b", "Second panel."),
            );

        let icon = |trigger: TabsTrigger, icon: CarbonIcon| {
            if self.icons {
                trigger.icon(icon)
            } else {
                trigger
            }
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section(
                "Live: click, or Tab into the list then ← → Home End (and Enter when manual)",
            ))
            .child(
                Tabs::new("live-tabs")
                    .variant(TabsVariant::ALL[self.variant])
                    .activation(TabsActivation::ALL[self.activation])
                    .full_width(self.full_width)
                    .value(self.value.clone())
                    .on_value_change({
                        let entity = entity.clone();
                        move |value, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.value = value;
                                cx.notify();
                            })
                        }
                    })
                    .trigger(icon(
                        TabsTrigger::new("ov", "Overview"),
                        CarbonIcon::Dashboard,
                    ))
                    .trigger(icon(
                        TabsTrigger::new("met", "Metrics"),
                        CarbonIcon::ChartLine,
                    ))
                    .trigger(icon(TabsTrigger::new("log", "Logs"), CarbonIcon::Document))
                    .trigger(
                        icon(TabsTrigger::new("set", "Settings"), CarbonIcon::Settings)
                            .disabled(self.disabled),
                    )
                    .content("ov", "3 of 3 replicas ready.")
                    .content("met", "CPU 34%")
                    .content("log", "No errors.")
                    .content("set", "Settings panel."),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("value: {}", self.value)),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "tabs-variant",
                    &TabsVariant::ALL.map(|v| v.name()),
                    self.variant,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.variant = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::field(
                "activation",
                controls::choices(
                    "tabs-activation",
                    &TabsActivation::ALL.map(|v| v.name()),
                    self.activation,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.activation = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "tabs-full",
                "fullWidth (contained)",
                self.full_width,
                toggle(|this, on| this.full_width = on),
            ))
            .child(controls::toggle(
                "tabs-disabled",
                "disabled tab",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "tabs-icons",
                "icons",
                self.icons,
                toggle(|this, on| this.icons = on),
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
