//! ScheduleEditor: when a scheduled task runs (`ScheduleEditor` in
//! `src/ghost/admin.jsx`, `.g-sched` in `src/ghost.css`): a segmented
//! "Daily at / Every N / On webhook" switch, that kind's fields and a
//! plain-language summary underneath ("Every weekday at 09:00"), so nobody
//! reads cron.
//!
//! - **Daily**: a [`TimeField`] and seven day toggles (`.g-day`, pressed =
//!   `layer-selected-inverse`).
//! - **Interval**: a [`Stepper`] (1–999, default 30) and a unit select.
//! - **Webhook**: a [`CopyField`] with the hook URL and a helper line.
//!
//! The value is one [`Schedule`] holding every kind's fields, like the
//! reference's `{...v, ...p}` merge: switching kind keeps what was typed for
//! the others. `value` makes it controlled; otherwise it keeps its own state
//! seeded from `default_value`. The kind switch is a radio group: only the
//! checked option is a tab stop and ←/→/↑/↓/Home/End move the choice
//! (`radioKeys`).
//!
//! ```ignore
//! ScheduleEditor::new("sched")
//!     .default_value(Schedule::default())
//!     .on_value_change(|s, _, _| println!("{}", s.kind.name()))
//! ```
use super::super::primitives::{CopyField, Stepper, TimeField, parts};
use crate::{
    components::{NativeSelect, form::FieldSize},
    density,
    icons::{CarbonIcon, Icon},
    t3::{
        base::radio_step,
        craft::{self, rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, ElementId, Entity, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// The schedule kind (`v.kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScheduleKind {
    /// `daily`: at a time on chosen weekdays.
    #[default]
    Daily,
    /// `interval`: every N minutes, hours or days.
    Interval,
    /// `webhook`: when a webhook is received.
    Webhook,
}

impl ScheduleKind {
    /// Every kind, in the switch's order.
    pub const ALL: [Self; 3] = [Self::Daily, Self::Interval, Self::Webhook];

    /// The reference's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Interval => "interval",
            Self::Webhook => "webhook",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Daily => 0,
            Self::Interval => 1,
            Self::Webhook => 2,
        }
    }
}

/// The interval unit (`v.unit`, default `minutes`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IntervalUnit {
    /// `minutes`.
    #[default]
    Minutes,
    /// `hours`.
    Hours,
    /// `days`.
    Days,
}

impl IntervalUnit {
    /// Every unit, in the select's order.
    pub const ALL: [Self; 3] = [Self::Minutes, Self::Hours, Self::Days];

    /// The reference's value.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Minutes => "minutes",
            Self::Hours => "hours",
            Self::Days => "days",
        }
    }

    /// The unit for a reference value.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|u| u.name() == name)
    }

    const fn index(self) -> usize {
        match self {
            Self::Minutes => 0,
            Self::Hours => 1,
            Self::Days => 2,
        }
    }
}

/// A schedule: the kind and every kind's fields (see the module docs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schedule {
    /// Which fields apply.
    pub kind: ScheduleKind,
    /// Daily: `HH:mm`.
    pub at: SharedString,
    /// Daily: the weekdays, 0 = Sunday … 6 = Saturday, ascending.
    pub days: Vec<u8>,
    /// Interval: the count (`every || 30`).
    pub every: Option<u32>,
    /// Interval: the unit.
    pub unit: IntervalUnit,
    /// Webhook: the hook URL (the texts' `default_url` when unset).
    pub url: Option<SharedString>,
    /// Webhook: the event named in the summary (`event || 'a webhook'`).
    pub event: Option<SharedString>,
}

impl Default for Schedule {
    /// `{ kind: 'daily', at: '09:00', days: [1, 2, 3, 4, 5] }`.
    fn default() -> Self {
        Self {
            kind: ScheduleKind::Daily,
            at: "09:00".into(),
            days: vec![1, 2, 3, 4, 5],
            every: None,
            unit: IntervalUnit::Minutes,
            url: None,
            event: None,
        }
    }
}

