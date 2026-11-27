//! T3 · Usage & workflows: [`UsageWindowBar`] and [`UsageShareBar`]
//! (`src/t3/workspace.jsx`, "Usage" in `src/t3.css` plus its later
//! overrides).
use super::parts::{card_sub, child_id, title_tooltip};
use crate::{
    components::chart::{legend, legend_item, palette_role},
    icons::Icon,
    t3::{craft::sans, press::rem},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// Spending against the window (`pace`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pace {
    /// More than 5 points over the even line.
    Ahead,
    /// Within 5 points of it.
    On,
    /// More than 5 points under it.
    Under,
}

impl Pace {
    /// `used > elapsed + 5 ? 'ahead' : used < elapsed - 5 ? 'under' : 'on'`.
    pub fn of(used: f32, elapsed: f32) -> Self {
        if used > elapsed + 5. {
            Self::Ahead
        } else if used < elapsed - 5. {
            Self::Under
        } else {
            Self::On
        }
    }

    /// The fill of the used bar: `.t3-ubar-track i { background:
    /// border-strong-01 }`, `.t3-ubar--ahead … { background: support-warning }`.
    pub fn fill(self) -> Hsla {
        color(match self {
            Self::Ahead => ColorRole::SupportWarning,
            Self::On | Self::Under => ColorRole::BorderStrong01,
        })
    }

    /// The pace label's (ink, weight): `.t3-pace--ahead { color: text-primary;
    /// font-weight: 600 }`, on and under `text-secondary` 400.
    pub fn label_style(self) -> (Hsla, u16) {
        match self {
            Self::Ahead => (color(ColorRole::TextPrimary), 600),
            Self::On | Self::Under => (color(ColorRole::TextSecondary), 400),
        }
    }
}

/// Texts of a [`UsageWindowBar`] (hard-coded in the reference).
#[derive(Clone)]
pub struct UsageTexts {
    /// `{n}% used`.
    pub used: Rc<dyn Fn(u32) -> SharedString>,
    /// `Resets {when}`.
    pub resets: Rc<dyn Fn(&str) -> SharedString>,
    /// The labels for ahead / on / under.
    pub pace: [SharedString; 3],
    /// Ahead with a limit time: `Over pace · limit ~{t}`.
    pub over: Rc<dyn Fn(&str) -> SharedString>,
    /// The pace tooltips (ahead / on / under).
    pub tips: [SharedString; 3],
    /// The even-pace line's tooltip.
    pub line: SharedString,
}

impl Default for UsageTexts {
    fn default() -> Self {
        Self {
            used: Rc::new(|n| format!("{n}% used").into()),
            resets: Rc::new(|w| format!("Resets {w}").into()),
            pace: [
                "Ahead of pace".into(),
                "On pace".into(),
                "Under pace".into(),
            ],
            over: Rc::new(|t| format!("Over pace \u{b7} limit ~{t}").into()),
            tips: [
                "Ahead of pace: spending faster than the window elapses".into(),
                "On pace with the window".into(),
                "Under pace: headroom left for the rest of the window".into(),
            ],
            line: "The line is where even spending would be.".into(),
        }
    }
}

impl UsageTexts {
    fn index(pace: Pace) -> usize {
        match pace {
            Pace::Ahead => 0,
            Pace::On => 1,
            Pace::Under => 2,
        }
    }
}

/// A usage window (`UsageWindowBar`, `.t3-ubar`): label, account and
/// `n% used`; an 8px track with the used share and a 2px even-pace line at
/// `elapsed`; the pace (ahead adds a warning glyph and turns the bar
/// yellow) and the reset time.
#[derive(IntoElement)]
pub struct UsageWindowBar {
    id: ElementId,
    label: SharedString,
    used: u32,
    elapsed: u32,
    resets: SharedString,
    account: Option<SharedString>,
    limit_at: Option<SharedString>,
    texts: UsageTexts,
}

