//! Message: the layout of one chat message (`.cu-msg*` in `custom.css`):
//! avatar, a content column with a header, the [`super::bubble::Bubble`] and a
//! footer of actions. Layout only; no state, no motion, nothing from Kit.
//!
//! - `.cu-msg { display: flex; gap: spacing-04; align-items: flex-start }`;
//!   `align="end"` reverses the row (`.cu-msg--end { flex-direction:
//!   row-reverse }`) and right-aligns the content column (`.cu-msg--end
//!   .cu-msg-content { align-items: flex-end }`).
//! - [`MessageAvatar`]: a 2rem circle (`border-radius-max`), user
//!   `chat-avatar-user` with the 16px `User` icon in `text-on-color`; agent
//!   `chat-avatar-agent` with caller text (the reference's "AI") in
//!   `text-inverse`, 0.75rem semibold (`.cu-msg-bot`).
//! - [`MessageContent`]: `flex: 1; flex-direction: column; gap: spacing-02;
//!   min-width: 0`. Bubbles inside should use [`super::bubble::Bubble::fill_message`]
//!   (`.cu-msg-content .cu-bubble { max-width: 100% }`).
//! - [`MessageHeader`]: a baseline row 8px apart, 0.75rem `text-secondary`, the
//!   name semibold in `text-primary` (`.cu-msg-header b`).
//! - [`MessageFooter`]: a centered row 4px apart, 0.75rem `text-helper`.
//! - [`MessageGroup`]: messages 24px apart (`.cu-msg-group { gap: spacing-06 }`).
use super::bubble::BubbleAlign;
use super::form::rem;
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{ColorRole, SPACING_02, SPACING_03, SPACING_04, SPACING_06, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _,
};

/// The message alignment (`index.d.ts` `Message.align`).
pub type MessageAlign = BubbleAlign;

/// `index.d.ts` `MessageAvatar.kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AvatarKind {
    /// The user: `chat-avatar-user` with the `User` icon.
    #[default]
    User,
    /// The assistant: `chat-avatar-agent` with text.
    Agent,
}

impl AvatarKind {
    /// Both kinds.
    pub const ALL: [AvatarKind; 2] = [Self::User, Self::Agent];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Agent => "agent",
        }
    }

    /// (background, foreground): `.cu-msg-avatar--user { background:
    /// chat-avatar-user; color: text-on-color }`, `.cu-msg-avatar--agent {
    /// background: chat-avatar-agent; color: text-inverse }`.
    pub fn colors(self) -> (Hsla, Hsla) {
        match self {
            Self::User => (
                color(ColorRole::ChatAvatarUser),
                color(ColorRole::TextOnColor),
            ),
            Self::Agent => (
                color(ColorRole::ChatAvatarAgent),
                color(ColorRole::TextInverse),
            ),
        }
    }
}

/// `.cu-msg-avatar { width: 2rem; height: 2rem }` in px.
pub const AVATAR: f32 = 32.;

