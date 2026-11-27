//! The agent's work ("Work log" in `t3.css`): [`WorkLog`] (`.t3-worklog`),
//! [`ToolActivityIcon`] (`.t3-tool`) and [`WorkingIndicator`]
//! (`.t3-working`), plus the `.t3-shimmer` text sweep they share with the
//! worktree card.
//!
//! - **WorkLog**: a toggle ("Worked for 2m 14s", "N steps") whose chevron
//!   turns 90° in 110ms (`.t3-chev`); open, a left-ruled list of 32px tool
//!   rows; clicking a row with `detail` opens a mono well under it.
//! - **Shimmer**: a 2.4s linear sweep of `text-primary` across
//!   `text-secondary` text; reduced motion shows plain `text-secondary`.
use crate::components::kbd::Kbd;
use crate::t3::base::{Spin, fmt_elapsed, use_ticker};
use crate::t3::press::{self, rem};
use crate::{
    icons::CarbonIcon,
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, ElementId, Hsla, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, StyledText, TextRun, Window,
    div, prelude::FluentBuilder as _,
};
use std::time::Duration;

/// `@keyframes t3-shim`, `animation: t3-shim 2.4s linear infinite`.
pub const SHIMMER_PERIOD: Duration = Duration::from_millis(2400);

/// The highlight strength (0…1) at `x` (0 = text start, 1 = end) at cycle
/// progress `t`: the 300%-wide gradient (`secondary 40%, primary 50%,
/// secondary 60%`) slides from `background-position: 100%` to `-50%`, so
/// its peak crosses from −0.5 to 2.5 text widths with a 0.3-wide ramp.
pub fn shimmer_intensity(x: f32, t: f32) -> f32 {
    let center = -0.5 + 3. * t.rem_euclid(1.);
    (1. - (x - center).abs() / 0.3).max(0.)
}

fn mix(a: Hsla, b: Hsla, k: f32) -> Hsla {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    gpui_kit::Rgba {
        r: a.r + (b.r - a.r) * k,
        g: a.g + (b.g - a.g) * k,
        b: a.b + (b.b - a.b) * k,
        a: a.a + (b.a - a.a) * k,
    }
    .into()
}

/// `text` under the `.t3-shimmer` sweep (per-character colors).
pub(crate) fn shimmer(
    id: impl Into<ElementId>,
    text: SharedString,
    weight: u16,
    cx: &App,
) -> AnyElement {
    let secondary = color(ColorRole::TextSecondary);
    if motion::reduced_motion(cx) {
        // `@media (prefers-reduced-motion: reduce) { .t3-shimmer { animation: none; color: text-secondary } }`.
        return div().text_color(secondary).child(text).into_any_element();
    }
    let primary = color(ColorRole::TextPrimary);
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = chars.len().max(1) as f32;
    div()
        .with_animation(
            id,
            Animation::new(SHIMMER_PERIOD).repeat(),
            move |this, t| {
                let runs = chars
                    .iter()
                    .enumerate()
                    .map(|(i, (_, c))| TextRun {
                        len: c.len_utf8(),
                        font: font(FontFamilyKey::Sans, weight),
                        color: mix(
                            secondary,
                            primary,
                            shimmer_intensity((i as f32 + 0.5) / n, t),
                        ),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    })
                    .collect::<Vec<_>>();
                this.child(StyledText::new(text.clone()).with_runs(runs))
            },
        )
        .into_any_element()
}

/// A tool call's kind (`TOOL_ICON`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ToolKind {
    #[default]
    Read,
    Edit,
    Write,
    Command,
    Search,
    Web,
    Browser,
    Mcp,
    Image,
    Todo,
    Think,
    Agent,
    /// Anything else (`I.Function`).
    Other,
}

