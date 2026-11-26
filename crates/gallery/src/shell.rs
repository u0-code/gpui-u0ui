//! The Carbon UI shell: the G100 header (`top_bar`), the side nav (u0ui's
//! [`Sidebar`], Carbon `SideNav`) grouped like the reference, and the page
//! area: the home, the overview, or a page with its title, description and
//! Overview / Specs tabs. ⌘K opens the [`CommandDialog`] palette.
use crate::registry::{self, PAGES, Section};
use crate::{home, overview::Overview, top_bar};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, FocusHandle, InteractiveElement,
    IntoElement, KeyBinding, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled, Window, actions, div, point,
    prelude::FluentBuilder as _, px, rems,
};
use std::collections::HashMap;
use u0ui::components::{
    Breadcrumb, BreadcrumbItem, Command, CommandDialog, CommandGroup, CommandItem, SideNavItem,
    Sidebar, Tabs, TabsTrigger,
};
use u0ui::motion::ReducedMotion;
use u0ui::prelude::*;

actions!(gallery, [OpenSearch]);

/// Bind the shell's keys (⌘K / Ctrl+K: the palette).
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-k", OpenSearch, None),
        KeyBinding::new("ctrl-k", OpenSearch, None),
    ]);
}

/// Below Carbon's `lg` breakpoint (66rem) the side nav folds away behind the
/// header's menu button and opens over the content.
pub const NAV_BREAKPOINT: f32 = 1056.;

/// Where the gallery is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// The landing page.
    Home,
    /// The catalog of every page, optionally one section's.
    Overview(Option<Section>),
    /// A page of [`PAGES`].
    Page(usize),
}

impl Route {
    /// `home`, `overview` or a page id (`--page`).
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "home" => Some(Self::Home),
            "overview" => Some(Self::Overview(None)),
            "foundations" => Some(Self::Overview(Some(Section::Foundations))),
            "components" => Some(Self::Overview(Some(Section::Components))),
            "patterns" => Some(Self::Overview(Some(Section::Patterns))),
            id => registry::find(id).map(Self::Page),
        }
    }

    /// The route's id (the inverse of [`Route::from_id`]).
    pub fn id(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Overview(None) => "overview",
            Self::Overview(Some(Section::Foundations)) => "foundations",
            Self::Overview(Some(Section::Components)) => "components",
            Self::Overview(Some(Section::Patterns)) => "patterns",
            Self::Page(index) => PAGES[index].id,
        }
    }

    /// The header section to mark current.
    pub fn section(self) -> Option<Section> {
        match self {
            Self::Home => None,
            Self::Overview(section) => section,
            Self::Page(index) => Some(PAGES[index].section()),
        }
    }
}

/// A component page's tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    /// Playground and examples.
    Overview,
    /// Anatomy, measurements and state colors.
    Specs,
}

/// The gallery window's root view.
pub struct Gallery {
    pub(crate) route: Route,
    pub(crate) tab: Tab,
    views: HashMap<&'static str, AnyView>,
    pub(crate) overview: Entity<Overview>,
    pub(crate) motion: ReducedMotion,
    pub(crate) palette_open: bool,
    /// The side nav, opened over the content below [`NAV_BREAKPOINT`].
    pub(crate) nav_open: bool,
    focus: FocusHandle,
    scroll: ScrollHandle,
}

impl Gallery {
    /// Open on `route`.
    pub fn new(route: Route, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let gallery = cx.entity().downgrade();
        let overview = cx.new(|cx| Overview::new(gallery, window, cx));
        if let Route::Overview(section) = route {
            overview.update(cx, |o, _| o.section = section);
        }
        Self {
            route,
            tab: Tab::Overview,
            views: HashMap::new(),
            overview,
            motion: ReducedMotion::System,
            palette_open: false,
            nav_open: false,
            focus,
            scroll: ScrollHandle::new(),
        }
    }

