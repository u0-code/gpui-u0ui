//! GhostThreadRow: Ghost's sidebar thread row (`GhostThreadRow` in
//! `src/ghost/workspace.jsx`, `.g-trow2*` and `.g-mini-tag` in
//! `src/ghost.css`).
//!
//! A `density-row-md` row (16px start, 8px end): a 16px status glyph
//! (spinner while working, the warning glyph when it needs you, a red error
//! when failed, else draft / archived / an unread dot / the chat glyph), the
//! title ("Draft · " in `text-secondary` before drafts), cool-gray mini tags,
//! the server it runs on as a small [`ServerChip`], a pin, and the time (the
//! elapsed time while working).
//!
//! Rows rest in `text-secondary`; hover `layer-hover-01` and `text-primary`;
//! unread titles are `text-primary` semibold; archived rows `text-helper`
//! with an italic title. The open thread (`current`) is
//! `state-current-background` with a 3px `state-current-indicator` start
//! bar; a multi-selected row (`selected`) is `state-selected-background`;
//! both can apply at once (the selected fill wins, the bar stays).
use super::server::{ServerChip, ServerChipSize, ServerState};
use crate::{
    density,
    ghost::primitives::parts::{focus_ring, focus_visible, keyed_focus},
    icons::{CarbonIcon, Icon},
    t3::{
        base::{Spin, fmt_elapsed, visually_hidden, warning_glyph},
        craft::{rem, sans, sized},
        shell::MachineKind,
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// A thread's run state (`status`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThreadStatus {
    /// `idle` (the default).
    #[default]
    Idle,
    /// `working`: a spinner and the elapsed time.
    Working,
    /// `needs-you`: the warning glyph.
    NeedsYou,
    /// `failed`: a red error glyph.
    Failed,
}

impl ThreadStatus {
    /// Every state.
    pub const ALL: [Self; 4] = [Self::Idle, Self::Working, Self::NeedsYou, Self::Failed];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::NeedsYou => "needs-you",
            Self::Failed => "failed",
        }
    }
}

/// The row's flags, for [`row_look`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowFlags {
    /// `current`.
    pub current: bool,
    /// `selected`.
    pub selected: bool,
    /// `unread`.
    pub unread: bool,
    /// `archived`.
    pub archived: bool,
    /// Under the pointer.
    pub hovered: bool,
}

/// (background, ink, start bar, semibold) in cascade order: `.g-trow2 {
/// color: text-secondary }`, `:hover { background: layer-hover-01; color:
/// text-primary }`, `--unread { color: text-primary; font-weight: 600 }`,
/// `--archived { color: text-helper }`, `--current { background:
/// state-current-background; color: text-primary; box-shadow: inset 3px 0 0
/// state-current-indicator }`, `--selected { background:
/// state-selected-background; color: text-primary }`.
pub fn row_look(f: RowFlags) -> (Option<Hsla>, Hsla, bool, bool) {
    let mut bg = None;
    let mut ink = ColorRole::TextSecondary;
    if f.hovered {
        bg = Some(ColorRole::LayerHover01);
        ink = ColorRole::TextPrimary;
    }
    if f.unread {
        ink = ColorRole::TextPrimary;
    }
    if f.archived {
        ink = ColorRole::TextHelper;
    }
    if f.current {
        bg = Some(ColorRole::StateCurrentBackground);
        ink = ColorRole::TextPrimary;
    }
    if f.selected {
        bg = Some(ColorRole::StateSelectedBackground);
        ink = ColorRole::TextPrimary;
    }
    (bg.map(color), color(ink), f.current, f.unread)
}

/// The server chip a row shows (`server`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowServer {
    /// `name`.
    pub name: SharedString,
    /// `kind`.
    pub kind: MachineKind,
    /// `state`.
    pub state: ServerState,
}

impl RowServer {
    /// An online laptop called `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            kind: MachineKind::Laptop,
            state: ServerState::Online,
        }
    }

    /// `kind`.
    pub fn kind(mut self, kind: MachineKind) -> Self {
        self.kind = kind;
        self
    }

    /// `state`.
    pub fn state(mut self, state: ServerState) -> Self {
        self.state = state;
        self
    }
}

/// The row's user-facing strings (the reference's defaults).
#[derive(Clone, Debug)]
pub struct GhostThreadRowTexts {
    /// The title prefix and hidden state of drafts: `"Draft"`.
    pub draft: SharedString,
    /// Hidden after the glyph: `", unread"`.
    pub unread: SharedString,
    /// Hidden after the glyph: `", archived"`.
    pub archived: SharedString,
    /// The pin's name: `"Pinned"`.
    pub pinned: SharedString,
}

