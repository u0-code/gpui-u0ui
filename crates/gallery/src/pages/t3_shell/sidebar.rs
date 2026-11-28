//! T3 · Shell sidebar pages: SidebarBrandHeader, ProjectFolderHeader,
//! SidebarShelfHeader, ThreadRow, UpdatePill.
use super::{label_text, panel, stack};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::base::use_ticker;
use u0ui::t3::review::PullRequestState;
use u0ui::t3::shell::{
    ConnectionStatus, Count, EnvironmentBadge, FolderCounts, MachineKind, ProjectFolderHeader,
    SidebarBrandHeader, SidebarShelfHeader, ThreadRow, UpdatePill, UpdateState,
};

/// "Last action: …" under a playground.
fn last_action(text: &SharedString) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(format!("Last action: {text}"))
}

/// A setter that records `what` as the page's last action.
fn record<T: 'static>(
    entity: &Entity<T>,
    what: &'static str,
    f: fn(&mut T) -> &mut SharedString,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let entity = entity.clone();
    move |_, cx| {
        entity.update(cx, |this, cx| {
            *f(this) = what.into();
            cx.notify();
        })
    }
}

// ── SidebarBrandHeader ──────────────────────────────────────────────────

struct BrandPage {
    can_back: bool,
    can_forward: bool,
    action: SharedString,
}

/// Build the SidebarBrandHeader page.
pub fn build_brand_header(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| BrandPage {
        can_back: true,
        can_forward: false,
        action: "none".into(),
    })
    .into()
}

/// Overview specimen.
pub fn brand_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    panel(240.)
        .child(SidebarBrandHeader::new("thumb-brand", "Workbench"))
        .into_any_element()
}

impl Render for BrandPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let act = |what| record(&entity, what, |p: &mut BrandPage| &mut p.action);
        let live = stack()
            .gap(px(16.))
            .child(
                panel(280.).child(
                    SidebarBrandHeader::new("live-brand", "Workbench")
                        .can_back(self.can_back)
                        .can_forward(self.can_forward)
                        .on_back(act("Back"))
                        .on_forward(act("Forward"))
                        .on_search(act("Search"))
                        .on_new_thread(act("New thread")),
                ),
            )
            .child(last_action(&self.action));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "brand-back",
                "canBack",
                self.can_back,
                controls::setter(&entity, |this: &mut BrandPage, on| this.can_back = on),
            ))
            .child(controls::toggle(
                "brand-forward",
                "canForward",
                self.can_forward,
                controls::setter(&entity, |this: &mut BrandPage, on| this.can_forward = on),
            ));
        // `components/SidebarBrandHeader/preview.html`: 280px on layer-01.
        let reference = panel(280.).child(SidebarBrandHeader::new("ref-brand", "Workbench"));
        Body::new()
            .playground(live)
            .hint("Hover the buttons for their shortcuts")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── ProjectFolderHeader ─────────────────────────────────────────────────

struct FolderPage {
    expanded: bool,
    pinned: bool,
    environment: bool,
    counts: bool,
    action: SharedString,
}

/// Build the ProjectFolderHeader page.
pub fn build_folder_header(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| FolderPage {
        expanded: true,
        pinned: true,
        environment: false,
        counts: true,
        action: "none".into(),
    })
    .into()
}

fn folder_reference(prefix: &str) -> impl IntoElement {
    let id = |n: &str| super::eid(prefix, n);
    panel(300.)
        .child(
            ProjectFolderHeader::new(id("a"), "carbon-ui")
                .counts(FolderCounts {
                    needs_you: 1,
                    working: 2,
                    done: 3,
                    ..Default::default()
                })
                .pinned(true),
        )
        .child(
            ProjectFolderHeader::new(id("b"), "marketing-site")
                .environment(
                    EnvironmentBadge::new("build-box")
                        .id(id("env"))
                        .kind(MachineKind::Server)
                        .status(ConnectionStatus::Connected),
                )
                .counts(FolderCounts {
                    failed: 1,
                    ..Default::default()
                }),
        )
        .child(ProjectFolderHeader::new(id("c"), "api-gateway").expanded(false))
}

/// Overview specimen.
pub fn folder_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    folder_reference("thumb-folder").into_any_element()
}

