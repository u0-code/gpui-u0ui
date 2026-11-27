//! Slider: shadcn `Slider` → Carbon `Slider` (`.cds--slider`), with the
//! value input beside it; `range` → two handles (`unstable_valueUpper`).
//!
//! - **Behavior** is Kit's headless slider over the caller's [`SliderState`]
//!   (min/max/step, single or range value): [`gpui_kit::base::Slider`] (role,
//!   ←/→ stepping, release), [`gpui_kit::base::SliderTrack`] (press to jump,
//!   drag), [`gpui_kit::base::SliderIndicator`] (measures the bar) and
//!   [`gpui_kit::base::SliderThumb`] (thumb drags). Each thumb is focusable
//!   and steps itself with ←/→ (Shift ×4, Carbon's `stepMultiplier`). The
//!   number inputs are Kit [`InputState`] editors that commit on Enter or blur.
//! - **Appearance**: `[min label] [track] [max label] [input]` 16px apart
//!   (`.cds--slider-container { gap: 1rem }`), range inputs on both ends; a
//!   2px `border-subtle` track with a 2×4 center tick, the filled part
//!   `layer-selected-inverse` (`border-interactive` while a thumb is
//!   focused); a 14px `layer-selected-inverse` thumb that grows to 20px
//!   (`scale(1.4286)`) on hover, press and focus, and turns `interactive` with
//!   an inner `layer` ring when focused; range labels `body-compact-01`; the
//!   input is a 64px Carbon text field. Disabled: `border-disabled` track and
//!   thumbs, `not-allowed`.
//!   Read-only: no thumb, the arrow cursor, a transparent input; focusable,
//!   but pointer and arrows leave the value alone.
//!   Range thumbs are Carbon's 16×24 lower/upper handle glyphs
//!   (`.cds--slider__thumb-wrapper--lower/--upper`, `@carbon/react`
//!   `LowerHandle`/`UpperHandle`): a wedge with a 0.92px bar on its inner edge,
//!   the lower one left of its value and the upper one right of it, filled
//!   `layer-selected-inverse` (hover `text-secondary`, disabled
//!   `border-disabled`); focused, the hollow `*HandleFocus` glyph in
//!   `interactive` over a 2px `background` strip that hides the track.
//!   Pressing a thumb focuses it, so a drag leaves it focused for the arrows.
//! - **Motion**: thumb size/color and fill color over 110ms productive
//!   (`.cds--slider__thumb { transition: transform 110ms, background 110ms }`);
//!   the size change is spatial. Range glyphs change at once
//!   (`.cds--slider__thumb--lower, --upper { transition: none }`).
//!
//! ```ignore
//! let memory = cx.new(|_| SliderState::new().min(0.).max(64.).default_value(24.));
//! Slider::new("memory", &memory).label("Memory (GB)")
//! ```
use super::form::{self, FieldSize, rem};
use super::input::{FieldLook, FieldState, InputState};
use crate::{
    density::{self, Density},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, AppContext as _, CursorStyle, ElementId, Entity, FocusHandle, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _, Size,
        input::{Enter, Input as KitInput},
    },
    div,
    prelude::FluentBuilder as _,
    px, relative,
};

/// Kit's slider state and value, re-exported.
pub use gpui_kit::component::slider::{SliderState, SliderValue};

/// `transition: transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9), background 110ms …`.
pub const THUMB_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// Thumb diameter at rest (`.cds--slider__thumb-wrapper { 0.875rem }`).
pub const THUMB: f32 = 14.;
/// Thumb diameter on hover/press/focus (`transform: scale(1.4286)`).
pub const THUMB_ACTIVE: f32 = 20.;
/// Carbon's `stepMultiplier` for Shift+arrow.
pub const STEP_MULTIPLIER: f32 = 4.;

/// The state a thumb look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThumbState {
    /// Pointer over it.
    pub hovered: bool,
    /// Pressed.
    pub pressed: bool,
    /// Keyboard focus.
    pub focused: bool,
    /// Disabled.
    pub disabled: bool,
}

/// The paint of one thumb.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThumbLook {
    /// Diameter in px.
    pub size: f32,
    /// Fill.
    pub fill: Hsla,
    /// Inner ring (`inset 0 0 0 2px interactive, inset 0 0 0 3px layer`): the
    /// 1px `layer` ring 2px in, when focused.
    pub inner_ring: Option<Hsla>,
}

