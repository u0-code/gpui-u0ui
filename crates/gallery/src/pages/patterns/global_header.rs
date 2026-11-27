//! Carbon · Patterns › PatternGlobalHeader: the UI shell's header (product
//! name, product links, a submenu, tool icons with a notification count)
//! over a side nav and the main content
//! (`components/PatternGlobalHeader/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::{Align, NavItem, NavigationMenu, SideNavItem, Sidebar, SidebarProvider};
use u0ui::patterns::{BadgeIndicator, Badged};
use u0ui::prelude::*;
use u0ui::typography::paragraph;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| GlobalHeaderPage {
        product: "Threads".into(),
        page: "Overview".into(),
        side_nav: true,
        badge: 1,
    })
    .into()
}

struct GlobalHeaderPage {
    /// The current product link in the header.
    product: SharedString,
    /// The current side-nav link (the content's title).
    page: SharedString,
    side_nav: bool,
    /// Index into [`BADGES`].
    badge: usize,
}

const STRUCTURE: [&str; 2] = ["header only", "header + side nav"];
const BADGES: [&str; 3] = ["none", "4", "1200"];
const BADGE_COUNTS: [Option<u32>; 3] = [None, Some(4), Some(1200)];

/// A header tool icon: `IconButton kind="ghost" size="lg"` with a 20px
/// icon (`h(I.Search, {size: 20})`). A plain `IconButton`, not
/// `HeaderGlobalAction`, so it follows the density (32px compact).
fn tool(id: SharedString, label: &'static str, icon: CarbonIcon, align: Align) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Lg)
        .icon(icon)
        .icon_size(IconSize::S20)
        .icon_only(true)
        .tooltip_align(align)
}

/// The three tool icons, the notification count on the second.
fn tools(prefix: &'static str, count: Option<u32>) -> [AnyElement; 3] {
    let id = |s: &str| SharedString::from(format!("{prefix}-{s}"));
    [
        tool(id("search"), "Search", CarbonIcon::Search, Align::Bottom).into_any_element(),
        Badged::new(
            tool(
                id("notifications"),
                "Notifications",
                CarbonIcon::Notification,
                Align::Bottom,
            ),
            count.map(BadgeIndicator::Count),
        )
        .into_any_element(),
        tool(
            id("apps"),
            "Switch apps",
            CarbonIcon::Grid,
            Align::BottomRight,
        )
        .into_any_element(),
    ]
}

/// The main content, 16px padding: an `h3` at 20px 400 (Carbon's `h3`
/// line height, 1.28572) and a `g-dim` `<p>` at 14px with the global `p`
/// rule's 1.5 line height, no margins between them.
fn content(title: SharedString) -> impl IntoElement {
    div()
        .p(tokens::SPACING_05)
        .flex()
        .flex_col()
        .child(
            div()
                .text_size(rems(1.25))
                .line_height(rems(1.25 * 1.28572))
                .text_color(color(ColorRole::TextPrimary))
                .child(title),
        )
        .child(
            paragraph()
                .text_size(rems(0.875))
                .line_height(rems(0.875 * 1.5))
                .text_color(color(ColorRole::TextSecondary))
                .child("Main content starts below the header and right of the side nav."),
        )
}

/// `style={{margin:-16, height:400, display:'flex', flexDirection:'column'}}`.
fn frame() -> gpui_kit::Div {
    div()
        .h(rems(400. / 16.))
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .overflow_hidden()
}

fn shell(
    prefix: &'static str,
    header: NavigationMenu,
    sidebar: Option<Sidebar>,
    title: SharedString,
) -> impl IntoElement {
    let body = div().flex_1().flex().min_h_0().min_w_0();
    let body = match sidebar {
        Some(sidebar) => body.child(
            // A bare `SidebarInset`: no trigger in the content.
            SidebarProvider::new(SharedString::from(format!("{prefix}-provider")))
                .show_trigger(false)
                .sidebar(sidebar)
                .child(content(title)),
        ),
        None => body.child(content(title)),
    };
    frame().child(header).child(body)
}

