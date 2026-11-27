//! TerminalView and TerminalTabs (`TerminalView`, `TerminalTabs` and
//! `parseAnsi` in `src/ghost/panels.jsx`, `.g-term*`, `.g-ttab*` and
//! `.g-ansi-*` in `src/ghost.css`).
//!
//! - **TerminalView**: a `terminal-background` screen in IBM Plex Mono
//!   12/16. Lines carry ANSI SGR escapes ([`parse_ansi`]: reset, bold, the
//!   8 + 8 foreground colors, default foreground) drawn with the `ansi-*`
//!   tokens; a line range can be selected (`terminal-selection`); the last
//!   line ends in a blinking `terminal-cursor` block (`g-blink`, 1s, steps)
//!   that pulses instead under reduced motion; an exit code replaces the
//!   cursor with `[process exited with code N]` (red when non-zero).
//!   `rows` fixes the height. The screen is a focusable, scrolling log.
//! - **TerminalTabs**: the tab strip above it, on the same dark surface:
//!   each tab shows a green dot while running (else the terminal glyph), its
//!   title, " · exited" when done, and a sibling close button; then "New
//!   terminal" and the current tab's working directory, truncated from the
//!   start. ←/→ move between tabs (roving focus), Delete closes the focused
//!   one.
use crate::{
    icons::{CarbonIcon, Icon},
    motion,
    t3::{
        base::{
            active_after_close, endless_phase, radio_step, reduced_pulse_opacity, visually_hidden,
        },
        craft::{mono, rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::parts::{focus_ring, focus_visible, keyed_focus},
};
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, CursorStyle, ElementId, FocusHandle, FontWeight,
    HighlightStyle, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, StyledText, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

// ── ANSI ────────────────────────────────────────────────────────────────

/// An ANSI foreground color (`g-ansi-<name>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnsiColor {
    /// 30 / 90 + 0.
    Black,
    /// 31.
    Red,
    /// 32.
    Green,
    /// 33.
    Yellow,
    /// 34.
    Blue,
    /// 35.
    Magenta,
    /// 36.
    Cyan,
    /// 37.
    White,
    /// 90.
    BrightBlack,
    /// 91.
    BrightRed,
    /// 92.
    BrightGreen,
    /// 93.
    BrightYellow,
    /// 94.
    BrightBlue,
    /// 95.
    BrightMagenta,
    /// 96.
    BrightCyan,
    /// 97.
    BrightWhite,
}

impl AnsiColor {
    /// `ANSI = ['black', 'red', 'green', 'yellow', 'blue', 'magenta', 'cyan', 'white']`.
    const BASE: [AnsiColor; 8] = [
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Blue,
        Self::Magenta,
        Self::Cyan,
        Self::White,
    ];
    const BRIGHT: [AnsiColor; 8] = [
        Self::BrightBlack,
        Self::BrightRed,
        Self::BrightGreen,
        Self::BrightYellow,
        Self::BrightBlue,
        Self::BrightMagenta,
        Self::BrightCyan,
        Self::BrightWhite,
    ];

    /// `.g-ansi-<name> { color: var(--ansi-<name>) }`.
    pub const fn role(self) -> ColorRole {
        match self {
            Self::Black => ColorRole::AnsiBlack,
            Self::Red => ColorRole::AnsiRed,
            Self::Green => ColorRole::AnsiGreen,
            Self::Yellow => ColorRole::AnsiYellow,
            Self::Blue => ColorRole::AnsiBlue,
            Self::Magenta => ColorRole::AnsiMagenta,
            Self::Cyan => ColorRole::AnsiCyan,
            Self::White => ColorRole::AnsiWhite,
            Self::BrightBlack => ColorRole::AnsiBrightBlack,
            Self::BrightRed => ColorRole::AnsiBrightRed,
            Self::BrightGreen => ColorRole::AnsiBrightGreen,
            Self::BrightYellow => ColorRole::AnsiBrightYellow,
            Self::BrightBlue => ColorRole::AnsiBrightBlue,
            Self::BrightMagenta => ColorRole::AnsiBrightMagenta,
            Self::BrightCyan => ColorRole::AnsiBrightCyan,
            Self::BrightWhite => ColorRole::AnsiBrightWhite,
        }
    }
}

/// A styled run of a parsed line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnsiRun {
    /// The text, escapes removed.
    pub text: String,
    /// The foreground (`None`: `terminal-text`).
    pub fg: Option<AnsiColor>,
    /// `.g-ansi-bold { font-weight: 600 }`.
    pub bold: bool,
}

