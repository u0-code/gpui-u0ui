//! Overlays › AlertDialog: the reference preview (open on arrival, as the
//! reference card is) and a configurable live alert.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::{AlertDialog, DialogSize};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AlertDialogPage {
        reference_open: true,
        live_open: false,
        danger: true,
        size: 1,
        last: "—".into(),
    })
    .into()
}

struct AlertDialogPage {
    reference_open: bool,
    live_open: bool,
    danger: bool,
    size: usize,
    last: SharedString,
}

impl Render for AlertDialogPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let update = |f: fn(&mut AlertDialogPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/AlertDialog/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                AlertDialog::new("ref-alert")
                    .open(self.reference_open)
                    .on_open_change(update(|this, open| this.reference_open = open))
                    .trigger(
                        Button::new("ref-alert-open", "Delete bucket")
                            .variant(ButtonVariant::Destructive),
                    )
                    .label("Storage")
                    .title("Delete logs-2026?")
                    .description("This permanently deletes 12,408 objects. This cannot be undone.")
                    .action_text("Delete bucket")
                    .cancel_text("Cancel")
                    .on_action({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.reference_open = false;
                                cx.notify();
                            })
                        }
                    }),
            );

        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(tokens::SPACING_05)
                .p(tokens::SPACING_05)
                .bg(color(ColorRole::Layer01))
                .child(
                    AlertDialog::new("live-alert")
                        .open(self.live_open)
                        .on_open_change(update(|this, open| {
                            this.live_open = open;
                            if !open {
                                this.last = "Cancelled".into();
                            }
                        }))
                        .trigger(
                            Button::new(
                                "live-alert-open",
                                if self.danger {
                                    "Delete project"
                                } else {
                                    "Publish"
                                },
                            )
                            .variant(if self.danger {
                                ButtonVariant::DangerTertiary
                            } else {
                                ButtonVariant::Tertiary
                            }),
                        )
                        .danger(self.danger)
                        .size(DialogSize::ALL[self.size])
                        .label("Projects")
                        .title(if self.danger {
                            "Delete carbon-ui?"
                        } else {
                            "Publish carbon-ui?"
                        })
                        .description(if self.danger {
                            "Its 3 environments and their logs are deleted too."
                        } else {
                            "Everyone in the organization can see published projects."
                        })
                        .action_text(if self.danger {
                            "Delete project"
                        } else {
                            "Publish project"
                        })
                        .cancel_text("Cancel")
                        .on_action({
                            let entity = entity.clone();
                            move |_, cx| {
                                entity.update(cx, |this, cx| {
                                    this.last = "Confirmed".into();
                                    this.live_open = false;
                                    cx.notify();
                                })
                            }
                        }),
                )
                .child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("Result: {}", self.last)),
                ),
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "alert-size",
                    &DialogSize::ALL.map(|s| s.name()),
                    self.size,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.size = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "alert-danger",
                "danger",
                self.danger,
                update(|this, on| this.danger = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}
