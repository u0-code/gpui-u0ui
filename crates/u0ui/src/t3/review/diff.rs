//! T3 · Diff & review: [`DiffStat`], [`DiffFileStatus`],
//! [`StartTruncatedPath`], [`DiffFileHeader`], [`DiffView`] and
//! [`DiffFileTree`] (`src/t3/review.jsx`, `src/t3.css` "Diffs").
use crate::{
    components::checkbox::{CheckboxLook, CheckboxLookState, checkbox_box},
    components::{CheckboxState, CopyButton, form},
    icons::{CarbonIcon, Icon},
    motion::{self, Motion},
    t3::craft::{self, focus_handle, focus_visible, mono, press, rem, sans},
    theme::{self, LayerLevel, color},
    tokens::{CarbonTheme, ColorRole},
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, StyledText, TextRun, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// The ink of the `+N` count: `.t3-add { color: var(--support-success) }`,
/// `[data-theme="white"|"g10"] .t3-add { color: var(--tag-color-green) }`
/// (the success green is too light for text on white); a zero is
/// `text-helper` (`.t3-diffstat .t3-zero`).
pub fn added_ink(theme: CarbonTheme, count: usize) -> Hsla {
    if count == 0 {
        return theme::color_in(theme, ColorRole::TextHelper);
    }
    theme::color_in(
        theme,
        match theme {
            CarbonTheme::White | CarbonTheme::G10 => ColorRole::TagColorGreen,
            CarbonTheme::G90 | CarbonTheme::G100 => ColorRole::SupportSuccess,
        },
    )
}

/// The ink of the `−N` count: `.t3-del { color: var(--text-error) }`, a zero
/// `text-helper`.
pub fn removed_ink(theme: CarbonTheme, count: usize) -> Hsla {
    theme::color_in(
        theme,
        if count == 0 {
            ColorRole::TextHelper
        } else {
            ColorRole::TextError
        },
    )
}

/// One block of the diff-stat bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatBlock {
    /// `.t3-add-bg { background: var(--support-success) }`.
    Added,
    /// `.t3-del-bg { background: var(--support-error) }`.
    Removed,
    /// `.t3-diffstat-bar i:not([class]) { background: var(--layer-accent-01) }`.
    Neutral,
}

impl StatBlock {
    /// The block's fill.
    pub fn fill(self) -> Hsla {
        color(match self {
            Self::Added => ColorRole::SupportSuccess,
            Self::Removed => ColorRole::SupportError,
            Self::Neutral => ColorRole::LayerAccent01,
        })
    }
}

/// The five bar blocks (`DiffStat` in `review.jsx`): `min(total, 5)` blocks
/// filled in proportion, at least one green when anything was added and at
/// least one red when anything was removed; the rest neutral.
pub fn stat_blocks(added: usize, removed: usize) -> [StatBlock; 5] {
    let total = added + removed;
    let k = total.min(5);
    let mut g = if total > 0 {
        ((added as f64 / total as f64) * k as f64).round() as usize
    } else {
        0
    };
    if added > 0 && g == 0 {
        g = 1;
    }
    if removed > 0 && k > 0 && g >= k {
        g = k - 1;
    }
    std::array::from_fn(|i| {
        if i < g {
            StatBlock::Added
        } else if i < k {
            StatBlock::Removed
        } else {
            StatBlock::Neutral
        }
    })
}

/// `+added −removed`, optionally with the five-block bar
/// (`.t3-diffstat`: mono 12px, 4px gaps; bar blocks 6×6, 1px apart).
#[derive(IntoElement, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiffStat {
    added: usize,
    removed: usize,
    bar: bool,
    dimmed: bool,
}

impl DiffStat {
    /// Counts of added and removed lines.
    pub fn new(added: usize, removed: usize) -> Self {
        Self {
            added,
            removed,
            bar: false,
            dimmed: false,
        }
    }

    /// Show the proportional bar (`bar`).
    pub fn bar(mut self, bar: bool) -> Self {
        self.bar = bar;
        self
    }

    /// Dim to 55% (`.t3-ftree-file--viewed .t3-diffstat { opacity: 0.55 }`).
    pub fn dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }
}

