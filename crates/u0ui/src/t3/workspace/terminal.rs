//! T3 · Files & preview: [`TerminalDrawer`] (`src/t3/workspace.jsx`,
//! "Terminal" in `src/t3.css`).
use super::parts::{child_id, icon_button};
use crate::{
    components::{Button, ScrollArea, ScrollAxes},
    icons::CarbonIcon,
    motion,
    t3::{
        base::active_after_close,
        craft::{mono, sans},
        press::{self, Handler, rem},
    },
    theme::color,
    tokens::{BORDER_RADIUS_02, ColorRole},
    u0::primitives::parts::{focus_slot, insert_slot_anchor, vacate_slot},
};
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, CursorStyle, ElementId, HighlightStyle,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledText, TestSupportExt as _, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::{rc::Rc, time::Duration};

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// One terminal tab (`tabs[]`: `{ title, running }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalTab {
    /// `title`.
    pub title: SharedString,
    /// `running`: a process is running (green dot).
    pub running: bool,
}

impl TerminalTab {
    /// A tab titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            running: false,
        }
    }

    /// A process is running.
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }
}

/// Texts of a [`TerminalDrawer`] (hard-coded in the reference).
#[derive(Clone)]
pub struct TerminalTexts {
    /// Close a tab (`Close {title}`).
    pub close: Rc<dyn Fn(&str) -> SharedString>,
    /// `New terminal`.
    pub new_terminal: SharedString,
    /// `Split right`.
    pub split: SharedString,
    /// `Add selection to chat`.
    pub add_to_chat: SharedString,
    /// `Maximize`.
    pub maximize: SharedString,
    /// `Hide terminal (⌘J)`.
    pub hide: SharedString,
    /// The exit line (`Process exited with code {n}`).
    pub exited: Rc<dyn Fn(i32) -> SharedString>,
}

impl Default for TerminalTexts {
    fn default() -> Self {
        Self {
            close: Rc::new(|t| format!("Close {t}").into()),
            new_terminal: "New terminal".into(),
            split: "Split right".into(),
            add_to_chat: "Add selection to chat".into(),
            maximize: "Maximize".into(),
            hide: "Hide terminal (\u{2318}J)".into(),
            exited: Rc::new(|n| format!("Process exited with code {n}").into()),
        }
    }
}

/// `@keyframes cu-blink { 50% { opacity: 0 } }`, 1s `steps(1)`.
pub const BLINK_PERIOD: Duration = Duration::from_secs(1);

/// The cursor's opacity at cycle progress `t` (`steps(1)`: on, then off
/// from the middle).
pub fn blink_opacity(t: f32) -> f32 {
    if t.rem_euclid(1.) < 0.5 { 1. } else { 0. }
}

/// The `.t3-term-view` text: the lines joined by newlines (`white-space:
/// pre-wrap` keeps their spaces), then, while running, the cursor's space
/// (`.t3-term-cursor`) with its byte range.
pub fn screen_text(
    lines: &[SharedString],
    running: bool,
) -> (SharedString, Option<std::ops::Range<usize>>) {
    let mut text = lines
        .iter()
        .map(|l| l.as_ref())
        .collect::<Vec<_>>()
        .join("\n");
    if !running {
        return (text.into(), None);
    }
    let at = text.len();
    text.push(' ');
    (text.into(), Some(at..at + 1))
}

/// The bottom terminal drawer (`TerminalDrawer`, `.t3-term`): a 32px tab
/// bar (the active tab on `background` under a 2px `border-interactive`
/// top line, a green dot while running, a close ×), New terminal, Split,
/// Add selection to chat, Maximize and Hide; under it the output on
/// `terminal-background` with a blinking block cursor, or the exit line.
#[derive(IntoElement)]
pub struct TerminalDrawer {
    id: ElementId,
    tabs: Vec<TerminalTab>,
    active: usize,
    controlled: bool,
    lines: Vec<SharedString>,
    exited: Option<i32>,
    texts: TerminalTexts,
    viewport: Option<AnyElement>,
    bar_extra: Vec<AnyElement>,
    on_select: Option<IndexHandler>,
    on_close: Option<IndexHandler>,
    on_new: Option<Handler>,
    on_split: Option<Handler>,
    on_add_to_chat: Option<Handler>,
    on_maximize: Option<Handler>,
    on_hide: Option<Handler>,
}

