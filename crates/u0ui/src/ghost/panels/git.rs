//! Git history and changes in the right column (`CommitRow`, `StashRow`,
//! `CommitComposer`, `LinkedItemsSection` in `src/ghost/panels.jsx`;
//! `.g-commit*`, `.g-ref*`, `.g-stash*`, `.g-cc*`, `.g-check`, `.g-linked*`
//! in `src/ghost.css`, with the `gemini-B` craft pass in `src/t3.css`).
//!
//! - **CommitRow**: a full-width button: the graph lane (a `charts-1` line
//!   with a ringed node, filled on the current commit; `start` / `end` cut
//!   the line at the node), the message with its refs (`HEAD` green, local
//!   branches blue, `origin/*` purple), short SHA · author · time, and the
//!   diff stat. Hover `layer-hover-01`, current `state-current-background`.
//! - **StashRow**: `stash@{n}` and message, branch · files · time, then
//!   Apply, Pop and Drop (callers should confirm a drop).
//! - **CommitComposer**: the Changes list with a check per file (all staged
//!   at first), a commit message box with "Write it for me" (the host drafts
//!   it through [`CommitComposer::on_generate`]), Commit N files and Commit
//!   & push (disabled until a file is staged and the message is not blank),
//!   and how far the branch is ahead.
//! - **LinkedItemsSection**: the pull requests and issues linked to a
//!   session, each with its state glyph, checks and Unlink; an empty line
//!   when there are none.
use super::head::{Handler, ghost_icon, ghost_text};
use crate::ghost::shell::{option_focus, toggles_checkbox};
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    density,
    ghost::primitives::parts::{focus_ring, focus_visible, keyed_focus},
    icons::{CarbonIcon, Icon},
    t3::{
        base::{Spin, visually_hidden},
        craft::{mono, rem, sans},
        review::{
            ChecksGlyph, ChecksStatus, DiffFileStatus, DiffStat, FileStatus, IssueState,
            IssueStateGlyph, PullRequestState, PullRequestStateGlyph,
        },
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, CursorStyle, Div, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _, Size,
        input::{Textarea as KitTextarea, TextareaState},
    },
    div,
    prelude::FluentBuilder as _,
    px, relative,
};
use std::collections::BTreeSet;
use std::rc::Rc;

/// A keyed hover flag and its `on_hover` updater.
fn hover_state(
    id: ElementId,
    window: &mut Window,
    cx: &mut App,
) -> (bool, impl Fn(&bool, &mut Window, &mut App) + 'static) {
    let store = window.use_keyed_state(id, cx, |_, _| false);
    let over = *store.read(cx);
    (over, move |h: &bool, _: &mut Window, cx: &mut App| {
        store.update(cx, |v, cx| {
            if *v != *h {
                *v = *h;
                cx.notify();
            }
        })
    })
}

/// `.g-commit-meta { display: flex; gap: var(--spacing-03); font-size:
/// 0.75rem; color: var(--text-secondary); white-space: nowrap; overflow:
/// hidden }`.
fn meta_line(runs: impl IntoIterator<Item = AnyElement>) -> Div {
    sans(div(), 12., 400)
        .flex()
        .gap(tokens::SPACING_03)
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_color(color(ColorRole::TextSecondary))
        .children(runs)
}

/// A `.g-mono` run (IBM Plex Mono 12px).
fn mono_run(text: impl Into<SharedString>) -> AnyElement {
    mono(div(), 12., 400)
        .flex_none()
        .child(text.into())
        .into_any_element()
}

/// A plain run.
fn run(text: impl Into<SharedString>) -> AnyElement {
    div().flex_none().child(text.into()).into_any_element()
}

/// `.g-commit-main, .g-stash-main { display: flex; flex: 1; min-width: 0;
/// flex-direction: column; gap: 2px; padding: var(--spacing-02) 0 }`.
fn main_column() -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap(px(2.))
        .py(tokens::SPACING_02)
}

// ── CommitRow ───────────────────────────────────────────────────────────

/// The commit graph lane (`graph`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CommitGraph {
    /// `start`: the newest commit; the line starts at the node.
    Start,
    /// `line`: a commit in the middle (the default).
    #[default]
    Line,
    /// `end`: the oldest commit; the line ends at the node.
    End,
}

impl CommitGraph {
    /// Every lane shape.
    pub const ALL: [Self; 3] = [Self::Start, Self::Line, Self::End];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Line => "line",
            Self::End => "end",
        }
    }

    /// Which halves of the lane the line covers (top, bottom):
    /// `.g-commit-graph--start::before { top: 50% }`, `--end::before { bottom: 50% }`.
    pub const fn halves(self) -> (bool, bool) {
        match self {
            Self::Start => (false, true),
            Self::Line => (true, true),
            Self::End => (true, false),
        }
    }
}

/// A ref chip's kind (`.g-ref`, `--remote`, `--head`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RefKind {
    /// `HEAD`: green.
    Head,
    /// A local branch or tag: blue.
    Local,
    /// `origin/*`: purple.
    Remote,
}

