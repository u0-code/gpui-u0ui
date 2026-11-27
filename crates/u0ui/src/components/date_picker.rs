//! DatePicker: shadcn `Date Picker` → Carbon `DatePicker` / `DatePickerInput`
//! (`.cds--date-picker`, flatpickr underneath).
//!
//! - **Behavior**: the date text is Kit's [`InputState`] editor (typing, IME,
//!   paste): Enter or leaving the field parses it with `dateFormat`. Focusing
//!   or pressing a field (or the calendar icon) opens the calendar panel from
//!   [`super::calendar`] under the fields; picking a date (both ends for
//!   `range`) closes it, as do Escape and a press outside. The panel is placed
//!   by Kit's [`gpui_kit::base::Popup`] under the fields, or above them when
//!   it only fits there (flatpickr's `position: "auto"`), never over them
//!   ([`super::overlay::DropSide`]).
//! - **Keyboard** (Carbon `DatePicker`'s input and calendar `keydown`
//!   handlers): Tab from an open field moves into the calendar (its
//!   selected day, or today); Shift+Tab there returns to that field. Tab in
//!   the calendar goes on to the range's end field when it came from the
//!   start, else it closes the calendar and moves to the stop after the
//!   fields. Focus leaving the fields and the calendar any other way (Shift+Tab
//!   from a field, a click elsewhere) closes it. The Tab bindings come from
//!   [`init`] and beat Base's window-level Tab.
//! - **Appearance**: Carbon date inputs in IBM Plex Mono (`code-02`) on the
//!   field (`field` fill, `border-strong` rule, 2px `focus` outline, invalid
//!   2px `support-error`), 16px calendar icon 1rem from the end
//!   (`.cds--date-picker__icon`), replaced by the error/warning icon when
//!   invalid/warn; single 18rem wide, range two 8.96875rem fields 1px apart,
//!   `simple` a 7.5rem field without a calendar. Disabled: no rule,
//!   `text-disabled`; read-only: transparent with `border-subtle`.
//! - **Motion**: the panel fades in while sliding 20px down over 110ms
//!   (`.flatpickr-calendar.animate.open { animation: fp-fade-in-down 110ms
//!   cubic-bezier(0, 0, 0.38, 0.9) }`); the slide is dropped under reduced
//!   motion. Closing has no animation: flatpickr removes `.open` and the panel
//!   is gone at once.
//!
//! ```ignore
//! DatePicker::new("start").label("Start date").placeholder("mm/dd/yyyy")
//! ```
use super::calendar::{self, CalendarDate, CalendarLocale, CalendarMode, Panel, Selection};
use super::form::{self, FieldSize, Validation, rem};
use super::input::{FIELD_MOTION, FieldLook, FieldState, InputState};
use super::overlay::DropSide;
use crate::{
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnimationExt as _, App, AppContext as _, CursorStyle, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    Styled, Window,
    component::{
        Sizable as _, Size,
        input::{Enter, Escape, Input as KitInput},
    },
    div,
    prelude::FluentBuilder as _,
    px,
};
use gpui_kit::{FocusHandle, KeyBinding, TestSupportExt as _, actions};
use std::rc::Rc;

actions!(
    u0ui_date_picker,
    [
        /// Tab from a field while the calendar is open: into the calendar.
        IntoCalendar,
        /// Tab in the calendar: on to the end field, or close and move on.
        CalendarTab,
        /// Shift+Tab in the calendar: back to the field it came from.
        CalendarBack
    ]
);

/// The key context of a field while its calendar is open.
const FIELD_CONTEXT: &str = "U0DateField";
/// The key context of the open calendar.
const CALENDAR_CONTEXT: &str = "U0DateCalendar";

/// Bind Tab and Shift+Tab between the fields and the calendar. A binding in
/// the element's own key context is deeper than Base's window-level
/// `root::Tab`, so it wins. Called by [`crate::init`].
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", IntoCalendar, Some(FIELD_CONTEXT)),
        KeyBinding::new("tab", CalendarTab, Some(CALENDAR_CONTEXT)),
        KeyBinding::new("shift-tab", CalendarBack, Some(CALENDAR_CONTEXT)),
    ]);
}

