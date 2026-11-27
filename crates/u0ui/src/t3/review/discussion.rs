//! T3 · Pull requests & issues: the discussion parts — [`ReactionBar`],
//! [`PullRequestTimeline`] and [`ReviewForm`] (`src/t3.css` "PR vocabulary").
use super::pull_request::{ReviewDecision, ReviewDecisionBadge, icon_button};
use crate::{
    components::{AvatarColor, Button, ButtonSize, ButtonVariant, Textarea, TextareaState},
    icons::{CarbonIcon, Icon},
    t3::craft::{self, Tone, focus_handle, focus_visible, press, rem, sans},
    theme::{LayerLevel, color},
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
    relative,
};
use std::rc::Rc;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type EventEmojiHandler = Rc<dyn Fn(usize, SharedString, &mut Window, &mut App)>;

/// One reaction (`reactions[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reaction {
    /// `emoji`.
    pub emoji: SharedString,
    /// `count`.
    pub count: usize,
    /// `mine`: the viewer reacted (`aria-pressed`).
    pub mine: bool,
}

impl Reaction {
    /// A reaction.
    pub fn new(emoji: impl Into<SharedString>, count: usize) -> Self {
        Self {
            emoji: emoji.into(),
            count,
            mine: false,
        }
    }

    /// The viewer's own reaction.
    pub fn mine(mut self, mine: bool) -> Self {
        self.mine = mine;
        self
    }
}

type EmojiHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// Emoji reaction pills (`.t3-react`: 24px, 16px radius, `layer-accent-01`;
/// mine = blue tag pair with a 1px `border-interactive` ring) and an Add
/// reaction icon button.
#[derive(IntoElement)]
pub struct ReactionBar {
    id: ElementId,
    reactions: Vec<Reaction>,
    on_react: Option<EmojiHandler>,
    on_add: Option<Handler>,
    add_label: SharedString,
}

impl ReactionBar {
    /// A reaction bar.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            reactions: Vec::new(),
            on_react: None,
            on_add: None,
            add_label: "Add reaction".into(),
        }
    }

    /// The reactions.
    pub fn reactions(mut self, reactions: impl IntoIterator<Item = Reaction>) -> Self {
        self.reactions = reactions.into_iter().collect();
        self
    }

    /// A pill was pressed (`onReact(emoji)`).
    pub fn on_react(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_react = Some(Rc::new(f));
        self
    }

    /// Add reaction pressed.
    pub fn on_add(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add = Some(Rc::new(f));
        self
    }

    /// The add button's name (`Add reaction`).
    pub fn add_label(mut self, label: impl Into<SharedString>) -> Self {
        self.add_label = label.into();
        self
    }
}

/// (fill, ink, ring) of a reaction pill.
pub fn reaction_look(mine: bool, hovered: bool) -> (Hsla, Hsla, Option<Hsla>) {
    if mine {
        // `.t3-react--mine` (0,1,0) after `.t3-react:hover` (0,2,0): hover wins the fill.
        (
            color(if hovered {
                ColorRole::LayerAccentHover01
            } else {
                ColorRole::TagBackgroundBlue
            }),
            color(ColorRole::TagColorBlue),
            Some(color(ColorRole::BorderInteractive)),
        )
    } else {
        (
            color(if hovered {
                ColorRole::LayerAccentHover01
            } else {
                ColorRole::LayerAccent01
            }),
            color(ColorRole::TextPrimary),
            None,
        )
    }
}

impl RenderOnce for ReactionBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(4.))
            .children(self.reactions.iter().enumerate().map(|(i, r)| {
                let rid = craft::sub(&id, "r", i);
                let focus = focus_handle(&rid, window, cx);
                let ring = focus_visible(&focus, window);
                let hovered = window
                    .use_keyed_state((rid.clone(), "hover"), cx, |_, _| false)
                    .clone();
                let (fill, ink, edge) = reaction_look(r.mine, *hovered.read(cx));
                let on_react = self.on_react.clone();
                let emoji = r.emoji.clone();
                sans(press(rid, &focus), 12., 400)
                    .gap(rem(4.))
                    .h(rem(24.))
                    .px(rem(8.))
                    .rounded(rem(16.))
                    .bg(fill)
                    .text_color(ink)
                    .when_some(edge, |this, c| this.border_1().border_color(c).px(rem(7.)))
                    .on_hover(move |h, _, cx| {
                        hovered.update(cx, |v, cx| {
                            *v = *h;
                            cx.notify();
                        })
                    })
                    .when_some(on_react, |this, f| {
                        this.on_click(move |_, w, cx| f(emoji.clone(), w, cx))
                    })
                    .child(r.emoji.clone())
                    .child(r.count.to_string())
                    .when(ring, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top(px(-2.))
                                .left(px(-2.))
                                .right(px(-2.))
                                .bottom(px(-2.))
                                .rounded(rem(16.))
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    })
            }))
            .child(icon_button(
                (id, "add"),
                self.add_label.clone(),
                CarbonIcon::FaceAdd,
                self.on_add.clone(),
            ))
    }
}

