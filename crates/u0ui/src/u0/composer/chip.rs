//! ComposerChip: context attached to the composer (`ComposerChip`, `.g-cchip*`).
//!
//! A bordered `layer-02` card up to 22rem wide: the kind's glyph, then a
//! button (opens the source) with an 11px uppercase kind label, an optional
//! PR/issue state (a colored dot and the capitalised state) and meta, the
//! 13px title and an excerpt clamped to two lines: quotes get a left rule
//! and italics, terminal output the terminal surface in mono. A 28px remove
//! button closes it.
use crate::{
    icons::{CarbonIcon, Icon},
    t3::{
        craft::{mono, rem, sans},
        press::inset_frame,
    },
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::{
        IconButton,
        parts::{focus_ring, focus_visible, keyed_focus},
    },
};
use gpui_kit::{
    App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// What the chip carries (`kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// `quote` (the default): a quoted passage.
    #[default]
    Quote,
    /// `terminal-output`.
    TerminalOutput,
    /// `review-comment`.
    ReviewComment,
    /// `pr`: a pull request.
    PullRequest,
    /// `issue`.
    Issue,
}

impl ChipKind {
    /// Every kind.
    pub const ALL: [Self; 5] = [
        Self::Quote,
        Self::TerminalOutput,
        Self::ReviewComment,
        Self::PullRequest,
        Self::Issue,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Quote => "quote",
            Self::TerminalOutput => "terminal-output",
            Self::ReviewComment => "review-comment",
            Self::PullRequest => "pr",
            Self::Issue => "issue",
        }
    }

    /// `CV[kind]`: the glyph and the reference's label.
    pub const fn glyph_and_label(self) -> (CarbonIcon, &'static str) {
        match self {
            Self::Quote => (CarbonIcon::Quotes, "Quote"),
            Self::TerminalOutput => (CarbonIcon::Terminal, "Terminal output"),
            Self::ReviewComment => (CarbonIcon::Chat, "Review comment"),
            Self::PullRequest => (CarbonIcon::PullRequest, "Pull request"),
            Self::Issue => (CarbonIcon::Ticket, "Issue"),
        }
    }
}

/// A PR or issue state (`state`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChipState {
    /// `open`.
    Open,
    /// `merged`.
    Merged,
    /// `closed`.
    Closed,
    /// `draft`.
    Draft,
}

impl ChipState {
    /// Every state.
    pub const ALL: [Self; 4] = [Self::Open, Self::Merged, Self::Closed, Self::Draft];

    /// The state's name (shown capitalised, `text-transform: capitalize`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Merged => "merged",
            Self::Closed => "closed",
            Self::Draft => "draft",
        }
    }

    /// The capitalised label.
    pub fn label(self) -> String {
        let name = self.name();
        name[..1].to_uppercase() + &name[1..]
    }
}

/// The state dot: `.g-cchip-state::before { background: var(--icon-secondary) }`,
/// `--open` `support-success`, `--merged` `support-info`, `--closed` `support-error`.
pub fn chip_state_dot(state: ChipState) -> Hsla {
    color(match state {
        ChipState::Open => ColorRole::SupportSuccess,
        ChipState::Merged => ColorRole::SupportInfo,
        ChipState::Closed => ColorRole::SupportError,
        ChipState::Draft => ColorRole::IconSecondary,
    })
}

