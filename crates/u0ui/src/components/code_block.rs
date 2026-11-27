//! CodeBlock (`.cu-code*`: Carbon `CodeSnippet` multi anatomy with a header),
//! Carbon `CodeSnippet` (`.cds--snippet--single/multi/inline`) and the
//! `CopyButton` (`.cds--copy-btn`) they share.
//!
//! - **CodeBlock**: `layer-01`, IBM Plex Mono. An optional header (`size-md`
//!   tall, 1rem start padding, 1px `border-subtle-01` rule) holds the title
//!   (14px semibold), the language (`label-01`, `text-secondary`) and a copy
//!   button; without a header the copy button floats at the top end. Lines are
//!   14/20 (`sm`: 12/16) with 1rem side padding inside a 1rem (sm 0.5rem)
//!   vertical pad; optional line numbers (2.5rem, right-aligned, `text-helper`)
//!   and a 1rem diff mark column (`+` in `support-success` on
//!   `diff-added-background`, `−` in `support-error` on
//!   `diff-removed-background`, `highlight` rows). `destructive` adds a 3px
//!   `support-error` inset bar at the start; `wrap` wraps long lines, else the
//!   code scrolls horizontally; an optional footer (`label-01`, top rule).
//! - **CodeSnippet**: `single` is a 2.5rem `layer` row in `code-01` with a
//!   40px copy button at the end and a 2rem fade before it; `multi` pads 1rem,
//!   shows at most 15 rows (16px each) until "show more" expands it (height
//!   `transition: max-height 150ms`, chevron 150ms / 300ms open), copy button
//!   2rem at 0.5rem from the top end; `inline` is a 1.25rem `layer` chip
//!   (radius 4, 0 0.5rem padding) that copies on click (`layer-hover` /
//!   `layer-active`, 1px `focus` border).
//! - **CopyButton**: `layer` square, hover `layer-hover`, active
//!   `layer-active`, `Copy` icon in `icon-primary`, 2px inset `focus`. On press
//!   it writes the text to the clipboard and shows the caller's feedback
//!   ("Copied!") in an inverse tooltip box for `feedbackTimeout` 2000ms, fading
//!   in and out over 110ms (`cds--show-feedback` / `cds--hide-feedback`,
//!   `cubic-bezier(0.2, 0, 0.38, 0.9)`; opacity only, so reduced motion keeps it).
//! - **Behavior**: Base `Button` (focus, Enter/Space, pointer) and GPUI's
//!   clipboard. Kit's `Clipboard` button has no Carbon feedback, so it is drawn here.
//! - **Snippet focus**: the code area (single: `.cds--snippet-container`;
//!   multi: its `pre`) is a horizontal [`ScrollArea`] with Carbon's 8px bar
//!   and a tab stop before the copy button (`tabIndex={0}` unless disabled,
//!   Carbon's DOM order: code, copy, "Show more"). A press focuses it,
//!   Left/Right scroll it, a vertical wheel scrolls the page. Single draws
//!   `:focus { outline: 2px solid focus; outline-offset: -2px }` and drops its
//!   end fade on `:focus-within`; multi draws the container's `:focus-within`
//!   outline at offset 0. "Show more" draws `:focus { outline: 2px solid
//!   focus; outline-offset: -2px }`.
use super::form::rem;
use super::overlay::floating_shadow;
use super::scroll_area::{ScrollArea, ScrollAxes, ScrollFocusRing};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{ColorRole, FontFamilyKey, TypeStyle},
    typography::{TypeStyled as _, font},
};
use gpui_kit::{
    AnyElement, App, ClipboardItem, CursorStyle, ElementId, FocusHandle, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled, Window, deferred, div, prelude::FluentBuilder as _,
    px, relative,
};
use std::time::{Duration, Instant};

/// `feedbackTimeout` default (Carbon `Copy`): 2000ms.
pub const FEEDBACK_TIMEOUT: Duration = Duration::from_millis(2000);
/// `cds--show-feedback` / `cds--hide-feedback`: 110ms productive standard.
pub const FEEDBACK_FADE: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// `.cds--snippet .cds--snippet__icon { transition: all 70ms … }` and button backgrounds.
const HOVER: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);
/// Carbon `CopyButton`'s `align = 'bottom'` default: its feedback shows below.
pub const COPY_FEEDBACK_BELOW: bool = true;
/// Carbon `CodeSnippet` `maxCollapsedNumberOfRows`.
pub const MAX_COLLAPSED_ROWS: usize = 15;
/// `code-01` line height in px (rows are 16px).
pub const ROW: f32 = 16.;

/// Feedback opacity `elapsed` after a copy: fade in over 110ms, hold until
/// 2000ms, fade out over 110ms; `None` once hidden.
pub fn feedback_opacity(elapsed: Duration) -> Option<f32> {
    let fade = FEEDBACK_FADE.speed.duration().as_secs_f32();
    let t = elapsed.as_secs_f32();
    let hold = FEEDBACK_TIMEOUT.as_secs_f32();
    let ease = |x: f32| FEEDBACK_FADE.curve.ease(x);
    if t < fade {
        Some(ease(t / fade))
    } else if t < hold {
        Some(1.)
    } else if t < hold + fade {
        Some(1. - ease((t - hold) / fade))
    } else {
        None
    }
}