impl UsageWindowBar {
    /// A window bar for `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            used: 0,
            elapsed: 0,
            resets: SharedString::default(),
            account: None,
            limit_at: None,
            texts: UsageTexts::default(),
        }
    }

    /// Percent of the limit used (`used`, 0–100).
    pub fn used(mut self, used: u32) -> Self {
        self.used = used.min(100);
        self
    }

    /// Percent of the window elapsed (`elapsed`, 0–100).
    pub fn elapsed(mut self, elapsed: u32) -> Self {
        self.elapsed = elapsed.min(100);
        self
    }

    /// When the window resets (`resets`).
    pub fn resets(mut self, resets: impl Into<SharedString>) -> Self {
        self.resets = resets.into();
        self
    }

    /// The account (`account`).
    pub fn account(mut self, account: impl Into<SharedString>) -> Self {
        self.account = Some(account.into());
        self
    }

    /// When the limit is reached at this pace (`limitAt`): ahead reads
    /// `Over pace · limit ~{limitAt}`.
    pub fn limit_at(mut self, at: impl Into<SharedString>) -> Self {
        self.limit_at = Some(at.into());
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: UsageTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for UsageWindowBar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let pace = Pace::of(self.used as f32, self.elapsed as f32);
        let k = UsageTexts::index(pace);
        let label = match (&self.limit_at, pace) {
            (Some(at), Pace::Ahead) => (self.texts.over)(at),
            _ => self.texts.pace[k].clone(),
        };
        let (pace_ink, pace_weight) = pace.label_style();
        let fill = div()
            .h_full()
            .w(relative(self.used as f32 / 100.))
            .bg(pace.fill())
            // `.t3-ubar--ahead .t3-ubar-track i { box-shadow: inset 0 0 0 1px rgba(0,0,0,.28) }`.
            .when(pace == Pace::Ahead, |this| {
                this.shadow(crate::t3::press::inset_frame(gpui_kit::hsla(
                    0., 0., 0., 0.28,
                )))
            });
        // `.t3-ubar { display: flex; flex-direction: column; gap: spacing-02; max-width: 30rem;
        // font-size: 0.875rem }`.
        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .flex_col()
            .gap(rem(4.))
            .w_full()
            .max_w(rem(480.))
            .text_color(color(ColorRole::TextPrimary))
            .child(
                // `.t3-ubar-top { display: flex; align-items: baseline; gap: spacing-03 }`.
                div()
                    .flex()
                    .flex_row()
                    .items_baseline()
                    .gap(rem(8.))
                    .child(sans(div(), 14., 600).child(self.label.clone()))
                    .children(self.account.clone().map(card_sub))
                    .child(div().flex_1())
                    .child((self.texts.used)(self.used)),
            )
            .child(
                // `.t3-ubar-track { position: relative; height: 8px; background: layer-accent-01 }`.
                div()
                    .relative()
                    .h(px(8.))
                    .bg(color(ColorRole::LayerAccent01))
                    .child(fill)
                    // `.t3-ubar-pace { position: absolute; top: -3px; bottom: -3px; width: 2px;
                    // background: icon-primary }` at `left: elapsed%`.
                    .child(
                        div()
                            .id((id.clone(), "line"))
                            .absolute()
                            .top(px(-3.))
                            .bottom(px(-3.))
                            .left(relative(self.elapsed as f32 / 100.))
                            .w(px(2.))
                            .bg(color(ColorRole::IconPrimary))
                            .tooltip(title_tooltip(self.texts.line.clone())),
                    ),
            )
            .child(
                // `.t3-ubar-sub { display: flex; justify-content: space-between;
                // font-size: 0.75rem; color: text-secondary }`.
                sans(div(), 12., 400)
                    .flex()
                    .flex_row()
                    .justify_between()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(
                        sans(div(), 12., pace_weight)
                            .id((id.clone(), "pace"))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(4.))
                            .text_color(pace_ink)
                            .tooltip(title_tooltip(self.texts.tips[k].clone()))
                            // `.t3-ubar--ahead .t3-pace::before { content: "warning";
                            // 'Material Symbols Sharp'; FILL 1; 16px; color: support-warning }`.
                            .when(pace == Pace::Ahead, |this| {
                                this.child(
                                    Icon::named("warning")
                                        .filled(true)
                                        .size_length(px(16.))
                                        .color(color(ColorRole::SupportWarning)),
                                )
                            })
                            .child(label),
                    )
                    .child((self.texts.resets)(&self.resets)),
            )
    }
}

