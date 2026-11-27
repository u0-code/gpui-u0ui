//! Status indicators: Carbon's `IconIndicator` (`.cds--icon-indicator`),
//! `ShapeIndicator` (`.cds--shape-indicator`) and the patterns' difference
//! indicator (`.cu-pdiff`).
//!
//! - **IconIndicator**: a status icon (Material Symbols in place of
//!   Carbon's, as the reference does) in its `status-*` color, then the
//!   label in `text-secondary`, 0.5rem apart. 16px icon with
//!   `body-compact-01` text, or 20px with `body-compact-02`.
//! - **ShapeIndicator**: a 16px shape for severity before the label,
//!   `helper-text-01` (12px) or `body-compact-01` (14px).
//! - **Difference**: an up or down arrow in `support-success` /
//!   `support-error` before `text-primary` text, `spacing-02` apart; the
//!   caller's text carries the sign ("+12.5%", "−3.1%").
//!
//! Each uses at least two of color, shape and symbol, always with text.
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div};

/// `.cds--icon-indicator { gap: 0.5rem }`, `.cds--shape-indicator { gap: 0.5rem }`.
pub const GAP: tokens::TokenLength = tokens::SPACING_03;
/// `.cu-pdiff { gap: var(--spacing-02) }`.
pub const DIFF_GAP: tokens::TokenLength = tokens::SPACING_02;

/// `IconIndicator kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconIndicatorKind {
    /// `failed`.
    Failed,
    /// `caution-major`.
    CautionMajor,
    /// `caution-minor`.
    CautionMinor,
    /// `undefined`.
    Undefined,
    /// `succeeded`.
    Succeeded,
    /// `normal`.
    Normal,
    /// `in-progress`.
    InProgress,
    /// `incomplete`.
    Incomplete,
    /// `not-started`.
    NotStarted,
    /// `pending`.
    Pending,
    /// `unknown`.
    Unknown,
    /// `informative`.
    Informative,
}

impl IconIndicatorKind {
    /// Every kind, in Carbon's order.
    pub const ALL: [Self; 12] = [
        Self::Failed,
        Self::CautionMajor,
        Self::CautionMinor,
        Self::Undefined,
        Self::Succeeded,
        Self::Normal,
        Self::InProgress,
        Self::Incomplete,
        Self::NotStarted,
        Self::Pending,
        Self::Unknown,
        Self::Informative,
    ];

    /// Carbon's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::CautionMajor => "caution-major",
            Self::CautionMinor => "caution-minor",
            Self::Undefined => "undefined",
            Self::Succeeded => "succeeded",
            Self::Normal => "normal",
            Self::InProgress => "in-progress",
            Self::Incomplete => "incomplete",
            Self::NotStarted => "not-started",
            Self::Pending => "pending",
            Self::Unknown => "unknown",
            Self::Informative => "informative",
        }
    }

    /// Carbon's icon for the kind (drawn with its Material Symbol).
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Failed => CarbonIcon::ErrorFilled,
            Self::CautionMajor => CarbonIcon::WarningAltInvertedFilled,
            Self::CautionMinor => CarbonIcon::WarningAltFilled,
            Self::Undefined => CarbonIcon::UndefinedFilled,
            Self::Succeeded => CarbonIcon::CheckmarkFilled,
            Self::Normal => CarbonIcon::CheckmarkOutline,
            Self::InProgress => CarbonIcon::InProgress,
            Self::Incomplete => CarbonIcon::Incomplete,
            Self::NotStarted => CarbonIcon::CircleDash,
            Self::Pending => CarbonIcon::PendingFilled,
            Self::Unknown => CarbonIcon::UnknownFilled,
            Self::Informative => CarbonIcon::InformationSquareFilled,
        }
    }

    /// The icon's fill role (`.cds--icon-indicator--<kind> { fill: … }`).
    pub const fn role(self) -> ColorRole {
        match self {
            Self::Failed => ColorRole::StatusRed,
            Self::CautionMajor => ColorRole::StatusOrange,
            Self::CautionMinor => ColorRole::StatusYellow,
            Self::Undefined => ColorRole::StatusPurple,
            Self::Succeeded => ColorRole::StatusGreen,
            Self::Normal | Self::InProgress | Self::Incomplete | Self::Informative => {
                ColorRole::StatusBlue
            }
            Self::NotStarted | Self::Pending | Self::Unknown => ColorRole::StatusGray,
        }
    }
}