/// Copy-button / inline-snippet background: `layer`, hover `layer-hover`,
/// active `layer-active` (`.cds--snippet .cds--copy-btn`).
pub fn button_background(level: LayerLevel, hovered: bool, pressed: bool) -> Hsla {
    color(if pressed {
        level.layer_active()
    } else if hovered {
        level.layer_hover()
    } else {
        level.layer()
    })
}

#[derive(Default)]
struct CopyState {
    hovered: bool,
    pressed: bool,
    copied_at: Option<Instant>,
}

/// The inverse feedback box (`.cds--copy-btn__feedback`: `background-inverse`,
/// `text-inverse`, `body-compact-01`, padding 0.1875rem 1rem, radius 2px, the
/// floating shadow), centered on the anchor.
fn feedback_box(text: SharedString, opacity: f32, below: bool) -> impl IntoElement {
    let bubble = div()
        .flex_none()
        .px(rem(16.))
        .py(rem(3.))
        .rounded(px(2.))
        .bg(color(ColorRole::BackgroundInverse))
        .text_color(color(ColorRole::TextInverse))
        .type_style(TypeStyle::BodyCompact01)
        .whitespace_nowrap()
        .shadow(floating_shadow())
        .opacity(opacity)
        .child(text);
    // A zero-width column centered on the anchor; the bubble overflows it both ways.
    let anchor = div()
        .absolute()
        .left(relative(0.5))
        .w(px(0.))
        .flex()
        .flex_col()
        .items_center()
        .map(|this| {
            if below {
                this.top(relative(1.)).pt(rem(4.))
            } else {
                this.bottom(relative(1.)).pb(rem(4.))
            }
        })
        .child(bubble);
    deferred(anchor).with_priority(6000)
}

/// A focus handle kept under `(id, key)`.
fn keyed_focus(
    id: &ElementId,
    key: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    window
        .use_keyed_state((id.clone(), key), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone()
}

/// Keyed copy state plus the current feedback opacity.
fn copy_state(
    id: &ElementId,
    window: &mut Window,
    cx: &mut App,
) -> (gpui_kit::Entity<CopyState>, Option<f32>) {
    let state = window.use_keyed_state((id.clone(), "copy"), cx, |_, _| CopyState::default());
    let opacity = state
        .read(cx)
        .copied_at
        .and_then(|at| feedback_opacity(at.elapsed()));
    if opacity.is_some() {
        window.request_animation_frame();
    }
    (state, opacity)
}

fn press_handlers(
    el: gpui_kit::Stateful<gpui_kit::Div>,
    state: &gpui_kit::Entity<CopyState>,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let (a, b, c) = (state.clone(), state.clone(), state.clone());
    el.on_hover(move |h, _, cx| {
        a.update(cx, |s, cx| {
            s.hovered = *h;
            if !*h {
                s.pressed = false;
            }
            cx.notify();
        })
    })
    .on_mouse_down(gpui_kit::MouseButton::Left, move |_, _, cx| {
        b.update(cx, |s, cx| {
            s.pressed = true;
            cx.notify();
        })
    })
    .on_mouse_up(gpui_kit::MouseButton::Left, move |_, _, cx| {
        c.update(cx, |s, cx| {
            s.pressed = false;
            cx.notify();
        })
    })
}

/// A Carbon copy button. See the module docs.
#[derive(IntoElement)]
pub struct CopyButton {
    id: ElementId,
    text: SharedString,
    label: SharedString,
    feedback: SharedString,
    size: f32,
    level: LayerLevel,
    disabled: bool,
    quiet: bool,
    focus: Option<FocusHandle>,
}

impl CopyButton {
    /// A 40px copy button that copies `text`.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            label: SharedString::default(),
            feedback: SharedString::default(),
            size: 40.,
            level: LayerLevel::One,
            disabled: false,
            quiet: false,
            focus: None,
        }
    }

    /// Use a caller-owned focus handle (a snippet reads it for its
    /// `:focus-within` rules).
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }

    /// No fill at rest, `background-hover` under the pointer, for a copy
    /// button set into a pattern's own surface (`.t3-copy.cds--copy-btn {
    /// background: transparent }`, `:hover { background: background-hover }`).
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Accessible name (`iconDescription`, e.g. "Copy to clipboard").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The feedback shown after copying (`feedback`, e.g. "Copied!").
    pub fn feedback(mut self, feedback: impl Into<SharedString>) -> Self {
        self.feedback = feedback.into();
        self
    }

    /// The square's side in px (40 default; 32 in a multi-line snippet).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// The layer it sits on (paints `layer`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Disabled (`.cds--snippet--disabled .cds--copy-btn`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for CopyButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (state, opacity) = copy_state(&id, window, cx);
        let focus = self
            .focus
            .clone()
            .unwrap_or_else(|| keyed_focus(&id, "focus", window, cx));
        let focused = focus.is_focused(window);
        let (hovered, pressed) = {
            let s = state.read(cx);
            (s.hovered && !self.disabled, s.pressed && !self.disabled)
        };
        let bg = if self.quiet && !pressed {
            if hovered && !self.disabled {
                color(ColorRole::BackgroundHover)
            } else {
                gpui_kit::transparent_black()
            }
        } else if self.disabled {
            color(self.level.layer())
        } else {
            button_background(self.level, hovered, pressed)
        };
        let bg = motion::animate_color((id.clone(), "bg"), bg, HOVER, window, cx);
        let text = self.text.clone();
        let on_copy = state.clone();
        let button = gpui_kit::base::Button::new(id.clone())
            .track_focus(&focus)
            .disabled(self.disabled)
            .accessibility_label(self.label.clone())
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(rem(self.size))
            .bg(bg)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
                on_copy.update(cx, |s, cx| {
                    s.copied_at = Some(Instant::now());
                    cx.notify();
                });
            })
            .child(
                Icon::carbon(CarbonIcon::Copy)
                    .size(IconSize::S16)
                    .color(color(if self.disabled {
                        ColorRole::IconDisabled
                    } else {
                        ColorRole::IconPrimary
                    })),
            )
            .when(focused, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            })
            // Carbon `CopyButton` defaults to `align="bottom"`: the feedback
            // shows below the button.
            .when_some(opacity, |this, opacity| {
                this.child(feedback_box(
                    self.feedback.clone(),
                    opacity,
                    COPY_FEEDBACK_BELOW,
                ))
            });
        press_handlers(div().id((id, "press")), &state)
            .flex_none()
            .child(button)
    }
}