impl Render for FolderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.expanded = !this.expanded;
                    this.action = "Toggle".into();
                    cx.notify();
                })
            }
        };
        let mut header = ProjectFolderHeader::new("live-folder", "carbon-ui")
            .expanded(self.expanded)
            .pinned(self.pinned)
            .on_toggle(toggle)
            .on_new_thread(record(&entity, "New thread", |p: &mut FolderPage| {
                &mut p.action
            }));
        if self.counts {
            header = header.counts(FolderCounts {
                needs_you: 1,
                failed: 1,
                working: 2,
                done: 3,
            });
        }
        if self.environment {
            header = header.environment(
                EnvironmentBadge::new("build-box")
                    .id("live-folder-env")
                    .kind(MachineKind::Server)
                    .status(ConnectionStatus::Connecting),
            );
        }
        let live = stack()
            .gap(px(16.))
            .child(panel(300.).child(header))
            .child(last_action(&self.action));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "folder-expanded",
                "expanded",
                self.expanded,
                controls::setter(&entity, |this: &mut FolderPage, on| this.expanded = on),
            ))
            .child(controls::toggle(
                "folder-pinned",
                "pinned",
                self.pinned,
                controls::setter(&entity, |this: &mut FolderPage, on| this.pinned = on),
            ))
            .child(controls::toggle(
                "folder-counts",
                "counts",
                self.counts,
                controls::setter(&entity, |this: &mut FolderPage, on| this.counts = on),
            ))
            .child(controls::toggle(
                "folder-env",
                "environment",
                self.environment,
                controls::setter(&entity, |this: &mut FolderPage, on| this.environment = on),
            ));
        Body::new()
            .playground(live)
            .hint("Click the name to collapse; Tab to reach the add button")
            .config(config)
            .example(controls::block(
                "Reference preview",
                folder_reference("ref-folder"),
            ))
            .example(controls::block("Count", counts()))
    }
}

/// Every count a header shows: quiet (a shelf's), then each status's,
/// one, two and three digits.
fn counts() -> impl IntoElement {
    let row = |status: Option<Status>| {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .children([1, 12, 128].map(|n| {
                let count = Count::new(n);
                status.map_or(count, |s| Count::new(n).status(s))
            }))
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(row(None))
        .children(
            [
                Status::NeedsYou,
                Status::Failed,
                Status::Working,
                Status::Done,
            ]
            .map(|s| row(Some(s))),
        )
}

// ── SidebarShelfHeader ──────────────────────────────────────────────────

struct ShelfPage {
    expanded: bool,
    count: bool,
    action: bool,
}

/// Build the SidebarShelfHeader page.
pub fn build_shelf_header(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ShelfPage {
        expanded: true,
        count: true,
        action: false,
    })
    .into()
}

fn shelf_reference(prefix: &str) -> impl IntoElement {
    let id = |n: &str| super::eid(prefix, n);
    panel(300.)
        .child(SidebarShelfHeader::new(id("a"), "Working").count(2))
        .child(
            SidebarShelfHeader::new(id("b"), "Snoozed")
                .count(4)
                .expanded(false),
        )
        .child(
            SidebarShelfHeader::new(id("c"), "Settled")
                .count(31)
                .expanded(false),
        )
}

/// Overview specimen.
pub fn shelf_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    shelf_reference("thumb-shelf").into_any_element()
}

impl Render for ShelfPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = controls::setter(&entity, |this: &mut ShelfPage, (): ()| {
            this.expanded = !this.expanded
        });
        let mut shelf = SidebarShelfHeader::new("live-shelf", "Working")
            .expanded(self.expanded)
            .on_toggle(move |w, cx| toggle((), w, cx));
        if self.count {
            shelf = shelf.count(2);
        }
        if self.action {
            shelf = shelf.action(
                Button::new("live-shelf-action", "Settle all")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm),
            );
        }
        let rows = self.expanded.then(|| {
            stack()
                .child(ThreadRow::new("shelf-t1", "Add dark theme tokens").status(Status::Working))
                .child(ThreadRow::new("shelf-t2", "Update the cover").status(Status::Working))
        });
        let live = panel(300.).child(shelf).children(rows);
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "shelf-expanded",
                "expanded",
                self.expanded,
                controls::setter(&entity, |this: &mut ShelfPage, on| this.expanded = on),
            ))
            .child(controls::toggle(
                "shelf-count",
                "count",
                self.count,
                controls::setter(&entity, |this: &mut ShelfPage, on| this.count = on),
            ))
            .child(controls::toggle(
                "shelf-action",
                "action",
                self.action,
                controls::setter(&entity, |this: &mut ShelfPage, on| this.action = on),
            ));
        Body::new()
            .playground(live)
            .hint("Click the label to fold the shelf")
            .config(config)
            .example(controls::block(
                "Reference preview",
                shelf_reference("ref-shelf"),
            ))
    }
}