/// `IconIndicator size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IndicatorSize {
    /// 16px icon, `body-compact-01` (the default).
    #[default]
    S16,
    /// 20px icon, `body-compact-02` (`.cds--icon-indicator--20`).
    S20,
}

impl IndicatorSize {
    /// Both sizes.
    pub const ALL: [Self; 2] = [Self::S16, Self::S20];

    /// Carbon's value.
    pub const fn name(self) -> &'static str {
        match self {
            Self::S16 => "16",
            Self::S20 => "20",
        }
    }

    /// The icon size and the label's type style.
    pub const fn parts(self) -> (IconSize, TypeStyle) {
        match self {
            Self::S16 => (IconSize::S16, TypeStyle::BodyCompact01),
            Self::S20 => (IconSize::S20, TypeStyle::BodyCompact02),
        }
    }
}

fn row(
    icon: Icon,
    label: SharedString,
    style: TypeStyle,
    gap: tokens::TokenLength,
) -> gpui_kit::Div {
    div()
        // `display: flex; align-items: center`.
        .flex()
        .flex_row()
        .items_center()
        .gap(gap)
        .type_style(style)
        // `color: var(--text-secondary)`.
        .text_color(color(ColorRole::TextSecondary))
        .child(div().flex_none().flex().child(icon))
        .child(label)
}

/// An icon indicator. The caller passes the label.
#[derive(IntoElement)]
pub struct IconIndicator {
    kind: IconIndicatorKind,
    label: SharedString,
    size: IndicatorSize,
}

impl IconIndicator {
    /// `kind` with its `label`.
    pub fn new(kind: IconIndicatorKind, label: impl Into<SharedString>) -> Self {
        Self {
            kind,
            label: label.into(),
            size: IndicatorSize::S16,
        }
    }

    /// `size` (default 16).
    pub fn size(mut self, size: IndicatorSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for IconIndicator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (icon, style) = self.size.parts();
        row(
            Icon::carbon(self.kind.icon())
                .size(icon)
                .color(color(self.kind.role())),
            self.label,
            style,
            GAP,
        )
    }
}

/// `ShapeIndicator kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeIndicatorKind {
    /// `failed`.
    Failed,
    /// `critical`.
    Critical,
    /// `high`.
    High,
    /// `medium`.
    Medium,
    /// `low`.
    Low,
    /// `cautious`.
    Cautious,
    /// `undefined`.
    Undefined,
    /// `stable`.
    Stable,
    /// `informative`.
    Informative,
    /// `incomplete`.
    Incomplete,
    /// `draft`.
    Draft,
}

impl ShapeIndicatorKind {
    /// Every kind, in Carbon's order.
    pub const ALL: [Self; 11] = [
        Self::Failed,
        Self::Critical,
        Self::High,
        Self::Medium,
        Self::Low,
        Self::Cautious,
        Self::Undefined,
        Self::Stable,
        Self::Informative,
        Self::Incomplete,
        Self::Draft,
    ];

