//! Navigation › Sidebar: the reference preview (provider, side nav, inset
//! with the trigger) and a configurable live one whose links navigate.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::{SideNavItem, Sidebar, SidebarProvider};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SidebarPage {
        current: "Dashboard".into(),
        expanded: true,
        header: true,
        footer: false,
    })
    .into()
}

struct SidebarPage {
    current: SharedString,
    expanded: bool,
    header: bool,
    footer: bool,
}

fn frame() -> gpui_kit::Div {
    // `margin: -16; height: 400; display: flex`.
    div()
        .h(rems(400. / 16.))
        .flex()
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .overflow_hidden()
}

impl Render for SidebarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut SidebarPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        // `components/Sidebar/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                frame().child(
                    SidebarProvider::new("ref-provider")
                        .sidebar(
                            Sidebar::new("ref-sidebar")
                                .header("Cloud console")
                                .item(
                                    SideNavItem::link("Dashboard")
                                        .icon(CarbonIcon::Dashboard)
                                        .active(true),
                                )
                                .item(
                                    SideNavItem::menu(
                                        "Resources",
                                        [
                                            SideNavItem::link("Compute"),
                                            SideNavItem::link("Storage"),
                                            SideNavItem::link("Networking"),
                                        ],
                                    )
                                    .icon(CarbonIcon::Folder)
                                    .expanded(true),
                                )
                                .item(SideNavItem::divider())
                                .item(SideNavItem::link("Settings").icon(CarbonIcon::Settings)),
                        )
                        .child(div().mt(rems(0.5)).child("Page content")),
                ),
            );

        let link = |label: &'static str| {
            let entity = entity.clone();
            SideNavItem::link(label)
                .active(self.current == label)
                .on_click(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.current = label.into();
                        cx.notify();
                    })
                })
        };
        let mut sidebar = Sidebar::new("live-sidebar")
            .label("Side navigation")
            .expanded(self.expanded)
            .item(link("Dashboard").icon(CarbonIcon::Dashboard))
            .item(
                SideNavItem::menu(
                    "Resources",
                    [link("Compute"), link("Storage"), link("Networking")],
                )
                .icon(CarbonIcon::Folder)
                .expanded(true),
            )
            .item(
                SideNavItem::menu("Billing", [link("Invoices"), link("Usage")])
                    .icon(CarbonIcon::Analytics),
            )
            .item(SideNavItem::divider())
            .item(link("Settings").icon(CarbonIcon::Settings));
        if self.header {
            sidebar = sidebar.header("Cloud console");
        }
        if self.footer {
            sidebar = sidebar.footer(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child("v4.2 · us-south"),
            );
        }
        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            frame().child(
                SidebarProvider::new("live-provider")
                    .sidebar(sidebar)
                    .child(
                        div()
                            .mt(rems(0.5))
                            .type_style(TypeStyle::Heading03)
                            .child(self.current.clone()),
                    ),
            ),
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "sb-expanded",
                "expanded (off: collapsed out of its slot)",
                self.expanded,
                toggle(|this, on| this.expanded = on),
            ))
            .child(controls::toggle(
                "sb-header",
                "header",
                self.header,
                toggle(|this, on| this.header = on),
            ))
            .child(controls::toggle(
                "sb-footer",
                "footer",
                self.footer,
                toggle(|this, on| this.footer = on),
            ));

        Body::new()
            .playground(live)
            .hint("the trigger hides the nav; links navigate")
            .config(config)
            .example(reference)
    }
}
