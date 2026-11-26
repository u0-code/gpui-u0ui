//! Select: shadcn `Select` → Carbon `Dropdown` (`.cds--dropdown`, `.cds--list-box`).
//!
//! - **Behavior** is Kit's [`gpui_kit::base::Select`] root: the trigger's focus,
//!   combobox semantics, ↑/↓ and Enter/Space opening, Enter confirming,
//!   Escape dismissing. This component adds what Base leaves to the app:
//!   moving the highlight with ↑/↓ (Downshift-style, clamped), type-ahead
//!   (letters jump to the next matching option), pointer picking, closing on
//!   an outside press, and the open/close motion. The menu is placed by Kit's
//!   [`gpui_kit::base::Popup`] ([`super::list_box`]).
//! - **Appearance**: the Carbon list-box field: `field` fill, 1px
//!   `border-strong` rule (`border-subtle` while open), hover `field-hover`,
//!   2px `focus` outline while focused, 2px `support-error` outline + error
//!   icon when invalid, the warning icon when `warn`, a chevron that turns 180°.
//!   `type="inline"` drops the fill and rule and puts the label beside the field.
//!
//! ```ignore
//! Select::new("profile")
//!     .label("Instance profile")
//!     .items([("bx2-2x8", "bx2-2x8 · 2 vCPU / 8 GB"), ("bx2-4x16", "bx2-4x16 · 4 vCPU / 16 GB")])
//!     .default_value("bx2-4x16")
//! ```
use super::form::{self, FieldSize, Validation, rem};
use super::input::FIELD_MOTION;
use super::list_box::{self, ListItem, ListState, MENU_MOTION, Menu};
use crate::{
    density::{self, Density},
    motion,
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Entity, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window,
    base::actions::{SelectDown, SelectUp},
    div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `index.d.ts` `SelectProps.type`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SelectType {
    /// The filled field (`.cds--dropdown`).
    #[default]
    Default,
    /// Borderless, label beside it (`.cds--dropdown--inline`).
    Inline,
}

impl SelectType {
    /// Both types.
    pub const ALL: [Self; 2] = [Self::Default, Self::Inline];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Inline => "inline",
        }
    }
}

/// The state a list-box field look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TriggerState {
    /// Pointer over it.
    pub hovered: bool,
    /// The menu is open.
    pub open: bool,
    /// Disabled.
    pub disabled: bool,
    /// Read-only.
    pub readonly: bool,
    /// `type="inline"`.
    pub inline: bool,
}

/// The colors of a list-box field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerLook {
    /// Fill.
    pub background: Hsla,
    /// Bottom rule.
    pub border: Hsla,
    /// Value text.
    pub text: Hsla,
}

impl TriggerLook {
    /// The look of a list-box field (`.cds--list-box*`, `.cds--dropdown*`).
    pub fn of(level: LayerLevel, state: TriggerState) -> Self {
        let clear = Hsla::transparent_black();
        let text = color(if state.disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextPrimary
        });
        if state.inline {
            // `.cds--dropdown--inline { background-color: transparent; border-block-end-color:
            // transparent }`, `:hover { background-color: var(--layer-hover) }`, open transparent.
            let hover = state.hovered && !state.open && !state.disabled && !state.readonly;
            return Self {
                background: if hover {
                    color(level.layer_hover())
                } else {
                    clear
                },
                border: clear,
                text,
            };
        }
        if state.disabled {
            // `.cds--list-box--disabled { border-block-end-color: transparent }`, `:hover { field }`.
            return Self {
                background: color(level.field()),
                border: clear,
                text,
            };
        }
        if state.readonly {
            // `.cds--dropdown--readonly { background-color: transparent; border-block-end-color: var(--border-subtle) }`.
            return Self {
                background: clear,
                border: color(level.border_subtle()),
                text,
            };
        }
        // `.cds--list-box:hover { field-hover }`, `--expanded:hover { field }`,
        // `--expanded { border-block-end-color: var(--border-subtle) }`.
        Self {
            background: color(if state.hovered && !state.open {
                level.field_hover()
            } else {
                level.field()
            }),
            border: color(if state.open {
                level.border_subtle()
            } else {
                level.border_strong()
            }),
            text,
        }
    }
}