/// `index.d.ts` `CodeBlockProps.lines[].variant`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineVariant {
    /// `+` on `diff-added-background`.
    Added,
    /// `−` on `diff-removed-background`.
    Removed,
    /// `highlight` background.
    Highlighted,
}

/// One code line.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeLine {
    /// The text.
    pub text: SharedString,
    /// Its diff/highlight variant.
    pub variant: Option<LineVariant>,
}

impl CodeLine {
    /// A plain line.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            variant: None,
        }
    }

    /// An added line.
    pub fn added(text: impl Into<SharedString>) -> Self {
        Self {
            variant: Some(LineVariant::Added),
            ..Self::new(text)
        }
    }

    /// A removed line.
    pub fn removed(text: impl Into<SharedString>) -> Self {
        Self {
            variant: Some(LineVariant::Removed),
            ..Self::new(text)
        }
    }

    /// A highlighted line.
    pub fn highlighted(text: impl Into<SharedString>) -> Self {
        Self {
            variant: Some(LineVariant::Highlighted),
            ..Self::new(text)
        }
    }
}

/// `code` split as `index.jsx` does: one trailing newline dropped, then lines.
pub fn split_code(code: &str) -> Vec<CodeLine> {
    code.strip_suffix('\n')
        .unwrap_or(code)
        .split('\n')
        .map(|l| CodeLine::new(l.to_string()))
        .collect()
}

/// The row background and mark color of a line variant.
pub fn line_colors(variant: Option<LineVariant>) -> (Option<Hsla>, Hsla) {
    use ColorRole::*;
    match variant {
        // `.cu-code-line--added { background: diff-added-background } … .cu-code-mark { color: support-success }`.
        Some(LineVariant::Added) => (Some(color(DiffAddedBackground)), color(SupportSuccess)),
        Some(LineVariant::Removed) => (Some(color(DiffRemovedBackground)), color(SupportError)),
        // `.cu-code-line--highlighted { background: var(--highlight) }`.
        Some(LineVariant::Highlighted) => (Some(color(Highlight)), color(TextSecondary)),
        None => (None, color(TextSecondary)),
    }
}

/// `index.d.ts` `CodeBlockProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CodeBlockSize {
    /// 14/20 text, 1rem vertical padding.
    #[default]
    Default,
    /// 12/16 text, 0.5rem vertical padding (`.cu-code--sm`).
    Sm,
}

impl CodeBlockSize {
    /// Both sizes.
    pub const ALL: [CodeBlockSize; 2] = [Self::Default, Self::Sm];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
        }
    }

    /// (font size, line height, vertical padding) in px: `.cu-code-pre {
    /// font-size: 0.875rem; line-height: 1.42857; padding: spacing-05 0 }`,
    /// `.cu-code--sm .cu-code-pre { 0.75rem; 1.33333; spacing-03 0 }`.
    pub const fn metrics(self) -> (f32, f32, f32) {
        match self {
            Self::Default => (14., 20., 16.),
            Self::Sm => (12., 16., 8.),
        }
    }
}

/// `index.d.ts` `CodeBlockProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CodeBlockVariant {
    /// Plain.
    #[default]
    Default,
    /// `box-shadow: inset 3px 0 0 var(--support-error)`.
    Destructive,
}

