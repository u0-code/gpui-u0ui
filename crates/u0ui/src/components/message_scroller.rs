//! MessageScroller: a chat transcript that follows new messages
//! (`.cu-scroller*` in `custom.css`, `MessageScroller` in `index.jsx`).
//!
//! - **Behavior**: a [`ScrollArea`] (Base scrollbar, Carbon thumb) over a
//!   caller-sized viewport. While the view is within [`STICK_THRESHOLD`] of the
//!   end it follows new content (`scrollHeight - scrollTop - clientHeight <
//!   24`, then `scrollTop = scrollHeight` after every render); scrolling up past
//!   that stops following and shows the "Jump to latest" button, which scrolls
//!   to the end smoothly ([`JUMP`], moderate-02 productive standard; reduced
//!   motion jumps). Kit's `MessageScroller` is not used: it is a virtual list
//!   over an entity-owned row renderer with Kit's own button and a bottom fade,
//!   while the reference takes ordinary children and a Carbon button.
//! - **Appearance**: `.cu-scroller { background: chat-shell-background;
//!   outline: 1px solid border-subtle-01; outline-offset: -1px }`, content
//!   `.cu-scroller-content { display: flex; flex-direction: column; gap:
//!   spacing-06; padding: spacing-05 }`; the jump button is a Carbon secondary
//!   `sm` button with the `ArrowDown` icon, `bottom: spacing-05`, centered, with
//!   the floating `0 2px 6px shadow`. It appears and disappears without motion
//!   (the reference renders it conditionally).
//! - `role="log"` / `aria-live` have no GPUI equivalent beyond the region's
//!   accessible name ([`MessageScroller::label`]).
use super::button::{Button, ButtonSize, ButtonVariant};
use super::overlay::floating_shadow;
use super::scroll_area::{ScrollArea, ScrollAxes};
use crate::{
    icons::CarbonIcon,
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, SPACING_05, SPACING_06},
};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, Length, ParentElement,
    RenderOnce, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    point, prelude::FluentBuilder as _, px,
};
use std::time::Instant;

/// `el.scrollHeight - el.scrollTop - el.clientHeight < 24`.
pub const STICK_THRESHOLD: f32 = 24.;
/// The "Jump to latest" scroll: smooth, moderate-02 productive standard, spatial.
pub const JUMP: Motion = Motion::new(Curve::ProductiveStandard, Speed::Moderate02).spatial(true);
/// `MessageScroller.height` default (`height = 320`).
pub const DEFAULT_HEIGHT: f32 = 320.;

/// Whether a view `distance` px above the end counts as at the end.
pub fn at_end(distance: f32) -> bool {
    distance < STICK_THRESHOLD
}

/// The offset (GPUI, ≤ 0) of a jump from `from` to `to` at `elapsed` of
/// `duration`, eased with [`JUMP`]'s curve; `None` once finished.
pub fn jump_offset(from: f32, to: f32, elapsed: f32, duration: f32) -> Option<f32> {
    if duration <= 0. || elapsed >= duration {
        return None;
    }
    let t = JUMP.curve.ease((elapsed / duration).clamp(0., 1.));
    Some(from + (to - from) * t)
}

/// One transcript entry (`MessageScrollerItem`, `messageId`): a column, so a
/// bubble inside can align itself.
#[derive(IntoElement)]
pub struct MessageScrollerItem {
    id: ElementId,
    children: Vec<AnyElement>,
}

impl MessageScrollerItem {
    /// An entry keyed by its message id.
    pub fn new(message_id: impl Into<ElementId>) -> Self {
        Self {
            id: message_id.into(),
            children: Vec::new(),
        }
    }
}

impl ParentElement for MessageScrollerItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageScrollerItem {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().id(self.id).flex().flex_col().children(self.children)
    }
}

#[derive(Default)]
struct ScrollerState {
    handle: ScrollHandle,
    /// A smooth jump in progress: (start time, start offset).
    jump: Option<(Instant, f32)>,
}

