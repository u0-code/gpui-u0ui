//! T3 · Mobile › MobileList: the reference Settings screen in a phone frame
//! (Workspace rows, a working toggle, a Default model choice whose check
//! moves, Sign out) and the same list in the shared phone chrome.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, InteractiveElement as _, IntoElement,
    ParentElement, Render, StatefulInteractiveElement as _, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::mobile::{Leading, ListRow, MobileAppBar, MobileListSection, PhoneFrame, RowKind};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { model: 0 }).into()
}

struct Page {
    model: usize,
}

fn sections(
    prefix: &'static str,
    model: usize,
    phone: bool,
    entity: Option<gpui_kit::Entity<Page>>,
) -> Vec<MobileListSection> {
    let choice = |i: usize, title: &'static str| {
        let row = ListRow::new((prefix, i), title)
            .kind(RowKind::Choice)
            .checked(model == i);
        match entity.clone() {
            Some(entity) => row.on_press(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.model = i;
                    cx.notify();
                })
            }),
            None => row,
        }
    };
    vec![
        MobileListSection::new("Workspace")
            .phone(phone)
            .row(
                ListRow::new((prefix, 10usize), "Environments")
                    .icon(CarbonIcon::Laptop)
                    .value("3"),
            )
            .row(
                ListRow::new((prefix, 11usize), "Providers")
                    .icon(CarbonIcon::Bot)
                    .value("Claude, Codex"),
            )
            .row(
                ListRow::new((prefix, 12usize), "Notify when a thread needs you")
                    .icon(CarbonIcon::Notification)
                    .kind(RowKind::Toggle)
                    .default_on(true),
            ),
        MobileListSection::new("Default model")
            .phone(phone)
            .footer("Used for new threads started from this phone.")
            .row(choice(0, "Opus"))
            .row(choice(1, "Sonnet")),
        MobileListSection::new("").phone(phone).row(
            ListRow::new((prefix, 20usize), "Sign out")
                .kind(RowKind::Action)
                .destructive(true),
        ),
    ]
}

fn screen(
    prefix: &'static str,
    model: usize,
    phone: bool,
    entity: Option<gpui_kit::Entity<Page>>,
) -> PhoneFrame {
    PhoneFrame::new()
        .height(520.)
        .child(
            MobileAppBar::new((prefix, 99usize), "Settings")
                .large(true)
                .leading(Leading::None)
                .phone(phone),
        )
        .child(
            div()
                .id((prefix, 98usize))
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .children(sections(prefix, model, phone, entity)),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .children(sections("ml-thumb", 0, false, None).into_iter().take(1))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let config = controls::field(
            "Default model",
            controls::choices(
                "ml-model",
                &["Opus", "Sonnet"],
                self.model,
                controls::setter(&entity, |this: &mut Page, i| this.model = i),
            ),
        );
        Body::new()
            .playground(screen("ml-live", self.model, false, Some(entity.clone())))
            .hint("Rows press, the toggle switches, a model choice moves the check")
            .config(config)
            .example(controls::block(
                "Reference preview",
                screen("ml-ref", 0, false, None),
            ))
            .example(controls::block(
                "Phone chrome ([data-mb])",
                screen("ml-mb", self.model, true, Some(entity)),
            ))
    }
}
