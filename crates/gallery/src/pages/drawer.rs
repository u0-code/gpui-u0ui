//! Overlays › Drawer: the reference preview (a contained drawer, open) and a
//! live window-level drawer with every side.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::{Checkbox, Drawer, Sheet, SheetSide};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DrawerPage {
        reference_open: true,
        live_open: false,
        side: 3,
        running: true,
        stopped: false,
        close_button: true,
    })
    .into()
}

struct DrawerPage {
    reference_open: bool,
    live_open: bool,
    side: usize,
    running: bool,
    stopped: bool,
    close_button: bool,
}

impl Render for DrawerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let update = |f: fn(&mut DrawerPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let filters = |drawer: Sheet, prefix: &'static str| {
            drawer
                .title("Filter results")
                .footer(
                    Button::new((prefix, 0usize), "Reset")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Xl)
                        .full_width(true)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.running = false;
                                    this.stopped = false;
                                    cx.notify();
                                })
                            }
                        }),
                )
                .footer(
                    Button::new((prefix, 1usize), "Apply")
                        .size(ButtonSize::Xl)
                        .full_width(true),
                )
                .child(
                    Checkbox::new((prefix, 2usize))
                        .label("Running")
                        .checked(self.running)
                        .on_checked_change(update(|this, on| this.running = on)),
                )
                .child(
                    Checkbox::new((prefix, 3usize))
                        .label("Stopped")
                        .checked(self.stopped)
                        .on_checked_change(update(|this, on| this.stopped = on)),
                )
        };

        // `components/Drawer/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .relative()
                    .h(rems(330. / 16.))
                    .overflow_hidden()
                    .bg(color(ColorRole::Background))
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(
                        div().flex().p(tokens::SPACING_05).child(
                            Button::new("ref-drawer-reopen", "Open drawer")
                                .variant(ButtonVariant::Tertiary)
                                .size(ButtonSize::Sm)
                                .on_click({
                                    let entity = entity.clone();
                                    move |_, _, cx| {
                                        entity.update(cx, |this, cx| {
                                            this.reference_open = true;
                                            cx.notify();
                                        })
                                    }
                                }),
                        ),
                    )
                    .child(filters(
                        Drawer::new("ref-drawer")
                            .contained(true)
                            .open(self.reference_open)
                            .on_open_change(update(|this, open| this.reference_open = open)),
                        "ref-drawer",
                    )),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live (window)"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .p(tokens::SPACING_05)
                    .bg(color(ColorRole::Layer01))
                    .child(filters(
                        Drawer::new("live-drawer")
                            .side(SheetSide::ALL[self.side])
                            .show_close_button(self.close_button)
                            .open(self.live_open)
                            .on_open_change(update(|this, open| this.live_open = open))
                            .trigger(
                                Button::new("live-drawer-open", "Open drawer")
                                    .variant(ButtonVariant::Tertiary),
                            ),
                        "live-drawer",
                    )),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "side",
                controls::choices(
                    "drawer-side",
                    &SheetSide::ALL.map(|s| s.name()),
                    self.side,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.side = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "drawer-close",
                "showCloseButton",
                self.close_button,
                update(|this, on| this.close_button = on),
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
