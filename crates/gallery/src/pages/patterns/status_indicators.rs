//! Carbon · Patterns › PatternStatusIndicators: icon indicators, shape
//! indicators for severity, count and dot badges, and difference
//! indicators (`components/PatternStatusIndicators/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::patterns::actionable_toast::inverse_theme;
use u0ui::patterns::indicator::{IndicatorSize, ShapeTextSize, Trend};
use u0ui::patterns::{
    BadgeIndicator, Badged, Caption, DiffIndicator, IconIndicator, IconIndicatorKind, PatternGrid,
    ShapeIndicator, ShapeIndicatorKind,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StatusPage {
        icon_size: 0,
        text_size: 1,
        count: 4,
        dot: true,
    })
    .into()
}

struct StatusPage {
    icon_size: usize,
    text_size: usize,
    count: u32,
    dot: bool,
}

/// The reference's labels per icon kind (`informative` is not in it).
const ICONS: [(IconIndicatorKind, &str); 11] = [
    (IconIndicatorKind::Failed, "Failed"),
    (IconIndicatorKind::CautionMajor, "Needs approval"),
    (IconIndicatorKind::CautionMinor, "Token expiring"),
    (IconIndicatorKind::Succeeded, "Merged"),
    (IconIndicatorKind::Normal, "Healthy"),
    (IconIndicatorKind::InProgress, "Running"),
    (IconIndicatorKind::Incomplete, "Partially synced"),
    (IconIndicatorKind::NotStarted, "Queued"),
    (IconIndicatorKind::Pending, "Pending review"),
    (IconIndicatorKind::Unknown, "Unknown"),
    (IconIndicatorKind::Undefined, "Not configured"),
];

const SHAPES: [(ShapeIndicatorKind, &str); 11] = [
    (ShapeIndicatorKind::Failed, "Failed"),
    (ShapeIndicatorKind::Critical, "Critical"),
    (ShapeIndicatorKind::High, "High"),
    (ShapeIndicatorKind::Medium, "Medium"),
    (ShapeIndicatorKind::Low, "Low"),
    (ShapeIndicatorKind::Cautious, "Cautious"),
    (ShapeIndicatorKind::Stable, "Stable"),
    (ShapeIndicatorKind::Informative, "Informative"),
    (ShapeIndicatorKind::Incomplete, "Incomplete"),
    (ShapeIndicatorKind::Draft, "Draft"),
    (ShapeIndicatorKind::Undefined, "Undefined"),
];

/// `display: grid; gap: 10`.
fn list(items: impl IntoIterator<Item = AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(rems(10. / 16.))
        .children(items)
}

fn icon_list(size: IndicatorSize) -> impl IntoElement {
    list(ICONS.iter().map(move |&(kind, label)| {
        IconIndicator::new(kind, label)
            .size(size)
            .into_any_element()
    }))
}

fn shape_list(size: ShapeTextSize) -> impl IntoElement {
    list(SHAPES.iter().map(move |&(kind, label)| {
        ShapeIndicator::new(kind, label)
            .text_size(size)
            .into_any_element()
    }))
}

/// A 48px header icon button with a count (`IconButton kind="ghost"
/// size="lg" badgeCount className="cu-pinv"`).
fn header_bell(id: SharedString, label: SharedString, count: u32) -> AnyElement {
    Badged::new(
        Button::new(id, label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Lg)
            .density(Density::Comfortable)
            .icon(CarbonIcon::Notification)
            .icon_only(true),
        (count > 0).then_some(BadgeIndicator::Count(count)),
    )
    .into_any_element()
}