impl Schedule {
    /// The default schedule switched to `kind` (the preview's
    /// `defaultValue: { kind: 'webhook' }`).
    pub fn of_kind(kind: ScheduleKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    /// The interval count shown (`v.every || 30`).
    pub fn every_or_default(&self) -> u32 {
        self.every.filter(|n| *n > 0).unwrap_or(30)
    }

    /// Day `d` pressed or released: removed if present, else added and
    /// sorted (`[...v.days, i].sort()`).
    pub fn toggle_day(&mut self, d: u8) {
        if let Some(i) = self.days.iter().position(|x| *x == d) {
            self.days.remove(i);
        } else {
            self.days.push(d);
            self.days.sort_unstable();
        }
    }

    /// The summary line in `texts`' words (see [`ScheduleTexts`]).
    pub fn summary(&self, texts: &ScheduleTexts) -> SharedString {
        match self.kind {
            ScheduleKind::Daily => {
                let days: SharedString = if self.days.len() == 7 {
                    texts.every_day.clone()
                } else if self.days.len() == 5 && !self.days.contains(&0) && !self.days.contains(&6)
                {
                    texts.every_weekday.clone()
                } else {
                    self.days
                        .iter()
                        .filter_map(|d| texts.day_short.get(*d as usize))
                        .map(|s| s.as_ref())
                        .collect::<Vec<&str>>()
                        .join(", ")
                        .into()
                };
                (texts.summary_daily)(&days, &self.at)
            }
            ScheduleKind::Interval => {
                (texts.summary_interval)(self.every_or_default(), &texts.units[self.unit.index()])
            }
            ScheduleKind::Webhook => (texts.summary_webhook)(
                self.event
                    .as_deref()
                    .filter(|e| !e.is_empty())
                    .unwrap_or(&texts.default_event),
            ),
        }
    }
}

/// `Every {days} at {at}` from the days phrase and the time.
pub type DailySummary = Rc<dyn Fn(&str, &str) -> SharedString>;
/// `Every {n} {unit}` from the count and the unit's word.
pub type IntervalSummary = Rc<dyn Fn(u32, &str) -> SharedString>;

/// The editor's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ScheduleTexts {
    /// The switch: `"Daily at"`, `"Every N"`, `"On webhook"`.
    pub kinds: [SharedString; 3],
    /// The switch's accessible name: `"Schedule type"`.
    pub kind_group: SharedString,
    /// `"Time"`.
    pub time: SharedString,
    /// `"Days"`.
    pub days: SharedString,
    /// The day toggles: `S M T W T F S`.
    pub day_letters: [SharedString; 7],
    /// Their accessible names: `Sunday` … `Saturday`.
    pub day_names: [SharedString; 7],
    /// In the summary: `Sun` … `Sat`.
    pub day_short: [SharedString; 7],
    /// The summary for all seven days: `"day"`.
    pub every_day: SharedString,
    /// The summary for Monday–Friday: `"weekday"`.
    pub every_weekday: SharedString,
    /// The interval count's label: `"Every"`.
    pub every: SharedString,
    /// The unit select's label: `"Unit"`.
    pub unit: SharedString,
    /// The units: `minutes`, `hours`, `days`.
    pub units: [SharedString; 3],
    /// `"Webhook URL"`.
    pub webhook_url: SharedString,
    /// The URL shown before one is set.
    pub default_url: SharedString,
    /// Under the URL: what a POST does.
    pub webhook_helper: SharedString,
    /// The webhook summary's event before one is set: `"a webhook"`.
    pub default_event: SharedString,
    /// `"Every {days} at {at}"`.
    pub summary_daily: DailySummary,
    /// `"Every {n} {unit}"`.
    pub summary_interval: IntervalSummary,
    /// `"When {event} is received"`.
    pub summary_webhook: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for ScheduleTexts {
    fn default() -> Self {
        let s = |v: [&str; 7]| v.map(SharedString::from);
        Self {
            kinds: ["Daily at".into(), "Every N".into(), "On webhook".into()],
            kind_group: "Schedule type".into(),
            time: "Time".into(),
            days: "Days".into(),
            day_letters: s(["S", "M", "T", "W", "T", "F", "S"]),
            day_names: s([
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ]),
            day_short: s(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]),
            every_day: "day".into(),
            every_weekday: "weekday".into(),
            every: "Every".into(),
            unit: "Unit".into(),
            units: ["minutes".into(), "hours".into(), "days".into()],
            webhook_url: "Webhook URL".into(),
            default_url: "https://u0.local/hooks/7f3a9c2e".into(),
            webhook_helper:
                "POST to this URL to start the task. The JSON body is passed to the agent as context."
                    .into(),
            default_event: "a webhook".into(),
            summary_daily: Rc::new(|days, at| format!("Every {days} at {at}").into()),
            summary_interval: Rc::new(|n, unit| format!("Every {n} {unit}").into()),
            summary_webhook: Rc::new(|event| format!("When {event} is received").into()),
        }
    }
}

