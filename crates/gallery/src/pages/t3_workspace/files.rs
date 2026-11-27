//! T3 · Files & preview: DeviceControlsRail, FileBrowserPanel,
//! FileSurfaceState, MiniPlayer, PreviewChrome, PreviewEmptyState and
//! TerminalDrawer.
use super::{note, row, stack};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use u0ui::prelude::*;
use u0ui::t3::review::FileStatus;
use u0ui::t3::workspace::{
    DeviceAction, DeviceControlsRail, FileBreadcrumbs, FileBrowserPanel, FileNode, FileSurfaceKind,
    FileSurfaceState, LocalServer, MiniPlayer, MiniPlayerKind, PreviewChrome, PreviewEmptyState,
    TerminalDrawer, TerminalTab, preview::VIEWPORTS,
};

fn status(s: impl Into<SharedString>) -> gpui_kit::Div {
    note(s)
}

// ── DeviceControlsRail ──────────────────────────────────────────────────

/// The phone frame next to the rail (`width: 150; height: 300; border: 6px solid
/// icon-primary; border-radius: 24; background: layer-01`).
fn phone() -> gpui_kit::Div {
    div()
        .w(px(150.))
        .h(px(300.))
        .border(px(6.))
        .border_color(color(ColorRole::IconPrimary))
        .rounded(px(24.))
        .bg(color(ColorRole::Layer01))
}

struct RailPage {
    keyboard: bool,
    last: Option<SharedString>,
}

/// Build the DeviceControlsRail page.
pub fn build_device_controls_rail(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RailPage {
        keyboard: false,
        last: None,
    })
    .into()
}

/// The overview tile.
pub fn device_controls_rail_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    DeviceControlsRail::new("rail-thumb")
        .actions(["Home", "Rotate", "Screenshot", "Record"].map(DeviceAction::new))
        .into_any_element()
}

impl Render for RailPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut actions: Vec<&str> = u0ui::t3::workspace::preview::DEVICE_ACTIONS.to_vec();
        if self.keyboard {
            actions.push("Keyboard");
        }
        let labels: Vec<SharedString> = actions.iter().map(|a| SharedString::from(*a)).collect();
        let live = stack(12.)
            .child(
                row().child(phone()).child(
                    DeviceControlsRail::new("rail-live")
                        .actions(actions.iter().map(|a| DeviceAction::new(*a)))
                        .on_action({
                            let entity = entity.clone();
                            move |i, _, cx| {
                                let label = labels[i].clone();
                                entity.update(cx, |this, cx| {
                                    this.last = Some(label);
                                    cx.notify();
                                })
                            }
                        }),
                ),
            )
            .when_some(self.last.clone(), |this, l| {
                this.child(status(format!("Pressed: {l}")))
            });
        let config = controls::toggle(
            "rail-keyboard",
            "Keyboard action",
            self.keyboard,
            controls::setter(&entity, |this: &mut RailPage, v| this.keyboard = v),
        );
        // `components/DeviceControlsRail/preview.html`.
        let reference = row()
            .child(phone())
            .child(DeviceControlsRail::new("rail-ref"));
        Body::new()
            .playground(live)
            .hint("Hover a button for its tooltip (on the left)")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── FileBrowserPanel ────────────────────────────────────────────────────

fn file_tree() -> Vec<FileNode> {
    vec![
        FileNode::dir(
            "src",
            "src",
            [
                FileNode::dir(
                    "src/components",
                    "components",
                    [FileNode::file("src/components/Button.tsx", "Button.tsx")],
                ),
                FileNode::file("src/tokens.json", "tokens.json").status(FileStatus::Modified),
                FileNode::file("src/index.ts", "index.ts"),
            ],
        )
        .open(true),
        FileNode::file("README.md", "README.md"),
        FileNode::file("package.json", "package.json"),
        FileNode::file("logo.png", "logo.png").status(FileStatus::Added),
    ]
}

struct FilesPage {
    loading: bool,
    selected: SharedString,
    crumb: Option<usize>,
}

/// Build the FileBrowserPanel page.
pub fn build_file_browser_panel(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| FilesPage {
        loading: false,
        selected: "src/tokens.json".into(),
        crumb: None,
    })
    .into()
}