impl ToolKind {
    /// Every kind.
    pub const ALL: [ToolKind; 13] = [
        Self::Read,
        Self::Edit,
        Self::Write,
        Self::Command,
        Self::Search,
        Self::Web,
        Self::Browser,
        Self::Mcp,
        Self::Image,
        Self::Todo,
        Self::Think,
        Self::Agent,
        Self::Other,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Edit => "edit",
            Self::Write => "write",
            Self::Command => "command",
            Self::Search => "search",
            Self::Web => "web",
            Self::Browser => "browser",
            Self::Mcp => "mcp",
            Self::Image => "image",
            Self::Todo => "todo",
            Self::Think => "think",
            Self::Agent => "agent",
            Self::Other => "other",
        }
    }

    /// The glyph.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Read => CarbonIcon::Document,
            Self::Edit => CarbonIcon::Edit,
            Self::Write => CarbonIcon::DocumentAdd,
            Self::Command => CarbonIcon::Terminal,
            Self::Search => CarbonIcon::Search,
            Self::Web | Self::Browser => CarbonIcon::Earth,
            Self::Mcp => CarbonIcon::Plug,
            Self::Image => CarbonIcon::Image,
            Self::Todo => CarbonIcon::ListChecked,
            Self::Think => CarbonIcon::Idea,
            Self::Agent => CarbonIcon::Bot,
            Self::Other => CarbonIcon::Function,
        }
    }
}

/// A tool call's status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToolStatus {
    /// Finished.
    #[default]
    Done,
    /// Running: a spinner replaces the icon.
    Running,
    /// Failed: red verb and an error mark.
    Failed,
}

/// The 16px tool glyph, or a spinner while running.
#[derive(IntoElement)]
pub struct ToolActivityIcon {
    id: ElementId,
    kind: ToolKind,
    status: ToolStatus,
    running_label: SharedString,
}

impl ToolActivityIcon {
    /// The icon for `kind` in `status`.
    pub fn new(id: impl Into<ElementId>, kind: ToolKind, status: ToolStatus) -> Self {
        Self {
            id: id.into(),
            kind,
            status,
            running_label: "Running".into(),
        }
    }

    /// The spinner's hidden label (default "Running").
    pub fn running_label(mut self, label: impl Into<SharedString>) -> Self {
        self.running_label = label.into();
        self
    }
}

impl RenderOnce for ToolActivityIcon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.t3-tool { display: flex; width: 1rem; justify-content: center; flex: none }`,
        // `svg { fill: icon-secondary }`.
        div().flex().flex_none().w(rem(16.)).justify_center().child(
            if self.status == ToolStatus::Running {
                Spin::new(self.id)
                    .label(self.running_label)
                    .into_any_element()
            } else {
                press::icon(self.kind.icon(), 16., color(ColorRole::IconSecondary))
                    .into_any_element()
            },
        )
    }
}

/// One work-log entry (`{ kind, verb, target, stat, status, detail }`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkEntry {
    /// `kind`.
    pub kind: ToolKind,
    /// `verb` ("Read", "Edited").
    pub verb: SharedString,
    /// `target` (mono).
    pub target: SharedString,
    /// `stat` ("+48 −3").
    pub stat: Option<SharedString>,
    /// `status`.
    pub status: ToolStatus,
    /// `detail`: opens under the row on click.
    pub detail: Option<SharedString>,
}

impl WorkEntry {
    /// An entry.
    pub fn new(
        kind: ToolKind,
        verb: impl Into<SharedString>,
        target: impl Into<SharedString>,
    ) -> Self {
        Self {
            kind,
            verb: verb.into(),
            target: target.into(),
            ..Default::default()
        }
    }

    /// `stat`.
    pub fn stat(mut self, stat: impl Into<SharedString>) -> Self {
        self.stat = Some(stat.into());
        self
    }

    /// `status`.
    pub fn status(mut self, status: ToolStatus) -> Self {
        self.status = status;
        self
    }

    /// `detail`.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// `WorkLog`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct WorkLogTexts {
    /// "Worked for " (done).
    pub worked: SharedString,
    /// "Working for " (live).
    pub working: SharedString,
    /// `"{n} steps"`.
    pub steps: std::rc::Rc<dyn Fn(usize) -> SharedString>,
    /// The failed mark's hidden label.
    pub failed: SharedString,
}

impl Default for WorkLogTexts {
    fn default() -> Self {
        Self {
            worked: "Worked for ".into(),
            working: "Working for ".into(),
            steps: std::rc::Rc::new(|n| format!("{n} steps").into()),
            failed: "Failed".into(),
        }
    }
}

/// The work log. See the module docs.
#[derive(IntoElement)]
pub struct WorkLog {
    id: ElementId,
    entries: Vec<WorkEntry>,
    elapsed: u64,
    default_open: bool,
    live: bool,
    texts: WorkLogTexts,
}

impl WorkLog {
    /// A log of `entries` that took `elapsed` seconds.
    pub fn new(id: impl Into<ElementId>, entries: Vec<WorkEntry>, elapsed: u64) -> Self {
        Self {
            id: id.into(),
            entries,
            elapsed,
            default_open: false,
            live: false,
            texts: WorkLogTexts::default(),
        }
    }

    /// `defaultOpen`.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// `live`: "Working for".
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: WorkLogTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.t3-chev { transition: transform 110ms cubic-bezier(0.2,0,0.38,0.9) }`.
pub const CHEVRON_MOTION: Motion =
    Motion::new(Curve::ProductiveStandard, Speed::Fast02).spatial(true);