/// A timeline event kind (`events[].kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// `commit`.
    #[default]
    Commit,
    /// `label`.
    Label,
    /// `merged`.
    Merged,
    /// `push`.
    Push,
    /// `ref`.
    Ref,
    /// `comment`: a card.
    Comment,
    /// `review`: a card with a decision.
    Review,
}

impl EventKind {
    /// Every kind.
    pub const ALL: [EventKind; 7] = [
        Self::Commit,
        Self::Label,
        Self::Merged,
        Self::Push,
        Self::Ref,
        Self::Comment,
        Self::Review,
    ];

    /// Comments and reviews render as cards.
    pub const fn is_card(self) -> bool {
        matches!(self, Self::Comment | Self::Review)
    }

    /// The marker icon of a plain event (`Commit`, `Tag`, `Merge`, `Upload`, `Link`).
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Commit => CarbonIcon::Commit,
            Self::Label => CarbonIcon::Tag,
            Self::Merged => CarbonIcon::Merge,
            Self::Push => CarbonIcon::Upload,
            Self::Ref => CarbonIcon::Link,
            Self::Comment | Self::Review => CarbonIcon::CircleFilled,
        }
    }

    /// Marker ink: `icon-secondary`; merged `tag-color-purple`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Merged => ColorRole::TagColorPurple,
            _ => ColorRole::IconSecondary,
        })
    }
}

/// One timeline event.
pub struct TimelineEvent {
    kind: EventKind,
    actor: SharedString,
    text: SharedString,
    body: SharedString,
    review: Option<ReviewDecision>,
    time: SharedString,
    color: Option<AvatarColor>,
    marker: Option<AnyElement>,
    reactions: Option<Vec<Reaction>>,
}

impl TimelineEvent {
    /// An event of `kind` by `actor` at `time`.
    pub fn new(
        kind: EventKind,
        actor: impl Into<SharedString>,
        time: impl Into<SharedString>,
    ) -> Self {
        Self {
            kind,
            actor: actor.into(),
            text: SharedString::default(),
            body: SharedString::default(),
            review: None,
            time: time.into(),
            color: None,
            marker: None,
            reactions: None,
        }
    }

    /// The event kind.
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    /// The event text after the actor (plain events).
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }

    /// The card body (comments and reviews).
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = body.into();
        self
    }

    /// The review decision.
    pub fn review(mut self, review: ReviewDecision) -> Self {
        self.review = Some(review);
        self
    }

    /// The avatar color (default: a stable tone hashed from the actor).
    pub fn color(mut self, color: AvatarColor) -> Self {
        self.color = Some(color);
        self
    }

    /// A custom card marker in place of the avatar (`provider` →
    /// `ProviderIcon size="sm"` in the reference).
    pub fn marker(mut self, marker: impl IntoElement) -> Self {
        self.marker = Some(marker.into_any_element());
        self
    }

    /// Reactions under the body.
    pub fn reactions(mut self, reactions: impl IntoIterator<Item = Reaction>) -> Self {
        self.reactions = Some(reactions.into_iter().collect());
        self
    }
}

/// The PR event timeline (`.t3-prtl`): a 1px rule at x = 11px joins 24px
/// markers; plain events are one line, comments and reviews are `layer-01`
/// cards with actor, decision, time, Add to thread, body and reactions.
#[derive(IntoElement)]
pub struct PullRequestTimeline {
    id: ElementId,
    events: Vec<TimelineEvent>,
    add_label: SharedString,
    on_add_to_thread: Option<IndexHandler>,
    on_react: Option<EventEmojiHandler>,
}

