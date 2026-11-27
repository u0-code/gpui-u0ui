//! u0 · Right column › CommitRow, StashRow, CommitComposer and
//! LinkedItemsSection: each live with its configuration, plus the
//! reference previews.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{ChecksStatus, FileStatus, IssueState, PullRequestState};
use u0ui::u0::panels::{
    ChangedFile, CommitComposer, CommitGraph, CommitRow, LinkedItem, LinkedItemsSection, StashRow,
};

/// A line of what the live component last reported.
fn log(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

fn on_layer(width: f32) -> gpui_kit::Div {
    div()
        .w_full()
        .max_w(rems(width / 16.))
        .bg(color(ColorRole::Layer01))
}

// ── CommitRow ───────────────────────────────────────────────────────────

/// `components/CommitRow/preview.html`: three commits, 480px on `layer-01`.
fn commit_reference() -> impl IntoElement {
    on_layer(480.)
        .child(
            CommitRow::new("cr-0", "a1b2c3d4", "Add G90 layer tokens")
                .author("Ana")
                .time("2m")
                .refs(["HEAD", "feat/dark-tokens"])
                .stat(42, 3)
                .current(true)
                .graph(CommitGraph::Start),
        )
        .child(
            CommitRow::new("cr-1", "e5f6a7b8", "Fix date picker focus trap")
                .author("Ben")
                .time("1h")
                .refs(["origin/main"])
                .stat(8, 5),
        )
        .child(
            CommitRow::new("cr-2", "c9d0e1f2", "Bump @carbon/react to 1.121")
                .author("Ana")
                .time("1d")
                .graph(CommitGraph::End),
        )
}

/// The CommitRow overview tile.
pub fn commit_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(
            CommitRow::new("cr-thumb", "a1b2c3d4", "Add G90 layer tokens")
                .author("Ana")
                .time("2m")
                .refs(["HEAD"])
                .current(true)
                .graph(CommitGraph::Start),
        )
        .into_any_element()
}

struct CommitPage {
    graph: usize,
    current: bool,
    refs: bool,
    stat: bool,
    clicks: usize,
}

/// Build the CommitRow page.
pub fn build_commit_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CommitPage {
        graph: 1,
        current: false,
        refs: true,
        stat: true,
        clicks: 0,
    })
    .into()
}

impl Render for CommitPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let click = e.clone();
        let names: Vec<&'static str> = CommitGraph::ALL.iter().map(|g| g.name()).collect();
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                on_layer(480.).child(
                    CommitRow::new("cr-live", "a1b2c3d4e5", "Add G90 layer tokens")
                        .author("Ana")
                        .time("2m")
                        .when(self.refs, |r| {
                            r.refs(["HEAD", "feat/dark-tokens", "origin/main"])
                        })
                        .when(self.stat, |r| r.stat(42, 3))
                        .current(self.current)
                        .graph(CommitGraph::ALL[self.graph])
                        .on_click(move |_, cx| {
                            click.update(cx, |p, cx| {
                                p.clicks += 1;
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(log(format!("onClick × {}", self.clicks)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "graph",
                controls::choices(
                    "cr-graph",
                    &names,
                    self.graph,
                    controls::setter(&e, |p: &mut CommitPage, v| p.graph = v),
                ),
            ))
            .child(controls::toggle(
                "cr-current",
                "current",
                self.current,
                controls::setter(&e, |p: &mut CommitPage, v| p.current = v),
            ))
            .child(controls::toggle(
                "cr-refs",
                "refs",
                self.refs,
                controls::setter(&e, |p: &mut CommitPage, v| p.refs = v),
            ))
            .child(controls::toggle(
                "cr-stat",
                "added / removed",
                self.stat,
                controls::setter(&e, |p: &mut CommitPage, v| p.stat = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", commit_reference()))
    }
}

// ── StashRow ────────────────────────────────────────────────────────────

/// `components/StashRow/preview.html`: two stashes, 480px on `layer-01`.
fn stash_reference() -> impl IntoElement {
    on_layer(480.)
        .child(
            StashRow::new("sr-0", 0, "WIP on theme tokens")
                .branch("feat/dark-tokens")
                .files(3)
                .time("1h"),
        )
        .child(
            StashRow::new("sr-1", 1, "Experiment: rounded tags")
                .branch("main")
                .files(1)
                .time("2d"),
        )
}

/// The StashRow overview tile.
pub fn stash_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            StashRow::new("sr-thumb", 0, "WIP on tokens")
                .branch("main")
                .files(3)
                .time("1h"),
        )
        .into_any_element()
}

struct StashPage {
    last: &'static str,
}

/// Build the StashRow page.
pub fn build_stash_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StashPage { last: "—" }).into()
}

impl Render for StashPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let report = |what: &'static str| {
            let e = e.clone();
            move |_: &mut Window, cx: &mut App| {
                e.update(cx, |p, cx| {
                    p.last = what;
                    cx.notify();
                })
            }
        };
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                on_layer(480.).child(
                    StashRow::new("sr-live", 0, "WIP on theme tokens")
                        .branch("feat/dark-tokens")
                        .files(3)
                        .time("1h")
                        .on_apply(report("onApply"))
                        .on_pop(report("onPop"))
                        .on_drop(report("onDrop")),
                ),
            )
            .child(log(format!("last: {}", self.last)));
        Body::new()
            .playground(live)
            .example(controls::block("Reference preview", stash_reference()))
    }
}

