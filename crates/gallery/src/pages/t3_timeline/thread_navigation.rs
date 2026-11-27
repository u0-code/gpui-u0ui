//! T3 · Timeline › ThreadNavigation: the find bar ("contrast", 2 of 7), the
//! jump-to-latest control with and without unread messages, and the
//! minimap with seven marks (`components/ThreadNavigation/preview.html`).
//! In the playground the find bar steps through matches (Enter, Shift+Enter,
//! the arrows), Escape closes it, and minimap marks move the viewport.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{
    MinimapKind, MinimapMark, ScrollToEndButton, ThreadFindBar, TimelineMinimap,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| NavPage {
        current: 2,
        total: 7,
        open: true,
        unread: 3,
        viewport: (0.6, 0.85),
        last: "".into(),
    })
    .into()
}

struct NavPage {
    current: usize,
    total: usize,
    open: bool,
    unread: usize,
    viewport: (f32, f32),
    last: SharedString,
}

fn marks() -> Vec<MinimapMark> {
    vec![
        MinimapMark::new(0.05, MinimapKind::User, "Your message"),
        MinimapMark::new(0.2, MinimapKind::Tool, "Work"),
        MinimapMark::new(0.35, MinimapKind::Approval, "Approval"),
        MinimapMark::new(0.5, MinimapKind::User, "Your message"),
        MinimapMark::new(0.62, MinimapKind::Find, "Match"),
        MinimapMark::new(0.78, MinimapKind::Error, "Error"),
        MinimapMark::new(0.9, MinimapKind::User, "Your message"),
    ]
}

/// The reference preview: a 32px-gap row of the find / jump stack and the minimap.
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_07)
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(tokens::SPACING_05)
                .child(ThreadFindBar::new("nav-ref-find", "contrast").matches(2, 7))
                .child(ScrollToEndButton::new("nav-ref-end-0", 3))
                .child(ScrollToEndButton::new("nav-ref-end-1", 0)),
        )
        .child(TimelineMinimap::new("nav-ref-map", marks()))
}

impl Render for NavPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let step = |d: isize| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    if this.total > 0 {
                        let n = this.total as isize;
                        this.current = ((this.current as isize - 1 + d).rem_euclid(n) + 1) as usize;
                    }
                    cx.notify();
                })
            }
        };
        let find = if self.open {
            ThreadFindBar::new("nav-live-find", "contrast")
                .matches(self.current, self.total)
                .on_next(step(1))
                .on_previous(step(-1))
                .on_query({
                    let entity = entity.clone();
                    move |q, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.total = if q.is_empty() { 0 } else { 7 };
                            this.current = usize::from(this.total > 0);
                            cx.notify();
                        })
                    }
                })
                .on_close({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.open = false;
                            cx.notify();
                        })
                    }
                })
                .into_any_element()
        } else {
            Button::new("nav-live-reopen", "Find in thread")
                .variant(ButtonVariant::Tertiary)
                .size(ButtonSize::Sm)
                .icon(CarbonIcon::Search)
                .on_click({
                    let entity = entity.clone();
                    move |_, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.open = true;
                            cx.notify();
                        })
                    }
                })
                .into_any_element()
        };
        let all = marks();
        let live = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_start()
            .gap(tokens::SPACING_07)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(tokens::SPACING_05)
                    .child(find)
                    .child(
                        ScrollToEndButton::new("nav-live-end", self.unread).on_click({
                            let entity = entity.clone();
                            move |_, cx| {
                                entity.update(cx, |this, cx| {
                                    this.unread = 0;
                                    this.viewport = (0.75, 1.);
                                    this.last = "Jumped to latest".into();
                                    cx.notify();
                                })
                            }
                        }),
                    )
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(self.last.clone()),
                    ),
            )
            .child(
                TimelineMinimap::new("nav-live-map", all.clone())
                    .viewport(self.viewport.0, self.viewport.1)
                    .on_jump({
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                let at = all[i].at;
                                this.viewport = ((at - 0.1).max(0.), (at + 0.15).min(1.));
                                this.last = format!("onJump: {}", all[i].label).into();
                                cx.notify();
                            })
                        }
                    }),
            );
        let unread = ["0", "3", "12"];
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "ScrollToEndButton unread",
                controls::choices(
                    "nav-unread",
                    &unread,
                    match self.unread {
                        0 => 0,
                        3 => 1,
                        _ => 2,
                    },
                    controls::setter(&entity, |this: &mut Self, i| {
                        this.unread = [0, 3, 12][i];
                    }),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("In the find field: Enter next, Shift+Enter previous, Escape closes.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(tokens::SPACING_05)
        .child(ScrollToEndButton::new("nav-th-end", 3))
        .child(TimelineMinimap::new("nav-th-map", marks()))
        .into_any_element()
}