/// The transcript. See the module docs.
#[derive(IntoElement)]
pub struct MessageScroller {
    id: ElementId,
    auto_scroll: bool,
    height: Length,
    jump_label: Option<SharedString>,
    label: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl MessageScroller {
    /// A following transcript, 320px tall.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            auto_scroll: true,
            height: px(DEFAULT_HEIGHT).into(),
            jump_label: None,
            label: None,
            children: Vec::new(),
        }
    }

    /// `autoScroll` (default true): follow new content while at the end.
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// `height` (default 320px).
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// The jump button's label (the reference's "Jump to latest"); without
    /// one the button is not shown.
    pub fn jump_label(mut self, label: impl Into<SharedString>) -> Self {
        self.jump_label = Some(label.into());
        self
    }

    /// The region's accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl ParentElement for MessageScroller {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MessageScroller {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = window.use_keyed_state((id.clone(), "scroller"), cx, |_, _| {
            ScrollerState::default()
        });
        let (handle, jump) = {
            let s = state.read(cx);
            (s.handle.clone(), s.jump)
        };
        // Distance from the end, from the last layout.
        let max = f32::from(handle.max_offset().y);
        let offset = f32::from(handle.offset().y);
        let distance = (max + offset).max(0.);
        let duration = JUMP.speed.duration().as_secs_f32();

        if let Some((start, from)) = jump {
            match jump_offset(from, -max, start.elapsed().as_secs_f32(), duration) {
                Some(y) => {
                    handle.set_offset(point(handle.offset().x, px(y)));
                    window.request_animation_frame();
                }
                None => {
                    handle.scroll_to_bottom();
                    state.update(cx, |s, _| s.jump = None);
                }
            }
        } else if self.auto_scroll && at_end(distance) {
            // `useEffect(() => { if (autoScroll && atEnd) el.scrollTop = el.scrollHeight })`.
            handle.scroll_to_bottom();
        }
        let show_jump = !at_end(distance) && jump.is_none();

        let jump_button = self.jump_label.filter(|_| show_jump).map(|label| {
            let state = state.clone();
            let handle = handle.clone();
            // `.cu-scroller-jump { position: absolute; left: 50%; bottom: spacing-05;
            // transform: translateX(-50%); box-shadow: 0 2px 6px var(--shadow) }`.
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom(SPACING_05)
                .flex()
                .justify_center()
                .child(
                    div().shadow(floating_shadow()).child(
                        Button::new((id.clone(), "jump"), label)
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .icon(CarbonIcon::ArrowDown)
                            .on_click(move |_, window, cx| {
                                if motion::reduced_motion(cx) {
                                    handle.scroll_to_bottom();
                                } else {
                                    let from = f32::from(handle.offset().y);
                                    state.update(cx, |s, _| s.jump = Some((Instant::now(), from)));
                                }
                                window.refresh();
                            }),
                    ),
                )
        });

        div()
            .id(id.clone())
            .when_some(self.label, |this, label| this.aria_label(label))
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .h(self.height)
            .bg(color(ColorRole::ChatShellBackground))
            .child(
                ScrollArea::new((id.clone(), "viewport"))
                    .axes(ScrollAxes::Vertical)
                    .focusable(false)
                    .track_scroll(&handle)
                    .size_full()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(SPACING_06)
                            .p(SPACING_05)
                            .children(self.children),
                    ),
            )
            // `outline: 1px solid border-subtle-01; outline-offset: -1px`.
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01)),
            )
            .children(jump_button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_within_the_threshold() {
        assert!(at_end(0.));
        assert!(at_end(23.9));
        assert!(!at_end(24.));
        assert_eq!(STICK_THRESHOLD, 24.);
    }

    #[test]
    fn jump_eases_to_the_end() {
        assert_eq!(JUMP.speed.millis(), 240);
        const { assert!(JUMP.spatial) };
        assert_eq!(jump_offset(-100., -500., 0., 0.24), Some(-100.));
        let mid = jump_offset(-100., -500., 0.12, 0.24).unwrap();
        assert!(mid < -100. && mid > -500.);
        assert_eq!(jump_offset(-100., -500., 0.24, 0.24), None);
    }
}
