//! Messages ("Messages" in `t3.css`): [`UserMessage`] (`.t3-umsg`) and
//! [`AssistantMessage`] (`.t3-amsg`), with [`Prose`] for the agent's text.
//!
//! - **User**: right-aligned; author and time over optional images, then the
//!   square `chat-bubble-user` bubble (craft pass) with its context chips and
//!   text; `folded` clips the text to five lines under a fade and adds Show
//!   more / Show less. Copy and Edit fade in (110ms) on hover or keyboard
//!   focus, overlapping what follows instead of reserving a row (d-fix).
//! - **Assistant**: provenance first (provider, model · duration · time) with
//!   24px Copy / Reply / Fork icon buttons at its end (gemini-A), then the
//!   14/22 prose body; an app that draws its agents' marks itself puts its
//!   own in place of the provider's ([`AssistantMessage::mark`]); inline code is 13px mono on a quiet `layer-accent-01`
//!   tint (r2-desk), drawn by the helper shared with the u0
//!   MarkdownProse ([`crate::components::inline_code`]).
//!
//! Each action button shows only with a handler to call: a message with
//! none set (a side chat's, say) draws its text alone.
use super::super::action::{ActionButton, ActionKind, Part};
use crate::components::button::{Button, ButtonKind, ButtonSize, ButtonVariant};
use crate::components::floating::Align;
use crate::components::inline_code::{push_inline_code, with_code_boxes};
use crate::t3::base::fmt_elapsed;
use crate::t3::composer::{AttachmentImage, ComposerAttachmentStrip, ContextChip};
use crate::t3::press::{self, Handler, rem};
use crate::t3::shell::{IdentitySize, ProviderIcon};
use crate::{
    icons::CarbonIcon,
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, StyledText,
    TestSupportExt as _, TextRun, Window, div, linear_color_stop, linear_gradient,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// One run of [`Prose`]: text or inline code.
#[derive(Clone, Debug, PartialEq)]
pub enum ProseSpan {
    /// Body text.
    Text(SharedString),
    /// Inline `code`.
    Code(SharedString),
}

/// A paragraph of agent prose (`.t3-amsg-body p`): 14/22 body text with
/// 13px inline code in IBM Plex Mono on a `layer-accent-01` 60% tint.
#[derive(IntoElement, Clone)]
pub struct Prose {
    spans: Vec<ProseSpan>,
}

impl Prose {
    /// A paragraph of `spans`.
    pub fn new(spans: impl IntoIterator<Item = ProseSpan>) -> Self {
        Self {
            spans: spans.into_iter().collect(),
        }
    }

    /// A paragraph split at backticks: `` "add the `g90` theme" ``.
    pub fn markdown(text: &str) -> Self {
        let mut spans = Vec::new();
        for (i, part) in text.split('`').enumerate() {
            if part.is_empty() {
                continue;
            }
            spans.push(if i % 2 == 1 {
                ProseSpan::Code(part.to_string().into())
            } else {
                ProseSpan::Text(part.to_string().into())
            });
        }
        Self { spans }
    }
}

pub use crate::components::inline_code::inline_code_fill;

/// `.t3-amsg-body { font-size: 0.875rem; line-height: 1.375rem }`.
const BODY: (f32, f32) = (14., 22.);

impl RenderOnce for Prose {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut text = String::new();
        let mut runs = Vec::new();
        let mut code = Vec::new();
        let ink = color(ColorRole::TextPrimary);
        for span in &self.spans {
            match span {
                ProseSpan::Text(s) => {
                    text.push_str(s);
                    runs.push(TextRun {
                        len: s.len(),
                        font: font(FontFamilyKey::Sans, 400),
                        color: ink,
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    });
                }
                ProseSpan::Code(s) => {
                    code.push(push_inline_code(&mut text, &mut runs, s, ink, BODY.0));
                }
            }
        }
        // `.t3-amsg-body p { margin: 0 0 spacing-04 }`.
        div()
            .mb(rem(12.))
            .text_size(rem(BODY.0))
            .line_height(rem(BODY.1))
            .text_color(ink)
            .child(with_code_boxes(
                StyledText::new(text).with_runs(runs),
                code,
                BODY.0,
            ))
    }
}

/// `UserMessage`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct UserMessageTexts {
    /// Show more.
    pub show_more: SharedString,
    /// Show less.
    pub show_less: SharedString,
    /// Copy.
    pub copy: SharedString,
    /// Edit and resend.
    pub edit: SharedString,
}