impl PullRequestTimeline {
    /// A timeline.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            events: Vec::new(),
            add_label: "Add to thread".into(),
            on_add_to_thread: None,
            on_react: None,
        }
    }

    /// The events, oldest first.
    pub fn events(mut self, events: impl IntoIterator<Item = TimelineEvent>) -> Self {
        self.events = events.into_iter().collect();
        self
    }

    /// The card action text (`Add to thread`).
    pub fn add_label(mut self, label: impl Into<SharedString>) -> Self {
        self.add_label = label.into();
        self
    }

    /// Add to thread pressed on event `i`.
    pub fn on_add_to_thread(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_add_to_thread = Some(Rc::new(f));
        self
    }

    /// A reaction pill on event `i` was pressed.
    pub fn on_react(
        mut self,
        f: impl Fn(usize, SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_react = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PullRequestTimeline {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let secondary = color(ColorRole::TextSecondary);
        let items = self.events.into_iter().enumerate().map(|(i, e)| {
            let marker = if e.kind.is_card() {
                e.marker.unwrap_or_else(|| {
                    let tone = e
                        .color
                        .map(Tone::Avatar)
                        .unwrap_or_else(|| craft::actor_tone(&e.actor));
                    craft::initials(craft::two_letters(&e.actor), tone, 24.).into_any_element()
                })
            } else {
                Icon::carbon(e.kind.icon())
                    .size_length(rem(16.))
                    .color(e.kind.ink())
                    .into_any_element()
            };
            // `.t3-prtl-marker { 24×24; background: var(--background); z-index: 1 }`.
            let marker = div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(rem(24.))
                .bg(color(ColorRole::Background))
                .child(marker);
            let content = if e.kind.is_card() {
                let on_add = self.on_add_to_thread.clone();
                let on_react = self.on_react.clone();
                div()
                    .flex_1()
                    .min_w_0()
                    .bg(color(ColorRole::Layer01))
                    .child(
                        sans(div(), 12., 400)
                            .flex()
                            .items_center()
                            .gap(rem(8.))
                            .min_h(rem(40.))
                            .pl(rem(12.))
                            .pr(rem(4.))
                            .border_b_1()
                            .border_color(color(ColorRole::BorderSubtle01))
                            .text_color(secondary)
                            .child(
                                sans(div(), 14., 600)
                                    .text_color(color(ColorRole::TextPrimary))
                                    .child(e.actor.clone()),
                            )
                            .when_some(e.review, |this, r| this.child(ReviewDecisionBadge::new(r)))
                            .child(e.time.clone())
                            .child(div().flex_1())
                            .child(
                                Button::new(craft::sub(&id, "add", i), self.add_label.clone())
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Sm)
                                    .icon(CarbonIcon::Chat)
                                    .when_some(on_add, |b, f| {
                                        b.on_click(move |_, w, cx| f(i, w, cx))
                                    }),
                            ),
                    )
                    .child(
                        sans(div(), 14., 400)
                            .p(rem(12.))
                            .line_height(relative(1.42857))
                            .text_color(color(ColorRole::TextPrimary))
                            .child(e.body.clone()),
                    )
                    .when_some(e.reactions, |this, reactions| {
                        this.child(
                            div().px(rem(12.)).pb(rem(12.)).child(
                                ReactionBar::new(craft::sub(&id, "reactions", i))
                                    .reactions(reactions)
                                    .when_some(on_react, |bar, f| {
                                        bar.on_react(move |emoji, w, cx| f(i, emoji, w, cx))
                                    }),
                            ),
                        )
                    })
                    .into_any_element()
            } else {
                sans(div(), 14., 400)
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_x(rem(8.))
                    .gap_y(rem(4.))
                    .min_h(rem(24.))
                    .text_color(secondary)
                    .child(
                        sans(div(), 14., 600)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(e.actor.clone()),
                    )
                    .child(e.text.clone())
                    .child(craft::text(e.time.clone(), 12., ColorRole::TextSecondary))
                    .into_any_element()
            };
            div()
                .flex()
                .gap(rem(12.))
                .pb(rem(16.))
                .child(marker)
                .child(content)
        });
        div()
            .relative()
            .flex()
            .flex_col()
            // `.t3-prtl::before { left: 11px; width: 1px; background: border-subtle-01 }`.
            .child(
                div()
                    .absolute()
                    .left(px(11.))
                    .top_0()
                    .bottom_0()
                    .w(px(1.))
                    .bg(color(ColorRole::BorderSubtle01)),
            )
            .children(items)
    }
}

