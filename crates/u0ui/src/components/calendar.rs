//! Calendar: shadcn `Calendar` → Carbon `DatePicker` `inline` (flatpickr's
//! calendar as Carbon styles it, `.flatpickr-*`), plus the date math the
//! [`super::DatePicker`] shares.
//!
//! - **Behavior**: Kit's calendar cannot be restyled to flatpickr's grid, so
//!   the grid is drawn here with its keyboard: the grid is one tab stop; ←/→
//!   move a day, ↑/↓ a week, PageUp/PageDown a month, Home/End to the week's
//!   ends, Enter/Space pick; `range` picks a start then an end. Days outside
//!   `minDate`/`maxDate` are disabled.
//! - **Appearance** (`.flatpickr-*`): an 18rem panel on `layer-01` (inline:
//!   no shadow, a 1px `border-subtle-01` outline, `.cu-calendar`), a 40px month
//!   row with 40×40 prev/next buttons (hover `layer-hover`) and the
//!   `heading-compact-01` month and year, a 40px weekday row, 40×40 days in
//!   `body-compact-01`: other-month days `text-helper`, hover `layer-hover`,
//!   today `link-primary` semibold with a 4×4 dot, selected `button-primary`
//!   with `text-on-color`, range `highlight`, keyboard focus a 2px
//!   `button-primary` outline, disabled `text-disabled`.
//! - **Motion**: day colors over 70ms productive (`transition: all 70ms`).
//!
//! Month and weekday names come from a [`CalendarLocale`] (English by default).
//!
//! ```ignore
//! Calendar::new("due").default_value(CalendarDate::new(2026, 11, 26))
//! ```
use super::form::rem;
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, FocusHandle, Hsla, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `.flatpickr-day { transition: all 70ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const DAY_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);
/// `.flatpickr-day { inline-size: 2.5rem; block-size: 2.5rem }`.
pub const DAY: f32 = 40.;
/// `.flatpickr-calendar.open { inline-size: 18rem }`.
pub const PANEL_WIDTH: f32 = 288.;

// ───────────────────────── Dates ─────────────────────────

/// A calendar date (proleptic Gregorian).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Year.
    pub year: i32,
    /// Month 1–12.
    pub month: u32,
    /// Day 1–31.
    pub day: u32,
}

impl CalendarDate {
    /// A date; out-of-range days clamp to the month's last day.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        let month = month.clamp(1, 12);
        let day = day.clamp(1, days_in_month(year, month));
        Self { year, month, day }
    }

    /// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
    pub fn to_days(self) -> i64 {
        let y = self.year as i64 - if self.month <= 2 { 1 } else { 0 };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01 (`civil_from_days`).
    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = (yoe + era * 400 + if month <= 2 { 1 } else { 0 }) as i32;
        Self { year, month, day }
    }

    /// Today (UTC).
    pub fn today() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self::from_days(secs.div_euclid(86_400))
    }

    /// Day of the week, 0 = Sunday.
    pub fn weekday(self) -> u32 {
        (self.to_days() + 4).rem_euclid(7) as u32
    }

    /// `n` days later (or earlier).
    pub fn add_days(self, n: i64) -> Self {
        Self::from_days(self.to_days() + n)
    }

    /// `n` months later, clamping the day.
    pub fn add_months(self, n: i32) -> Self {
        let index = self.year * 12 + self.month as i32 - 1 + n;
        Self::new(
            index.div_euclid(12),
            (index.rem_euclid(12) + 1) as u32,
            self.day,
        )
    }

    /// The first of this month.
    pub fn first_of_month(self) -> Self {
        Self::new(self.year, self.month, 1)
    }

    /// Format with flatpickr tokens: `m`/`d` (two digits), `n`/`j` (no
    /// padding), `Y` (four digits), `y` (two digits); other characters copy.
    pub fn format(self, pattern: &str) -> String {
        let mut out = String::new();
        for ch in pattern.chars() {
            match ch {
                'm' => out.push_str(&format!("{:02}", self.month)),
                'n' => out.push_str(&self.month.to_string()),
                'd' => out.push_str(&format!("{:02}", self.day)),
                'j' => out.push_str(&self.day.to_string()),
                'Y' => out.push_str(&format!("{:04}", self.year)),
                'y' => out.push_str(&format!("{:02}", self.year.rem_euclid(100))),
                other => out.push(other),
            }
        }
        out
    }

    /// Parse text written in `pattern` (see [`Self::format`]); `None` if it
    /// does not match or the date does not exist.
    pub fn parse(text: &str, pattern: &str) -> Option<Self> {
        let mut numbers = text
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty());
        let (mut year, mut month, mut day) = (None, None, None);
        for token in pattern.chars().filter(|c| "mndjYy".contains(*c)) {
            let value: i64 = numbers.next()?.parse().ok()?;
            match token {
                'm' | 'n' => month = Some(value as u32),
                'd' | 'j' => day = Some(value as u32),
                'Y' => year = Some(value as i32),
                'y' => year = Some(2000 + value as i32),
                _ => {}
            }
        }
        let (year, month, day) = (year?, month?, day?);
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }
}

/// Whether `year` is a leap year.
pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in `month` of `year`.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The 6×7 grid flatpickr shows for the month of `view`: it starts on the
/// Sunday on or before the 1st and always has 42 days.
pub fn month_grid(view: CalendarDate) -> Vec<CalendarDate> {
    let first = view.first_of_month();
    let start = first.add_days(-(first.weekday() as i64));
    (0..42).map(|i| start.add_days(i)).collect()
}

/// Month and weekday names.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarLocale {
    /// January … December.
    pub months: [SharedString; 12],
    /// Sunday-first weekday abbreviations.
    pub weekdays: [SharedString; 7],
}

impl CalendarLocale {
    /// flatpickr's English locale with Carbon's two-letter weekdays.
    pub fn english() -> Self {
        Self {
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .map(SharedString::new_static),
            weekdays: ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"].map(SharedString::new_static),
        }
    }
}

impl Default for CalendarLocale {
    fn default() -> Self {
        Self::english()
    }
}

/// `index.d.ts` `CalendarProps.mode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CalendarMode {
    /// One date.
    #[default]
    Single,
    /// A start and an end.
    Range,
}

