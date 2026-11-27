//! NoteLine and CheckpointMarker: the transcript's centred lines (`NoteLine`,
//! `CheckpointMarker` in `src/ghost/workspace.jsx`; `.g-note*`, `.g-ckpt*`
//! in `src/ghost.css`, with the `gemini-A` / `gemini-B` craft pass in
//! `src/t3.css`).
//!
//! - **NoteLine**: a 12px centred note: stopped, note, a usage limit (with
//!   "· resumes at 14:00") or an error (`role="alert"`, `text-error`), an
//!   optional action after it.
//! - **CheckpointMarker**: a symmetric divider (two `border-subtle-01`
//!   hairlines around a bookmark chip: label · time · N files). "Restore to
//!   here" floats over the right end of the line on `background` while the
//!   marker is hovered or the button has keyboard focus, so it reserves no
//!   slot (gemini-A/B). Callers should confirm before restoring.
use crate::{
    ghost::primitives::parts::{focus_ring, focus_visible, keyed_focus},
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// A note's kind (`kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NoteKind {
    /// `stopped`: the user stopped the agent.
    Stopped,
    /// `note` (the default).
    #[default]
    Note,
    /// `limit`: a usage limit was reached.
    Limit,
    /// `error`: announced as an alert.
    Error,
}

impl NoteKind {
    /// Every kind.
    pub const ALL: [Self; 4] = [Self::Stopped, Self::Note, Self::Limit, Self::Error];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Note => "note",
            Self::Limit => "limit",
            Self::Error => "error",
        }
    }

    /// `{ stopped: StopFilled, note: InformationFilled, limit: Hourglass,
    /// error: ErrorFilled }`.
    pub const fn glyph(self) -> CarbonIcon {
        match self {
            Self::Stopped => CarbonIcon::StopFilled,
            Self::Note => CarbonIcon::InformationFilled,
            Self::Limit => CarbonIcon::Hourglass,
            Self::Error => CarbonIcon::ErrorFilled,
        }
    }

    /// (text, glyph): `.g-note { color: var(--text-secondary) }`, `svg {
    /// fill: var(--icon-secondary) }`; `--error` `text-error` /
    /// `support-error`; `--limit svg` `support-warning`.
    pub fn colors(self) -> (Hsla, Hsla) {
        match self {
            Self::Error => (color(ColorRole::TextError), color(ColorRole::SupportError)),
            Self::Limit => (
                color(ColorRole::TextSecondary),
                color(ColorRole::SupportWarning),
            ),
            Self::Stopped | Self::Note => (
                color(ColorRole::TextSecondary),
                color(ColorRole::IconSecondary),
            ),
        }
    }
}

/// The note's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct NoteLineTexts {
    /// `"· resumes at {time}"`.
    pub resumes_at: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for NoteLineTexts {
    fn default() -> Self {
        Self {
            resumes_at: Rc::new(|t| format!("· resumes at {t}").into()),
        }
    }
}

/// A centred transcript note. See the module docs.
#[derive(IntoElement)]
pub struct NoteLine {
    kind: NoteKind,
    text: SharedString,
    resumes_at: Option<SharedString>,
    action: Option<AnyElement>,
    texts: NoteLineTexts,
}

impl NoteLine {
    /// A note of `kind` saying `text` (`children`).
    pub fn new(kind: NoteKind, text: impl Into<SharedString>) -> Self {
        Self {
            kind,
            text: text.into(),
            resumes_at: None,
            action: None,
            texts: NoteLineTexts::default(),
        }
    }

    /// When a limit lifts (`resumesAt`).
    pub fn resumes_at(mut self, time: impl Into<SharedString>) -> Self {
        self.resumes_at = Some(time.into());
        self
    }

    /// An action after the text (`action`), e.g. a ghost "Retry".
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: NoteLineTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for NoteLine {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (ink, glyph) = self.kind.colors();
        // `.g-note { display: flex; align-items: center; justify-content:
        // center; gap: var(--spacing-03); padding: var(--spacing-03) 0;
        // font-size: 0.75rem }`.
        sans(div(), 12., 400)
            .flex()
            .items_center()
            .justify_center()
            .gap(tokens::SPACING_03)
            .py(tokens::SPACING_03)
            .text_color(ink)
            .child(
                Icon::carbon(self.kind.glyph())
                    .size_length(rem(14.))
                    .color(glyph)
                    .flex_none(),
            )
            .child(self.text)
            .when_some(self.resumes_at, |d, t| {
                d.child(
                    div()
                        .text_color(color(ColorRole::TextSecondary))
                        .child((self.texts.resumes_at)(&t)),
                )
            })
            .children(self.action)
    }
}

// ── CheckpointMarker ────────────────────────────────────────────────────

/// Whether Restore shows: the marker is hovered or the button has keyboard
/// focus (`.g-ckpt:hover .g-ckpt-restore, .g-ckpt-restore:focus-visible {
/// opacity: 1 }`; it rests at `opacity: 0`).
pub fn restore_visible(marker_hovered: bool, focus_visible: bool) -> bool {
    marker_hovered || focus_visible
}

/// The Restore button's fill: `background` (craft pass, so it masks the
/// line it floats over), `layer-hover-01` under the pointer.
pub fn restore_fill(hovered: bool) -> Hsla {
    color(if hovered {
        ColorRole::LayerHover01
    } else {
        ColorRole::Background
    })
}

/// The marker's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct CheckpointMarkerTexts {
    /// `"Restore to here"`.
    pub restore: SharedString,
    /// `" · {n} files"`.
    pub files: Rc<dyn Fn(usize) -> SharedString>,
}

