//! Overlays › ContextMenu: the reference preview (open at 220, 40) and a
//! configurable live area.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::{ContextMenu, MenuItem, MenuSize};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ContextMenuPage {
        size: 1,
        last: "—".into(),
    })
    .into()
}

struct ContextMenuPage {
    size: usize,
    last: SharedString,
}

/// The reference area: 200×180, dashed `border-strong-01` outline, centered
/// `text-secondary` text.
fn area(text: &'static str) -> impl IntoElement {
    div()
        .w(rems(200. / 16.))
        .h(rems(180. / 16.))
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_dashed()
        .border_color(color(ColorRole::BorderStrong01))
        .text_color(color(ColorRole::TextSecondary))
        .child(text)
}

/// The reference items.
fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Copy").shortcut("⌘C"),
        MenuItem::item("Rename"),
        MenuItem::divider(),
        MenuItem::checkbox("Pin to top").default_selected(true),
        MenuItem::divider(),
        MenuItem::item("Delete").danger(true),
    ]
}

impl Render for ContextMenuPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let log = |text: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = text.into();
                    cx.notify();
                })
            }
        };
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().flex().child(
                    ContextMenu::new("ref-context", area("Right-click here"))
                        .label("Context menu")
                        .default_open_at(220., 40.)
                        .items(items()),
                ),
            );

        let live_items = vec![
            MenuItem::item("Copy").shortcut("⌘C").on_select(log("Copy")),
            MenuItem::item("Rename").on_select(log("Rename")),
            MenuItem::item("Share").submenu([
                MenuItem::item("Email").on_select(log("Share by email")),
                MenuItem::item("Slack").on_select(log("Share to Slack")),
            ]),
            MenuItem::divider(),
            MenuItem::checkbox("Pin to top").default_selected(true),
            MenuItem::divider(),
            MenuItem::item("Delete")
                .danger(true)
                .on_select(log("Delete")),
        ];
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section(
                "Live: right-click (or Shift+F10 while focused) anywhere in the box",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(tokens::SPACING_05)
                    .child(
                        ContextMenu::new("live-context", area("Right-click here"))
                            .label("Context menu")
                            .size(MenuSize::ALL[self.size])
                            .items(live_items),
                    )
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("Last action: {}", self.last)),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "menu size",
                controls::choices("cm-size", &MenuSize::ALL.map(|s| s.name()), self.size, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.size = i;
                            cx.notify();
                        })
                    }
                }),
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