impl RenderOnce for DiffStat {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let theme = theme::current();
        mono(div(), 12., 400)
            .flex()
            .flex_none()
            .items_center()
            .gap(rem(4.))
            .whitespace_nowrap()
            .when(self.dimmed, |this| this.opacity(0.55))
            .child(
                div()
                    .text_color(added_ink(theme, self.added))
                    .child(format!("+{}", self.added)),
            )
            .child(
                div()
                    .text_color(removed_ink(theme, self.removed))
                    .child(format!("\u{2212}{}", self.removed)),
            )
            .when(self.bar, |this| {
                this.child(div().flex().gap(px(1.)).children(
                    stat_blocks(self.added, self.removed).map(|b| div().size(px(6.)).bg(b.fill())),
                ))
            })
    }
}

/// A file's change status (`DiffFileStatus` `status`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FileStatus {
    /// `A`.
    Added,
    /// `M`, the default.
    #[default]
    Modified,
    /// `D`.
    Deleted,
    /// `R`.
    Renamed,
}

impl FileStatus {
    /// Every status.
    pub const ALL: [FileStatus; 4] = [Self::Added, Self::Modified, Self::Deleted, Self::Renamed];

    /// The status letter (the `index.d.ts` value).
    pub const fn letter(self) -> &'static str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Deleted => "D",
            Self::Renamed => "R",
        }
    }

    /// The reference's accessible name (`FSTAT`).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Modified => "Modified",
            Self::Deleted => "Deleted",
            Self::Renamed => "Renamed",
        }
    }

    /// (fill, ink): `.t3-fstat--A` green, `--M` blue, `--D` red, `--R`
    /// purple tag pairs.
    pub fn colors(self) -> (Hsla, Hsla) {
        use ColorRole::*;
        let (bg, fg) = match self {
            Self::Added => (TagBackgroundGreen, TagColorGreen),
            Self::Modified => (TagBackgroundBlue, TagColorBlue),
            Self::Deleted => (TagBackgroundRed, TagColorRed),
            Self::Renamed => (TagBackgroundPurple, TagColorPurple),
        };
        (color(bg), color(fg))
    }
}

/// The 16px status letter (`.t3-fstat`: mono 11px semibold on a tag pair).
#[derive(IntoElement, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffFileStatus {
    status: FileStatus,
}

impl DiffFileStatus {
    /// A status letter.
    pub fn new(status: FileStatus) -> Self {
        Self { status }
    }
}

impl RenderOnce for DiffFileStatus {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (bg, fg) = self.status.colors();
        let el = mono(div(), 11., 600)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(rem(16.))
            .bg(bg)
            .text_color(fg)
            .child(self.status.letter());
        craft::titled(el, "fstat", self.status.label())
    }
}

/// Split `path` at its last `/`: (directory with the slash, file name).
pub fn split_path(path: &str) -> (&str, &str) {
    match path.rfind('/') {
        Some(i) => (&path[..=i], &path[i + 1..]),
        None => ("", path),
    }
}

/// A path whose directory part truncates from the start while the file name
/// stays whole (`.t3-path`: mono 12px; `.t3-path-dir { direction: rtl;
/// text-overflow: ellipsis }` in `text-secondary`, the file in `text-primary`).
#[derive(IntoElement, Clone, Debug)]
pub struct StartTruncatedPath {
    path: SharedString,
}

impl StartTruncatedPath {
    /// The full path.
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self { path: path.into() }
    }
}