impl Default for GhostThreadRowTexts {
    fn default() -> Self {
        Self {
            draft: "Draft".into(),
            unread: ", unread".into(),
            archived: ", archived".into(),
            pinned: "Pinned".into(),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A Ghost sidebar thread row. See the module docs.
#[derive(IntoElement)]
pub struct GhostThreadRow {
    id: ElementId,
    title: SharedString,
    status: ThreadStatus,
    time: Option<SharedString>,
    elapsed: Option<u64>,
    unread: bool,
    pinned: bool,
    archived: bool,
    draft: bool,
    tags: Vec<SharedString>,
    server: Option<RowServer>,
    current: bool,
    selected: bool,
    on_click: Option<Handler>,
    texts: GhostThreadRowTexts,
}

impl GhostThreadRow {
    /// A row for the thread `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status: ThreadStatus::Idle,
            time: None,
            elapsed: None,
            unread: false,
            pinned: false,
            archived: false,
            draft: false,
            tags: Vec::new(),
            server: None,
            current: false,
            selected: false,
            on_click: None,
            texts: GhostThreadRowTexts::default(),
        }
    }

    /// `status`.
    pub fn status(mut self, status: ThreadStatus) -> Self {
        self.status = status;
        self
    }

    /// `time`.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// Seconds running, shown instead of the time while working (`elapsed`).
    pub fn elapsed(mut self, seconds: u64) -> Self {
        self.elapsed = Some(seconds);
        self
    }

    /// `unread`.
    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    /// `pinned`.
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    /// `archived`.
    pub fn archived(mut self, archived: bool) -> Self {
        self.archived = archived;
        self
    }

    /// `draft`.
    pub fn draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }

    /// `tags`.
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    /// The server it runs on (`server`).
    pub fn server(mut self, server: RowServer) -> Self {
        self.server = Some(server);
        self
    }

    /// The open thread (`current`).
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Multi-selected (`selected`).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Activation (`onClick`).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: GhostThreadRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// The hidden state text after the glyph (`draft ? 'Draft' : status`,
    /// then ", unread" / ", archived").
    pub fn hidden_state(&self) -> String {
        let mut s = if self.draft {
            self.texts.draft.to_string()
        } else if self.status == ThreadStatus::Idle {
            String::new()
        } else {
            self.status.name().to_string()
        };
        if self.unread {
            s.push_str(&self.texts.unread);
        }
        if self.archived {
            s.push_str(&self.texts.archived);
        }
        s
    }
}