/// `parseAnsi(s)`: split a line at its SGR escapes (`\x1b[…m`) into runs.
/// Parameters apply in order: `0` (or none) resets, `1` bolds, `30–37` and
/// `90–97` pick the foreground, `39` restores it; others are ignored. An
/// empty parameter counts as `0` (`Number('')`).
pub fn parse_ansi(s: &str) -> Vec<AnsiRun> {
    let mut out = Vec::new();
    let (mut fg, mut bold) = (None, false);
    let mut push = |text: &str, fg: Option<AnsiColor>, bold: bool| {
        if !text.is_empty() {
            out.push(AnsiRun {
                text: text.to_string(),
                fg,
                bold,
            });
        }
    };
    let mut last = 0;
    let mut i = 0;
    while let Some(at) = s[i..].find("\x1b[").map(|p| p + i) {
        // `/\x1b\[([\d;]*)m/`: digits and semicolons, then `m`.
        let params_start = at + 2;
        let params_end = params_start
            + s[params_start..]
                .find(|c: char| !(c.is_ascii_digit() || c == ';'))
                .unwrap_or(s.len() - params_start);
        if !s[params_end..].starts_with('m') {
            i = at + 2;
            continue;
        }
        push(&s[last..at], fg, bold);
        let params = &s[params_start..params_end];
        let params = if params.is_empty() { "0" } else { params };
        for code in params.split(';').map(|p| p.parse::<u32>().unwrap_or(0)) {
            match code {
                0 => {
                    fg = None;
                    bold = false;
                }
                1 => bold = true,
                30..=37 => fg = Some(AnsiColor::BASE[(code - 30) as usize]),
                90..=97 => fg = Some(AnsiColor::BRIGHT[(code - 90) as usize]),
                39 => fg = None,
                _ => {}
            }
        }
        last = params_end + 1;
        i = last;
    }
    push(&s[last..], fg, bold);
    out
}

/// The runs as one string plus highlight ranges, for a `StyledText`.
pub fn ansi_highlights(runs: &[AnsiRun]) -> (String, Vec<(Range<usize>, HighlightStyle)>) {
    let mut text = String::new();
    let mut styles = Vec::new();
    for run in runs {
        let start = text.len();
        text.push_str(&run.text);
        if run.fg.is_some() || run.bold {
            styles.push((
                start..text.len(),
                HighlightStyle {
                    color: run.fg.map(|c| color(c.role())),
                    font_weight: run.bold.then_some(FontWeight::SEMIBOLD),
                    ..Default::default()
                },
            ));
        }
    }
    (text, styles)
}

// ── TerminalView ────────────────────────────────────────────────────────

/// `.g-term { font-size: 0.75rem; line-height: 1.33333 }`: 16px lines.
pub const LINE: f32 = 16.;
/// `.g-term-screen { padding: var(--spacing-03) var(--spacing-04) }`.
pub const SCREEN_PAD: (f32, f32) = (8., 12.);
/// `@keyframes g-blink { 50% { opacity: 0 } }`, `1s steps(1) infinite`.
pub const BLINK_PERIOD: Duration = Duration::from_millis(1000);
/// Reduced motion: `animation: g-pulse 1.4s ease-in-out infinite`.
pub const REDUCED_PULSE: Duration = Duration::from_millis(1400);

/// `height: calc(rows * 1.33333 * 0.75rem + 1rem)`.
pub fn rows_height(rows: usize) -> f32 {
    rows as f32 * LINE + 16.
}

/// The cursor's opacity at blink progress `t` (0..1): on for the first
/// half, off for the second (`steps(1)` holds each keyframe's value).
pub fn blink_opacity(t: f32) -> f32 {
    if t.rem_euclid(1.) < 0.5 { 1. } else { 0. }
}