impl RefKind {
    /// `r === 'HEAD'` → head, `r.startsWith('origin/')` → remote, else local.
    pub fn of(name: &str) -> Self {
        if name == "HEAD" {
            Self::Head
        } else if name.starts_with("origin/") {
            Self::Remote
        } else {
            Self::Local
        }
    }

    /// (fill, ink): `.g-ref { background: var(--tag-background-blue); color:
    /// var(--tag-color-blue) }`, `--remote` purple, `--head` green.
    pub fn colors(self) -> (Hsla, Hsla) {
        use ColorRole::*;
        let (bg, fg) = match self {
            Self::Head => (TagBackgroundGreen, TagColorGreen),
            Self::Local => (TagBackgroundBlue, TagColorBlue),
            Self::Remote => (TagBackgroundPurple, TagColorPurple),
        };
        (color(bg), color(fg))
    }
}

/// `.g-ref { flex: none; padding: 0 var(--spacing-02); font-family:
/// var(--font-mono); font-size: 0.6875rem; line-height: 1rem }`.
fn ref_chip(name: SharedString) -> AnyElement {
    let (bg, fg) = RefKind::of(&name).colors();
    mono(div(), 11., 400)
        .flex_none()
        .line_height(rem(16.))
        .px(tokens::SPACING_02)
        .bg(bg)
        .text_color(fg)
        .child(name)
        .into_any_element()
}

/// The row's background: `layer-01`, hover `layer-hover-01`, current
/// `state-current-background` (`.g-commit--current` comes later in the
/// cascade than `:hover`, so it wins).
pub fn commit_background(current: bool, hovered: bool) -> Hsla {
    color(if current {
        ColorRole::StateCurrentBackground
    } else if hovered {
        ColorRole::LayerHover01
    } else {
        ColorRole::Layer01
    })
}

/// One commit in the history. See the module docs.
#[derive(IntoElement)]
pub struct CommitRow {
    id: ElementId,
    sha: SharedString,
    message: SharedString,
    author: SharedString,
    time: SharedString,
    refs: Vec<SharedString>,
    stat: Option<(usize, usize)>,
    current: bool,
    graph: CommitGraph,
    on_click: Option<Handler>,
}

impl CommitRow {
    /// A commit `sha` with its `message`.
    pub fn new(
        id: impl Into<ElementId>,
        sha: impl Into<SharedString>,
        message: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            sha: sha.into(),
            message: message.into(),
            author: SharedString::default(),
            time: SharedString::default(),
            refs: Vec::new(),
            stat: None,
            current: false,
            graph: CommitGraph::Line,
            on_click: None,
        }
    }

    /// `author`.
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = author.into();
        self
    }

    /// `time`.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = time.into();
        self
    }

    /// Branch, tag and remote refs pointing here (`refs`).
    pub fn refs(mut self, refs: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.refs = refs.into_iter().map(Into::into).collect();
        self
    }

    /// Lines added and removed (`added`, `removed`).
    pub fn stat(mut self, added: usize, removed: usize) -> Self {
        self.stat = Some((added, removed));
        self
    }

    /// The checked-out commit (`current`).
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// The lane shape (`graph`).
    pub fn graph(mut self, graph: CommitGraph) -> Self {
        self.graph = graph;
        self
    }

    /// Activation (`onClick`).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// `.g-commit-graph { position: relative; align-self: stretch; width:
/// 1.5rem }` with its 2px `charts-1` line and 10px node.
fn graph_lane(graph: CommitGraph, current: bool) -> Div {
    let line = color(ColorRole::Charts1);
    let (top, bottom) = graph.halves();
    let half = |top_half: bool| {
        div()
            .absolute()
            .left(relative(0.5))
            .ml(px(-1.))
            .w(px(2.))
            .h(relative(0.5))
            .bg(line)
            .when(top_half, |d| d.top_0())
            .when(!top_half, |d| d.bottom_0())
    };
    div()
        .relative()
        .flex_none()
        .self_stretch()
        .w(rem(24.))
        .when(top, |d| d.child(half(true)))
        .when(bottom, |d| d.child(half(false)))
        .child(
            // `.g-commit-graph i { width: 10px; height: 10px; margin: -5px 0 0
            // -5px; border: 2px solid var(--charts-1); border-radius: 50%;
            // background: var(--layer-01) }`; current fills it.
            div()
                .absolute()
                .left(relative(0.5))
                .top(relative(0.5))
                .ml(px(-5.))
                .mt(px(-5.))
                .size(px(10.))
                .rounded_full()
                .border_2()
                .border_color(line)
                .bg(if current {
                    line
                } else {
                    color(ColorRole::Layer01)
                }),
        )
}