type ChangeHandler = Rc<dyn Fn(Option<&SharedString>, &mut Window, &mut App)>;

/// A Carbon dropdown. See the module docs.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    label: Option<SharedString>,
    items: Vec<ListItem>,
    value: Option<Option<SharedString>>,
    default_value: Option<SharedString>,
    placeholder: Option<SharedString>,
    helper_text: Option<SharedString>,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    size: FieldSize,
    density: Option<Density>,
    kind: SelectType,
    disabled: bool,
    readonly: bool,
    level: LayerLevel,
    native: bool,
    on_value_change: Option<ChangeHandler>,
}

impl Select {
    /// A dropdown with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            items: Vec::new(),
            value: None,
            default_value: None,
            placeholder: None,
            helper_text: None,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            size: FieldSize::Md,
            density: None,
            kind: SelectType::Default,
            disabled: false,
            readonly: false,
            level: LayerLevel::One,
            native: false,
            on_value_change: None,
        }
    }

    /// `label` (Carbon `titleText`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `items`: strings or `(value, label)` pairs.
    pub fn items<I: Into<ListItem>>(mut self, items: impl IntoIterator<Item = I>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// Controlled `value` (`None`: nothing selected).
    pub fn value(mut self, value: Option<impl Into<SharedString>>) -> Self {
        self.value = Some(value.map(Into::into));
        self
    }

    /// `defaultValue`.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// `placeholder` (Carbon `label`), shown when nothing is selected.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// `helperText`.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
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

    /// `size`.
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Override the app density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// `type`.
    pub fn kind(mut self, kind: SelectType) -> Self {
        self.kind = kind;
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

    /// The layer the field sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Draw the `.cds--select-input` field (no expanded state, static arrow)
    /// for [`super::NativeSelect`].
    pub(crate) fn native(mut self, native: bool) -> Self {
        self.native = native;
        self
    }

    /// `onValueChange`.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Option<&SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

fn set_open(state: &Entity<ListState>, open: bool, selected: Option<usize>, cx: &mut App) {
    state.update(cx, |s, cx| {
        if s.open != open {
            s.open = open;
            if open {
                s.highlighted = selected;
                s.hovered = None;
            }
            cx.notify();
        }
    });
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let density = self.density.unwrap_or_else(density::current);
        let inline = self.kind == SelectType::Inline;
        let height = if inline {
            32.
        } else {
            self.size.height(density)
        };
        let row = list_box::row_height(self.size.height(density), density);
        let (value, store) = form::controlled(
            (id.clone(), "value"),
            self.value.clone(),
            self.default_value.clone(),
            window,
            cx,
        );
        let state = window.use_keyed_state((id.clone(), "list"), cx, |_, _| ListState::default());
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let inert = self.disabled || self.readonly;
        let items = Rc::new(self.items.clone());
        let selected = value
            .as_ref()
            .and_then(|v| items.iter().position(|i| &i.value == v));
        let (open, highlighted, hover_row, anchor) = {
            let s = state.read(cx);
            (s.open && !inert, s.highlighted, s.hovered, s.anchor.clone())
        };
        let progress = motion::animate(
            (id.clone(), "menu"),
            if open { 1f32 } else { 0. },
            MENU_MOTION,
            window,
            cx,
        );
        let focused = focus.is_focused(window);
        let validation = if inert {
            Validation::None
        } else {
            Validation::from_flags(
                self.invalid,
                self.invalid_text.clone(),
                self.warn,
                self.warn_text.clone(),
            )
        };
        let look = TriggerLook::of(
            self.level,
            TriggerState {
                hovered: *hovered.read(cx),
                // `.cds--select-input` has no expanded state.
                open: open && !self.native,
                disabled: self.disabled,
                readonly: self.readonly,
                inline,
            },
        );
        let background = motion::animate_color(
            (id.clone(), "bg"),
            look.background,
            FIELD_MOTION,
            window,
            cx,
        );
        let outline = form::field_outline(focused && !inert, &validation)
            .unwrap_or(Hsla::transparent_black());

        let on_change = self.on_value_change.clone();
        let pick: list_box::PickFn = Rc::new({
            let state = state.clone();
            let items = items.clone();
            let focus = focus.clone();
            move |i, window, cx| {
                let next = items[i].value.clone();
                store.update(cx, |v, cx| {
                    *v = Some(next.clone());
                    cx.notify();
                });
                set_open(&state, false, None, cx);
                focus.focus(window, cx);
                if let Some(handler) = on_change.as_ref() {
                    handler(Some(&next), window, cx);
                }
            }
        });

        let shown_label = selected
            .map(|i| items[i].label.clone())
            .or(self.placeholder.clone())
            .unwrap_or_default();
        // `.cds--list-box__field { padding-inline: 1rem 3rem }` (+1rem for the status icon).
        let pad_end = if validation.has_icon() { 64. } else { 48. };
        let status = form::status_icon(&validation).map(|icon| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .right(rem(if inline { 32. } else { 40. }))
                .flex()
                .items_center()
                .child(icon)
        });

        let trigger = gpui_kit::base::Select::new((id.clone(), "trigger"))
            .focus_handle(&focus)
            .open(open)
            .disabled(inert)
            .when_some(self.label.clone(), |this, label| {
                this.accessibility_label(label)
            })
            .accessibility_value(shown_label.clone())
            .on_open_change({
                let state = state.clone();
                move |next, _, cx| set_open(&state, next, selected, cx)
            })
            .on_confirm({
                let state = state.clone();
                let pick = pick.clone();
                move |window, cx| {
                    if let Some(i) = state.read(cx).highlighted {
                        pick(i, window, cx);
                    }
                }
            })
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(rem(height))
            .pl(rem(if inline { 12. } else { 16. }))
            .pr(rem(if inline { pad_end - 16. } else { pad_end }))
            .bg(background)
            .pb(gpui_kit::px(form::FIELD_RULE))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(look.text)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else if self.readonly {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            })
            .child(anchor.measure_trigger())
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(shown_label),
            )
            .children(status)
            // `.cds--list-box__menu-icon { inset-inline-end: 0.75rem; 1.5rem box }`.
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right(rem(if inline { 8. } else { 12. }))
                    .w(rem(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(list_box::chevron(
                        id.clone(),
                        open && !self.native,
                        self.disabled || self.readonly,
                        window,
                        cx,
                    )),
            )
            .child(form::bottom_rule(look.border))
            .child(form::inset_outline(2., outline));

        let trigger = div()
            .id((id.clone(), "hit"))
            .w_full()
            .on_hover({
                let hovered = hovered.clone();
                move |over, _, cx| {
                    hovered.update(cx, |h, cx| {
                        if *h != *over {
                            *h = *over;
                            cx.notify();
                        }
                    })
                }
            })
            .when(!inert, |this| {
                let state = state.clone();
                let focus = focus.clone();
                this.on_click(move |_, window, cx| {
                    focus.focus(window, cx);
                    set_open(&state, !open, selected, cx);
                })
            })
            .child(trigger);

        let shown: Vec<usize> = (0..items.len()).collect();
        let (side, room) = anchor.place(list_box::menu_height(&items, &shown, row), window);
        let popup = gpui_kit::base::Popup::new((id.clone(), "popup"), trigger)
            .anchor(side.anchor())
            .w_full()
            .when(open || progress > 0., |this| {
                let selected_flags: Vec<bool> =
                    (0..items.len()).map(|i| Some(i) == selected).collect();
                let menu_width = if inline {
                    anchor.width().clamp(18. * 16., 30. * 16.)
                } else {
                    anchor.width()
                };
                this.content(list_box::menu(
                    Menu {
                        id: id.clone(),
                        items: &items,
                        shown,
                        selected: &selected_flags,
                        highlighted,
                        hovered: hover_row,
                        row_height: row,
                        width: menu_width,
                        max_height: room,
                        level: self.level,
                        checkboxes: false,
                        empty: None,
                        on_pick: pick.clone(),
                        on_hover: Rc::new({
                            let state = state.clone();
                            move |row, _, cx| {
                                state.update(cx, |s, cx| {
                                    if row.is_some() || s.hovered.is_some() {
                                        s.hovered = row;
                                        // Downshift highlights the row under the pointer.
                                        if row.is_some() {
                                            s.highlighted = row;
                                        }
                                        cx.notify();
                                    }
                                })
                            }
                        }),
                        on_close: Rc::new({
                            let state = state.clone();
                            move |_, cx| set_open(&state, false, None, cx)
                        }),
                    },
                    progress,
                ))
            });

        let disabled_flags: Vec<bool> = items.iter().map(|i| i.disabled).collect();
        let field = div()
            .id((id.clone(), "field"))
            .relative()
            .w_full()
            .min_w_0()
            .on_action({
                let state = state.clone();
                let disabled = disabled_flags.clone();
                move |_: &SelectDown, _, cx| {
                    if open {
                        state.update(cx, |s, cx| {
                            s.highlighted = list_box::move_highlight(s.highlighted, 1, &disabled);
                            cx.notify();
                        })
                    }
                }
            })
            .on_action({
                let state = state.clone();
                let disabled = disabled_flags.clone();
                move |_: &SelectUp, _, cx| {
                    if open {
                        state.update(cx, |s, cx| {
                            s.highlighted = list_box::move_highlight(s.highlighted, -1, &disabled);
                            cx.notify();
                        })
                    }
                }
            })
            .on_key_down({
                let state = state.clone();
                let items = items.clone();
                let pick = pick.clone();
                move |event: &KeyDownEvent, window, cx| {
                    let keystroke = &event.keystroke;
                    if inert
                        || keystroke.modifiers.control
                        || keystroke.modifiers.platform
                        || keystroke.modifiers.alt
                    {
                        return;
                    }
                    let Some(ch) = keystroke.key_char.clone().filter(|c| {
                        c.chars().count() == 1 && c.chars().all(|c| c.is_alphanumeric())
                    }) else {
                        return;
                    };
                    let (prefix, from) =
                        state.update(cx, |s, _| (s.push_typed(&ch), s.highlighted.or(selected)));
                    if let Some(i) = list_box::type_ahead(&items, &prefix, from) {
                        cx.stop_propagation();
                        if open {
                            state.update(cx, |s, cx| {
                                s.highlighted = Some(i);
                                cx.notify();
                            });
                        } else {
                            // Closed: type-ahead selects directly, like a native select.
                            pick(i, window, cx);
                        }
                    }
                }
            })
            .child(popup);

        let label = self.label.clone().filter(|l| !l.is_empty());
        if inline {
            // `.cds--dropdown__wrapper--inline { display: inline-grid; gap: 0 0.5rem }`,
            // the label in `body-compact-01`.
            return div()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(rem(8.))
                        .when_some(label, |this, label| {
                            this.child(
                                div()
                                    .type_style(TypeStyle::BodyCompact01)
                                    .text_color(form::label_color(self.disabled))
                                    .whitespace_nowrap()
                                    .child(label),
                            )
                        })
                        .child(div().min_w(rem(8.)).child(field)),
                )
                .children(form::message(
                    &validation,
                    self.helper_text.as_ref(),
                    self.disabled,
                ));
        }
        form::labeled(
            label.as_ref(),
            self.disabled,
            field,
            form::message(&validation, self.helper_text.as_ref(), self.disabled),
        )
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
    fn trigger_colors_follow_list_box_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = TriggerLook::of(LayerLevel::One, TriggerState::default());
        assert_eq!(hex(rest.background), "#f4f4f4");
        assert_eq!(hex(rest.border), "#8d8d8d");
        let hover = TriggerLook::of(
            LayerLevel::One,
            TriggerState {
                hovered: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(hover.background), "#e8e8e8");
        let open = TriggerLook::of(
            LayerLevel::One,
            TriggerState {
                hovered: true,
                open: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(open.background), "#f4f4f4");
        assert_eq!(hex(open.border), "#c6c6c6");
        let disabled = TriggerLook::of(
            LayerLevel::One,
            TriggerState {
                disabled: true,
                ..Default::default()
            },
        );
        assert_eq!(disabled.border.a, 0.);
        let inline = TriggerLook::of(
            LayerLevel::One,
            TriggerState {
                inline: true,
                ..Default::default()
            },
        );
        assert_eq!(inline.background.a, 0.);
        assert_eq!(inline.border.a, 0.);
    }
}