/// Whether line `i` is inside `selection` (`i >= from && i <= to`).
pub fn is_selected(i: usize, selection: Option<(usize, usize)>) -> bool {
    selection.is_some_and(|(from, to)| i >= from && i <= to)
}

/// The terminal's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct TerminalTexts {
    /// The log's name without a title: `"Terminal"`.
    pub name: SharedString,
    /// `"[process exited with code {n}]"`.
    pub exited: Rc<dyn Fn(i32) -> SharedString>,
}

impl Default for TerminalTexts {
    fn default() -> Self {
        Self {
            name: "Terminal".into(),
            exited: Rc::new(|n| format!("[process exited with code {n}]").into()),
        }
    }
}

/// A terminal screen. See the module docs.
#[derive(IntoElement)]
pub struct TerminalView {
    id: ElementId,
    lines: Vec<SharedString>,
    cursor: bool,
    selection: Option<(usize, usize)>,
    title: Option<SharedString>,
    exited: Option<i32>,
    rows: Option<usize>,
    texts: TerminalTexts,
}

impl TerminalView {
    /// An empty terminal with a cursor.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            lines: Vec::new(),
            cursor: true,
            selection: None,
            title: None,
            exited: None,
            rows: None,
            texts: TerminalTexts::default(),
        }
    }

    /// The screen's lines, with ANSI SGR escapes (`lines`).
    pub fn lines(mut self, lines: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.lines = lines.into_iter().map(Into::into).collect();
        self
    }

    /// Show the cursor at the end of the last line (`cursor`, default on).
    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }

    /// Select lines `from..=to` (`selection: [from, to]`).
    pub fn selection(mut self, from: usize, to: usize) -> Self {
        self.selection = Some((from, to));
        self
    }

    /// The log's accessible name (`title`).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The process exited with `code` (`exited`); hides the cursor.
    pub fn exited(mut self, code: i32) -> Self {
        self.exited = Some(code);
        self
    }

    /// Fix the height to `rows` lines (`rows`).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = Some(rows);
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: TerminalTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.g-term-cursor { width: 0.6em; height: 1.2em; vertical-align:
/// text-bottom; background: var(--terminal-cursor) }`, blinking.
/// `pulse` is the reduced-motion opacity (`g-pulse`, see
/// [`crate::t3::base::REDUCED_PULSE_PERIOD`]); `None` blinks.
fn cursor(id: ElementId, pulse: Option<f32>) -> AnyElement {
    let block = div()
        .flex_none()
        .w(rem(12. * 0.6))
        .h(rem(12. * 1.2))
        .bg(color(ColorRole::TerminalCursor));
    if let Some(opacity) = pulse {
        // GPUI freezes `with_animation` loops under reduced motion; this
        // one keeps running (`ghost.css`).
        block.opacity(opacity).into_any_element()
    } else {
        block
            .with_animation(id, Animation::new(BLINK_PERIOD).repeat(), |el, t| {
                el.opacity(blink_opacity(t))
            })
            .into_any_element()
    }
}

impl RenderOnce for TerminalView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let reduced = motion::reduced_motion(cx);
        let last = self.lines.len().saturating_sub(1);
        let cursor_pulse = (reduced && self.cursor && self.exited.is_none())
            .then(|| reduced_pulse_opacity(endless_phase(REDUCED_PULSE, window)));
        let show_cursor = self.cursor && self.exited.is_none();
        let lines = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let (text, styles) = ansi_highlights(&parse_ansi(line));
                div()
                    .flex()
                    .items_end()
                    .flex_wrap()
                    .min_h(rem(LINE))
                    .when(is_selected(i, self.selection), |d| {
                        d.bg(color(ColorRole::TerminalSelection))
                    })
                    .child(StyledText::new(text).with_highlights(styles))
                    .when(show_cursor && i == last, |d| {
                        d.child(cursor((id.clone(), "cursor").into(), cursor_pulse))
                    })
            })
            .collect::<Vec<_>>();
        let exit = self.exited.map(|code| {
            // `.g-term-exit { margin-top: var(--spacing-03) }`, red when
            // non-zero, else `ansi-bright-black`.
            div()
                .mt(tokens::SPACING_03)
                .text_color(color(if code != 0 {
                    ColorRole::AnsiRed
                } else {
                    ColorRole::AnsiBrightBlack
                }))
                .child((self.texts.exited)(code))
        });
        let name = self.title.clone().unwrap_or(self.texts.name.clone());
        div()
            .relative()
            .min_w_0()
            .when_some(self.rows, |d, r| d.h(rem(rows_height(r))))
            .child(
                mono(div(), 12., 400)
                    .id(id)
                    // A tracked handle carries its own tab-stop flag; the div's
                    // `.tab_stop` does not reach it.
                    .track_focus(&focus.clone().tab_index(0).tab_stop(true))
                    .size_full()
                    .overflow_y_scroll()
                    .bg(color(ColorRole::TerminalBackground))
                    .text_color(color(ColorRole::TerminalText))
                    .line_height(rem(LINE))
                    .child(visually_hidden(name))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .py(rem(SCREEN_PAD.0))
                            .px(rem(SCREEN_PAD.1))
                            .children(lines)
                            .when_some(exit, |d, e| d.child(e)),
                    ),
            )
            .when(ring, |d| d.child(focus_ring()))
    }
}

