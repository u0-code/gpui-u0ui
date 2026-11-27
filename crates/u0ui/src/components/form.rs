//! Carbon form anatomy shared by every field (`.cds--form-item`).
//!
//! The parts each Carbon input repeats: the `.cds--label` above the control,
//! the `.cds--form__helper-text` under it, the `.cds--form-requirement`
//! (invalid or warning message) that replaces the helper text, the status icon
//! at the field's end, the 2px inset outline used for focus and errors, and
//! the field sizes under [`Density`].
//!
//! Components compose these; applications can too, to build a custom control
//! that lines up with Carbon fields.
use crate::{
    density::Density,
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, Div, Hsla, IntoElement, ParentElement, Rems, SharedString, Styled, div,
    prelude::FluentBuilder as _, px,
};

/// px → rems at Carbon's 16px rem, so text-related lengths follow zoom.
pub(crate) fn rem(px_value: f32) -> Rems {
    gpui_kit::rems(px_value / tokens::REM_PX)
}

/// shadcn's controlled/uncontrolled value (`value` + `defaultValue`): the
/// caller's `value` when given, else the value kept in keyed state under `id`
/// (seeded with `default`). Returns the value to show and the keyed store,
/// which a change handler must update so an uncontrolled control remembers.
pub(crate) fn controlled<T: Clone + 'static>(
    id: impl Into<gpui_kit::ElementId>,
    value: Option<T>,
    default: T,
    window: &mut gpui_kit::Window,
    cx: &mut gpui_kit::App,
) -> (T, gpui_kit::Entity<T>) {
    let store = window.use_keyed_state(id, cx, |_, _| default);
    let shown = value.unwrap_or_else(|| store.read(cx).clone());
    (shown, store)
}

/// `index.d.ts` `Size` (`'sm' | 'md' | 'lg'`) for fields and field-like controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FieldSize {
    /// `sm`: 32px (`--layout-size-height-sm`).
    Sm,
    /// `md`: 40px, the Carbon default (`--layout-size-height-md`).
    #[default]
    Md,
    /// `lg`: 48px (`--layout-size-height-lg`).
    Lg,
}

impl FieldSize {
    /// Every size.
    pub const ALL: [FieldSize; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// Carbon height in px (`--layout-size-height-sm/md/lg`).
    pub const fn carbon_height(self) -> f32 {
        match self {
            Self::Sm => tokens::SIZE_SM.px_value(),
            Self::Md => tokens::SIZE_MD.px_value(),
            Self::Lg => tokens::SIZE_LG.px_value(),
        }
    }

    /// Field height under `density`. Compact sets every Carbon field to
    /// `density-field-md` (`[data-density="compact"] .cds--text-input,
    /// .cds--select-input, .cds--list-box, .cds--date-picker__input { block-size:
    /// var(--density-field-md) }`).
    pub const fn height(self, density: Density) -> f32 {
        match density {
            Density::Compact => tokens::DENSITY_FIELD_MD.px_value(),
            Density::Comfortable => self.carbon_height(),
        }
    }
}

/// A field's validation state (`invalid`/`invalidText`, `warn`/`warnText`).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Validation {
    /// No message.
    #[default]
    None,
    /// `invalid`: 2px `support-error` outline, error icon, `text-error` message.
    Invalid(SharedString),
    /// `warn`: warning icon, `text-primary` message, no outline.
    Warn(SharedString),
}

impl Validation {
    /// Build from the `index.d.ts` flags; invalid wins over warn, as in Carbon.
    pub fn from_flags(
        invalid: bool,
        invalid_text: Option<SharedString>,
        warn: bool,
        warn_text: Option<SharedString>,
    ) -> Self {
        if invalid {
            Self::Invalid(invalid_text.unwrap_or_default())
        } else if warn {
            Self::Warn(warn_text.unwrap_or_default())
        } else {
            Self::None
        }
    }

