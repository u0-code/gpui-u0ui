//! Small drawing helpers shared by the T3 review and mobile modules: the
//! inherited text box of the reference page, the 2px `:focus-visible` ring,
//! pointer-tracked pressables and initials discs.
//!
//! T3 CSS sets `font-size` on most parts and inherits the page's
//! `body { font-size: 0.875rem; line-height: 1.28572 }` (`src/custom.css`), so
//! text boxes are `size × 1.28572` unless a rule says otherwise.
use crate::{
    components::avatar::AvatarColor,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, Div, ElementId, FocusHandle, Hsla, IntoElement, ParentElement, Rems,
    SharedString, Styled, Window, div, px, relative,
};

/// `body { line-height: 1.28572 }` (`src/custom.css`), inherited by T3 parts.
pub(crate) const BODY_LINE: f32 = 1.28572;

/// px → rems at Carbon's 16px rem.
pub(crate) fn rem(px_value: f32) -> Rems {
    crate::components::form::rem(px_value)
}

/// IBM Plex Sans at `size` px and `weight`, with the inherited line height.
pub(crate) fn sans<E: Styled>(el: E, size: f32, weight: u16) -> E {
    el.font(font(FontFamilyKey::Sans, weight))
        .text_size(rem(size))
        .line_height(relative(BODY_LINE))
}

/// IBM Plex Mono (`var(--font-mono)`) at `size` px and `weight`.
pub(crate) fn mono<E: Styled>(el: E, size: f32, weight: u16) -> E {
    el.font(font(FontFamilyKey::Mono, weight))
        .text_size(rem(size))
        .line_height(relative(BODY_LINE))
}

/// A text run: `div` with sans text in `role`.
pub(crate) fn text(s: impl Into<SharedString>, size: f32, role: ColorRole) -> Div {
    sans(div(), size, 400)
        .text_color(color(role))
        .child(s.into())
}

/// `outline: 2px solid var(--focus); outline-offset: -2px`: an inset ring
/// laid over a `relative` parent.
pub(crate) fn focus_ring() -> AnyElement {
    focus_ring_offset(0.)
}

/// A 2px `focus` ring `offset` px outside the parent box (negative = inside
/// by that much more than the ring itself).
pub(crate) fn focus_ring_offset(offset: f32) -> AnyElement {
    div()
        .absolute()
        .top(px(-offset))
        .left(px(-offset))
        .right(px(-offset))
        .bottom(px(-offset))
        .border_2()
        .border_color(color(ColorRole::Focus))
        .into_any_element()
}

/// A plain hover tooltip standing in for an HTML `title` attribute (a
/// native browser tooltip, not a Carbon `Tooltip`): Kit's window tooltip.
pub(crate) fn title_tooltip(
    text: SharedString,
) -> impl Fn(&mut Window, &mut App) -> gpui_kit::AnyView {
    move |window, cx| gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
}

/// `title="…"` on a small element: an id derived from `kind` and the text,
/// plus the hover tooltip.
pub(crate) fn titled(
    el: Div,
    kind: &str,
    text: impl Into<SharedString>,
) -> gpui_kit::Stateful<Div> {
    use gpui_kit::{InteractiveElement as _, StatefulInteractiveElement as _};
    let text: SharedString = text.into();
    el.id(ElementId::Name(format!("t3-{kind}-{text}").into()))
        .tooltip(title_tooltip(text))
}

/// A child id `name-i` under `id`.
pub(crate) fn sub(id: &ElementId, name: &str, i: usize) -> ElementId {
    (id.clone(), SharedString::from(format!("{name}-{i}"))).into()
}

/// A keyed focus handle for `id`.
pub(crate) fn focus_handle(id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
    window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

/// Whether `focus` shows `:focus-visible` (focused by keyboard).
pub(crate) fn focus_visible(focus: &FocusHandle, window: &Window) -> bool {
    focus.is_focused(window) && window.last_input_was_keyboard()
}

/// An unstyled Base button for `id` tracking `focus`: start-aligned, the
/// inherited line height (Base centres and sets `line-height: 1`).
pub(crate) fn press(id: impl Into<ElementId>, focus: &FocusHandle) -> gpui_kit::base::Button {
    gpui_kit::base::Button::new(id)
        .track_focus(focus)
        .relative()
        .justify_start()
        .line_height(relative(BODY_LINE))
        .cursor_pointer()
}

/// A tone for an initials disc: the avatar tag pairs, or none
/// (`AVATAR_TONES` includes `cyan` and `green`, which have no
/// `.cu-avatar--<c>` rule: no fill, inherited ink).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AvatarTone {
    /// A `.cu-avatar--<c>` pair.
    Avatar(AvatarColor),
    /// No fill (`cyan`, `green`).
    Plain,
}

impl AvatarTone {
    /// (fill, ink) for the tone; `Plain` paints `ink` on nothing.
    pub fn colors(self, ink: Hsla) -> (Hsla, Hsla) {
        match self {
            Self::Avatar(c) => c.colors(),
            Self::Plain => (Hsla::transparent_black(), ink),
        }
    }
}

/// An initials disc `diameter` px wide (`.cu-avatar` + `.cu-avatar-fallback`
/// `{ font-size: 0.75rem; font-weight: 600 }`; the fallback keeps 12px even
/// where a parent rule shrinks the disc, and clips).
pub(crate) fn initials(text: impl Into<SharedString>, tone: AvatarTone, diameter: f32) -> Div {
    let (bg, fg) = tone.colors(color(ColorRole::TextPrimary));
    sans(div(), 12., 600)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(rem(diameter))
        .rounded_full()
        .overflow_hidden()
        .whitespace_nowrap()
        .bg(bg)
        .text_color(fg)
        .child(text.into())
}

/// The stable tone of an actor (`AVATAR_TONES[hash % 6]` in
/// `PullRequestTimeline`: `h = h * 33 + code` from 17, over UTF-16 units).
pub fn actor_tone(actor: &str) -> AvatarTone {
    const TONES: [AvatarTone; 6] = [
        AvatarTone::Avatar(AvatarColor::Purple),
        AvatarTone::Plain, // cyan
        AvatarTone::Avatar(AvatarColor::Teal),
        AvatarTone::Avatar(AvatarColor::Magenta),
        AvatarTone::Avatar(AvatarColor::Blue),
        AvatarTone::Plain, // green
    ];
    let h = actor
        .encode_utf16()
        .fold(17u32, |h, c| h.wrapping_mul(33).wrapping_add(c as u32));
    TONES[(h % 6) as usize]
}

/// The first two characters of `name`, upper-cased (`.slice(0, 2).toUpperCase()`).
pub fn two_letters(name: &str) -> String {
    name.chars().take(2).collect::<String>().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_tones_hash_like_the_reference() {
        // ((17·33 + 106)·33 + 100) % 6 = 1 → cyan (no fill) for "jd".
        assert_eq!(actor_tone("jd"), AvatarTone::Plain);
        assert_eq!(actor_tone("ak"), AvatarTone::Plain);
        assert_eq!(actor_tone("mr"), AvatarTone::Avatar(AvatarColor::Purple));
        // 17 % 6 = 5 → green (no fill).
        assert_eq!(actor_tone(""), AvatarTone::Plain);
    }

    #[test]
    fn initials_take_two_upper_case_letters() {
        assert_eq!(two_letters("jd"), "JD");
        assert_eq!(two_letters("you"), "YO");
        assert_eq!(two_letters("a"), "A");
    }
}