    /// Go to `route`: scroll to the top, close the palette and the folded nav.
    pub fn navigate(&mut self, route: Route, cx: &mut Context<Self>) {
        if let Route::Overview(section) = route {
            self.overview.update(cx, |o, cx| {
                o.section = section;
                cx.notify();
            });
        }
        if self.route != route {
            self.tab = Tab::Overview;
            self.scroll.set_offset(point(px(0.), px(0.)));
        }
        self.route = route;
        self.palette_open = false;
        self.nav_open = false;
        cx.notify();
    }

    /// The page's live view, built the first time it is shown.
    pub(crate) fn page_view(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyView {
        let page = &PAGES[index];
        self.views
            .entry(page.id)
            .or_insert_with(|| (page.build)(window, cx))
            .clone()
    }

    fn side_nav(&self, cx: &mut Context<Self>) -> Sidebar {
        let entity = cx.entity();
        let go = move |route: Route| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| this.navigate(route, cx))
            }
        };
        let current_group = match self.route {
            Route::Page(index) => Some(PAGES[index].group),
            _ => None,
        };
        let mut items = vec![
            SideNavItem::link("Home")
                .active(self.route == Route::Home)
                .on_click(go(Route::Home)),
            SideNavItem::link("Overview")
                .active(matches!(self.route, Route::Overview(_)))
                .on_click(go(Route::Overview(None))),
            SideNavItem::divider(),
        ];
        for group in registry::groups() {
            let links = registry::pages_in(group).map(|(index, page)| {
                SideNavItem::link(page.title)
                    .active(self.route == Route::Page(index))
                    .on_click(go(Route::Page(index)))
            });
            // Component groups start open; the pattern libraries start closed
            // unless the gallery opened on one of their pages.
            let open =
                Section::of_group(group) != Section::Patterns || current_group == Some(group);
            items.push(SideNavItem::menu(group, links).expanded(open));
        }
        Sidebar::new("side-nav").label("Gallery").items(items)
    }

    /// The command palette: every page, then the display settings.
    fn palette(&self, cx: &mut Context<Self>) -> CommandDialog {
        let entity = cx.entity();
        let go = |route: Route| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| this.navigate(route, cx))
            }
        };
        let mut command = Command::new("palette-command")
            .placeholder("Search pages, themes and settings")
            .empty_text("No page matches.")
            .label("Search the gallery")
            .group(
                CommandGroup::new("Gallery")
                    .item(
                        CommandItem::new("Home")
                            .icon(Icon::named("home"))
                            .on_select(go(Route::Home)),
                    )
                    .item(
                        CommandItem::new("Overview")
                            .icon(Icon::named("grid_view"))
                            .on_select(go(Route::Overview(None))),
                    ),
            );
        for group in registry::groups() {
            let mut items = CommandGroup::new(group);
            for (index, page) in registry::pages_in(group) {
                items = items.item(
                    CommandItem::new(page.title)
                        .value(SharedString::from(format!(
                            "{} {} {}",
                            page.title, page.id, group
                        )))
                        .icon(Icon::named(registry::group_icon(group)))
                        .on_select(go(Route::Page(index))),
                );
            }
            command = command.group(items);
        }
        let mut settings = CommandGroup::new("Display");
        for theme in CarbonTheme::ALL {
            let entity = entity.clone();
            settings = settings.item(
                CommandItem::new(format!("Theme: {}", top_bar::theme_name(theme)))
                    .icon(Icon::named("contrast"))
                    .on_select(move |_, cx| {
                        theme::set_theme(cx, theme);
                        entity.update(cx, |this, cx| {
                            this.palette_open = false;
                            cx.notify();
                        });
                    }),
            );
        }
        for density in Density::ALL {
            let entity = entity.clone();
            settings = settings.item(
                CommandItem::new(format!("Density: {}", top_bar::density_name(density)))
                    .icon(Icon::named("density_medium"))
                    .on_select(move |_, cx| {
                        density::set_density(cx, density);
                        entity.update(cx, |this, cx| {
                            this.palette_open = false;
                            cx.notify();
                        });
                    }),
            );
        }
        command = command.group(settings);
        CommandDialog::new(
            "palette",
            self.palette_open,
            {
                let entity = entity.clone();
                move |open, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.palette_open = open;
                        cx.notify();
                    })
                }
            },
            command,
        )
    }

    /// A page: breadcrumb, title, description, tabs, then its body.
    fn page(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let page = &PAGES[index];
        let entity = cx.entity();
        let section = page.section();
        let to_section = |entity: Entity<Gallery>| {
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.navigate(Route::Overview(Some(section)), cx)
                })
            }
        };
        let mut crumbs = Breadcrumb::new("page-breadcrumb")
            .item(BreadcrumbItem::new(section.title()).on_click(to_section(entity.clone())));
        // A group named like its section (Foundations) is not repeated.
        if page.group != section.title() {
            crumbs =
                crumbs.item(BreadcrumbItem::new(page.group).on_click(to_section(entity.clone())));
        }
        let crumbs = crumbs
            .item(BreadcrumbItem::new(page.title).current(true))
            .no_trailing_slash(true);
        let tabs = page.specs.map(|_| {
            Tabs::new("page-tabs")
                .value(match self.tab {
                    Tab::Overview => "overview",
                    Tab::Specs => "specs",
                })
                .on_value_change({
                    let entity = entity.clone();
                    move |value, _, cx| {
                        let tab = if value.as_ref() == "specs" {
                            Tab::Specs
                        } else {
                            Tab::Overview
                        };
                        entity.update(cx, |this, cx| {
                            this.tab = tab;
                            cx.notify();
                        })
                    }
                })
                .trigger(TabsTrigger::new("overview", "Overview"))
                .trigger(TabsTrigger::new("specs", "Specs"))
                .label("Page view")
        });
        let body: AnyElement = match (self.tab, page.specs) {
            (Tab::Specs, Some(specs)) => specs(window, cx),
            _ => self.page_view(index, window, cx).into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_07)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_05)
                    .child(crumbs)
                    .child(
                        div()
                            .type_style(TypeStyle::Heading05)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(page.title),
                    )
                    .child(div().max_w(rems(40.)).type_style(TypeStyle::Body02).child(
                        crate::controls::prose(page.description, color(ColorRole::TextSecondary)),
                    )),
            )
            .children(tabs)
            .child(body)
            .into_any_element()
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let narrow = window.viewport_size().width < px(NAV_BREAKPOINT);
        let content: AnyElement = match self.route {
            Route::Home => home::render(cx.entity(), window, cx),
            Route::Overview(_) => self.overview.clone().into_any_element(),
            Route::Page(index) => self.page(index, window, cx),
        };
        let nav = (!narrow).then(|| self.side_nav(cx));
        let folded = (narrow && self.nav_open).then(|| self.side_nav(cx));
        let entity = cx.entity();
        let page_area = div()
            .id("page")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                div()
                    .w_full()
                    .max_w(rems(90.))
                    .px(tokens::SPACING_07)
                    .pt(tokens::SPACING_07)
                    .pb(tokens::SPACING_10)
                    .child(content),
            );
        div()
            .id("gallery")
            .key_context("Gallery")
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &OpenSearch, _, cx| {
                this.palette_open = true;
                cx.notify();
            }))
            .size_full()
            .flex()
            .flex_col()
            .bg(color(ColorRole::Background))
            .text_color(color(ColorRole::TextPrimary))
            .type_style(TypeStyle::BodyCompact01)
            .child(top_bar::render(self, narrow, cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .children(nav)
                    .child(page_area)
                    .when(narrow && self.nav_open, |this| {
                        // The folded nav opens over the content, with
                        // Carbon's `overlay` scrim behind it.
                        this.child(
                            div()
                                .id("nav-scrim")
                                .absolute()
                                .inset_0()
                                .bg(color(ColorRole::Overlay))
                                .on_click(move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.nav_open = false;
                                        cx.notify();
                                    })
                                }),
                        )
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .bottom_0()
                                .children(folded),
                        )
                    }),
            )
            .child(self.palette(cx))
    }
}
