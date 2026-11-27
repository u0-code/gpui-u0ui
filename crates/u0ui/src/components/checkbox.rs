//! Checkbox: shadcn `Checkbox` → Carbon `Checkbox` (`.cds--checkbox`).
//!
//! - **Behavior** is Kit's [`gpui_kit::base::Checkbox`]: focus, Space and
//!   pointer activation on the whole row (box and label), `aria-checked`
//!   including `mixed`, disabled.
//! - **Appearance**: a 16px box with a 2px radius, 1px `icon-primary` border,
//!   3px in from the row (`.cds--checkbox-label::before`); checked or
//!   indeterminate fills `icon-primary` (dark, not blue) and draws the
//!   `icon-inverse` check or 8×2 dash; the label is `body-compact-01` 30px from
//!   the row start. Focus: 2px `focus` ring 1px outside the box. Disabled:
//!   `icon-disabled` box, `text-disabled` label. Invalid: `support-error` box
//!   border plus the error icon and message; warn: warning icon and message.
//!   Read-only: hollow `icon-disabled` box with a `text-primary` check.
//! - **Motion**: none; the fill and check change instantly (`.cds--checkbox-label`
//!   has no `transition` in `bundle.css`).
//!
//! ```ignore
//! Checkbox::new("notes").label("Send me release notes").default_checked(true)
//! ```
use super::form::{self, Validation, rem};
use crate::{
    icons::Icon,
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

pub use gpui_kit::base::CheckboxState;

/// The state a checkbox look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckboxLookState {
    /// Checked, unchecked or indeterminate.
    pub value: CheckboxState,
    /// Disabled.
    pub disabled: bool,
    /// Invalid (`.cds--checkbox-wrapper--invalid`).
    pub invalid: bool,
    /// Read-only.
    pub readonly: bool,
}

/// The colors of a checkbox in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxLook {
    /// Box fill.
    pub fill: Hsla,
    /// Box border (`None`: no border, the filled box).
    pub border: Option<Hsla>,
    /// Check / dash color.
    pub mark: Hsla,
    /// Label color.
    pub label: Hsla,
}

impl CheckboxLook {
    /// The look of a checkbox (`.cds--checkbox*` rules).
    pub fn of(state: CheckboxLookState) -> Self {
        let on = state.value != CheckboxState::Unchecked;
        let clear = Hsla::transparent_black();
        if state.disabled {
            // `.cds--checkbox:disabled + .cds--checkbox-label::before { border-color:
            // var(--icon-disabled) }`; checked `background-color: var(--icon-disabled)`.
            return Self {
                fill: if on {
                    color(ColorRole::IconDisabled)
                } else {
                    clear
                },
                border: (!on).then(|| color(ColorRole::IconDisabled)),
                mark: color(ColorRole::IconInverse),
                label: color(ColorRole::TextDisabled),
            };
        }
        if state.readonly {
            // `--readonly .cds--checkbox:checked + …::before { border: 1px solid
            // var(--icon-disabled); background: transparent }`, `::after { border-color:
            // var(--text-primary) }`.
            return Self {
                fill: clear,
                border: Some(color(ColorRole::IconDisabled)),
                mark: color(ColorRole::TextPrimary),
                label: color(ColorRole::TextPrimary),
            };
        }
        // `::before { border: 1px solid var(--icon-primary) }`; checked/indeterminate
        // `border: none; background-color: var(--icon-primary)`; invalid keeps a
        // `support-error` border even when checked.
        let border = if state.invalid {
            Some(color(ColorRole::SupportError))
        } else if on {
            None
        } else {
            Some(color(ColorRole::IconPrimary))
        };
        Self {
            fill: if on {
                color(ColorRole::IconPrimary)
            } else {
                clear
            },
            border,
            mark: color(ColorRole::IconInverse),
            label: color(ColorRole::TextPrimary),
        }
    }
}

/// Box geometry (px): size 16, inset 3px from the row's top-left
/// (`inset-block-start: 0.125rem; margin-block-start: 0.0625rem; margin-inline-start: 0.1875rem`),
/// label text start 30px (`padding-inline-start: 1.25rem` + `0.625rem`).
pub const BOX: f32 = 16.;
/// Box offset from the row's top and left.
pub const BOX_INSET: f32 = 3.;
/// Label start.
pub const LABEL_START: f32 = 30.;
/// Space under a checkbox (`.cds--form-item.cds--checkbox-wrapper { margin-block-end: 0.375rem }`),
/// so stacked checkboxes sit 26px apart (20px row + 6px).
pub const MARGIN_BOTTOM: f32 = 6.;
/// Space under the last checkbox of a group
/// (`.cds--form-item.cds--checkbox-wrapper:last-of-type { margin-block-end: 0.1875rem }`).
pub const MARGIN_BOTTOM_LAST: f32 = 3.;