impl CodeBlockVariant {
    /// Both variants.
    pub const ALL: [CodeBlockVariant; 2] = [Self::Default, Self::Destructive];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
        }
    }
}

/// A code block. See the module docs.
#[derive(IntoElement)]
pub struct CodeBlock {
    id: ElementId,
    title: Option<SharedString>,
    language: Option<SharedString>,
    lines: Vec<CodeLine>,
    line_numbers: bool,
    start_line: usize,
    wrap: bool,
    size: CodeBlockSize,
    variant: CodeBlockVariant,
    footer: Option<AnyElement>,
    copy_label: SharedString,
    copy_feedback: SharedString,
}

impl CodeBlock {
    /// A code block with a stable `id` showing `code` (`code` prop).
    pub fn new(id: impl Into<ElementId>, code: &str) -> Self {
        Self {
            id: id.into(),
            title: None,
            language: None,
            lines: split_code(code),
            line_numbers: false,
            start_line: 1,
            wrap: false,
            size: CodeBlockSize::Default,
            variant: CodeBlockVariant::Default,
            footer: None,
            copy_label: SharedString::default(),
            copy_feedback: SharedString::default(),
        }
    }

    /// Replace the code with explicit `lines` (with diff variants).
    pub fn lines(mut self, lines: impl IntoIterator<Item = CodeLine>) -> Self {
        self.lines = lines.into_iter().collect();
        self
    }

    /// Header title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Header language label.
    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Show line numbers.
    pub fn line_numbers(mut self, on: bool) -> Self {
        self.line_numbers = on;
        self
    }

    /// The first line number (default 1).
    pub fn start_line(mut self, start: usize) -> Self {
        self.start_line = start;
        self
    }

    /// Wrap long lines instead of scrolling.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// `default` or `sm`.
    pub fn size(mut self, size: CodeBlockSize) -> Self {
        self.size = size;
        self
    }

    /// `default` or `destructive`.
    pub fn variant(mut self, variant: CodeBlockVariant) -> Self {
        self.variant = variant;
        self
    }

    /// A footer under the code.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// The copy button's accessible name and feedback ("Copy to clipboard", "Copied!").
    pub fn copy_texts(
        mut self,
        label: impl Into<SharedString>,
        feedback: impl Into<SharedString>,
    ) -> Self {
        self.copy_label = label.into();
        self.copy_feedback = feedback.into();
        self
    }

    /// The text the copy button writes: the lines joined by `\n`.
    pub fn raw(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text.as_ref())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl RenderOnce for CodeBlock {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let raw = self.raw();
        let (font_size, line_height, pad_y) = self.size.metrics();
        let has_header = self.title.is_some() || self.language.is_some();
        let marks = self.line_numbers || self.lines.iter().any(|l| l.variant.is_some());
        let copy = CopyButton::new((self.id.clone(), "copy"), raw.clone())
            .label(self.copy_label.clone())
            .feedback(self.copy_feedback.clone())
            .layer(LayerLevel::One);
        let rule = color(ColorRole::BorderSubtle01);
        let mono = font(FontFamilyKey::Mono, 400);

        let header = has_header.then(|| {
            // `.cu-code-header { height: var(--size-md); padding-left: spacing-05;
            // border-bottom: 1px solid border-subtle-01; gap: spacing-03 }`.
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(8.))
                .h(rem(40.))
                .pl(rem(16.))
                .border_b_1()
                .border_color(rule)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .font(font(FontFamilyKey::Sans, 600))
                        .text_size(rem(14.))
                        .line_height(rem(18.))
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .children(self.title.clone()),
                )
                .when_some(self.language.clone(), |this, lang| {
                    this.child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(lang),
                    )
                })
                .child(copy)
        });
        let start = self.start_line;
        let line_numbers = self.line_numbers;
        let wrap = self.wrap;
        let rows = self.lines.into_iter().enumerate().map(move |(i, line)| {
            let (bg, mark_color) = line_colors(line.variant);
            let mark = match line.variant {
                Some(LineVariant::Added) => "+",
                Some(LineVariant::Removed) => "−",
                _ => "",
            };
            // `.cu-code-line { display: flex; padding: 0 var(--spacing-05) }`.
            div()
                .flex()
                .flex_row()
                .px(rem(16.))
                .when(!wrap, |this| this.min_w_full().flex_none())
                .when_some(bg, |this, bg| this.bg(bg))
                .when(line_numbers, |this| {
                    // `.cu-code-ln { width: 2.5rem; padding-right: spacing-04; text-align: right; color: text-helper }`.
                    this.child(
                        div()
                            .flex_none()
                            .w(rem(40.))
                            .pr(rem(12.))
                            .flex()
                            .justify_end()
                            .text_color(color(ColorRole::TextHelper))
                            .child(SharedString::from((start + i).to_string())),
                    )
                })
                .when(marks, |this| {
                    // `.cu-code-mark { width: 1rem; color: var(--text-secondary) }`.
                    this.child(
                        div()
                            .flex_none()
                            .w(rem(16.))
                            .text_color(mark_color)
                            .child(mark),
                    )
                })
                .child(
                    div()
                        .when(wrap, |this| this.flex_1().min_w_0())
                        .when(!wrap, |this| this.whitespace_nowrap())
                        .child(if line.text.is_empty() {
                            SharedString::from(" ")
                        } else {
                            line.text
                        }),
                )
        });
        // `.cu-code-pre { overflow-x: auto }`: a horizontal Carbon scroller.
        let pre = code_scroller((self.id.clone(), "pre")).child(
            div()
                .flex()
                .flex_col()
                .py(rem(pad_y))
                .font(mono)
                .text_size(rem(font_size))
                .line_height(rem(line_height))
                .children(rows),
        );

        div()
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .text_color(color(ColorRole::TextPrimary))
            .children(header)
            .child(pre)
            .when(!has_header, |this| {
                // `.cu-code-float { position: absolute; top: 0; right: 0 }`.
                this.child(
                    div().absolute().top_0().right_0().child(
                        CopyButton::new((self.id.clone(), "copy-float"), raw)
                            .label(self.copy_label.clone())
                            .feedback(self.copy_feedback.clone()),
                    ),
                )
            })
            .when_some(self.footer, |this, footer| {
                // `.cu-code-footer { padding: spacing-03 spacing-05; border-top: 1px solid
                // border-subtle-01; font-size: 0.75rem; color: text-secondary }`.
                this.child(
                    div()
                        .px(rem(16.))
                        .py(rem(8.))
                        .border_t_1()
                        .border_color(rule)
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(footer),
                )
            })
            .when(self.variant == CodeBlockVariant::Destructive, |this| {
                // `.cu-code--destructive { box-shadow: inset 3px 0 0 var(--support-error) }`.
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .w(px(3.))
                        .bg(color(ColorRole::SupportError)),
                )
            })
    }
}