/// One provider's share (`segments[]`: `{ label, value }`).
#[derive(Clone, Debug, PartialEq)]
pub struct ShareSegment {
    /// `label`.
    pub label: SharedString,
    /// `value`.
    pub value: f32,
}

impl ShareSegment {
    /// A segment.
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

/// Each segment's fraction of the total (`value / (Σ || 1)`).
pub fn shares(segments: &[ShareSegment]) -> Vec<f32> {
    let total: f32 = segments.iter().map(|s| s.value).sum();
    let total = if total == 0. { 1. } else { total };
    segments.iter().map(|s| s.value / total).collect()
}

/// A share in whole percent (`Math.round(share × 100)`, halves up).
pub fn share_percent(share: f32) -> i32 {
    (share * 100. + 0.5).floor() as i32
}

/// Usage split by provider (`UsageShareBar`, `.t3-share`): a 12px bar of
/// `charts-N` segments 1px apart, each titled `label: value`, and a chart
/// legend with whole percents.
#[derive(IntoElement)]
pub struct UsageShareBar {
    id: ElementId,
    segments: Vec<ShareSegment>,
}

impl UsageShareBar {
    /// A share bar over `segments`.
    pub fn new(id: impl Into<ElementId>, segments: impl IntoIterator<Item = ShareSegment>) -> Self {
        Self {
            id: id.into(),
            segments: segments.into_iter().collect(),
        }
    }
}

/// `value` as JavaScript prints it (`58`, `2.5`).
fn js_number(v: f32) -> String {
    if v.fract() == 0. {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl RenderOnce for UsageShareBar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let fractions = shares(&self.segments);
        // `.t3-share-bar { display: flex; gap: 1px; height: 12px; margin-bottom: spacing-03 }`.
        let bar = div()
            .flex()
            .flex_row()
            .gap(px(1.))
            .h(px(12.))
            .mb(rem(8.))
            .children(
                self.segments
                    .iter()
                    .zip(&fractions)
                    .enumerate()
                    .map(|(i, (s, f))| {
                        div()
                            .id(child_id(&id, "seg", i))
                            .h_full()
                            .w(relative(*f))
                            .bg(color(palette_role(i)))
                            .tooltip(title_tooltip(
                                format!("{}: {}", s.label, js_number(s.value)).into(),
                            ))
                    }),
            );
        // `.cu-chart-legend { margin-bottom: spacing-03 }`.
        let key = legend(false).mb(rem(8.)).children(
            self.segments
                .iter()
                .zip(&fractions)
                .enumerate()
                .map(|(i, (s, f))| {
                    legend_item(
                        color(palette_role(i)),
                        format!("{} {}%", s.label, share_percent(*f)).into(),
                    )
                }),
        );
        // `.t3-share { max-width: 30rem }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(480.))
            .child(bar)
            .child(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pace_compares_used_with_elapsed_within_5_points() {
        assert_eq!(Pace::of(72., 48.), Pace::Ahead);
        assert_eq!(Pace::of(31., 55.), Pace::Under);
        assert_eq!(Pace::of(50., 55.), Pace::On);
        assert_eq!(Pace::of(60., 55.), Pace::On);
        assert_eq!(Pace::of(61., 55.), Pace::Ahead);
    }

    #[test]
    fn shares_round_like_the_legend() {
        let s = [
            ShareSegment::new("Claude", 58.),
            ShareSegment::new("Codex", 27.),
            ShareSegment::new("Cursor", 15.),
        ];
        let f = shares(&s);
        assert_eq!(
            f.iter().map(|x| share_percent(*x)).collect::<Vec<_>>(),
            vec![58, 27, 15]
        );
        // No values: every share is 0 (the total falls back to 1).
        assert_eq!(shares(&[ShareSegment::new("a", 0.)]), vec![0.]);
        assert_eq!(js_number(58.), "58");
        assert_eq!(js_number(2.5), "2.5");
    }

    #[test]
    fn ahead_uses_the_warning_fill() {
        assert_eq!(Pace::Ahead.fill(), color(ColorRole::SupportWarning));
        assert_eq!(Pace::On.fill(), color(ColorRole::BorderStrong01));
        assert_eq!(Pace::Under.label_style().1, 400);
    }
}
