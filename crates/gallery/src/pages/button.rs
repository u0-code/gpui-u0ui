//! Actions › Button: the reference preview, a configurable live button, and
//! every kind × size and state, and buttons as the triggers of Kit popovers
//! (selected while open).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ButtonPage {
        variant: 0,
        size: 2,
        disabled: false,
        loading: false,
        with_icon: true,
        icon_only: false,
        clicks: 0,
    })
    .into()
}

struct ButtonPage {
    /// Index into [`ButtonVariant::ALL`].
    variant: usize,
    /// Index into [`ButtonSize::ALL`].
    size: usize,
    disabled: bool,
    loading: bool,
    with_icon: bool,
    icon_only: bool,
    clicks: usize,
}

fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
}

fn caption(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

impl Render for ButtonPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let set = |f: fn(&mut ButtonPage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };
        let toggle = |f: fn(&mut ButtonPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let variant = ButtonVariant::ALL[self.variant];
        let size = ButtonSize::ALL[self.size];

        // The reference preview (`components/Button/preview.html`).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                // `cu-row` with `gap: 0`: Carbon buttons sit flush.
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .child(Button::new("ref-create", "Create project").icon(CarbonIcon::Add))
                    .child(
                        Button::new("ref-secondary", "Secondary").variant(ButtonVariant::Secondary),
                    )
                    .child(Button::new("ref-tertiary", "Tertiary").variant(ButtonVariant::Outline))
                    .child(Button::new("ref-ghost", "Ghost").variant(ButtonVariant::Ghost))
                    .child(Button::new("ref-delete", "Delete").variant(ButtonVariant::Destructive)),
            )
            .child(
                row()
                    .child(Button::new("ref-small", "Small").size(ButtonSize::Sm))
                    .child(
                        Button::new("ref-medium", "Medium")
                            .size(ButtonSize::Md)
                            .variant(ButtonVariant::Secondary),
                    )
                    .child(Button::new("ref-disabled", "Disabled").disabled(true))
                    .child(
                        Button::new("ref-add", "Add item")
                            .variant(ButtonVariant::Ghost)
                            .icon(CarbonIcon::Add)
                            .icon_only(true),
                    )
                    .child(Button::new("ref-link", "Link button").variant(ButtonVariant::Link)),
            );

        // The configurable live button.
        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            row()
                .min_h(gpui_kit::rems(5.5))
                .p(tokens::SPACING_05)
                .bg(color(ColorRole::Layer01))
                .child(
                    Button::new("live", "Create project")
                        .variant(variant)
                        .size(size)
                        .disabled(self.disabled)
                        .loading(self.loading)
                        .icon_only(self.icon_only)
                        .when_icon(self.with_icon || self.icon_only)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.clicks += 1;
                                    cx.notify();
                                })
                            }
                        }),
                )
                .child(caption(format!("Activations: {}", self.clicks))),
        );

        // Every kind × size.
        let matrix = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Kinds × sizes"))
            .children(ButtonSize::ALL.iter().map(|size| {
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_03)
                    .child(caption(format!(
                        "{} ({}px Carbon, {}px now)",
                        size.name(),
                        size.carbon_height(),
                        size.height(density::current())
                    )))
                    .child(
                        row().children(
                            ButtonKind::BUTTONS
                                .iter()
                                .zip([
                                    ButtonVariant::Primary,
                                    ButtonVariant::Secondary,
                                    ButtonVariant::Tertiary,
                                    ButtonVariant::Ghost,
                                    ButtonVariant::Danger,
                                    ButtonVariant::DangerTertiary,
                                    ButtonVariant::DangerGhost,
                                ])
                                .map(|(kind, variant)| {
                                    Button::new(
                                        SharedString::from(format!("m-{}-{kind:?}", size.name())),
                                        variant.name(),
                                    )
                                    .variant(variant)
                                    .size(*size)
                                    .icon(CarbonIcon::Add)
                                }),
                        ),
                    )
            }));

        // States that can be shown statically.
        let states = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Disabled, loading, icon only"))
            .child(row().children(ButtonVariant::ALL.iter().map(|v| {
                Button::new(SharedString::from(format!("d-{}", v.name())), v.name())
                    .variant(*v)
                    .size(ButtonSize::Md)
                    .disabled(true)
            })))
            .child(row().children(ButtonVariant::ALL[..9].iter().map(|v| {
                Button::new(SharedString::from(format!("l-{}", v.name())), "Saving")
                    .variant(*v)
                    .size(ButtonSize::Md)
                    .loading(true)
            })))
            .child(
                row().children(
                    [
                        ButtonVariant::Default,
                        ButtonVariant::Primary,
                        ButtonVariant::Secondary,
                        ButtonVariant::Tertiary,
                        ButtonVariant::Danger,
                        ButtonVariant::DangerGhost,
                    ]
                    .iter()
                    .map(|v| {
                        Button::new(SharedString::from(format!("i-{}", v.name())), "Add item")
                            .variant(*v)
                            .icon(CarbonIcon::Add)
                            .icon_only(true)
                    }),
                ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "button-variant",
                    &ButtonVariant::ALL.map(|v| v.name()),
                    self.variant,
                    set(|this, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "button-size",
                    &ButtonSize::ALL.map(|s| s.name()),
                    self.size,
                    set(|this, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "button-icon",
                "icon",
                self.with_icon,
                toggle(|this, on| this.with_icon = on),
            ))
            .child(controls::toggle(
                "button-icon-only",
                "iconOnly",
                self.icon_only,
                toggle(|this, on| this.icon_only = on),
            ))
            .child(controls::toggle(
                "button-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "button-loading",
                "loading",
                self.loading,
                toggle(|this, on| this.loading = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
            .example(states)
            .example(matrix)
            .example(controls::block(
                "Popover triggers (Kit Popover; selected while open)",
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .child(popover_with(
                        "button-pop-ghost",
                        Button::new("button-pop-ghost-trigger", "Filter")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .icon(CarbonIcon::Filter),
                    ))
                    .child(popover_with(
                        "button-pop-icon",
                        Button::new("button-pop-icon-trigger", "Options")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .icon(CarbonIcon::OverflowMenuVertical)
                            .icon_only(true),
                    ))
                    .child(popover_with(
                        "button-pop-tertiary",
                        Button::new("button-pop-tertiary-trigger", "Choose")
                            .variant(ButtonVariant::Tertiary)
                            .size(ButtonSize::Sm),
                    )),
            ))
    }
}

/// A Kit popover opened from a u0ui button.
fn popover_with(id: &'static str, trigger: Button) -> impl IntoElement {
    gpui_kit::component::popover::Popover::new(id)
        .trigger(trigger)
        .content(|_, _, _| {
            div()
                .p(tokens::SPACING_05)
                .type_style(TypeStyle::BodyCompact01)
                .child("The trigger stays selected while this is open.")
        })
}

/// Add the demo icon when `on`.
trait WhenIcon {
    fn when_icon(self, on: bool) -> Self;
}

impl WhenIcon for Button {
    fn when_icon(self, on: bool) -> Self {
        if on { self.icon(CarbonIcon::Add) } else { self }
    }
}