impl RenderOnce for WorkLog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, {
            let open = self.default_open;
            move |_, _| open
        });
        let open = *open_state.read(cx);
        let detail_state = window.use_keyed_state((id.clone(), "detail"), cx, |_, _| None::<usize>);
        let detail = *detail_state.read(cx);
        let (focus, ring) = press::focus((id.clone(), "toggle-focus"), window, cx);
        let (hovered, hover) = press::hover((id.clone(), "toggle-hover"), window, cx);
        let angle = motion::animate(
            (id.clone(), "chev"),
            if open {
                std::f32::consts::FRAC_PI_2
            } else {
                0.
            },
            CHEVRON_MOTION,
            window,
            cx,
        );
        let texts = self.texts.clone();
        let ink = color(if hovered {
            ColorRole::TextPrimary
        } else {
            ColorRole::TextSecondary
        });
        let toggle_state = open_state.clone();
        // `.t3-worklog-toggle { display: flex; align-items: center; gap: spacing-02;
        // padding: spacing-02 spacing-02 spacing-02 0; color: text-secondary }`, `:hover { color: text-primary }`.
        let toggle = gpui_kit::base::Button::new((id.clone(), "toggle"))
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .self_start()
            .gap(rem(4.))
            .pt(rem(4.))
            .pb(rem(4.))
            .pr(rem(4.))
            .text_color(ink)
            .cursor_pointer()
            .on_hover(press::track_hover(hover))
            .on_click(move |_, _, cx| {
                toggle_state.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                })
            })
            .child(
                press::icon(
                    CarbonIcon::ChevronRight,
                    16.,
                    color(ColorRole::IconSecondary),
                )
                .rotate(gpui_kit::radians(angle)),
            )
            .child(SharedString::from(format!(
                "{}{}",
                if self.live {
                    &texts.working
                } else {
                    &texts.worked
                },
                fmt_elapsed(self.elapsed)
            )))
            // `.t3-worklog-count { margin-left: spacing-03; font-size: 0.75rem; color: text-helper }`.
            .child(
                div()
                    .ml(rem(8.))
                    .text_size(rem(12.))
                    .text_color(color(ColorRole::TextHelper))
                    .child((texts.steps)(self.entries.len())),
            )
            .when(ring, |this| this.child(press::ring()));
        let list = open.then(|| {
            let rows: Vec<AnyElement> = self
                .entries
                .into_iter()
                .enumerate()
                .map(|(i, entry)| {
                    let key = |p: &str| -> ElementId {
                        (id.clone(), SharedString::from(format!("{p}-{i}"))).into()
                    };
                    let (row_hovered, row_hover) = press::hover(key("hover"), window, cx);
                    let (row_focus, row_ring) = press::focus(key("focus"), window, cx);
                    let failed = entry.status == ToolStatus::Failed;
                    let row_ink = color(if row_hovered {
                        ColorRole::TextPrimary
                    } else {
                        ColorRole::TextSecondary
                    });
                    let detail_state = detail_state.clone();
                    let open_detail = detail == Some(i) && entry.detail.is_some();
                    // `.t3-wrow { display: flex; width: 100%; align-items: center; gap: spacing-03;
                    // min-height: size-sm; padding: 0 spacing-03; color: text-secondary }`,
                    // `:hover { background: layer-hover-01; color: text-primary }`.
                    let row = gpui_kit::base::Button::new(key("row"))
                        .track_focus(&row_focus)
                        .relative()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_start()
                        .w_full()
                        .gap(rem(8.))
                        .min_h(rem(32.))
                        .px(rem(8.))
                        .text_color(row_ink)
                        .cursor_pointer()
                        .when(row_hovered, |this| this.bg(color(ColorRole::LayerHover01)))
                        .on_hover(press::track_hover(row_hover))
                        .on_click(move |_, _, cx| {
                            detail_state.update(cx, |d, cx| {
                                *d = if *d == Some(i) { None } else { Some(i) };
                                cx.notify();
                            })
                        })
                        .child(ToolActivityIcon::new(key("tool"), entry.kind, entry.status))
                        // `.t3-wrow-verb { color: text-primary; white-space: nowrap }`;
                        // `.t3-wrow--failed .t3-wrow-verb { color: text-error }`.
                        .child(
                            div()
                                .flex_none()
                                .whitespace_nowrap()
                                .text_color(color(if failed {
                                    ColorRole::TextError
                                } else {
                                    ColorRole::TextPrimary
                                }))
                                .child(entry.verb.clone()),
                        )
                        // `.t3-wrow-target { flex: 1; text-overflow: ellipsis; font-family: mono; font-size: 0.75rem }`.
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .font(font(FontFamilyKey::Mono, 400))
                                .text_size(rem(12.))
                                .child(entry.target.clone()),
                        )
                        // `.t3-wrow-stat { font-size: 0.75rem; white-space: nowrap }`.
                        .when_some(entry.stat.clone(), |this, s| {
                            this.child(
                                div()
                                    .flex_none()
                                    .whitespace_nowrap()
                                    .text_size(rem(12.))
                                    .child(s),
                            )
                        })
                        .when(failed, |this| {
                            this.child(press::icon(
                                CarbonIcon::ErrorFilled,
                                16.,
                                color(ColorRole::SupportError),
                            ))
                        })
                        .when(row_ring, |this| this.child(press::ring()));
                    div()
                        .flex()
                        .flex_col()
                        .child(row)
                        .when(open_detail, |this| {
                            // `.t3-wrow-detail { margin: 0 0 spacing-03 spacing-08; padding: spacing-03 spacing-04;
                            // background: layer-01; font-family: mono; font-size: 0.75rem;
                            // white-space: pre-wrap; color: text-secondary }`.
                            this.child(
                                div()
                                    .mb(rem(8.))
                                    .ml(rem(40.))
                                    .py(rem(8.))
                                    .px(rem(12.))
                                    .bg(color(ColorRole::Layer01))
                                    .font(font(FontFamilyKey::Mono, 400))
                                    .text_size(rem(12.))
                                    .line_height(rem(16.))
                                    .text_color(color(ColorRole::TextSecondary))
                                    .children(entry.detail.clone()),
                            )
                        })
                        .into_any_element()
                })
                .collect();
            // `.t3-worklog-list { margin: spacing-02 0 0 spacing-03; padding: 0 0 0 spacing-04;
            // border-left: 1px solid border-subtle-01 }`.
            div()
                .flex()
                .flex_col()
                .mt(rem(4.))
                .ml(rem(8.))
                .pl(rem(12.))
                .border_l_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .children(rows)
        });
        // `.t3-worklog { font-size: 0.875rem }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .text_size(rem(14.))
            .line_height(rem(18.))
            .child(toggle)
            .children(list)
    }
}

