//! T3 · Shell page chrome pages: WorkspacePageHeader (with
//! WorkspaceBreadcrumb), RightPanelTabs, PanelLayoutControls.
use super::{label_text, stack};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::shell::{
    Crumb, LayoutPanel, PanelLayoutControls, RightPanelTabs, SurfaceKind, SurfaceTab,
    WorkspaceBreadcrumb, WorkspacePageHeader,
};

fn status_line(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

// ── WorkspacePageHeader ─────────────────────────────────────────────────

struct HeaderPage {
    crumbs: bool,
    description: bool,
    actions: bool,
    action: SharedString,
}

/// Build the WorkspacePageHeader page.
pub fn build_page_header(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| HeaderPage {
        crumbs: true,
        description: true,
        actions: true,
        action: "none".into(),
    })
    .into()
}

fn reference_header(id: &str) -> WorkspacePageHeader {
    WorkspacePageHeader::new(SharedString::from(id.to_string()), "Pull requests")
        .crumbs([
            Crumb::new("carbon-ui").on_click(|_, _| {}),
            Crumb::new("Pull requests"),
        ])
        .description("Open changes across all repositories in this project.")
        .action(
            Button::new(SharedString::from(format!("{id}-new")), "New pull request")
                .size(ButtonSize::Md)
                .icon(CarbonIcon::Add),
        )
}

/// Overview specimen.
pub fn header_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(260.))
        .child(
            WorkspacePageHeader::new("thumb-head", "Pull requests").crumbs([
                Crumb::new("carbon-ui").on_click(|_, _| {}),
                Crumb::new("Pull requests"),
            ]),
        )
        .into_any_element()
}

impl Render for HeaderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let record = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.action = what.into();
                    cx.notify();
                })
            }
        };
        let mut header = WorkspacePageHeader::new("live-head", "Pull requests");
        if self.crumbs {
            header = header.crumbs([
                Crumb::new("carbon-ui").on_click(record("carbon-ui link")),
                Crumb::new("Pull requests"),
            ]);
        }
        if self.description {
            header = header.description("Open changes across all repositories in this project.");
        }
        if self.actions {
            let new = record("New pull request");
            header = header.action(
                Button::new("live-head-new", "New pull request")
                    .size(ButtonSize::Md)
                    .icon(CarbonIcon::Add)
                    .on_click(move |_, w, cx| new(w, cx)),
            );
        }
        let live = stack()
            .w_full()
            .gap(px(16.))
            .child(header)
            .child(status_line(format!("Last action: {}", self.action)));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "head-crumbs",
                "crumbs",
                self.crumbs,
                controls::setter(&entity, |this: &mut HeaderPage, on| this.crumbs = on),
            ))
            .child(controls::toggle(
                "head-desc",
                "description",
                self.description,
                controls::setter(&entity, |this: &mut HeaderPage, on| this.description = on),
            ))
            .child(controls::toggle(
                "head-actions",
                "actions",
                self.actions,
                controls::setter(&entity, |this: &mut HeaderPage, on| this.actions = on),
            ));
        let crumbs = WorkspaceBreadcrumb::new(
            "crumbs-alone",
            [
                Crumb::new("carbon-ui")
                    .icon(CarbonIcon::Folder)
                    .on_click(|_, _| {}),
                Crumb::new("Settings").on_click(|_, _| {}),
                Crumb::new("Environments"),
            ],
        );
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().child(reference_header("ref-head")),
            ))
            .example(controls::block("WorkspaceBreadcrumb on its own", crumbs))
    }
}

// ── RightPanelTabs ──────────────────────────────────────────────────────

/// The preview.html tabs: Diff, localhost, Terminal, Files, Pull request,
/// Device.
fn reference_tabs() -> Vec<SurfaceTab> {
    vec![
        SurfaceTab::new("diff", "Diff", SurfaceKind::Diff),
        SurfaceTab::new("b", "localhost:5173", SurfaceKind::Browser).closable(true),
        SurfaceTab::new("t", "Terminal", SurfaceKind::Terminal).closable(true),
        SurfaceTab::new("f", "Files", SurfaceKind::Files),
        SurfaceTab::new("pr", "Pull request", SurfaceKind::Pr)
            .disabled_reason("No pull request on this branch yet."),
        SurfaceTab::new("dev", "Device", SurfaceKind::Device)
            .disabled_reason("Only available in the desktop app."),
    ]
}

/// The playground's tabs: the reference set plus u0's Details surface
/// after Files.
fn live_tabs() -> Vec<SurfaceTab> {
    let mut tabs = reference_tabs();
    tabs.insert(4, SurfaceTab::new("info", "Details", SurfaceKind::Details));
    tabs
}

const MAX_VISIBLE: [&str; 4] = ["all", "4", "3", "2"];

struct TabsPage {
    tabs: Vec<SurfaceTab>,
    value: SharedString,
    max: usize,
    add: bool,
    centred: bool,
    action: SharedString,
}

/// Build the RightPanelTabs page.
pub fn build_right_panel_tabs(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TabsPage {
        tabs: live_tabs(),
        value: "diff".into(),
        max: 0,
        add: true,
        centred: false,
        action: "none".into(),
    })
    .into()
}

/// Overview specimen.
pub fn tabs_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(260.))
        .child(RightPanelTabs::new(
            "thumb-tabs",
            [
                SurfaceTab::new("diff", "Diff", SurfaceKind::Diff),
                SurfaceTab::new("b", "Browser", SurfaceKind::Browser).closable(true),
            ],
        ))
        .into_any_element()
}