impl ThumbLook {
    /// The look of a thumb (`.cds--slider__thumb*`).
    pub fn of(level: LayerLevel, state: ThumbState) -> Self {
        if state.disabled {
            // `.cds--slider--disabled .cds--slider__thumb { background-color: var(--border-disabled) }`.
            return Self {
                size: THUMB,
                fill: color(ColorRole::BorderDisabled),
                inner_ring: None,
            };
        }
        let grown = state.hovered || state.pressed || state.focused;
        Self {
            size: if grown { THUMB_ACTIVE } else { THUMB },
            // `:focus { background-color: var(--interactive) }`.
            fill: color(if state.focused {
                ColorRole::Interactive
            } else {
                ColorRole::LayerSelectedInverse
            }),
            inner_ring: state.focused.then(|| color(level.layer())),
        }
    }
}

/// Range handle glyph box (`.cds--slider__thumb-wrapper--lower, --upper
/// { block-size: 24px; inline-size: 16px }`).
pub const HANDLE: (f32, f32) = (16., 24.);

/// How a range handle is painted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HandleLook {
    /// The hollow focus glyph (`.cds--slider__thumb-icon--focus`) instead of
    /// the filled one.
    pub focus_glyph: bool,
    /// Glyph fill.
    pub fill: Hsla,
}

impl HandleLook {
    /// The look of a range handle (`.cds--slider__thumb--lower/--upper` rules).
    pub fn of(state: ThumbState) -> Self {
        if state.disabled {
            // `.cds--slider--disabled .cds--slider__thumb--lower .cds--slider__thumb-icon
            // { fill: var(--border-disabled) }`.
            return Self {
                focus_glyph: false,
                fill: color(ColorRole::BorderDisabled),
            };
        }
        if state.focused {
            // `:focus .cds--slider__thumb-icon--focus { display: block }`, `fill: interactive`.
            return Self {
                focus_glyph: true,
                fill: color(ColorRole::Interactive),
            };
        }
        Self {
            focus_glyph: false,
            // `:hover .cds--slider__thumb-icon { fill: var(--text-secondary) }`, else
            // `.cds--slider__thumb-icon { fill: var(--layer-selected-inverse) }`.
            fill: color(if state.hovered {
                ColorRole::TextSecondary
            } else {
                ColorRole::LayerSelectedInverse
            }),
        }
    }
}

/// The closed contours of a range handle glyph in its 16×24 box, filled
/// even-odd (`@carbon/react` `SliderHandles`): the inner-edge bar
/// (`M15.08 6.46H16v11.08h-.92z`), the wedge (`M0 .087 15.08 6.46v11.08L0
/// 23.913V.087Z`) and, for the focus glyph, the wedge's hollow (`m1.5 2.26
/// 12.08 5.105v9.096L1.5 21.652V2.347Z`). The upper handle mirrors the lower.
pub fn handle_contours(upper: bool, focus: bool) -> Vec<Vec<(f32, f32)>> {
    let mut contours = vec![
        vec![(15.08, 6.46), (16., 6.46), (16., 17.54), (15.08, 17.54)],
        vec![(0., 0.087), (15.08, 6.46), (15.08, 17.54), (0., 23.913)],
    ];
    if focus {
        contours.push(vec![
            (1.5, 2.347),
            (13.58, 7.452),
            (13.58, 16.548),
            (1.5, 21.652),
        ]);
    }
    if upper {
        for contour in &mut contours {
            for point in contour.iter_mut() {
                point.0 = HANDLE.0 - point.0;
            }
        }
    }
    contours
}

/// A range handle's box: its left edge relative to the value point and its
/// width (`--lower { transform: translate(-100%, -50%) }`, `--upper {
/// translate(0, -50%) }`).
pub fn handle_offset(upper: bool) -> f32 {
    if upper { 0. } else { -HANDLE.0 }
}

/// The track colors: (rail, fill).
pub fn track_colors(focused: bool, disabled: bool) -> (Hsla, Hsla) {
    if disabled {
        let c = color(ColorRole::BorderDisabled);
        return (c, c);
    }
    (
        // `.cds--slider__track { background: var(--border-subtle) }`.
        color(ColorRole::BorderSubtle01),
        // `.cds--slider__filled-track { background: var(--layer-selected-inverse) }`,
        // `.cds--slider__thumb:focus ~ .cds--slider__filled-track { var(--border-interactive) }`.
        color(if focused {
            ColorRole::BorderInteractive
        } else {
            ColorRole::LayerSelectedInverse
        }),
    )
}