    /// Carbon's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Cautious => "cautious",
            Self::Undefined => "undefined",
            Self::Stable => "stable",
            Self::Informative => "informative",
            Self::Incomplete => "incomplete",
            Self::Draft => "draft",
        }
    }

    /// Carbon's shape for the kind: the bundle's `shapeTypes` map (`dR`:
    /// `cautious` shares `high`'s triangle, `informative` shares `low`'s
    /// glyph, `undefined` shares `medium`'s diamond). `incomplete` is
    /// Carbon's own two-path SVG, drawn by [`Self::icon_element`].
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Failed => CarbonIcon::Critical,
            Self::Critical => CarbonIcon::CriticalSeverity,
            Self::High | Self::Cautious => CarbonIcon::Caution,
            Self::Medium | Self::Undefined => CarbonIcon::DiamondFill,
            Self::Low | Self::Informative => CarbonIcon::LowSeverity,
            Self::Stable => CarbonIcon::CircleFill,
            Self::Incomplete => CarbonIcon::Incomplete,
            Self::Draft => CarbonIcon::CircleStroke,
        }
    }

    /// The shape's fill role. Each glyph is one path, so the
    /// `path:first-of-type` rules win where they exist: `.cds--shape-indicator
    /// --medium path:first-of-type { fill: status-orange-outline }` and
    /// `--cautious path:first-of-type { fill: status-yellow-outline }`; `low`'s
    /// `path:nth-of-type(2)` rule matches nothing, so it stays `status-yellow`.
    pub const fn role(self) -> ColorRole {
        match self {
            Self::Failed | Self::Critical | Self::High => ColorRole::StatusRed,
            Self::Medium => ColorRole::StatusOrangeOutline,
            Self::Low => ColorRole::StatusYellow,
            Self::Cautious => ColorRole::StatusYellowOutline,
            Self::Undefined => ColorRole::StatusPurple,
            Self::Stable => ColorRole::StatusGreen,
            // `.cds--shape-indicator--incomplete path:nth-of-type(2) { fill: status-blue }`.
            Self::Informative | Self::Incomplete => ColorRole::StatusBlue,
            Self::Draft => ColorRole::StatusGray,
        }
    }

    /// The 16px shape in its color.
    pub fn icon_element(self) -> Icon {
        let icon = match self {
            Self::Incomplete => Icon::from_path(INCOMPLETE_ASSET),
            _ => Icon::carbon(self.icon()),
        };
        icon.size(IconSize::S16).color(color(self.role()))
    }
}

/// The asset path of Carbon's `incomplete` shape, served by [`crate::Assets`].
pub const INCOMPLETE_ASSET: &str = "u0ui/shapes/incomplete.svg";

/// Carbon's `incomplete` shape (`uR` in the bundle): a 12px disc with the top
/// half of its inner 8px circle cut out. Its first path is a transparent
/// 16px square; the second, this one, is filled `status-blue`.
pub const INCOMPLETE_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\"><path fill=\"#161616\" d=\"M8 2a6 6 0 1 0 0 12A6 6 0 0 0 8 2Zm0 2a4.004 4.004 0 0 1 4 4H4a4.004 4.004 0 0 1 4-4Z\"/></svg>";

/// `ShapeIndicator textSize`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShapeTextSize {
    /// 12px, `helper-text-01` (the default).
    #[default]
    S12,
    /// 14px, `body-compact-01` (`.cds--shape-indicator--14`).
    S14,
}

impl ShapeTextSize {
    /// Both sizes.
    pub const ALL: [Self; 2] = [Self::S12, Self::S14];

    /// Carbon's value.
    pub const fn name(self) -> &'static str {
        match self {
            Self::S12 => "12",
            Self::S14 => "14",
        }
    }

    /// The label's type style.
    pub const fn style(self) -> TypeStyle {
        match self {
            Self::S12 => TypeStyle::HelperText01,
            Self::S14 => TypeStyle::BodyCompact01,
        }
    }
}

/// A shape indicator. The caller passes the label.
#[derive(IntoElement)]
pub struct ShapeIndicator {
    kind: ShapeIndicatorKind,
    label: SharedString,
    text_size: ShapeTextSize,
}

impl ShapeIndicator {
    /// `kind` with its `label`.
    pub fn new(kind: ShapeIndicatorKind, label: impl Into<SharedString>) -> Self {
        Self {
            kind,
            label: label.into(),
            text_size: ShapeTextSize::S12,
        }
    }

    /// `textSize` (default 12).
    pub fn text_size(mut self, size: ShapeTextSize) -> Self {
        self.text_size = size;
        self
    }
}

impl RenderOnce for ShapeIndicator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        row(
            self.kind.icon_element(),
            self.label,
            self.text_size.style(),
            GAP,
        )
    }
}