impl RenderOnce for StartTruncatedPath {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (dir, file) = split_path(&self.path);
        let el = mono(div(), 12., 400)
            .flex()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .when(!dir.is_empty(), |this| {
                this.child(
                    div()
                        .min_w_0()
                        .overflow_hidden()
                        // `direction: rtl`: leftover width falls on the start side.
                        .text_right()
                        .text_ellipsis_start()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(SharedString::from(dir.to_string())),
                )
            })
            .child(
                div()
                    .flex_none()
                    .text_color(color(ColorRole::TextPrimary))
                    .child(SharedString::from(file.to_string())),
            );
        craft::titled(el, "path", self.path.clone())
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type BoolHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Texts of a [`DiffFileHeader`] (the reference hard-codes them).
#[derive(Clone, Debug)]
pub struct DiffFileHeaderTexts {
    /// Toggle name while expanded (`Collapse file`).
    pub collapse: SharedString,
    /// Toggle name while collapsed (`Expand file`).
    pub expand: SharedString,
    /// Copy button name (`Copy path`).
    pub copy: SharedString,
    /// Copy feedback (`Copied!`, Carbon `CopyButton`'s default).
    pub copied: SharedString,
    /// The checkbox label (`Viewed`).
    pub viewed: SharedString,
}

impl Default for DiffFileHeaderTexts {
    fn default() -> Self {
        Self {
            collapse: "Collapse file".into(),
            expand: "Expand file".into(),
            copy: "Copy path".into(),
            copied: "Copied!".into(),
            viewed: "Viewed".into(),
        }
    }
}

/// A diff file's sticky header row (`.t3-dfh`): collapse toggle, status,
/// start-truncated path, copy button, `+/−` with bar, "Viewed" checkbox.
///
/// The row is 40px on `layer-01` with a `border-subtle-01` bottom rule. The
/// reference makes it `position: sticky; top: 0; z-index: 1`; GPUI has no
/// sticky positioning, so a scrolling list of files pins it with
/// [`crate::components::sticky::StickyScroll`] (one section per file: this
/// header over its [`DiffView`]).
#[derive(IntoElement)]
pub struct DiffFileHeader {
    id: ElementId,
    path: SharedString,
    status: FileStatus,
    added: usize,
    removed: usize,
    viewed: Option<bool>,
    default_viewed: bool,
    collapsed: bool,
    on_toggle: Option<Handler>,
    on_viewed: Option<BoolHandler>,
    texts: DiffFileHeaderTexts,
}

impl DiffFileHeader {
    /// A header for `path`.
    pub fn new(id: impl Into<ElementId>, path: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            status: FileStatus::Modified,
            added: 0,
            removed: 0,
            viewed: None,
            default_viewed: false,
            collapsed: false,
            on_toggle: None,
            on_viewed: None,
            texts: DiffFileHeaderTexts::default(),
        }
    }

    /// The change status (default `M`).
    pub fn status(mut self, status: FileStatus) -> Self {
        self.status = status;
        self
    }

    /// Added and removed line counts.
    pub fn stat(mut self, added: usize, removed: usize) -> Self {
        self.added = added;
        self.removed = removed;
        self
    }

    /// Controlled "Viewed" state.
    pub fn viewed(mut self, viewed: bool) -> Self {
        self.viewed = Some(viewed);
        self
    }

    /// Initial "Viewed" state when uncontrolled (`defaultChecked={viewed}`).
    pub fn default_viewed(mut self, viewed: bool) -> Self {
        self.default_viewed = viewed;
        self
    }

    /// Whether the file body is collapsed (the chevron turns −90°).
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// The collapse toggle was pressed (`onToggle`).
    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// The "Viewed" checkbox changed (`onViewed`).
    pub fn on_viewed(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_viewed = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: DiffFileHeaderTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// The collapse chevron's rotation: `.t3-rot { transform: rotate(-90deg) }`
/// when collapsed.
pub fn chevron_angle(collapsed: bool) -> f32 {
    if collapsed {
        -std::f32::consts::FRAC_PI_2
    } else {
        0.
    }
}

impl RenderOnce for DiffFileHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let toggle_id: ElementId = (id.clone(), "toggle").into();
        let toggle_focus = focus_handle(&toggle_id, window, cx);
        let toggle_ring = focus_visible(&toggle_focus, window);
        // `.t3-dfh-toggle svg { transition: transform 110ms }`.
        let angle = motion::animate(
            (id.clone(), "chevron"),
            chevron_angle(self.collapsed),
            Motion::TOGGLE,
            window,
            cx,
        );
        let on_toggle = self.on_toggle.clone();
        let toggle = press(toggle_id, &toggle_focus)
            .accessibility_label(if self.collapsed {
                self.texts.expand.clone()
            } else {
                self.texts.collapse.clone()
            })
            .flex_none()
            .justify_center()
            .w(rem(40.))
            .h_full()
            .hover(|s| s.bg(color(ColorRole::LayerHover01)))
            .when_some(on_toggle, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
            .child(
                Icon::carbon(CarbonIcon::ChevronDown)
                    .size_length(rem(16.))
                    .color(color(ColorRole::IconPrimary))
                    .rotate(gpui_kit::radians(angle)),
            )
            .when(toggle_ring, |this| this.child(craft::focus_ring()));

        // `<label class="t3-viewed"><input type="checkbox" defaultChecked …/>Viewed</label>`.
        let viewed_id: ElementId = (id.clone(), "viewed").into();
        let (viewed, store) = form::controlled(
            (id.clone(), "viewed-value"),
            self.viewed,
            self.default_viewed,
            window,
            cx,
        );
        let viewed_focus = focus_handle(&viewed_id, window, cx);
        let viewed_ring = focus_visible(&viewed_focus, window);
        let look = CheckboxLook::of(CheckboxLookState {
            value: if viewed {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            },
            ..Default::default()
        });
        let on_viewed = self.on_viewed.clone();
        let viewed_box = press(viewed_id, &viewed_focus)
            .role(gpui_kit::Role::CheckBox)
            .accessibility_label(SharedString::from(format!(
                "{} {}",
                self.texts.viewed, self.path
            )))
            .flex_none()
            .gap(rem(4.))
            .h(rem(24.))
            .px(rem(8.))
            .on_click(move |_, window, cx| {
                let next = !viewed;
                store.update(cx, |v, cx| {
                    *v = next;
                    cx.notify();
                });
                if let Some(f) = &on_viewed {
                    f(next, window, cx);
                }
            })
            .child(checkbox_box(
                look,
                if viewed {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                },
                viewed_ring,
            ))
            .child(craft::text(
                self.texts.viewed.clone(),
                12.,
                ColorRole::TextSecondary,
            ));

        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .items_center()
            .gap(rem(8.))
            .h(rem(40.))
            .pr(rem(12.))
            .bg(color(ColorRole::Layer01))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(toggle)
            .child(DiffFileStatus::new(self.status))
            .child(StartTruncatedPath::new(self.path.clone()))
            // `.t3-copy.cds--copy-btn { width: var(--size-sm); height: var(--size-sm) }`.
            .child(
                CopyButton::new((id.clone(), "copy"), self.path.clone())
                    .label(self.texts.copy.clone())
                    .feedback(self.texts.copied.clone())
                    .size(32.)
                    .layer(LayerLevel::One),
            )
            .child(div().flex_1())
            .child(DiffStat::new(self.added, self.removed).bar(true))
            .child(viewed_box)
    }
}

/// A diff line's kind (`t`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LineKind {
    /// Unchanged context.
    #[default]
    Context,
    /// `+`.
    Added,
    /// `-`.
    Removed,
}