/// Carbon `CodeSnippet` types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SnippetType {
    /// One line, 2.5rem tall.
    #[default]
    Single,
    /// Many lines, collapsible after 15 rows.
    Multi,
    /// Inline in text.
    Inline,
}

impl SnippetType {
    /// Every type.
    pub const ALL: [SnippetType; 3] = [Self::Single, Self::Multi, Self::Inline];

    /// Carbon's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multi => "multi",
            Self::Inline => "inline",
        }
    }
}

/// The collapsed height of a multi snippet in px (`maxCollapsedNumberOfRows × 16`).
pub fn collapsed_height(rows: usize) -> Option<f32> {
    (rows > MAX_COLLAPSED_ROWS).then_some(MAX_COLLAPSED_ROWS as f32 * ROW)
}

/// The multi snippet's `.cds--snippet-container` height in px. Collapsed, its
/// `max-height` is `maxCollapsedNumberOfRows × 16` = 240 and clips everything
/// past 15 rows (the `pre`'s padding included); otherwise it holds the rows
/// plus `pre { padding-block-end: 1.5rem }`, and expanded adds
/// `.cds--snippet--expand .cds--snippet-container { padding-block-end: 1rem }`.
pub fn container_height(rows: usize, expanded: bool) -> f32 {
    let natural = rows as f32 * ROW + 24.;
    match collapsed_height(rows) {
        Some(c) if !expanded => c,
        Some(_) => natural + 16.,
        None => natural,
    }
}

/// Carbon's `CodeSnippet`. See the module docs.
#[derive(IntoElement)]
pub struct CodeSnippet {
    id: ElementId,
    kind: SnippetType,
    code: SharedString,
    level: LayerLevel,
    disabled: bool,
    wrap: bool,
    copy_label: SharedString,
    copy_feedback: SharedString,
    show_more: SharedString,
    show_less: SharedString,
    focus: Option<FocusHandle>,
    scroll: Option<ScrollHandle>,
}

impl CodeSnippet {
    /// A snippet of `kind` showing `code`.
    pub fn new(id: impl Into<ElementId>, kind: SnippetType, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind,
            code: code.into(),
            level: LayerLevel::One,
            disabled: false,
            wrap: false,
            copy_label: SharedString::default(),
            copy_feedback: SharedString::default(),
            show_more: SharedString::default(),
            show_less: SharedString::default(),
            focus: None,
            scroll: None,
        }
    }

    /// Use a caller-owned focus handle for the code area (single: the
    /// `.cds--snippet-container`; multi: its `pre`), e.g. to focus it.
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }

    /// Use a caller-owned scroll handle for the code area's horizontal
    /// scroller (single and multi), e.g. to read or set its offset.
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }

    /// The layer it sits on (paints `layer`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Disabled (`.cds--snippet--disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Wrap text (multi; `.cds--snippet--wraptext`).
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Copy button name and feedback ("Copy to clipboard", "Copied!").
    pub fn copy_texts(
        mut self,
        label: impl Into<SharedString>,
        feedback: impl Into<SharedString>,
    ) -> Self {
        self.copy_label = label.into();
        self.copy_feedback = feedback.into();
        self
    }

    /// Expand button texts ("Show more", "Show less").
    pub fn expand_texts(
        mut self,
        more: impl Into<SharedString>,
        less: impl Into<SharedString>,
    ) -> Self {
        self.show_more = more.into();
        self.show_less = less.into();
        self
    }
}