impl CalendarMode {
    /// Both modes.
    pub const ALL: [Self; 2] = [Self::Single, Self::Range];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Range => "range",
        }
    }
}

/// The selection: one date, or a range being built.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    /// The single date, or the range start.
    pub start: Option<CalendarDate>,
    /// The range end.
    pub end: Option<CalendarDate>,
}

impl Selection {
    /// One date.
    pub fn single(date: CalendarDate) -> Self {
        Self {
            start: Some(date),
            end: None,
        }
    }

    /// A range.
    pub fn range(start: CalendarDate, end: CalendarDate) -> Self {
        Self {
            start: Some(start.min(end)),
            end: Some(start.max(end)),
        }
    }

    /// The selection after picking `date`: single replaces; range starts over
    /// after a complete range, else completes it (ordering the ends).
    pub fn pick(self, mode: CalendarMode, date: CalendarDate) -> Self {
        match (mode, self.start, self.end) {
            (CalendarMode::Single, _, _) => Self::single(date),
            (CalendarMode::Range, Some(start), None) => Self::range(start, date),
            (CalendarMode::Range, _, _) => Self::single(date),
        }
    }

    /// Whether the selection is finished for `mode`.
    pub fn complete(self, mode: CalendarMode) -> bool {
        match mode {
            CalendarMode::Single => self.start.is_some(),
            CalendarMode::Range => self.start.is_some() && self.end.is_some(),
        }
    }
}

// ───────────────────────── Looks ─────────────────────────

/// The state a day look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DayState {
    /// In the shown month.
    pub in_month: bool,
    /// Today.
    pub today: bool,
    /// Selected (single date, range start or end).
    pub selected: bool,
    /// Strictly inside a selected range.
    pub in_range: bool,
    /// Pointer over it.
    pub hovered: bool,
    /// The hovered end of a range being picked (flatpickr `endRange:hover`).
    pub range_end: bool,
    /// Keyboard focus.
    pub focused: bool,
    /// Outside `minDate`/`maxDate`.
    pub disabled: bool,
}