/// The 2rem round avatar. See the module docs.
#[derive(IntoElement, Default)]
pub struct MessageAvatar {
    kind: AvatarKind,
    text: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl MessageAvatar {
    /// A user avatar (the `User` icon).
    pub fn user() -> Self {
        Self::default()
    }

    /// An agent avatar showing `text` (the reference shows "AI").
    pub fn agent(text: impl Into<SharedString>) -> Self {
        Self {
            kind: AvatarKind::Agent,
            text: Some(text.into()),
            children: Vec::new(),
        }
    }

    /// An avatar of `kind`; add custom content with `child` (replaces the default).
    pub fn new(kind: AvatarKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    /// The agent text.
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }
}

impl ParentElement for MessageAvatar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageAvatar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (bg, fg) = self.kind.colors();
        let base = div()
            .flex()
            .flex_none()
            .size(rem(AVATAR))
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(bg)
            .text_color(fg);
        if !self.children.is_empty() {
            return base.children(self.children);
        }
        match (self.kind, self.text) {
            (AvatarKind::User, None) => {
                base.child(Icon::carbon(CarbonIcon::User).size(IconSize::S16).color(fg))
            }
            (_, text) => base.child(
                // `.cu-msg-bot { font-size: 0.75rem; font-weight: 600 }`.
                div()
                    .text_size(rem(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .children(text),
            ),
        }
    }
}

/// The content column. See the module docs.
#[derive(IntoElement, Default)]
pub struct MessageContent {
    align: MessageAlign,
    children: Vec<AnyElement>,
}

impl MessageContent {
    /// An empty column (its alignment is set by the [`Message`]).
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for MessageContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageContent {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_1()
            .flex_col()
            .gap(SPACING_02)
            .min_w_0()
            .when(self.align == MessageAlign::End, |this| this.items_end())
            .children(self.children)
    }
}

/// The header row: an optional bold name, then the rest (time…).
#[derive(IntoElement, Default)]
pub struct MessageHeader {
    name: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl MessageHeader {
    /// An empty header.
    pub fn new() -> Self {
        Self::default()
    }

    /// The sender's name (`<b>`: semibold, `text-primary`).
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl ParentElement for MessageHeader {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageHeader {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `font-size: 0.75rem; letter-spacing: 0.32px` (label-01 metrics).
        div()
            .flex()
            .flex_row()
            .items_baseline()
            .gap(SPACING_03)
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .when_some(self.name, |this, name| {
                this.child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(color(ColorRole::TextPrimary))
                        .child(name),
                )
            })
            .children(self.children)
    }
}

/// The footer row of actions.
#[derive(IntoElement, Default)]
pub struct MessageFooter {
    children: Vec<AnyElement>,
}

impl MessageFooter {
    /// An empty footer.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for MessageFooter {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageFooter {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(SPACING_02)
            .text_size(rem(12.))
            .text_color(color(ColorRole::TextHelper))
            .children(self.children)
    }
}

/// One message: avatar + content column. See the module docs.
#[derive(IntoElement, Default)]
pub struct Message {
    align: MessageAlign,
    avatar: Option<MessageAvatar>,
    content: Option<MessageContent>,
    children: Vec<AnyElement>,
}

impl Message {
    /// A start-aligned message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start (other party) or end (the user: row reversed).
    pub fn align(mut self, align: MessageAlign) -> Self {
        self.align = align;
        self
    }

    /// The avatar.
    pub fn avatar(mut self, avatar: MessageAvatar) -> Self {
        self.avatar = Some(avatar);
        self
    }

    /// The content column.
    pub fn content(mut self, content: MessageContent) -> Self {
        self.content = Some(content);
        self
    }
}

impl ParentElement for Message {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Message {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let align = self.align;
        div()
            .flex()
            .map(|this| match align {
                MessageAlign::Start => this.flex_row(),
                MessageAlign::End => this.flex_row_reverse(),
            })
            .items_start()
            .gap(SPACING_04)
            .children(self.avatar)
            .children(self.content.map(|mut c| {
                c.align = align;
                c
            }))
            .children(self.children)
    }
}

/// Messages 24px apart.
#[derive(IntoElement, Default)]
pub struct MessageGroup {
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl MessageGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Styled for MessageGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for MessageGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut group = div()
            .flex()
            .flex_col()
            .gap(SPACING_06)
            .children(self.children);
        gpui_kit::Refineable::refine(group.style(), &self.style);
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn avatars_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            AvatarKind::User.colors(),
            (
                color(ColorRole::ChatAvatarUser),
                color(ColorRole::TextOnColor)
            )
        );
        assert_eq!(
            AvatarKind::Agent.colors(),
            (
                color(ColorRole::ChatAvatarAgent),
                color(ColorRole::TextInverse)
            )
        );
        assert_eq!(AVATAR, 32.);
        // `.cu-msg { gap: spacing-04 }`, `.cu-msg-group { gap: spacing-06 }`.
        assert_eq!((SPACING_04.px_value(), SPACING_06.px_value()), (12., 24.));
    }
}
