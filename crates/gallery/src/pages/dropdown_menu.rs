//! Overlays › DropdownMenu: the reference preview (a MenuButton and an open
//! overflow menu), and a configurable live menu with every item type.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::{DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DropdownMenuPage {
        kind: 0,
        size: 2,
        labeled: true,
        icons: false,
        disabled_item: true,
        last: "—".into(),
    })
    .into()
}

struct DropdownMenuPage {
    kind: usize,
    size: usize,
    labeled: bool,
    icons: bool,
    disabled_item: bool,
    last: SharedString,
}

/// The reference MenuButton items.
fn actions() -> Vec<MenuItem> {
    vec![
        MenuItem::item("Restart"),
        MenuItem::item("Resize").shortcut("⌘R"),
        MenuItem::item("Move to")
            .submenu([MenuItem::item("Production"), MenuItem::item("Staging")]),
        MenuItem::divider(),
        MenuItem::item("Delete").danger(true),
    ]
}

/// The reference overflow items.
fn view_options() -> Vec<MenuItem> {
    vec![
        MenuItem::checkbox("Show archived").default_selected(true),
        MenuItem::divider(),
        MenuItem::radio_group("Sort", ["Name", "Date"]).default_selected_item(1),
    ]
}

impl Render for DropdownMenuPage {
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
        let toggle = |f: fn(&mut DropdownMenuPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/DropdownMenu/preview.html` (`cu-row`, align flex-start).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(tokens::SPACING_05)
                    .pb(rems(9.))
                    .child(
                        DropdownMenu::new("ref-actions")
                            .label("Actions")
                            .kind(DropdownMenuKind::Primary)
                            .size(DropdownMenuSize::Md)
                            .items(actions()),
                    )
                    .child(
                        DropdownMenu::new("ref-overflow")
                            .default_open(true)
                            .items(view_options()),
                    ),
            );

        let icon = |item: MenuItem, icon: CarbonIcon| {
            if self.icons { item.icon(icon) } else { item }
        };
        let items = vec![
            icon(MenuItem::item("Restart"), CarbonIcon::Restart).on_select(log("Restart")),
            icon(MenuItem::item("Resize"), CarbonIcon::Maximize)
                .shortcut("⌘R")
                .on_select(log("Resize")),
            icon(MenuItem::item("Export"), CarbonIcon::Download)
                .disabled(self.disabled_item)
                .on_select(log("Export")),
            icon(MenuItem::item("Move to"), CarbonIcon::Folder).submenu([
                MenuItem::item("Production").on_select(log("Move to Production")),
                MenuItem::item("Staging").on_select(log("Move to Staging")),
                MenuItem::item("Archive").submenu([
                    MenuItem::item("2025").on_select(log("Archive 2025")),
                    MenuItem::item("2026").on_select(log("Archive 2026")),
                ]),
            ]),
            MenuItem::divider(),
            MenuItem::group(
                "View",
                [
                    MenuItem::checkbox("Show archived").default_selected(true),
                    MenuItem::radio_group("Sort", ["Name", "Date"]).default_selected_item(1),
                ],
            ),
            MenuItem::divider(),
            icon(MenuItem::item("Delete"), CarbonIcon::TrashCan)
                .danger(true)
                .on_select(log("Delete")),
        ];
        let mut live_menu =
            DropdownMenu::new(SharedString::from(format!("live-menu-{}", self.labeled)))
                .kind(DropdownMenuKind::ALL[self.kind])
                .size(DropdownMenuSize::ALL[self.size])
                .items(items);
        if self.labeled {
            live_menu = live_menu.label("Actions");
        }

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section(
                "Live: click or Enter; ↑↓ Home End, → submenu, ← back, Esc, type to jump",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(tokens::SPACING_05)
                    .p(tokens::SPACING_05)
                    .pb(rems(18.))
                    .bg(color(ColorRole::Layer01))
                    .child(live_menu)
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
            .child(controls::toggle(
                "dm-labeled",
                "label (MenuButton) / none (overflow)",
                self.labeled,
                toggle(|this, on| this.labeled = on),
            ))
            .child(controls::field(
                "kind",
                controls::choices(
                    "dm-kind",
                    &DropdownMenuKind::ALL.map(|k| k.name()),
                    self.kind,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.kind = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "dm-size",
                    &DropdownMenuSize::ALL.map(|s| s.name()),
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
                "dm-icons",
                "item icons",
                self.icons,
                toggle(|this, on| this.icons = on),
            ))
            .child(controls::toggle(
                "dm-disabled",
                "disabled item",
                self.disabled_item,
                toggle(|this, on| this.disabled_item = on),
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