/// A segment's (fill, ink): `.g-seg-btn { background: transparent; color:
/// var(--text-secondary) }`, `:hover { background: var(--layer-hover-01);
/// color: var(--text-primary) }`, `--on { background:
/// var(--layer-selected-inverse); color: var(--text-inverse) }`.
pub fn segment_look(on: bool, hovered: bool) -> (Hsla, Hsla) {
    if on {
        (
            color(ColorRole::LayerSelectedInverse),
            color(ColorRole::TextInverse),
        )
    } else if hovered {
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::TextPrimary),
        )
    } else {
        (Hsla::transparent_black(), color(ColorRole::TextSecondary))
    }
}

/// A day toggle's (fill, ink, bottom rule): `.g-day { background:
/// var(--field-01); border-bottom: 1px solid var(--border-strong-01); color:
/// var(--text-secondary) }`, `:hover { background: var(--field-hover-01) }`,
/// `--on { background: var(--layer-selected-inverse); color:
/// var(--text-inverse); border-bottom-color: transparent }`.
pub fn day_look(on: bool, hovered: bool) -> (Hsla, Hsla, Hsla) {
    if on {
        (
            color(ColorRole::LayerSelectedInverse),
            color(ColorRole::TextInverse),
            Hsla::transparent_black(),
        )
    } else {
        (
            color(if hovered {
                ColorRole::FieldHover01
            } else {
                ColorRole::Field01
            }),
            color(ColorRole::TextSecondary),
            color(ColorRole::BorderStrong01),
        )
    }
}

type ChangeHandler = Rc<dyn Fn(&Schedule, &mut Window, &mut App)>;
type Set = Rc<dyn Fn(Schedule, &mut Window, &mut App)>;

/// A schedule editor. See the module docs.
#[derive(IntoElement)]
pub struct ScheduleEditor {
    id: ElementId,
    value: Option<Schedule>,
    default_value: Schedule,
    texts: ScheduleTexts,
    on_value_change: Option<ChangeHandler>,
}