/// The overview tile.
pub fn file_browser_panel_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .h(px(256.))
        .child(FileBrowserPanel::new("files-thumb", file_tree()).selected("src/tokens.json"))
        .into_any_element()
}

const CRUMB_PATH: &str = "carbon-ui/src/components/date-picker/DatePicker.tsx";

impl Render for FilesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = stack(12.)
            .child(
                div().w(px(300.)).h(px(320.)).child(
                    FileBrowserPanel::new("files-live", file_tree())
                        .selected(self.selected.clone())
                        .loading(self.loading)
                        .on_select({
                            let entity = entity.clone();
                            move |path, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.selected = path;
                                    cx.notify();
                                })
                            }
                        }),
                ),
            )
            .child(status(format!("Selected: {}", self.selected)));
        let config = controls::toggle(
            "files-loading",
            "loading",
            self.loading,
            controls::setter(&entity, |this: &mut FilesPage, v| this.loading = v),
        );
        // `components/FileBrowserPanel/preview.html`: the panel and FileBreadcrumbs.
        let reference =
            row()
                .gap(px(24.))
                .child(div().w(px(300.)).h(px(320.)).child(
                    FileBrowserPanel::new("files-ref", file_tree()).selected("src/tokens.json"),
                ))
                .child(FileBreadcrumbs::new("crumbs-ref", CRUMB_PATH));
        let crumbs = stack(8.)
            .child(FileBreadcrumbs::new("crumbs-live", CRUMB_PATH).on_select({
                let entity = entity.clone();
                move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.crumb = Some(i);
                        cx.notify();
                    })
                }
            }))
            .when_some(self.crumb, |this, i| {
                let seg = CRUMB_PATH.split('/').nth(i).unwrap_or_default();
                this.child(status(format!("Pressed: {seg}")))
            });
        Body::new()
            .playground(live)
            .hint("Click folders to fold them; type to filter; Tab and Enter work too")
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block("FileBreadcrumbs", crumbs))
    }
}

// ── FileSurfaceState ────────────────────────────────────────────────────

struct SurfacePage {
    kind: usize,
    note: bool,
    action: bool,
}

/// Build the FileSurfaceState page.
pub fn build_file_surface_state(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SurfacePage {
        kind: 4,
        note: true,
        action: true,
    })
    .into()
}

/// The overview tile.
pub fn file_surface_state_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            FileSurfaceState::new("fsurf-thumb", FileSurfaceKind::Binary)
                .title("Binary file not shown"),
        )
        .into_any_element()
}

fn surface_reference() -> impl IntoElement {
    // `display: grid; grid-template-columns: 1fr 1fr; gap: 16`.
    let cell = |e: AnyElement| div().flex_1().min_w(px(240.)).child(e);
    stack(16.)
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(16.))
                .child(cell(
                    FileSurfaceState::new("fsurf-ref-0", FileSurfaceKind::Loading)
                        .into_any_element(),
                ))
                .child(cell(
                    FileSurfaceState::new("fsurf-ref-1", FileSurfaceKind::Large)
                        .title("This file is 48 MB")
                        .body("Large files can slow down the preview.")
                        .action(
                            Button::new("fsurf-ref-1-show", "Show anyway")
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Secondary),
                        )
                        .into_any_element(),
                )),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(16.))
                .child(cell(
                    FileSurfaceState::new("fsurf-ref-2", FileSurfaceKind::Binary)
                        .title("Binary file not shown")
                        .into_any_element(),
                ))
                .child(cell(
                    FileSurfaceState::new("fsurf-ref-3", FileSurfaceKind::Failure)
                        .title("Could not read file")
                        .body("Permission denied.")
                        .action(
                            Button::new("fsurf-ref-3-retry", "Retry")
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Ghost),
                        )
                        .into_any_element(),
                )),
        )
}

