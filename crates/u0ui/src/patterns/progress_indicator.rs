//! `ProgressIndicator` / `.cds--progress`: Carbon's step indicator, the
//! wizard header of progress dialogs.
//!
//! - **Anatomy**: per step a 2px line along the top, a 16px status icon
//!   (complete `CheckmarkOutline`, current `Incomplete`, incomplete
//!   `CircleDash`, invalid `Warning`) and the label, with an optional
//!   secondary label under it.
//! - **Behavior**: with [`ProgressIndicator::on_change`] every step but the
//!   current one is a button (Kit base button: focus, Enter / Space);
//!   without it the steps are static (`.cds--progress-step-button--unclickable`).
//!
//! ```ignore
//! ProgressIndicator::new("wizard")
//!     .current_index(1)
//!     .space_equally(true)
//!     .step(ProgressStep::new("Folder"))
//!     .step(ProgressStep::new("Agents"))
//!     .step(ProgressStep::new("Review"))
//! ```
use crate::{
    components::form::{self, rem},
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// Where a step stands relative to the current index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepStatus {
    /// Before the current step (`.cds--progress-step--complete`).
    Complete,
    /// The current step (`.cds--progress-step--current`).
    Current,
    /// After it (`.cds--progress-step--incomplete`).
    Incomplete,
}

impl StepStatus {
    /// The status of step `index` when `current` is the current index
    /// (Carbon: `index < currentIndex` complete, `===` current, else incomplete).
    pub fn of(index: usize, current: usize) -> Self {
        match index.cmp(&current) {
            std::cmp::Ordering::Less => Self::Complete,
            std::cmp::Ordering::Equal => Self::Current,
            std::cmp::Ordering::Greater => Self::Incomplete,
        }
    }
}

/// The colors and icon of one step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepLook {
    /// The status icon.
    pub icon: CarbonIcon,
    /// `.cds--progress-step svg { fill: var(--interactive) }`; incomplete
    /// `icon-primary`; disabled `icon-disabled`; invalid `support-error`.
    pub icon_color: Hsla,
    /// `.cds--progress-step--current/--complete .cds--progress-line {
    /// background-color: var(--interactive) }`; incomplete `border-subtle`.
    pub line: Hsla,
    /// `.cds--progress-label { color: var(--text-primary) }`; disabled
    /// `text-disabled`; hover on a clickable step `link-primary-hover`.
    pub label: Hsla,
    /// `.cds--progress-label:hover { color: var(--link-primary-hover) }`.
    pub label_hover: Hsla,
}

impl StepLook {
    /// The look of a step.
    pub fn of(status: StepStatus, disabled: bool, invalid: bool) -> Self {
        let icon = if invalid {
            CarbonIcon::Warning
        } else {
            match status {
                StepStatus::Complete => CarbonIcon::CheckmarkOutline,
                StepStatus::Current => CarbonIcon::Incomplete,
                StepStatus::Incomplete => CarbonIcon::CircleDash,
            }
        };
        let icon_color = color(if disabled {
            ColorRole::IconDisabled
        } else if invalid {
            ColorRole::SupportError
        } else if status == StepStatus::Incomplete {
            ColorRole::IconPrimary
        } else {
            ColorRole::Interactive
        });
        let line = color(match status {
            StepStatus::Incomplete => ColorRole::BorderSubtle01,
            _ => ColorRole::Interactive,
        });
        let label = color(if disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextPrimary
        });
        Self {
            icon,
            icon_color,
            line,
            label,
            label_hover: color(ColorRole::LinkPrimaryHover),
        }
    }
}