impl TerminalDrawer {
    /// A drawer with `tabs`.
    pub fn new(id: impl Into<ElementId>, tabs: impl IntoIterator<Item = TerminalTab>) -> Self {
        Self {
            id: id.into(),
            tabs: tabs.into_iter().collect(),
            active: 0,
            controlled: false,
            lines: Vec::new(),
            exited: None,
            texts: TerminalTexts::default(),
            viewport: None,
            bar_extra: Vec::new(),
            on_select: None,
            on_close: None,
            on_new: None,
            on_split: None,
            on_add_to_chat: None,
            on_maximize: None,
            on_hide: None,
        }
    }

    /// Controlled active tab index.
    pub fn controlled(mut self, active: usize) -> Self {
        self.active = active;
        self.controlled = true;
        self
    }

    /// Custom interactive terminal viewport element replacing static `lines`.
    pub fn viewport(mut self, viewport: impl IntoElement) -> Self {
        self.viewport = Some(viewport.into_any_element());
        self
    }

    /// Extra element placed in the 32px tab strip before the right-hand actions.
    pub fn bar_extra(mut self, element: impl IntoElement) -> Self {
        self.bar_extra.push(element.into_any_element());
        self
    }

    /// The tab selected at first (`active`, default 0); the drawer then
    /// keeps its own selection. A tab's × moves it as a browser tab strip
    /// does ([`active_after_close`]), which assumes the `on_close` handler
    /// removes that tab.
    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    /// The output lines (`lines`).
    pub fn lines<S: Into<SharedString>>(mut self, lines: impl IntoIterator<Item = S>) -> Self {
        self.lines = lines.into_iter().map(Into::into).collect();
        self
    }

