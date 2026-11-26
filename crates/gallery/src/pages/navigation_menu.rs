//! Navigation › NavigationMenu: the reference header and a live one whose
//! links navigate.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::{NavItem, NavigationMenu};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| NavigationMenuPage {
        current: "Catalog".into(),
        with_actions: true,
    })
    .into()
}

struct NavigationMenuPage {
    current: SharedString,
    with_actions: bool,
}

fn search_action(id: &'static str) -> Button {
    // `HeaderGlobalAction`: a 3rem ghost icon button.
    Button::new(id, "Search")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Lg)
        .density(Density::Comfortable)
        .icon(CarbonIcon::Search)
        .icon_only(true)
}

impl Render for NavigationMenuPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/NavigationMenu/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().pb(rems(10.)).child(
                    NavigationMenu::new("ref-header")
                        .brand_prefix("IBM")
                        .brand("[Platform]")
                        .item(NavItem::link("Catalog").active(true))
                        .item(NavItem::link("Docs"))
                        .item(NavItem::menu(
                            "Manage",
                            [
                                NavItem::link("Access (IAM)"),
                                NavItem::link("Billing"),
                                NavItem::link("Support"),
                            ],
                        ))
                        .action(search_action("ref-search")),
                ),
            );

        let link = |label: &'static str| {
            let entity = entity.clone();
            NavItem::link(label)
                .active(self.current == label)
                .on_click(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.current = label.into();
                        cx.notify();
                    })
                })
        };
        let mut header = NavigationMenu::new("live-header")
            .brand_prefix("IBM")
            .brand("[Platform]")
            .item(link("Catalog"))
            .item(link("Docs"))
            .item(NavItem::menu(
                "Manage",
                [link("Access (IAM)"), link("Billing"), link("Support")],
            ));
        if self.with_actions {
            header = header.action(search_action("live-search"));
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(div().pb(rems(10.)).child(header))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("current: {}", self.current)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "nav-actions",
                "actions",
                self.with_actions,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.with_actions = on;
                        cx.notify();
                    })
                },
            ));
        Body::new()
            .playground(live)
            .hint("click links; Manage opens on click or ↓, then ↑↓ Enter Esc")
            .config(config)
            .example(reference)
    }
}
