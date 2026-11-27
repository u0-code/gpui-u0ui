//! Switch: shadcn `Switch` → Carbon `Toggle` (`.cds--toggle`).
//!
//! - **Behavior** is Kit's [`gpui_kit::base::Switch`]: focus, Space/Enter and
//!   pointer activation, `role=switch`, disabled. The whole appearance (track
//!   and state text) is its child, so clicking the text toggles too.
//! - **Appearance**: a 48×24 (`sm` 32×16) pill track in `toggle-off`, checked
//!   `support-success`; an 18px (`sm` 10px) `icon-on-color` knob inset 3px; the
//!   `sm` checked knob carries a `support-success` check; the state text
//!   (`labelOff`/`labelOn`, `body-01`) shows beside the track; the label
//!   above is `label-01` `text-secondary` 16px over the track. Focus and press draw a
//!   2px `focus` ring 1px outside the track. Disabled: `button-disabled`
//!   track, `icon-on-color-disabled` knob, `text-disabled`. Read-only: hollow
//!   track (`icon-disabled` border) with an `icon-primary` knob.
//! - **Label and state text** ([`state_text`]): as the reference `Switch`
//!   (`labelOn = 'On'`, `labelOff = 'Off'` → Carbon `labelB`/`labelA`), a
//!   switch with a label shows "On"/"Off" unless `label_on`/`label_off`
//!   override them. Without a label the reference passes `hideLabel`, which
//!   drops the state text; explicit `label_on`/`label_off` still show here.
//!   [`Switch::hide_label`] is `cu-switch--bare`: no label above and no state
//!   text (`.cu-switch--bare .cds--toggle__text { display: none }`), only the
//!   track, with the label kept as the accessible name.
//! - **Box** ([`box_width`]): a switch with no label and no state text (the
//!   reference passes `hideLabel={!label}`) still takes the 8px column gap
//!   after the track: Carbon's computed `.cds--toggle` is 56px wide around
//!   the 48px switch (40 around `sm`), so such a switch in a row sits 8px
//!   further in than its track alone (`SettingsRow`: 28px from the row's
//!   right edge). [`Switch::hide_label`] drops the gap
//!   (`.cu-switch--bare .cds--toggle__appearance { column-gap: 0 }`): it
//!   stands in for the reference's `g-switch` (`SwitchCompact`), a bare
//!   32px track.
//! - **Motion**: `transition: background-color 70ms, transform 70ms
//!   cubic-bezier(0.2, 0, 1, 0.9)`; the knob slide is spatial, so reduced
//!   motion snaps it.
//!
//! ```ignore
//! Switch::new("autoscale").label("Auto-scaling").default_checked(true)
//!     .label_on("On").label_off("Off")
//! ```
use super::form::{self, rem};
use crate::{
    icons::{CarbonIcon, Icon},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.cds--toggle__switch { transition: background-color 70ms cubic-bezier(0.2, 0, 1, 0.9) }`.
pub const SWITCH_COLOR_MOTION: Motion = Motion::new(Curve::ProductiveExit, Speed::Fast01);
/// `.cds--toggle__switch::before { transition: transform 70ms cubic-bezier(0.2, 0, 1, 0.9) }`.
pub const SWITCH_KNOB_MOTION: Motion =
    Motion::new(Curve::ProductiveExit, Speed::Fast01).spatial(true);

/// `index.d.ts` `SwitchProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SwitchSize {
    /// 48×24 (`.cds--toggle__switch`).
    #[default]
    Md,
    /// 32×16 (`.cds--toggle__appearance--sm`).
    Sm,
}

impl SwitchSize {
    /// Both sizes, in `index.d.ts` order.
    pub const ALL: [Self; 2] = [Self::Md, Self::Sm];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Md => "md",
            Self::Sm => "sm",
        }
    }
}

/// Track and knob geometry in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchGeometry {
    /// Track width (`inline-size`).
    pub width: f32,
    /// Track height (`block-size`).
    pub height: f32,
    /// Knob diameter.
    pub knob: f32,
    /// Knob inset from the track's top-left (`inset-block-start/inline-start`).
    pub inset: f32,
    /// Knob travel when checked (`translateX`).
    pub travel: f32,
}