/// `.flatpickr-calendar.animate.open { animation: fp-fade-in-down 110ms cubic-bezier(0, 0, 0.38, 0.9) }`.
pub const PANEL_MOTION: Motion = Motion::new(Curve::ProductiveEntrance, Speed::Fast02);
/// `fp-fade-in-down`: `translate3d(0, -20px, 0)` → 0.
pub const PANEL_TRAVEL: f32 = 20.;

/// The Popup offset placing the calendar on `side` of the input, given the
/// label over it (`label_inset` px between the measured row's top and the
/// input's). Below it hangs flush from the row's (= the input's) bottom:
/// flatpickr's `+ 2` less the 2px `margin-block-start: -0.125rem`. Above, its
/// bottom edge sits `2 + 2` px over the input's top (`input.top - height - 2`,
/// then the same margin).
pub fn above_offset(side: DropSide, label_inset: f32) -> f32 {
    match side {
        DropSide::Below => 0.,
        // Base's `BottomLeft` anchor puts the surface's bottom at
        // `row.top - offset`.
        DropSide::Above => 4. - label_inset,
    }
}

/// `index.d.ts` `DatePickerProps.mode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DatePickerMode {
    /// One date with a calendar (`--single`).
    #[default]
    Single,
    /// Start and end fields with one calendar (`--range`).
    Range,
    /// A bare date field, no calendar (`--simple`).
    Simple,
}

impl DatePickerMode {
    /// Every mode.
    pub const ALL: [Self; 3] = [Self::Single, Self::Range, Self::Simple];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Range => "range",
            Self::Simple => "simple",
        }
    }

    /// Field width in px: `18rem` single, `8.96875rem` per range field, `7.5rem` simple.
    pub const fn field_width(self) -> f32 {
        match self {
            Self::Single => 288.,
            Self::Range => 143.5,
            Self::Simple => 120.,
        }
    }
}

type ChangeHandler = Rc<dyn Fn(Selection, &mut Window, &mut App)>;

struct Fields {
    inputs: [Entity<InputState>; 2],
    was_focused: [bool; 2],
    /// The field Tab last entered the calendar from.
    origin: usize,
    /// Focus has been in the fields or the open calendar since it opened.
    had_focus: bool,
    open: bool,
    /// Counts openings, so each one replays the entrance animation.
    opened: u64,
}

/// A Carbon date picker. See the module docs.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    mode: DatePickerMode,
    label: Option<SharedString>,
    end_label: Option<SharedString>,
    placeholder: Option<SharedString>,
    value: Option<Selection>,
    default_value: Selection,
    min: Option<CalendarDate>,
    max: Option<CalendarDate>,
    format: SharedString,
    locale: CalendarLocale,
    helper_text: Option<SharedString>,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    size: FieldSize,
    density: Option<Density>,
    disabled: bool,
    readonly: bool,
    level: LayerLevel,
    on_value_change: Option<ChangeHandler>,
}

impl DatePicker {
    /// A single date picker with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            mode: DatePickerMode::Single,
            label: None,
            end_label: None,
            placeholder: None,
            value: None,
            default_value: Selection::default(),
            min: None,
            max: None,
            format: "m/d/Y".into(),
            locale: CalendarLocale::english(),
            helper_text: None,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            size: FieldSize::Md,
            density: None,
            disabled: false,
            readonly: false,
            level: LayerLevel::One,
            on_value_change: None,
        }
    }

    /// `mode`.
    pub fn mode(mut self, mode: DatePickerMode) -> Self {
        self.mode = mode;
        self
    }

    /// `label` (the start field's label in `range`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The end field's label in `range`.
    pub fn end_label(mut self, label: impl Into<SharedString>) -> Self {
        self.end_label = Some(label.into());
        self
    }

    /// `placeholder` (the reference uses `mm/dd/yyyy`).
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Controlled `value`.
    pub fn value(mut self, value: Selection) -> Self {
        self.value = Some(value);
        self
    }

    /// The initial value of an uncontrolled picker.
    pub fn default_value(mut self, value: Selection) -> Self {
        self.default_value = value;
        self
    }

    /// `minDate`.
    pub fn min_date(mut self, date: CalendarDate) -> Self {
        self.min = Some(date);
        self
    }

    /// `maxDate`.
    pub fn max_date(mut self, date: CalendarDate) -> Self {
        self.max = Some(date);
        self
    }

    /// `dateFormat` in flatpickr tokens (default `m/d/Y`).
    pub fn date_format(mut self, format: impl Into<SharedString>) -> Self {
        self.format = format.into();
        self
    }

    /// Month and weekday names.
    pub fn locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
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

    /// `size` (`.cds--date-picker__input--sm/lg`).
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Override the app density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
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

    /// The layer the fields sit on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// `onValueChange`.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(Selection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

