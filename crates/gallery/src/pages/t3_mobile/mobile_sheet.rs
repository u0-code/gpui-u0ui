//! T3 · Mobile › MobileSheet: the reference "New thread" sheet over the
//! overlay in a phone frame; Close dismisses it and a button reopens it.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::{Button, ButtonSize, ButtonVariant};
use u0ui::prelude::*;
use u0ui::t3::mobile::chrome::PHONE_SCREEN_RADIUS;
use u0ui::t3::mobile::{ListRow, MobileListSection, MobileSheet, PhoneFrame};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        open: true,
        intro: true,
        primary: true,
    })
    .into()
}

struct Page {
    open: bool,
    intro: bool,
    primary: bool,
}

fn rows(prefix: &'static str) -> MobileListSection {
    MobileListSection::new("")
        .raised(true)
        .row(
            ListRow::new((prefix, 0usize), "Run on")
                .icon(CarbonIcon::Laptop)
                .value("MacBook Pro"),
        )
        .row(
            ListRow::new((prefix, 1usize), "Branch")
                .icon(CarbonIcon::Branch)
                .value("main"),
        )
        .row(
            ListRow::new((prefix, 2usize), "Model")
                .icon(CarbonIcon::Bot)
                .value("Opus"),
        )
}

fn backdrop() -> gpui_kit::Div {
    // `flex: 1; background: var(--overlay); display: flex; flex-direction: column`.
    // It fills to the screen's bottom edge, so it follows the screen's
    // curve there (see `screen_bottom_radius`).
    div()
        .flex()
        .flex_col()
        .flex_1()
        .rounded_b(rems(PHONE_SCREEN_RADIUS / 16.))
        .bg(color(ColorRole::Overlay))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(MobileSheet::new("ms-thumb", "New thread").primary("Start"))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let sheet = MobileSheet::new("ms-live", "New thread")
            .when(self.primary, |s| s.primary("Start"))
            .when(self.intro, |s| s.intro("Choose where the agent runs."))
            .on_close({
                let entity = entity.clone();
                move |_: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.open = false;
                        cx.notify();
                    })
                }
            })
            .on_primary({
                let entity = entity.clone();
                move |_: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.open = false;
                        cx.notify();
                    })
                }
            })
            .child(rows("ms-live-rows"));
        let live = PhoneFrame::new().height(420.).child(
            backdrop()
                .when(!self.open, |d| {
                    d.items_center().justify_center().child(
                        Button::new("ms-reopen", "Open sheet")
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .on_click({
                                let entity = entity.clone();
                                move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.open = true;
                                        cx.notify();
                                    })
                                }
                            }),
                    )
                })
                .when(self.open, |d| d.child(sheet)),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "ms-intro",
                "intro",
                self.intro,
                controls::setter(&entity, |this: &mut Page, v| this.intro = v),
            ))
            .child(controls::toggle(
                "ms-primary",
                "primary",
                self.primary,
                controls::setter(&entity, |this: &mut Page, v| this.primary = v),
            ));
        Body::new()
            .playground(live)
            .hint("Close or Start dismisses the sheet")
            .config(config)
            .example(controls::block(
                "Reference preview",
                PhoneFrame::new().height(420.).child(
                    backdrop().child(
                        MobileSheet::new("ms-ref", "New thread")
                            .primary("Start")
                            .intro("Choose where the agent runs.")
                            .child(rows("ms-ref-rows")),
                    ),
                ),
            ))
    }
}