/// While a range has only its start, hovering a later day previews the range
/// (flatpickr `onMouseOver`): the days strictly between start and the hovered
/// day are `inRange`; the hovered day is `endRange` when after the start.
/// Returns (in range, range end) for `date`.
pub fn range_preview(
    mode: CalendarMode,
    selection: Selection,
    hovered: Option<CalendarDate>,
    date: CalendarDate,
) -> (bool, bool) {
    match (mode, selection.start, selection.end, hovered) {
        (CalendarMode::Range, Some(start), None, Some(hover)) => {
            let (lo, hi) = (start.min(hover), start.max(hover));
            (date > lo && date < hi, date == hover && hover > start)
        }
        _ => (false, false),
    }
}

/// The paint of one day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayLook {
    /// Cell fill.
    pub background: Hsla,
    /// Number color.
    pub text: Hsla,
    /// Semibold (today).
    pub bold: bool,
    /// The today dot.
    pub dot: Option<Hsla>,
    /// 2px inset outline.
    pub outline: Option<Hsla>,
}

impl DayLook {
    /// The look of a day (`.flatpickr-day*`).
    pub fn of(state: DayState) -> Self {
        let clear = Hsla::transparent_black();
        if state.disabled {
            // `.flatpickr-day.flatpickr-disabled { color: var(--text-disabled) }`.
            return Self {
                background: clear,
                text: color(ColorRole::TextDisabled),
                bold: false,
                dot: None,
                outline: None,
            };
        }
        // `:focus { outline: 2px solid var(--button-primary); outline-offset: -2px }`;
        // `.today.selected { outline: 2px solid var(--focus) }`.
        let outline = if state.focused {
            Some(color(ColorRole::ButtonPrimary))
        } else if state.today && state.selected {
            Some(color(ColorRole::Focus))
        } else {
            None
        };
        if state.selected {
            return Self {
                // `.flatpickr-day.selected { background-color: var(--button-primary);
                // color: var(--text-on-color) }`.
                background: color(ColorRole::ButtonPrimary),
                text: color(ColorRole::TextOnColor),
                bold: state.today,
                dot: None,
                // `.selected:focus { outline: 1px solid var(--layer-02); outline-offset: -3px }`.
                outline: if state.focused {
                    Some(color(ColorRole::Layer02))
                } else {
                    outline
                },
            };
        }
        if state.range_end {
            // `.flatpickr-day.endRange:hover { outline: 2px solid var(--focus);
            // background: var(--layer-01); color: var(--text-primary) }`.
            return Self {
                background: color(ColorRole::Layer01),
                text: color(ColorRole::TextPrimary),
                bold: state.today,
                dot: state.today.then(|| color(ColorRole::LinkPrimary)),
                outline: Some(color(ColorRole::Focus)),
            };
        }
        let background = if state.in_range {
            // `.flatpickr-day.inRange { background-color: var(--highlight) }`.
            color(ColorRole::Highlight)
        } else if state.hovered {
            color(ColorRole::LayerHover01)
        } else {
            clear
        };
        let text = if state.today {
            // `.flatpickr-day.today { color: var(--link-primary); font-weight: 600 }`.
            color(ColorRole::LinkPrimary)
        } else if !state.in_month {
            // `.nextMonthDay, .prevMonthDay { color: var(--text-helper) }`.
            color(ColorRole::TextHelper)
        } else {
            color(ColorRole::TextPrimary)
        };
        Self {
            background,
            text,
            bold: state.today,
            dot: state.today.then(|| color(ColorRole::LinkPrimary)),
            outline,
        }
    }
}

// ───────────────────────── Component ─────────────────────────

/// Per-calendar state kept across frames.
pub(crate) struct CalendarView {
    /// Any day of the shown month.
    pub view: CalendarDate,
    /// The keyboard-focused day.
    pub cursor: Option<CalendarDate>,
    /// The day under the pointer.
    pub hovered: Option<CalendarDate>,
    /// The grid's focus handle.
    pub focus: FocusHandle,
}

type PickHandler = Rc<dyn Fn(Selection, &mut Window, &mut App)>;