impl RenderOnce for CommitRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let (over, on_hover) = hover_state((id.clone(), "hover").into(), window, cx);
        let short: String = self.sha.chars().take(7).collect();
        let on_click = self.on_click.clone();
        // `.g-commit { display: flex; align-items: center; gap:
        // var(--spacing-03); min-height: var(--density-row-lg); padding: 0
        // var(--spacing-04) 0 var(--spacing-02) }` (gemini-B: the lane starts
        // on the 16px panel edge).
        sans(gpui_kit::base::Button::new(id), 14., 400)
            .track_focus(&focus)
            .relative()
            .flex()
            .items_center()
            .w_full()
            .gap(tokens::SPACING_03)
            .min_h(density::current().row_lg())
            .pl(tokens::SPACING_02)
            .pr(tokens::SPACING_04)
            .bg(commit_background(self.current, over))
            .text_color(color(ColorRole::TextPrimary))
            .cursor(CursorStyle::PointingHand)
            .on_hover(on_hover)
            .on_click(move |_, w, cx| {
                if let Some(f) = &on_click {
                    f(w, cx);
                }
            })
            .child(graph_lane(self.graph, self.current))
            .child(
                main_column()
                    .child(
                        // `.g-commit-msg { display: flex; align-items: center; gap:
                        // var(--spacing-03); font-size: 0.875rem; overflow: hidden;
                        // white-space: nowrap }`.
                        div()
                            .flex()
                            .items_center()
                            .gap(tokens::SPACING_03)
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(div().min_w_0().truncate().child(self.message.clone()))
                            .children(self.refs.iter().cloned().map(ref_chip)),
                    )
                    .child(meta_line([
                        mono_run(short),
                        run(self.author.clone()),
                        run(self.time.clone()),
                    ])),
            )
            .when_some(self.stat, |b, (a, r)| b.child(DiffStat::new(a, r)))
            .when(ring, |b| b.child(focus_ring()))
    }
}

// ── StashRow ────────────────────────────────────────────────────────────

/// The stash row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct StashRowTexts {
    /// `"{n} files"`.
    pub files: Rc<dyn Fn(usize) -> SharedString>,
    /// `"Apply"`.
    pub apply: SharedString,
    /// `"Pop"`.
    pub pop: SharedString,
    /// `"Drop stash"`.
    pub drop: SharedString,
}

impl Default for StashRowTexts {
    fn default() -> Self {
        Self {
            files: Rc::new(|n| format!("{n} files").into()),
            apply: "Apply".into(),
            pop: "Pop".into(),
            drop: "Drop stash".into(),
        }
    }
}

/// One stash entry. See the module docs.
#[derive(IntoElement)]
pub struct StashRow {
    id: ElementId,
    index: usize,
    message: SharedString,
    branch: SharedString,
    time: SharedString,
    files: usize,
    on_apply: Option<Handler>,
    on_pop: Option<Handler>,
    on_drop: Option<Handler>,
    texts: StashRowTexts,
}

impl StashRow {
    /// Stash `stash@{index}` with its `message`.
    pub fn new(id: impl Into<ElementId>, index: usize, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            index,
            message: message.into(),
            branch: SharedString::default(),
            time: SharedString::default(),
            files: 0,
            on_apply: None,
            on_pop: None,
            on_drop: None,
            texts: StashRowTexts::default(),
        }
    }

    /// The branch it was made on (`branch`).
    pub fn branch(mut self, branch: impl Into<SharedString>) -> Self {
        self.branch = branch.into();
        self
    }

    /// How many files it holds (`files`).
    pub fn files(mut self, files: usize) -> Self {
        self.files = files;
        self
    }

    /// When it was made (`time`).
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = time.into();
        self
    }

    /// Apply pressed (`onApply`).
    pub fn on_apply(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_apply = Some(Rc::new(f));
        self
    }

    /// Pop pressed (`onPop`).
    pub fn on_pop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_pop = Some(Rc::new(f));
        self
    }

    /// Drop pressed (`onDrop`); confirm before dropping.
    pub fn on_drop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_drop = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: StashRowTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for StashRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // `.g-stash { display: flex; align-items: center; gap: var(--spacing-03);
        // min-height: var(--density-row-lg); padding: 0 var(--spacing-02) 0
        // var(--spacing-05); background: var(--layer-01); border-bottom: 1px
        // solid var(--border-subtle-01) }` (gemini-B: 16px start).
        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .items_center()
            .w_full()
            .gap(tokens::SPACING_03)
            .min_h(density::current().row_lg())
            .pl(tokens::SPACING_05)
            .pr(tokens::SPACING_02)
            .bg(color(ColorRole::Layer01))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextPrimary))
            .child(
                Icon::carbon(CarbonIcon::Archive)
                    .size_length(rem(16.))
                    .color(color(ColorRole::IconSecondary))
                    .flex_none(),
            )
            .child(
                main_column()
                    .child(
                        // `.g-stash-title { overflow: hidden; white-space: nowrap;
                        // text-overflow: ellipsis }`.
                        div()
                            .flex()
                            .items_center()
                            .gap(tokens::SPACING_02)
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(
                                mono(div(), 12., 400)
                                    .flex_none()
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(format!("stash@{{{}}}", self.index)),
                            )
                            .child(div().min_w_0().truncate().child(self.message.clone())),
                    )
                    .child(meta_line([
                        mono_run(self.branch.clone()),
                        run((self.texts.files)(self.files)),
                        run(self.time.clone()),
                    ])),
            )
            // `.g-stash > .cds--btn { flex: none }`.
            .child(ghost_text(
                (id.clone(), "apply"),
                self.texts.apply.clone(),
                self.on_apply,
            ))
            .child(ghost_text(
                (id.clone(), "pop"),
                self.texts.pop.clone(),
                self.on_pop,
            ))
            .child(ghost_icon(
                (id, "drop"),
                self.texts.drop.clone(),
                CarbonIcon::TrashCan,
                false,
                self.on_drop,
            ))
    }
}