// ── TerminalTabs ────────────────────────────────────────────────────────

/// One terminal tab (`tabs[i]`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerminalTab {
    /// `title`.
    pub title: SharedString,
    /// `running`: a green dot instead of the glyph.
    pub running: bool,
    /// `exited`: " · exited".
    pub exited: bool,
    /// `cwd`: shown at the strip's end for the current tab.
    pub cwd: Option<SharedString>,
}

impl TerminalTab {
    /// A tab titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Mark it running.
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// Mark it exited.
    pub fn exited(mut self, exited: bool) -> Self {
        self.exited = exited;
        self
    }

    /// Its working directory.
    pub fn cwd(mut self, cwd: impl Into<SharedString>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

/// `color-mix(in srgb, var(--terminal-text) <pct>%, transparent)`.
pub fn terminal_tint(pct: f32) -> Hsla {
    let mut c = color(ColorRole::TerminalText);
    c.a *= pct / 100.;
    c
}

/// The tab strip's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct TerminalTabsTexts {
    /// The tab list's name: `"Terminals"`.
    pub list: SharedString,
    /// Hidden after a running tab's title: `"(running)"`.
    pub running: SharedString,
    /// After an exited tab's title: `" · exited"`.
    pub exited: SharedString,
    /// `"Close {title}"`.
    pub close: Rc<dyn Fn(&str) -> SharedString>,
    /// `"New terminal"`.
    pub new_terminal: SharedString,
}

impl Default for TerminalTabsTexts {
    fn default() -> Self {
        Self {
            list: "Terminals".into(),
            running: "(running)".into(),
            exited: " · exited".into(),
            close: Rc::new(|t| format!("Close {t}").into()),
            new_terminal: "New terminal".into(),
        }
    }
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The terminal tab strip. See the module docs.
#[derive(IntoElement)]
pub struct TerminalTabs {
    id: ElementId,
    tabs: Vec<TerminalTab>,
    value: Option<usize>,
    default_value: usize,
    on_value_change: Option<IndexHandler>,
    on_new: Option<Handler>,
    on_close: Option<IndexHandler>,
    texts: TerminalTabsTexts,
}

impl TerminalTabs {
    /// A strip of `tabs`.
    pub fn new(id: impl Into<ElementId>, tabs: impl IntoIterator<Item = TerminalTab>) -> Self {
        Self {
            id: id.into(),
            tabs: tabs.into_iter().collect(),
            value: None,
            default_value: 0,
            on_value_change: None,
            on_new: None,
            on_close: None,
            texts: TerminalTabsTexts::default(),
        }
    }

    /// The current tab, controlled (`value`).
    pub fn value(mut self, value: usize) -> Self {
        self.value = Some(value);
        self
    }