fn set_open(fields: &Entity<Fields>, open: bool, cx: &mut App) {
    fields.update(cx, |f, cx| {
        if f.open != open {
            f.open = open;
            if open {
                f.opened += 1;
            }
            cx.notify();
        }
    });
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let mode = self.mode;
        let range = mode == DatePickerMode::Range;
        let has_calendar = mode != DatePickerMode::Simple;
        let density = self.density.unwrap_or_else(density::current);
        let height = self.size.height(density);
        let format = self.format.clone();
        let (selection, store) = form::controlled(
            (id.clone(), "value"),
            self.value,
            self.default_value,
            window,
            cx,
        );
        let fields = window.use_keyed_state((id.clone(), "fields"), cx, |window, cx| Fields {
            inputs: [
                cx.new(|cx| InputState::new(window, cx)),
                cx.new(|cx| InputState::new(window, cx)),
            ],
            was_focused: [false, false],
            origin: 0,
            had_focus: false,
            open: false,
            opened: 0,
        });
        let inputs = fields.read(cx).inputs.clone();
        let inert = self.disabled || self.readonly;
        let on_change = self.on_value_change.clone();
        let commit: ChangeHandler = Rc::new({
            let store = store.clone();
            move |next, window, cx| {
                store.update(cx, |v, cx| {
                    *v = next;
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(next, window, cx);
                }
            }
        });

        // Text ↔ value: parse on blur, show the value while not editing, and open
        // the calendar when a field gains focus.
        let count = if range { 2 } else { 1 };
        for (k, editor) in inputs.iter().enumerate().take(count) {
            let editor = editor.clone();
            let focused = editor.read(cx).focus_handle(cx).is_focused(window);
            let was = fields.read(cx).was_focused[k];
            if was && !focused {
                parse_into(&editor, k, range, selection, &format, &commit, window, cx);
            }
            if was != focused {
                fields.update(cx, |f, _| {
                    f.was_focused[k] = focused;
                    if focused {
                        f.origin = k;
                    }
                });
                if focused && has_calendar && !inert {
                    set_open(&fields, true, cx);
                }
            }
            let shown = if k == 0 {
                selection.start
            } else {
                selection.end
            }
            .map(|d| d.format(&format))
            .unwrap_or_default();
            if !focused && editor.read(cx).value() != shown.as_str() {
                editor.update(cx, |e, cx| e.set_value(shown, window, cx));
            }
        }
        let open = fields.read(cx).open && has_calendar && !inert;
        let calendar_id: ElementId = (id.clone(), "calendar").into();
        let today = CalendarDate::today();
        let calendar =
            calendar::view_state(&calendar_id, selection.start.unwrap_or(today), window, cx);
        let calendar_focus = calendar.read(cx).focus.clone();
        let focuses: Vec<FocusHandle> = inputs
            .iter()
            .take(count)
            .map(|i| i.read(cx).focus_handle(cx))
            .collect();
        // Focus leaving the fields and the open calendar closes it (Carbon's
        // calendar closes on blur), once focus has been inside.
        // Anything inside the calendar counts (its month arrows take focus
        // on a press).
        let inside = focuses.iter().any(|f| f.is_focused(window))
            || calendar_focus.contains_focused(window, cx);
        let had_focus = fields.read(cx).had_focus;
        if !open || inside {
            if had_focus != (open && inside) {
                fields.update(cx, |f, _| f.had_focus = open && inside);
            }
        } else if had_focus {
            fields.update(cx, |f, _| f.had_focus = false);
            set_open(&fields, false, cx);
        }
        let opened = fields.read(cx).opened;
        let slide = !motion::reduced_motion(cx);
        let anchor = super::overlay::drop_anchor((id.clone(), "drop"), window, cx);

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
        let mut look = FieldLook::of(
            self.level,
            FieldState {
                focused: false,
                disabled: self.disabled,
                readonly: self.readonly,
            },
        );
        if self.disabled {
            // `.cds--date-picker__input:disabled { border-block-end: 1px solid transparent }`.
            look.border_bottom = Hsla::transparent_black();
        }
        let labels = [self.label.clone(), self.end_label.clone()];

        let field = |k: usize, window: &mut Window, cx: &mut App| {
            let editor = inputs[k].clone();
            let focus = editor.read(cx).focus_handle(cx);
            let focused = focus.is_focused(window) && !self.disabled;
            let empty = editor.read(cx).value().is_empty();
            let outline =
                form::field_outline(focused, &validation).unwrap_or(Hsla::transparent_black());
            let outline = motion::animate_color(
                ((id.clone(), format!("field-{k}")), "outline"),
                outline,
                FIELD_MOTION,
                window,
                cx,
            );
            let width = mode.field_width();
            // `.cds--date-picker__icon ~ .cds--date-picker__input { padding-inline-end: 3rem }`.
            let pad_end = if has_calendar || validation.has_icon() {
                48.
            } else {
                16.
            };
            let icon = if validation.has_icon() {
                form::status_icon(&validation)
            } else if has_calendar {
                Some(
                    Icon::carbon(CarbonIcon::Calendar)
                        .size(IconSize::S16)
                        .color(color(if inert {
                            ColorRole::IconDisabled
                        } else {
                            ColorRole::IconPrimary
                        }))
                        .into_any_element(),
                )
            } else {
                None
            };
            let commit = commit.clone();
            let enter_editor = editor.clone();
            let format = format.clone();
            let open_fields = fields.clone();
            let escape_fields = fields.clone();
            let enter_fields = fields.clone();
            let input = div()
                .id((id.clone(), format!("input-{k}")))
                .relative()
                .flex()
                .items_center()
                .w(rem(width))
                .h(rem(height))
                .pl(rem(16.))
                .pr(rem(pad_end))
                .bg(look.background)
                .pb(gpui_kit::px(form::FIELD_RULE))
                .font(crate::typography::font(tokens::FontFamilyKey::Mono, 400))
                .text_size(rem(14.))
                .line_height(rem(20.))
                .text_color(look.text)
                .cursor(if self.disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::IBeam
                })
                .when(!self.disabled, |this| {
                    let focus = focus.clone();
                    this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        focus.focus(window, cx);
                        if has_calendar && !inert {
                            set_open(&open_fields, true, cx);
                        }
                    })
                })
                .capture_action(move |action: &Enter, window, cx| {
                    if !action.secondary {
                        parse_into(
                            &enter_editor,
                            k,
                            range,
                            selection,
                            &format,
                            &commit,
                            window,
                            cx,
                        );
                        set_open(&enter_fields, false, cx);
                    }
                })
                .when(open, |this| {
                    let calendar = calendar.clone();
                    let fields = fields.clone();
                    this.key_context(FIELD_CONTEXT).on_action(
                        move |_: &IntoCalendar, window, cx| {
                            fields.update(cx, |f, _| f.origin = k);
                            let day = if k == 0 {
                                selection.start
                            } else {
                                selection.end.or(selection.start)
                            };
                            let focus = calendar.update(cx, |c, cx| {
                                c.cursor = Some(day.unwrap_or(today));
                                c.view = c.cursor.unwrap_or(c.view);
                                cx.notify();
                                c.focus.clone()
                            });
                            focus.focus(window, cx);
                            cx.stop_propagation();
                        },
                    )
                })
                .capture_action(move |_: &Escape, _, cx| {
                    if escape_fields.read(cx).open {
                        set_open(&escape_fields, false, cx);
                        cx.stop_propagation();
                    }
                })
                .child(
                    KitInput::new(&editor)
                        .appearance(false)
                        .with_size(Size::XSmall)
                        .disabled(self.disabled)
                        .readonly(self.readonly)
                        .when_some(labels[k].clone(), |this, label| this.aria_label(label))
                        .p_0()
                        .h_full()
                        .flex_1()
                        .min_w_0()
                        .text_color(look.text),
                )
                .when_some(self.placeholder.clone().filter(|_| empty), |this, text| {
                    this.child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(rem(16.))
                            .flex()
                            .items_center()
                            .text_color(look.placeholder)
                            .child(text),
                    )
                })
                .children(icon.map(|icon| {
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .right(rem(16.))
                        .flex()
                        .items_center()
                        .child(icon)
                }))
                .child(form::bottom_rule(look.border_bottom))
                .child(form::inset_outline(2., outline));
            // `.cds--date-picker-container`: the label over its field.
            div()
                .flex()
                .flex_col()
                .when_some(
                    labels[k].clone().filter(|l| !l.is_empty()),
                    |this, label| this.child(form::label(label, self.disabled)),
                )
                .child(input)
        };

        let mut row = div()
            .relative()
            .flex()
            .flex_row()
            .items_end()
            .child(anchor.measure_trigger());
        row = row.child(field(0, window, cx));
        if range {
            // `.cds--date-picker--range > .cds--date-picker-container:first-child { margin-inline-end: 0.0625rem }`.
            row = row.child(div().w(px(1.))).child(field(1, window, cx));
        }

        // flatpickr positions the calendar against the input, not the label
        // over it: below, `top = input.bottom + 2`; above,
        // `top = input.top - height - 2`; and
        // `.flatpickr-calendar.open { margin-block-start: -0.125rem }` lifts
        // both by 2px: flush below, 4px clear of the input above.
        let input_height = height / 16. * f32::from(window.rem_size());
        let label_inset = (anchor.height() - input_height).max(0.);
        anchor.set_top_inset(label_inset);
        let side = anchor.place_measured(window);
        let popup = gpui_kit::base::Popup::new((id.clone(), "popup"), row)
            .anchor(side.anchor())
            .offset(px(above_offset(side, label_inset)))
            .when(open, |this| {
                let commit = commit.clone();
                let pick_fields = fields.clone();
                let pick_calendar = calendar_focus.clone();
                let pick_focuses = focuses.clone();
                let close_fields = fields.clone();
                let panel = calendar::panel(
                    Panel {
                        id: (id.clone(), "calendar").into(),
                        mode: if range {
                            CalendarMode::Range
                        } else {
                            CalendarMode::Single
                        },
                        selection,
                        min: self.min,
                        max: self.max,
                        locale: self.locale.clone(),
                        inline: false,
                        prev_label: None,
                        next_label: None,
                        on_pick: Rc::new(move |next, window, cx| {
                            commit(next, window, cx);
                            let mode = if range {
                                CalendarMode::Range
                            } else {
                                CalendarMode::Single
                            };
                            if next.complete(mode) {
                                set_open(&pick_fields, false, cx);
                                // A pick in the calendar would leave focus on
                                // it as it goes: back to the field it came
                                // from (without reopening it), so a popover
                                // holding the picker keeps focus inside.
                                if pick_calendar.contains_focused(window, cx) {
                                    let k = pick_fields.update(cx, |f, _| {
                                        let k = f.origin.min(pick_focuses.len() - 1);
                                        f.was_focused[k] = true;
                                        k
                                    });
                                    pick_focuses[k].focus(window, cx);
                                }
                            }
                        }),
                    },
                    window,
                    cx,
                )
                // `.flatpickr-calendar.open { box-shadow: 0 2px 6px var(--shadow) }`.
                .shadow(super::overlay::floating_shadow())
                .on_mouse_down_out(move |_, _, cx| set_open(&close_fields, false, cx))
                .key_context(CALENDAR_CONTEXT)
                .on_action({
                    let fields = fields.clone();
                    let focuses = focuses.clone();
                    let calendar_focus = calendar_focus.clone();
                    move |_: &CalendarTab, window, cx| {
                        cx.stop_propagation();
                        let origin = fields.read(cx).origin;
                        if origin + 1 < focuses.len() {
                            focuses[origin + 1].focus(window, cx);
                            return;
                        }
                        set_open(&fields, false, cx);
                        super::overlay::tab_past(
                            &focuses[focuses.len() - 1],
                            &calendar_focus,
                            window,
                            cx,
                        );
                    }
                })
                .on_action({
                    let fields = fields.clone();
                    let focuses = focuses.clone();
                    move |_: &CalendarBack, window, cx| {
                        cx.stop_propagation();
                        let origin = fields.read(cx).origin.min(focuses.len() - 1);
                        focuses[origin].focus(window, cx);
                    }
                })
                .test_support();
                let measure = anchor.measure_surface();
                this.content(div().relative().child(panel).child(measure).with_animation(
                    (id.clone(), format!("open-{opened}")),
                    PANEL_MOTION.animation(),
                    move |this, t| {
                        let travel = if slide { -PANEL_TRAVEL * (1. - t) } else { 0. };
                        this.opacity(t).top(px(travel))
                    },
                ))
            });

        // `.cds--form-item { flex: 1 1 auto }`: as wide as its fields, so two
        // pickers in a `cu-row` sit side by side instead of each taking the row.
        form::form_item()
            .w_auto()
            .child(popup)
            .children(form::message(
                &validation,
                self.helper_text.as_ref(),
                self.disabled,
            ))
            .type_style(TypeStyle::BodyCompact01)
    }
}