/// The chip's user-facing strings beyond the kind labels.
#[derive(Clone)]
pub struct ComposerChipTexts {
    /// The kind label, overriding the English default.
    pub kind: Option<SharedString>,
    /// The remove button's name: `"Remove {kind}"` (lower-cased label).
    pub remove: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for ComposerChipTexts {
    fn default() -> Self {
        Self {
            kind: None,
            remove: Rc::new(|kind| format!("Remove {}", kind.to_lowercase()).into()),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An attached-context chip. See the module docs.
#[derive(IntoElement)]
pub struct ComposerChip {
    id: ElementId,
    kind: ChipKind,
    title: SharedString,
    excerpt: Option<SharedString>,
    meta: Option<SharedString>,
    state: Option<ChipState>,
    on_open: Option<Handler>,
    on_remove: Option<Handler>,
    texts: ComposerChipTexts,
}

impl ComposerChip {
    /// A chip of `kind` titled `title`.
    pub fn new(id: impl Into<ElementId>, kind: ChipKind, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind,
            title: title.into(),
            excerpt: None,
            meta: None,
            state: None,
            on_open: None,
            on_remove: None,
            texts: ComposerChipTexts::default(),
        }
    }

    /// Up to two lines of the content (`excerpt`).
    pub fn excerpt(mut self, excerpt: impl Into<SharedString>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    /// A secondary note after the kind (`meta`, e.g. the reviewer).
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// The PR/issue state (`state`).
    pub fn state(mut self, state: ChipState) -> Self {
        self.state = Some(state);
        self
    }

    /// Open the source (`onOpen`).
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// Remove it (`onRemove`); without one there is no remove button.
    pub fn on_remove(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ComposerChipTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for ComposerChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let over = *hovered.read(cx);
        let (glyph, label) = self.kind.glyph_and_label();
        let label: SharedString = self.texts.kind.clone().unwrap_or(label.into());
        let secondary = color(ColorRole::TextSecondary);
        let on_open = self.on_open.clone();
        let excerpt = self.excerpt.clone().map(|ex| {
            // `.g-cchip-ex { font-size: 0.75rem; color: var(--text-secondary);
            // -webkit-line-clamp: 2 }`.
            let base = match self.kind {
                ChipKind::TerminalOutput => mono(div(), 12., 400),
                _ => sans(div(), 12., 400),
            };
            base.w_full()
                .line_clamp(2)
                .overflow_hidden()
                .text_color(secondary)
                .map(|d| match self.kind {
                    // `.g-cchip--quote .g-cchip-ex { border-left: 2px solid
                    // var(--border-strong-01); padding-left: var(--spacing-03);
                    // font-style: italic }`.
                    ChipKind::Quote => d
                        .border_l_2()
                        .border_color(color(ColorRole::BorderStrong01))
                        .pl(tokens::SPACING_03)
                        .italic(),
                    // `.g-cchip--terminal-output .g-cchip-ex { padding: 2px
                    // var(--spacing-02); background: var(--terminal-background);
                    // color: var(--terminal-text) }`.
                    ChipKind::TerminalOutput => d
                        .py(px(2.))
                        .px(tokens::SPACING_02)
                        .bg(color(ColorRole::TerminalBackground))
                        .text_color(color(ColorRole::TerminalText)),
                    _ => d,
                })
                .child(ex)
        });
        let remove = self.on_remove.clone().map(|f| {
            // `.g-cchip > .g-iconbtn { width: 1.75rem; height: 1.75rem }`.
            IconButton::new(
                (id.clone(), "remove"),
                CarbonIcon::Close,
                (self.texts.remove)(&label),
            )
            .size_px(28.)
            .icon_px(14.)
            .on_click(move |_, w, cx| f(w, cx))
        });
        // `.g-cchip { display: flex; align-items: stretch; max-width: 22rem;
        // background: var(--layer-02); box-shadow: inset 0 0 0 1px
        // var(--border-subtle-01) }`.
        div()
            .id(id.clone())
            .flex()
            .items_stretch()
            .w_full()
            .max_w(rem(352.))
            .bg(color(ColorRole::Layer02))
            .shadow(inset_frame(color(ColorRole::BorderSubtle01)))
            .child(
                // `.g-cchip-icon { padding: var(--spacing-03) 0 0 var(--spacing-03) }`.
                div()
                    .flex_none()
                    .pt(tokens::SPACING_03)
                    .pl(tokens::SPACING_03)
                    .child(
                        Icon::carbon(glyph)
                            .size_length(rem(16.))
                            .color(color(ColorRole::IconSecondary)),
                    ),
            )
            .child(
                // `.g-cchip-main { flex: 1; flex-direction: column; gap: 2px;
                // padding: var(--spacing-03) }`, hover `layer-hover-01`.
                gpui_kit::base::Button::new((id.clone(), "main"))
                    .track_focus(&focus)
                    .relative()
                    .flex()
                    .flex_col()
                    .items_start()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .p(tokens::SPACING_03)
                    .when(over, |b| b.bg(color(ColorRole::LayerHover01)))
                    .text_color(color(ColorRole::TextPrimary))
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
                        if let Some(f) = &on_open {
                            f(w, cx);
                        }
                    })
                    .child(
                        // `.g-cchip-top { display: flex; gap: var(--spacing-03);
                        // font-size: 0.6875rem }`.
                        sans(div(), 12., 400)
                            .flex()
                            .items_center()
                            .gap(tokens::SPACING_03)
                            .child(
                                // `.g-cchip-kind { text-transform: uppercase }`.
                                div()
                                    .text_color(secondary)
                                    .child(SharedString::from(label.to_uppercase())),
                            )
                            .when_some(self.state, |d, s| {
                                // `.g-cchip-state { color: var(--text-primary);
                                // font-weight: 600 }` with its 6px dot, 4px before.
                                d.child(
                                    sans(div(), 12., 600)
                                        .flex()
                                        .items_center()
                                        .gap(px(4.))
                                        .text_color(color(ColorRole::TextPrimary))
                                        .child(
                                            div().size(px(6.)).rounded_full().bg(chip_state_dot(s)),
                                        )
                                        .child(s.label()),
                                )
                            })
                            .when_some(self.meta.clone(), |d, m| {
                                d.child(div().text_color(secondary).child(m))
                            }),
                    )
                    .child(
                        // `.g-cchip-title { font-size: 0.8125rem; text-overflow: ellipsis }`.
                        sans(div(), 13., 400)
                            .min_w_0()
                            .truncate()
                            .child(self.title.clone()),
                    )
                    .children(excerpt)
                    .when(ring, |b| b.child(focus_ring())),
            )
            .children(remove)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn kinds_and_states() {
        assert_eq!(
            ChipKind::TerminalOutput.glyph_and_label(),
            (CarbonIcon::Terminal, "Terminal output")
        );
        assert_eq!(ChipKind::PullRequest.name(), "pr");
        assert_eq!(ChipState::Closed.label(), "Closed");
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            chip_state_dot(ChipState::Open),
            c(ColorRole::SupportSuccess)
        );
        assert_eq!(chip_state_dot(ChipState::Merged), c(ColorRole::SupportInfo));
        assert_eq!(
            chip_state_dot(ChipState::Closed),
            c(ColorRole::SupportError)
        );
        assert_eq!(
            chip_state_dot(ChipState::Draft),
            c(ColorRole::IconSecondary)
        );
        assert_eq!(
            (ComposerChipTexts::default().remove)("Terminal output"),
            "Remove terminal output"
        );
    }
}
