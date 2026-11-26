//! Progress: shadcn `Progress` → Carbon `ProgressBar` (`.cds--progress-bar*`).
//!
//! - **Appearance**: a `body-compact-01` label row (`text-primary`, 8px
//!   above the track, ellipsized, with the status icon 1rem after it), an
//!   8px track (`small`: 4px) in `border-subtle` with an `interactive` bar
//!   scaled to `value / max`, and `helper-text-01` in `text-secondary` 8px
//!   under it. `finished` fills the bar in `support-success` with
//!   `CheckmarkFilled`; `error` fills it in `support-error` with `ErrorFilled`
//!   and turns the helper text `text-error`.
//! - **Types**: `inline` puts the label before the track on one row (helper
//!   text visually hidden; finished/error hide the track); `indented` pads
//!   label and helper text by 1rem.
//! - **Motion**: the bar's `transform: scaleX()` eases in 110ms productive
//!   standard. Without a `value` the bar is indeterminate: a quarter-width
//!   `interactive` segment slides left → right in 1120ms, linear, then waits
//!   280ms (`cds--progress-bar-indeterminate` 1400ms: `background-position-x`
//!   25% → −105% at 80%) ([`indeterminate_segment`]). Reduced motion: Carbon
//!   keeps both; u0ui keeps the value transition and replaces the slide with
//!   a pulsing full-width bar (movement becomes a fade).
//! - **Behavior**: none. Kit's `Progress` is a rounded bar in its own colors.
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::FluentBuilder as _, relative, rems,
};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// `.cds--progress-bar__bar { transition: transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const BAR: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// `.cds--progress-bar--indeterminate .cds--progress-bar__track::after { animation-duration: 1400ms }`.
pub const INDETERMINATE_PERIOD: Duration = Duration::from_millis(1400);

/// `index.d.ts` `ProgressProps.status`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProgressStatus {
    /// In progress (default).
    #[default]
    Active,
    /// Done: full `support-success` bar and `CheckmarkFilled`.
    Finished,
    /// Failed: full `support-error` bar, `ErrorFilled`, `text-error` helper.
    Error,
}

impl ProgressStatus {
    /// Every status.
    pub const ALL: [ProgressStatus; 3] = [Self::Active, Self::Finished, Self::Error];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Finished => "finished",
            Self::Error => "error",
        }
    }
}

/// `index.d.ts` `ProgressProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProgressSize {
    /// 8px track (default).
    #[default]
    Big,
    /// 4px track.
    Small,
}

impl ProgressSize {
    /// Both sizes.
    pub const ALL: [ProgressSize; 2] = [Self::Big, Self::Small];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Big => "big",
            Self::Small => "small",
        }
    }

    /// Track height: `.cds--progress-bar--big .cds--progress-bar__track { block-size: 0.5rem }`,
    /// `--small` 0.25rem.
    pub const fn track(self) -> f32 {
        match self {
            Self::Big => 8.,
            Self::Small => 4.,
        }
    }
}

/// `index.d.ts` `ProgressProps.type`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProgressType {
    /// Label above, helper text below.
    #[default]
    Default,
    /// Label and track on one row.
    Inline,
    /// Label and helper text indented by 1rem.
    Indented,
}

impl ProgressType {
    /// Every type.
    pub const ALL: [ProgressType; 3] = [Self::Default, Self::Inline, Self::Indented];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Inline => "inline",
            Self::Indented => "indented",
        }
    }
}

/// The bar fraction: finished and error fill it (`transform: scaleX(1)`),
/// otherwise `value / max` clamped to 0..=1; `None` = indeterminate.
pub fn fraction(value: Option<f32>, max: f32, status: ProgressStatus) -> Option<f32> {
    if status != ProgressStatus::Active {
        return Some(1.);
    }
    let max = if max > 0. { max } else { 100. };
    value.map(|v| (v / max).clamp(0., 1.))
}

/// The bar color: `interactive`; `support-success` finished, `support-error` error.
pub fn bar_color(status: ProgressStatus) -> Hsla {
    color(match status {
        ProgressStatus::Active => ColorRole::Interactive,
        ProgressStatus::Finished => ColorRole::SupportSuccess,
        ProgressStatus::Error => ColorRole::SupportError,
    })
}

/// The indeterminate segment at `t` (0..1 of 1400ms) as (left, width)
/// fractions of the track: the `linear-gradient(90deg, interactive 12.5%,
/// transparent 12.5%)` image is 200% wide, so its solid part is 25% of the
/// track; `background-position-x: p` puts it at `−p × width`, and p goes
/// 25% → −105% over the first 80%, then holds.
pub fn indeterminate_segment(t: f32) -> (f32, f32) {
    let t = t.rem_euclid(1.);
    let p = if t < 0.8 {
        0.25 + (-1.05 - 0.25) * (t / 0.8)
    } else {
        -1.05
    };
    (-p, 0.25)
}

fn epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

/// A Carbon progress bar. See the module docs.
#[derive(IntoElement)]
pub struct Progress {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<f32>,
    max: f32,
    helper_text: Option<SharedString>,
    status: ProgressStatus,
    size: ProgressSize,
    kind: ProgressType,
    level: LayerLevel,
}