fn code_01() -> (gpui_kit::Font, gpui_kit::Rems, gpui_kit::Rems) {
    // `code-01`: 12/16 IBM Plex Mono.
    (font(FontFamilyKey::Mono, 400), rem(12.), rem(16.))
}

impl RenderOnce for CodeSnippet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (mono, size, lh) = code_01();
        let text_color = color(if self.disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextPrimary
        });
        let layer = color(self.level.layer());
        match self.kind {
            SnippetType::Inline => {
                let (state, opacity) = copy_state(&id, window, cx);
                let focus = window
                    .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                let focused = focus.is_focused(window);
                let (h, p) = {
                    let s = state.read(cx);
                    (s.hovered && !self.disabled, s.pressed && !self.disabled)
                };
                let bg = motion::animate_color(
                    (id.clone(), "bg"),
                    button_background(self.level, h, p),
                    HOVER,
                    window,
                    cx,
                );
                let code = self.code.clone();
                let on_copy = state.clone();
                let chip = gpui_kit::base::Button::new(id.clone())
                    .track_focus(&focus)
                    .disabled(self.disabled)
                    .accessibility_label(self.copy_label.clone())
                    .relative()
                    .flex()
                    .items_center()
                    .h(rem(20.))
                    .px(rem(8.))
                    .rounded(px(4.))
                    .border_1()
                    // `.cds--snippet--inline:focus { border: 1px solid var(--focus) }`.
                    .border_color(if focused {
                        color(ColorRole::Focus)
                    } else {
                        Hsla::transparent_black()
                    })
                    .bg(bg)
                    .text_color(text_color)
                    .font(mono)
                    .text_size(size)
                    .line_height(lh)
                    .whitespace_nowrap()
                    .cursor(CursorStyle::PointingHand)
                    .on_click(move |_, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                        on_copy.update(cx, |s, cx| {
                            s.copied_at = Some(Instant::now());
                            cx.notify();
                        });
                    })
                    .child(self.code.clone())
                    // `.cds--snippet--inline .cds--btn--copy__feedback { inset-block-start: calc(100% - 0.25rem) }`.
                    .when_some(opacity, |this, o| {
                        this.child(feedback_box(self.copy_feedback.clone(), o, true))
                    });
                press_handlers(div().id((id, "press")), &state)
                    .flex_none()
                    .child(chip)
                    .into_any_element()
            }
            SnippetType::Single => {
                let focus = self
                    .focus
                    .clone()
                    .unwrap_or_else(|| keyed_focus(&id, "code-focus", window, cx));
                let copy_focus = keyed_focus(&id, "copy-focus", window, cx);
                let focus_within = focus.is_focused(window) || copy_focus.is_focused(window);
                // `.cds--snippet--single .cds--snippet-container { display: flex;
                // align-items: center; block-size: 100%; overflow-x: auto;
                // padding-inline-start: 1rem }`: the code area is a Carbon
                // horizontal scroller, a tab stop (`tabIndex={0}` unless
                // disabled) before the copy button, scrolled by Left/Right.
                // A vertical wheel passes through to the page.
                let container = code_scroller((id.clone(), "scroll"))
                    .focusable(!self.disabled)
                    .track_focus(&focus)
                    // `.cds--snippet--single .cds--snippet-container:focus
                    // { outline: 2px solid var(--focus); outline-offset: -2px }`.
                    .focus_ring(ScrollFocusRing::Focus)
                    .when_some(self.scroll.clone(), |this, handle| {
                        this.track_scroll(&handle)
                    })
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .pl(rem(16.))
                            // `.cds--snippet--single pre { padding-inline-end: 2rem }`.
                            .pr(rem(32.))
                            .font(mono)
                            .text_size(size)
                            .line_height(lh)
                            .whitespace_nowrap()
                            .child(self.code.clone()),
                    );
                // `.cds--snippet--single { block-size: 2.5rem; padding-inline-end: 2.5rem;
                // max-inline-size: 48rem; background: layer }`.
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .w_full()
                    .max_w(rem(768.))
                    .h(rem(40.))
                    .pr(rem(40.))
                    .bg(layer)
                    .text_color(text_color)
                    .child(container)
                    // `.cds--snippet--single:focus-within .cds--snippet-container
                    // { mask-image: none }`.
                    .when(!focus_within, |this| this.child(fade_end(layer)))
                    .child(
                        div().absolute().top_0().right_0().child(
                            CopyButton::new((id.clone(), "copy"), self.code.clone())
                                .track_focus(&copy_focus)
                                .label(self.copy_label.clone())
                                .feedback(self.copy_feedback.clone())
                                .layer(self.level)
                                .disabled(self.disabled),
                        ),
                    )
                    .into_any_element()
            }
            SnippetType::Multi => {
                let lines: Vec<SharedString> = self
                    .code
                    .split('\n')
                    .map(|l| SharedString::from(l.to_string()))
                    .collect();
                let collapsed = collapsed_height(lines.len());
                let expanded_state =
                    window.use_keyed_state((id.clone(), "expanded"), cx, |_, _| false);
                let expanded = *expanded_state.read(cx);
                // `.cds--snippet-container { transition: max-height 150ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
                let height = motion::animate(
                    (id.clone(), "height"),
                    container_height(lines.len(), expanded),
                    Motion::new(Curve::ProductiveStandard, Speed::Moderate01).spatial(true),
                    window,
                    cx,
                );
                let rotation = motion::animate(
                    (id.clone(), "chevron"),
                    if expanded { std::f32::consts::PI } else { 0. },
                    // `.cds--snippet--expand … { transition: transform 300ms }`, closing 150ms.
                    if expanded {
                        Motion::new(Curve::ProductiveStandard, Speed::Moderate02).spatial(true)
                    } else {
                        Motion::new(Curve::ProductiveStandard, Speed::Moderate01).spatial(true)
                    },
                    window,
                    cx,
                );
                let wrap = self.wrap;
                let disabled = self.disabled;
                let focus = self
                    .focus
                    .clone()
                    .unwrap_or_else(|| keyed_focus(&id, "code-focus", window, cx));
                let code_focused = focus.is_focused(window);
                let expand_button = collapsed.map(|_| {
                    let expand_focus = keyed_focus(&id, "expand-focus", window, cx);
                    let expand_focused = expand_focus.is_focused(window);
                    let hovered =
                        window.use_keyed_state((id.clone(), "expand-hover"), cx, |_, _| false);
                    let bg = motion::animate_color(
                        (id.clone(), "expand-bg"),
                        button_background(self.level, *hovered.read(cx), false),
                        HOVER,
                        window,
                        cx,
                    );
                    let toggle = expanded_state.clone();
                    // `.cds--snippet-btn--expand { position: absolute; inset: auto 0 0 auto;
                    // padding: 0.5rem 1rem; block-size: 2rem; body-compact-01; background: layer }`.
                    // Base `Button`: a tab stop after the copy button (DOM
                    // order), toggled by click, Enter and Space.
                    gpui_kit::base::Button::new((id.clone(), "expand"))
                        .track_focus(&expand_focus)
                        .disabled(disabled)
                        .absolute()
                        .bottom_0()
                        .right_0()
                        .flex()
                        .items_center()
                        .h(rem(32.))
                        .px(rem(16.))
                        .bg(bg)
                        .text_color(color(ColorRole::TextPrimary))
                        .type_style(TypeStyle::BodyCompact01)
                        .cursor(CursorStyle::PointingHand)
                        .on_hover(move |h, _, cx| {
                            hovered.update(cx, |v, cx| {
                                *v = *h;
                                cx.notify();
                            })
                        })
                        .on_click(move |_, _, cx| {
                            toggle.update(cx, |v, cx| {
                                *v = !*v;
                                cx.notify();
                            })
                        })
                        .child(if expanded {
                            self.show_less.clone()
                        } else {
                            self.show_more.clone()
                        })
                        .child(
                            div().ml(rem(8.)).child(
                                Icon::carbon(CarbonIcon::ChevronDown)
                                    .size(IconSize::S16)
                                    .color(color(ColorRole::IconPrimary))
                                    .rotate(gpui_kit::radians(rotation)),
                            ),
                        )
                        // `.cds--snippet-btn--expand:focus { outline: 2px solid
                        // var(--focus); outline-offset: -2px }` (over the ghost
                        // button's inset focus shadows, which it covers).
                        .when(expand_focused, |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .size_full()
                                    .border_2()
                                    .border_color(color(ColorRole::Focus)),
                            )
                        })
                });
                // `.cds--snippet--multi { padding: 1rem; display: flex }`; `pre { padding-block-end:
                // 1.5rem; padding-inline-end: 1.5rem }`; expanded container `padding-block-end: 1rem`.
                div()
                    .relative()
                    .w_full()
                    .max_w(rem(768.))
                    .p(rem(16.))
                    .bg(layer)
                    .text_color(text_color)
                    .child(
                        // `.cds--snippet--multi .cds--snippet-container:focus-within
                        // { outline: 2px solid var(--focus); outline-offset: 0 }`:
                        // drawn just outside the container, which clips.
                        div()
                            .relative()
                            .w_full()
                            .child(
                                // `.cds--snippet-container { max-height; overflow-y: auto }`
                                // clips the rows; its `pre { overflow: auto }` scrolls
                                // them sideways under a Carbon bar. The `pre` is the tab
                                // stop (`tabIndex={0}` unless disabled), before the copy
                                // button; Left/Right scroll it.
                                div()
                                    .w_full()
                                    .h(rem(height))
                                    .overflow_hidden()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        code_scroller((id.clone(), "scroll"))
                                            .focusable(!disabled)
                                            .track_focus(&focus)
                                            .focus_ring(ScrollFocusRing::Hidden)
                                            .when_some(self.scroll.clone(), |this, handle| {
                                                this.track_scroll(&handle)
                                            })
                                            .h_full()
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .pr(rem(24.))
                                                    .font(mono)
                                                    .text_size(size)
                                                    .line_height(lh)
                                                    .children(lines.into_iter().map(move |l| {
                                                        div()
                                                            .when(!wrap, |this| {
                                                                this.whitespace_nowrap()
                                                            })
                                                            .child(if l.is_empty() {
                                                                SharedString::from(" ")
                                                            } else {
                                                                l
                                                            })
                                                    })),
                                            ),
                                    ),
                            )
                            .when(code_focused, |this| {
                                this.child(
                                    div()
                                        .absolute()
                                        .top(px(-2.))
                                        .left(px(-2.))
                                        .right(px(-2.))
                                        .bottom(px(-2.))
                                        .border_2()
                                        .border_color(color(ColorRole::Focus)),
                                )
                            }),
                    )
                    .child(
                        div().absolute().top(rem(8.)).right(rem(8.)).child(
                            CopyButton::new((id.clone(), "copy"), self.code.clone())
                                .size(32.)
                                .label(self.copy_label.clone())
                                .feedback(self.copy_feedback.clone())
                                .layer(self.level)
                                .disabled(self.disabled),
                        ),
                    )
                    .children(expand_button)
                    .into_any_element()
            }
        }
    }
}

