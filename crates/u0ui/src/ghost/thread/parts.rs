//! Pieces the Thread cards share: the fold heads' ink (`.g-tool-head`,
//! `.g-think-head`), the turning chevron (`.t3-chev`, `.t3-rot`), the
//! `.g-dim` run, and keyed open state for `defaultOpen` folds.
use crate::{
    ghost::primitives::tree::CHEVRON_MOTION,
    icons::{CarbonIcon, Icon},
    motion,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{App, ElementId, Entity, Hsla, Styled, Window};

pub(crate) use crate::t3::press::{focus, hover, rem, ring, track_hover};

/// The ink of a fold head (`.g-tool-head`, `.g-think-head`):
/// `text-secondary`, `text-primary` while hovered (`:hover { color: text-primary }`).
pub fn head_ink(hovered: bool) -> Hsla {
    color(if hovered {
        ColorRole::TextPrimary
    } else {
        ColorRole::TextSecondary
    })
}

/// `.g-mono { font-family: var(--font-mono); font-size: 0.75rem }`.
pub(crate) fn mono<E: Styled>(el: E) -> E {
    el.font(font(FontFamilyKey::Mono, 400)).text_size(rem(12.))
}

/// Keyed open state seeded from `default_open` (`useState(defaultOpen)`).
pub(crate) fn open_state(
    id: impl Into<ElementId>,
    default_open: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<bool> {
    window.use_keyed_state(id, cx, move |_, _| default_open)
}

/// Flip an [`open_state`].
pub(crate) fn toggle(state: &Entity<bool>, cx: &mut App) {
    state.update(cx, |open, cx| {
        *open = !*open;
        cx.notify();
    });
}

/// The `t3-chev` angle: 90° open (`.t3-chev--open { transform: rotate(90deg) }`),
/// animated over 110ms productive (`transition: transform 110ms`; a cut
/// under reduced motion).
pub(crate) fn chevron_right(
    id: impl Into<ElementId>,
    open: bool,
    size: f32,
    window: &mut Window,
    cx: &mut App,
) -> Icon {
    let id: ElementId = id.into();
    let angle = motion::animate(
        (id, "chev"),
        if open {
            std::f32::consts::FRAC_PI_2
        } else {
            0.
        },
        CHEVRON_MOTION,
        window,
        cx,
    );
    // `.t3-chev { fill: var(--icon-secondary); flex: none }`.
    Icon::carbon(CarbonIcon::ChevronRight)
        .size_length(rem(size))
        .color(color(ColorRole::IconSecondary))
        .rotate(gpui_kit::radians(angle))
}

/// The `ChevronDown` that turns −90° while closed (`.t3-rot { transform:
/// rotate(-90deg) }`, no transition), in `ink`.
pub(crate) fn chevron_down(open: bool, size: f32, ink: Hsla) -> Icon {
    Icon::carbon(CarbonIcon::ChevronDown)
        .size_length(rem(size))
        .color(ink)
        .rotate(gpui_kit::radians(chevron_down_angle(open)))
}

/// The [`chevron_down`] angle: 0 open, −90° closed.
pub fn chevron_down_angle(open: bool) -> f32 {
    if open {
        0.
    } else {
        -std::f32::consts::FRAC_PI_2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn heads_brighten_on_hover_and_chevrons_turn() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(head_ink(false), color(ColorRole::TextSecondary));
        assert_eq!(head_ink(true), color(ColorRole::TextPrimary));
        assert_eq!(chevron_down_angle(true), 0.);
        assert_eq!(chevron_down_angle(false), -std::f32::consts::FRAC_PI_2);
    }
}
