//! u0 · Shell › ThreadRow and FilterMenu: live with their
//! configuration, plus the reference previews.
use super::last_action;
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::t3::shell::MachineKind;
use u0ui::u0::shell::{FilterItem, FilterMenu, RowServer, ThreadRow, ThreadStatus};

// ── ThreadRow ──────────────────────────────────────────────────────

/// `components/GhostThreadRow/preview.html`: seven rows, 380px on `layer-01`.
fn row_reference() -> impl IntoElement {
    div()
        .w_full()
        .max_w(rems(23.75))
        .bg(color(ColorRole::Layer01))
        .child(
            ThreadRow::new("gtr-0", "Add dark theme tokens")
                .status(ThreadStatus::Working)
                .elapsed(74)
                .current(true),
        )
        .child(
            ThreadRow::new("gtr-1", "Approve: run database migration")
                .status(ThreadStatus::NeedsYou)
                .time("3m")
                .unread(true),
        )
        .child(
            ThreadRow::new("gtr-2", "Investigate memory spike")
                .time("2h")
                .unread(true)
                .tags(["infra"]),
        )
        .child(
            ThreadRow::new("gtr-3", "Refactor toolbar")
                .draft(true)
                .time("2d"),
        )
        .child(
            ThreadRow::new("gtr-4", "Release checklist")
                .time("1d")
                .pinned(true)
                .server(RowServer::new("build-box").kind(MachineKind::Server)),
        )
        .child(
            ThreadRow::new("gtr-5", "Fix flaky date picker test")
                .status(ThreadStatus::Failed)
                .time("12m")
                .selected(true),
        )
        .child(
            ThreadRow::new("gtr-6", "Old spike")
                .archived(true)
                .time("3w"),
        )
}

/// The ThreadRow overview tile.
pub fn row_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(
            ThreadRow::new("gtr-thumb-0", "Add dark theme tokens")
                .status(ThreadStatus::Working)
                .elapsed(74)
                .current(true),
        )
        .child(
            ThreadRow::new("gtr-thumb-1", "Investigate spike")
                .time("2h")
                .unread(true),
        )
        .into_any_element()
}

struct RowPage {
    status: usize,
    current: bool,
    selected: bool,
    unread: bool,
    draft: bool,
    archived: bool,
    pinned: bool,
    tags: bool,
    server: bool,
    last: SharedString,
}

/// Build the ThreadRow page.
pub fn build_thread_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RowPage {
        status: 1,
        current: true,
        selected: false,
        unread: false,
        draft: false,
        archived: false,
        pinned: true,
        tags: true,
        server: true,
        last: "—".into(),
    })
    .into()
}

impl Render for RowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let click = e.clone();
        let names: Vec<&'static str> = ThreadStatus::ALL.iter().map(|s| s.name()).collect();
        let live = div()
            .w_full()
            .max_w(rems(23.75))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                div().bg(color(ColorRole::Layer01)).child(
                    ThreadRow::new("gtr-live", "Add dark theme tokens")
                        .status(ThreadStatus::ALL[self.status])
                        .time("2m")
                        .elapsed(74)
                        .current(self.current)
                        .selected(self.selected)
                        .unread(self.unread)
                        .draft(self.draft)
                        .archived(self.archived)
                        .pinned(self.pinned)
                        .when(self.tags, |r| r.tags(["infra"]))
                        .when(self.server, |r| {
                            r.server(RowServer::new("build-box").kind(MachineKind::Server))
                        })
                        .on_click(move |_, cx| {
                            click.update(cx, |p, cx| {
                                p.last = "onClick".into();
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(last_action(&self.last));
        let flag = |id: &'static str, label: &'static str, on: bool, f: fn(&mut RowPage, bool)| {
            controls::toggle(id, label, on, controls::setter(&e, f))
        };
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(controls::field(
                "status",
                controls::choices(
                    "gtr-status",
                    &names,
                    self.status,
                    controls::setter(&e, |p: &mut RowPage, v| p.status = v),
                ),
            ))
            .child(flag("gtr-current", "current", self.current, |p, v| {
                p.current = v
            }))
            .child(flag("gtr-selected", "selected", self.selected, |p, v| {
                p.selected = v
            }))
            .child(flag("gtr-unread", "unread", self.unread, |p, v| {
                p.unread = v
            }))
            .child(flag("gtr-draft", "draft", self.draft, |p, v| p.draft = v))
            .child(flag("gtr-archived", "archived", self.archived, |p, v| {
                p.archived = v
            }))
            .child(flag("gtr-pinned", "pinned", self.pinned, |p, v| {
                p.pinned = v
            }))
            .child(flag("gtr-tags", "tags", self.tags, |p, v| p.tags = v))
            .child(flag("gtr-server", "server", self.server, |p, v| {
                p.server = v
            }));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", row_reference()))
    }
}

// ── FilterMenu ──────────────────────────────────────────────────────────

fn projects() -> Vec<FilterItem> {
    vec![
        FilterItem::new("carbon-ui", "carbon-ui").count(6),
        FilterItem::new("marketing-site", "marketing-site").count(2),
        FilterItem::new("api-gateway", "api-gateway").count(9),
        FilterItem::new("scratch", "scratch").count(1),
    ]
}

/// The FilterMenu overview tile.
pub fn filter_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    FilterMenu::new("fm-thumb", "Projects", projects())
        .default_value(["carbon-ui"])
        .into_any_element()
}

struct FilterPage {
    chosen: Vec<SharedString>,
}

/// Build the FilterMenu page.
pub fn build_filter_menu(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| FilterPage {
        chosen: vec!["carbon-ui".into()],
    })
    .into()
}

impl Render for FilterPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let live = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(tokens::SPACING_03)
            .h(rems(20.))
            .child(
                FilterMenu::new("fm-live", "Projects", projects())
                    .value(self.chosen.clone())
                    .on_value_change(move |v, _, cx| {
                        e.update(cx, |p, cx| {
                            p.chosen = v;
                            cx.notify();
                        })
                    }),
            )
            .child(last_action(&SharedString::from(format!(
                "value = [{}]",
                self.chosen.join(", ")
            ))));
        // `components/FilterMenu/preview.html`: 300px tall, carbon-ui chosen.
        // The trigger is an inline-block at the start edge (`.g-fmenu {
        // position: relative; display: inline-block }`).
        let reference =
            div().flex().flex_col().items_start().h(rems(18.75)).child(
                FilterMenu::new("fm-ref", "Projects", projects()).default_value(["carbon-ui"]),
            );
        Body::new()
            .playground(live)
            .hint("Open it, type to filter, toggle options; Esc or a press outside closes it")
            .example(controls::block("Reference preview", reference))
    }
}