// ── ThreadRow ───────────────────────────────────────────────────────────

const META: [&str; 4] = ["none", "provider + branch", "snoozed", "all"];
const SIDE: [&str; 4] = ["time", "PR badge", "jump hint", "pinned"];

struct ThreadPage {
    status: usize,
    unread: bool,
    draft: bool,
    active: bool,
    meta: usize,
    side: usize,
    clicks: usize,
}

/// Build the ThreadRow page.
pub fn build_thread_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ThreadPage {
        status: 0,
        unread: false,
        draft: false,
        active: true,
        meta: 1,
        side: 0,
        clicks: 0,
    })
    .into()
}

fn thread_reference(prefix: &str) -> impl IntoElement {
    let id = |n: &str| super::eid(prefix, n);
    panel(320.)
        .child(
            ThreadRow::new(id("1"), "Add dark theme tokens")
                .status(Status::Working)
                .elapsed(74)
                .active(true)
                .provider("Claude")
                .branch("feat/dark-tokens"),
        )
        .child(
            ThreadRow::new(id("2"), "Approve: run database migration")
                .status(Status::NeedsYou)
                .time("3m")
                .unread(true),
        )
        .child(
            ThreadRow::new(id("3"), "Fix flaky date picker test")
                .status(Status::Failed)
                .time("12m"),
        )
        .child(
            ThreadRow::new(id("4"), "Write migration guide")
                .status(Status::Done)
                .time("1h")
                .pull_request(482, PullRequestState::Open),
        )
        .child(
            ThreadRow::new(id("5"), "Refactor toolbar")
                .draft(true)
                .time("2d"),
        )
        .child(
            ThreadRow::new(id("6"), "Investigate memory spike")
                .time("3d")
                .pinned(true)
                .snoozed_until("Mon 9:00"),
        )
        .child(
            ThreadRow::new(id("7"), "Release notes for 1.4")
                .time("5d")
                .jump_hint("⌘3"),
        )
}

/// Overview specimen.
pub fn thread_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    let id = |n: &str| super::eid("thumb-thread", n);
    panel(240.)
        .child(
            ThreadRow::new(id("1"), "Add dark theme tokens")
                .status(Status::Working)
                .elapsed(74)
                .active(true),
        )
        .child(
            ThreadRow::new(id("2"), "Approve: run migration")
                .status(Status::NeedsYou)
                .time("3m")
                .unread(true),
        )
        .into_any_element()
}