    /// The first current tab when uncontrolled (`defaultValue`, default 0).
    /// Closing a tab then moves it as a browser tab strip does
    /// ([`active_after_close`]), which assumes `on_close` removes that tab.
    pub fn default_value(mut self, value: usize) -> Self {
        self.default_value = value;
        self
    }

    /// Called with the tab the user picks (`onValueChange`).
    pub fn on_value_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// "New terminal" pressed (`onNew`).
    pub fn on_new(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new = Some(Rc::new(f));
        self
    }

    /// A tab's close button or Delete (`onClose(i)`).
    pub fn on_close(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: TerminalTabsTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// A keyed hover flag and its `on_hover` updater.
fn hover_state(
    id: ElementId,
    window: &mut Window,
    cx: &mut App,
) -> (bool, impl Fn(&bool, &mut Window, &mut App) + 'static) {
    let store = window.use_keyed_state(id, cx, |_, _| false);
    let over = *store.read(cx);
    (over, move |h: &bool, _: &mut Window, cx: &mut App| {
        store.update(cx, |v, cx| {
            if *v != *h {
                *v = *h;
                cx.notify();
            }
        })
    })
}

impl RenderOnce for TerminalTabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let n = self.tabs.len();
        let default = self.default_value;
        let store = window.use_keyed_state((id.clone(), "value"), cx, |_, _| default);
        let controlled = self.value.is_some();
        let current = self
            .value
            .unwrap_or(*store.read(cx))
            .min(n.saturating_sub(1));
        let handles: Vec<FocusHandle> = (0..n)
            .map(|i| {
                keyed_focus(
                    (id.clone(), SharedString::from(format!("tab-{i}"))),
                    window,
                    cx,
                )
            })
            .collect();
        let select: IndexHandler = {
            let store = store.clone();
            let on_change = self.on_value_change.clone();
            Rc::new(move |i, window, cx| {
                store.update(cx, |v, cx| {
                    *v = i;
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(i, window, cx);
                }
            })
        };
        // A tab closed (× or Delete): when uncontrolled, the selection moves
        // as a browser tab strip's does ([`active_after_close`]); this assumes
        // the `on_close` handler removes that tab.
        let on_close: Option<IndexHandler> = self.on_close.clone().map(|f| {
            let store = store.clone();
            Rc::new(move |i, window: &mut Window, cx: &mut App| {
                if !controlled {
                    store.update(cx, |v, cx| {
                        *v = active_after_close(current, i, n);
                        cx.notify();
                    });
                }
                f(i, window, cx)
            }) as IndexHandler
        });
        let rule = terminal_tint(14.);
        let tabs = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let tid: ElementId = (id.clone(), SharedString::from(format!("t{i}"))).into();
                let on = i == current;
                let focus = handles[i].clone();
                let ring = focus_visible(&focus, window);
                let (x_over, x_hover) = hover_state((tid.clone(), "x-hover").into(), window, cx);
                let x_focus = keyed_focus((tid.clone(), "x-focus"), window, cx);
                let x_ring = focus_visible(&x_focus, window);
                let pick = select.clone();
                let keys = select.clone();
                let handles = handles.clone();
                let close = on_close.clone();
                let close_key = on_close.clone();
                let glyph: AnyElement = if tab.running {
                    // `.g-ttab-dot { width: 6px; height: 6px; border-radius: 50%;
                    // background: var(--ansi-green) }`.
                    div()
                        .flex_none()
                        .size(px(6.))
                        .rounded_full()
                        .bg(color(ColorRole::AnsiGreen))
                        .into_any_element()
                } else {
                    Icon::carbon(CarbonIcon::Terminal)
                        .size_length(rem(14.))
                        .color(color(ColorRole::TerminalText))
                        .into_any_element()
                };
                // `.g-ttab { border-right: 1px solid <14%> }`, `--on { background:
                // <9%>; box-shadow: inset 0 2px 0 var(--ansi-blue) }`.
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .border_r_1()
                    .border_color(rule)
                    .when(on, |d| {
                        d.bg(terminal_tint(9.)).child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .h(px(2.))
                                .bg(color(ColorRole::AnsiBlue)),
                        )
                    })
                    .child(
                        // `.g-ttab-btn { gap: var(--spacing-02); padding: 0
                        // var(--spacing-02) 0 var(--spacing-04); opacity: .7;
                        // font-size: 0.75rem }`, `--on .g-ttab-btn { opacity: 1 }`.
                        sans(gpui_kit::base::Button::new(tid.clone()), 12., 400)
                            .track_focus(&focus)
                            .tab_stop(on)
                            // `aria-selected={v === i}`.
                            .aria_selected(on)
                            .relative()
                            .flex()
                            .items_center()
                            .h_full()
                            .gap(tokens::SPACING_02)
                            .pl(tokens::SPACING_04)
                            .pr(tokens::SPACING_02)
                            .opacity(if on { 1. } else { 0.7 })
                            .cursor(CursorStyle::PointingHand)
                            .on_click(move |_, w, cx| pick(i, w, cx))
                            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                                let key = e.keystroke.key.as_str();
                                // `{ ArrowRight: 1, ArrowLeft: -1 }`, wrapping.
                                let step = match key {
                                    "right" | "left" => radio_step(key, n, Some(i)),
                                    _ => None,
                                };
                                if let Some(next) = step {
                                    keys(next, window, cx);
                                    window.focus(&handles[next], cx);
                                    cx.stop_propagation();
                                } else if key == "delete"
                                    && let Some(f) = &close_key
                                {
                                    f(i, window, cx);
                                    cx.stop_propagation();
                                }
                            })
                            .child(glyph)
                            .child(tab.title.clone())
                            .when(tab.running, |b| {
                                b.child(visually_hidden(self.texts.running.clone()))
                            })
                            .when(tab.exited, |b| {
                                b.child(div().opacity(0.6).child(self.texts.exited.clone()))
                            })
                            .when(ring, |b| b.child(focus_ring())),
                    )
                    .child(
                        // `.g-ttab-x { width: 1.25rem; height: 1.25rem; margin-right:
                        // var(--spacing-02); opacity: .6 }`, hover `opacity: 1;
                        // background: <14%>`.
                        gpui_kit::base::Button::new((tid.clone(), "x"))
                            .track_focus(&x_focus)
                            .tab_stop(false)
                            .accessibility_label((self.texts.close)(&tab.title))
                            .relative()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .size(rem(20.))
                            .mr(tokens::SPACING_02)
                            .opacity(if x_over { 1. } else { 0.6 })
                            .when(x_over, |b| b.bg(rule))
                            .cursor(CursorStyle::PointingHand)
                            .on_hover(x_hover)
                            .on_click(move |_, w, cx| {
                                if let Some(f) = &close {
                                    f(i, w, cx);
                                }
                            })
                            .child(
                                Icon::carbon(CarbonIcon::Close)
                                    .size_length(rem(12.))
                                    .color(color(ColorRole::TerminalText)),
                            )
                            .when(x_ring, |b| b.child(focus_ring())),
                    )
            })
            .collect::<Vec<_>>();
        let (new_over, new_hover) = hover_state((id.clone(), "new-hover").into(), window, cx);
        let new_focus = keyed_focus((id.clone(), "new-focus"), window, cx);
        let new_ring = focus_visible(&new_focus, window);
        let on_new = self.on_new.clone();
        let row = crate::density::current().row();
        let cwd = self.tabs.get(current).and_then(|t| t.cwd.clone());
        // `.g-ttabs { height: var(--density-row-md); background:
        // var(--terminal-background); color: var(--terminal-text);
        // border-bottom: 1px solid <14%> }`.
        div()
            .id(id.clone())
            .flex()
            .items_stretch()
            .h(row)
            .min_w_0()
            .bg(color(ColorRole::TerminalBackground))
            .text_color(color(ColorRole::TerminalText))
            .border_b_1()
            .border_color(rule)
            .child(visually_hidden(self.texts.list.clone()))
            .child(
                // `.g-ttabs-list { display: flex; min-width: 0; overflow-x: auto }`.
                div()
                    .id((id.clone(), "list"))
                    .flex()
                    .items_stretch()
                    .min_w_0()
                    .overflow_x_scroll()
                    .children(tabs),
            )
            .child(
                // `.g-ttabs .g-iconbtn { width/height: var(--density-row-md);
                // color: var(--terminal-text) }`, hover `<9%>`.
                gpui_kit::base::Button::new((id.clone(), "new"))
                    .track_focus(&new_focus)
                    .accessibility_label(self.texts.new_terminal.clone())
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .w(row)
                    .h(row)
                    .when(new_over, |b| b.bg(terminal_tint(9.)))
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(new_hover)
                    .on_click(move |_, w, cx| {
                        if let Some(f) = &on_new {
                            f(w, cx);
                        }
                    })
                    .child(
                        Icon::carbon(CarbonIcon::Add)
                            .size_length(rem(16.))
                            .color(color(ColorRole::TerminalText)),
                    )
                    .when(new_ring, |b| b.child(focus_ring())),
            )
            .child(div().flex_1())
            .when_some(cwd, |d, cwd| {
                // `.g-ttabs-cwd { align-self: center; padding-right:
                // var(--spacing-04); opacity: .6; max-width: 40%; direction:
                // rtl; text-overflow: ellipsis }`: truncated from the start.
                d.child(
                    mono(div(), 12., 400)
                        .self_center()
                        .min_w_0()
                        .max_w(relative(0.4))
                        .pr(tokens::SPACING_04)
                        .opacity(0.6)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis_start()
                        .child(cwd),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, fg: Option<AnsiColor>, bold: bool) -> AnsiRun {
        AnsiRun {
            text: text.into(),
            fg,
            bold,
        }
    }

    #[test]
    fn parse_ansi_follows_the_reference() {
        use AnsiColor::*;
        assert_eq!(
            parse_ansi("  \x1b[32mVITE\x1b[0m v6.0.3  ready in \x1b[1m412\x1b[0m ms"),
            vec![
                run("  ", None, false),
                run("VITE", Some(Green), false),
                run(" v6.0.3  ready in ", None, false),
                run("412", None, true),
                run(" ms", None, false),
            ]
        );
        // Bright colors, `39` restores the foreground, an empty list resets.
        assert_eq!(
            parse_ansi("\x1b[90m10:42\x1b[39m x \x1b[1;33mw\x1b[m!"),
            vec![
                run("10:42", Some(BrightBlack), false),
                run(" x ", None, false),
                run("w", Some(Yellow), true),
                run("!", None, false),
            ]
        );
        // A trailing `;` is a `0`: `1;` bolds then resets.
        assert_eq!(parse_ansi("\x1b[1;ma"), vec![run("a", None, false)]);
        // Not SGR: left in the text.
        assert_eq!(parse_ansi("\x1b[2Kx"), vec![run("\x1b[2Kx", None, false)]);
        assert_eq!(parse_ansi(""), vec![]);
    }

    #[test]
    fn ansi_colors_use_their_tokens() {
        assert_eq!(AnsiColor::Red.role(), ColorRole::AnsiRed);
        assert_eq!(AnsiColor::BrightCyan.role(), ColorRole::AnsiBrightCyan);
        let (text, styles) = ansi_highlights(&parse_ansi("a\x1b[36mb\x1b[0m"));
        assert_eq!(text, "ab");
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].0, 1..2);
    }

    #[test]
    fn geometry_and_timing() {
        // `calc(10 * 1.33333 * 0.75rem + 1rem)` = 176px.
        assert_eq!(rows_height(10), 176.);
        // `steps(1)`: on, then off at 50%.
        assert_eq!(blink_opacity(0.2), 1.);
        assert_eq!(blink_opacity(0.6), 0.);
        assert!(is_selected(2, Some((2, 3))));
        assert!(is_selected(3, Some((2, 3))));
        assert!(!is_selected(4, Some((2, 3))));
        assert!(!is_selected(0, None));
    }

    #[test]
    fn tints_mix_terminal_text() {
        crate::theme::tests_set(crate::CarbonTheme::White);
        let t = terminal_tint(14.);
        let base = color(ColorRole::TerminalText);
        assert_eq!((t.h, t.s, t.l), (base.h, base.s, base.l));
        assert!((t.a - base.a * 0.14).abs() < 1e-6);
    }
}