/// The badges: two header bells on `background-inverse`, then a toolbar
/// button with the "new" dot beside a plain one.
fn badges(prefix: &'static str, counts: [u32; 2], dot: bool) -> impl IntoElement {
    let bells: Vec<(SharedString, SharedString, u32)> = counts
        .iter()
        .enumerate()
        .map(|(i, &n)| {
            let label = if i == 0 {
                "Notifications".to_string()
            } else {
                format!("Notifications, {n}")
            };
            (format!("{prefix}-bell-{i}").into(), label.into(), n)
        })
        .collect();
    // `.cu-pinv`: icons in `icon-inverse`, hover `background-inverse-hover`:
    // the buttons render in the opposite theme on `background-inverse`.
    let inverse = theme::zone(inverse_theme(theme::current()), move |_, _| {
        div()
            .flex()
            .flex_row()
            .gap(tokens::SPACING_03)
            .children(
                bells
                    .into_iter()
                    .map(|(id, label, n)| header_bell(id, label, n)),
            )
            .into_any_element()
    });
    let tool = |id: &'static str, label: &'static str, icon: CarbonIcon| {
        Button::new(SharedString::from(format!("{prefix}-{id}")), label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Md)
            .icon(icon)
            .icon_only(true)
    };
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .flex()
                .flex_row()
                .child(div().bg(color(ColorRole::BackgroundInverse)).child(inverse)),
        )
        .child(
            // `display: flex; gap: 4; margin-top: 12`.
            div()
                .flex()
                .flex_row()
                .gap(tokens::SPACING_02)
                .mt(tokens::SPACING_04)
                .mb(tokens::SPACING_03)
                .child(Badged::new(
                    tool("changes", "Changes (new)", CarbonIcon::Code),
                    dot.then_some(BadgeIndicator::Dot),
                ))
                .child(tool("terminal", "Terminal", CarbonIcon::Terminal)),
        )
}

/// `display: grid; gap: 8; font-size: 14`.
fn diffs() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(tokens::SPACING_03)
        .child(DiffIndicator::new(Trend::Up, "+12.5% threads this week"))
        .child(DiffIndicator::new(Trend::Down, "−3.1% success rate"))
}

fn layout(
    icons: impl IntoElement,
    shapes: impl IntoElement,
    badges: impl IntoElement,
) -> impl IntoElement {
    PatternGrid::new(3)
        .align_start(true)
        .cell(
            div()
                .child(Caption::new("图标指示：图标 + 形状 + 颜色 + 文字"))
                .child(icons),
        )
        .cell(
            div()
                .child(Caption::new("形状指示：形状在文字前，用于严重程度"))
                .child(shapes),
        )
        .cell(
            div()
                .child(Caption::new(
                    "数字角标只在 48px 页头图标上，最多 3 位（1200 显示为 999+）；工具栏用圆点角标",
                ))
                .child(badges)
                .child(Caption::new("差异指示：符号 + 方向 + 颜色 + 文字"))
                .child(diffs()),
        )
}

/// `components/PatternStatusIndicators/preview.html`.
pub fn reference() -> impl IntoElement {
    layout(
        icon_list(IndicatorSize::S16),
        shape_list(ShapeTextSize::S14),
        badges("si-ref", [4, 1200], true),
    )
}

/// The overview tile: a few icon indicators.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(
        ICONS[3..6]
            .iter()
            .map(|&(kind, label)| IconIndicator::new(kind, label).into_any_element()),
    )
    .into_any_element()
}

const COUNTS: [u32; 5] = [0, 4, 99, 999, 1200];
/// The badge-count choices as shown.
const COUNT_NAMES: [&str; 5] = ["none", "4", "99", "999", "1200"];

impl Render for StatusPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = layout(
            icon_list(IndicatorSize::ALL[self.icon_size]),
            shape_list(ShapeTextSize::ALL[self.text_size]),
            badges("si-live", [self.count, 1200], self.dot),
        );

        let icon_sizes = IndicatorSize::ALL.map(|s| s.name());
        let text_sizes = ShapeTextSize::ALL.map(|s| s.name());
        let count_index = COUNTS.iter().position(|&n| n == self.count).unwrap_or(0);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "icon indicator size",
                controls::choices(
                    "si-icon-size",
                    &icon_sizes,
                    self.icon_size,
                    controls::setter(&entity, |this: &mut Self, i| this.icon_size = i),
                ),
            ))
            .child(controls::field(
                "shape text size",
                controls::choices(
                    "si-text-size",
                    &text_sizes,
                    self.text_size,
                    controls::setter(&entity, |this: &mut Self, i| this.text_size = i),
                ),
            ))
            .child(controls::field(
                "badge count",
                controls::choices(
                    "si-count",
                    &COUNT_NAMES,
                    count_index,
                    controls::setter(&entity, |this: &mut Self, i: usize| this.count = COUNTS[i]),
                ),
            ))
            .child(controls::toggle(
                "si-dot",
                "new-content dot",
                self.dot,
                controls::setter(&entity, |this: &mut Self, on| this.dot = on),
            ));

        Body::new()
            .playground(live)
            .hint("counts over 999 read 999+; the dot marks new content on a toolbar icon")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_names_match_the_counts() {
        for (n, name) in COUNTS.iter().zip(COUNT_NAMES) {
            let expected = if *n == 0 {
                "none".to_string()
            } else {
                n.to_string()
            };
            assert_eq!(name, expected);
        }
    }
}
