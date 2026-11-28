//! T3 · Diff & review: [`ReviewThreadCard`], an inline review discussion
//! (`.t3-rthread` in `src/t3.css`), and its unsent draft form (`.rc-foot` in
//! u0's `Main` board).
use crate::{
    components::{
        Avatar, AvatarColor, AvatarFallback, AvatarSize, Badge, BadgeSize, BadgeVariant, Button,
        ButtonSize, ButtonVariant, FieldSize, Input, InputState,
    },
    icons::CarbonIcon,
    t3::craft::{self, AvatarTone, rem, sans},
    theme::{LayerLevel, color},
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
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
    /// Said after the time, in the head (`<span>on line 15</span>`).
    pub note: Option<SharedString>,
    /// An icon in the avatar instead of the author's initials
    /// (`AvatarFallback color="gray"` > `Icons.User`).
    pub icon: Option<CarbonIcon>,
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
            note: None,
            icon: None,
        }
    }

    /// A note after the time (e.g. `on line 15`).
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Show `icon` in the avatar instead of initials.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
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
    /// The draft's footer note (`Not sent yet`).
    pub not_sent: SharedString,
    /// `Edit`.
    pub edit: SharedString,
    /// `Delete`.
    pub delete: SharedString,
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
            not_sent: "Not sent yet".into(),
            edit: "Edit".into(),
            delete: "Delete".into(),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An inline review thread: comments (24px avatar, author, time, body), a
/// reply field, Resolve and Add to chat. `layer-02` with a 1px inset
/// `border-subtle-01` frame; `resolved` fades to 70%; `pending` (an
/// unsubmitted draft) adds a 3px `support-warning` start bar and a Pending tag.
/// A [`draft`](Self::draft) is your own comment not sent yet: pending, and its
/// footer is "Not sent yet" with Edit and Delete instead of the reply field.
#[derive(IntoElement)]
pub struct ReviewThreadCard {
    id: ElementId,
    comments: Vec<ReviewComment>,
    resolved: bool,
    pending: bool,
    draft: bool,
    in_diff: bool,
    on_resolve: Option<Handler>,
    on_add_to_chat: Option<Handler>,
    on_edit: Option<Handler>,
    on_delete: Option<Handler>,
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
            draft: false,
            in_diff: false,
            on_resolve: None,
            on_add_to_chat: None,
            on_edit: None,
            on_delete: None,
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