/// `WorkingIndicator` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WorkingState {
    /// Working.
    #[default]
    Working,
    /// Thinking.
    Thinking,
    /// Compacting.
    Compacting,
    /// Setting up worktree…
    Setup,
    /// Background setup.
    Background,
}

impl WorkingState {
    /// Every state.
    pub const ALL: [WorkingState; 5] = [
        Self::Working,
        Self::Thinking,
        Self::Compacting,
        Self::Setup,
        Self::Background,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Thinking => "thinking",
            Self::Compacting => "compacting",
            Self::Setup => "setup",
            Self::Background => "background",
        }
    }

    /// The default text.
    pub const fn default_label(self) -> &'static str {
        match self {
            Self::Working => "Working",
            Self::Thinking => "Thinking",
            Self::Compacting => "Compacting",
            Self::Setup => "Setting up worktree…",
            Self::Background => "Background setup",
        }
    }
}

/// The live "Working 1m 14s Esc to interrupt" line at the timeline's end.
#[derive(IntoElement)]
pub struct WorkingIndicator {
    id: ElementId,
    state: WorkingState,
    seconds: u64,
    label: Option<SharedString>,
    key: SharedString,
    hint: SharedString,
}

impl WorkingIndicator {
    /// The indicator for `state`, counting up from `seconds`.
    pub fn new(id: impl Into<ElementId>, state: WorkingState, seconds: u64) -> Self {
        Self {
            id: id.into(),
            state,
            seconds,
            label: None,
            key: "Esc".into(),
            hint: "to interrupt".into(),
        }
    }