impl RenderOnce for GhostThreadRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let (bg, ink, bar, bold) = row_look(RowFlags {
            current: self.current,
            selected: self.selected,
            unread: self.unread,
            archived: self.archived,
            hovered: *hovered.read(cx),
        });
        let glyph = |name: CarbonIcon, c: Hsla| {
            Icon::carbon(name)
                .size_length(rem(16.))
                .color(c)
                .into_any_element()
        };
        let icon: AnyElement = match self.status {
            ThreadStatus::Working => Spin::new((id.clone(), "spin")).into_any_element(),
            ThreadStatus::NeedsYou => warning_glyph(16.).into_any_element(),
            ThreadStatus::Failed => glyph(CarbonIcon::ErrorFilled, color(ColorRole::SupportError)),
            ThreadStatus::Idle if self.draft => glyph(CarbonIcon::Edit, ink),
            ThreadStatus::Idle if self.archived => glyph(CarbonIcon::Archive, ink),
            // `.t3-unread { width: 8px; height: 8px; border-radius: 50%;
            // background: var(--interactive) }`.
            ThreadStatus::Idle if self.unread => div()
                .size(px(8.))
                .rounded_full()
                .bg(color(ColorRole::Interactive))
                .into_any_element(),
            ThreadStatus::Idle => glyph(CarbonIcon::Chat, ink),
        };
        let time = match (self.status, self.elapsed) {
            (ThreadStatus::Working, Some(s)) => Some(SharedString::from(fmt_elapsed(s))),
            _ => self.time.clone(),
        };
        let hidden = self.hidden_state();
        let on_click = self.on_click.clone();
        // `.g-trow2 { display: flex; align-items: center; gap: var(--spacing-03);
        // height: var(--density-row-md); padding: 0 var(--spacing-03) 0
        // var(--spacing-05); font-size: 0.875rem }`.
        sans(
            gpui_kit::base::Button::new(id.clone()),
            14.,
            if bold { 600 } else { 400 },
        )
        .track_focus(&focus)
        .relative()
        .flex()
        .items_center()
        .w_full()
        .gap(tokens::SPACING_03)
        .h(density::current().row())
        .pl(tokens::SPACING_05)
        .pr(tokens::SPACING_03)
        .text_color(ink)
        .when_some(bg, |b, c| b.bg(c))
        .cursor(CursorStyle::PointingHand)
        .on_hover(move |h, _, cx| {
            hovered.update(cx, |v, cx| {
                if *v != *h {
                    *v = *h;
                    cx.notify();
                }
            })
        })
        .on_click(move |_, w, cx| {
            if let Some(f) = &on_click {
                f(w, cx);
            }
        })
        .when(bar, |b| {
            b.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(px(3.))
                    .bg(color(ColorRole::StateCurrentIndicator)),
            )
        })
        // `.g-trow2-icon { display: inline-flex; width: 16px; justify-content: center }`.
        .child(
            div()
                .relative()
                .flex()
                .flex_none()
                .w(rem(16.))
                .justify_center()
                .child(icon)
                .child(visually_hidden(hidden)),
        )
        // `.g-trow2-title { flex: 1; min-width: 0; text-overflow: ellipsis }`;
        // archived titles are italic.
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .when(self.archived, |d| d.italic())
                .when(self.draft, |d| {
                    d.child(
                        div()
                            .flex_none()
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("{} · ", self.texts.draft)),
                    )
                })
                .child(div().min_w_0().truncate().child(self.title.clone())),
        )
        .children(self.tags.iter().map(|t| {
            // `.g-mini-tag { padding: 0 var(--spacing-02); background:
            // var(--tag-background-cool-gray); color: var(--tag-color-cool-gray);
            // font-size: 0.6875rem; line-height: 1rem; font-weight: 400;
            // border-radius: 16 }`.
            sans(div(), 11., 400)
                .flex_none()
                .line_height(rem(16.))
                .px(tokens::SPACING_02)
                .rounded(tokens::BORDER_RADIUS_16.rems())
                .bg(color(ColorRole::TagBackgroundCoolGray))
                .text_color(color(ColorRole::TagColorCoolGray))
                .child(t.clone())
        }))
        .when_some(self.server.clone(), |b, s| {
            b.child(
                ServerChip::new((id.clone(), "server"), s.name)
                    .kind(s.kind)
                    .state(s.state)
                    .size(ServerChipSize::Sm),
            )
        })
        .when(self.pinned, |b| {
            b.child(
                Icon::carbon(CarbonIcon::PinFilled)
                    .size_length(rem(12.))
                    .color(color(ColorRole::IconSecondary)),
            )
        })
        .when_some(time, |b, t| {
            // `.g-trow2-time { font-size: 0.75rem; color: var(--text-helper);
            // font-variant-numeric: tabular-nums }`. No weight of its own: it
            // inherits the row's, so unread rows show it semibold.
            b.child(
                sized(div(), 12.)
                    .flex_none()
                    .text_color(color(ColorRole::TextHelper))
                    .child(t),
            )
        })
        .when(ring, |b| b.child(focus_ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn looks_follow_the_cascade() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        let rest = row_look(RowFlags::default());
        assert_eq!((rest.0, rest.1), (None, c(ColorRole::TextSecondary)));
        let hover = row_look(RowFlags {
            hovered: true,
            ..Default::default()
        });
        assert_eq!(hover.0, Some(c(ColorRole::LayerHover01)));
        let unread = row_look(RowFlags {
            unread: true,
            ..Default::default()
        });
        assert_eq!((unread.1, unread.3), (c(ColorRole::TextPrimary), true));
        let archived = row_look(RowFlags {
            archived: true,
            hovered: true,
            ..Default::default()
        });
        assert_eq!(archived.1, c(ColorRole::TextHelper));
        let current = row_look(RowFlags {
            current: true,
            ..Default::default()
        });
        assert_eq!(
            (current.0, current.2),
            (Some(c(ColorRole::StateCurrentBackground)), true)
        );
        // Current and selected together: the selected fill, the current bar.
        let both = row_look(RowFlags {
            current: true,
            selected: true,
            ..Default::default()
        });
        assert_eq!(
            (both.0, both.2),
            (Some(c(ColorRole::StateSelectedBackground)), true)
        );
    }

    #[test]
    fn hidden_state_text() {
        let row = GhostThreadRow::new("r", "t")
            .status(ThreadStatus::NeedsYou)
            .unread(true);
        assert_eq!(row.hidden_state(), "needs-you, unread");
        let row = GhostThreadRow::new("r", "t").draft(true).archived(true);
        assert_eq!(row.hidden_state(), "Draft, archived");
        assert_eq!(GhostThreadRow::new("r", "t").hidden_state(), "");
    }
}