    /// `invalid`.
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    /// `invalid` or `warn`: the field reserves room for the status icon.
    pub fn has_icon(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// The message, if any.
    pub fn message(&self) -> Option<&SharedString> {
        match self {
            Self::None => None,
            Self::Invalid(text) | Self::Warn(text) => Some(text).filter(|t| !t.is_empty()),
        }
    }

    /// `.cds--form-requirement` color: `text-error` when invalid
    /// (`.cds--text-input__field-wrapper[data-invalid] ~ .cds--form-requirement`),
    /// otherwise inherited `text-primary`.
    pub fn message_color(&self) -> Hsla {
        color(if self.is_invalid() {
            ColorRole::TextError
        } else {
            ColorRole::TextPrimary
        })
    }
}

/// `.cds--label`: `label-01` in `text-secondary`, 8px below
/// (`margin-block-end: 0.5rem`); `text-disabled` when disabled
/// (`.cds--label--disabled`).
pub fn label(text: impl Into<SharedString>, disabled: bool) -> Div {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(label_color(disabled))
        .mb(tokens::SPACING_03)
        .child(text.into())
}

/// The `.cds--label` color.
pub fn label_color(disabled: bool) -> Hsla {
    color(if disabled {
        ColorRole::TextDisabled
    } else {
        ColorRole::TextSecondary
    })
}

/// `.cds--form__helper-text`: `helper-text-01` in `text-helper`, 4px above
/// (`margin-block-start: 0.25rem`); `text-disabled` when disabled.
pub fn helper_text(text: impl Into<SharedString>, disabled: bool) -> Div {
    div()
        .w_full()
        .type_style(TypeStyle::HelperText01)
        .text_color(color(if disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextHelper
        }))
        .mt(tokens::SPACING_02)
        .child(text.into())
}

/// `.cds--form-requirement`: `helper-text-01`, 4px above (`margin: 0.25rem 0 0`),
/// colored by [`Validation::message_color`]. `None` when there is no message.
pub fn requirement(validation: &Validation) -> Option<Div> {
    validation.message().map(|text| {
        div()
            .w_full()
            .type_style(TypeStyle::HelperText01)
            .text_color(validation.message_color())
            .mt(tokens::SPACING_02)
            .child(text.clone())
    })
}

/// The message row under a field: the requirement when invalid/warn,
/// otherwise the helper text (Carbon hides the helper while a requirement shows).
pub fn message(
    validation: &Validation,
    helper: Option<&SharedString>,
    disabled: bool,
) -> Option<AnyElement> {
    if !disabled && let Some(requirement) = requirement(validation) {
        return Some(requirement.into_any_element());
    }
    helper.map(|text| helper_text(text.clone(), disabled).into_any_element())
}

/// The 16px status icon at a field's end: `WarningFilled` in `support-error`
/// (`.cds--text-input__invalid-icon`), or `WarningAltFilled` in
/// `support-warning` with its inner mark black
/// (`.cds--text-input__invalid-icon--warning path:first-of-type { fill: #000 }`).
pub fn status_icon(validation: &Validation) -> Option<AnyElement> {
    match validation {
        Validation::None => None,
        Validation::Invalid(_) => Some(
            Icon::carbon(CarbonIcon::WarningFilled)
                .size(IconSize::S16)
                .color(color(ColorRole::SupportError))
                .into_any_element(),
        ),
        Validation::Warn(_) => Some(
            div()
                .relative()
                .size(rem(16.))
                .flex_shrink_0()
                // The glyph's "!" is a cut-out; fill it black like the
                // reference's opaque inner path.
                .child(
                    div()
                        .absolute()
                        .left(rem(7.))
                        .top(rem(6.))
                        .w(rem(2.))
                        .h(rem(7.5))
                        .bg(gpui_kit::black()),
                )
                .child(
                    Icon::carbon(CarbonIcon::WarningAltFilled)
                        .size(IconSize::S16)
                        .color(color(ColorRole::SupportWarning))
                        .absolute()
                        .top_0()
                        .left_0(),
                )
                .into_any_element(),
        ),
    }
}

/// A field's 1px bottom rule (`border-block-end: 1px solid var(--border-strong)`).
pub const FIELD_RULE: f32 = 1.;

/// The field's bottom rule, drawn as an absolutely positioned child along the
/// bottom edge. The field reserves it with `.pb(px(FIELD_RULE))` instead of a
/// GPUI border, and adds this child before its [`inset_outline`]: GPUI paints
/// an element's own border above its children, so a real border would show
/// through the 2px ring, while CSS paints `outline: 2px; outline-offset: -2px`
/// over the border (`.cds--text-input:focus`, `.cu-ig:focus-within`). The
/// parent must be `relative` and have no bottom border.
pub fn bottom_rule(rule: Hsla) -> Div {
    div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .h(px(FIELD_RULE))
        .bg(rule)
}

/// An inset outline (`outline: <width> solid <color>; outline-offset: -<width>`)
/// drawn as an absolutely positioned border on top of its parent's content.
/// The parent must be `relative`; it covers the whole border box only when the
/// parent has no GPUI border (see [`bottom_rule`]).
pub fn inset_outline(width: f32, outline: Hsla) -> Div {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .border(px(width))
        .border_color(outline)
}