/// Which way a difference goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trend {
    /// Positive (`.cu-pdiff--up`).
    Up,
    /// Negative (`.cu-pdiff--down`).
    Down,
}

impl Trend {
    /// The arrow and its fill role.
    pub const fn parts(self) -> (CarbonIcon, ColorRole) {
        match self {
            // `.cu-pdiff--up svg { fill: var(--support-success) }`.
            Self::Up => (CarbonIcon::ArrowUp, ColorRole::SupportSuccess),
            // `.cu-pdiff--down svg { fill: var(--support-error) }`.
            Self::Down => (CarbonIcon::ArrowDown, ColorRole::SupportError),
        }
    }
}

/// A difference indicator: arrow + color + the caller's signed text.
#[derive(IntoElement)]
pub struct DiffIndicator {
    trend: Trend,
    text: SharedString,
}

impl DiffIndicator {
    /// `trend` with its `text` ("+12.5% threads this week").
    pub fn new(trend: Trend, text: impl Into<SharedString>) -> Self {
        Self {
            trend,
            text: text.into(),
        }
    }
}

impl RenderOnce for DiffIndicator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (icon, role) = self.trend.parts();
        // `.cu-pdiff--up, .cu-pdiff--down { color: var(--text-primary) }`.
        row(
            Icon::carbon(icon).size(IconSize::S16).color(color(role)),
            self.text,
            TypeStyle::BodyCompact01,
            DIFF_GAP,
        )
        .text_color(color(ColorRole::TextPrimary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_kinds_take_their_status_colors() {
        use IconIndicatorKind::*;
        assert_eq!(Failed.role(), ColorRole::StatusRed);
        assert_eq!(CautionMajor.role(), ColorRole::StatusOrange);
        assert_eq!(CautionMinor.role(), ColorRole::StatusYellow);
        assert_eq!(Undefined.role(), ColorRole::StatusPurple);
        assert_eq!(Succeeded.role(), ColorRole::StatusGreen);
        assert_eq!(InProgress.role(), ColorRole::StatusBlue);
        assert_eq!(NotStarted.role(), ColorRole::StatusGray);
        assert_eq!(NotStarted.icon(), CarbonIcon::CircleDash);
        assert_eq!(InProgress.icon(), CarbonIcon::InProgress);
        assert_eq!(IconIndicatorKind::ALL.len(), 12);
    }

    #[test]
    fn shape_kinds_take_their_status_colors() {
        use ShapeIndicatorKind::*;
        assert_eq!(Critical.role(), ColorRole::StatusRed);
        // `path:first-of-type` outline colors on single-path glyphs.
        assert_eq!(Medium.role(), ColorRole::StatusOrangeOutline);
        assert_eq!(Cautious.role(), ColorRole::StatusYellowOutline);
        assert_eq!(Low.role(), ColorRole::StatusYellow);
        assert_eq!(Undefined.role(), ColorRole::StatusPurple);
        assert_eq!(Incomplete.role(), ColorRole::StatusBlue);
        // The bundle's `shapeTypes`.
        assert_eq!(Cautious.icon(), High.icon());
        assert_eq!(Informative.icon(), Low.icon());
        assert_eq!(Undefined.icon(), Medium.icon());
        assert_eq!(
            Incomplete.icon_element().asset_path().as_ref(),
            INCOMPLETE_ASSET
        );
        assert!(INCOMPLETE_SVG.contains("H4a4.004"));
        assert_eq!(Stable.role(), ColorRole::StatusGreen);
        assert_eq!(Draft.role(), ColorRole::StatusGray);
        assert_eq!(Medium.icon(), CarbonIcon::DiamondFill);
    }

    #[test]
    fn sizes_follow_the_css() {
        assert_eq!(
            IndicatorSize::S20.parts(),
            (IconSize::S20, TypeStyle::BodyCompact02)
        );
        assert_eq!(ShapeTextSize::S12.style(), TypeStyle::HelperText01);
        assert_eq!(GAP.px_value(), 8.);
        assert_eq!(DIFF_GAP.px_value(), 4.);
        assert_eq!(Trend::Up.parts().1, ColorRole::SupportSuccess);
    }
}