// ── CommitComposer ──────────────────────────────────────────────────────

/// A changed file in the composer (`files[i]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedFile {
    /// `path`.
    pub path: SharedString,
    /// `status` (`A`, `M`, `D`, `R`).
    pub status: FileStatus,
    /// `added`.
    pub added: usize,
    /// `removed`.
    pub removed: usize,
}

impl ChangedFile {
    /// A changed file.
    pub fn new(
        path: impl Into<SharedString>,
        status: FileStatus,
        added: usize,
        removed: usize,
    ) -> Self {
        Self {
            path: path.into(),
            status,
            added,
            removed,
        }
    }
}

/// Whether Commit is enabled: `!sel.size || !msg.trim()` disables it.
pub fn can_commit(staged: usize, message: &str) -> bool {
    staged > 0 && !message.trim().is_empty()
}

/// The composer's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct CommitComposerTexts {
    /// `"Changes"`.
    pub changes: SharedString,
    /// `"{staged} of {total} staged"`.
    pub staged: Rc<dyn Fn(usize, usize) -> SharedString>,
    /// The message box's placeholder and name: `"Commit message"`.
    pub message: SharedString,
    /// `"Write it for me"`.
    pub generate: SharedString,
    /// While drafting: `"Writing…"`.
    pub generating: SharedString,
    /// `"Commit {n} file"` / `"Commit {n} files"`.
    pub commit: Rc<dyn Fn(usize) -> SharedString>,
    /// `"Commit & push"`.
    pub commit_and_push: SharedString,
    /// `"{n} ahead of origin"`.
    pub ahead: Rc<dyn Fn(usize) -> SharedString>,
}

impl Default for CommitComposerTexts {
    fn default() -> Self {
        Self {
            changes: "Changes".into(),
            staged: Rc::new(|s, t| format!("{s} of {t} staged").into()),
            message: "Commit message".into(),
            generate: "Write it for me".into(),
            generating: "Writing…".into(),
            commit: Rc::new(|n| format!("Commit {n} file{}", if n == 1 { "" } else { "s" }).into()),
            commit_and_push: "Commit & push".into(),
            ahead: Rc::new(|n| format!("{n} ahead of origin").into()),
        }
    }
}

/// Sets the commit message (handed to [`CommitComposer::on_generate`]).
pub type MessageSetter = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type GenerateHandler = Rc<dyn Fn(MessageSetter, &mut Window, &mut App)>;
type CommitHandler = Rc<dyn Fn(SharedString, Vec<SharedString>, &mut Window, &mut App)>;

struct ComposerModel {
    staged: BTreeSet<SharedString>,
    message: Entity<TextareaState>,
}

/// The commit panel. See the module docs.
#[derive(IntoElement)]
pub struct CommitComposer {
    id: ElementId,
    files: Vec<ChangedFile>,
    branch: SharedString,
    ahead: usize,
    generating: bool,
    on_generate: Option<GenerateHandler>,
    on_commit: Option<CommitHandler>,
    on_commit_and_push: Option<CommitHandler>,
    texts: CommitComposerTexts,
}

impl CommitComposer {
    /// A composer over the changed `files`, all staged at first.
    pub fn new(id: impl Into<ElementId>, files: impl IntoIterator<Item = ChangedFile>) -> Self {
        Self {
            id: id.into(),
            files: files.into_iter().collect(),
            branch: "main".into(),
            ahead: 0,
            generating: false,
            on_generate: None,
            on_commit: None,
            on_commit_and_push: None,
            texts: CommitComposerTexts::default(),
        }
    }

    /// The branch (`branch`, default `main`).
    pub fn branch(mut self, branch: impl Into<SharedString>) -> Self {
        self.branch = branch.into();
        self
    }

    /// Commits ahead of origin (`ahead`).
    pub fn ahead(mut self, ahead: usize) -> Self {
        self.ahead = ahead;
        self
    }

    /// The host is drafting a message (`generating`): "Writing…" with a spinner.
    pub fn generating(mut self, generating: bool) -> Self {
        self.generating = generating;
        self
    }