fn header(prefix: &'static str, count: Option<u32>, items: Vec<NavItem>) -> NavigationMenu {
    let mut header = NavigationMenu::new(SharedString::from(format!("{prefix}-header")))
        .brand_prefix("")
        .brand("[Product]");
    for item in items {
        header = header.item(item);
    }
    for action in tools(prefix, count) {
        header = header.action(action);
    }
    header
}

/// `components/PatternGlobalHeader/preview.html`.
pub fn reference() -> impl IntoElement {
    let items = vec![
        NavItem::link("Threads").active(true),
        NavItem::link("Workflows"),
        NavItem::menu(
            "Manage",
            [
                NavItem::link("Servers"),
                NavItem::link("Devices"),
                NavItem::link("Billing"),
            ],
        ),
    ];
    let sidebar = Sidebar::new("gh-ref-sidebar")
        .item(
            SideNavItem::link("Overview")
                .icon(CarbonIcon::Dashboard)
                .active(true),
        )
        .item(SideNavItem::link("Threads").icon(CarbonIcon::Chat))
        .item(
            SideNavItem::menu(
                "Servers",
                [
                    SideNavItem::link("MacBook Pro"),
                    SideNavItem::link("build-box"),
                ],
            )
            .icon(CarbonIcon::Laptop)
            .expanded(true),
        )
        .item(SideNavItem::divider())
        .item(SideNavItem::link("Settings").icon(CarbonIcon::Settings));
    shell(
        "gh-ref",
        header("gh-ref", Some(4), items),
        Some(sidebar),
        "Overview".into(),
    )
}

/// The overview tile: the header's tool icons with the count.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .bg(color(ColorRole::Background))
        .border_b_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .children(tools("gh-thumb", Some(4)))
        .into_any_element()
}

impl GlobalHeaderPage {
    fn nav_link(&self, entity: &Entity<Self>, label: &'static str) -> NavItem {
        let entity = entity.clone();
        NavItem::link(label)
            .active(self.product == label)
            .on_click(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.product = label.into();
                    cx.notify();
                })
            })
    }

    fn side_link(&self, entity: &Entity<Self>, label: &'static str) -> SideNavItem {
        let entity = entity.clone();
        SideNavItem::link(label)
            .active(self.page == label)
            .on_click(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.page = label.into();
                    cx.notify();
                })
            })
    }
}

impl Render for GlobalHeaderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let items = vec![
            self.nav_link(&entity, "Threads"),
            self.nav_link(&entity, "Workflows"),
            NavItem::menu(
                "Manage",
                [
                    self.nav_link(&entity, "Servers"),
                    self.nav_link(&entity, "Devices"),
                    self.nav_link(&entity, "Billing"),
                ],
            ),
        ];
        let sidebar = self.side_nav.then(|| {
            Sidebar::new("gh-live-sidebar")
                .item(
                    self.side_link(&entity, "Overview")
                        .icon(CarbonIcon::Dashboard),
                )
                .item(self.side_link(&entity, "Threads").icon(CarbonIcon::Chat))
                .item(
                    SideNavItem::menu(
                        "Servers",
                        [
                            self.side_link(&entity, "MacBook Pro"),
                            self.side_link(&entity, "build-box"),
                        ],
                    )
                    .icon(CarbonIcon::Laptop)
                    .expanded(true),
                )
                .item(SideNavItem::divider())
                .item(
                    self.side_link(&entity, "Settings")
                        .icon(CarbonIcon::Settings),
                )
        });
        let title = if self.side_nav {
            self.page.clone()
        } else {
            self.product.clone()
        };
        let live = shell(
            "gh-live",
            header("gh-live", BADGE_COUNTS[self.badge], items),
            sidebar,
            title,
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "structure",
                controls::choices(
                    "gh-structure",
                    &STRUCTURE,
                    usize::from(self.side_nav),
                    controls::setter(&entity, |this: &mut Self, i: usize| this.side_nav = i == 1),
                ),
            ))
            .child(controls::field(
                "notification count",
                controls::choices(
                    "gh-badge",
                    &BADGES,
                    self.badge,
                    controls::setter(&entity, |this: &mut Self, i| this.badge = i),
                ),
            ));

        Body::new()
            .playground(live)
            .hint("header links and side-nav links navigate; Manage opens its submenu")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
