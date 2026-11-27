//! Bubble: a chat message bubble (`.cu-bubble*` in `custom.css`), painted with
//! Carbon's chat tokens (`chat-bubble-*`). Kit's `bubble` draws its own shape
//! and colors; this is a plain styled box, so nothing of Kit's is wrapped.
//!
//! - **Box**: `.cu-bubble { max-width: min(36rem, 85%); padding: spacing-04
//!   spacing-05; font-size: 0.875rem; line-height: 1.42857 }` (`body-01`),
//!   `border-radius: border-radius-08` (8px) with the sender-side bottom
//!   corner square (`.cu-bubble--end { border-bottom-right-radius:
//!   border-radius-00 }`, `--start` bottom-left). The `t3.css` craft pass
//!   that squares chat bubbles lists only the T3 ones (`.t3-umsg-bubble,
//!   .t3-cchip, .t3-ctx, .t3-mention, .t3-attach, .t3-composer-box
//!   { border-radius: 0 !important }`), not `.cu-bubble`, so the core
//!   bubble keeps its radius. `align-self` follows `align`. Inside a [`super::message::MessageContent`] the cap is 100%
//!   (`.cu-msg-content .cu-bubble { max-width: 100% }`): call
//!   [`Bubble::fill_message`].
//! - **Variants** ([`BubbleLook::of`]): default (user) `chat-bubble-user` /
//!   `chat-bubble-user-text` with a 1px inset `chat-bubble-border`; secondary
//!   (agent) `chat-bubble-agent` / `-agent-text` with the same line; muted
//!   `layer-01`; tinted `notification-background-info`; outline transparent
//!   with a 1px `border-strong-01`; ghost transparent with no inline padding;
//!   destructive `notification-background-error` with a 1px `support-error`.
//!   The CSS `outline` with `outline-offset: -1px` is drawn as an inset layer so
//!   the padding is unchanged.
//! - [`BubbleReactions`]: an absolutely positioned pill (`layer-02`, 1px
//!   `border-subtle-01` outline, `border-radius-16`, 12px / 20px text, 0 4px
//!   padding, 2px gap) 0.75rem past the top or bottom edge, 8px from the start
//!   or end. [`BubbleGroup`] stacks bubbles 4px apart; [`BubbleContent`] is a
//!   column for rich content.
//! - No state, no motion.
use super::form::rem;
use crate::{
    theme::color,
    tokens::{
        BORDER_RADIUS_00, BORDER_RADIUS_08, BORDER_RADIUS_16, ColorRole, SPACING_02, SPACING_03,
        SPACING_04, SPACING_05, TypeStyle,
    },
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, Hsla, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _, px, relative,
};

/// `index.d.ts` `BubbleProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BubbleVariant {
    /// The user's message (`chat-bubble-user`).
    #[default]
    Default,
    /// The assistant's message (`chat-bubble-agent`).
    Secondary,
    /// `layer-01`.
    Muted,
    /// `notification-background-info`.
    Tinted,
    /// Transparent with a `border-strong-01` line.
    Outline,
    /// Transparent, no inline padding.
    Ghost,
    /// `notification-background-error` with a `support-error` line.
    Destructive,
}

impl BubbleVariant {
    /// Every variant, in `index.d.ts` order.
    pub const ALL: [BubbleVariant; 7] = [
        Self::Default,
        Self::Secondary,
        Self::Muted,
        Self::Tinted,
        Self::Outline,
        Self::Ghost,
        Self::Destructive,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Secondary => "secondary",
            Self::Muted => "muted",
            Self::Tinted => "tinted",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
        }
    }
}

/// `index.d.ts` `BubbleProps.align` (also `BubbleReactions.align`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BubbleAlign {
    /// The other party's side (default): left, bottom-left corner square.
    #[default]
    Start,
    /// The sender's side: right, bottom-right corner square.
    End,
}

impl BubbleAlign {
    /// Both alignments.
    pub const ALL: [BubbleAlign; 2] = [Self::Start, Self::End];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// `BubbleReactions.side`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReactionsSide {
    /// Above the top edge.
    Top,
    /// Below the bottom edge (default).
    #[default]
    Bottom,
}

impl ReactionsSide {
    /// Both sides.
    pub const ALL: [ReactionsSide; 2] = [Self::Top, Self::Bottom];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// The colors of one bubble variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BubbleLook {
    /// Fill (transparent for outline/ghost).
    pub background: Hsla,
    /// Text color.
    pub text: Hsla,
    /// The 1px inset outline, if any.
    pub outline: Option<Hsla>,
    /// Inline padding is dropped (`.cu-bubble--ghost { padding-left: 0; padding-right: 0 }`).
    pub flush: bool,
}