impl Render for ThreadPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let status = Status::ALL[self.status];
        // A live elapsed counter while working (`useTicker`).
        let elapsed = use_ticker(
            "thread-page-ticker",
            74,
            status == Status::Working,
            window,
            cx,
        );
        let mut row = ThreadRow::new("live-thread", "Add dark theme tokens")
            .status(status)
            .elapsed(elapsed)
            .time("3m")
            .unread(self.unread)
            .draft(self.draft)
            .active(self.active)
            .on_click(
                controls::setter(&entity, |this: &mut ThreadPage, (): ()| this.clicks += 1)
                    .clone_into_handler(),
            );
        if matches!(self.meta, 1 | 3) {
            row = row.provider("Claude").branch("feat/dark-tokens");
        }
        if matches!(self.meta, 2 | 3) {
            row = row.snoozed_until("Mon 9:00");
        }
        row = match self.side {
            1 => row.pull_request(482, PullRequestState::Open),
            2 => row.jump_hint("⌘3"),
            3 => row.pinned(true),
            _ => row,
        };
        let live = stack()
            .gap(px(16.))
            .child(panel(320.).child(row))
            .child(last_action(&format!("{} clicks", self.clicks).into()));
        let statuses = Status::ALL.map(|s| s.name());
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "status",
                controls::choices(
                    "thread-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut ThreadPage, i| this.status = i),
                ),
            ))
            .child(controls::field(
                "meta line",
                controls::choices(
                    "thread-meta",
                    &META,
                    self.meta,
                    controls::setter(&entity, |this: &mut ThreadPage, i| this.meta = i),
                ),
            ))
            .child(controls::field(
                "side",
                controls::choices(
                    "thread-side",
                    &SIDE,
                    self.side,
                    controls::setter(&entity, |this: &mut ThreadPage, i| this.side = i),
                ),
            ))
            .child(controls::toggle(
                "thread-active",
                "active",
                self.active,
                controls::setter(&entity, |this: &mut ThreadPage, on| this.active = on),
            ))
            .child(controls::toggle(
                "thread-unread",
                "unread",
                self.unread,
                controls::setter(&entity, |this: &mut ThreadPage, on| this.unread = on),
            ))
            .child(controls::toggle(
                "thread-draft",
                "draft",
                self.draft,
                controls::setter(&entity, |this: &mut ThreadPage, on| this.draft = on),
            ));
        let prs = panel(320.).children(PullRequestState::ALL.map(|s| {
            ThreadRow::new(super::eid("pr-row", s.name()), format!("PR {}", s.name()))
                .status(Status::Done)
                .time("1h")
                .pull_request(482, s)
        }));
        Body::new()
            .playground(live)
            .hint("Working rows tick their elapsed time; hover the PR badge for its title")
            .config(config)
            .example(controls::block(
                "Reference preview",
                thread_reference("ref-thread"),
            ))
            .example(controls::block("PullRequestMiniBadge states", prs))
    }
}

/// Turns a `controls::setter` taking `()` into a plain handler.
trait IntoHandler {
    fn clone_into_handler(self) -> impl Fn(&mut Window, &mut App) + 'static;
}

impl<F: Fn((), &mut Window, &mut App) + 'static> IntoHandler for F {
    fn clone_into_handler(self) -> impl Fn(&mut Window, &mut App) + 'static {
        move |w, cx| self((), w, cx)
    }
}

// ── UpdatePill ──────────────────────────────────────────────────────────

const PROGRESS: [&str; 5] = ["0", "25", "62", "90", "100"];

struct UpdatePage {
    state: usize,
    progress: usize,
    clicks: usize,
}

/// Build the UpdatePill page.
pub fn build_update_pill(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| UpdatePage {
        state: 1,
        progress: 2,
        clicks: 0,
    })
    .into()
}

fn update_reference(prefix: &str) -> impl IntoElement {
    let id = |n: &str| super::eid(prefix, n);
    // `width: 280px; flex-direction: column; gap: 8px`.
    stack()
        .w(px(280.))
        .gap(px(8.))
        .child(UpdatePill::new(id("a"), UpdateState::Available).version("1.4.2"))
        .child(UpdatePill::new(id("b"), UpdateState::Downloading).progress(62))
        .child(UpdatePill::new(id("c"), UpdateState::Ready))
        .child(UpdatePill::new(id("d"), UpdateState::Error))
}

/// Overview specimen.
pub fn update_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    stack()
        .w(px(220.))
        .gap(px(8.))
        .child(UpdatePill::new("thumb-up-a", UpdateState::Downloading).progress(62))
        .child(UpdatePill::new("thumb-up-b", UpdateState::Ready))
        .into_any_element()
}

impl Render for UpdatePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let progress: u8 = PROGRESS[self.progress].parse().unwrap_or(0);
        let live = stack()
            .gap(px(16.))
            .child(
                div().w(px(280.)).child(
                    UpdatePill::new("live-update", UpdateState::ALL[self.state])
                        .version("1.4.2")
                        .progress(progress)
                        .on_click(
                            controls::setter(&entity, |this: &mut UpdatePage, (): ()| {
                                this.clicks += 1
                            })
                            .clone_into_handler(),
                        ),
                ),
            )
            .child(label_text(
                format!("{} clicks", self.clicks),
                TypeStyle::Label01,
            ));
        let states = UpdateState::ALL.map(|s| s.name());
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "update-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut UpdatePage, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "progress",
                controls::choices(
                    "update-progress",
                    &PROGRESS,
                    self.progress,
                    controls::setter(&entity, |this: &mut UpdatePage, i| this.progress = i),
                ),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                update_reference("ref-update"),
            ))
    }
}
