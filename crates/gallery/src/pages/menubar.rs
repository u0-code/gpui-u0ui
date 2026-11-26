//! Navigation › Menubar: the reference preview (File open) and a live bar.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::{MenuItem, Menubar};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| MenubarPage { last: "—".into() }).into()
}

struct MenubarPage {
    last: SharedString,
}

type Log = std::rc::Rc<dyn Fn(&'static str) -> Box<dyn Fn(&mut Window, &mut App)>>;

/// The reference menus; `log` wires item activation in the live bar.
fn menus(bar: Menubar, log: Option<Log>) -> Menubar {
    let item = |label: &'static str| {
        let item = MenuItem::item(label);
        match &log {
            Some(log) => {
                let f = log(label);
                item.on_select(move |window, cx| f(window, cx))
            }
            None => item,
        }
    };
    bar.menu(
        "File",
        vec![
            item("New tab").shortcut("⌘T"),
            item("Open…").shortcut("⌘O"),
            MenuItem::divider(),
            MenuItem::item("Share").submenu([item("Email"), item("Slack")]),
            MenuItem::divider(),
            item("Delete").danger(true),
        ],
    )
    .menu(
        "View",
        vec![
            MenuItem::checkbox("Show grid").default_selected(true),
            MenuItem::checkbox("Show rulers"),
            MenuItem::divider(),
            MenuItem::radio_group("Density", ["Compact", "Normal"]).default_selected_item(1),
        ],
    )
    .menu("Help", vec![item("Documentation")])
}

impl Render for MenubarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let log: Log = std::rc::Rc::new(move |label: &'static str| {
            let entity = entity.clone();
            Box::new(move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = label.into();
                    cx.notify();
                })
            })
        });
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .max_w(rems(420. / 16.))
                    .pb(rems(14.))
                    .child(menus(Menubar::new("ref-menubar").default_open(0), None)),
            );
        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_03)
                .pb(rems(14.))
                .child(menus(Menubar::new("live-menubar"), Some(log)))
                .child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("Last action: {}", self.last)),
                ),
        );
        Body::new()
            .playground(live)
            .hint("click a menu, then point at the others or use ← →")
            .example(reference)
    }
}