impl SwitchGeometry {
    /// The geometry of `size`; read-only moves the knob to a 2px inset.
    pub const fn of(size: SwitchSize, readonly: bool) -> Self {
        let inset = if readonly { 2. } else { 3. };
        match size {
            // `.cds--toggle__switch { block-size: 1.5rem; inline-size: 3rem }`,
            // `::before { block-size: 1.125rem; inset: 0.1875rem }`, checked `translateX(1.5rem)`.
            SwitchSize::Md => Self {
                width: 48.,
                height: 24.,
                knob: 18.,
                inset,
                travel: 24.,
            },
            // `--sm .cds--toggle__switch { block-size: 1rem; inline-size: 2rem }`,
            // `::before { 0.625rem }`, checked `translateX(1rem)`.
            SwitchSize::Sm => Self {
                width: 32.,
                height: 16.,
                knob: 10.,
                inset,
                travel: 16.,
            },
        }
    }
}

impl SwitchGeometry {
    /// `.cds--toggle__appearance { column-gap: 0.5rem }`: the gap after the
    /// track.
    pub const TEXT_GAP: f32 = 8.;
}

/// The width of a switch with no state text: the track plus the column
/// gap (computed `.cds--toggle`: 56px for `md`, 40px for `sm`), or the
/// track alone when `hide_label` (`.cu-switch--bare`).
pub const fn box_width(size: SwitchSize, hide_label: bool) -> f32 {
    let track = SwitchGeometry::of(size, false).width;
    if hide_label {
        track
    } else {
        track + SwitchGeometry::TEXT_GAP
    }
}

/// The state a switch look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwitchState {
    /// On.
    pub checked: bool,
    /// Disabled.
    pub disabled: bool,
    /// Read-only.
    pub readonly: bool,
}

/// The colors of a switch in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchLook {
    /// Track fill.
    pub track: Hsla,
    /// Track border (read-only only).
    pub track_border: Option<Hsla>,
    /// Knob fill.
    pub knob: Hsla,
    /// The `sm` check mark (`.cds--toggle__check`).
    pub check: Hsla,
    /// State text (`.cds--toggle__text`).
    pub text: Hsla,
    /// Label above (`.cds--toggle__label-text`).
    pub label: Hsla,
}

impl SwitchLook {
    /// The look of a switch in `state` (`.cds--toggle*` rules).
    pub fn of(state: SwitchState) -> Self {
        if state.disabled {
            // `.cds--toggle--disabled …`.
            return Self {
                track: color(ColorRole::ButtonDisabled),
                track_border: None,
                knob: color(ColorRole::IconOnColorDisabled),
                check: color(ColorRole::ButtonDisabled),
                text: color(ColorRole::TextDisabled),
                label: color(ColorRole::TextDisabled),
            };
        }
        if state.readonly {
            // `.cds--toggle--readonly .cds--toggle__switch { border: 1px solid
            // var(--icon-disabled); background-color: transparent }`, knob `icon-primary`,
            // check `background`.
            return Self {
                track: Hsla::transparent_black(),
                track_border: Some(color(ColorRole::IconDisabled)),
                knob: color(ColorRole::IconPrimary),
                check: color(ColorRole::Background),
                text: color(ColorRole::TextPrimary),
                label: color(ColorRole::TextSecondary),
            };
        }
        Self {
            // `toggle-off`, checked `support-success`.
            track: color(if state.checked {
                ColorRole::SupportSuccess
            } else {
                ColorRole::ToggleOff
            }),
            track_border: None,
            knob: color(ColorRole::IconOnColor),
            check: color(ColorRole::SupportSuccess),
            text: color(ColorRole::TextPrimary),
            label: color(ColorRole::TextSecondary),
        }
    }
}

/// The reference's default `labelOn` / `labelOff`.
pub const DEFAULT_LABEL_ON: &str = "On";
/// See [`DEFAULT_LABEL_ON`].
pub const DEFAULT_LABEL_OFF: &str = "Off";

/// The state text beside the track (`.cds--toggle__text`), or `None` when
/// none is drawn. See the module docs.
pub fn state_text(
    checked: bool,
    has_label: bool,
    hide_label: bool,
    label_on: Option<SharedString>,
    label_off: Option<SharedString>,
) -> Option<SharedString> {
    if hide_label {
        return None;
    }
    let (text, default) = if checked {
        (label_on, DEFAULT_LABEL_ON)
    } else {
        (label_off, DEFAULT_LABEL_OFF)
    };
    text.or_else(|| has_label.then(|| default.into()))
}

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A Carbon toggle switch. See the module docs.
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    label: Option<SharedString>,
    hide_label: bool,
    checked: Option<bool>,
    default_checked: bool,
    label_on: Option<SharedString>,
    label_off: Option<SharedString>,
    size: SwitchSize,
    disabled: bool,
    readonly: bool,
    on_checked_change: Option<ChangeHandler>,
}