/// A run of a diff line's code: plain, or a changed word
/// (`{ w }`: `.g-word-add` / `.g-word-del`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    /// Unchanged text.
    Text(SharedString),
    /// A changed word.
    Word(SharedString),
}

/// One unified-diff line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffLine {
    /// Old line number (`a`).
    pub old: Option<u32>,
    /// New line number (`b`).
    pub new: Option<u32>,
    /// `t`.
    pub kind: LineKind,
    /// `c`: plain text or segments.
    pub code: Vec<Segment>,
}

impl DiffLine {
    /// An unchanged line present on both sides.
    pub fn context(old: u32, new: u32, code: impl Into<SharedString>) -> Self {
        Self {
            old: Some(old),
            new: Some(new),
            kind: LineKind::Context,
            code: vec![Segment::Text(code.into())],
        }
    }

    /// An added line.
    pub fn added(new: u32, code: impl Into<SharedString>) -> Self {
        Self {
            old: None,
            new: Some(new),
            kind: LineKind::Added,
            code: vec![Segment::Text(code.into())],
        }
    }

    /// A removed line.
    pub fn removed(old: u32, code: impl Into<SharedString>) -> Self {
        Self {
            old: Some(old),
            new: None,
            kind: LineKind::Removed,
            code: vec![Segment::Text(code.into())],
        }
    }

    /// Replace the code with word-level segments.
    pub fn segments(mut self, segments: impl IntoIterator<Item = Segment>) -> Self {
        self.code = segments.into_iter().collect();
        self
    }
}

/// A hunk: its `@@` header and lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffHunk {
    /// `header`.
    pub header: SharedString,
    /// `lines`.
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// A hunk.
    pub fn new(header: impl Into<SharedString>, lines: impl IntoIterator<Item = DiffLine>) -> Self {
        Self {
            header: header.into(),
            lines: lines.into_iter().collect(),
        }
    }
}

