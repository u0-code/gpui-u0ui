//! TimeField: hours and minutes as two segments, optionally 12-hour
//! (`TimeField` in `src/ghost/primitives.jsx`; Carbon `TimePicker` is free
//! text plus an AM/PM select, segmented here to avoid format errors).
//!
//! - **Behavior**: each segment is a focusable spin field (Tab moves between
//!   them). ↑/↓ wrap hours by 1 and minutes by `step` (default 5); typed
//!   digits replace the value with the last two digits typed, clamped
//!   (`parseInt(value.slice(-2))`). The value is stored as 24-hour `HH:mm`.
//!   The AM/PM button flips 12 hours. Written here because Kit's text input
//!   cannot keep a segment formatted as two digits while typing.
//! - **Appearance** (`.g-time`): `field-01` fill, 1px `border-strong-01`
//!   rule, `focus` outline while a segment has focus, the focused segment on
//!   `highlight`; disabled at 50% without the rule.
use super::parts::{self, rem};
use crate::{
    components::form,
    density,
    icons::{CarbonIcon, Icon, IconSize},
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `v.split(':').map(Number)`: hours and minutes of `HH:mm` (0 when unparsable).
pub fn parse_hm(value: &str) -> (u32, u32) {
    let mut it = value
        .split(':')
        .map(|p| p.trim().parse::<u32>().unwrap_or(0));
    (it.next().unwrap_or(0) % 24, it.next().unwrap_or(0) % 60)
}

/// `HH:mm`.
pub fn format_hm(h: u32, m: u32) -> String {
    format!("{h:02}:{m:02}")
}

/// The hour shown: 24-hour as is, 12-hour `((n + 11) % 12) + 1`.
pub fn display_hour(h: u32, hour12: bool) -> u32 {
    if hour12 { (h + 11) % 12 + 1 } else { h }
}

/// ↑/↓ on a segment (`d` = ±1): hours wrap by 1, minutes by `step`.
pub fn arrow(h: u32, m: u32, segment: Segment, d: i32, step: u32) -> (u32, u32) {
    match segment {
        Segment::Hours => (((h as i32 + d + 24) % 24) as u32, m),
        Segment::Minutes => (h, ((m as i32 + d * step as i32).rem_euclid(60)) as u32),
    }
}

/// The value after the segment's text became `text` (`onChange`): the last
/// two characters as a number, clamped; `None` when not a number.
pub fn typed(h: u32, m: u32, segment: Segment, text: &str, hour12: bool) -> Option<(u32, u32)> {
    let chars: Vec<char> = text.chars().collect();
    let tail: String = chars[chars.len().saturating_sub(2)..].iter().collect();
    let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
    let n: u32 = digits.parse().ok()?;
    Some(match segment {
        Segment::Hours if hour12 => ((n.clamp(1, 12) % 12) + if h >= 12 { 12 } else { 0 }, m),
        Segment::Hours => (n.min(23), m),
        Segment::Minutes => (h, n.min(59)),
    })
}

/// The AM/PM button: `(h + 12) % 24`.
pub fn toggle_meridiem(h: u32) -> u32 {
    (h + 12) % 24
}

/// The focus handle a segment tracks: a tab stop in document order.
pub fn segment_tab_stop(focus: &FocusHandle) -> FocusHandle {
    focus.clone().tab_index(0).tab_stop(true)
}

/// A time segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Segment {
    /// Hours.
    Hours,
    /// Minutes.
    Minutes,
}

/// `.g-time` box colors on [`LayerLevel::One`]: (fill, rule, outline).
pub fn box_look(focus_within: bool, disabled: bool) -> (Hsla, Hsla, Hsla) {
    box_look_on(LayerLevel::One, focus_within, disabled)
}

/// `.g-time` box colors on `level`: (fill, rule, outline).
pub fn box_look_on(level: LayerLevel, focus_within: bool, disabled: bool) -> (Hsla, Hsla, Hsla) {
    let clear = Hsla::transparent_black();
    (
        color(level.field()),
        if disabled {
            clear
        } else {
            color(level.border_strong())
        },
        if focus_within && !disabled {
            color(ColorRole::Focus)
        } else {
            clear
        },
    )
}

type Handler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type SetTime = Rc<dyn Fn(u32, u32, &mut Window, &mut App)>;

/// A segmented time field. See the module docs.
#[derive(IntoElement)]
pub struct TimeField {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<SharedString>,
    default_value: SharedString,
    hour12: bool,
    disabled: bool,
    step: u32,
    level: LayerLevel,
    labels: (SharedString, SharedString, SharedString, SharedString),
    on_value_change: Option<Handler>,
}