impl Switch {
    /// A switch with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            hide_label: false,
            checked: None,
            default_checked: false,
            label_on: None,
            label_off: None,
            size: SwitchSize::Md,
            disabled: false,
            readonly: false,
            on_checked_change: None,
        }
    }

    /// The label above the track (also the accessible name).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `hideLabel` (`cu-switch--bare`): draw only the track; the label stays
    /// the accessible name.
    pub fn hide_label(mut self, hide: bool) -> Self {
        self.hide_label = hide;
        self
    }

    /// Controlled `checked`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// `defaultChecked` for an uncontrolled switch.
    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }

    /// `labelOn`: state text while on (default "On" when a label is set).
    pub fn label_on(mut self, text: impl Into<SharedString>) -> Self {
        self.label_on = Some(text.into());
        self
    }

    /// `labelOff`: state text while off (default "Off" when a label is set).
    pub fn label_off(mut self, text: impl Into<SharedString>) -> Self {
        self.label_off = Some(text.into());
        self
    }

    /// `size`.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only (Carbon `readOnly`): shows the value, ignores activation.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
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

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (checked, store) = form::controlled(
            (id.clone(), "checked"),
            self.checked,
            self.default_checked,
            window,
            cx,
        );
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let pressed = window.use_keyed_state((id.clone(), "pressed"), cx, |_, _| false);
        let state = SwitchState {
            checked,
            disabled: self.disabled,
            readonly: self.readonly,
        };
        let look = SwitchLook::of(state);
        let geometry = SwitchGeometry::of(self.size, self.readonly);
        let track = motion::animate_color(
            (id.clone(), "track"),
            look.track,
            SWITCH_COLOR_MOTION,
            window,
            cx,
        );
        let knob_x = motion::animate(
            (id.clone(), "knob"),
            if checked { geometry.travel } else { 0. },
            SWITCH_KNOB_MOTION,
            window,
            cx,
        );
        // `.cds--toggle__button:focus + … ::after, .cds--toggle:not(--disabled):active ::after
        // { outline: 2px solid var(--focus); outline-offset: 1px; border-radius: 1rem }`.
        let ring = !self.disabled && (focus.is_focused(window) || *pressed.read(cx));
        let inert = self.disabled || self.readonly;
        let has_label = self.label.as_ref().is_some_and(|l| !l.is_empty());
        let text = state_text(
            checked,
            has_label,
            self.hide_label,
            self.label_on.clone(),
            self.label_off.clone(),
        );
        let on_change = self.on_checked_change.clone();

        let knob = div()
            .absolute()
            .top(rem(geometry.inset))
            .left(rem(geometry.inset + knob_x))
            .size(rem(geometry.knob))
            .rounded_full()
            .bg(look.knob)
            // `.cds--toggle__check`: 6×5 at top 6 / right 5 of the sm track, inside the knob.
            .when(self.size == SwitchSize::Sm && checked, |this| {
                this.flex().items_center().justify_center().child(
                    Icon::carbon(CarbonIcon::Checkmark)
                        .size_length(rem(8.))
                        .color(look.check),
                )
            });

        let track_el = div()
            .relative()
            .flex_shrink_0()
            .w(rem(geometry.width))
            .h(rem(geometry.height))
            .rounded(rem(geometry.height / 2.))
            .bg(track)
            .when_some(look.track_border, |this, border| {
                this.border_1().border_color(border)
            })
            .child(knob)
            .when(ring, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(rem(-3.))
                        .left(rem(-3.))
                        .right(rem(-3.))
                        .bottom(rem(-3.))
                        .rounded(tokens::BORDER_RADIUS_16)
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            });

        // `.cds--toggle__appearance { display: inline-grid; column-gap: 0.5rem }`.
        // Without state text the gap stays (`hideLabel={!label}`: the
        // computed `.cds--toggle` is 56px around the 48px switch), unless
        // `hide_label` (`.cu-switch--bare .cds--toggle__appearance {
        // column-gap: 0 }`).
        let keep_gap = text.is_none() && !self.hide_label;
        let appearance = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(tokens::SPACING_03)
            .when(keep_gap, |this| this.pr(rem(SwitchGeometry::TEXT_GAP)))
            .child(track_el)
            .when_some(text, |this, text| {
                this.child(
                    div()
                        .type_style(TypeStyle::Body01)
                        .text_color(look.text)
                        .whitespace_nowrap()
                        .child(text),
                )
            });

        let switch = gpui_kit::base::Switch::new(id.clone())
            .track_focus(&focus)
            .checked(checked)
            .disabled(inert)
            .when_some(self.label.clone(), |this, label| {
                this.accessibility_label(label)
            })
            .flex()
            .flex_col()
            .items_start()
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else if self.readonly {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            })
            .on_mouse_down(MouseButton::Left, {
                let pressed = pressed.clone();
                move |_, _, cx| {
                    pressed.update(cx, |p, cx| {
                        *p = true;
                        cx.notify();
                    })
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let pressed = pressed.clone();
                move |_, _, cx| {
                    pressed.update(cx, |p, cx| {
                        *p = false;
                        cx.notify();
                    })
                }
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                pressed.update(cx, |p, cx| {
                    if *p {
                        *p = false;
                        cx.notify();
                    }
                })
            })
            .on_change(move |next, _, window, cx| {
                store.update(cx, |c, cx| {
                    *c = next;
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(next, window, cx);
                }
            })
            // `.cds--toggle__label-text { label-01; color: text-secondary; margin-block-end: 1rem }`.
            .when_some(
                self.label
                    .clone()
                    .filter(|l| !l.is_empty() && !self.hide_label),
                |this, label| {
                    this.child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(look.label)
                            .mb(tokens::SPACING_05)
                            .child(label),
                    )
                },
            )
            .child(appearance);
        div().flex_shrink_0().child(switch)
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
    fn colors_follow_toggle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let off = SwitchLook::of(SwitchState::default());
        assert_eq!(hex(off.track), "#8d8d8d");
        assert_eq!(hex(off.knob), "#ffffff");
        let on = SwitchLook::of(SwitchState {
            checked: true,
            ..Default::default()
        });
        assert_eq!(hex(on.track), "#24a148");
        let disabled = SwitchLook::of(SwitchState {
            disabled: true,
            ..Default::default()
        });
        assert_eq!(hex(disabled.track), "#c6c6c6");
        assert_eq!(hex(disabled.knob), "#8d8d8d");
        let readonly = SwitchLook::of(SwitchState {
            readonly: true,
            ..Default::default()
        });
        assert_eq!(readonly.track.a, 0.);
        assert_eq!(hex(readonly.knob), "#161616");
    }

    #[test]
    fn state_text_follows_reference_toggle() {
        let s = |t: &str| Some(SharedString::from(t.to_string()));
        // A labelled switch defaults to On/Off.
        assert_eq!(state_text(true, true, false, None, None), s("On"));
        assert_eq!(state_text(false, true, false, None, None), s("Off"));
        // Overrides win.
        assert_eq!(state_text(true, true, false, s("Yes"), s("No")), s("Yes"));
        // No label: no default text; explicit texts still show.
        assert_eq!(state_text(true, false, false, None, None), None);
        assert_eq!(state_text(false, false, false, None, s("Dark")), s("Dark"));
        // hideLabel (cu-switch--bare) hides the state text.
        assert_eq!(state_text(true, true, true, s("Yes"), None), None);
    }

    #[test]
    fn geometry_follows_toggle_css() {
        let md = SwitchGeometry::of(SwitchSize::Md, false);
        assert_eq!(
            (md.width, md.height, md.knob, md.inset, md.travel),
            (48., 24., 18., 3., 24.)
        );
        let sm = SwitchGeometry::of(SwitchSize::Sm, false);
        assert_eq!(
            (sm.width, sm.height, sm.knob, sm.travel),
            (32., 16., 10., 16.)
        );
        // Knob ends 3px from the right edge when checked.
        assert_eq!(md.inset + md.travel + md.knob, md.width - 3.);
        assert_eq!(SwitchGeometry::of(SwitchSize::Md, true).inset, 2.);
        // A label-less `.cds--toggle` is 56px (computed), 40px when `sm`;
        // `.cu-switch--bare` drops the gap.
        assert_eq!(box_width(SwitchSize::Md, false), 56.);
        assert_eq!(box_width(SwitchSize::Sm, false), 40.);
        assert_eq!(box_width(SwitchSize::Md, true), 48.);
        assert_eq!(box_width(SwitchSize::Sm, true), 32.);
    }
}