/// Format a value for the labels and inputs: integers without decimals.
pub fn format_value(value: f32) -> String {
    if (value - value.round()).abs() < 1e-4 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// Parse typed text into a value clamped to `[min, max]`.
pub fn parse_value(text: &str, min: f32, max: f32) -> Option<f32> {
    text.trim().parse::<f32>().ok().map(|v| v.clamp(min, max))
}

#[derive(Default)]
struct ThumbPointer {
    hovered: bool,
    pressed: bool,
}

struct Inputs {
    lower: Entity<InputState>,
    upper: Entity<InputState>,
    focus: [FocusHandle; 2],
    was_focused: [bool; 2],
}

/// The track and thumb cursor: `not-allowed` when disabled, the arrow when
/// read-only (`.cds--slider--readonly { cursor: default }`), else the pointer.
pub fn pointer_cursor(disabled: bool, readonly: bool) -> CursorStyle {
    if disabled {
        CursorStyle::OperationNotAllowed
    } else if readonly {
        CursorStyle::Arrow
    } else {
        CursorStyle::PointingHand
    }
}

/// A Carbon slider. See the module docs.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    state: Entity<SliderState>,
    label: Option<SharedString>,
    disabled: bool,
    readonly: bool,
    hide_input: bool,
    density: Option<Density>,
    level: LayerLevel,
}