impl BubbleLook {
    /// The look of `variant` on the active theme (`.cu-bubble--<variant>`).
    pub fn of(variant: BubbleVariant) -> Self {
        use ColorRole::*;
        let clear = Hsla::transparent_black();
        let (background, text, outline) = match variant {
            BubbleVariant::Default => (
                color(ChatBubbleUser),
                color(ChatBubbleUserText),
                Some(color(ChatBubbleBorder)),
            ),
            BubbleVariant::Secondary => (
                color(ChatBubbleAgent),
                color(ChatBubbleAgentText),
                Some(color(ChatBubbleBorder)),
            ),
            BubbleVariant::Muted => (color(Layer01), color(TextPrimary), None),
            BubbleVariant::Tinted => (color(NotificationBackgroundInfo), color(TextPrimary), None),
            BubbleVariant::Outline => (clear, color(TextPrimary), Some(color(BorderStrong01))),
            BubbleVariant::Ghost => (clear, color(TextPrimary), None),
            BubbleVariant::Destructive => (
                color(NotificationBackgroundError),
                color(TextPrimary),
                Some(color(SupportError)),
            ),
        };
        Self {
            background,
            text,
            outline,
            flush: variant == BubbleVariant::Ghost,
        }
    }
}

/// `.cu-bubble { max-width: min(36rem, …) }` in px.
pub const MAX_WIDTH: f32 = 576.;
/// `.cu-bubble { max-width: min(…, 85%) }`.
pub const MAX_FRACTION: f32 = 0.85;
/// `.cu-bubble-reactions--top/bottom { top/bottom: -0.75rem }` in px.
pub const REACTIONS_OVERHANG: f32 = 12.;

/// The bubble's corner radii in px, `[top-left, top-right, bottom-right,
/// bottom-left]`: `.cu-bubble { border-radius: border-radius-08 }` with
/// `.cu-bubble--end { border-bottom-right-radius: border-radius-00 }` /
/// `.cu-bubble--start { border-bottom-left-radius: border-radius-00 }`.
pub fn corner_radii(align: BubbleAlign) -> [f32; 4] {
    let (r, square) = (BORDER_RADIUS_08.px_value(), BORDER_RADIUS_00.px_value());
    match align {
        BubbleAlign::End => [r, r, square, r],
        BubbleAlign::Start => [r, r, r, square],
    }
}

/// A chat bubble. See the module docs.
#[derive(IntoElement, Default)]
pub struct Bubble {
    variant: BubbleVariant,
    align: BubbleAlign,
    fill_message: bool,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl Bubble {
    /// A default (user) bubble at the start.
    pub fn new() -> Self {
        Self::default()
    }

    /// The variant.
    pub fn variant(mut self, variant: BubbleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The side it sits on (its square corner).
    pub fn align(mut self, align: BubbleAlign) -> Self {
        self.align = align;
        self
    }

    /// Inside a `MessageContent`: cap at 100% instead of `min(36rem, 85%)`.
    pub fn fill_message(mut self, fill: bool) -> Self {
        self.fill_message = fill;
        self
    }
}

impl Styled for Bubble {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Bubble {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Bubble {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let look = BubbleLook::of(self.variant);
        let [tl, tr, br, bl] = corner_radii(self.align).map(rem);
        let round = move |el: gpui_kit::Div| {
            el.rounded_tl(tl)
                .rounded_tr(tr)
                .rounded_br(br)
                .rounded_bl(bl)
        };
        let mut bubble = round(div())
            .relative()
            .flex()
            .flex_col()
            .min_w_0()
            .when(!self.fill_message, |this| this.max_w(rem(MAX_WIDTH)))
            .py(SPACING_04)
            .when(!look.flush, |this| this.px(SPACING_05))
            .bg(look.background)
            .text_color(look.text)
            .type_style(TypeStyle::Body01)
            .children(self.children)
            .when_some(look.outline, |this, line| {
                // `outline` follows `border-radius`.
                this.child(
                    round(div())
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_1()
                        .border_color(line),
                )
            });
        gpui_kit::Refineable::refine(bubble.style(), &self.style);
        // `max-width: min(36rem, 85%)`: the 85% lives on a wrapper that takes
        // the bubble's `align-self`. The bubble stretches across the wrapper
        // (up to its 36rem), so the pair sizes like one CSS box: shrunk to
        // its content in a flex column, filling the width in a block (a
        // `MessageScrollerItem`), where `align-self` does nothing.
        div()
            .flex()
            .flex_col()
            .min_w_0()
            .map(|this| match self.align {
                BubbleAlign::Start => this.self_start(),
                BubbleAlign::End => this.self_end(),
            })
            .map(|this| {
                if self.fill_message {
                    this.max_w_full()
                } else {
                    this.max_w(relative(MAX_FRACTION))
                }
            })
            .child(bubble)
    }
}

/// Rich bubble content: a column (`.cu-bubble-content`; first/last child
/// margins are reset, which GPUI has none of).
#[derive(IntoElement, Default)]
pub struct BubbleContent {
    children: Vec<AnyElement>,
}

impl BubbleContent {
    /// Empty content.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for BubbleContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BubbleContent {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_col().children(self.children)
    }
}

/// Bubbles stacked 4px apart (`.cu-bubble-group { gap: spacing-02 }`).
#[derive(IntoElement, Default)]
pub struct BubbleGroup {
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl BubbleGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Styled for BubbleGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for BubbleGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BubbleGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut group = div()
            .flex()
            .flex_col()
            .gap(SPACING_02)
            .children(self.children);
        gpui_kit::Refineable::refine(group.style(), &self.style);
        group
    }
}

/// The reaction pill on a bubble's edge. See the module docs.
#[derive(IntoElement, Default)]
pub struct BubbleReactions {
    side: ReactionsSide,
    align: Option<BubbleAlign>,
    children: Vec<AnyElement>,
}

impl BubbleReactions {
    /// Reactions at the bottom end (the reference defaults).
    pub fn new() -> Self {
        Self::default()
    }