    /// "Write it for me" (`onGenerate(setMessage)`): draft a message and
    /// hand it to the setter.
    pub fn on_generate(
        mut self,
        f: impl Fn(MessageSetter, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_generate = Some(Rc::new(f));
        self
    }

    /// Commit (`onCommit(message, paths)`).
    pub fn on_commit(
        mut self,
        f: impl Fn(SharedString, Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_commit = Some(Rc::new(f));
        self
    }

    /// Commit & push (`onCommitAndPush(message, paths)`).
    pub fn on_commit_and_push(
        mut self,
        f: impl Fn(SharedString, Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_commit_and_push = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: CommitComposerTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.g-check`: a 16px box, 1px `icon-primary` border, radius 2; checked it
/// fills `icon-primary` with an `icon-inverse` checkmark; a keyboard-focused
/// input rings it 1px outside (`outline-offset: 1px`).
pub fn check_box(checked: bool, ring: bool) -> Div {
    let ink = color(ColorRole::IconPrimary);
    div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(rem(16.))
        .border_1()
        .border_color(ink)
        .rounded(tokens::BORDER_RADIUS_02.rems())
        .when(checked, |d| {
            d.bg(ink).child(
                Icon::carbon(CarbonIcon::Checkmark)
                    .size_length(rem(12.))
                    .color(color(ColorRole::IconInverse)),
            )
        })
        .when(ring, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(-4.))
                    .left(px(-4.))
                    .right(px(-4.))
                    .bottom(px(-4.))
                    .border_2()
                    .border_color(color(ColorRole::Focus)),
            )
        })
}

impl RenderOnce for CommitComposer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let paths: Vec<SharedString> = self.files.iter().map(|f| f.path.clone()).collect();
        let model = window.use_keyed_state((id.clone(), "model"), cx, {
            let paths = paths.clone();
            move |window, cx| ComposerModel {
                staged: paths.into_iter().collect(),
                message: cx.new(|cx| TextareaState::new(window, cx)),
            }
        });
        let (staged, message) = {
            let m = model.read(cx);
            (m.staged.clone(), m.message.clone())
        };
        let text = message.read(cx).value().to_string();
        let enabled = can_commit(staged.len(), &text);
        let selected: Vec<SharedString> = paths
            .iter()
            .filter(|p| staged.contains(*p))
            .cloned()
            .collect();
        let secondary = color(ColorRole::TextSecondary);

        // `.g-cc-head { display: flex; align-items: center; gap:
        // var(--spacing-03); height: var(--density-row-lg); padding: 0
        // var(--spacing-04); font-size: 0.875rem }`.
        let head = sans(div(), 14., 400)
            .flex()
            .items_center()
            .gap(tokens::SPACING_03)
            .h(density::current().row_lg())
            .px(tokens::SPACING_04)
            .text_color(color(ColorRole::TextPrimary))
            // `.g-cc-head b` has no weight rule: Carbon's `b { font: inherit }`
            // keeps it 400.
            .child(
                sans(div(), 14., 400)
                    .flex_none()
                    .child(self.texts.changes.clone()),
            )
            .child(
                sans(div(), 12., 400)
                    .flex_none()
                    .text_color(secondary)
                    .child((self.texts.staged)(staged.len(), self.files.len())),
            )
            .child(div().flex_1())
            .child(
                mono(div(), 12., 400)
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .min_w_0()
                    .text_color(secondary)
                    .child(
                        Icon::carbon(CarbonIcon::Branch)
                            .size_length(rem(12.))
                            .color(secondary)
                            .flex_none(),
                    )
                    .child(div().min_w_0().truncate().child(self.branch.clone())),
            );

        let rows = self.files.iter().enumerate().map(|(i, f)| {
            let rid: ElementId = (id.clone(), SharedString::from(format!("file-{i}"))).into();
            let focus = keyed_focus((rid.clone(), "focus"), window, cx);
            let ring = focus_visible(&focus, window);
            let (over, on_hover) = hover_state((rid.clone(), "hover").into(), window, cx);
            let on = staged.contains(&f.path);
            let toggle = {
                let model = model.clone();
                let path = f.path.clone();
                move |_: &mut Window, cx: &mut App| {
                    model.update(cx, |m, cx| {
                        if !m.staged.remove(&path) {
                            m.staged.insert(path.clone());
                        }
                        cx.notify();
                    })
                }
            };
            // `.g-cc-files label { display: flex; align-items: center; gap:
            // var(--spacing-03); min-height: var(--density-row-md); padding: 0
            // var(--spacing-04); font-size: 0.875rem; cursor: pointer }`,
            // hover `layer-hover-01`.
            sans(div(), 14., 400)
                .id(rid)
                // A tab stop toggled by Space (once: GPUI already clicks a
                // focused element on Space), like the reference's checkbox.
                .track_focus(&option_focus(focus))
                .flex()
                .items_center()
                .gap(tokens::SPACING_03)
                .min_h(density::current().row())
                .px(tokens::SPACING_04)
                .when(over, |d| d.bg(color(ColorRole::LayerHover01)))
                .cursor(CursorStyle::PointingHand)
                .on_hover(on_hover)
                .on_click(move |e, w, cx| {
                    if toggles_checkbox(e) {
                        toggle(w, cx)
                    }
                })
                .child(check_box(on, ring))
                .child(DiffFileStatus::new(f.status))
                .child(
                    // `.g-cc-path { flex: 1; direction: rtl; text-overflow:
                    // ellipsis }`: truncated from the start.
                    mono(div(), 12., 400)
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis_start()
                        .child(f.path.clone()),
                )
                .child(DiffStat::new(f.added, f.removed))
                .into_any_element()
        });
        let rows: Vec<AnyElement> = rows.collect();

        // The message box: `.g-cc-msg { position: relative; padding:
        // var(--spacing-03) var(--spacing-04); border-top: 1px solid
        // var(--border-subtle-01) }`.
        let field_focus = message.read(cx).focus_handle(cx);
        let focused = field_focus.is_focused(window);
        let placeholder = text.is_empty().then(|| {
            sans(div(), 14., 400)
                .absolute()
                .top(rem(8.))
                .left(rem(12.))
                .text_color(color(ColorRole::TextPlaceholder))
                .child(self.texts.message.clone())
        });
        let (gen_over, gen_hover) = hover_state((id.clone(), "gen-hover").into(), window, cx);
        let gen_focus = keyed_focus((id.clone(), "gen-focus"), window, cx);
        let gen_ring = focus_visible(&gen_focus, window);
        let generating = self.generating;
        let link = color(ColorRole::LinkPrimary);
        let on_generate = self.on_generate.clone();
        let setter: MessageSetter = {
            let message = message.clone();
            Rc::new(move |text, window, cx| {
                message.update(cx, |s, cx| s.set_value(text, window, cx));
            })
        };
        let generate = sans(gpui_kit::base::Button::new((id.clone(), "gen")), 12., 400)
            .track_focus(&gen_focus)
            .disabled(generating)
            .absolute()
            // `.g-cc-gen { right: calc(var(--spacing-04) + 4px); bottom:
            // calc(var(--spacing-03) + 6px); height: 1.5rem; padding: 0
            // var(--spacing-03); color: var(--link-primary) }`.
            .right(rem(16.))
            .bottom(rem(14.))
            .flex()
            .items_center()
            .gap(tokens::SPACING_02)
            .h(rem(24.))
            .px(tokens::SPACING_03)
            .text_color(link)
            .when(gen_over && !generating, |b| {
                b.bg(color(ColorRole::LayerHover01))
            })
            .cursor(if generating {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            })
            .on_hover(gen_hover)
            .on_click(move |_, w, cx| {
                if let Some(f) = &on_generate {
                    f(setter.clone(), w, cx);
                }
            })
            .map(|b| {
                if generating {
                    b.child(Spin::new((id.clone(), "spin")).size(14.).color(link))
                        .child(self.texts.generating.clone())
                } else {
                    b.child(
                        Icon::carbon(CarbonIcon::MagicWand)
                            .size_length(rem(14.))
                            .color(link),
                    )
                    .child(self.texts.generate.clone())
                }
            })
            .when(gen_ring, |b| b.child(focus_ring()));
        let focus_box = field_focus.clone();
        // `.g-cc-msg textarea { padding: var(--spacing-03) var(--spacing-04)
        // 2rem; border-bottom: 1px solid var(--border-strong-01); background:
        // var(--field-02); font-size: 0.875rem; outline: 2px solid transparent }`,
        // `:focus { outline-color: var(--focus) }`; three rows.
        let field = div()
            .id((id.clone(), "msg-box"))
            .relative()
            .w_full()
            .h(rem(3. * 18. + 8. + 32. + 1.))
            .pt(rem(8. - 2.))
            .pb(rem(32.))
            .pl(rem(12. - 4.))
            .pr(rem(12. - 4.))
            .bg(color(ColorRole::Field02))
            .border_b_1()
            .border_color(color(ColorRole::BorderStrong01))
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus_box.focus(window, cx);
            })
            .child(
                KitTextarea::new(&message)
                    .appearance(false)
                    .with_size(Size::XSmall)
                    .aria_label(self.texts.message.clone())
                    .h(rem(3. * 18. + 4.))
                    .w_full()
                    .text_size(rem(14.))
                    .line_height(rem(18.))
                    .text_color(color(ColorRole::TextPrimary)),
            )
            .children(placeholder)
            .when(focused, |d| d.child(focus_ring()));
        let msg = div()
            .relative()
            .py(tokens::SPACING_03)
            .px(tokens::SPACING_04)
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(field)
            .child(generate);

        let commit =
            |key: &'static str, label: SharedString, variant, handler: Option<CommitHandler>| {
                let message = message.clone();
                let selected = selected.clone();
                Button::new((id.clone(), key), label)
                    .variant(variant)
                    .size(ButtonSize::Sm)
                    .disabled(!enabled)
                    .on_click(move |_, w, cx| {
                        let text: SharedString = message.read(cx).value().to_string().into();
                        if let Some(f) = &handler {
                            f(text, selected.clone(), w, cx);
                        }
                    })
            };
        // `.g-cc-actions { display: flex; flex-wrap: wrap; align-items: center;
        // gap: var(--spacing-03); padding: 0 var(--spacing-04) var(--spacing-04) }`.
        let actions = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_03)
            .px(tokens::SPACING_04)
            .pb(tokens::SPACING_04)
            .child(commit(
                "commit",
                (self.texts.commit)(staged.len()),
                ButtonVariant::Primary,
                self.on_commit.clone(),
            ))
            .child(commit(
                "push",
                self.texts.commit_and_push.clone(),
                ButtonVariant::Secondary,
                self.on_commit_and_push.clone(),
            ))
            .when(self.ahead > 0, |d| {
                d.child(
                    sans(div(), 12., 400)
                        .text_color(secondary)
                        .child((self.texts.ahead)(self.ahead)),
                )
            });

        div()
            .id(id)
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .child(head)
            .child(
                // `.g-cc-files { list-style: none; border-top: 1px solid
                // var(--border-subtle-01) }`.
                div()
                    .flex()
                    .flex_col()
                    .border_t_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .children(rows),
            )
            .child(msg)
            .child(actions)
    }
}