impl Slider {
    /// A slider with a stable `id` over the caller's `state` (a range value
    /// makes it a two-handle slider).
    pub fn new(id: impl Into<ElementId>, state: &Entity<SliderState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: None,
            disabled: false,
            readonly: false,
            hide_input: false,
            density: None,
            level: LayerLevel::One,
        }
    }

    /// `label` (Carbon `labelText`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only (`readOnly`): the value shows on the track without a thumb
    /// (`.cds--slider-container--readonly .cds--slider__thumb { block-size: 0;
    /// inline-size: 0 }`), the arrow cursor (`.cds--slider--readonly { cursor:
    /// default }`) and a transparent input
    /// (`.cds--slider-container--readonly .cds--slider-text-input { background-color:
    /// transparent }`); it keeps focus but pointer and keys do not change it.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Hide the value input (Carbon `hideTextInput`).
    pub fn hide_input(mut self, hide: bool) -> Self {
        self.hide_input = hide;
        self
    }

    /// Override the app density (the input height).
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// The layer the slider sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Read-only and disabled sliders take no pointer input.
        let inert = self.disabled || self.readonly;
        let readonly = self.readonly && !self.disabled;
        let cursor = pointer_cursor(self.disabled, self.readonly);
        let id = self.id.clone();
        let state = self.state.clone();
        let density = self.density.unwrap_or_else(density::current);
        let (value, min, max, pct) = {
            let s = state.read(cx);
            (s.value(), s.min_value(), s.max_value(), s.percentage())
        };
        let range = value.is_range();
        let inputs = window.use_keyed_state((id.clone(), "inputs"), cx, |window, cx| Inputs {
            lower: cx.new(|cx| InputState::new(window, cx)),
            upper: cx.new(|cx| InputState::new(window, cx)),
            focus: [cx.focus_handle(), cx.focus_handle()],
            was_focused: [false, false],
        });
        let thumb_focus = inputs.read(cx).focus.clone();
        let pointers = [
            window.use_keyed_state((id.clone(), "p0"), cx, |_, _| ThumbPointer::default()),
            window.use_keyed_state((id.clone(), "p1"), cx, |_, _| ThumbPointer::default()),
        ];

        // Keep each input in step with the value; commit typed text on blur.
        let values = [value.start(), value.end()];
        let editors = {
            let i = inputs.read(cx);
            [i.lower.clone(), i.upper.clone()]
        };
        for (k, editor) in editors.iter().enumerate() {
            let focused = editor.read(cx).focus_handle(cx).is_focused(window);
            let was = inputs.read(cx).was_focused[k];
            if was && !focused {
                commit_input(&state, editor, k == 0 && range, range, window, cx);
            }
            if was != focused {
                inputs.update(cx, |i, _| i.was_focused[k] = focused);
            }
            let text = format_value(values[k]);
            if !focused && editor.read(cx).value() != text.as_str() {
                editor.update(cx, |e, cx| e.set_value(text, window, cx));
            }
        }

        let any_focused = thumb_focus.iter().any(|f| f.is_focused(window));
        let (rail, fill_target) = track_colors(any_focused, self.disabled);
        let fill =
            motion::animate_color((id.clone(), "fill"), fill_target, THUMB_MOTION, window, cx);
        let (from, to) = if range {
            (pct.start, pct.end)
        } else {
            (0., pct.end)
        };

        let thumbs: Vec<_> = (0..if range { 2 } else { 1 })
            .map(|k| {
                // A single slider's thumb is the "end" thumb.
                let start = range && k == 0;
                let at = if start { pct.start } else { pct.end };
                let focus = thumb_focus[k].clone();
                let pointer = pointers[k].clone();
                let look = ThumbLook::of(
                    self.level,
                    ThumbState {
                        hovered: pointer.read(cx).hovered,
                        pressed: pointer.read(cx).pressed,
                        focused: focus.is_focused(window),
                        disabled: self.disabled,
                    },
                );
                let size = motion::animate(
                    ((id.clone(), format!("thumb-{k}")), "size"),
                    look.size,
                    THUMB_MOTION.spatial(true),
                    window,
                    cx,
                );
                let thumb_fill = motion::animate_color(
                    ((id.clone(), format!("thumb-{k}")), "fill"),
                    look.fill,
                    THUMB_MOTION,
                    window,
                    cx,
                );
                let key_state = state.clone();
                let handle = HandleLook::of(ThumbState {
                    hovered: pointer.read(cx).hovered,
                    pressed: pointer.read(cx).pressed,
                    focused: focus.is_focused(window),
                    disabled: self.disabled,
                });
                let upper = range && !start;
                let background = color(ColorRole::Background);
                let press_focus = focus.clone();
                let press_pointer = pointer.clone();
                gpui_kit::base::SliderThumb::new(&state)
                    .start(start)
                    .disabled(inert)
                    .track_focus(&focus.clone().tab_stop(!self.disabled))
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(relative(at))
                    .map(|this| {
                        if range {
                            this.ml(rem(handle_offset(upper))).w(rem(HANDLE.0))
                        } else {
                            this.ml(rem(-THUMB_ACTIVE / 2.)).w(rem(THUMB_ACTIVE))
                        }
                    })
                    // Kit's thumb stops the press from reaching the track, so it
                    // takes focus here, in the capture phase: a drag that starts
                    // on a thumb leaves it focused (`:focus`) for the arrow keys.
                    .when(!inert, |this| {
                        this.capture_any_mouse_down(move |event, window, cx| {
                            if event.button != MouseButton::Left {
                                return;
                            }
                            press_focus.focus(window, cx);
                            press_pointer.update(cx, |p, cx| {
                                if !p.pressed {
                                    p.pressed = true;
                                    cx.notify();
                                }
                            });
                        })
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor(cursor)
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        if readonly {
                            return;
                        }
                        let sign = match event.keystroke.key.as_str() {
                            "left" | "down" => -1.,
                            "right" | "up" => 1.,
                            _ => return,
                        };
                        let factor = if event.keystroke.modifiers.shift {
                            STEP_MULTIPLIER
                        } else {
                            1.
                        };
                        key_state.update(cx, |s, cx| {
                            let delta = sign * s.step_value() * factor;
                            s.step_thumb(start, delta, window, cx)
                        });
                        cx.stop_propagation();
                    })
                    .when(range && !readonly, |this| {
                        this.child(handle_glyph(upper, handle, background))
                    })
                    .when(!range && !readonly, |this| {
                        this.child(
                            div()
                                .size(rem(size))
                                .rounded_full()
                                .bg(thumb_fill)
                                .flex()
                                .items_center()
                                .justify_center()
                                .when_some(look.inner_ring, |this, ring| {
                                    this.child(
                                        div()
                                            .size(rem(size - 4.))
                                            .rounded_full()
                                            .border_1()
                                            .border_color(ring),
                                    )
                                }),
                        )
                    })
            })
            .collect();

        // `.cds--slider { padding: 1rem 0; min-inline-size: 12.5rem; max-inline-size: 40rem }`.
        let track = gpui_kit::base::SliderTrack::new(&state)
            .disabled(inert)
            .relative()
            .flex_1()
            .min_w(rem(200.))
            .max_w(rem(640.))
            .h(rem(32.))
            .cursor(cursor)
            .child(
                gpui_kit::base::SliderIndicator::new(&state)
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(rem(15.))
                    .h(rem(2.))
                    .bg(rail)
                    // `.cds--slider__track:before`: a 2×4px tick at 50%, 5px above.
                    .child(
                        div()
                            .absolute()
                            .left(relative(0.5))
                            .ml(px(-1.))
                            .top(rem(-5.))
                            .w(px(2.))
                            .h(rem(4.))
                            .bg(rail),
                    )
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(relative(from))
                            .w(relative((to - from).max(0.)))
                            .bg(fill),
                    ),
            )
            .children(thumbs);
        // Kit's track lays a hit area over the thumbs, so pointer hover and press
        // are worked out here from the position: the thumb under the pointer
        // (within its 20px hover size) is hovered; a press focuses the nearest one.
        let thumb_at = {
            let state = state.clone();
            move |x: gpui_kit::Pixels, cx: &App| -> usize {
                let s = state.read(cx);
                let b = s.bounds();
                let w = f32::from(b.size.width).max(1.);
                let p = (f32::from(x) - f32::from(b.origin.x)) / w;
                let pct = s.percentage();
                if range && (p - pct.start).abs() <= (p - pct.end).abs() {
                    0
                } else {
                    range as usize
                }
            }
        };
        let near_thumb = {
            let state = state.clone();
            move |position: gpui_kit::Point<gpui_kit::Pixels>, k: usize, cx: &App| -> bool {
                let s = state.read(cx);
                let b = s.bounds();
                let pct = s.percentage();
                let at = if range && k == 0 { pct.start } else { pct.end };
                let x = f32::from(b.origin.x) + at * f32::from(b.size.width);
                let cy_px = f32::from(b.origin.y) + f32::from(b.size.height) / 2.;
                let px_x = f32::from(position.x);
                let dy = (f32::from(position.y) - cy_px).abs();
                if range {
                    // The 16×24 handle box beside its value point.
                    let left = x + handle_offset(k == 1);
                    (left..=left + HANDLE.0).contains(&px_x) && dy <= HANDLE.1 / 2.
                } else {
                    let r = THUMB_ACTIVE / 2.;
                    (px_x - x).abs() <= r && dy <= r
                }
            }
        };
        let set_pointer = {
            let pointers = pointers.clone();
            move |f: &dyn Fn(usize, &mut ThumbPointer), cx: &mut App| {
                for (k, pointer) in pointers.iter().enumerate() {
                    pointer.update(cx, |p, cx| {
                        let before = (p.hovered, p.pressed);
                        f(k, p);
                        if before != (p.hovered, p.pressed) {
                            cx.notify();
                        }
                    });
                }
            }
        };
        let set_pointer = std::rc::Rc::new(set_pointer);
        let track = track.when(!inert, |this| {
            let set_move = set_pointer.clone();
            let set_down = set_pointer.clone();
            let set_up = set_pointer.clone();
            let set_up_out = set_pointer.clone();
            let thumb_focus = thumb_focus.clone();
            let count = if range { 2 } else { 1 };
            this.on_mouse_move(move |event, _, cx| {
                let near: Vec<bool> = (0..count)
                    .map(|k| near_thumb(event.position, k, cx))
                    .collect();
                set_move(
                    &|k, p| p.hovered = near.get(k).copied().unwrap_or(false),
                    cx,
                );
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let k = thumb_at(event.position.x, cx);
                thumb_focus[k].focus(window, cx);
                set_down(&|i, p| p.pressed = i == k, cx);
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                set_up(&|_, p| p.pressed = false, cx)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                set_up_out(&|_, p| p.pressed = false, cx)
            })
        });
        let track = div()
            .id((id.clone(), "track"))
            .flex_1()
            .min_w(rem(200.))
            .flex()
            .on_hover({
                let set_pointer = set_pointer.clone();
                move |over, _, cx| {
                    if !over {
                        set_pointer(&|_, p| p.hovered = false, cx)
                    }
                }
            })
            .child(track);

        // The slider is the row's only flexible item and holds the
        // `.cds--slider` inline-size bounds, so in a narrow row it shrinks
        // to 12.5rem and then the row overflows at its end rather than the
        // track running under the range labels.
        // `.cds--slider-container--two-handles .cds--slider { margin-inline: 0.25rem }`.
        let slider = gpui_kit::base::Slider::new(&state)
            .disabled(inert)
            .when_some(self.label.clone(), |this, label| {
                this.accessibility_label(label)
            })
            .flex_1()
            .min_w(rem(200.))
            .max_w(rem(640.))
            .when(range, |this| this.mx(rem(4.)))
            .flex()
            .child(track);

        let field_height = FieldSize::Md.height(density);
        let input = |k: usize| {
            let editor = editors[k].clone();
            let commit_state = state.clone();
            let commit_editor = editor.clone();
            let look = FieldLook::of(
                self.level,
                FieldState {
                    disabled: self.disabled,
                    readonly: self.readonly,
                    ..Default::default()
                },
            );
            let focused = editor.read(cx).focus_handle(cx).is_focused(window);
            let is_start = range && k == 0;
            div()
                .id((id.clone(), format!("input-{k}")))
                .relative()
                .flex_shrink_0()
                .flex()
                .items_center()
                // `.cds--slider-text-input { inline-size: 4rem; block-size: 2.5rem }`.
                .w(rem(64.))
                .h(rem(field_height))
                .px(rem(16.))
                .bg(look.background)
                .pb(px(form::FIELD_RULE))
                .capture_action(move |action: &Enter, window, cx| {
                    if !action.secondary {
                        commit_input(&commit_state, &commit_editor, is_start, range, window, cx);
                    }
                })
                .child(
                    KitInput::new(&editor)
                        .appearance(false)
                        .with_size(Size::XSmall)
                        .disabled(self.disabled)
                        .readonly(self.readonly)
                        .p_0()
                        .h_full()
                        .w_full()
                        .text_size(rem(14.))
                        .line_height(rem(18.))
                        .text_color(look.text),
                )
                .child(form::bottom_rule(if self.disabled {
                    Hsla::transparent_black()
                } else {
                    look.border_bottom
                }))
                .when(focused, |this| {
                    this.child(form::inset_outline(2., color(ColorRole::Focus)))
                })
        };
        let range_label = |value: f32| {
            div()
                .flex_shrink_0()
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(if self.disabled {
                    ColorRole::TextDisabled
                } else {
                    ColorRole::TextPrimary
                }))
                .whitespace_nowrap()
                .child(SharedString::from(format_value(value)))
        };

        let container = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(tokens::SPACING_05)
            .w_full()
            .when(range && !self.hide_input, |this| this.child(input(0)))
            .child(range_label(min))
            .child(slider)
            .child(range_label(max))
            .when(!self.hide_input, |this| {
                this.child(input(if range { 1 } else { 0 }))
            });
        form::labeled(self.label.as_ref(), self.disabled, container, None)
    }
}