// ── CommitComposer ──────────────────────────────────────────────────────

fn composer_files() -> Vec<ChangedFile> {
    vec![
        ChangedFile::new("src/theme/g90.ts", FileStatus::Modified, 42, 3),
        ChangedFile::new("src/theme/tokens.ts", FileStatus::Modified, 6, 1),
        ChangedFile::new("docs/migration.md", FileStatus::Added, 58, 0),
    ]
}

/// The draft the reference's `onGenerate` writes.
const DRAFT: &str = "Add G90 layer-accent tokens\n\nAlso documents the migration.";

/// `components/CommitComposer/preview.html`: 420px wide.
fn composer_reference() -> impl IntoElement {
    div().w_full().max_w(rems(26.25)).child(
        CommitComposer::new("cc-ref", composer_files())
            .branch("feat/dark-tokens")
            .ahead(2)
            .on_generate(|set, w, cx| set(DRAFT.into(), w, cx)),
    )
}

/// The CommitComposer overview tile.
pub fn composer_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(CommitComposer::new("cc-thumb", composer_files().into_iter().take(2)).ahead(2))
        .into_any_element()
}

struct ComposerPage {
    generating: bool,
    ahead: bool,
    last: SharedString,
}

/// Build the CommitComposer page.
pub fn build_commit_composer(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ComposerPage {
        generating: false,
        ahead: true,
        last: "—".into(),
    })
    .into()
}

impl Render for ComposerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let report = |verb: &'static str| {
            let e = e.clone();
            move |msg: SharedString, paths: Vec<SharedString>, _: &mut Window, cx: &mut App| {
                e.update(cx, |p, cx| {
                    let first = msg.lines().next().unwrap_or_default().to_string();
                    p.last = format!("{verb}(\"{first}\", {} paths)", paths.len()).into();
                    cx.notify();
                })
            }
        };
        let live = div()
            .w_full()
            .max_w(rems(26.25))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                CommitComposer::new("cc-live", composer_files())
                    .branch("feat/dark-tokens")
                    .when(self.ahead, |c| c.ahead(2))
                    .generating(self.generating)
                    .on_generate(|set, w, cx| set(DRAFT.into(), w, cx))
                    .on_commit(report("onCommit"))
                    .on_commit_and_push(report("onCommitAndPush")),
            )
            .child(log(self.last.clone()));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "cc-gen",
                "generating",
                self.generating,
                controls::setter(&e, |p: &mut ComposerPage, v| p.generating = v),
            ))
            .child(controls::toggle(
                "cc-ahead",
                "ahead = 2",
                self.ahead,
                controls::setter(&e, |p: &mut ComposerPage, v| p.ahead = v),
            ));
        Body::new()
            .playground(live)
            .hint("Commit needs a staged file and a message; try “Write it for me”")
            .config(config)
            .example(controls::block("Reference preview", composer_reference()))
    }
}

// ── LinkedItemsSection ──────────────────────────────────────────────────

fn linked_items() -> Vec<LinkedItem> {
    vec![
        LinkedItem::pull_request("#482", PullRequestState::Open, "Add G90 theme")
            .checks(ChecksStatus::Running),
        LinkedItem::issue("#77", IssueState::Open, "Dark mode contrast on tags"),
        LinkedItem::pull_request("#470", PullRequestState::Merged, "Token build script")
            .checks(ChecksStatus::Passed),
    ]
}

/// `components/LinkedItemsSection/preview.html`: 380px wide.
fn linked_reference() -> impl IntoElement {
    div()
        .w_full()
        .max_w(rems(23.75))
        .child(LinkedItemsSection::new("li-ref", linked_items()))
}

/// The LinkedItemsSection overview tile.
pub fn linked_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(LinkedItemsSection::new(
            "li-thumb",
            linked_items().into_iter().take(2),
        ))
        .into_any_element()
}

struct LinkedPage {
    items: Vec<LinkedItem>,
    linked: usize,
}

/// Build the LinkedItemsSection page.
pub fn build_linked_items(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| LinkedPage {
        items: linked_items(),
        linked: 0,
    })
    .into()
}

impl Render for LinkedPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (link, unlink) = (e.clone(), e.clone());
        let live = div().w_full().max_w(rems(23.75)).child(
            LinkedItemsSection::new("li-live", self.items.clone())
                .on_link(move |_, cx| {
                    link.update(cx, |p, cx| {
                        p.linked += 1;
                        p.items.push(LinkedItem::issue(
                            format!("#{}", 90 + p.linked),
                            IssueState::Progress,
                            "Linked from the demo",
                        ));
                        cx.notify();
                    })
                })
                .on_unlink(move |id, _, cx| {
                    unlink.update(cx, |p, cx| {
                        p.items.retain(|it| it.id != id);
                        cx.notify();
                    })
                }),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "li-empty",
                "empty",
                self.items.is_empty(),
                controls::setter(&e, |p: &mut LinkedPage, v: bool| {
                    p.items = if v { Vec::new() } else { linked_items() };
                }),
            ));
        Body::new()
            .playground(live)
            .hint("Link adds an item, Unlink removes one")
            .config(config)
            .example(controls::block("Reference preview", linked_reference()))
    }
}