// ── LinkedItemsSection ──────────────────────────────────────────────────

/// A linked item's kind and state (`type`, `state`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkedKind {
    /// `type: 'pr'`.
    PullRequest(PullRequestState),
    /// `type: 'issue'`.
    Issue(IssueState),
}

/// One linked pull request or issue (`items[i]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedItem {
    /// `id` (e.g. `#482`).
    pub id: SharedString,
    /// `type` and `state`.
    pub kind: LinkedKind,
    /// `title`.
    pub title: SharedString,
    /// `checks`.
    pub checks: Option<ChecksStatus>,
}

impl LinkedItem {
    /// A linked pull request.
    pub fn pull_request(
        id: impl Into<SharedString>,
        state: PullRequestState,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: LinkedKind::PullRequest(state),
            title: title.into(),
            checks: None,
        }
    }

    /// A linked issue.
    pub fn issue(
        id: impl Into<SharedString>,
        state: IssueState,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: LinkedKind::Issue(state),
            title: title.into(),
            checks: None,
        }
    }

    /// Its checks (`checks`).
    pub fn checks(mut self, checks: ChecksStatus) -> Self {
        self.checks = Some(checks);
        self
    }
}

/// The section's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct LinkedItemsTexts {
    /// `"Linked"`.
    pub title: SharedString,
    /// `"Link"`.
    pub link: SharedString,
    /// `"Unlink {id}"`.
    pub unlink: Rc<dyn Fn(&str) -> SharedString>,
    /// `"No pull requests or issues linked to this session."`.
    pub empty: SharedString,
}