    /// A draft not sent yet (implies [`pending`](Self::pending)): the footer
    /// is `.rc-foot`, "Not sent yet" (12px `text-helper`) with ghost sm Edit
    /// and Delete, in place of the reply field, Resolve and Add to chat.
    pub fn draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }

    /// Whether the card reads as pending (a draft always does).
    pub fn is_pending(&self) -> bool {
        self.pending || self.draft
    }

    /// Edit pressed on a draft.
    pub fn on_edit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }

    /// Delete pressed on a draft.
    pub fn on_delete(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_delete = Some(Rc::new(f));
        self
    }

    /// Shown inside a [`DiffView`](crate::t3::review::DiffView) annotation (`.t3-dl-anno`): the
    /// thread inherits the diff's text (`.t3-diff { font-size: 0.75rem;
    /// line-height: 1.25rem }`) instead of the page's 14px / 1.28572, so the
    /// comment head sits on a 20px line.
    pub fn in_diff(mut self, in_diff: bool) -> Self {
        self.in_diff = in_diff;
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

fn comment(c: &ReviewComment, pending_tag: Option<SharedString>, in_diff: bool) -> AnyElement {
    // `.t3-rcomment { display: flex; gap: 12px; padding: 12px; border-bottom: 1px subtle-01 }`.
    div()
        .flex()
        .gap(rem(12.))
        .p(rem(12.))
        .border_b_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(match c.icon {
            Some(icon) => Avatar::new()
                .size(AvatarSize::Sm)
                .fallback(AvatarFallback::icon(icon).color(c.color))
                .into_any_element(),
            None => craft::initials(
                craft::two_letters(&c.author),
                AvatarTone::Avatar(c.color),
                24.,
            )
            .into_any_element(),
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    // `.t3-rcomment-head { gap: 8px; 12px; text-secondary } b { text-primary }`;
                    // the line height is inherited (20px inside a diff).
                    div()
                        .font(font(FontFamilyKey::Sans, 400))
                        .text_size(rem(12.))
                        .when(!in_diff, |this| {
                            this.line_height(relative(craft::BODY_LINE))
                        })
                        .flex()
                        .items_center()
                        .gap(rem(8.))
                        .text_color(color(ColorRole::TextSecondary))
                        // No weight rule on the `<b>`: Carbon's `b { font: inherit }`
                        // keeps it 400.
                        .child(
                            div()
                                .text_color(color(ColorRole::TextPrimary))
                                .child(c.author.clone()),
                        )
                        .child(c.time.clone())
                        .when_some(c.note.clone(), |this, note| this.child(note))
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

/// The thread's footer (`.t3-rthread-foot { gap: 4px; padding: 8px 4px 8px
/// 12px; flex-wrap: wrap }`): the reply field, Resolve, Add to chat.
fn reply_foot(
    id: &ElementId,
    reply: &gpui_kit::Entity<InputState>,
    texts: &ReviewThreadTexts,
    resolved: bool,
    on_resolve: Option<Handler>,
    on_chat: Option<Handler>,
) -> AnyElement {
    div()
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
                    Input::new((id.clone(), "reply-field"), reply)
                        .label(texts.reply_label.clone())
                        .hide_label(true)
                        .placeholder(texts.reply.clone())
                        .size(FieldSize::Sm)
                        .layer(LayerLevel::Two),
                ),
        )
        .child(
            Button::new(
                (id.clone(), "resolve"),
                if resolved {
                    texts.unresolve.clone()
                } else {
                    texts.resolve.clone()
                },
            )
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .when_some(on_resolve, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
        )
        .child(
            Button::new((id.clone(), "chat"), texts.add_to_chat.clone())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .icon(CarbonIcon::Chat)
                .when_some(on_chat, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
        )
        .into_any_element()
}

/// A draft's footer (`.rc-foot { display: flex; align-items: center; gap:
/// 8px; padding: 8px 8px 8px 16px }`): "Not sent yet" (`flex: 1; font-size:
/// 12px; color: var(--text-helper)`), then ghost sm Edit and Delete.
fn draft_foot(
    id: &ElementId,
    texts: &ReviewThreadTexts,
    on_edit: Option<Handler>,
    on_delete: Option<Handler>,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(rem(8.))
        .pt(rem(8.))
        .pb(rem(8.))
        .pr(rem(8.))
        .pl(rem(16.))
        .child(
            sans(div(), 12., 400)
                .flex_1()
                .min_w_0()
                .text_color(color(ColorRole::TextHelper))
                .child(texts.not_sent.clone()),
        )
        .child(
            Button::new((id.clone(), "edit"), texts.edit.clone())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .when_some(on_edit, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
        )
        .child(
            Button::new((id.clone(), "delete"), texts.delete.clone())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .when_some(on_delete, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
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
        let pending = self.is_pending();
        let comments = self.comments.iter().enumerate().map(|(i, c)| {
            comment(
                c,
                (pending && i == 0).then(|| self.texts.pending.clone()),
                self.in_diff,
            )
        });
        let foot = if self.draft {
            draft_foot(
                &id,
                &self.texts,
                self.on_edit.clone(),
                self.on_delete.clone(),
            )
        } else {
            reply_foot(
                &id,
                &reply,
                &self.texts,
                self.resolved,
                self.on_resolve.clone(),
                self.on_add_to_chat.clone(),
            )
        };
        // Outside a diff the thread inherits the page's `body { 14px; 1.28572 }`;
        // inside one, `.t3-dl-anno`'s 12px / 20px.
        div()
            .font(font(FontFamilyKey::Sans, 400))
            .when(!self.in_diff, |this| {
                this.text_size(rem(14.))
                    .line_height(relative(craft::BODY_LINE))
            })
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
            .when(pending, |this| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_draft_reads_as_pending_and_keeps_its_note_and_icon() {
        assert!(!ReviewThreadCard::new("t").is_pending());
        assert!(ReviewThreadCard::new("t").draft(true).is_pending());
        assert!(ReviewThreadCard::new("t").pending(true).is_pending());
        let c = ReviewComment::new("You", "10:22", "Put g90 before g100.")
            .note("on line 15")
            .icon(CarbonIcon::User)
            .color(AvatarColor::Gray);
        assert_eq!(c.note.as_deref(), Some("on line 15"));
        assert_eq!(c.icon, Some(CarbonIcon::User));
        let t = ReviewThreadTexts::default();
        assert_eq!(
            (t.not_sent.as_ref(), t.edit.as_ref(), t.delete.as_ref()),
            ("Not sent yet", "Edit", "Delete")
        );
    }
}