/// (row background, number/marker ink) of a line (`.t3-dl--add`:
/// `diff-added-background`, `support-success`; `.t3-dl--del`:
/// `diff-removed-background`, `text-error`; context: none, numbers
/// `text-helper`, marker `text-secondary`). `.t3-dl:hover` paints
/// `background-hover` over any of them (it is the more specific rule).
pub fn line_look(kind: LineKind, hovered: bool) -> (Option<Hsla>, Option<Hsla>) {
    let bg = match kind {
        _ if hovered => Some(color(ColorRole::BackgroundHover)),
        LineKind::Context => None,
        LineKind::Added => Some(color(ColorRole::DiffAddedBackground)),
        LineKind::Removed => Some(color(ColorRole::DiffRemovedBackground)),
    };
    let ink = match kind {
        LineKind::Context => None,
        LineKind::Added => Some(color(ColorRole::SupportSuccess)),
        LineKind::Removed => Some(color(ColorRole::TextError)),
    };
    (bg, ink)
}

type CommentHandler = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// A unified diff (`.t3-diff`: mono 12/20 on `background`): hunk headers,
/// old/new numbers, `+`/`−` markers, added/removed backgrounds and word
/// highlights. Hovering a line shows the 18px "comment" button over the
/// gutter; annotations render under their line.
#[derive(IntoElement)]
pub struct DiffView {
    id: ElementId,
    hunks: Vec<DiffHunk>,
    annotations: Vec<((usize, usize), AnyElement)>,
    on_comment: Option<CommentHandler>,
    comment_label: SharedString,
}

impl DiffView {
    /// A diff view.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            hunks: Vec::new(),
            annotations: Vec::new(),
            on_comment: None,
            comment_label: "Comment on this line".into(),
        }
    }

    /// The hunks.
    pub fn hunks(mut self, hunks: impl IntoIterator<Item = DiffHunk>) -> Self {
        self.hunks = hunks.into_iter().collect();
        self
    }

    /// An annotation under line `line` of hunk `hunk` (`annotations['h:l']`).
    pub fn annotation(mut self, hunk: usize, line: usize, element: impl IntoElement) -> Self {
        self.annotations
            .push(((hunk, line), element.into_any_element()));
        self
    }

    /// The gutter "+" was pressed on (hunk, line).
    pub fn on_comment(mut self, f: impl Fn(usize, usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_comment = Some(Rc::new(f));
        self
    }

    /// The comment button's accessible name (`Comment on this line`).
    pub fn comment_label(mut self, label: impl Into<SharedString>) -> Self {
        self.comment_label = label.into();
        self
    }
}

fn code_text(line: &DiffLine) -> StyledText {
    let font = crate::typography::font(crate::tokens::FontFamilyKey::Mono, 400);
    let base = color(ColorRole::TextPrimary);
    let word_bg = color(match line.kind {
        LineKind::Removed => ColorRole::DiffRemovedWordBackground,
        _ => ColorRole::DiffAddedWordBackground,
    });
    let mut text = String::new();
    let mut runs = Vec::new();
    for seg in &line.code {
        let (s, word) = match seg {
            Segment::Text(s) => (s, false),
            Segment::Word(s) => (s, true),
        };
        if s.is_empty() {
            continue;
        }
        text.push_str(s);
        // `.g-word-del { text-decoration: line-through; text-decoration-color:
        // color-mix(in srgb, currentColor 40%, transparent) }`.
        let strike =
            (word && line.kind == LineKind::Removed).then(|| gpui_kit::StrikethroughStyle {
                thickness: px(1.),
                color: Some(base.opacity(0.4)),
            });
        runs.push(TextRun {
            len: s.len(),
            font: font.clone(),
            color: base,
            background_color: word.then_some(word_bg),
            underline: None,
            strikethrough: strike,
        });
    }
    if text.is_empty() {
        text.push(' ');
        runs.push(TextRun {
            len: 1,
            font,
            color: base,
            background_color: None,
            underline: None,
            strikethrough: None,
        });
    }
    StyledText::new(text).with_runs(runs)
}