impl Default for LinkedItemsTexts {
    fn default() -> Self {
        Self {
            title: "Linked".into(),
            link: "Link".into(),
            unlink: Rc::new(|id| format!("Unlink {id}").into()),
            empty: "No pull requests or issues linked to this session.".into(),
        }
    }
}

type UnlinkHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The linked pull requests and issues. See the module docs.
#[derive(IntoElement)]
pub struct LinkedItemsSection {
    id: ElementId,
    items: Vec<LinkedItem>,
    on_link: Option<Handler>,
    on_unlink: Option<UnlinkHandler>,
    texts: LinkedItemsTexts,
}

impl LinkedItemsSection {
    /// A section listing `items`.
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = LinkedItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            on_link: None,
            on_unlink: None,
            texts: LinkedItemsTexts::default(),
        }
    }

    /// "Link" pressed (`onLink`).
    pub fn on_link(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_link = Some(Rc::new(f));
        self
    }

    /// An item's Unlink pressed (`onUnlink(id)`).
    pub fn on_unlink(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_unlink = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: LinkedItemsTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.g-linked-head, .g-linked-row { display: flex; align-items: center; gap:
/// var(--spacing-03); min-height: var(--density-row-lg); padding: 0
/// var(--spacing-02) 0 var(--spacing-04); font-size: 0.875rem }`.
fn linked_line() -> Div {
    sans(div(), 14., 400)
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .min_h(density::current().row_lg())
        .pl(tokens::SPACING_04)
        .pr(tokens::SPACING_02)
        .text_color(color(ColorRole::TextPrimary))
}

impl RenderOnce for LinkedItemsSection {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let secondary = color(ColorRole::TextSecondary);
        let head = linked_line()
            // `.g-linked-head::before`: a 14px `expand_more` chevron in
            // `icon-secondary`, lining up with ThreadDetailsSection heads (d-fix).
            .child(
                Icon::named("expand_more")
                    .size_length(rem(14.))
                    .color(color(ColorRole::IconSecondary))
                    .flex_none(),
            )
            // `.g-linked-head b { font-weight: inherit }`.
            .child(div().flex_none().child(self.texts.title.clone()))
            .child(
                sans(div(), 11., 400)
                    .flex_none()
                    .line_height(rem(16.))
                    .px(tokens::SPACING_02)
                    .rounded(tokens::BORDER_RADIUS_16.rems())
                    .bg(color(ColorRole::LayerAccent01))
                    .text_color(secondary)
                    .child(self.items.len().to_string()),
            )
            .child(div().flex_1())
            .child(
                Button::new((id.clone(), "link"), self.texts.link.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(CarbonIcon::Add)
                    .when_some(self.on_link.clone(), |b, f| {
                        b.on_click(move |_, w, cx| f(w, cx))
                    }),
            );
        let rows: Vec<AnyElement> = if self.items.is_empty() {
            // `.g-empty-line { padding: var(--spacing-04) var(--spacing-05);
            // font-size: 0.75rem; color: var(--text-secondary) }`.
            vec![
                sans(div(), 12., 400)
                    .py(tokens::SPACING_04)
                    .px(tokens::SPACING_05)
                    .text_color(secondary)
                    .child(self.texts.empty.clone())
                    .into_any_element(),
            ]
        } else {
            self.items
                .iter()
                .enumerate()
                .map(|(i, it)| {
                    let glyph = match it.kind {
                        LinkedKind::PullRequest(s) => {
                            PullRequestStateGlyph::new(s).into_any_element()
                        }
                        LinkedKind::Issue(s) => IssueStateGlyph::new(s).into_any_element(),
                    };
                    let unlink = self.on_unlink.clone();
                    let item_id = it.id.clone();
                    linked_line()
                        .border_t_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .child(glyph)
                        .child(
                            // `.g-linked-title { flex: 1; overflow: hidden;
                            // white-space: nowrap; text-overflow: ellipsis }`.
                            div()
                                .flex()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    mono(div(), 12., 400)
                                        .flex_none()
                                        .text_color(secondary)
                                        .child(it.id.clone()),
                                )
                                .child(div().min_w_0().truncate().child(it.title.clone())),
                        )
                        .when_some(it.checks, |d, c| d.child(ChecksGlyph::new(c)))
                        .child(ghost_icon(
                            (id.clone(), SharedString::from(format!("unlink-{i}"))),
                            (self.texts.unlink)(&it.id),
                            CarbonIcon::Unlink,
                            false,
                            Some(Rc::new(move |w, cx| {
                                if let Some(f) = &unlink {
                                    f(item_id.clone(), w, cx);
                                }
                            })),
                        ))
                        .into_any_element()
                })
                .collect()
        };
        div()
            .id(id)
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .child(visually_hidden(self.texts.title.clone()))
            .child(head)
            .children(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn refs_and_lanes() {
        assert_eq!(RefKind::of("HEAD"), RefKind::Head);
        assert_eq!(RefKind::of("origin/main"), RefKind::Remote);
        assert_eq!(RefKind::of("feat/dark-tokens"), RefKind::Local);
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        // `.g-ref { background: tag-background-blue; color: tag-color-blue }`.
        assert_eq!(
            RefKind::Local.colors(),
            (c(ColorRole::TagBackgroundBlue), c(ColorRole::TagColorBlue))
        );
        assert_eq!(
            RefKind::Remote.colors().0,
            c(ColorRole::TagBackgroundPurple)
        );
        assert_eq!(RefKind::Head.colors().1, c(ColorRole::TagColorGreen));
        assert_eq!(CommitGraph::Start.halves(), (false, true));
        assert_eq!(CommitGraph::End.halves(), (true, false));
        assert_eq!(CommitGraph::Line.halves(), (true, true));
    }

    #[test]
    fn commit_backgrounds() {
        crate::theme::tests_set(CarbonTheme::G100);
        let c = |r| color_in(CarbonTheme::G100, r);
        assert_eq!(commit_background(false, false), c(ColorRole::Layer01));
        assert_eq!(commit_background(false, true), c(ColorRole::LayerHover01));
        assert_eq!(
            commit_background(true, true),
            c(ColorRole::StateCurrentBackground)
        );
    }

    #[test]
    fn commit_needs_files_and_a_message() {
        assert!(can_commit(2, "Add tokens"));
        assert!(!can_commit(0, "Add tokens"));
        assert!(!can_commit(2, "  \n "));
        let t = CommitComposerTexts::default();
        assert_eq!((t.commit)(1), "Commit 1 file");
        assert_eq!((t.commit)(3), "Commit 3 files");
        assert_eq!((t.staged)(2, 3), "2 of 3 staged");
    }
}