impl Default for UserMessageTexts {
    fn default() -> Self {
        Self {
            show_more: "Show more".into(),
            show_less: "Show less".into(),
            copy: "Copy".into(),
            edit: "Edit and resend".into(),
        }
    }
}

/// `.t3-umsg-bubble--folded .t3-umsg-text { max-height: 5rem }` in px.
pub const FOLDED_HEIGHT: f32 = 80.;

/// The user's message. See the module docs.
#[derive(IntoElement)]
pub struct UserMessage {
    id: ElementId,
    text: SharedString,
    author: SharedString,
    time: Option<SharedString>,
    contexts: Vec<ContextChip>,
    images: Vec<AttachmentImage>,
    folded: bool,
    texts: UserMessageTexts,
    on_copy: Option<Handler>,
    on_edit: Option<Handler>,
}

impl UserMessage {
    /// A message saying `text` (the author defaults to "You").
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            author: "You".into(),
            time: None,
            contexts: Vec::new(),
            images: Vec::new(),
            folded: false,
            texts: UserMessageTexts::default(),
            on_copy: None,
            on_edit: None,
        }
    }

    /// `author`.
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = author.into();
        self
    }

    /// `time`.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// `contexts`: chips above the text.
    pub fn contexts(mut self, contexts: Vec<ContextChip>) -> Self {
        self.contexts = contexts;
        self
    }

    /// `images`: thumbnails above the bubble.
    pub fn images(mut self, images: Vec<AttachmentImage>) -> Self {
        self.images = images;
        self
    }

    /// `folded`: starts folded with Show more.
    pub fn folded(mut self, folded: bool) -> Self {
        self.folded = folded;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: UserMessageTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onCopy`.
    pub fn on_copy(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_copy = Some(Rc::new(f));
        self
    }

    /// `onEdit`.
    pub fn on_edit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }
}

/// A ghost `sm` icon button tracking `focus`.
fn ghost_icon(
    id: ElementId,
    label: SharedString,
    icon: CarbonIcon,
    focus: &FocusHandle,
    on_click: Handler,
) -> Button {
    Button::new(id, label)
        .icon(icon)
        .icon_only(true)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .track_focus(focus)
        .on_click(move |_, w, cx| on_click(w, cx))
}

impl RenderOnce for UserMessage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, {
            let folded = self.folded;
            move |_, _| !folded
        });
        let open = *open_state.read(cx);
        let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
        let (copy_focus, _) = press::focus((id.clone(), "copy-focus"), window, cx);
        let (edit_focus, _) = press::focus((id.clone(), "edit-focus"), window, cx);
        let (link_focus, link_ring) = press::focus((id.clone(), "link-focus"), window, cx);
        let (link_hovered, link_hover) = press::hover((id.clone(), "link-hover"), window, cx);
        // `.t3-umsg:hover .t3-msg-actions, .t3-umsg:focus-within .t3-msg-actions { opacity: 1 }`,
        // `transition: opacity 110ms`.
        let shown = hovered
            || copy_focus.is_focused(window)
            || edit_focus.is_focused(window)
            || link_focus.is_focused(window);
        let opacity = motion::animate(
            (id.clone(), "actions"),
            if shown { 1f32 } else { 0. },
            Motion::new(motion::Curve::ProductiveStandard, motion::Speed::Fast02),
            window,
            cx,
        );
        let bubble_fill = color(ColorRole::ChatBubbleUser);
        let texts = self.texts.clone();
        let folded_now = self.folded && !open;

        let text = div()
            .relative()
            .text_size(rem(14.))
            .line_height(rem(20.))
            .whitespace_normal()
            .child(self.text.clone())
            .when(folded_now, |this| {
                // `max-height: 5rem; overflow: hidden; mask: linear-gradient(#000 60%, transparent)`:
                // GPUI has no masks, so the bubble color fades in over the last 40%.
                this.max_h(rem(FOLDED_HEIGHT)).overflow_hidden().child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(relative(0.4))
                        .bg(linear_gradient(
                            180.,
                            linear_color_stop(bubble_fill.opacity(0.), 0.),
                            linear_color_stop(bubble_fill, 1.),
                        )),
                )
            });
        let toggle = self.folded.then(|| {
            let open_state = open_state.clone();
            // `.t3-linkbtn { color: link-primary; font-size: 0.75rem }`, `:hover { text-decoration: underline }`.
            gpui_kit::base::Button::new((id.clone(), "fold"))
                .track_focus(&link_focus)
                .relative()
                .self_start()
                .text_size(rem(12.))
                .line_height(rem(16.))
                .text_color(color(ColorRole::LinkPrimary))
                .cursor_pointer()
                .on_hover(press::track_hover(link_hover))
                .on_click(move |_, _, cx| {
                    open_state.update(cx, |o, cx| {
                        *o = !*o;
                        cx.notify();
                    })
                })
                .child(
                    div()
                        .when(link_hovered, |this| this.underline())
                        .child(if open {
                            texts.show_less.clone()
                        } else {
                            texts.show_more.clone()
                        }),
                )
                .when(link_ring, |this| this.child(press::ring()))
        });
        // `.t3-umsg-bubble { max-width: min(36rem, 90%); padding: spacing-04 spacing-05;
        // background: chat-bubble-user; color: chat-bubble-user-text;
        // outline: 1px solid chat-bubble-border; outline-offset: -1px }`, square (craft pass).
        let bubble = div()
            .id((id.clone(), "bubble"))
            .test_support()
            .flex()
            .flex_col()
            .max_w(rem(576.))
            .min_w_0()
            .py(rem(12.))
            .px(rem(16.))
            .bg(bubble_fill)
            .text_color(color(ColorRole::ChatBubbleUserText))
            .shadow(press::inset_frame(color(ColorRole::ChatBubbleBorder)))
            .when(!self.contexts.is_empty(), |this| {
                // `.t3-umsg-ctx { flex-wrap: wrap; gap: spacing-02 spacing-04; margin-bottom: spacing-03 }`.
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap_y(rem(4.))
                        .gap_x(rem(16.))
                        .mb(rem(8.))
                        .children(self.contexts),
                )
            })
            .child(text)
            .children(toggle);
        // GPUI takes one max width, so the 90% half of `min(36rem, 90%)`
        // is a wrapper as wide as 90% of the column.
        let bubble = div()
            .flex()
            .flex_col()
            .items_end()
            .max_w(relative(0.9))
            .min_w_0()
            .child(bubble);
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .items_end()
            .w_full()
            .gap(rem(4.))
            .on_hover(press::track_hover(hover))
            // `.t3-umsg-head { gap: spacing-03; font-size: 0.75rem; color: text-secondary }`,
            // `b { color: text-primary; font-weight: 600 }`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(rem(8.))
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(self.author.clone()),
                    )
                    .children(self.time.clone()),
            )
            .when(!self.images.is_empty(), |this| {
                this.child(ComposerAttachmentStrip::new(
                    (id.clone(), "images"),
                    self.images.clone(),
                    Vec::new(),
                ))
            })
            .child(bubble)
            // `.t3-umsg > .t3-msg-actions { margin-bottom: -32px }` (d-fix): no reserved row.
            // Each action shows only with its handler; with neither, no row.
            .when(self.on_copy.is_some() || self.on_edit.is_some(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .mb(rem(-32.))
                        .opacity(opacity)
                        .children(self.on_copy.clone().map(|f| {
                            ghost_icon(
                                (id.clone(), "copy").into(),
                                texts.copy.clone(),
                                CarbonIcon::Copy,
                                &copy_focus,
                                f,
                            )
                        }))
                        .children(self.on_edit.clone().map(|f| {
                            ghost_icon(
                                (id.clone(), "edit").into(),
                                texts.edit.clone(),
                                CarbonIcon::Edit,
                                &edit_focus,
                                f,
                            )
                        })),
                )
            })
    }
}

