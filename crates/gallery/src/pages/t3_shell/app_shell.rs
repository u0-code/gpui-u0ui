//! T3 · Shell › AppShell: the reference frame (sidebar with brand, a
//! project and three threads; a chat header; the body; a Diff/Browser
//! right panel; a terminal drawer) with every region switchable. The header,
//! body and terminal are the real `ChatHeader`, `UserMessage` +
//! `WorkingIndicator` and `TerminalDrawer`, as in
//! `components/AppShell/preview.html`.
//!
//! The shell has no breakpoints of its own (`.t3-shell*` in `src/t3.css`):
//! its 16rem sidebar, 20rem right panel and a readable main column need
//! [`NATURAL_WIDTH`]. Narrower stages draw it scaled down whole
//! ([`fit_width`]) rather than scrolling sideways or dropping regions.
use super::stack;
use crate::controls;
use crate::fit::fit_width;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::review::{ChatHeader, DiffStat};
use u0ui::t3::shell::{
    AppShell, FolderCounts, ProjectFolderHeader, RightPanelTabs, SidebarBrandHeader, SurfaceKind,
    SurfaceTab, ThreadRow,
};
use u0ui::t3::timeline::{UserMessage, WorkingIndicator, WorkingState};
use u0ui::t3::workspace::{TerminalDrawer, TerminalTab};

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

/// `h(C.ChatHeader,{title:'Add dark theme tokens',project:'carbon-ui',
/// branch:'feat/dark-tokens',status:'working'})`.
fn chat_header(id: gpui_kit::ElementId) -> impl IntoElement {
    ChatHeader::new(id, "Add dark theme tokens")
        .project("carbon-ui")
        .branch("feat/dark-tokens")
        .status(Status::Working)
}

/// `h(C.TerminalDrawer,{tabs:[{title:'dev',running:true}],lines:[…]})`.
fn terminal_drawer(id: gpui_kit::ElementId) -> impl IntoElement {
    TerminalDrawer::new(id, [TerminalTab::new("dev").running(true)]).lines([
        "$ pnpm dev",
        "  VITE v6  ready in 412 ms",
        "  \u{279c}  Local: http://localhost:5173/",
    ])
}

/// `h('div',{style:{padding:24,display:'flex',flexDirection:'column',gap:16}},
/// h(C.UserMessage,{time:'10:42'},…),h(C.WorkingIndicator,{seconds:74}))`.
fn body(prefix: &str) -> impl IntoElement {
    let id = |n: &str| super::eid(prefix, n);
    stack()
        .p(px(24.))
        .gap(px(16.))
        .child(
            UserMessage::new(id("umsg"), "Add the G90 theme and update the cover.").time("10:42"),
        )
        .child(WorkingIndicator::new(
            id("working"),
            WorkingState::Working,
            74,
        ))
}

/// The width (rems) the reference layout needs: the 16rem sidebar, the
/// 20rem right panel and the 29rem the ChatHeader needs to show its status.
const NATURAL_WIDTH: f32 = 65.;

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
        frame = frame.header(chat_header(id("head")));
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
        frame = frame.terminal(terminal_drawer(id("term")));
    }
    frame.child(body(prefix)).into_any_element()
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
            .playground(fit_width("shell-live-fit", NATURAL_WIDTH, live))
            .hint("Click a thread to make it active")
            // The panel wraps below the stage before the shell has to scale.
            .stage_min_width(64.)
            .config(config)
            .example(controls::block(
                "Reference preview",
                fit_width(
                    "shell-ref-fit",
                    NATURAL_WIDTH,
                    shell((true, true, true, true, 0), "ref", None),
                ),
            ))
    }
}