impl Progress {
    /// An indeterminate progress bar with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            max: 100.,
            helper_text: None,
            status: ProgressStatus::Active,
            size: ProgressSize::Big,
            kind: ProgressType::Default,
            level: LayerLevel::One,
        }
    }

    /// The label (`hideLabel` when absent, as in the reference).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The value; leave unset for an indeterminate bar.
    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }

    /// Optional value (`None` = indeterminate).
    pub fn value_opt(mut self, value: Option<f32>) -> Self {
        self.value = value;
        self
    }

    /// The maximum (default 100).
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Helper text under the track.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// The status.
    pub fn status(mut self, status: ProgressStatus) -> Self {
        self.status = status;
        self
    }

    /// `big` or `small`.
    pub fn size(mut self, size: ProgressSize) -> Self {
        self.size = size;
        self
    }

    /// `default`, `inline` or `indented`.
    pub fn kind(mut self, kind: ProgressType) -> Self {
        self.kind = kind;
        self
    }

    /// The layer the track sits on (`border-subtle` is contextual).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for Progress {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fraction = fraction(self.value, self.max, self.status);
        let bar = bar_color(self.status);
        let reduced = motion::reduced_motion(cx);
        // `.cds--progress-bar__track { background-color: var(--border-subtle) }`
        // (`border-subtle-00` on the page background).
        let track_color = crate::components::separator::line_color(self.level);
        let inline = self.kind == ProgressType::Inline;
        let indent = if self.kind == ProgressType::Indented {
            16.
        } else {
            0.
        };
        let done = self.status != ProgressStatus::Active;

        let track_fill = match fraction {
            Some(f) => {
                let shown = motion::animate((self.id.clone(), "bar"), f, BAR, window, cx);
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .h_full()
                    .w(relative(shown))
                    .bg(bar)
            }
            None if reduced => {
                window.request_animation_frame();
                let t = epoch().elapsed().as_secs_f32() / INDETERMINATE_PERIOD.as_secs_f32();
                let pulse = 0.3 + 0.7 * (1. - ((t.fract() * 2.) - 1.).abs());
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .bg(bar)
                    .opacity(pulse)
            }
            None => {
                window.request_animation_frame();
                let t = epoch().elapsed().as_secs_f32() / INDETERMINATE_PERIOD.as_secs_f32();
                let (left, width) = indeterminate_segment(t);
                div()
                    .absolute()
                    .top_0()
                    .h_full()
                    .left(relative(left))
                    .w(relative(width))
                    .bg(bar)
            }
        };
        let track = div()
            .relative()
            .overflow_hidden()
            .h(rems(self.size.track() / crate::tokens::REM_PX))
            .min_w(rems(3.))
            .bg(track_color)
            .child(track_fill)
            .map(|this| if inline { this.flex_1() } else { this.w_full() });

        let status_icon = match self.status {
            ProgressStatus::Active => None,
            ProgressStatus::Finished => Some(CarbonIcon::CheckmarkFilled),
            ProgressStatus::Error => Some(CarbonIcon::ErrorFilled),
        }
        .map(|icon| {
            div()
                .flex_none()
                .ml(rems(1.))
                .child(Icon::carbon(icon).size(IconSize::S16).color(bar))
        });
        let label_row = self.label.clone().map(|label| {
            // `.cds--progress-bar__label { display: flex; justify-content: space-between;
            // margin-block-end: 0.5rem; min-inline-size: 3rem }`.
            div()
                .flex()
                .flex_row()
                .items_center()
                .min_w(rems(3.))
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .px(rems(indent / 16.))
                .map(|this| {
                    if inline {
                        // `--inline .cds--progress-bar__label { flex-shrink: 0; margin: 0 1rem 0 0 }`.
                        this.flex_none().mr(if done { rems(0.) } else { rems(1.) })
                    } else {
                        this.justify_between().mb(rems(0.5))
                    }
                })
                .child(
                    div()
                        .min_w_0()
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(label),
                )
                .children(status_icon)
        });
        let helper = (!inline)
            .then_some(self.helper_text.clone())
            .flatten()
            .map(|text| {
                div()
                    .mt(rems(0.5))
                    .px(rems(indent / 16.))
                    .type_style(TypeStyle::HelperText01)
                    .text_color(color(if self.status == ProgressStatus::Error {
                        ColorRole::TextError
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .child(text)
            });

        if inline {
            return div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .children(label_row)
                // `--finished.--inline / --error.--inline .cds--progress-bar__track`: visually hidden.
                .when(!done, |this| this.child(track));
        }
        div()
            .flex()
            .flex_col()
            .w_full()
            .children(label_row)
            .child(track)
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
    fn values_and_colors() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            fraction(Some(62.), 100., ProgressStatus::Active),
            Some(0.62)
        );
        assert_eq!(fraction(Some(150.), 100., ProgressStatus::Active), Some(1.));
        assert_eq!(fraction(None, 100., ProgressStatus::Active), None);
        // `.cds--progress-bar--finished .cds--progress-bar__bar { transform: scaleX(1) }`.
        assert_eq!(fraction(None, 100., ProgressStatus::Finished), Some(1.));
        assert_eq!(hex(bar_color(ProgressStatus::Active)), "#0f62fe");
        assert_eq!(hex(bar_color(ProgressStatus::Finished)), "#24a148");
        assert_eq!(hex(bar_color(ProgressStatus::Error)), "#da1e28");
        assert_eq!(
            (ProgressSize::Big.track(), ProgressSize::Small.track()),
            (8., 4.)
        );
        assert_eq!(BAR.speed.millis(), 110);
    }

    #[test]
    fn indeterminate_slides_then_waits() {
        // `background-position-x: 25%` → the segment just left of the track.
        let (l, w) = indeterminate_segment(0.);
        assert!((l + 0.25).abs() < 1e-5 && w == 0.25);
        // 80% → −105%: just right of the track, held until 100%.
        let (l, _) = indeterminate_segment(0.8);
        assert!((l - 1.05).abs() < 1e-5);
        assert_eq!(indeterminate_segment(0.9), indeterminate_segment(0.8));
        assert_eq!(INDETERMINATE_PERIOD.as_millis(), 1400);
    }
}
