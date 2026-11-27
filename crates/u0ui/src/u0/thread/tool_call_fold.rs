//! ToolCallFold: one tool call in a transcript, folded to a row
//! (`ToolCallFold`, `.g-tool` in `src/ghost.css`).
//!
//! The head is a button: a turning chevron, the tool's icon (a spinner while
//! running), the verb, the target in mono, an optional `+a −r` diff stat,
//! the failure mark and the elapsed time. Open, it shows the locations as
//! mono chips, then the caller's children. A 1px `border-subtle-01` rule on
//! the left joins consecutive calls into one group.
//!
//! ```ignore
//! ToolCallFold::new("call-2", ToolKind::Search)
//!     .title("\"layer-accent\" in src/**")
//!     .locations(["src/theme/tokens.ts:41", "src/theme/g100.ts:12"])
//!     .duration(2)
//!     .default_open(true)
//!     .child("2 matches")
//! ```
use super::parts::{self, rem};
use crate::{
    density,
    icons::{CarbonIcon, Icon},
    t3::{
        base::{Spin, fmt_elapsed, visually_hidden},
        review::DiffStat,
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};

/// What the tool did (`kind`), with its icon and verb (`TOOL` in
/// `workspace.jsx`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToolKind {
    /// `read`: Document, "Read" (the default and the fallback).
    #[default]
    Read,
    /// `edit`: Edit, "Edited".
    Edit,
    /// `create`: Edit, "Created".
    Create,
    /// `execute`: Terminal, "Ran".
    Execute,
    /// `search`: Search, "Searched".
    Search,
    /// `fetch`: Earth, "Fetched".
    Fetch,
    /// `think`: Idea, "Thought".
    Think,
    /// `delete`: TrashCan, "Deleted".
    Delete,
    /// `move`: ArrowRight, "Moved".
    Move,
}

impl ToolKind {
    /// Every kind, in the reference's order.
    pub const ALL: [ToolKind; 9] = [
        Self::Read,
        Self::Edit,
        Self::Create,
        Self::Execute,
        Self::Search,
        Self::Fetch,
        Self::Think,
        Self::Delete,
        Self::Move,
    ];

    /// The `kind` string.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Edit => "edit",
            Self::Create => "create",
            Self::Execute => "execute",
            Self::Search => "search",
            Self::Fetch => "fetch",
            Self::Think => "think",
            Self::Delete => "delete",
            Self::Move => "move",
        }
    }

    /// The kind's icon.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Read => CarbonIcon::Document,
            Self::Edit | Self::Create => CarbonIcon::Edit,
            Self::Execute => CarbonIcon::Terminal,
            Self::Search => CarbonIcon::Search,
            Self::Fetch => CarbonIcon::Earth,
            Self::Think => CarbonIcon::Idea,
            Self::Delete => CarbonIcon::TrashCan,
            Self::Move => CarbonIcon::ArrowRight,
        }
    }

    /// The reference's verb (callers may replace it with
    /// [`ToolCallFold::verb`]).
    pub const fn verb(self) -> &'static str {
        match self {
            Self::Read => "Read",
            Self::Edit => "Edited",
            Self::Create => "Created",
            Self::Execute => "Ran",
            Self::Search => "Searched",
            Self::Fetch => "Fetched",
            Self::Think => "Thought",
            Self::Delete => "Deleted",
            Self::Move => "Moved",
        }
    }
}

/// The call's state (`status`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToolStatus {
    /// `running`: a spinner replaces the icon.
    Running,
    /// `done`, the default.
    #[default]
    Done,
    /// `failed`: the title turns `text-error` and an error mark follows.
    Failed,
}

impl ToolStatus {
    /// Every status.
    pub const ALL: [ToolStatus; 3] = [Self::Running, Self::Done, Self::Failed];

    /// The `status` string.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// The colors of a [`ToolCallFold`] head.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolCallLook {
    /// The head's ink (verb, chevron row).
    pub ink: Hsla,
    /// The head's background.
    pub background: Hsla,
    /// The title's ink.
    pub title: Hsla,
}

impl ToolCallLook {
    /// The head look for `status` while `hovered`.
    pub fn of(status: ToolStatus, hovered: bool) -> Self {
        Self {
            // `.g-tool-head { color: text-secondary }`, `:hover { color: text-primary }`.
            ink: parts::head_ink(hovered),
            // `.g-tool-head { background: none }`, `:hover { background: layer-hover-01 }`.
            background: if hovered {
                color(ColorRole::LayerHover01)
            } else {
                gpui_kit::transparent_black()
            },
            title: color(if status == ToolStatus::Failed {
                // `.g-tool--failed .g-tool-title { color: text-error }`.
                ColorRole::TextError
            } else {
                // `.g-tool-title { color: text-primary }`.
                ColorRole::TextPrimary
            }),
        }
    }
}