/// A review verdict (`ReviewForm` options).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReviewVerdict {
    /// `comment`, the default.
    #[default]
    Comment,
    /// `approve`.
    Approve,
    /// `changes`.
    Changes,
}

impl ReviewVerdict {
    /// Every verdict, in order.
    pub const ALL: [ReviewVerdict; 3] = [Self::Comment, Self::Approve, Self::Changes];

    /// The value passed to `onSubmit`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Approve => "approve",
            Self::Changes => "changes",
        }
    }
}

/// Texts of a [`ReviewForm`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct ReviewFormTexts {
    /// `Review summary`.
    pub summary: SharedString,
    /// `Leave a comment`.
    pub placeholder: SharedString,
    /// `Review type` (the radio group's name).
    pub group: SharedString,
    /// (title, description) per verdict.
    pub options: [(SharedString, SharedString); 3],
}

impl Default for ReviewFormTexts {
    fn default() -> Self {
        Self {
            summary: "Review summary".into(),
            placeholder: "Leave a comment".into(),
            group: "Review type".into(),
            options: [
                (
                    "Comment".into(),
                    "Submit general feedback without explicit approval.".into(),
                ),
                (
                    "Approve".into(),
                    "Submit feedback and approve merging these changes.".into(),
                ),
                (
                    "Request changes".into(),
                    "Submit feedback that must be addressed before merging.".into(),
                ),
            ],
        }
    }
}

type VerdictHandler = Rc<dyn Fn(ReviewVerdict, &mut Window, &mut App)>;

/// The next option index for an arrow key in a radio group (wraps; Home and
/// End jump), or `None` for other keys (`radioKeys` in `base.jsx`).
pub fn radio_step(key: &str, current: usize, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match key {
        "right" | "down" => (current + 1) % len,
        "left" | "up" => (current + len - 1) % len,
        "home" => 0,
        "end" => len - 1,
        _ => return None,
    })
}

/// Submit a review (`.t3-reviewform`, `layer-01`, 30rem): the summary text
/// area on `field-02`, three radio cards (Comment / Approve / Request
/// changes, each with a description) and the submit button.
#[derive(IntoElement)]
pub struct ReviewForm {
    id: ElementId,
    submit_label: SharedString,
    submit_disabled: bool,
    on_submit: Option<VerdictHandler>,
    texts: ReviewFormTexts,
}