impl Render for SurfacePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = FileSurfaceKind::ALL[self.kind];
        let title = match kind {
            FileSurfaceKind::Loading => None,
            FileSurfaceKind::Notice => Some("Preview not available"),
            FileSurfaceKind::Failure => Some("Could not read file"),
            FileSurfaceKind::Binary => Some("Binary file not shown"),
            FileSurfaceKind::Large => Some("This file is 48 MB"),
        };
        let mut live = FileSurfaceState::new("fsurf-live", kind);
        if let Some(t) = title {
            live = live.title(t);
        }
        if self.note {
            live = live.body(match kind {
                FileSurfaceKind::Failure => "Permission denied.",
                FileSurfaceKind::Large => "Large files can slow down the preview.",
                _ => "Open it in your editor instead.",
            });
        }
        if self.action {
            live = live.action(
                Button::new("fsurf-live-action", "Show anyway")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary),
            );
        }
        let kinds = FileSurfaceKind::ALL.map(|k| k.name());
        let config = stack(24.)
            .child(controls::field(
                "kind",
                controls::choices(
                    "fsurf-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut SurfacePage, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "fsurf-note",
                "children (note)",
                self.note,
                controls::setter(&entity, |this: &mut SurfacePage, v| this.note = v),
            ))
            .child(controls::toggle(
                "fsurf-action",
                "actions",
                self.action,
                controls::setter(&entity, |this: &mut SurfacePage, v| this.action = v),
            ));
        Body::new()
            .playground(div().w(px(400.)).max_w_full().child(live))
            .config(config)
            .example(controls::block("Reference preview", surface_reference()))
    }
}

// ── MiniPlayer ──────────────────────────────────────────────────────────

struct MiniPage {
    kind: usize,
    subtitle: bool,
    closed: bool,
    expanded: usize,
}

/// Build the MiniPlayer page.
pub fn build_mini_player(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| MiniPage {
        kind: 0,
        subtitle: true,
        closed: false,
        expanded: 0,
    })
    .into()
}

/// The overview tile.
pub fn mini_player_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(MiniPlayer::new("mini-thumb", "localhost:5173").subtitle("Preview"))
        .into_any_element()
}

impl Render for MiniPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = MiniPlayerKind::ALL[self.kind];
        let title = match kind {
            MiniPlayerKind::Browser => "localhost:5173",
            MiniPlayerKind::Device => "iPhone 16",
        };
        let live = if self.closed {
            stack(12.)
                .child(status("Closed."))
                .child(
                    Button::new("mini-reopen", "Reopen")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Tertiary)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.closed = false;
                                    cx.notify();
                                })
                            }
                        }),
                )
                .into_any_element()
        } else {
            let mut mini = MiniPlayer::new("mini-live", title)
                .kind(kind)
                .on_expand({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.expanded += 1;
                            cx.notify();
                        })
                    }
                })
                .on_close({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.closed = true;
                            cx.notify();
                        })
                    }
                });
            if self.subtitle {
                mini = mini.subtitle(match kind {
                    MiniPlayerKind::Browser => "Preview",
                    MiniPlayerKind::Device => "Simulator",
                });
            }
            stack(12.)
                .child(mini)
                .when(self.expanded > 0, |this| {
                    this.child(status(format!("Expand pressed {}×", self.expanded)))
                })
                .into_any_element()
        };
        let kinds = MiniPlayerKind::ALL.map(|k| k.name());
        let config = stack(24.)
            .child(controls::field(
                "kind",
                controls::choices(
                    "mini-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut MiniPage, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "mini-subtitle",
                "subtitle",
                self.subtitle,
                controls::setter(&entity, |this: &mut MiniPage, v| this.subtitle = v),
            ));
        // `components/MiniPlayer/preview.html`.
        let reference = row()
            .child(MiniPlayer::new("mini-ref-0", "localhost:5173").subtitle("Preview"))
            .child(
                MiniPlayer::new("mini-ref-1", "iPhone 16")
                    .kind(MiniPlayerKind::Device)
                    .subtitle("Simulator"),
            );
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── PreviewChrome ───────────────────────────────────────────────────────

struct ChromePage {
    loading: bool,
    secure: bool,
    zoom: usize,
    viewport: usize,
    log: Option<SharedString>,
}

const ZOOMS: [&str; 4] = ["100", "75", "125", "150"];

/// Build the PreviewChrome page.
pub fn build_preview_chrome(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ChromePage {
        loading: false,
        secure: false,
        zoom: 0,
        viewport: 0,
        log: None,
    })
    .into()
}

