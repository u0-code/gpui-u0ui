//! Badge indicators on icon buttons: Carbon's `BadgeIndicator`
//! (`IconButton badgeCount`, `.cds--badge-indicator`) and the patterns'
//! "new content" dot (`.cu-pdot`).
//!
//! - **Count**: a `support-error` pill in the button's top-end corner,
//!   `helper-text-01` in `text-on-color`; counts over 999 read `999+`
//!   (`BadgeIndicator`: `count > 999 ? '999+' : count`). The reference uses
//!   it only on 48px header buttons.
//! - **Dot**: a 6px `support-error` circle 6px in from the top-end corner,
//!   for toolbar buttons with something new. It is decoration
//!   (`aria-hidden`); the button's label says what is new.
//!
//! [`Badged`] wraps a button and places the badge over it
//! (`position: absolute` inside the button / `.cu-pdot-wrap`).
use crate::{
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px,
    rems,
};

/// `.cds--badge-indicator--count { margin-block-start: 0.25rem; margin-inline-end: 0.25rem }`.
pub const COUNT_INSET: tokens::TokenLength = tokens::SPACING_02;
/// `.cds--badge-indicator { padding: 0 0.25rem 0.125rem }`, each side.
pub const COUNT_PADDING_X: tokens::TokenLength = tokens::SPACING_02;
/// `.cds--badge-indicator { max-block-size: 1rem }` (the 16px line fills it).
pub const COUNT_HEIGHT_REM: f32 = 1.;
/// `.cds--badge-indicator { min-inline-size: 0.5rem }`.
pub const COUNT_MIN_WIDTH_REM: f32 = 0.5;
/// `.cu-pdot { top: 6px; right: 6px; width: 6px; height: 6px }`.
pub const DOT: (f32, f32) = (6., 6.);

/// The text of a count badge: the number, or `999+` above 999.
pub fn count_text(count: u32) -> SharedString {
    if count > 999 {
        "999+".into()
    } else {
        count.to_string().into()
    }
}

/// A badge's colors: (`background`, `text`).
pub fn colors() -> (gpui_kit::Hsla, gpui_kit::Hsla) {
    // `.cds--badge-indicator { background: var(--support-error); color: var(--text-on-color) }`,
    // `.cu-pdot { background: var(--support-error) }`.
    (
        color(ColorRole::SupportError),
        color(ColorRole::TextOnColor),
    )
}

/// Which badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeIndicator {
    /// A number (`badgeCount`).
    Count(u32),
    /// The "new" dot (`.cu-pdot`).
    Dot,
}

impl BadgeIndicator {
    /// The badge, absolutely placed in its button's top-end corner.
    pub fn element(self) -> AnyElement {
        let (bg, fg) = colors();
        match self {
            Self::Count(n) => div()
                .absolute()
                // `inset-block-start: 0; inset-inline-end: 0` + the count margins.
                .top(COUNT_INSET)
                .right(COUNT_INSET)
                .flex()
                .items_center()
                .h(rems(COUNT_HEIGHT_REM))
                .min_w(rems(COUNT_MIN_WIDTH_REM))
                .px(COUNT_PADDING_X)
                .overflow_hidden()
                // `border-radius: 100px`.
                .rounded_full()
                .bg(bg)
                // `font-size: var(--helper-text-01-font-size)`.
                .type_style(TypeStyle::HelperText01)
                .text_color(fg)
                .child(count_text(n))
                .into_any_element(),
            Self::Dot => div()
                .absolute()
                .top(px(DOT.0))
                .right(px(DOT.0))
                .size(px(DOT.1))
                // `.cu-pdot { border-radius: 50% }`.
                .rounded_full()
                .bg(bg)
                .into_any_element(),
        }
    }
}

/// A button with a badge over it.
#[derive(IntoElement)]
pub struct Badged {
    button: AnyElement,
    badge: Option<BadgeIndicator>,
}

impl Badged {
    /// Wrap `button`; `badge: None` draws the button alone.
    pub fn new(button: impl IntoElement, badge: Option<BadgeIndicator>) -> Self {
        Self {
            button: button.into_any_element(),
            badge,
        }
    }
}

impl RenderOnce for Badged {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.cu-pdot-wrap { position: relative; display: inline-flex }`.
        div()
            .relative()
            .flex()
            .flex_none()
            .child(self.button)
            .children(self.badge.map(BadgeIndicator::element))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn counts_cap_at_three_digits() {
        assert_eq!(count_text(4), "4");
        assert_eq!(count_text(999), "999");
        assert_eq!(count_text(1200), "999+");
    }

    #[test]
    fn geometry_follows_the_css() {
        assert_eq!(COUNT_INSET.px_value(), 4.);
        assert_eq!(COUNT_PADDING_X.px_value(), 4.);
        assert_eq!(DOT, (6., 6.));
    }

    #[test]
    fn badges_are_support_error() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, fg) = colors();
        assert_eq!(
            bg,
            crate::theme::color_in(CarbonTheme::White, ColorRole::SupportError)
        );
        assert_eq!(
            fg,
            crate::theme::color_in(CarbonTheme::White, ColorRole::TextOnColor)
        );
    }
}