/// [`ToolCallFold`] geometry in px.
pub mod geometry {
    /// `.g-tool { margin-left: var(--spacing-03) }`.
    pub const INDENT: f32 = 8.;
    /// `.g-tool { border-left: 1px solid }`.
    pub const RULE: f32 = 1.;
    /// `.g-tool-head { gap: var(--spacing-03); padding: 0 var(--spacing-03) }`.
    pub const HEAD_GAP: f32 = 8.;
    /// `.g-tool-head { font-size: 0.8125rem }`, `.g-tool-body` too.
    pub const FONT: f32 = 13.;
    /// `I.ChevronRight size={14}`.
    pub const CHEVRON: f32 = 14.;
    /// `size: 16, className: 'g-tool-icon'`.
    pub const ICON: f32 = 16.;
    /// `.g-tool-body { padding: spacing-02 spacing-03 spacing-03 spacing-07 }`.
    pub const BODY_PADDING: (f32, f32, f32, f32) = (4., 8., 8., 32.);
    /// `.g-tool-loc { line-height: 1.25rem; padding: 0 var(--spacing-02) }`.
    pub const LOC_LINE: f32 = 20.;
}

/// The texts of a [`ToolCallFold`] besides the verb.
#[derive(Clone, Debug)]
pub struct ToolCallFoldTexts {
    /// The hidden word after the error mark (`<VH>failed</VH>`).
    pub failed: SharedString,
}

impl Default for ToolCallFoldTexts {
    fn default() -> Self {
        Self {
            failed: "failed".into(),
        }
    }
}

/// One folded tool call. See the module docs.
#[derive(IntoElement)]
pub struct ToolCallFold {
    id: ElementId,
    kind: ToolKind,
    verb: Option<SharedString>,
    title: SharedString,
    locations: Vec<SharedString>,
    duration: Option<u64>,
    status: ToolStatus,
    default_open: bool,
    added: Option<usize>,
    removed: Option<usize>,
    children: Vec<AnyElement>,
    texts: ToolCallFoldTexts,
}

impl ToolCallFold {
    /// A closed, finished call of `kind`.
    pub fn new(id: impl Into<ElementId>, kind: ToolKind) -> Self {
        Self {
            id: id.into(),
            kind,
            verb: None,
            title: SharedString::default(),
            locations: Vec::new(),
            duration: None,
            status: ToolStatus::Done,
            default_open: false,
            added: None,
            removed: None,
            children: Vec::new(),
            texts: ToolCallFoldTexts::default(),
        }
    }

    /// `title`: the file, command or query (mono, ellipsized).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// Replace the kind's verb (e.g. a translation).
    pub fn verb(mut self, verb: impl Into<SharedString>) -> Self {
        self.verb = Some(verb.into());
        self
    }

    /// `locations`: shown as chips when open.
    pub fn locations<S: Into<SharedString>>(
        mut self,
        locations: impl IntoIterator<Item = S>,
    ) -> Self {
        self.locations = locations.into_iter().map(Into::into).collect();
        self
    }

    /// `duration` in seconds (`fmtElapsed`).
    pub fn duration(mut self, seconds: u64) -> Self {
        self.duration = Some(seconds);
        self
    }

    /// `status` (default done).
    pub fn status(mut self, status: ToolStatus) -> Self {
        self.status = status;
        self
    }

    /// `defaultOpen`.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// `added`: lines added (shows the diff stat).
    pub fn added(mut self, added: usize) -> Self {
        self.added = Some(added);
        self
    }

    /// `removed`: lines removed (shows the diff stat).
    pub fn removed(mut self, removed: usize) -> Self {
        self.removed = Some(removed);
        self
    }

