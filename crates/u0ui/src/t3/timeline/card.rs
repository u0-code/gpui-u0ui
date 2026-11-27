//! The `.t3-card` family's shared anatomy ("Cards" in `t3.css`): the
//! layer-01 frame with an optional 3px status rail, the head row, labels,
//! notes and the end-aligned action row.
use crate::t3::press::rem;
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, BoxShadow, Div, FontWeight, Hsla, IntoElement, ParentElement, SharedString, Styled,
    div, prelude::FluentBuilder as _, px,
};

/// `inset 3px 0 0 <c>`: the left status rail (`.t3-approval`, `.t3-wt--*`,
/// `.t3-cmd--fail`, `.t3-tbanner`).
pub(crate) fn rail(c: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(3.), px(0.), c).inset()]
}

/// `.t3-card { display: flex; flex-direction: column; gap: spacing-04;
/// padding: spacing-05; background: layer-01 }` (and `max-width: none` in
/// the chat column).
pub(crate) fn frame() -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .gap(rem(12.))
        .p(rem(16.))
        .bg(color(ColorRole::Layer01))
        .text_color(color(ColorRole::TextPrimary))
}

/// `.t3-card-head { display: flex; align-items: center; gap: spacing-03;
/// font-size: 0.875rem }` (line height inherited, 18px).
pub(crate) fn head() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .min_w_0()
        .gap(rem(8.))
        .text_size(rem(14.))
        .line_height(rem(18.))
}

/// `.t3-card-head > svg { fill: icon-primary; flex: none }`.
pub(crate) fn head_icon(icon: CarbonIcon, ink: Hsla) -> AnyElement {
    crate::t3::press::icon(icon, 16., ink).into_any_element()
}

/// `.t3-card-head b { font-weight: 600 }`.
pub(crate) fn head_title(text: impl Into<SharedString>) -> Div {
    div()
        .min_w_0()
        .overflow_hidden()
        .text_ellipsis()
        .whitespace_nowrap()
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

/// `.t3-card-sub { font-size: 0.75rem; color: text-secondary }`.
pub(crate) fn sub(text: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .text_size(rem(12.))
        .line_height(rem(16.))
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// `.t3-card-label { font-size: 0.75rem; color: text-secondary;
/// margin-bottom: calc(-1 * spacing-03) }`.
pub(crate) fn label(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(rem(12.))
        .line_height(rem(16.))
        .mb(rem(-8.))
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// `.t3-card-note { font-size: 0.875rem; line-height: 1.42857; color: text-secondary }`.
pub(crate) fn note(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(rem(14.))
        .line_height(rem(20.))
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// `.t3-card-actions { display: flex; flex-wrap: wrap; justify-content:
/// flex-end; align-items: center; gap: spacing-03 }`.
pub(crate) fn actions() -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .justify_end()
        .items_center()
        .gap(rem(8.))
}

/// `.t3-grow { flex: 1 }`.
pub(crate) fn grow() -> Div {
    div().flex_1()
}

/// A labelled frame with a rail.
pub(crate) fn framed(rail_color: Option<Hsla>) -> Div {
    frame().when_some(rail_color, |this, c| this.shadow(rail(c)))
}