    /// `label`: replaces the state's text.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The key and hint after the timer (default "Esc", "to interrupt").
    pub fn hint(mut self, key: impl Into<SharedString>, hint: impl Into<SharedString>) -> Self {
        self.key = key.into();
        self.hint = hint.into();
        self
    }
}

impl RenderOnce for WorkingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let t = use_ticker((id.clone(), "ticker"), self.seconds, true, window, cx);
        let text = self
            .label
            .clone()
            .unwrap_or_else(|| self.state.default_label().into());
        let helper = color(ColorRole::TextHelper);
        // `.t3-working { display: flex; align-items: center; gap: spacing-03; font-size: 0.875rem }`.
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(8.))
            .text_size(rem(14.))
            .line_height(rem(18.))
            .child(shimmer((id.clone(), "shimmer"), text, 400, cx))
            // `.t3-working-time { font-variant-numeric: tabular-nums; color: text-helper; font-size: 0.75rem }`.
            .child(
                div()
                    .text_size(rem(12.))
                    .text_color(helper)
                    .child(SharedString::from(fmt_elapsed(t))),
            )
            .child(Kbd::new(self.key))
            .child(
                div()
                    .text_size(rem(12.))
                    .text_color(helper)
                    .child(self.hint),
            )
            .min_w_0()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shimmer_peak_sweeps_left_to_right() {
        // At t = 0 the peak sits half a width before the text.
        assert_eq!(shimmer_intensity(0., 0.), 0.);
        // At t = 1/6 the peak is on the text start.
        assert!((shimmer_intensity(0., 1. / 6.) - 1.).abs() < 1e-4);
        // At t = 0.5 it is past the middle (x = 1).
        assert!((shimmer_intensity(1., 0.5) - 1.).abs() < 1e-4);
        assert_eq!(shimmer_intensity(0., 0.5), 0.);
    }

    #[test]
    fn tool_kinds_map_to_reference_icons() {
        assert_eq!(ToolKind::Command.icon(), CarbonIcon::Terminal);
        assert_eq!(ToolKind::Browser.icon(), CarbonIcon::Earth);
        assert_eq!(ToolKind::Other.icon(), CarbonIcon::Function);
        assert_eq!(WorkingState::Setup.default_label(), "Setting up worktree…");
    }
}