impl TimeField {
    /// A time field with a stable `id` (uncontrolled, default `09:00`).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            default_value: "09:00".into(),
            hour12: false,
            disabled: false,
            step: 5,
            level: LayerLevel::One,
            // The reference's segment names and meridiem labels; override per locale.
            labels: ("Hours".into(), "Minutes".into(), "AM".into(), "PM".into()),
            on_value_change: None,
        }
    }

    /// The layer the field sits on (`LayerLevel::One` paints `field-01`,
    /// `LayerLevel::Two` `field-02`).
    pub fn level(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// The group label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Control the value (`HH:mm`, 24-hour).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The initial value when uncontrolled (`HH:mm`).
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Show 12-hour time with an AM/PM button.
    pub fn hour12(mut self, hour12: bool) -> Self {
        self.hour12 = hour12;
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Minutes per ↑/↓ (default 5).
    pub fn step(mut self, step: u32) -> Self {
        self.step = step.max(1);
        self
    }

    /// Segment names and meridiem labels (default Hours, Minutes, AM, PM).
    pub fn labels(
        mut self,
        hours: impl Into<SharedString>,
        minutes: impl Into<SharedString>,
        am: impl Into<SharedString>,
        pm: impl Into<SharedString>,
    ) -> Self {
        self.labels = (hours.into(), minutes.into(), am.into(), pm.into());
        self
    }

    /// Called with each new `HH:mm`.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for TimeField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let own = window.use_keyed_state((id.clone(), "value"), cx, {
            let d = self.default_value.clone();
            move |_, _| d
        });
        let value = self.value.clone().unwrap_or_else(|| own.read(cx).clone());
        let (h, m) = parse_hm(&value);
        let set: SetTime = {
            let own = own.clone();
            let on_change = self.on_value_change.clone();
            let controlled = self.value.is_some();
            Rc::new(move |h, m, window, cx| {
                let v: SharedString = format_hm(h, m).into();
                if !controlled {
                    own.update(cx, |s, cx| {
                        *s = v.clone();
                        cx.notify();
                    });
                }
                if let Some(f) = &on_change {
                    f(v, window, cx);
                }
            })
        };
        let field_h = density::current().field().px_value();
        let hf = parts::keyed_focus((id.clone(), "h-focus"), window, cx);
        let mf = parts::keyed_focus((id.clone(), "m-focus"), window, cx);
        let af = parts::keyed_focus((id.clone(), "ampm-focus"), window, cx);
        let focus_within = !self.disabled
            && (hf.is_focused(window) || mf.is_focused(window) || af.is_focused(window));
        let level = self.level;
        let (fill, rule, outline) = box_look_on(level, focus_within, self.disabled);
        let (hours_label, minutes_label, am, pm) = self.labels.clone();

        let segment = |segment: Segment, focus: &FocusHandle, label: SharedString| {
            let shown = match segment {
                Segment::Hours => display_hour(h, self.hour12),
                Segment::Minutes => m,
            };
            let text = format!("{shown:02}");
            let focused = focus.is_focused(window);
            let set = set.clone();
            let hour12 = self.hour12;
            let step = self.step;
            let disabled = self.disabled;
            div()
                .id((id.clone(), label.clone()))
                // Both segments are tab stops, like the reference's two `<input>`s.
                // A tracked handle carries its own tab-stop flag; a div's
                // `.tab_index` does not reach it.
                .when(!disabled, |d| d.track_focus(&segment_tab_stop(focus)))
                .flex()
                .items_center()
                .justify_center()
                // `.g-time-seg { width: 1.75rem; height: calc(var(--density-field-md) - 8px) }`.
                .w(rem(28.))
                .h(rem(field_h - 8.))
                // `.g-time-seg:focus { background: var(--highlight) }`.
                .when(focused, |d| d.bg(color(ColorRole::Highlight)))
                .text_color(color(ColorRole::TextPrimary))
                .cursor(if disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::IBeam
                })
                .on_key_down(move |e: &KeyDownEvent, window, cx| {
                    let ks = &e.keystroke;
                    if ks.modifiers.platform || ks.modifiers.control || ks.modifiers.alt {
                        return;
                    }
                    let next = match ks.key.as_str() {
                        "up" => Some(arrow(h, m, segment, 1, step)),
                        "down" => Some(arrow(h, m, segment, -1, step)),
                        "backspace" => {
                            let mut t = text.clone();
                            t.pop();
                            typed(h, m, segment, &t, hour12)
                        }
                        k if k.len() == 1 && k.chars().all(|c| c.is_ascii_digit()) => {
                            typed(h, m, segment, &format!("{text}{k}"), hour12)
                        }
                        _ => return,
                    };
                    cx.stop_propagation();
                    if let Some((h, m)) = next {
                        set(h, m, window, cx);
                    }
                })
                .child(format!("{shown:02}"))
        };
        let hours = segment(Segment::Hours, &hf, hours_label);
        let minutes = segment(Segment::Minutes, &mf, minutes_label);

        let ampm = self.hour12.then(|| {
            let set = set.clone();
            let ring = parts::focus_visible(&af, window);
            gpui_kit::base::Button::new((id.clone(), "ampm"))
                .track_focus(&af)
                .disabled(self.disabled)
                .relative()
                .flex()
                .items_center()
                // `.g-time-ampm { margin-left: spacing-02; padding: 0 spacing-03;
                // height: 1.5rem; background: layer-accent; font-size: 0.75rem }`.
                .ml(rem(4.))
                .px(rem(8.))
                .h(rem(24.))
                .bg(color(level.layer_accent()))
                .text_size(rem(12.))
                .text_color(color(ColorRole::TextPrimary))
                .cursor(if self.disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::PointingHand
                })
                .child(if h < 12 { am } else { pm })
                .when(ring, |b| b.child(parts::focus_ring()))
                .on_click(move |_, window, cx| set(toggle_meridiem(h), m, window, cx))
        });

        let group = div()
            .id((id.clone(), "group"))
            .relative()
            .flex()
            .items_center()
            .gap(px(2.))
            .px(rem(8.))
            .min_h(rem(field_h))
            .bg(fill)
            .type_style(TypeStyle::BodyCompact01)
            .when(self.disabled, |d| d.opacity(0.5))
            .when(!self.disabled, |d| {
                let hf = hf.clone();
                d.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if !window
                        .focused(cx)
                        .is_some_and(|f| f == hf || f == mf || f == af)
                    {
                        hf.focus(window, cx);
                    }
                })
            })
            .child(
                // `.g-time > svg { fill: var(--icon-secondary); margin-right: spacing-02 }`.
                div().mr(rem(4.)).child(
                    Icon::carbon(CarbonIcon::Time)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconSecondary)),
                ),
            )
            .child(hours)
            .child(div().text_color(color(ColorRole::TextPrimary)).child(":"))
            .child(minutes)
            .children(ampm)
            .child(form::bottom_rule(rule))
            .child(form::inset_outline(2., outline));

        parts::field()
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(div().flex().child(group))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_wrap_and_step() {
        assert_eq!(arrow(23, 0, Segment::Hours, 1, 5), (0, 0));
        assert_eq!(arrow(0, 0, Segment::Hours, -1, 5), (23, 0));
        assert_eq!(arrow(9, 55, Segment::Minutes, 1, 5), (9, 0));
        assert_eq!(arrow(9, 0, Segment::Minutes, -1, 5), (9, 55));
        assert_eq!(arrow(9, 2, Segment::Minutes, -1, 5), (9, 57));
    }

    #[test]
    fn typing_keeps_the_last_two_digits() {
        // 24-hour: "09" + "1" → "91" → min(23, 91).
        assert_eq!(typed(9, 0, Segment::Hours, "091", false), Some((23, 0)));
        assert_eq!(typed(9, 0, Segment::Hours, "0", false), Some((0, 0)));
        assert_eq!(typed(9, 0, Segment::Minutes, "007", false), Some((9, 7)));
        assert_eq!(typed(9, 0, Segment::Minutes, "0099", false), Some((9, 59)));
        // 12-hour keeps the half of the day: 18:30 shows 06; typing 1 → "61" → 12 → 12 % 12 + 12.
        assert_eq!(typed(18, 30, Segment::Hours, "061", true), Some((12, 30)));
        assert_eq!(typed(9, 0, Segment::Hours, "093", true), Some((0, 0)));
        assert_eq!(typed(9, 0, Segment::Hours, "03", true), Some((3, 0)));
        assert_eq!(typed(9, 0, Segment::Hours, "", false), None);
    }

    #[test]
    fn twelve_hour_display_and_meridiem() {
        assert_eq!(display_hour(0, true), 12);
        assert_eq!(display_hour(18, true), 6);
        assert_eq!(display_hour(12, true), 12);
        assert_eq!(display_hour(18, false), 18);
        assert_eq!(toggle_meridiem(18), 6);
        assert_eq!(toggle_meridiem(9), 21);
        assert_eq!(parse_hm("07:15"), (7, 15));
        assert_eq!(format_hm(7, 5), "07:05");
    }

    #[test]
    fn layer_levels_select_contextual_field_tokens() {
        crate::theme::tests_set(crate::CarbonTheme::G100);
        let (fill1, rule1, _) = box_look_on(LayerLevel::One, false, false);
        assert_eq!(fill1, color(ColorRole::Field01));
        assert_eq!(rule1, color(ColorRole::BorderStrong01));
        let (fill2, rule2, _) = box_look_on(LayerLevel::Two, false, false);
        assert_eq!(fill2, color(ColorRole::Field02));
        assert_eq!(rule2, color(ColorRole::BorderStrong02));
    }
}