/// The wrapper's bottom margin (px): none for an inline checkbox (Carbon
/// `InlineCheckbox`, no `.cds--form-item` wrapper), 3px for the last of
/// its siblings, else 6px.
pub fn margin_bottom(inline: bool, last_of_type: bool) -> f32 {
    if inline {
        0.
    } else if last_of_type {
        MARGIN_BOTTOM_LAST
    } else {
        MARGIN_BOTTOM
    }
}

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A Carbon checkbox. See the module docs.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    indeterminate: bool,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    helper_text: Option<SharedString>,
    on_checked_change: Option<ChangeHandler>,
    inline: bool,
    last_of_type: bool,
}

impl Checkbox {
    /// A checkbox with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            checked: None,
            default_checked: false,
            indeterminate: false,
            disabled: false,
            readonly: false,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            helper_text: None,
            on_checked_change: None,
            inline: false,
            last_of_type: false,
        }
    }

    /// Carbon `InlineCheckbox` (a table's selection column): the bare box
    /// and label, without the form item's bottom margin.
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// The last checkbox of its group: 3px under it instead of 6px
    /// (`.cds--checkbox-wrapper:last-of-type`; GPUI has no sibling selectors,
    /// so the caller says which one is last).
    pub fn last_of_type(mut self, last: bool) -> Self {
        self.last_of_type = last;
        self
    }

    /// The label (`label` / children).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Controlled `checked`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// `defaultChecked` for an uncontrolled checkbox.
    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }

    /// `indeterminate` (drawn as a dash; activating it checks the box).
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// `invalid`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// `invalidText`.
    pub fn invalid_text(mut self, text: impl Into<SharedString>) -> Self {
        self.invalid_text = Some(text.into());
        self
    }

    /// `warn`.
    pub fn warn(mut self, warn: bool) -> Self {
        self.warn = warn;
        self
    }

    /// `warnText`.
    pub fn warn_text(mut self, text: impl Into<SharedString>) -> Self {
        self.warn_text = Some(text.into());
        self
    }

    /// Helper text under the label.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// `onCheckedChange`.
    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Rc::new(handler));
        self
    }
}

/// The focus ring's inset (px, negative = outside) from the box's padding
/// edge, where absolutely positioned children are placed: `outline: 2px;
/// outline-offset: 1px` puts the ring 1–3px outside the border edge, and a
/// bordered box's padding edge is its 1px border further in.
pub fn focus_ring_inset(bordered: bool) -> f32 {
    const OFFSET: f32 = 1.;
    const RING: f32 = 2.;
    let border = if bordered { 1. } else { 0. };
    -(OFFSET + RING + border)
}