/// The overview tile.
pub fn preview_chrome_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(PreviewChrome::new("chrome-thumb", "http://localhost:5173").loading(true))
        .into_any_element()
}

impl Render for ChromePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let say = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.log = Some(what.into());
                    cx.notify();
                })
            }
        };
        let live = stack(12.)
            .child(
                PreviewChrome::new(
                    "chrome-live",
                    if self.secure {
                        "https://staging.acme.dev/login"
                    } else {
                        "http://localhost:5173/settings"
                    },
                )
                .loading(self.loading)
                .secure(self.secure)
                .zoom(ZOOMS[self.zoom].parse().unwrap_or(100))
                .viewport(VIEWPORTS[self.viewport])
                .on_back(say("Back"))
                .on_reload({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.loading = true;
                            this.log = Some("Reload".into());
                            cx.notify();
                        })
                    }
                })
                .on_stop({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.loading = false;
                            this.log = Some("Stop".into());
                            cx.notify();
                        })
                    }
                })
                .on_navigate({
                    let entity = entity.clone();
                    move |url, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.log = Some(format!("Navigate to {url}").into());
                            cx.notify();
                        })
                    }
                })
                .on_viewport_change({
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.viewport = i;
                            cx.notify();
                        })
                    }
                })
                .on_open_external(say("Open in system browser"))
                .on_more(say("More")),
            )
            .when_some(self.log.clone(), |this, l| this.child(status(l)));
        let config = stack(24.)
            .child(controls::toggle(
                "chrome-loading",
                "loading",
                self.loading,
                controls::setter(&entity, |this: &mut ChromePage, v| this.loading = v),
            ))
            .child(controls::toggle(
                "chrome-secure",
                "secure",
                self.secure,
                controls::setter(&entity, |this: &mut ChromePage, v| this.secure = v),
            ))
            .child(controls::field(
                "zoom",
                controls::choices(
                    "chrome-zoom",
                    &ZOOMS,
                    self.zoom,
                    controls::setter(&entity, |this: &mut ChromePage, i| this.zoom = i),
                ),
            ))
            .child(controls::field(
                "viewport",
                controls::choices(
                    "chrome-viewport",
                    &VIEWPORTS,
                    self.viewport,
                    controls::setter(&entity, |this: &mut ChromePage, i| this.viewport = i),
                ),
            ));
        // `components/PreviewChrome/preview.html` (`cu-stack`, gap 12).
        let reference = stack(12.)
            .child(PreviewChrome::new(
                "chrome-ref-0",
                "http://localhost:5173/settings",
            ))
            .child(
                PreviewChrome::new("chrome-ref-1", "https://staging.acme.dev/login")
                    .loading(true)
                    .secure(true)
                    .zoom(125)
                    .viewport("iPhone 16"),
            );
        Body::new()
            .playground(div().w_full().child(live))
            .hint("Reload starts loading, Stop ends it; edit the address and press Enter")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── PreviewEmptyState ───────────────────────────────────────────────────

struct EmptyPage {
    servers: bool,
    recent: bool,
    picked: Option<SharedString>,
}

/// Build the PreviewEmptyState page.
pub fn build_preview_empty_state(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| EmptyPage {
        servers: true,
        recent: true,
        picked: None,
    })
    .into()
}

fn servers() -> [LocalServer; 2] {
    [
        LocalServer::new(5173, "vite dev"),
        LocalServer::new(6006, "storybook"),
    ]
}

const RECENT: [&str; 2] = ["localhost:5173/settings", "staging.acme.dev"];

/// The overview tile.
pub fn preview_empty_state_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(PreviewEmptyState::new("pempty-thumb").servers([LocalServer::new(5173, "vite")]))
        .into_any_element()
}