impl ReviewForm {
    /// A review form.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            submit_label: "Submit review".into(),
            submit_disabled: false,
            on_submit: None,
            texts: ReviewFormTexts::default(),
        }
    }

    /// The button text (`Submit review`).
    pub fn submit_label(mut self, label: impl Into<SharedString>) -> Self {
        self.submit_label = label.into();
        self
    }

    /// Disable the submit button.
    pub fn submit_disabled(mut self, disabled: bool) -> Self {
        self.submit_disabled = disabled;
        self
    }

    /// Submit pressed with the chosen verdict (`onSubmit(type)`).
    pub fn on_submit(mut self, f: impl Fn(ReviewVerdict, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: ReviewFormTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// The radio circle drawn by `.t3-reviewform .t3-opt::before`: 16px, 1px
/// `icon-primary` ring; checked adds a 7px `icon-primary` dot.
fn radio_circle(checked: bool) -> gpui_kit::Div {
    let ink = color(ColorRole::IconPrimary);
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .mt(px(2.))
        .size(px(16.))
        .rounded_full()
        .border_1()
        .border_color(ink)
        .when(checked, |this| {
            this.child(div().size(px(7.)).rounded_full().bg(ink))
        })
}

impl RenderOnce for ReviewForm {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let text = window
            .use_keyed_state((id.clone(), "text"), cx, |window, cx| {
                cx.new(|cx| TextareaState::new(window, cx))
            })
            .read(cx)
            .clone();
        let verdict = window
            .use_keyed_state((id.clone(), "verdict"), cx, |_, _| ReviewVerdict::Comment)
            .clone();
        let chosen = *verdict.read(cx);
        let handles: Vec<FocusHandle> = (0..3)
            .map(|i| focus_handle(&craft::sub(&id, "opt", i), window, cx))
            .collect();
        let options = ReviewVerdict::ALL.iter().enumerate().map(|(i, v)| {
            let on = *v == chosen;
            let focus = handles[i].clone();
            let ring = focus.is_focused(window);
            let (title, desc) = self.texts.options[i].clone();
            let verdict = verdict.clone();
            let v = *v;
            let hovered = window
                .use_keyed_state(craft::sub(&id, "opt-hover", i), cx, |_, _| false)
                .clone();
            let is_hovered = *hovered.read(cx);
            press(craft::sub(&id, "opt", i), &focus)
                .role(gpui_kit::Role::RadioButton)
                .accessibility_label(title.clone())
                // Only the checked radio is a tab stop, like a native group.
                .tab_stop(on)
                .w_full()
                .items_start()
                .gap(rem(12.))
                .p(rem(12.))
                .bg(color(if is_hovered {
                    ColorRole::LayerHover02
                } else {
                    ColorRole::Layer02
                }))
                .on_hover(move |h, _, cx| {
                    hovered.update(cx, |x, cx| {
                        *x = *h;
                        cx.notify();
                    })
                })
                .on_click(move |_, _, cx| {
                    verdict.update(cx, |x, cx| {
                        *x = v;
                        cx.notify();
                    })
                })
                .child(radio_circle(on))
                .child(
                    sans(div(), 14., 400)
                        .flex()
                        .flex_col()
                        .text_color(color(ColorRole::TextPrimary))
                        .child(sans(div(), 14., 600).child(title))
                        .child(craft::text(desc, 12., ColorRole::TextSecondary).mt(px(2.))),
                )
                // `.t3-opt--on { box-shadow: inset 0 0 0 2px var(--border-interactive) }`.
                .when(on, |this| {
                    this.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .border_2()
                            .border_color(color(ColorRole::BorderInteractive)),
                    )
                })
                // `.t3-opt:focus-within { outline: 2px solid var(--focus); outline-offset: -2px }`.
                .when(ring, |this| this.child(craft::focus_ring()))
        });
        let group_keys = {
            let verdict = verdict.clone();
            let handles = handles.clone();
            move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let current = ReviewVerdict::ALL
                    .iter()
                    .position(|v| *v == *verdict.read(cx))
                    .unwrap_or(0);
                if let Some(next) = radio_step(e.keystroke.key.as_str(), current, 3) {
                    cx.stop_propagation();
                    verdict.update(cx, |x, cx| {
                        *x = ReviewVerdict::ALL[next];
                        cx.notify();
                    });
                    handles[next].focus(window, cx);
                }
            }
        };
        let on_submit = self.on_submit.clone();
        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .flex_col()
            .gap(rem(16.))
            .w_full()
            .max_w(rem(480.))
            .p(rem(16.))
            .bg(color(ColorRole::Layer01))
            .child(
                Textarea::new((id.clone(), "summary"), &text)
                    .label(self.texts.summary.clone())
                    .placeholder(self.texts.placeholder.clone())
                    .rows(3)
                    .layer(LayerLevel::Two),
            )
            .child(
                div()
                    .id((id.clone(), "group"))
                    .flex()
                    .flex_col()
                    .gap(px(1.))
                    .on_key_down(group_keys)
                    .children(options),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_end()
                    .items_center()
                    .gap(rem(8.))
                    .child(
                        Button::new((id, "submit"), self.submit_label.clone())
                            .size(ButtonSize::Sm)
                            .disabled(self.submit_disabled)
                            .when_some(on_submit, |b, f| {
                                b.on_click(move |_, w, cx| f(*verdict.read(cx), w, cx))
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_keys_wrap() {
        assert_eq!(radio_step("down", 2, 3), Some(0));
        assert_eq!(radio_step("up", 0, 3), Some(2));
        assert_eq!(radio_step("end", 0, 3), Some(2));
        assert_eq!(radio_step("tab", 0, 3), None);
    }

    #[test]
    fn reaction_and_event_colors() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        assert_eq!(EventKind::Merged.ink(), color(ColorRole::TagColorPurple));
        let (fill, ink, ring) = reaction_look(true, false);
        assert_eq!(fill, color(ColorRole::TagBackgroundBlue));
        assert_eq!(ink, color(ColorRole::TagColorBlue));
        assert_eq!(ring, Some(color(ColorRole::BorderInteractive)));
        let (fill, _, ring) = reaction_look(false, true);
        assert_eq!(fill, color(ColorRole::LayerAccentHover01));
        assert!(ring.is_none());
    }
}