/// The outline a field shows: 2px `focus` while focused, else 2px
/// `support-error` when invalid (`input[data-invalid]:not(:focus)`), else none.
pub fn field_outline(focused: bool, validation: &Validation) -> Option<Hsla> {
    if focused {
        Some(color(ColorRole::Focus))
    } else if validation.is_invalid() {
        Some(color(ColorRole::SupportError))
    } else {
        None
    }
}

/// A `.cds--form-item`: a column, items at the start, full width. Under a
/// right-to-left [`super::direction::DirectionProvider`] the start is the
/// right edge and text aligns right (label, helper, messages).
pub fn form_item() -> Div {
    let item = div().flex().flex_col().w_full().min_w_0();
    if super::direction::is_rtl() {
        item.items_end().text_right()
    } else {
        item.items_start()
    }
}

/// `content` in a `.cds--form-item` that sits in a flex row: Carbon gives
/// the form item `flex: 1 1 auto`, so a checkbox (`.cds--form-item
/// .cds--checkbox-wrapper`) grows into the row's free space and pushes what
/// follows it to the end. Use it where the reference puts a form-item
/// control in a `display: flex` row; components keep their own width
/// elsewhere.
pub fn flex_form_item(content: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .items_start()
        .flex_auto()
        .child(content)
}

/// A `.cds--form-item` with an optional label above `control` and the message
/// row under it.
pub fn labeled(
    label_text: Option<&SharedString>,
    disabled: bool,
    control: impl IntoElement,
    message_row: Option<AnyElement>,
) -> Div {
    form_item()
        .when_some(label_text.filter(|l| !l.is_empty()), |this, text| {
            this.child(label(text.clone(), disabled))
        })
        .child(control)
        .children(message_row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn flex_form_item_grows_like_cds_form_item() {
        // `.cds--form-item { display: flex; flex: 1 1 auto; flex-direction:
        // column; align-items: flex-start }`.
        let mut item = flex_form_item(div());
        let style = item.style();
        assert_eq!(style.flex_grow, Some(1.));
        assert_eq!(style.flex_shrink, Some(1.));
        assert_eq!(style.flex_basis, Some(gpui_kit::Length::Auto));
        assert_eq!(style.flex_direction, Some(gpui_kit::FlexDirection::Column));
        assert_eq!(style.align_items, Some(gpui_kit::AlignItems::FlexStart));
    }

    #[test]
    fn field_heights_follow_layout_size_and_density() {
        // `--layout-size-height-sm/md/lg`: 2rem / 2.5rem / 3rem.
        let heights = FieldSize::ALL.map(|s| s.height(Density::Comfortable));
        assert_eq!(heights, [32., 40., 48.]);
        // `[data-density="compact"] .cds--text-input { block-size: var(--density-field-md) }`.
        let compact = FieldSize::ALL.map(|s| s.height(Density::Compact));
        assert_eq!(compact, [32., 32., 32.]);
    }

    #[test]
    fn validation_colors_follow_form_requirement() {
        crate::theme::tests_set(CarbonTheme::White);
        let invalid = Validation::from_flags(true, Some("Bad".into()), true, None);
        assert!(invalid.is_invalid());
        // `.cds--form-requirement { color: var(--text-error) }` when invalid.
        assert_eq!(hex(invalid.message_color()), "#da1e28");
        let warn = Validation::from_flags(false, None, true, Some("Hm".into()));
        assert_eq!(hex(warn.message_color()), "#161616");
        assert_eq!(warn.message().map(|s| s.as_ref()), Some("Hm"));
        assert_eq!(Validation::None.message(), None);
        // `.cds--label` text-secondary; disabled text-disabled (alpha).
        assert_eq!(hex(label_color(false)), "#525252");
        assert!(label_color(true).a < 1.);
    }

    #[test]
    fn outline_prefers_focus_over_invalid() {
        crate::theme::tests_set(CarbonTheme::White);
        let invalid = Validation::Invalid("x".into());
        assert_eq!(
            field_outline(true, &invalid).map(hex).as_deref(),
            Some("#0f62fe")
        );
        assert_eq!(
            field_outline(false, &invalid).map(hex).as_deref(),
            Some("#da1e28")
        );
        assert_eq!(field_outline(false, &Validation::Warn("w".into())), None);
    }
}
