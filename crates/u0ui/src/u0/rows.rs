//! The list-row frame the u0 admin rows share (`src/ghost.css` "Tasks &
//! workflows": `.g-task, .g-wfrow, .g-slrow, .g-agent, .g-acct, .g-script,
//! .g-dev, .g-tunnel, .g-log`, `.g-task-main`, `.g-slrow-main`,
//! `.g-task-meta`, `.g-dev-meta`) and the small text parts they use (`.g-dim`,
//! `.g-mono`, `.g-ok` / `.g-warn` / `.g-err`).
use crate::{
    icons::{CarbonIcon, Icon},
    t3::{
        base::warning_glyph,
        craft::{mono, rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{AnyElement, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div, px};

/// The row frame u0 · Tasks defined (`.g-task, .g-wfrow, .g-slrow, …`).
pub(crate) use crate::u0::tasks::row::frame;

/// `.g-slrow-main { display: flex; flex: 1; min-width: 0; flex-direction:
/// column; gap: 2px }`: the title line, then the second lines at 12px
/// (`.g-slrow-main > span:not(:first-child) { font-size: 0.75rem }`).
pub(crate) fn slrow_main(title: Div, lines: impl IntoIterator<Item = AnyElement>) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap(px(2.))
        .child(title)
        .children(
            lines
                .into_iter()
                .map(|l| sans(div(), 12., 400).min_w_0().child(l)),
        )
}

/// `.g-slrow-main b { display: flex; align-items: center; gap:
/// var(--spacing-03); font-weight: 400; overflow: hidden; white-space:
/// nowrap; text-overflow: ellipsis }` in `ink`.
pub(crate) fn title(ink: Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_color(ink)
}

/// The title's text run, truncated with an ellipsis.
pub(crate) fn title_text(text: impl Into<SharedString>) -> Div {
    div()
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(text.into())
}

/// `.g-dev-meta` (= `.g-task-meta`): `display: flex; flex-wrap: wrap;
/// align-items: center; gap: var(--spacing-02) var(--spacing-03); font-size:
/// 0.75rem; color: var(--text-secondary)`.
pub(crate) fn meta(runs: impl IntoIterator<Item = AnyElement>) -> Div {
    sans(div(), 12., 400)
        .flex()
        .flex_wrap()
        .items_center()
        .gap_x(tokens::SPACING_03)
        .gap_y(tokens::SPACING_02)
        .text_color(color(ColorRole::TextSecondary))
        .children(runs)
}

/// `.g-dim { color: var(--text-secondary) }` at `size` px.
pub(crate) fn dim(text: impl Into<SharedString>, size: f32) -> Div {
    sans(div(), size, 400)
        .whitespace_nowrap()
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// `.g-mono { font-family: var(--font-mono); font-size: 0.75rem }`.
pub(crate) fn mono_text(text: impl Into<SharedString>) -> Div {
    mono(div(), 12., 400).whitespace_nowrap().child(text.into())
}

/// The tone of a `.g-ok` / `.g-warn` / `.g-err` state line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StateTone {
    /// `.g-ok`: `text-secondary`, a `support-success` glyph (`CheckmarkFilled`).
    Ok,
    /// `.g-warn`: `text-secondary`, the `support-warning` glyph with its dark
    /// "!" (`WarningAltFilled.t3-warnicon`).
    Warn,
    /// `.g-err`: `text-error`, a `support-error` glyph (`ErrorFilled`).
    Err,
}

impl StateTone {
    /// The text ink: `.g-ok, .g-warn { color: var(--text-secondary) }`,
    /// `.g-err { color: var(--text-error) }`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Ok | Self::Warn => ColorRole::TextSecondary,
            Self::Err => ColorRole::TextError,
        })
    }

    /// The glyph fill: `.g-ok svg { fill: var(--support-success) }`,
    /// `.g-warn svg { fill: var(--support-warning) }`, `.g-err svg { fill:
    /// var(--support-error) }`.
    pub fn glyph_ink(self) -> Hsla {
        color(match self {
            Self::Ok => ColorRole::SupportSuccess,
            Self::Warn => ColorRole::SupportWarning,
            Self::Err => ColorRole::SupportError,
        })
    }

    /// The glyph the rows pair with the tone, `size` px.
    pub fn glyph(self, size: f32) -> AnyElement {
        match self {
            Self::Warn => warning_glyph(size).into_any_element(),
            Self::Ok => Icon::carbon(CarbonIcon::CheckmarkFilled)
                .size_length(rem(size))
                .color(self.glyph_ink())
                .into_any_element(),
            Self::Err => Icon::carbon(CarbonIcon::ErrorFilled)
                .size_length(rem(size))
                .color(self.glyph_ink())
                .into_any_element(),
        }
    }
}

/// `.g-ok, .g-warn, .g-err, .g-statetext { display: inline-flex;
/// align-items: center; gap: var(--spacing-02); font-size: 0.75rem }` with a
/// 12px glyph.
pub(crate) fn state_text(tone: StateTone, text: impl Into<SharedString>) -> Div {
    sans(div(), 12., 400)
        .flex()
        .flex_none()
        .items_center()
        .gap(tokens::SPACING_02)
        .whitespace_nowrap()
        .text_color(tone.ink())
        .child(div().flex().flex_none().child(tone.glyph(12.)))
        .child(text.into())
}

/// [`state_text`] that wraps at word boundaries in a narrow column instead of
/// overflowing (the reference's inline-flex text wraps; the glyph stays
/// centred beside it).
pub(crate) fn state_text_wrapping(tone: StateTone, text: impl Into<SharedString>) -> Div {
    sans(div(), 12., 400)
        .flex()
        .items_center()
        .gap(tokens::SPACING_02)
        .min_w_0()
        .text_color(tone.ink())
        .child(div().flex().flex_none().child(tone.glyph(12.)))
        .child(div().min_w_0().whitespace_normal().child(text.into()))
}

/// A `flex: none` wrapper for a trailing control.
pub(crate) fn tail(el: impl IntoElement) -> Div {
    div().flex().flex_none().items_center().child(el)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn state_tones_follow_g_ok_warn_err() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(StateTone::Ok.ink(), color(ColorRole::TextSecondary));
        assert_eq!(StateTone::Warn.ink(), color(ColorRole::TextSecondary));
        assert_eq!(StateTone::Err.ink(), color(ColorRole::TextError));
        assert_eq!(StateTone::Ok.glyph_ink(), color(ColorRole::SupportSuccess));
        assert_eq!(
            StateTone::Warn.glyph_ink(),
            color(ColorRole::SupportWarning)
        );
        assert_eq!(StateTone::Err.glyph_ink(), color(ColorRole::SupportError));
    }
}