/// `AssistantMessage`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct AssistantMessageTexts {
    /// Copy.
    pub copy: SharedString,
    /// Reply to this response.
    pub reply: SharedString,
    /// Fork from this response.
    pub fork: SharedString,
}

impl Default for AssistantMessageTexts {
    fn default() -> Self {
        Self {
            copy: "Copy".into(),
            reply: "Reply to this response".into(),
            fork: "Fork from this response".into(),
        }
    }
}

/// The agent's reply. See the module docs.
#[derive(IntoElement)]
pub struct AssistantMessage {
    id: ElementId,
    body: Vec<AnyElement>,
    provider: Option<SharedString>,
    mark: Option<AnyElement>,
    model: Option<SharedString>,
    elapsed: Option<u64>,
    time: Option<SharedString>,
    texts: AssistantMessageTexts,
    on_copy: Option<Handler>,
    on_reply: Option<Handler>,
    on_fork: Option<Handler>,
}

impl AssistantMessage {
    /// An empty reply; add the body as children ([`Prose`] paragraphs or any
    /// rendered markdown).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            body: Vec::new(),
            provider: None,
            mark: None,
            model: None,
            elapsed: None,
            time: None,
            texts: AssistantMessageTexts::default(),
            on_copy: None,
            on_reply: None,
            on_fork: None,
        }
    }

    /// `provider`: its icon leads the meta line.
    pub fn provider(mut self, provider: impl Into<SharedString>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// What leads the meta line in place of the provider icon: an app's
    /// own mark for its agent (an SVG logo, say), in a 16px box.
    pub fn mark(mut self, mark: impl IntoElement) -> Self {
        self.mark = Some(mark.into_any_element());
        self
    }

    /// `model`.
    pub fn model(mut self, model: impl Into<SharedString>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// `elapsed` seconds.
    pub fn elapsed(mut self, seconds: u64) -> Self {
        self.elapsed = Some(seconds);
        self
    }

    /// `time`.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: AssistantMessageTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onCopy`.
    pub fn on_copy(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_copy = Some(Rc::new(f));
        self
    }

    /// `onReply`.
    pub fn on_reply(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reply = Some(Rc::new(f));
        self
    }

    /// `onFork`.
    pub fn on_fork(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_fork = Some(Rc::new(f));
        self
    }
}

impl ParentElement for AssistantMessage {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

/// A 24px ghost icon button (`.t3-amsg-meta .cds--btn--icon-only { height: 1.5rem;
/// width: 1.5rem }`, glyph `icon-secondary`).
fn meta_button(
    id: ElementId,
    label: SharedString,
    icon: CarbonIcon,
    on_click: Handler,
) -> ActionButton {
    ActionButton::new(id, label.clone(), ActionKind::Carbon(ButtonKind::Ghost))
        .height(24.)
        .padding(0., 0.)
        .square(true)
        .ink(color(ColorRole::IconSecondary))
        .part(Part::Icon(icon))
        .tooltip(label)
        .tooltip_align(Align::Top)
        .on_click(Some(on_click))
}

impl RenderOnce for AssistantMessage {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let dot = || {
            // `.t3-metadot { color: text-helper; padding: 0 2px }`.
            div().px(px(2.)).child("·")
        };
        let texts = self.texts.clone();
        // `.t3-amsg-meta { display: flex; align-items: center; gap: spacing-02; font-size: 0.75rem;
        // color: text-helper; order: -1; min-height: 1.5rem }`.
        let meta = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(4.))
            .min_h(rem(24.))
            .text_size(rem(12.))
            .line_height(rem(16.))
            .text_color(color(ColorRole::TextHelper))
            .map(|this| match (self.mark, self.provider.clone()) {
                // The 16px glyph of `ProviderIcon size="xs"`.
                (Some(mark), _) => this.child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(rem(16.))
                        .child(mark),
                ),
                (None, Some(p)) => this.child(ProviderIcon::new(p).size(IdentitySize::Xs)),
                (None, None) => this,
            })
            .children(self.model.clone())
            .when_some(self.elapsed, |this, s| {
                this.child(dot()).child(SharedString::from(fmt_elapsed(s)))
            })
            .when_some(self.time.clone(), |this, t| this.child(dot()).child(t))
            .child(div().flex_1())
            // Each action shows only with its handler.
            .children(
                [
                    ("copy", texts.copy, CarbonIcon::Copy, self.on_copy),
                    ("reply", texts.reply, CarbonIcon::Reply, self.on_reply),
                    ("fork", texts.fork, CarbonIcon::Fork, self.on_fork),
                ]
                .into_iter()
                .filter_map(|(part, label, icon, f)| {
                    f.map(|f| meta_button((id.clone(), part).into(), label, icon, f))
                }),
            );
        // `.t3-amsg { display: flex; flex-direction: column; gap: spacing-03 }`,
        // `.t3-amsg-body { font-size: 0.875rem; line-height: 1.375rem; color: text-primary }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .gap(rem(8.))
            .child(meta)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .text_size(rem(14.))
                    .line_height(rem(22.))
                    .text_color(color(ColorRole::TextPrimary))
                    .children(self.body),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_splits_inline_code() {
        let p = Prose::markdown("I added the `g90` theme.");
        assert_eq!(
            p.spans,
            vec![
                ProseSpan::Text("I added the ".into()),
                ProseSpan::Code("g90".into()),
                ProseSpan::Text(" theme.".into()),
            ]
        );
    }

    #[test]
    fn inline_code_is_a_quiet_tint() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let fill = inline_code_fill();
        assert!((fill.a - 0.6).abs() < 1e-3);
    }
}