impl Render for EmptyPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut live = PreviewEmptyState::new("pempty-live")
            .on_server({
                let entity = entity.clone();
                move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.picked = Some(format!("Open :{}", servers()[i].port).into());
                        cx.notify();
                    })
                }
            })
            .on_recent({
                let entity = entity.clone();
                move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.picked = Some(format!("Open {}", RECENT[i]).into());
                        cx.notify();
                    })
                }
            });
        if self.servers {
            live = live.servers(servers());
        }
        if self.recent {
            live = live.recent(RECENT);
        }
        let config = stack(24.)
            .child(controls::toggle(
                "pempty-servers",
                "servers",
                self.servers,
                controls::setter(&entity, |this: &mut EmptyPage, v| this.servers = v),
            ))
            .child(controls::toggle(
                "pempty-recent",
                "recent",
                self.recent,
                controls::setter(&entity, |this: &mut EmptyPage, v| this.recent = v),
            ));
        Body::new()
            .playground(
                stack(8.)
                    .child(live)
                    .when_some(self.picked.clone(), |this, p| this.child(status(p))),
            )
            .config(config)
            .example(controls::block(
                "Reference preview",
                PreviewEmptyState::new("pempty-ref")
                    .servers(servers())
                    .recent(RECENT),
            ))
    }
}

// ── TerminalDrawer ──────────────────────────────────────────────────────

const LINES: [&str; 6] = [
    "$ pnpm dev",
    "",
    "  VITE v6.0.7  ready in 412 ms",
    "",
    "  \u{279c}  Local:   http://localhost:5173/",
    "  \u{279c}  Network: use --host to expose",
];

struct TerminalPage {
    tabs: Vec<TerminalTab>,
    exited: bool,
    running: bool,
    log: Option<SharedString>,
}

fn reference_tabs() -> Vec<TerminalTab> {
    vec![
        TerminalTab::new("dev").running(true),
        TerminalTab::new("test"),
    ]
}

/// Build the TerminalDrawer page.
pub fn build_terminal_drawer(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TerminalPage {
        tabs: reference_tabs(),
        exited: false,
        running: true,
        log: None,
    })
    .into()
}

/// The overview tile.
pub fn terminal_drawer_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .h(px(192.))
        .child(
            TerminalDrawer::new("term-thumb", [TerminalTab::new("dev").running(true)])
                .lines(LINES.iter().take(3).copied()),
        )
        .into_any_element()
}

impl Render for TerminalPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let say = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.log = Some(what.into());
                    cx.notify();
                })
            }
        };
        let mut tabs = self.tabs.clone();
        if let Some(first) = tabs.first_mut() {
            first.running = self.running && !self.exited;
        }
        let live = stack(12.)
            .child(
                div().h(px(220.)).child(
                    TerminalDrawer::new("term-live", tabs)
                        .lines(LINES)
                        .exited(self.exited.then_some(0))
                        .on_close({
                            let entity = entity.clone();
                            move |i, _, cx| {
                                entity.update(cx, |this, cx| {
                                    if i < this.tabs.len() {
                                        let t = this.tabs.remove(i);
                                        this.log = Some(format!("Closed {}", t.title).into());
                                    }
                                    cx.notify();
                                })
                            }
                        })
                        .on_new({
                            let entity = entity.clone();
                            move |_, cx| {
                                entity.update(cx, |this, cx| {
                                    let n = this.tabs.len() + 1;
                                    this.tabs.push(TerminalTab::new(format!("zsh {n}")));
                                    cx.notify();
                                })
                            }
                        })
                        .on_split(say("Split right"))
                        .on_add_to_chat(say("Add selection to chat"))
                        .on_maximize(say("Maximize"))
                        .on_hide(say("Hide terminal")),
                ),
            )
            .when_some(self.log.clone(), |this, l| this.child(status(l)));
        let config = stack(24.)
            .child(controls::toggle(
                "term-running",
                "running (first tab)",
                self.running,
                controls::setter(&entity, |this: &mut TerminalPage, v| this.running = v),
            ))
            .child(controls::toggle(
                "term-exited",
                "exited (code 0)",
                self.exited,
                controls::setter(&entity, |this: &mut TerminalPage, v| this.exited = v),
            ))
            .child(
                Button::new("term-reset", "Restore tabs")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Tertiary)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.tabs = reference_tabs();
                                cx.notify();
                            })
                        }
                    }),
            );
        // `components/TerminalDrawer/preview.html` (height 220).
        let reference = div()
            .h(px(220.))
            .child(TerminalDrawer::new("term-ref", reference_tabs()).lines(LINES));
        Body::new()
            .playground(div().w_full().child(live))
            .hint("Click tabs to switch; × closes one; + adds one")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