    /// The process exited with `code` (`exited`): the exit line replaces the cursor.
    pub fn exited(mut self, code: Option<i32>) -> Self {
        self.exited = code;
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: TerminalTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A tab selected.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// A tab's × pressed (`onClose(i)`).
    pub fn on_close(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// New terminal pressed.
    pub fn on_new(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new = Some(Rc::new(f));
        self
    }

    /// Split right pressed.
    pub fn on_split(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_split = Some(Rc::new(f));
        self
    }

    /// Add selection to chat pressed.
    pub fn on_add_to_chat(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add_to_chat = Some(Rc::new(f));
        self
    }

    /// Maximize pressed.
    pub fn on_maximize(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_maximize = Some(Rc::new(f));
        self
    }

    /// Hide pressed.
    pub fn on_hide(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_hide = Some(Rc::new(f));
        self
    }
}

fn click(b: Button, f: Option<Handler>) -> Button {
    match f {
        Some(f) => b.on_click(move |_, w, cx| f(w, cx)),
        None => b,
    }
}

impl RenderOnce for TerminalDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.active;
        let controlled = self.controlled;
        let selected = window.use_keyed_state((id.clone(), "active"), cx, move |_, _| initial);
        let len = self.tabs.len();
        let active = if controlled {
            self.active
        } else {
            *selected.read(cx)
        }
        .min(len.saturating_sub(1));
        let mut bar: Vec<AnyElement> = Vec::new();
        // Where focus rests when a focused tab closes: Chromium keeps the
        // Tab starting point where a removed focused element was.
        let slot = focus_slot((id.clone(), "slot"), window, cx);
        for (i, tab) in self.tabs.iter().enumerate() {
            let on = i == active;
            let tab_id = child_id(&id, "tab", i);
            let (tab_focus, tab_ring) = press::focus((tab_id.clone(), "focus"), window, cx);
            let x_id = child_id(&id, "tab-x", i);
            let (x_hovered, x_store) = press::hover((x_id.clone(), "hover"), window, cx);
            let (x_focus, x_ring) = press::focus((x_id.clone(), "focus"), window, cx);
            let ink = color(if on {
                ColorRole::TextPrimary
            } else {
                ColorRole::TextSecondary
            });
            let on_select = self.on_select.clone();
            let on_close = self.on_close.clone();
            let (selected, close_selected) = (selected.clone(), selected.clone());
            // `.t3-term-tab { display: flex; align-items: center; gap: spacing-03; height: 100%;
            // padding: 0 spacing-03 0 spacing-04; color: text-secondary; font-size: 0.75rem }`.
            let button = sans(
                gpui_kit::base::Button::new(tab_id)
                    .track_focus(&tab_focus)
                    .accessibility_label(tab.title.clone())
                    // `aria-selected={a === i}`.
                    .aria_selected(on),
                12.,
                400,
            )
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(8.))
            .h_full()
            .pl(rem(12.))
            .pr(rem(8.))
            .text_color(ink)
            .cursor(CursorStyle::PointingHand)
            .on_click(move |_, window, cx| {
                selected.update(cx, |a, cx| {
                    *a = i;
                    cx.notify();
                });
                if let Some(f) = on_select.as_ref() {
                    f(i, window, cx);
                }
            })
            .child(press::icon(CarbonIcon::Terminal, 14., ink).flex_none())
            .child(tab.title.clone())
            .when(tab.running, |this| {
                // `.t3-term-run { width: 6px; height: 6px; border-radius: 50%; background: support-success }`.
                this.child(
                    div()
                        .flex_none()
                        .size(rem(6.))
                        .rounded_full()
                        .bg(color(ColorRole::SupportSuccess)),
                )
            })
            .when(tab_ring, |this| this.child(press::ring()));
            // `.t3-rptab-x { display: inline-flex; align-self: center; margin-right: spacing-03;
            // padding: 2px; border-radius: border-radius-02; color: icon-secondary }`,
            // `:hover { background: background-hover; color: icon-primary }`.
            let x_ink = color(if x_hovered {
                ColorRole::IconPrimary
            } else {
                ColorRole::IconSecondary
            });
            let close = gpui_kit::base::Button::new(x_id)
                .track_focus(&x_focus)
                .accessibility_label((self.texts.close)(&tab.title))
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .mr(rem(8.))
                .p(px(2.))
                .rounded(BORDER_RADIUS_02)
                .when(x_hovered, |this| this.bg(color(ColorRole::BackgroundHover)))
                .cursor(CursorStyle::PointingHand)
                .on_hover(press::track_hover(x_store))
                .when_some(on_close, |this, f| {
                    let selected = close_selected;
                    let slot = slot.clone();
                    let parts = [tab_focus.clone(), x_focus.clone()];
                    this.on_click(move |_, w, cx| {
                        vacate_slot(&slot, i, &parts, w, cx);
                        // `.t3-term-tab--on` must follow the strip: the
                        // neighbor of a closed active tab becomes active.
                        if !controlled {
                            selected.update(cx, |a, cx| {
                                *a = active_after_close(active, i, len);
                                cx.notify();
                            });
                        }
                        f(i, w, cx)
                    })
                })
                .child(press::icon(CarbonIcon::Close, 12., x_ink))
                .when(x_ring, |this| this.child(press::ring()));
            // `.t3-term-tabwrap { display: flex; align-items: center; height: 100%;
            // border-right: 1px solid border-subtle-01 }`; `.t3-term-tab--on { background:
            // background; box-shadow: inset 0 2px 0 border-interactive }`.
            bar.push(
                div()
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h_full()
                    .border_r_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .when(on, |this| this.bg(color(ColorRole::Background)))
                    .child(button)
                    .child(close)
                    .when(on, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .h(px(2.))
                                .bg(color(ColorRole::BorderInteractive)),
                        )
                    })
                    .into_any_element(),
            );
        }
        insert_slot_anchor(&mut bar, (id.clone(), "anchor"), &slot, cx);
        let texts = self.texts.clone();
        bar.push(
            click(
                icon_button(
                    (id.clone(), "new"),
                    texts.new_terminal.clone(),
                    CarbonIcon::Add,
                ),
                self.on_new.clone(),
            )
            .into_any_element(),
        );
        bar.push(div().flex_1().into_any_element());
        bar.extend(self.bar_extra);
        for (key, label, glyph, f) in [
            (
                "split",
                texts.split.clone(),
                CarbonIcon::SidePanelOpen,
                self.on_split.clone(),
            ),
            (
                "chat",
                texts.add_to_chat.clone(),
                CarbonIcon::Chat,
                self.on_add_to_chat.clone(),
            ),
            (
                "max",
                texts.maximize.clone(),
                CarbonIcon::Maximize,
                self.on_maximize.clone(),
            ),
            (
                "hide",
                texts.hide.clone(),
                CarbonIcon::ChevronDown,
                self.on_hide.clone(),
            ),
        ] {
            bar.push(click(icon_button((id.clone(), key), label, glyph), f).into_any_element());
        }

        let ink = color(ColorRole::TerminalText);
        let blink = !motion::reduced_motion(cx);
        // `<pre className="t3-term-view">{lines.join('\n')}<span className="t3-term-cursor">
        // </span></pre>`: one pre-wrapped text run, so a long line wraps at the column edge and
        // the cursor sits inline at the end of the last (wrapped) line.
        let (screen, cursor) = screen_text(&self.lines, self.exited.is_none());
        let mut view: Vec<AnyElement> = Vec::new();
        if let Some(range) = cursor {
            // `.t3-term-cursor { display: inline-block; width: 7px; background: terminal-text;
            // animation: cu-blink 1s steps(1) infinite }` over its space: a highlighted space
            // in the monospace run (one 0.6em advance, 7.2px at 12px) one line tall; steady
            // under reduced motion (blink stops).
            let text = move |on: bool| {
                StyledText::new(screen.clone()).with_highlights(on.then(|| {
                    (
                        range.clone(),
                        HighlightStyle {
                            background_color: Some(ink),
                            ..Default::default()
                        },
                    )
                }))
            };
            view.push(if blink {
                text(true)
                    .with_animation(
                        (id.clone(), "cursor"),
                        Animation::new(BLINK_PERIOD).repeat(),
                        move |_, t| text(blink_opacity(t) > 0.),
                    )
                    .into_any_element()
            } else {
                text(true).into_any_element()
            });
        } else if !self.lines.is_empty() {
            view.push(StyledText::new(screen).into_any_element());
        }
        if let Some(code) = self.exited {
            // `.t3-term-exit { color: terminal-text; opacity: .7 }` (after a newline).
            view.push(
                div()
                    .opacity(0.7)
                    .child((texts.exited)(code))
                    .into_any_element(),
            );
        }
        // `.t3-term { display: flex; flex-direction: column; height: 100%; min-height: 12rem;
        // background: terminal-background; color: terminal-text }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .size_full()
            .min_h(rem(192.))
            .bg(color(ColorRole::TerminalBackground))
            .text_color(ink)
            .child(
                // `.t3-term-bar { display: flex; align-items: center; height: size-sm;
                // background: layer-01; color: text-primary; border-bottom: 1px solid border-subtle-01 }`.
                div()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .items_center()
                    .h(rem(32.))
                    .bg(color(ColorRole::Layer01))
                    .text_color(color(ColorRole::TextPrimary))
                    .border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .children(bar),
            )
            .child(match self.viewport {
                Some(viewport) => viewport,
                None => {
                    // `.t3-term-view { flex: 1; padding: spacing-03 spacing-04; overflow: auto;
                    // font-family: mono; font-size: 0.75rem; line-height: 1.33333; white-space: pre-wrap }`.
                    // `white-space: pre-wrap` never overflows sideways: vertical scroll only.
                    ScrollArea::new((id.clone(), "view"))
                        .axes(ScrollAxes::Vertical)
                        .flex_1()
                        .min_h_0()
                        .child(
                            mono(div(), 12., 400)
                                .id((id.clone(), "screen"))
                                .test_support()
                                .w_full()
                                .line_height(relative(1.33333))
                                .whitespace_normal()
                                .py(rem(8.))
                                .px(rem(12.))
                                .children(view),
                        )
                        .into_any_element()
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_blinks_in_one_step() {
        assert_eq!(blink_opacity(0.), 1.);
        assert_eq!(blink_opacity(0.49), 1.);
        assert_eq!(blink_opacity(0.5), 0.);
        assert_eq!(blink_opacity(0.99), 0.);
    }

    #[test]
    fn screen_text_joins_lines_and_ends_in_the_cursor() {
        let lines: Vec<SharedString> = vec!["$ dev".into(), "".into(), "  ok".into()];
        let (text, cursor) = screen_text(&lines, true);
        assert_eq!(text.as_ref(), "$ dev\n\n  ok ");
        assert_eq!(cursor, Some(11..12));
        let (text, cursor) = screen_text(&lines, false);
        assert_eq!(text.as_ref(), "$ dev\n\n  ok");
        assert_eq!(cursor, None);
        assert_eq!(screen_text(&[], true), (" ".into(), Some(0..1)));
    }

    #[test]
    fn default_texts_match_the_reference() {
        let t = TerminalTexts::default();
        assert_eq!((t.close)("dev").as_ref(), "Close dev");
        assert_eq!((t.exited)(1).as_ref(), "Process exited with code 1");
    }
}