impl RenderOnce for DiffView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let hovered = window
            .use_keyed_state((id.clone(), "hover"), cx, |_, _| None::<(usize, usize)>)
            .clone();
        let current = *hovered.read(cx);
        let mut annotations = self.annotations;
        let mut rows: Vec<AnyElement> = Vec::new();
        for (hi, hunk) in self.hunks.iter().enumerate() {
            // `.t3-hunk { padding: 4px 16px; background: layer-accent-01; color: text-secondary }`.
            rows.push(
                div()
                    .px(rem(16.))
                    .py(rem(4.))
                    .bg(color(ColorRole::LayerAccent01))
                    .text_color(color(ColorRole::TextSecondary))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(hunk.header.clone())
                    .into_any_element(),
            );
            for (li, line) in hunk.lines.iter().enumerate() {
                let key = (hi, li);
                let is_hovered = current == Some(key);
                let (bg, ink) = line_look(line.kind, is_hovered);
                let number_ink = ink.unwrap_or(color(ColorRole::TextHelper));
                let marker_ink = ink.unwrap_or(color(ColorRole::TextSecondary));
                let number = |n: Option<u32>| {
                    div()
                        .flex_none()
                        .w(rem(48.))
                        .pr(rem(8.))
                        .text_right()
                        .text_color(number_ink)
                        .child(n.map(|n| n.to_string()).unwrap_or_default())
                };
                let marker = match line.kind {
                    LineKind::Context => " ",
                    LineKind::Added => "+",
                    LineKind::Removed => "-",
                };
                let add_id: ElementId =
                    (id.clone(), SharedString::from(format!("add-{hi}-{li}"))).into();
                let add_focus = focus_handle(&add_id, window, cx);
                let add_ring = focus_visible(&add_focus, window);
                let show_add = is_hovered || add_ring;
                let on_comment = self.on_comment.clone();
                let add = show_add.then(|| {
                    // `.t3-dl-add { position: absolute; left: 5.25rem; top: 1px;
                    // 18×18; background: button-primary; color: text-on-color }`.
                    press(add_id, &add_focus)
                        .accessibility_label(self.comment_label.clone())
                        .absolute()
                        .left(rem(84.))
                        .top(px(1.))
                        .size(px(18.))
                        .justify_center()
                        .bg(color(ColorRole::ButtonPrimary))
                        .when_some(on_comment, |this, f| {
                            this.on_click(move |_, w, cx| f(hi, li, w, cx))
                        })
                        .child(
                            Icon::carbon(CarbonIcon::Add)
                                .size_length(px(12.))
                                .color(color(ColorRole::TextOnColor)),
                        )
                        .when(add_ring, |this| this.child(craft::focus_ring()))
                });
                rows.push(
                    div()
                        .id((id.clone(), SharedString::from(format!("line-{hi}-{li}"))))
                        .relative()
                        .flex()
                        .when_some(bg, |this, bg| this.bg(bg))
                        .on_hover({
                            let hovered = hovered.clone();
                            move |h, _, cx| {
                                hovered.update(cx, |v, cx| {
                                    if *h {
                                        *v = Some(key);
                                    } else if *v == Some(key) {
                                        *v = None;
                                    }
                                    cx.notify();
                                })
                            }
                        })
                        .child(number(line.old))
                        .child(number(line.new))
                        .child(
                            div()
                                .flex_none()
                                .w(rem(20.))
                                .text_center()
                                .text_color(marker_ink)
                                .child(marker),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(code_text(line)),
                        )
                        .children(add)
                        .into_any_element(),
                );
                if let Some(pos) = annotations.iter().position(|(k, _)| *k == key) {
                    let (_, element) = annotations.remove(pos);
                    // `.t3-dl-anno { padding: 8px 16px 8px 7.25rem; background:
                    // layer-01; font-family: sans; border-top/bottom: subtle-01 }`.
                    rows.push(
                        sans(div(), 12., 400)
                            .line_height(rem(20.))
                            .pt(rem(8.))
                            .pb(rem(8.))
                            .pr(rem(16.))
                            .pl(rem(116.))
                            .bg(color(ColorRole::Layer01))
                            .border_t_1()
                            .border_b_1()
                            .border_color(color(ColorRole::BorderSubtle01))
                            .child(element)
                            .into_any_element(),
                    );
                }
            }
        }
        mono(div(), 12., 400)
            .id(id)
            .line_height(rem(20.))
            .flex()
            .flex_col()
            .bg(color(ColorRole::Background))
            .text_color(color(ColorRole::TextPrimary))
            .children(rows)
    }
}

/// One changed file in a [`DiffFileTree`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeFile {
    /// `path`.
    pub path: SharedString,
    /// `status`.
    pub status: FileStatus,
    /// `added`.
    pub added: usize,
    /// `removed`.
    pub removed: usize,
    /// `viewed`.
    pub viewed: bool,
}

impl TreeFile {
    /// A file entry.
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
            viewed: false,
        }
    }

    /// Mark it viewed.
    pub fn viewed(mut self, viewed: bool) -> Self {
        self.viewed = viewed;
        self
    }
}