/// Parse field `k`'s text into the selection (start or end) and commit it.
#[allow(clippy::too_many_arguments)]
fn parse_into(
    editor: &Entity<InputState>,
    k: usize,
    range: bool,
    selection: Selection,
    format: &str,
    commit: &ChangeHandler,
    window: &mut Window,
    cx: &mut App,
) {
    let text = editor.read(cx).value();
    if text.trim().is_empty() {
        return;
    }
    let Some(date) = CalendarDate::parse(&text, format) else {
        return;
    };
    let next = match (range, k) {
        (false, _) => Selection::single(date),
        (true, 0) => match selection.end {
            Some(end) => Selection::range(date, end),
            None => Selection::single(date),
        },
        (true, _) => match selection.start {
            Some(start) => Selection::range(start, date),
            None => Selection {
                start: None,
                end: Some(date),
            },
        },
    };
    if next != selection {
        commit(next, window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_follow_date_picker_css() {
        // `--single .cds--date-picker__input { inline-size: 18rem }`.
        assert_eq!(DatePickerMode::Single.field_width(), 18. * 16.);
        // `--range … { inline-size: 8.96875rem }`.
        assert_eq!(DatePickerMode::Range.field_width(), 8.96875 * 16.);
        // `--simple … { inline-size: 7.5rem }`.
        assert_eq!(DatePickerMode::Simple.field_width(), 7.5 * 16.);
        assert_eq!(PANEL_TRAVEL, 20.);
        assert_eq!(PANEL_MOTION.speed.millis(), 110);
    }

    #[test]
    fn calendar_anchors_to_the_input_not_its_label() {
        // Below: flush with the row's bottom, which is the input's.
        assert_eq!(above_offset(DropSide::Below, 24.), 0.);
        // Above, under a 16px label + 8px gap: the surface's bottom lands at
        // `row.top + 24 - 4`, i.e. 4px over the input's top edge.
        let row_top = 100.;
        let bottom = row_top - above_offset(DropSide::Above, 24.);
        assert_eq!(bottom, row_top + 24. - 4.);
        // Without a label the input is the row: 4px clear of it.
        assert_eq!(row_top - above_offset(DropSide::Above, 0.), row_top - 4.);
    }
}