    /// The hidden texts.
    pub fn texts(mut self, texts: ToolCallFoldTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl ParentElement for ToolCallFold {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ToolCallFold {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let open_state = parts::open_state((id.clone(), "open"), self.default_open, window, cx);
        let open = *open_state.read(cx);
        let (focus, ring) = parts::focus((id.clone(), "focus"), window, cx);
        let (hovered, hover) = parts::hover((id.clone(), "hover"), window, cx);
        let look = ToolCallLook::of(self.status, hovered);
        let chevron = parts::chevron_right(id.clone(), open, geometry::CHEVRON, window, cx);
        let glyph = if self.status == ToolStatus::Running {
            Spin::new((id.clone(), "spin")).into_any_element()
        } else {
            // `.g-tool-icon { fill: var(--icon-secondary); flex: none }`.
            Icon::carbon(self.kind.icon())
                .size_length(rem(geometry::ICON))
                .color(color(ColorRole::IconSecondary))
                .into_any_element()
        };
        let verb = self
            .verb
            .unwrap_or_else(|| SharedString::new_static(self.kind.verb()));
        let has_stat = self.added.is_some() || self.removed.is_some();
        let head = gpui_kit::base::Button::new((id.clone(), "head"))
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .w_full()
            .min_h(density::current().row_sm())
            .px(rem(geometry::HEAD_GAP))
            .gap(rem(geometry::HEAD_GAP))
            .bg(look.background)
            .text_color(look.ink)
            .text_size(rem(geometry::FONT))
            .line_height(relative(crate::t3::craft::BODY_LINE))
            .cursor_pointer()
            .on_hover(parts::track_hover(hover))
            .on_click({
                let open_state = open_state.clone();
                move |_, _, cx| parts::toggle(&open_state, cx)
            })
            .child(chevron)
            .child(glyph)
            // `.g-tool-verb { flex: none }`.
            .child(div().flex_none().whitespace_nowrap().child(verb))
            // `.g-tool-title { flex: 1; min-width: 0; overflow: hidden; white-space: nowrap;
            // text-overflow: ellipsis; color: text-primary; font-family: mono; font-size: 0.75rem }`.
            .child(
                parts::mono(div())
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(look.title)
                    .child(self.title),
            )
            // `.g-tool-head .t3-diffstat { margin-left: auto }` (`t3.css`).
            .when(has_stat, |b| {
                b.child(DiffStat::new(
                    self.added.unwrap_or(0),
                    self.removed.unwrap_or(0),
                ))
            })
            .when(self.status == ToolStatus::Failed, |b| {
                // `I.ErrorFilled size={14} className="t3-err"`: `fill: support-error`.
                b.child(
                    Icon::carbon(CarbonIcon::ErrorFilled)
                        .size_length(rem(14.))
                        .color(color(ColorRole::SupportError)),
                )
                .child(visually_hidden(self.texts.failed.clone()))
            })
            .when_some(self.duration, |b, secs| {
                // `.g-tool-dur { font-size: 0.75rem; color: text-helper; tabular-nums }`.
                b.child(
                    div()
                        .flex_none()
                        .text_size(rem(12.))
                        .text_color(color(ColorRole::TextHelper))
                        .whitespace_nowrap()
                        .child(fmt_elapsed(secs)),
                )
            })
            .when(ring, |b| b.child(parts::ring()));
        let (pt, pr, pb, pl) = geometry::BODY_PADDING;
        let body = open.then(|| {
            div()
                .pt(rem(pt))
                .pr(rem(pr))
                .pb(rem(pb))
                .pl(rem(pl))
                .text_size(rem(geometry::FONT))
                .text_color(color(ColorRole::TextPrimary))
                .when(!self.locations.is_empty(), |this| {
                    // `.g-tool-locs { display: flex; flex-wrap: wrap; gap: spacing-02;
                    // margin-bottom: spacing-03 }`.
                    this.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(tokens::SPACING_02)
                            .mb(tokens::SPACING_03)
                            .children(self.locations.into_iter().map(|loc| {
                                // `.g-tool-loc { padding: 0 spacing-02; background:
                                // layer-accent-01; color: text-secondary; line-height: 1.25rem }`.
                                parts::mono(div())
                                    .px(tokens::SPACING_02)
                                    .bg(color(ColorRole::LayerAccent01))
                                    .text_color(color(ColorRole::TextSecondary))
                                    .line_height(rem(geometry::LOC_LINE))
                                    .whitespace_nowrap()
                                    .child(loc)
                            })),
                    )
                })
                .children(self.children)
        });
        // `.g-tool { border-left: 1px solid border-subtle-01; margin-left: spacing-03 }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .ml(rem(geometry::INDENT))
            .border_l(px(geometry::RULE))
            .border_color(color(ColorRole::BorderSubtle01))
            .child(head)
            .children(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn kinds_map_to_the_reference_table() {
        assert_eq!(ToolKind::default(), ToolKind::Read);
        assert_eq!(ToolKind::Execute.verb(), "Ran");
        assert_eq!(ToolKind::Execute.icon(), CarbonIcon::Terminal);
        assert_eq!(ToolKind::Create.icon(), ToolKind::Edit.icon());
        assert_eq!(ToolKind::Fetch.icon(), CarbonIcon::Earth);
        assert_eq!(ToolKind::Move.verb(), "Moved");
        let names: Vec<_> = ToolKind::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(names.len(), 9);
        assert!(names.contains(&"delete"));
        assert_eq!(ToolStatus::default(), ToolStatus::Done);
    }

    #[test]
    fn failed_titles_turn_error_and_hover_lifts_the_head() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = ToolCallLook::of(ToolStatus::Done, false);
        assert_eq!(rest.ink, color(ColorRole::TextSecondary));
        assert_eq!(rest.background.a, 0.);
        assert_eq!(rest.title, color(ColorRole::TextPrimary));
        let hover = ToolCallLook::of(ToolStatus::Running, true);
        assert_eq!(hover.ink, color(ColorRole::TextPrimary));
        assert_eq!(hover.background, color(ColorRole::LayerHover01));
        let failed = ToolCallLook::of(ToolStatus::Failed, false);
        assert_eq!(failed.title, color(ColorRole::TextError));
        assert_eq!(geometry::BODY_PADDING.3, 32.);
    }
}