/// The grid panel, shared by [`Calendar`] and [`super::DatePicker`].
pub(crate) struct Panel {
    pub id: ElementId,
    pub mode: CalendarMode,
    pub selection: Selection,
    pub min: Option<CalendarDate>,
    pub max: Option<CalendarDate>,
    pub locale: CalendarLocale,
    pub inline: bool,
    pub prev_label: Option<SharedString>,
    pub next_label: Option<SharedString>,
    pub on_pick: PickHandler,
}

fn out_of_bounds(date: CalendarDate, min: Option<CalendarDate>, max: Option<CalendarDate>) -> bool {
    min.is_some_and(|m| date < m) || max.is_some_and(|m| date > m)
}

/// The keyed view state of the panel `id`, showing `initial`'s month first.
pub(crate) fn view_state(
    id: &ElementId,
    initial: CalendarDate,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::Entity<CalendarView> {
    window.use_keyed_state((id.clone(), "calendar"), cx, |_, cx| CalendarView {
        view: initial,
        cursor: None,
        hovered: None,
        focus: cx.focus_handle(),
    })
}

/// Render the flatpickr panel.
pub(crate) fn panel(
    panel: Panel,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let Panel {
        id,
        mode,
        selection,
        min,
        max,
        locale,
        inline,
        prev_label,
        next_label,
        on_pick,
    } = panel;
    let today = CalendarDate::today();
    let state = view_state(&id, selection.start.unwrap_or(today), window, cx);
    let (view, cursor, hovered, focus) = {
        let s = state.read(cx);
        (s.view, s.cursor, s.hovered, s.focus.clone())
    };
    let focused = focus.is_focused(window);
    let grid = month_grid(view);

    let nav = |delta: i32, label: Option<SharedString>, icon: CarbonIcon| {
        let state = state.clone();
        let disabled = match delta {
            d if d < 0 => min.is_some_and(|m| view.first_of_month().add_days(-1) < m),
            _ => max.is_some_and(|m| view.add_months(1).first_of_month() > m),
        };
        // flatpickr's arrows are plain spans with no tooltip: the label is only
        // the accessible name. They are not tab stops (the grid is).
        gpui_kit::base::Button::new((id.clone(), if delta < 0 { "prev" } else { "next" }))
            .tab_stop(false)
            .disabled(disabled)
            .when_some(label, |this, label| this.accessibility_label(label))
            .size(rem(DAY))
            .flex()
            .items_center()
            .justify_center()
            .when(!disabled, |this| {
                this.cursor(CursorStyle::PointingHand)
                    // `.flatpickr-prev-month:hover { background-color: var(--layer-hover) }`.
                    .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                    .on_click(move |_, _, cx| {
                        state.update(cx, |s, cx| {
                            s.view = s.view.add_months(delta);
                            cx.notify();
                        })
                    })
            })
            .child(
                Icon::carbon(icon)
                    .size(IconSize::S16)
                    .color(color(if disabled {
                        ColorRole::IconDisabled
                    } else {
                        ColorRole::IconPrimary
                    })),
            )
    };

    let month_row = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(rem(DAY))
        .child(nav(-1, prev_label, CarbonIcon::ChevronLeft))
        .child(
            div()
                .type_style(TypeStyle::HeadingCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child(SharedString::from(format!(
                    "{} {}",
                    locale.months[(view.month - 1) as usize],
                    view.year
                ))),
        )
        .child(nav(1, next_label, CarbonIcon::ChevronRight));

    let weekdays = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(rem(DAY))
        .children(locale.weekdays.iter().map(|w| {
            div()
                .flex_1()
                .flex()
                .justify_center()
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child(w.clone())
        }));

    let days = grid.iter().map(|&date| {
        let selected = selection.start == Some(date) || selection.end == Some(date);
        let disabled = out_of_bounds(date, min, max);
        let hover_day = hovered.filter(|&h| !out_of_bounds(h, min, max));
        let (preview, range_end) = range_preview(mode, selection, hover_day, date);
        let in_range = preview
            || matches!((selection.start, selection.end), (Some(a), Some(b)) if date > a && date < b);
        let look = DayLook::of(DayState {
            in_month: date.month == view.month,
            today: date == today,
            selected,
            in_range,
            hovered: hovered == Some(date) && !disabled,
            range_end: range_end && !selected,
            focused: focused && cursor == Some(date),
            disabled,
        });
        let key = (id.clone(), format!("day-{}", date.to_days()));
        // flatpickr rebuilds the day cells when the month changes, so the 70ms
        // transition only runs within one month (hover, selection): key the
        // animation by the shown month too, so a new month starts at rest.
        let anim = (
            id.clone(),
            format!("day-{}-{}-{}", view.year, view.month, date.to_days()),
        );
        let background =
            motion::animate_color((anim.clone(), "bg"), look.background, DAY_MOTION, window, cx);
        let text = motion::animate_color((anim, "text"), look.text, DAY_MOTION, window, cx);
        let on_pick = on_pick.clone();
        let hover_state = state.clone();
        let click_state = state.clone();
        let focus = focus.clone();
        let outline_inset = if selected && look.outline.is_some() && focused {
            3.
        } else {
            0.
        };
        div()
            .id(key)
            .relative()
            .size(rem(DAY))
            .flex()
            .items_center()
            .justify_center()
            .bg(background)
            .type_style(TypeStyle::BodyCompact01)
            .when(look.bold, |this| {
                this.font_weight(gpui_kit::FontWeight::SEMIBOLD)
            })
            .text_color(text)
            .cursor(if disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover(move |over, _, cx| {
                hover_state.update(cx, |s, cx| {
                    let next = if *over {
                        Some(date)
                    } else if s.hovered == Some(date) {
                        None
                    } else {
                        s.hovered
                    };
                    if next != s.hovered {
                        s.hovered = next;
                        cx.notify();
                    }
                })
            })
            .when(!disabled, |this| {
                this.on_click(move |_, window, cx| {
                    click_state.update(cx, |s, cx| {
                        s.cursor = Some(date);
                        if date.month != s.view.month {
                            s.view = date;
                        }
                        cx.notify();
                    });
                    focus.focus(window, cx);
                    on_pick(selection.pick(mode, date), window, cx);
                })
            })
            .child(SharedString::from(date.day.to_string()))
            // `.flatpickr-day.today::after`: 4×4px, 7px from the bottom, centered.
            .when_some(look.dot, |this, dot| {
                this.child(
                    div()
                        .absolute()
                        .bottom(rem(7.))
                        .left(rem(DAY / 2. - 2.))
                        .size(rem(4.))
                        .bg(dot),
                )
            })
            .when_some(look.outline, |this, outline| {
                let width = if outline_inset > 0. { 1. } else { 2. };
                this.child(
                    div()
                        .absolute()
                        .top(px(outline_inset))
                        .left(px(outline_inset))
                        .right(px(outline_inset))
                        .bottom(px(outline_inset))
                        .border(px(width))
                        .border_color(outline),
                )
            })
    });

    let on_key = {
        let state = state.clone();
        let on_pick = on_pick.clone();
        move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
            let cursor = state
                .read(cx)
                .cursor
                .unwrap_or(selection.start.unwrap_or(view));
            let next = match event.keystroke.key.as_str() {
                "left" => cursor.add_days(-1),
                "right" => cursor.add_days(1),
                "up" => cursor.add_days(-7),
                "down" => cursor.add_days(7),
                "pageup" => cursor.add_months(-1),
                "pagedown" => cursor.add_months(1),
                "home" => cursor.add_days(-(cursor.weekday() as i64)),
                "end" => cursor.add_days(6 - cursor.weekday() as i64),
                "enter" | "space" => {
                    if !out_of_bounds(cursor, min, max) {
                        on_pick(selection.pick(mode, cursor), window, cx);
                    }
                    cx.stop_propagation();
                    return;
                }
                _ => return,
            };
            cx.stop_propagation();
            state.update(cx, |s, cx| {
                s.cursor = Some(next);
                if (next.year, next.month) != (s.view.year, s.view.month) {
                    s.view = next;
                }
                cx.notify();
            });
        }
    };

    div()
        .id((id.clone(), "panel"))
        .track_focus(&focus.clone().tab_stop(true))
        .on_key_down(on_key)
        .on_hover({
            let state = state.clone();
            move |over, _, cx| {
                if !over {
                    state.update(cx, |s, cx| {
                        if s.hovered.take().is_some() {
                            cx.notify();
                        }
                    })
                }
            }
        })
        // `.flatpickr-calendar.open { padding: 0.25rem 0.25rem 0.5rem; inline-size: 18rem;
        // background-color: var(--layer-01) }`.
        .w(rem(PANEL_WIDTH))
        .pt(rem(4.))
        .px(rem(4.))
        .pb(rem(8.))
        .bg(color(LayerLevel::One.layer()))
        .flex()
        .flex_col()
        .items_center()
        // `.cu-calendar .flatpickr-calendar.inline { outline: 1px solid var(--border-subtle-01) }`.
        .when(inline, |this| {
            this.border_1()
                .border_color(color(ColorRole::BorderSubtle01))
        })
        .child(month_row)
        .child(weekdays)
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .w(rem(DAY * 7.))
                .children(days),
        )
}

type ChangeHandler = Rc<dyn Fn(Selection, &mut Window, &mut App)>;

/// An inline Carbon calendar. See the module docs.
#[derive(IntoElement)]
pub struct Calendar {
    id: ElementId,
    mode: CalendarMode,
    value: Option<Selection>,
    default_value: Selection,
    min: Option<CalendarDate>,
    max: Option<CalendarDate>,
    locale: CalendarLocale,
    prev_label: Option<SharedString>,
    next_label: Option<SharedString>,
    on_value_change: Option<ChangeHandler>,
}

impl Calendar {
    /// A single-date calendar with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            mode: CalendarMode::Single,
            value: None,
            default_value: Selection::default(),
            min: None,
            max: None,
            locale: CalendarLocale::english(),
            prev_label: None,
            next_label: None,
            on_value_change: None,
        }
    }

    /// `mode`.
    pub fn mode(mut self, mode: CalendarMode) -> Self {
        self.mode = mode;
        self
    }

    /// Controlled `value`.
    pub fn value(mut self, value: Selection) -> Self {
        self.value = Some(value);
        self
    }

    /// The initial value of an uncontrolled calendar.
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

    /// Month and weekday names.
    pub fn locale(mut self, locale: CalendarLocale) -> Self {
        self.locale = locale;
        self
    }

    /// Accessible names of the previous/next month buttons.
    pub fn nav_labels(
        mut self,
        prev: impl Into<SharedString>,
        next: impl Into<SharedString>,
    ) -> Self {
        self.prev_label = Some(prev.into());
        self.next_label = Some(next.into());
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

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (selection, store) = super::form::controlled(
            (self.id.clone(), "value"),
            self.value,
            self.default_value,
            window,
            cx,
        );
        let on_change = self.on_value_change.clone();
        panel(
            Panel {
                id: self.id.clone(),
                mode: self.mode,
                selection,
                min: self.min,
                max: self.max,
                locale: self.locale,
                inline: true,
                prev_label: self.prev_label,
                next_label: self.next_label,
                on_pick: Rc::new(move |next, window, cx| {
                    store.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                    if let Some(handler) = on_change.as_ref() {
                        handler(next, window, cx);
                    }
                }),
            },
            window,
            cx,
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
    fn date_math_round_trips() {
        let d = CalendarDate::new(2026, 11, 26);
        assert_eq!(CalendarDate::from_days(d.to_days()), d);
        assert_eq!(CalendarDate::from_days(0), CalendarDate::new(1970, 1, 1));
        // 2026-11-26 is a Thursday.
        assert_eq!(d.weekday(), 4);
        assert_eq!(d.add_months(3), CalendarDate::new(2027, 2, 26));
        assert_eq!(
            CalendarDate::new(2024, 1, 31).add_months(1),
            CalendarDate::new(2024, 2, 29)
        );
        assert_eq!(days_in_month(2100, 2), 28);
    }

    #[test]
    fn grid_is_six_weeks_from_sunday() {
        let grid = month_grid(CalendarDate::new(2026, 11, 26));
        assert_eq!(grid.len(), 42);
        // November 2026 starts on a Sunday.
        assert_eq!(grid[0], CalendarDate::new(2026, 11, 1));
        assert_eq!(grid[0].weekday(), 0);
    }

    #[test]
    fn format_and_parse_follow_flatpickr_tokens() {
        let d = CalendarDate::new(2026, 3, 7);
        assert_eq!(d.format("m/d/Y"), "03/07/2026");
        assert_eq!(d.format("Y-m-d"), "2026-03-07");
        assert_eq!(
            CalendarDate::parse("11/26/2026", "m/d/Y"),
            Some(CalendarDate::new(2026, 11, 26))
        );
        assert_eq!(CalendarDate::parse("02/30/2026", "m/d/Y"), None);
        assert_eq!(CalendarDate::parse("nope", "m/d/Y"), None);
    }

    #[test]
    fn range_picking_orders_and_restarts() {
        let a = CalendarDate::new(2026, 11, 20);
        let b = CalendarDate::new(2026, 11, 10);
        let s = Selection::default().pick(CalendarMode::Range, a);
        assert!(!s.complete(CalendarMode::Range));
        let s = s.pick(CalendarMode::Range, b);
        assert_eq!((s.start, s.end), (Some(b), Some(a)));
        assert_eq!(s.pick(CalendarMode::Range, a).end, None);
    }

    #[test]
    fn day_colors_follow_flatpickr_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let today = DayLook::of(DayState {
            in_month: true,
            today: true,
            ..Default::default()
        });
        assert_eq!(hex(today.text), "#0f62fe");
        assert!(today.bold && today.dot.is_some());
        let selected = DayLook::of(DayState {
            in_month: true,
            selected: true,
            ..Default::default()
        });
        assert_eq!(
            (hex(selected.background), hex(selected.text)),
            ("#0f62fe".into(), "#ffffff".into())
        );
        let range = DayLook::of(DayState {
            in_month: true,
            in_range: true,
            ..Default::default()
        });
        assert_eq!(hex(range.background), "#d0e2ff");
        let other = DayLook::of(DayState::default());
        assert_eq!(hex(other.text), "#6f6f6f");
        // `.flatpickr-day.endRange:hover`: layer-01 with a 2px focus outline.
        let end = DayLook::of(DayState {
            in_month: true,
            hovered: true,
            range_end: true,
            ..Default::default()
        });
        assert_eq!(hex(end.background), "#f4f4f4");
        assert_eq!(end.outline.map(hex).as_deref(), Some("#0f62fe"));
    }

    #[test]
    fn hovering_after_a_range_start_previews_the_range() {
        let start = CalendarDate::new(2026, 11, 10);
        let hover = CalendarDate::new(2026, 11, 13);
        let sel = Selection::single(start);
        let at = |d: u32| {
            range_preview(
                CalendarMode::Range,
                sel,
                Some(hover),
                CalendarDate::new(2026, 11, d),
            )
        };
        assert_eq!(at(10), (false, false));
        assert_eq!(at(11), (true, false));
        assert_eq!(at(12), (true, false));
        assert_eq!(at(13), (false, true));
        assert_eq!(at(14), (false, false));
        // Hovering before the start: between days fill, no end marker.
        let before = CalendarDate::new(2026, 11, 7);
        assert_eq!(
            range_preview(
                CalendarMode::Range,
                sel,
                Some(before),
                CalendarDate::new(2026, 11, 8)
            ),
            (true, false)
        );
        assert_eq!(
            range_preview(CalendarMode::Range, sel, Some(before), before),
            (false, false)
        );
        // Single mode and complete ranges show no preview.
        assert_eq!(
            range_preview(
                CalendarMode::Single,
                sel,
                Some(hover),
                CalendarDate::new(2026, 11, 11)
            ),
            (false, false)
        );
        let done = Selection::range(start, hover);
        assert_eq!(
            range_preview(
                CalendarMode::Range,
                done,
                Some(CalendarDate::new(2026, 11, 20)),
                CalendarDate::new(2026, 11, 15)
            ),
            (false, false)
        );
    }
}