/// A code scroller (`overflow-x: auto`): a horizontal [`ScrollArea`] with
/// Carbon's 8px bar, scrolled by a horizontal wheel or trackpad, Shift+wheel
/// and the bar. A vertical wheel passes through to the page.
fn code_scroller(id: impl Into<ElementId>) -> ScrollArea {
    ScrollArea::new(id)
        .axes(ScrollAxes::Horizontal)
        .focusable(false)
        .restrict_wheel_to_axis()
        .w_full()
}

/// The right-edge fade of a single snippet (`mask-image: linear-gradient(to
/// right, #000 calc(100% - 2rem), transparent)`), drawn as a `layer`
/// gradient over the last 2rem before the copy button.
fn fade_end(layer: Hsla) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .right(rem(40.))
        .h_full()
        .w(rem(32.))
        .bg(gpui_kit::linear_gradient(
            90.,
            gpui_kit::linear_color_stop(layer.opacity(0.), 0.),
            gpui_kit::linear_color_stop(layer, 1.),
        ))
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
    fn code_splits_like_index_jsx() {
        let lines = split_code("npm install @carbon/react\nnpm install carbon-ui\n");
        assert_eq!(lines.len(), 2);
        let block = CodeBlock::new("c", "a\nb");
        assert_eq!(block.raw(), "a\nb");
        assert_eq!(CodeBlockSize::Default.metrics(), (14., 20., 16.));
        assert_eq!(CodeBlockSize::Sm.metrics(), (12., 16., 8.));
    }

    #[test]
    fn colors_follow_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, mark) = line_colors(Some(LineVariant::Added));
        assert!(bg.is_some());
        assert_eq!(hex(mark), "#24a148");
        let (_, mark) = line_colors(Some(LineVariant::Removed));
        assert_eq!(hex(mark), "#da1e28");
        // `.cds--snippet .cds--copy-btn`: layer #f4f4f4, hover #e8e8e8, active #c6c6c6.
        assert_eq!(
            hex(button_background(LayerLevel::One, false, false)),
            "#f4f4f4"
        );
        assert_eq!(
            hex(button_background(LayerLevel::One, true, false)),
            "#e8e8e8"
        );
        assert_eq!(
            hex(button_background(LayerLevel::One, true, true)),
            "#c6c6c6"
        );
    }

    #[test]
    fn feedback_timing_and_collapse() {
        assert_eq!(feedback_opacity(Duration::ZERO), Some(0.));
        assert_eq!(feedback_opacity(Duration::from_millis(500)), Some(1.));
        let out = feedback_opacity(Duration::from_millis(2055)).unwrap();
        assert!(out > 0. && out < 1.);
        assert_eq!(feedback_opacity(Duration::from_millis(2111)), None);
        assert_eq!(collapsed_height(10), None);
        assert_eq!(collapsed_height(20), Some(240.));
        // Collapsed: exactly 15 rows of 16px, nothing more.
        assert_eq!(container_height(20, false), 15. * ROW);
        assert_eq!(container_height(20, true), 20. * ROW + 24. + 16.);
        assert_eq!(container_height(4, false), 4. * ROW + 24.);
    }

    #[test]
    fn copy_feedback_shows_below_like_carbon() {
        // `CopyButton({ align = 'bottom' })` in @carbon/react.
        const { assert!(COPY_FEEDBACK_BELOW) };
    }
}