/// The 16px box with its mark, for checkbox-like rows (also used by
/// multi-select menus).
pub(crate) fn checkbox_box(
    look: CheckboxLook,
    value: CheckboxState,
    focus_ring: bool,
) -> impl IntoElement {
    let on = value != CheckboxState::Unchecked;
    let ring = focus_ring_inset(look.border.is_some());
    div()
        .relative()
        .flex_shrink_0()
        .size(rem(BOX))
        .rounded(rem(2.))
        .bg(look.fill)
        .when_some(look.border, |this, border| {
            this.border_1().border_color(border)
        })
        .flex()
        .items_center()
        .justify_center()
        .map(|this| match value {
            // `:indeterminate + …::after { border-block-end: 2px solid; inline-size: 0.5rem }`.
            CheckboxState::Indeterminate => this.child(div().w(rem(8.)).h(rem(2.)).bg(look.mark)),
            _ if on => this.child(Icon::named("check").size_length(rem(14.)).color(look.mark)),
            _ => this,
        })
        // `:focus + …::before { outline: 2px solid var(--focus); outline-offset: 1px }`.
        .when(focus_ring, |this| {
            this.child(
                div()
                    .absolute()
                    .top(px(ring))
                    .left(px(ring))
                    .right(px(ring))
                    .bottom(px(ring))
                    .rounded(rem(4.))
                    .border_2()
                    .border_color(color(ColorRole::Focus)),
            )
        })
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (checked, store) = form::controlled(
            (id.clone(), "checked"),
            self.checked,
            self.default_checked,
            window,
            cx,
        );
        let value = if self.indeterminate {
            CheckboxState::Indeterminate
        } else if checked {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        };
        let validation = if self.disabled || self.readonly {
            Validation::None
        } else {
            Validation::from_flags(
                self.invalid,
                self.invalid_text.clone(),
                self.warn,
                self.warn_text.clone(),
            )
        };
        let look = CheckboxLook::of(CheckboxLookState {
            value,
            disabled: self.disabled,
            invalid: validation.is_invalid(),
            readonly: self.readonly,
        });
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = focus.is_focused(window) && !self.disabled;
        let on_change = self.on_checked_change.clone();
        let mark = checkbox_box(look, value, focused);

        let row = gpui_kit::base::Checkbox::new(id.clone())
            .track_focus(&focus)
            .state(value)
            .disabled(self.disabled || self.readonly)
            .when_some(self.label.clone(), |this, label| {
                this.accessibility_label(label)
            })
            // `.cds--checkbox-label { display: flex; min-block-size: 1.25rem;
            // padding-block-start: 0.0625rem }`.
            .relative()
            .flex()
            .flex_row()
            .min_h(rem(20.))
            .pt(rem(1.))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(look.label)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else if self.readonly {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            })
            .on_change(move |next, _, window, cx| {
                let next = next == CheckboxState::Checked;
                store.update(cx, |c, cx| {
                    *c = next;
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(next, window, cx);
                }
            })
            .child(
                div()
                    .absolute()
                    .top(rem(BOX_INSET))
                    .left(rem(BOX_INSET))
                    .child(mark),
            )
            .child(
                div()
                    .pl(rem(LABEL_START))
                    .when_some(self.label.clone(), |this, label| this.child(label)),
            );

        // `.cds--checkbox__validation-msg { display: flex; margin-block-start: 0.25rem }`
        // with the icon (`margin: 1px 1px 0 3px`) and the requirement beside it.
        let message = validation.message().map(|text| {
            div()
                .flex()
                .flex_row()
                .items_start()
                .mt(rem(4.))
                .children(
                    form::status_icon(&validation)
                        .map(|icon| div().mt(rem(1.)).mr(rem(1.)).ml(rem(3.)).child(icon)),
                )
                .child(
                    div()
                        .ml(rem(8.))
                        .type_style(TypeStyle::HelperText01)
                        .text_color(validation.message_color())
                        .child(text.clone()),
                )
        });
        let helper = self
            .helper_text
            .clone()
            .filter(|_| message.is_none())
            .map(|text| form::helper_text(text, self.disabled));

        div()
            .flex()
            .flex_col()
            .items_start()
            .mb(rem(margin_bottom(self.inline, self.last_of_type)))
            .child(row)
            .children(message)
            .children(helper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn stacked_checkboxes_are_26px_apart() {
        // A 20px row (`.cds--checkbox-label { min-block-size: 1.25rem }`)
        // plus the wrapper's 6px margin; 3px under the last; none inline.
        assert_eq!(20. + margin_bottom(false, false), 26.);
        assert_eq!(margin_bottom(false, true), 3.);
        assert_eq!(margin_bottom(true, false), 0.);
        assert_eq!(margin_bottom(true, true), 0.);
    }

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    fn look(value: CheckboxState) -> CheckboxLook {
        CheckboxLook::of(CheckboxLookState {
            value,
            ..Default::default()
        })
    }

    #[test]
    fn colors_follow_checkbox_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let off = look(CheckboxState::Unchecked);
        assert_eq!(off.fill.a, 0.);
        assert_eq!(off.border.map(hex).as_deref(), Some("#161616"));
        // Checked fills `icon-primary` (dark, not blue) with an `icon-inverse` mark.
        let on = look(CheckboxState::Checked);
        assert_eq!(hex(on.fill), "#161616");
        assert_eq!(on.border, None);
        assert_eq!(hex(on.mark), "#ffffff");
        assert_eq!(look(CheckboxState::Indeterminate).fill, on.fill);
        let invalid = CheckboxLook::of(CheckboxLookState {
            value: CheckboxState::Checked,
            invalid: true,
            ..Default::default()
        });
        assert_eq!(invalid.border.map(hex).as_deref(), Some("#da1e28"));
        let disabled = CheckboxLook::of(CheckboxLookState {
            value: CheckboxState::Checked,
            disabled: true,
            ..Default::default()
        });
        assert!(disabled.fill.a < 1.);
        assert!(disabled.label.a < 1.);
        let readonly = CheckboxLook::of(CheckboxLookState {
            value: CheckboxState::Checked,
            readonly: true,
            ..Default::default()
        });
        assert_eq!(readonly.fill.a, 0.);
        assert_eq!(hex(readonly.mark), "#161616");
    }

    #[test]
    fn geometry_follows_checkbox_label_css() {
        assert_eq!(BOX, 16.);
        // 0.125rem + 0.0625rem margin; 0.1875rem inline margin.
        assert_eq!(BOX_INSET, 3.);
        // 1.25rem + 0.625rem.
        assert_eq!(LABEL_START, 30.);
    }

    #[test]
    fn focus_ring_sits_one_px_outside_the_box() {
        // `outline: 2px solid var(--focus); outline-offset: 1px`: the ring spans
        // 1–3px outside the border edge. Unchecked boxes have a 1px border, so
        // from their padding edge the ring starts 4px out.
        assert_eq!(focus_ring_inset(true), -4.);
        assert_eq!(focus_ring_inset(false), -3.);
    }
}