/// A range handle glyph (16×24), centered in its thumb's hit area.
fn handle_glyph(upper: bool, look: HandleLook, background: Hsla) -> impl IntoElement {
    gpui_kit::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let scale = f32::from(bounds.size.width) / HANDLE.0;
            let at = |(x, y): (f32, f32)| {
                gpui_kit::point(
                    bounds.origin.x + px(x * scale),
                    bounds.origin.y + px(y * scale),
                )
            };
            if look.focus_glyph {
                // `:focus::before { inline-size: 100%; block-size: 0.125rem; background:
                // var(--background); inset-block-start: calc(50% - 0.0625rem) }`.
                window.paint_quad(gpui_kit::fill(
                    gpui_kit::Bounds::new(
                        at((0., HANDLE.1 / 2. - 1.)),
                        gpui_kit::size(px(HANDLE.0 * scale), px(2. * scale)),
                    ),
                    background,
                ));
            }
            let mut path = gpui_kit::PathBuilder::fill();
            for contour in handle_contours(upper, look.focus_glyph) {
                let mut points = contour.into_iter();
                if let Some(first) = points.next() {
                    path.move_to(at(first));
                    for p in points {
                        path.line_to(at(p));
                    }
                    path.close();
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, look.fill);
            }
        },
    )
    .w(rem(HANDLE.0))
    .h(rem(HANDLE.1))
}