impl Default for CheckpointMarkerTexts {
    fn default() -> Self {
        Self {
            restore: "Restore to here".into(),
            files: Rc::new(|n| format!(" · {n} files").into()),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A checkpoint divider. See the module docs.
#[derive(IntoElement)]
pub struct CheckpointMarker {
    id: ElementId,
    label: SharedString,
    time: Option<SharedString>,
    files: Option<usize>,
    on_restore: Option<Handler>,
    texts: CheckpointMarkerTexts,
}

impl CheckpointMarker {
    /// A marker labelled `label` (`label`, the reference's "Checkpoint").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            time: None,
            files: None,
            on_restore: None,
            texts: CheckpointMarkerTexts::default(),
        }
    }

    /// When it was taken (`time`).
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// How many files it holds (`files`).
    pub fn files(mut self, files: usize) -> Self {
        self.files = Some(files);
        self
    }

    /// "Restore to here" (`onRestore`).
    pub fn on_restore(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_restore = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: CheckpointMarkerTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for CheckpointMarker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let hover = |key: &'static str, window: &mut Window, cx: &mut App| {
            window.use_keyed_state((id.clone(), key), cx, |_, _| false)
        };
        let (marker, button) = (hover("hover", window, cx), hover("btn-hover", window, cx));
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let shown = restore_visible(*marker.read(cx), ring);
        let link = color(ColorRole::LinkPrimary);
        let dim = color(ColorRole::TextSecondary);
        let track = |store: gpui_kit::Entity<bool>| {
            move |h: &bool, _: &mut Window, cx: &mut App| {
                store.update(cx, |v, cx| {
                    if *v != *h {
                        *v = *h;
                        cx.notify();
                    }
                })
            }
        };
        // `.g-ckpt-line { flex: 1; height: 1px; background: var(--border-subtle-01) }`
        // (the craft pass replaces the dashed `border-strong-01` rule).
        let line = || {
            div()
                .flex_1()
                .h(px(1.))
                .bg(color(ColorRole::BorderSubtle01))
        };
        let on_restore = self.on_restore.clone();
        let over = *button.read(cx);
        // `.g-ckpt-restore { position: absolute; right: 0; top: 50%;
        // transform: translateY(-50%); display: inline-flex; gap:
        // var(--spacing-02); height: 1.5rem; padding: 0 var(--spacing-03);
        // color: var(--link-primary); font-size: 0.75rem }`.
        let restore = sans(
            gpui_kit::base::Button::new((id.clone(), "restore")),
            12.,
            400,
        )
        .track_focus(&focus)
        .absolute()
        .right_0()
        .top(gpui_kit::relative(0.5))
        .mt(rem(-12.))
        .flex()
        .items_center()
        .gap(tokens::SPACING_02)
        .h(rem(24.))
        .px(tokens::SPACING_03)
        .bg(restore_fill(over))
        .text_color(link)
        .opacity(if shown { 1. } else { 0. })
        .cursor(CursorStyle::PointingHand)
        .on_hover(track(button))
        .on_click(move |_, w, cx| {
            if let Some(f) = &on_restore {
                f(w, cx);
            }
        })
        .child(
            Icon::carbon(CarbonIcon::Undo)
                .size_length(rem(12.))
                .color(link),
        )
        .child(self.texts.restore.clone())
        .when(ring, |b| b.child(focus_ring()));
        // `.g-ckpt { position: relative; display: flex; align-items: center;
        // gap: var(--spacing-03); font-size: 0.75rem; color: var(--text-secondary) }`.
        sans(div(), 12., 400)
            .id(id)
            .relative()
            .flex()
            .items_center()
            .w_full()
            .min_h(rem(24.))
            .gap(tokens::SPACING_03)
            .text_color(dim)
            .on_hover(track(marker))
            .child(line())
            .child(
                // `.g-ckpt-chip { display: inline-flex; gap: var(--spacing-02) }`.
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(tokens::SPACING_02)
                    .child(
                        Icon::carbon(CarbonIcon::Bookmark)
                            .size_length(rem(12.))
                            .color(dim),
                    )
                    .child(
                        div()
                            .flex()
                            .child(self.label.clone())
                            .when_some(self.time.clone(), |d, t| d.child(format!(" · {t}")))
                            .when_some(self.files, |d, n| d.child((self.texts.files)(n))),
                    ),
            )
            .child(line())
            .child(restore)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn notes_follow_the_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            NoteKind::Error.colors(),
            (c(ColorRole::TextError), c(ColorRole::SupportError))
        );
        assert_eq!(NoteKind::Limit.colors().1, c(ColorRole::SupportWarning));
        assert_eq!(NoteKind::Note.colors().1, c(ColorRole::IconSecondary));
        assert_eq!(NoteKind::Limit.glyph(), CarbonIcon::Hourglass);
        assert_eq!(
            (NoteLineTexts::default().resumes_at)("14:00"),
            "· resumes at 14:00"
        );
    }

    #[test]
    fn restore_floats_on_hover_or_focus() {
        assert!(!restore_visible(false, false));
        assert!(restore_visible(true, false));
        assert!(restore_visible(false, true));
        crate::theme::tests_set(CarbonTheme::G100);
        let c = |r| color_in(CarbonTheme::G100, r);
        assert_eq!(restore_fill(false), c(ColorRole::Background));
        assert_eq!(restore_fill(true), c(ColorRole::LayerHover01));
    }
}
