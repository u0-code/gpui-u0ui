//! T3 · Shell › AppShell: the reference frame (sidebar with brand, a
//! project and three threads; a chat header; the body; a Diff/Browser
//! right panel; a terminal drawer) with every region switchable.
//!
//! The reference fills the header, body and terminal with `ChatHeader`,
//! `UserMessage`, `WorkingIndicator` and `TerminalDrawer`, which other T3
//! modules own; until they land, plain stand-ins with the same text hold
//! their place.
use super::stack;
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::review::DiffStat;
use u0ui::t3::shell::{
    AppShell, FolderCounts, ProjectFolderHeader, RightPanelTabs, SidebarBrandHeader, SurfaceKind,
    SurfaceTab, ThreadRow,
};

struct ShellPage {
    sidebar: bool,
    header: bool,
    right: bool,
    terminal: bool,
    active: usize,
}

/// Build the AppShell page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ShellPage {
        sidebar: true,
        header: true,
        right: true,
        terminal: true,
        active: 0,
    })
    .into()
}

const THREADS: [(&str, Status, &str); 3] = [
    ("Add dark theme tokens", Status::Working, ""),
    ("Fix flaky date picker test", Status::NeedsYou, "3m"),
    ("Write migration guide", Status::Done, "1h"),
];

fn text(s: &'static str, style: TypeStyle, role: ColorRole) -> gpui_kit::Div {
    div().type_style(style).text_color(color(role)).child(s)
}

/// Stand-in for `ChatHeader` (title, project · branch, status).
fn header_stand_in() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .h(px(48.))
        .px(px(16.))
        .border_b_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(text(
            "Add dark theme tokens",
            TypeStyle::HeadingCompact01,
            ColorRole::TextPrimary,
        ))
        .child(text(
            "carbon-ui · feat/dark-tokens",
            TypeStyle::Label01,
            ColorRole::TextSecondary,
        ))
        .child(u0ui::t3::StatusPill::new(Status::Working).id("shell-head-pill"))
}

/// Stand-in for `TerminalDrawer` (one `dev` tab and its lines).
fn terminal_stand_in() -> impl IntoElement {
    let mono = |s: &'static str| {
        div()
            .font_family("IBM Plex Mono")
            .text_size(px(12.))
            .text_color(color(ColorRole::TextPrimary))
            .child(s)
    };
    stack()
        .size_full()
        .bg(color(ColorRole::Layer01))
        .child(
            div()
                .flex()
                .items_center()
                .h(px(32.))
                .px(px(16.))
                .border_b_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .child(text("dev", TypeStyle::Label01, ColorRole::TextPrimary)),
        )
        .child(
            stack()
                .p(px(12.))
                .gap(px(2.))
                .child(mono("$ pnpm dev"))
                .child(mono("  VITE v6  ready in 412 ms"))
                .child(mono("  ➜  Local: http://localhost:5173/")),
        )
}

/// Stand-in for the body's `UserMessage` and `WorkingIndicator`.
fn body_stand_in() -> impl IntoElement {
    stack()
        .p(px(24.))
        .gap(px(16.))
        .child(
            div().flex().justify_end().child(
                div()
                    .min_w_0()
                    .px(px(16.))
                    .py(px(12.))
                    .bg(color(ColorRole::Layer01))
                    .child(text(
                        "Add the G90 theme and update the cover.",
                        TypeStyle::Body01,
                        ColorRole::TextPrimary,
                    )),
            ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(u0ui::t3::Spin::new("shell-body-spin"))
                .child(text(
                    "Working · 1m 14s",
                    TypeStyle::BodyCompact01,
                    ColorRole::TextSecondary,
                )),
        )
}

/// Region switches: sidebar, header, right panel, terminal, active thread.
type Flags = (bool, bool, bool, bool, usize);

fn shell(flags: Flags, prefix: &str, entity: Option<Entity<ShellPage>>) -> AnyElement {
    let id = |n: &str| super::eid(prefix, n);
    let (sidebar, header, right, terminal, active) = flags;
    let mut frame = AppShell::new(id("shell")).height(Some(500.));
    if sidebar {
        let rows = THREADS
            .iter()
            .enumerate()
            .map(|(i, (title, status, time))| {
                let mut row = ThreadRow::new(id(&format!("t{i}")), *title)
                    .status(*status)
                    .active(i == active);
                row = if *status == Status::Working {
                    row.elapsed(74)
                } else {
                    row.time(*time)
                };
                if let Some(e) = entity.clone() {
                    row = row.on_click(move |_, cx| {
                        e.update(cx, |this, cx| {
                            this.active = i;
                            cx.notify();
                        })
                    });
                }
                row
            });
        frame = frame.sidebar(
            stack()
                .child(SidebarBrandHeader::new(id("brand"), "Workbench"))
                .child(
                    ProjectFolderHeader::new(id("folder"), "carbon-ui").counts(FolderCounts {
                        working: 1,
                        needs_you: 1,
                        ..Default::default()
                    }),
                )
                .children(rows),
        );
    }
    if header {
        frame = frame.header(header_stand_in());
    }
    if right {
        frame = frame.right_panel(
            stack()
                .child(RightPanelTabs::new(
                    id("tabs"),
                    [
                        SurfaceTab::new("d", "Diff", SurfaceKind::Diff),
                        SurfaceTab::new("b", "Browser", SurfaceKind::Browser).closable(true),
                    ],
                ))
                .child(div().p(px(16.)).child(DiffStat::new(128, 14).bar(true))),
        );
    }
    if terminal {
        frame = frame.terminal(terminal_stand_in());
    }
    frame.child(body_stand_in()).into_any_element()
}

/// Overview specimen.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            AppShell::new("thumb-shell")
                .height(Some(150.))
                .sidebar(
                    stack()
                        .w(px(80.))
                        .child(ThreadRow::new("thumb-shell-t", "Tokens").status(Status::Working)),
                )
                .child(div().p(px(8.)).child(text(
                    "Add the G90 theme",
                    TypeStyle::Label01,
                    ColorRole::TextPrimary,
                ))),
        )
        .into_any_element()
}

impl Render for ShellPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let flags = (
            self.sidebar,
            self.header,
            self.right,
            self.terminal,
            self.active,
        );
        let live = shell(flags, "live", Some(entity.clone()));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "shell-sidebar",
                "sidebar",
                self.sidebar,
                controls::setter(&entity, |this: &mut ShellPage, on| this.sidebar = on),
            ))
            .child(controls::toggle(
                "shell-header",
                "header",
                self.header,
                controls::setter(&entity, |this: &mut ShellPage, on| this.header = on),
            ))
            .child(controls::toggle(
                "shell-right",
                "rightPanel",
                self.right,
                controls::setter(&entity, |this: &mut ShellPage, on| this.right = on),
            ))
            .child(controls::toggle(
                "shell-terminal",
                "terminal",
                self.terminal,
                controls::setter(&entity, |this: &mut ShellPage, on| this.terminal = on),
            ));
        Body::new()
            .playground(div().w_full().child(live))
            .hint("Click a thread to make it active")
            .stage_min_width(64.)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .child(shell((true, true, true, true, 0), "ref", None)),
            ))
    }
}