/// Group files by directory in first-seen order (`dirs[d]`, `'.'` for the
/// root).
pub fn group_by_dir(files: &[TreeFile]) -> Vec<(String, Vec<&TreeFile>)> {
    let mut groups: Vec<(String, Vec<&TreeFile>)> = Vec::new();
    for f in files {
        let dir = match f.path.rfind('/') {
            Some(0) | None => ".".to_string(),
            Some(i) => f.path[..i].to_string(),
        };
        match groups.iter_mut().find(|(d, _)| *d == dir) {
            Some((_, list)) => list.push(f),
            None => groups.push((dir, vec![f])),
        }
    }
    groups
}

type SelectHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The changed-files tree (`.t3-ftree`): folder rows (mono 12px,
/// `text-secondary`), file rows 32px tall at a 32px indent with status,
/// mono 13px name, viewed check and `+/−`. The selected file paints
/// `layer-selected-01` with a 3px `border-interactive` start bar.
#[derive(IntoElement)]
pub struct DiffFileTree {
    id: ElementId,
    files: Vec<TreeFile>,
    selected: Option<Option<SharedString>>,
    default_selected: Option<SharedString>,
    on_select: Option<SelectHandler>,
    viewed_label: SharedString,
}

impl DiffFileTree {
    /// A tree.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            files: Vec::new(),
            selected: None,
            default_selected: None,
            on_select: None,
            viewed_label: "Viewed".into(),
        }
    }

    /// The files.
    pub fn files(mut self, files: impl IntoIterator<Item = TreeFile>) -> Self {
        self.files = files.into_iter().collect();
        self
    }

    /// Controlled selection.
    pub fn selected(mut self, path: Option<SharedString>) -> Self {
        self.selected = Some(path);
        self
    }

    /// Initial selection when uncontrolled.
    pub fn default_selected(mut self, path: impl Into<SharedString>) -> Self {
        self.default_selected = Some(path.into());
        self
    }

    /// A file was chosen (click, Enter or Space).
    pub fn on_select(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// The viewed check's accessible name (`Viewed`).
    pub fn viewed_label(mut self, label: impl Into<SharedString>) -> Self {
        self.viewed_label = label.into();
        self
    }
}