impl ScheduleEditor {
    /// An editor with a stable `id`, seeded with [`Schedule::default`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: Schedule::default(),
            texts: ScheduleTexts::default(),
            on_value_change: None,
        }
    }

    /// The controlled schedule (`value`).
    pub fn value(mut self, value: Schedule) -> Self {
        self.value = Some(value);
        self
    }

    /// The first schedule while uncontrolled (`defaultValue`).
    pub fn default_value(mut self, value: Schedule) -> Self {
        self.default_value = value;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ScheduleTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Called with each new schedule (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(&Schedule, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

/// A keyed hover flag.
fn hover_state(id: ElementId, window: &mut Window, cx: &mut App) -> Entity<bool> {
    window.use_keyed_state(id, cx, |_, _| false)
}

fn set_hover(state: &Entity<bool>, h: bool, cx: &mut App) {
    state.update(cx, |v, cx| {
        if *v != h {
            *v = h;
            cx.notify();
        }
    })
}

impl ScheduleEditor {
    /// `.g-seg[role=radiogroup]`: the kind switch.
    fn kind_switch(
        &self,
        v: &Schedule,
        set: &Set,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        let id = &self.id;
        let handles: Vec<_> = ScheduleKind::ALL
            .iter()
            .enumerate()
            .map(|(i, _)| craft::focus_handle(&craft::sub(id, "kind", i), window, cx))
            .collect();
        let mut buttons = Vec::new();
        for (i, kind) in ScheduleKind::ALL.into_iter().enumerate() {
            let on = v.kind == kind;
            let hovered = hover_state(craft::sub(id, "kind-hover", i), window, cx);
            let (fill, ink) = segment_look(on, *hovered.read(cx));
            let ring = craft::focus_visible(&handles[i], window);
            let set = set.clone();
            let next = Schedule { kind, ..v.clone() };
            buttons.push(
                // `.g-seg-btn { height: var(--density-field-sm); padding: 0
                // var(--spacing-04); border: 1px solid var(--border-inverse);
                // border-left-width: 0; font-size: 0.75rem }`,
                // `:first-child { border-left-width: 1px }`.
                sans(
                    craft::press(craft::sub(id, "kind-btn", i), &handles[i]),
                    12.,
                    400,
                )
                .role(gpui_kit::Role::RadioButton)
                .accessibility_label(self.texts.kinds[i].clone())
                // Only the checked radio is a tab stop (`tabIndex={on ? 0 : -1}`).
                .tab_stop(on)
                .flex()
                .items_center()
                .h(density::current().field_sm())
                .px(tokens::SPACING_04)
                .border_t_1()
                .border_b_1()
                .border_r_1()
                .when(i == 0, |b| b.border_l_1())
                .border_color(color(ColorRole::BorderInverse))
                .bg(fill)
                .text_color(ink)
                .whitespace_nowrap()
                .on_hover(move |h, _, cx| set_hover(&hovered, *h, cx))
                .on_click(move |_, window, cx| set(next.clone(), window, cx))
                .child(self.texts.kinds[i].clone())
                // `.g-seg-btn:focus-visible { outline: 2px solid var(--focus); outline-offset: -2px }`.
                .when(ring, |b| b.child(craft::focus_ring())),
            );
        }
        let keys = {
            let set = set.clone();
            let v = v.clone();
            move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                if let Some(next) = radio_step(e.keystroke.key.as_str(), 3, Some(v.kind.index())) {
                    cx.stop_propagation();
                    handles[next].focus(window, cx);
                    set(
                        Schedule {
                            kind: ScheduleKind::ALL[next],
                            ..v.clone()
                        },
                        window,
                        cx,
                    );
                }
            }
        };
        // `.g-seg { display: inline-flex }`.
        div()
            .id((id.clone(), "kinds"))
            .flex()
            .flex_none()
            .on_key_down(keys)
            .children(buttons)
    }

    /// `.g-days[role=group]`: the seven day toggles.
    fn days(&self, v: &Schedule, set: &Set, window: &mut Window, cx: &mut App) -> gpui_kit::Div {
        let id = &self.id;
        let size = density::current().field();
        // `.g-days { display: flex; gap: 2px }`.
        let mut row = div().flex().gap(px(2.));
        for d in 0..7u8 {
            let i = d as usize;
            let on = v.days.contains(&d);
            let focus = craft::focus_handle(&craft::sub(id, "day", i), window, cx);
            let hovered = hover_state(craft::sub(id, "day-hover", i), window, cx);
            let (fill, ink, rule) = day_look(on, *hovered.read(cx));
            let ring = craft::focus_visible(&focus, window);
            let set = set.clone();
            let mut next = v.clone();
            next.toggle_day(d);
            row = row.child(
                // `.g-day { width: var(--density-field-md); height:
                // var(--density-field-md); font-size: 0.75rem }`.
                sans(craft::press(craft::sub(id, "day-btn", i), &focus), 12., 400)
                    // `aria-pressed`: Base's button has no pressed state to
                    // expose; the look carries it.
                    .accessibility_label(self.texts.day_names[i].clone())
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(size)
                    .bg(fill)
                    .text_color(ink)
                    .border_b_1()
                    .border_color(rule)
                    .on_hover(move |h, _, cx| set_hover(&hovered, *h, cx))
                    .on_click(move |_, window, cx| set(next.clone(), window, cx))
                    .child(self.texts.day_letters[i].clone())
                    .when(ring, |b| b.child(craft::focus_ring())),
            );
        }
        row
    }
}

impl RenderOnce for ScheduleEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let own = window.use_keyed_state((id.clone(), "value"), cx, {
            let d = self.default_value.clone();
            move |_, _| d
        });
        let v = self.value.clone().unwrap_or_else(|| own.read(cx).clone());
        let set: Set = {
            let own = own.clone();
            let controlled = self.value.is_some();
            let on_change = self.on_value_change.clone();
            Rc::new(move |next: Schedule, window, cx| {
                if !controlled {
                    own.update(cx, |s, cx| {
                        *s = next.clone();
                        cx.notify();
                    });
                }
                if let Some(f) = &on_change {
                    f(&next, window, cx);
                }
            })
        };
        let texts = &self.texts;

        // `.g-sched-row { display: flex; align-items: flex-end; gap:
        // var(--spacing-05); flex-wrap: wrap }`.
        let row = || div().flex().flex_wrap().items_end().gap(tokens::SPACING_05);
        let fields = match v.kind {
            ScheduleKind::Daily => {
                let set_at = set.clone();
                let base = v.clone();
                row()
                    .child(
                        TimeField::new((id.clone(), "at"))
                            .label(texts.time.clone())
                            .value(v.at.clone())
                            .on_value_change(move |at, window, cx| {
                                set_at(Schedule { at, ..base.clone() }, window, cx)
                            }),
                    )
                    .child(
                        parts::field()
                            .child(parts::label(texts.days.clone()))
                            .child(self.days(&v, &set, window, cx)),
                    )
            }
            ScheduleKind::Interval => {
                let set_every = set.clone();
                let set_unit = set.clone();
                let base = v.clone();
                let base_unit = v.clone();
                let units = IntervalUnit::ALL
                    .iter()
                    .map(|u| (u.name(), texts.units[u.index()].as_ref()));
                row()
                    .child(
                        Stepper::new((id.clone(), "every"))
                            .label(texts.every.clone())
                            .value(v.every_or_default() as f64)
                            .min(1.)
                            .max(999.)
                            .on_value_change(move |n, window, cx| {
                                set_every(
                                    Schedule {
                                        every: Some(n.round().clamp(1., 999.) as u32),
                                        ..base.clone()
                                    },
                                    window,
                                    cx,
                                )
                            }),
                    )
                    .child(
                        // `.g-select { width: 8rem; height: var(--density-field-md) }`:
                        // Carbon's select at the compact field height, behavior from
                        // `NativeSelect` (GPUI has no native `<select>`).
                        div().w(rem(128.)).child(
                            NativeSelect::new((id.clone(), "unit"))
                                .label(texts.unit.clone())
                                .options(units)
                                .value(v.unit.name())
                                .size(FieldSize::Sm)
                                .on_value_change(move |name, window, cx| {
                                    if let Some(unit) = IntervalUnit::from_name(name) {
                                        set_unit(
                                            Schedule {
                                                unit,
                                                ..base_unit.clone()
                                            },
                                            window,
                                            cx,
                                        )
                                    }
                                }),
                        ),
                    )
            }
            // `.g-sched-row--col { flex-direction: column; align-items:
            // stretch; gap: var(--spacing-03) }`.
            ScheduleKind::Webhook => div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_03)
                .child(
                    CopyField::new(
                        (id.clone(), "url"),
                        v.url.clone().unwrap_or_else(|| texts.default_url.clone()),
                    )
                    .label(texts.webhook_url.clone()),
                )
                .child(parts::helper(texts.webhook_helper.clone())),
        };

        // `.g-sched { display: flex; flex-direction: column; gap: var(--spacing-05) }`.
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .min_w_0()
            .child(self.kind_switch(&v, &set, window, cx))
            .child(fields)
            .child(
                // `.g-sched-sum { display: flex; align-items: center; gap:
                // var(--spacing-03); padding: var(--spacing-03)
                // var(--spacing-04); background: var(--layer-01); font-size:
                // 0.875rem }`; `svg { fill: var(--icon-secondary) }`.
                sans(div(), 14., 400)
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .py(tokens::SPACING_03)
                    .px(tokens::SPACING_04)
                    .bg(color(ColorRole::Layer01))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(
                        Icon::carbon(CarbonIcon::Time)
                            .size_length(rem(14.))
                            .color(color(ColorRole::IconSecondary)),
                    )
                    .child(div().min_w_0().child(v.summary(texts))),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    fn daily(days: &[u8]) -> Schedule {
        Schedule {
            days: days.to_vec(),
            ..Schedule::default()
        }
    }

    #[test]
    fn daily_summaries() {
        let t = ScheduleTexts::default();
        assert_eq!(Schedule::default().summary(&t), "Every weekday at 09:00");
        assert_eq!(
            daily(&[0, 1, 2, 3, 4, 5, 6]).summary(&t),
            "Every day at 09:00"
        );
        assert_eq!(daily(&[1, 3]).summary(&t), "Every Mon, Wed at 09:00");
        // Five days including the weekend are listed, not "weekday".
        assert_eq!(
            daily(&[0, 1, 2, 3, 4]).summary(&t),
            "Every Sun, Mon, Tue, Wed, Thu at 09:00"
        );
    }

    #[test]
    fn interval_and_webhook_summaries() {
        let t = ScheduleTexts::default();
        let mut s = Schedule::of_kind(ScheduleKind::Interval);
        assert_eq!(s.summary(&t), "Every 30 minutes");
        s.every = Some(2);
        s.unit = IntervalUnit::Hours;
        assert_eq!(s.summary(&t), "Every 2 hours");
        let mut w = Schedule::of_kind(ScheduleKind::Webhook);
        assert_eq!(w.summary(&t), "When a webhook is received");
        w.event = Some("a push to main".into());
        assert_eq!(w.summary(&t), "When a push to main is received");
    }

    #[test]
    fn toggling_days_keeps_them_sorted() {
        let mut s = daily(&[1, 5]);
        s.toggle_day(3);
        assert_eq!(s.days, vec![1, 3, 5]);
        s.toggle_day(1);
        assert_eq!(s.days, vec![3, 5]);
        s.toggle_day(0);
        assert_eq!(s.days, vec![0, 3, 5]);
    }

    #[test]
    fn names_match_the_reference() {
        assert_eq!(
            ScheduleKind::ALL.map(ScheduleKind::name),
            ["daily", "interval", "webhook"]
        );
        assert_eq!(
            IntervalUnit::ALL.map(IntervalUnit::name),
            ["minutes", "hours", "days"]
        );
        assert_eq!(IntervalUnit::from_name("hours"), Some(IntervalUnit::Hours));
        assert_eq!(IntervalUnit::from_name("weeks"), None);
    }

    #[test]
    fn segment_and_day_looks() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(
            segment_look(true, true),
            (
                color(ColorRole::LayerSelectedInverse),
                color(ColorRole::TextInverse)
            )
        );
        assert_eq!(
            segment_look(false, true),
            (
                color(ColorRole::LayerHover01),
                color(ColorRole::TextPrimary)
            )
        );
        assert_eq!(
            segment_look(false, false).1,
            color(ColorRole::TextSecondary)
        );
        let (fill, ink, rule) = day_look(false, false);
        assert_eq!(fill, color(ColorRole::Field01));
        assert_eq!(ink, color(ColorRole::TextSecondary));
        assert_eq!(rule, color(ColorRole::BorderStrong01));
        assert_eq!(day_look(false, true).0, color(ColorRole::FieldHover01));
        let (fill, ink, rule) = day_look(true, false);
        assert_eq!(fill, color(ColorRole::LayerSelectedInverse));
        assert_eq!(ink, color(ColorRole::TextInverse));
        assert_eq!(rule, Hsla::transparent_black());
    }
}