/// Parse an input's text and move the matching thumb.
fn commit_input(
    state: &Entity<SliderState>,
    editor: &Entity<InputState>,
    start: bool,
    range: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let text = editor.read(cx).value();
    state.update(cx, |s, cx| {
        let Some(v) = parse_value(&text, s.min_value(), s.max_value()) else {
            return;
        };
        let next = match (range, s.value()) {
            (true, SliderValue::Range(a, b)) => {
                if start {
                    SliderValue::Range(v.min(b), b)
                } else {
                    SliderValue::Range(a, v.max(a))
                }
            }
            _ => SliderValue::Single(v),
        };
        if next != s.value() {
            s.set_value(next, window, cx);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn read_only_takes_the_arrow_and_disabled_wins() {
        assert_eq!(pointer_cursor(false, true), CursorStyle::Arrow);
        assert_eq!(pointer_cursor(true, true), CursorStyle::OperationNotAllowed);
        assert_eq!(pointer_cursor(false, false), CursorStyle::PointingHand);
    }

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn thumb_follows_slider_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = ThumbLook::of(LayerLevel::One, ThumbState::default());
        assert_eq!((rest.size, hex(rest.fill)), (14., "#161616".into()));
        let hover = ThumbLook::of(
            LayerLevel::One,
            ThumbState {
                hovered: true,
                ..Default::default()
            },
        );
        // `transform: scale(1.4286)` of 14px.
        assert!((hover.size - 14. * 1.4286).abs() < 0.01);
        let focus = ThumbLook::of(
            LayerLevel::One,
            ThumbState {
                focused: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(focus.fill), "#0f62fe");
        assert_eq!(focus.inner_ring.map(hex).as_deref(), Some("#f4f4f4"));
        let disabled = ThumbLook::of(
            LayerLevel::One,
            ThumbState {
                disabled: true,
                hovered: true,
                ..Default::default()
            },
        );
        assert_eq!((disabled.size, hex(disabled.fill)), (14., "#c6c6c6".into()));
        let (rail, fill) = track_colors(false, false);
        assert_eq!((hex(rail), hex(fill)), ("#c6c6c6".into(), "#161616".into()));
        assert_eq!(hex(track_colors(true, false).1), "#0f62fe");
    }

    #[test]
    fn range_handles_follow_thumb_wrapper_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = HandleLook::of(ThumbState::default());
        assert_eq!(
            (rest.focus_glyph, hex(rest.fill)),
            (false, "#161616".into())
        );
        let hover = HandleLook::of(ThumbState {
            hovered: true,
            ..Default::default()
        });
        assert_eq!(hex(hover.fill), "#525252");
        let focus = HandleLook::of(ThumbState {
            focused: true,
            hovered: true,
            ..Default::default()
        });
        assert_eq!(
            (focus.focus_glyph, hex(focus.fill)),
            (true, "#0f62fe".into())
        );
        let disabled = HandleLook::of(ThumbState {
            disabled: true,
            focused: true,
            ..Default::default()
        });
        assert_eq!(
            (disabled.focus_glyph, hex(disabled.fill)),
            (false, "#c6c6c6".into())
        );
        // Lower sits left of its value, upper right of it.
        assert_eq!(handle_offset(false), -16.);
        assert_eq!(handle_offset(true), 0.);
    }

    #[test]
    fn handle_glyphs_mirror_and_hollow_on_focus() {
        let lower = handle_contours(false, false);
        let upper = handle_contours(true, false);
        assert_eq!(lower.len(), 2);
        assert_eq!(handle_contours(false, true).len(), 3);
        // The bar is on the inner edge: right for lower, left for upper.
        assert!(lower[0].iter().all(|p| p.0 >= 15.));
        assert!(upper[0].iter().all(|p| p.0 <= 1.));
        for (a, b) in lower.iter().flatten().zip(upper.iter().flatten()) {
            assert!((a.0 + b.0 - 16.).abs() < 1e-4);
            assert_eq!(a.1, b.1);
        }
        // Everything fits the 16×24 box.
        for (x, y) in handle_contours(true, true).into_iter().flatten() {
            assert!((0. ..=16.).contains(&x) && (0. ..=24.).contains(&y));
        }
    }

    #[test]
    fn values_format_and_parse() {
        assert_eq!(format_value(24.), "24");
        assert_eq!(format_value(0.5), "0.5");
        assert_eq!(parse_value(" 80 ", 0., 64.), Some(64.));
        assert_eq!(parse_value("x", 0., 64.), None);
    }
}
