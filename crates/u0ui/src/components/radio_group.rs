//! RadioGroup: shadcn `RadioGroup` / `RadioGroupItem` → Carbon
//! `RadioButtonGroup` / `RadioButton` (`.cds--radio-button*`).
//!
//! - **Behavior** is Kit's: [`gpui_kit::base::RadioGroup`] (role, orientation)
//!   around [`gpui_kit::base::Radio`]s (focus, Space/Enter and pointer
//!   activation, position in set). Kit's primitives leave arrow keys to the
//!   app, so the group adds the native radio keyboard: roving focus (only the
//!   selected radio is a tab stop) and ←/↑ / →/↓ moving the selection, wrapping
//!   and skipping disabled items.
//! - **Appearance**: an 18px circle with a 1px `icon-primary` border, the
//!   checked dot `icon-primary` at half size (`transform: scale(0.5)`), the
//!   label `body-compact-01` 10px after it; horizontal items 16px apart,
//!   vertical ones 6px; the `legend` is a `.cds--label`. Focus: 2px `focus`
//!   ring 1.5px outside the circle. Disabled `icon-disabled`/`text-disabled`;
//!   read-only `icon-disabled` circle; invalid `support-error` circles plus the
//!   error icon and message; warn the warning icon and message.
//! - **Motion**: none; the dot appears instantly (`.cds--radio-button__appearance`
//!   has no `transition` in `bundle.css`).
//!
//! ```ignore
//! RadioGroup::new("plan").legend("Plan").default_value("std")
//!     .item(RadioGroupItem::new("lite").label("Lite"))
//!     .item(RadioGroupItem::new("std").label("Standard"))
//! ```
use super::form::{self, Validation, rem};
use crate::{
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, Axis, CursorStyle, ElementId, FocusHandle, Hsla, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `index.d.ts` `RadioGroupProps.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RadioOrientation {
    /// In a row (`.cds--radio-button-group`), the reference default.
    #[default]
    Horizontal,
    /// In a column (`.cds--radio-button-group--vertical`).
    Vertical,
}

impl RadioOrientation {
    /// Both orientations.
    pub const ALL: [Self; 2] = [Self::Horizontal, Self::Vertical];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// One option (`RadioGroupItem`).
pub struct RadioGroupItem {
    value: SharedString,
    label: Option<SharedString>,
    disabled: bool,
}

impl RadioGroupItem {
    /// An option with its `value`.
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
        }
    }

    /// The label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// The state a radio look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RadioState {
    /// Selected.
    pub checked: bool,
    /// Disabled.
    pub disabled: bool,
    /// Read-only group.
    pub readonly: bool,
    /// Invalid group.
    pub invalid: bool,
}

/// The colors of one radio button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioLook {
    /// Circle border.
    pub border: Hsla,
    /// Checked dot.
    pub dot: Hsla,
    /// Label.
    pub label: Hsla,
}

impl RadioLook {
    /// The look of a radio (`.cds--radio-button*` rules).
    pub fn of(state: RadioState) -> Self {
        if state.disabled {
            // `:disabled + … .cds--radio-button__appearance { border-color: var(--icon-disabled) }`,
            // `::before { background-color: var(--text-disabled) }`.
            return Self {
                border: color(ColorRole::IconDisabled),
                dot: color(ColorRole::TextDisabled),
                label: color(ColorRole::TextDisabled),
            };
        }
        let border = if state.readonly {
            // `.cds--radio-button-group--readonly … { border-color: var(--icon-disabled) }`.
            color(ColorRole::IconDisabled)
        } else if state.invalid {
            // `.cds--radio-button-group--invalid … { border-color: var(--support-error) }`.
            color(ColorRole::SupportError)
        } else {
            color(ColorRole::IconPrimary)
        };
        Self {
            border,
            dot: color(ColorRole::IconPrimary),
            label: color(ColorRole::TextPrimary),
        }
    }
}

/// The next enabled index from `current` by `delta`, wrapping.
pub fn step(current: usize, delta: isize, disabled: &[bool]) -> Option<usize> {
    super::toggle_group::step(current, delta, disabled)
}

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type CommitFn = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// A Carbon radio button group. See the module docs.
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    legend: Option<SharedString>,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    orientation: RadioOrientation,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    helper_text: Option<SharedString>,
    items: Vec<RadioGroupItem>,
    on_value_change: Option<ChangeHandler>,
}

impl RadioGroup {
    /// A group with a stable `id` (Carbon's `name`).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            legend: None,
            value: None,
            default_value: None,
            orientation: RadioOrientation::Horizontal,
            disabled: false,
            readonly: false,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            helper_text: None,
            items: Vec::new(),
            on_value_change: None,
        }
    }

    /// `legend`.
    pub fn legend(mut self, legend: impl Into<SharedString>) -> Self {
        self.legend = Some(legend.into());
        self
    }

    /// Controlled `value`.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `defaultValue` (Carbon: a group always has a default selection).
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// `onValueChange`.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }

    /// `orientation`.
    pub fn orientation(mut self, orientation: RadioOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disable every option.
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

    /// Helper text under the group.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Append an option.
    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.items.push(item);
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let vertical = self.orientation == RadioOrientation::Vertical;
        let (value, store) = form::controlled(
            (id.clone(), "value"),
            self.value.clone(),
            self.default_value.clone().unwrap_or_default(),
            window,
            cx,
        );
        let count = self.items.len();
        let focus_store =
            window.use_keyed_state((id.clone(), "focus"), cx, |_, _| Vec::<FocusHandle>::new());
        focus_store.update(cx, |handles, cx| {
            while handles.len() < count {
                handles.push(cx.focus_handle());
            }
        });
        let focus = focus_store.read(cx).clone();
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
        let disabled: Vec<bool> = self
            .items
            .iter()
            .map(|i| i.disabled || self.disabled || self.readonly)
            .collect();
        let values: Vec<SharedString> = self.items.iter().map(|i| i.value.clone()).collect();
        let selected = values.iter().position(|v| *v == value);
        let on_change = self.on_value_change.clone();
        let commit: CommitFn = Rc::new({
            move |next, window, cx| {
                store.update(cx, |v, cx| {
                    *v = next.clone();
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(&next, window, cx);
                }
            }
        });
        // The roving tab stop: the selected radio, else the first enabled one.
        let tab_stop = selected
            .filter(|i| !disabled[*i])
            .or_else(|| disabled.iter().position(|d| !d));

        let on_key = {
            let commit = commit.clone();
            let focus = focus.clone();
            let disabled = disabled.clone();
            let values = values.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let delta = match event.keystroke.key.as_str() {
                    "left" | "up" => -1,
                    "right" | "down" => 1,
                    _ => return,
                };
                let Some(current) = focus.iter().position(|f| f.is_focused(window)) else {
                    return;
                };
                cx.stop_propagation();
                if let Some(next) = step(current, delta, &disabled) {
                    focus[next].focus(window, cx);
                    commit(values[next].clone(), window, cx);
                }
            }
        };

        let group = gpui_kit::base::RadioGroup::new((id.clone(), "group"))
            .axis(if vertical {
                Axis::Vertical
            } else {
                Axis::Horizontal
            })
            .flex()
            .when(vertical, |this| this.flex_col().items_start().gap(rem(6.)))
            .when(!vertical, |this| {
                this.flex_row().flex_wrap().items_center().gap(rem(16.))
            })
            .on_key_down(on_key)
            .children(self.items.into_iter().enumerate().map(|(i, item)| {
                let checked = selected == Some(i);
                let item_disabled = item.disabled || self.disabled;
                let look = RadioLook::of(RadioState {
                    checked,
                    disabled: item_disabled,
                    readonly: self.readonly,
                    invalid: validation.is_invalid(),
                });
                let focused = focus[i].is_focused(window) && !item_disabled;
                // `.cds--radio-button:checked + … .cds--radio-button__appearance::before
                // { transform: scale(0.5) }`, shown at once.
                let dot = if checked { 0.5f32 } else { 0. };
                let commit = commit.clone();
                let value = item.value.clone();
                let circle = div()
                    .relative()
                    .flex_shrink_0()
                    .size(rem(18.))
                    .rounded_full()
                    .border_1()
                    .border_color(look.border)
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(dot > 0., |this| {
                        // 16px inside the border, scaled.
                        this.child(div().size(rem(16. * dot)).rounded_full().bg(look.dot))
                    })
                    // `:focus + … .cds--radio-button__appearance { outline: 2px solid
                    // var(--focus); outline-offset: 1.5px }`.
                    .when(focused, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top(px(-4.5))
                                .left(px(-4.5))
                                .right(px(-4.5))
                                .bottom(px(-4.5))
                                .rounded_full()
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    });
                gpui_kit::base::Radio::new((id.clone(), format!("radio-{i}")))
                    .track_focus(&focus[i])
                    .checked(checked)
                    .disabled(item_disabled || self.readonly)
                    .tab_stop(tab_stop == Some(i))
                    .set_position(i + 1, count)
                    .when_some(item.label.clone(), |this, label| {
                        this.accessibility_label(label)
                    })
                    .flex()
                    .flex_row()
                    .items_start()
                    .when(vertical, |this| this.pt(rem(2.)))
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(look.label)
                    .cursor(if item_disabled {
                        CursorStyle::OperationNotAllowed
                    } else if self.readonly {
                        CursorStyle::Arrow
                    } else {
                        CursorStyle::PointingHand
                    })
                    .on_change(move |_, _, window, cx| commit(value.clone(), window, cx))
                    // `.cds--radio-button__appearance { margin-block: 0.0625rem 0.125rem;
                    // margin-inline: 0.125rem 0.625rem }` (vertical: no block margin).
                    .child(
                        div()
                            .when(!vertical, |this| this.mt(rem(1.)).mb(rem(2.)))
                            .ml(rem(2.))
                            .mr(rem(10.))
                            .child(circle),
                    )
                    .when_some(item.label, |this, label| {
                        this.child(div().when(!vertical, |this| this.mt(rem(1.))).child(label))
                    })
            }));

        // `.cds--radio-button__validation-msg { display: flex; margin-block-start: 0.375rem }`.
        let message = validation.message().map(|text| {
            div()
                .flex()
                .flex_row()
                .items_start()
                .mt(rem(6.))
                .children(
                    form::status_icon(&validation)
                        .map(|icon| div().ml(rem(3.)).mr(rem(1.)).child(icon)),
                )
                .child(
                    div()
                        .ml(rem(8.))
                        .type_style(TypeStyle::HelperText01)
                        .text_color(validation.message_color())
                        .child(text.clone()),
                )
        });
        // `.cds--radio-button-group ~ .cds--form__helper-text { margin-block-start: 0.375rem }`.
        let helper = self
            .helper_text
            .clone()
            .filter(|_| message.is_none())
            .map(|text| form::helper_text(text, self.disabled).mt(rem(6.)));

        form::form_item()
            .when_some(self.legend.clone(), |this, legend| {
                this.child(form::label(legend, self.disabled))
            })
            .child(group)
            .children(message)
            .children(helper)
    }
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
    fn colors_follow_radio_button_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = RadioLook::of(RadioState::default());
        assert_eq!(hex(rest.border), "#161616");
        assert_eq!(hex(rest.dot), "#161616");
        let invalid = RadioLook::of(RadioState {
            invalid: true,
            ..Default::default()
        });
        assert_eq!(hex(invalid.border), "#da1e28");
        let disabled = RadioLook::of(RadioState {
            disabled: true,
            invalid: true,
            ..Default::default()
        });
        assert!(disabled.border.a < 1.);
        assert!(disabled.label.a < 1.);
        let readonly = RadioLook::of(RadioState {
            readonly: true,
            ..Default::default()
        });
        assert!(readonly.border.a < 1.);
    }

    #[test]
    fn arrows_skip_disabled_and_wrap() {
        let disabled = [false, false, true];
        assert_eq!(step(1, 1, &disabled), Some(0));
        assert_eq!(step(0, -1, &disabled), Some(1));
    }
}