/// The indicator's measurements, in px at 100% zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressGeometry {
    /// `.cds--progress-step { inline-size: 8rem }`.
    pub step_width: f32,
    /// `.cds--progress-step { min-inline-size: 7rem }`.
    pub step_min: f32,
    /// `.cds--progress--space-equal .cds--progress-step { min-inline-size: 8rem }`.
    pub step_min_equal: f32,
    /// `.cds--progress-line { block-size: 2px }`.
    pub line: f32,
    /// `.cds--progress-step svg { block-size: 1rem; margin-block-start: 0.625rem;
    /// margin-inline-end: 0.5rem }`: side.
    pub icon: f32,
    /// … top margin.
    pub icon_top: f32,
    /// … end margin.
    pub icon_gap: f32,
    /// `.cds--progress-label { margin: 0.5rem 0 0 0 }`.
    pub label_top: f32,
    /// `.cds--progress-label { line-height: 1.45 }` at 14px.
    pub label_line: f32,
    /// `.cds--progress-label { max-inline-size: 5.5rem }`.
    pub label_max: f32,
    /// `.cds--progress--space-equal .cds--progress-label { margin-inline-end: 0.75rem }`.
    pub label_end: f32,
}

impl ProgressGeometry {
    /// The reference geometry.
    pub const REFERENCE: Self = Self {
        step_width: 128.,
        step_min: 112.,
        step_min_equal: 128.,
        line: 2.,
        icon: 16.,
        icon_top: 10.,
        icon_gap: 8.,
        label_top: 8.,
        label_line: 14. * 1.45,
        label_max: 88.,
        label_end: 12.,
    };
}

/// One step (`ProgressStep`).
#[derive(Clone)]
pub struct ProgressStep {
    label: SharedString,
    secondary_label: Option<SharedString>,
    disabled: bool,
    invalid: bool,
}

impl ProgressStep {
    /// A step reading `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            secondary_label: None,
            disabled: false,
            invalid: false,
        }
    }

    /// `secondaryLabel` (`.cds--progress-optional`: `label-01`, `text-secondary`).
    pub fn secondary_label(mut self, text: impl Into<SharedString>) -> Self {
        self.secondary_label = Some(text.into());
        self
    }

    /// `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `invalid`: the warning icon.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A Carbon progress indicator. See the module docs.
#[derive(IntoElement)]
pub struct ProgressIndicator {
    id: ElementId,
    current: usize,
    space_equally: bool,
    steps: Vec<ProgressStep>,
    on_change: Option<ChangeHandler>,
}

impl ProgressIndicator {
    /// An indicator with a stable `id`, the first step current.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            current: 0,
            space_equally: false,
            steps: Vec::new(),
            on_change: None,
        }
    }

    /// `currentIndex`.
    pub fn current_index(mut self, index: usize) -> Self {
        self.current = index;
        self
    }

    /// `spaceEqually`: steps share the width (`flex-grow: 1`).
    pub fn space_equally(mut self, equal: bool) -> Self {
        self.space_equally = equal;
        self
    }

    /// Append a step.
    pub fn step(mut self, step: ProgressStep) -> Self {
        self.steps.push(step);
        self
    }

    /// `onChange`: makes the other steps clickable; called with the index.
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// Whether step `index` is a button: there is a handler, it is not the
/// current step and not disabled.
pub fn clickable(has_handler: bool, index: usize, current: usize, disabled: bool) -> bool {
    has_handler && index != current && !disabled
}