    /// Top or bottom edge.
    pub fn side(mut self, side: ReactionsSide) -> Self {
        self.side = side;
        self
    }

    /// Start or end (default end).
    pub fn align(mut self, align: BubbleAlign) -> Self {
        self.align = Some(align);
        self
    }
}

impl ParentElement for BubbleReactions {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BubbleReactions {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let overhang = px(-REACTIONS_OVERHANG);
        let r = BORDER_RADIUS_16;
        div()
            .absolute()
            .map(|this| match self.side {
                ReactionsSide::Top => this.top(overhang),
                ReactionsSide::Bottom => this.bottom(overhang),
            })
            .map(|this| match self.align.unwrap_or(BubbleAlign::End) {
                BubbleAlign::End => this.right(SPACING_03),
                BubbleAlign::Start => this.left(SPACING_03),
            })
            .flex()
            .flex_row()
            .gap(px(2.))
            .px(SPACING_02)
            .rounded(r)
            .bg(color(ColorRole::Layer02))
            .text_color(color(ColorRole::TextPrimary))
            .text_size(rem(12.))
            .line_height(rem(20.))
            .children(self.children)
            // `outline: 1px solid border-subtle-01` (outside the box).
            .child(
                div()
                    .absolute()
                    .top(px(-1.))
                    .left(px(-1.))
                    .right(px(-1.))
                    .bottom(px(-1.))
                    .rounded(r)
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn variants_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let user = BubbleLook::of(BubbleVariant::Default);
        assert_eq!(user.background, color(ColorRole::ChatBubbleUser));
        assert_eq!(user.text, color(ColorRole::ChatBubbleUserText));
        assert_eq!(user.outline, Some(color(ColorRole::ChatBubbleBorder)));
        let agent = BubbleLook::of(BubbleVariant::Secondary);
        assert_eq!(agent.background, color(ColorRole::ChatBubbleAgent));
        assert_eq!(agent.text, color(ColorRole::ChatBubbleAgentText));
        assert_eq!(
            BubbleLook::of(BubbleVariant::Muted).background,
            color(ColorRole::Layer01)
        );
        assert_eq!(
            BubbleLook::of(BubbleVariant::Tinted).background,
            color(ColorRole::NotificationBackgroundInfo)
        );
        let outline = BubbleLook::of(BubbleVariant::Outline);
        assert_eq!(outline.background.a, 0.);
        assert_eq!(outline.outline, Some(color(ColorRole::BorderStrong01)));
        assert!(BubbleLook::of(BubbleVariant::Ghost).flush);
        let error = BubbleLook::of(BubbleVariant::Destructive);
        assert_eq!(
            error.background,
            color(ColorRole::NotificationBackgroundError)
        );
        assert_eq!(error.outline, Some(color(ColorRole::SupportError)));
    }

    #[test]
    fn geometry_follows_custom_css() {
        assert_eq!((MAX_WIDTH, MAX_FRACTION), (576., 0.85));
        assert_eq!((SPACING_04.px_value(), SPACING_05.px_value()), (12., 16.));
        assert_eq!(REACTIONS_OVERHANG, 12.);
        assert_eq!(SPACING_02.px_value(), 4.);
    }

    #[test]
    fn corners_follow_custom_css_not_the_t3_craft_pass() {
        // `.cu-bubble { border-radius: border-radius-08 }`, square tail corner.
        assert_eq!(corner_radii(BubbleAlign::End), [8., 8., 0., 8.]);
        assert_eq!(corner_radii(BubbleAlign::Start), [8., 8., 8., 0.]);
    }
}