impl Render for TabsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut tabs = RightPanelTabs::new("live-tabs", self.tabs.clone())
            .value(self.value.clone())
            .on_value_change({
                let entity = entity.clone();
                move |v, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.action = format!("Selected {v}").into();
                        this.value = v;
                        cx.notify();
                    })
                }
            })
            .on_close({
                let entity = entity.clone();
                move |v, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.tabs.retain(|t| t.id != v);
                        if this.value == v {
                            this.value =
                                this.tabs.first().map(|t| t.id.clone()).unwrap_or_default();
                        }
                        this.action = format!("Closed {v}").into();
                        cx.notify();
                    })
                }
            });
        tabs = tabs.center_controls(self.centred);
        if self.max > 0 {
            tabs = tabs.max_visible(MAX_VISIBLE[self.max].parse().unwrap_or(4));
        }
        if self.add {
            let entity = entity.clone();
            tabs = tabs.on_add(move |_, cx| {
                entity.update(cx, |this, cx| {
                    let n = this.tabs.len();
                    this.tabs.push(
                        SurfaceTab::new(
                            SharedString::from(format!("chat-{n}")),
                            "Side chat",
                            SurfaceKind::Chat,
                        )
                        .closable(true),
                    );
                    this.action = "Open surface".into();
                    cx.notify();
                })
            });
        }
        let live = stack()
            .w_full()
            .gap(px(16.))
            .child(tabs)
            .child(status_line(format!("Last action: {}", self.action)));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "maxVisible",
                controls::choices(
                    "tabs-max",
                    &MAX_VISIBLE,
                    self.max,
                    controls::setter(&entity, |this: &mut TabsPage, i| this.max = i),
                ),
            ))
            .child(controls::toggle(
                "tabs-add",
                "onAdd",
                self.add,
                controls::setter(&entity, |this: &mut TabsPage, on| this.add = on),
            ))
            .child(controls::toggle(
                "tabs-centred",
                "center_controls",
                self.centred,
                controls::setter(&entity, |this: &mut TabsPage, on| this.centred = on),
            ))
            .child(
                Button::new("tabs-reset", "Reset tabs")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.tabs = live_tabs();
                                this.value = "diff".into();
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .hint("Hover a disabled tab for its reason; close, add and overflow are live")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .child(RightPanelTabs::new("ref-tabs", reference_tabs()).on_add(|_, _| {})),
            ))
            .example(controls::block(
                "u0's right column (Main board, .rp-tabs): \"+\" ghost sm, centred, 4px from the end",
                div().w(px(400.)).child(
                    RightPanelTabs::new(
                        "rp-tabs",
                        [
                            SurfaceTab::new("diff", "Diff", SurfaceKind::Diff),
                            SurfaceTab::new("info", "Details", SurfaceKind::Details),
                        ],
                    )
                    .center_controls(true)
                    .labels("Open another tab (⌘T)", "Close {}", "{} more tabs")
                    .on_add(|_, _| {}),
                ),
            ))
    }
}

// ── PanelLayoutControls ─────────────────────────────────────────────────

struct LayoutPage {
    sidebar: bool,
    terminal: bool,
    right: bool,
}

/// Build the PanelLayoutControls page.
pub fn build_layout_controls(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| LayoutPage {
        sidebar: true,
        terminal: false,
        right: true,
    })
    .into()
}

/// Overview specimen.
pub fn layout_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    PanelLayoutControls::new("thumb-layout")
        .sidebar(true)
        .right_panel(true)
        .into_any_element()
}

impl Render for LayoutPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = {
            let entity = entity.clone();
            move |panel: LayoutPanel, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    match panel {
                        LayoutPanel::Sidebar => this.sidebar = !this.sidebar,
                        LayoutPanel::Terminal => this.terminal = !this.terminal,
                        LayoutPanel::Right => this.right = !this.right,
                    }
                    cx.notify();
                })
            }
        };
        let on = |b: bool| if b { "shown" } else { "hidden" };
        let live = stack()
            .gap(px(16.))
            .child(
                PanelLayoutControls::new("live-layout")
                    .sidebar(self.sidebar)
                    .terminal(self.terminal)
                    .right_panel(self.right)
                    .on_toggle(toggle),
            )
            .child(label_text(
                format!(
                    "sidebar {} · terminal {} · right panel {}",
                    on(self.sidebar),
                    on(self.terminal),
                    on(self.right)
                ),
                TypeStyle::Label01,
            ));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "layout-sidebar",
                "sidebar",
                self.sidebar,
                controls::setter(&entity, |this: &mut LayoutPage, v| this.sidebar = v),
            ))
            .child(controls::toggle(
                "layout-terminal",
                "terminal",
                self.terminal,
                controls::setter(&entity, |this: &mut LayoutPage, v| this.terminal = v),
            ))
            .child(controls::toggle(
                "layout-right",
                "rightPanel",
                self.right,
                controls::setter(&entity, |this: &mut LayoutPage, v| this.right = v),
            ));
        Body::new()
            .playground(live)
            .hint("Click a toggle; hover for its shortcut")
            .config(config)
            .example(controls::block(
                "Reference preview",
                PanelLayoutControls::new("ref-layout")
                    .sidebar(true)
                    .terminal(false)
                    .right_panel(true),
            ))
    }
}