impl RenderOnce for ProgressIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = ProgressGeometry::REFERENCE;
        let id = self.id;
        let equal = self.space_equally;
        let current = self.current;
        let steps = self.steps.into_iter().enumerate().map(|(i, step)| {
            let status = StepStatus::of(i, current);
            let look = StepLook::of(status, step.disabled, step.invalid);
            let can_click = clickable(self.on_change.is_some(), i, current, step.disabled);
            let focus = window
                .use_keyed_state((id.clone(), format!("focus{i}")), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone();
            let focused =
                can_click && focus.is_focused(window) && crate::focus::focus_visible_mode(window);
            let hovered_id: ElementId = (id.clone(), format!("label{i}")).into();
            let label = div()
                .id(hovered_id)
                .relative()
                .mt(rem(g.label_top))
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .line_height(rem(g.label_line))
                .text_color(if focused {
                    color(ColorRole::Focus)
                } else {
                    look.label
                })
                .map(|d| {
                    if equal {
                        d.mr(rem(g.label_end))
                    } else {
                        d.max_w(rem(g.label_max))
                    }
                })
                // `:hover { box-shadow: 0 0.0625rem var(--link-primary-hover); color: … }`.
                .when(can_click, |d| {
                    let hover = look.label_hover;
                    d.hover(move |s| s.text_color(hover).underline())
                })
                // `:focus-visible .cds--progress-label { outline: 0.0625rem solid var(--focus) }`.
                .when(focused, |d| {
                    d.child(form::inset_outline(1., color(ColorRole::Focus)))
                })
                .child(step.label);
            let text = div()
                .flex()
                .flex_col()
                .min_w_0()
                .flex_1()
                .child(label)
                .when_some(step.secondary_label, |d, text| {
                    d.child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(text),
                    )
                });
            let on_change = self.on_change.clone();
            let body = gpui_kit::base::Button::new((id.clone(), format!("step{i}")))
                .track_focus(&focus)
                .disabled(!can_click)
                .flex()
                .flex_row()
                .w_full()
                .min_w_0()
                .cursor(if step.disabled {
                    CursorStyle::OperationNotAllowed
                } else if can_click {
                    CursorStyle::PointingHand
                } else {
                    CursorStyle::Arrow
                })
                .child(
                    div()
                        .flex_none()
                        .mt(rem(g.icon_top))
                        .mr(rem(g.icon_gap))
                        .child(
                            Icon::carbon(look.icon)
                                .size(IconSize::S16)
                                .color(look.icon_color),
                        ),
                )
                .child(text)
                .on_click(move |_, window, cx| {
                    if let Some(f) = &on_change {
                        f(i, window, cx)
                    }
                });
            div()
                .relative()
                .flex()
                .flex_row()
                .min_w_0()
                .map(|d| {
                    if equal {
                        d.flex_1().min_w(rem(g.step_min_equal))
                    } else {
                        d.w(rem(g.step_width)).min_w(rem(g.step_min))
                    }
                })
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .h(rem(g.line))
                        .bg(look.line),
                )
                .child(body)
        });
        div()
            .id(id.clone())
            .flex()
            .flex_row()
            .w_full()
            .type_style(TypeStyle::BodyCompact01)
            .children(steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_follows_the_current_index() {
        assert_eq!(StepStatus::of(0, 1), StepStatus::Complete);
        assert_eq!(StepStatus::of(1, 1), StepStatus::Current);
        assert_eq!(StepStatus::of(2, 1), StepStatus::Incomplete);
    }

    #[test]
    fn look_matches_the_css() {
        let done = StepLook::of(StepStatus::Complete, false, false);
        assert_eq!(done.icon, CarbonIcon::CheckmarkOutline);
        assert_eq!(done.icon_color, color(ColorRole::Interactive));
        assert_eq!(done.line, color(ColorRole::Interactive));
        let now = StepLook::of(StepStatus::Current, false, false);
        assert_eq!(now.icon, CarbonIcon::Incomplete);
        let next = StepLook::of(StepStatus::Incomplete, false, false);
        assert_eq!(next.icon, CarbonIcon::CircleDash);
        assert_eq!(next.icon_color, color(ColorRole::IconPrimary));
        assert_eq!(next.line, color(ColorRole::BorderSubtle01));
        let off = StepLook::of(StepStatus::Incomplete, true, false);
        assert_eq!(off.icon_color, color(ColorRole::IconDisabled));
        assert_eq!(off.label, color(ColorRole::TextDisabled));
        let bad = StepLook::of(StepStatus::Complete, false, true);
        assert_eq!(bad.icon, CarbonIcon::Warning);
        assert_eq!(bad.icon_color, color(ColorRole::SupportError));
    }

    #[test]
    fn geometry_matches_the_css() {
        let g = ProgressGeometry::REFERENCE;
        assert_eq!((g.step_width, g.line, g.icon), (128., 2., 16.));
        assert_eq!((g.icon_top, g.icon_gap, g.label_top), (10., 8., 8.));
        assert_eq!((g.label_max, g.label_end), (88., 12.));
    }

    #[test]
    fn only_other_enabled_steps_click() {
        assert!(clickable(true, 0, 1, false));
        assert!(!clickable(true, 1, 1, false));
        assert!(!clickable(true, 2, 1, true));
        assert!(!clickable(false, 0, 1, false));
    }
}