impl RenderOnce for DiffFileTree {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (selected, store) = form::controlled(
            (id.clone(), "selected"),
            self.selected.clone(),
            self.default_selected.clone(),
            window,
            cx,
        );
        let mut children: Vec<AnyElement> = Vec::new();
        for (dir, files) in group_by_dir(&self.files) {
            children.push(
                mono(div(), 12., 400)
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .h(rem(32.))
                    .px(rem(12.))
                    .text_color(color(ColorRole::TextSecondary))
                    .whitespace_nowrap()
                    .child(
                        Icon::carbon(CarbonIcon::FolderOpen)
                            .size_length(rem(16.))
                            .color(color(ColorRole::IconSecondary)),
                    )
                    .child(SharedString::from(dir))
                    .into_any_element(),
            );
            for f in files {
                let row_id: ElementId = (id.clone(), f.path.clone()).into();
                let focus = focus_handle(&row_id, window, cx);
                let ring = focus_visible(&focus, window);
                let on = selected.as_ref() == Some(&f.path);
                let path = f.path.clone();
                let store = store.clone();
                let on_select = self.on_select.clone();
                let (_, name) = split_path(&f.path);
                children.push(
                    press(row_id, &focus)
                        .role(gpui_kit::Role::TreeItem)
                        .flex()
                        .items_center()
                        .gap(rem(8.))
                        .h(rem(32.))
                        .pl(rem(32.))
                        .pr(rem(12.))
                        .text_color(color(if f.viewed {
                            ColorRole::TextSecondary
                        } else {
                            ColorRole::TextPrimary
                        }))
                        .when(on, |this| this.bg(color(ColorRole::LayerSelected01)))
                        .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                        .on_click(move |_, window, cx| {
                            store.update(cx, |v, cx| {
                                *v = Some(path.clone());
                                cx.notify();
                            });
                            if let Some(f) = &on_select {
                                f(path.clone(), window, cx);
                            }
                        })
                        .when(on, |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .bottom_0()
                                    .w(px(3.))
                                    .bg(color(ColorRole::BorderInteractive)),
                            )
                        })
                        .child(DiffFileStatus::new(f.status))
                        .child(
                            mono(div(), 13., 400)
                                .min_w_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(SharedString::from(name.to_string())),
                        )
                        .child(div().flex_1())
                        .when(f.viewed, |this| {
                            this.child(
                                div().flex_none().flex().child(
                                    Icon::carbon(CarbonIcon::CheckmarkOutline)
                                        .size_length(rem(14.))
                                        .color(color(ColorRole::IconSecondary)),
                                ),
                            )
                        })
                        .child(DiffStat::new(f.added, f.removed).dimmed(f.viewed))
                        .when(ring, |this| this.child(craft::focus_ring()))
                        .into_any_element(),
                );
            }
        }
        sans(div(), 14., 400)
            .id(id)
            .flex()
            .flex_col()
            .line_height(relative(craft::BODY_LINE))
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn stat_blocks_fill_in_proportion() {
        use StatBlock::*;
        // 312/14 → five blocks, one red kept for the removals.
        assert_eq!(stat_blocks(312, 14), [Added, Added, Added, Added, Removed]);
        // 24/9 → round(24/33·5) = 4 green, 1 red.
        assert_eq!(stat_blocks(24, 9), [Added, Added, Added, Added, Removed]);
        // 217/0 → all green.
        assert_eq!(stat_blocks(217, 0), [Added; 5]);
        // 0/88 → all red.
        assert_eq!(stat_blocks(0, 88), [Removed; 5]);
        // 3/1 → four blocks: round(0.75·4) = 3 green, 1 red, 1 neutral.
        assert_eq!(stat_blocks(3, 1), [Added, Added, Added, Removed, Neutral]);
        // 1/30 keeps one green.
        assert_eq!(
            stat_blocks(1, 30),
            [Added, Removed, Removed, Removed, Removed]
        );
        assert_eq!(stat_blocks(0, 0), [Neutral; 5]);
    }

    #[test]
    fn counts_follow_theme_rules() {
        // `[data-theme="white"] .t3-add { color: var(--tag-color-green) }`.
        assert_eq!(hex(added_ink(CarbonTheme::White, 3)), "#0e6027");
        // `.t3-add { color: var(--support-success) }` in G100.
        assert_eq!(hex(added_ink(CarbonTheme::G100, 3)), "#42be65");
        // `.t3-diffstat .t3-zero { color: var(--text-helper) }`.
        assert_eq!(hex(added_ink(CarbonTheme::White, 0)), "#6f6f6f");
        assert_eq!(hex(removed_ink(CarbonTheme::White, 2)), "#da1e28");
    }

    #[test]
    fn paths_split_at_the_last_slash() {
        assert_eq!(
            split_path("src/tokens/g90.json"),
            ("src/tokens/", "g90.json")
        );
        assert_eq!(split_path("README.md"), ("", "README.md"));
    }

    #[test]
    fn files_group_by_directory_in_order() {
        let files = [
            TreeFile::new("src/components/DatePicker.tsx", FileStatus::Modified, 24, 9),
            TreeFile::new("src/components/Calendar.tsx", FileStatus::Modified, 3, 1),
            TreeFile::new("src/tokens/g90.json", FileStatus::Added, 217, 0),
            TreeFile::new("README.md", FileStatus::Modified, 1, 0),
        ];
        let groups = group_by_dir(&files);
        let dirs: Vec<_> = groups.iter().map(|(d, f)| (d.as_str(), f.len())).collect();
        assert_eq!(dirs, [("src/components", 2), ("src/tokens", 1), (".", 1)]);
    }

    #[test]
    fn line_backgrounds_and_hover() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, ink) = line_look(LineKind::Added, false);
        assert_eq!(
            bg.map(hex),
            Some(hex(color(ColorRole::DiffAddedBackground)))
        );
        assert_eq!(ink.map(hex).as_deref(), Some("#24a148"));
        // `.t3-dl:hover` (0,2,0) beats `.t3-dl--add` (0,1,0).
        let (bg, _) = line_look(LineKind::Added, true);
        assert_eq!(bg.map(hex), Some(hex(color(ColorRole::BackgroundHover))));
        let (bg, ink) = line_look(LineKind::Context, false);
        assert!(bg.is_none() && ink.is_none());
    }

    #[test]
    fn status_letters_and_chevron() {
        assert_eq!(FileStatus::Renamed.letter(), "R");
        assert_eq!(FileStatus::Deleted.label(), "Deleted");
        assert_eq!(chevron_angle(false), 0.);
        assert!((chevron_angle(true) + std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }
}
