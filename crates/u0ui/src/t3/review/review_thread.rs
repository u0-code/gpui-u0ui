//! T3 · Diff & review: [`ReviewThreadCard`], an inline review discussion
//! (`.t3-rthread` in `src/t3.css`).
use crate::{
    components::{
        AvatarColor, Badge, BadgeSize, BadgeVariant, Button, ButtonSize, ButtonVariant, FieldSize,
        Input, InputState,
    },
    icons::CarbonIcon,
    t3::craft::{self, Tone, rem, sans},
    theme::{LayerLevel, color},
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ElementId, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
    relative,
};
use std::rc::Rc;

/// One review comment (`comments[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewComment {
    /// `author`.
    pub author: SharedString,
    /// `time`.
    pub time: SharedString,
    /// `body`.
    pub body: SharedString,
    /// `color` (default `blue`).
    pub color: AvatarColor,
}

impl ReviewComment {
    /// A comment.
    pub fn new(
        author: impl Into<SharedString>,
        time: impl Into<SharedString>,
        body: impl Into<SharedString>,
    ) -> Self {
        Self {
            author: author.into(),
            time: time.into(),
            body: body.into(),
            color: AvatarColor::Blue,
        }
    }

    /// The avatar color.
    pub fn color(mut self, color: AvatarColor) -> Self {
        self.color = color;
        self
    }
}

/// Texts of a [`ReviewThreadCard`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct ReviewThreadTexts {
    /// Reply placeholder and name (`Reply…`, `Reply`).
    pub reply: SharedString,
    /// Reply field name (`Reply`).
    pub reply_label: SharedString,
    /// `Resolve conversation`.
    pub resolve: SharedString,
    /// `Unresolve`.
    pub unresolve: SharedString,
    /// `Add to chat`.
    pub add_to_chat: SharedString,
    /// The draft tag (`Pending`).
    pub pending: SharedString,
}

impl Default for ReviewThreadTexts {
    fn default() -> Self {
        Self {
            reply: "Reply\u{2026}".into(),
            reply_label: "Reply".into(),
            resolve: "Resolve conversation".into(),
            unresolve: "Unresolve".into(),
            add_to_chat: "Add to chat".into(),
            pending: "Pending".into(),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An inline review thread: comments (24px avatar, author, time, body), a
/// reply field, Resolve and Add to chat. `layer-02` with a 1px inset
/// `border-subtle-01` frame; `resolved` fades to 70%; `pending` (an
/// unsubmitted draft) adds a 3px `support-warning` start bar and a Pending tag.
#[derive(IntoElement)]
pub struct ReviewThreadCard {
    id: ElementId,
    comments: Vec<ReviewComment>,
    resolved: bool,
    pending: bool,
    on_resolve: Option<Handler>,
    on_add_to_chat: Option<Handler>,
    texts: ReviewThreadTexts,
}

impl ReviewThreadCard {
    /// A thread card.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            comments: Vec::new(),
            resolved: false,
            pending: false,
            on_resolve: None,
            on_add_to_chat: None,
            texts: ReviewThreadTexts::default(),
        }
    }

    /// The comments.
    pub fn comments(mut self, comments: impl IntoIterator<Item = ReviewComment>) -> Self {
        self.comments = comments.into_iter().collect();
        self
    }

    /// Resolved (`.t3-rthread--resolved { opacity: .7 }`, button reads Unresolve).
    pub fn resolved(mut self, resolved: bool) -> Self {
        self.resolved = resolved;
        self
    }

    /// An unsubmitted draft (`.t3-rthread--pending`).
    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    /// Resolve / Unresolve pressed (`onResolve`).
    pub fn on_resolve(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_resolve = Some(Rc::new(f));
        self
    }

    /// Add to chat pressed.
    pub fn on_add_to_chat(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add_to_chat = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: ReviewThreadTexts) -> Self {
        self.texts = texts;
        self
    }
}

fn comment(c: &ReviewComment, pending_tag: Option<SharedString>) -> AnyElement {
    // `.t3-rcomment { display: flex; gap: 12px; padding: 12px; border-bottom: 1px subtle-01 }`.
    div()
        .flex()
        .gap(rem(12.))
        .p(rem(12.))
        .border_b_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(craft::initials(
            craft::two_letters(&c.author),
            Tone::Avatar(c.color),
            24.,
        ))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    // `.t3-rcomment-head { gap: 8px; 12px; text-secondary } b { text-primary }`.
                    sans(div(), 12., 400)
                        .flex()
                        .items_center()
                        .gap(rem(8.))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(
                            sans(div(), 12., 600)
                                .text_color(color(ColorRole::TextPrimary))
                                .child(c.author.clone()),
                        )
                        .child(c.time.clone())
                        .when_some(pending_tag, |this, tag| {
                            this.child(
                                Badge::new(tag)
                                    .variant(BadgeVariant::WarmGray)
                                    .size(BadgeSize::Sm),
                            )
                        }),
                )
                .child(
                    // `.t3-rcomment-body { margin-top: 4px; 14px; line-height: 1.42857 }`.
                    sans(div(), 14., 400)
                        .mt(rem(4.))
                        .line_height(relative(1.42857))
                        .text_color(color(ColorRole::TextPrimary))
                        .child(c.body.clone()),
                ),
        )
        .into_any_element()
}

impl RenderOnce for ReviewThreadCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let reply = window
            .use_keyed_state((id.clone(), "reply"), cx, |window, cx| {
                cx.new(|cx| InputState::new(window, cx))
            })
            .read(cx)
            .clone();
        let on_resolve = self.on_resolve.clone();
        let on_chat = self.on_add_to_chat.clone();
        let comments = self.comments.iter().enumerate().map(|(i, c)| {
            comment(
                c,
                (self.pending && i == 0).then(|| self.texts.pending.clone()),
            )
        });
        // `.t3-rthread-foot { gap: 4px; padding: 8px 4px 8px 12px; flex-wrap: wrap }`.
        let foot = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(4.))
            .pt(rem(8.))
            .pb(rem(8.))
            .pr(rem(4.))
            .pl(rem(12.))
            .child(
                // `.t3-rthread-reply { flex: 1 1 12rem; height: size-sm; field-02 }`.
                div()
                    .flex_grow_1()
                    .flex_shrink_1()
                    .flex_basis(rem(192.))
                    .min_w_0()
                    .child(
                        Input::new((id.clone(), "reply-field"), &reply)
                            .label(self.texts.reply_label.clone())
                            .hide_label(true)
                            .placeholder(self.texts.reply.clone())
                            .size(FieldSize::Sm)
                            .layer(LayerLevel::Two),
                    ),
            )
            .child(
                Button::new(
                    (id.clone(), "resolve"),
                    if self.resolved {
                        self.texts.unresolve.clone()
                    } else {
                        self.texts.resolve.clone()
                    },
                )
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .when_some(on_resolve, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            )
            .child(
                Button::new((id.clone(), "chat"), self.texts.add_to_chat.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(CarbonIcon::Chat)
                    .when_some(on_chat, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            );
        sans(div(), 14., 400)
            .id(id)
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(576.))
            .bg(color(ColorRole::Layer02))
            .when(self.resolved, |this| this.opacity(0.7))
            .children(comments)
            .child(foot)
            // `outline: 1px solid var(--border-subtle-01); outline-offset: -1px`.
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01)),
            )
            // `.t3-rthread--pending { box-shadow: inset 3px 0 0 var(--support-warning) }`.
            .when(self.pending, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .bottom_0()
                        .w(px(3.))
                        .bg(color(ColorRole::SupportWarning)),
                )
            })
    }
}
