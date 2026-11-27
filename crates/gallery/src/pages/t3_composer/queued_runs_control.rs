//! T3 · Composer › QueuedRunsControl: three queued messages, one being
//! edited and one that failed (`components/QueuedRunsControl/preview.html`).
//! The playground's queue is live: Send now and Remove take the message
//! out, Edit marks it as being edited, Earlier moves it up. Controls turn on
//! the u0 board `ThreadComposer` options (`.tc-later*`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{QueuedRun, QueuedRunsControl};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| QueuePage {
        items: reference_items(),
        last: "".into(),
        board: false,
        collapsible: true,
        numbered: true,
        single_line: false,
        earlier: false,
    })
    .into()
}

struct QueuePage {
    items: Vec<QueuedRun>,
    last: SharedString,
    /// The board's head: Time icon, the caller's text, a hairline frame.
    board: bool,
    collapsible: bool,
    numbered: bool,
    single_line: bool,
    earlier: bool,
}

/// The u0 board's two waiting messages.
fn board_items() -> Vec<QueuedRun> {
    vec![
        QueuedRun::new("Also update the README badges and the changelog for the release"),
        QueuedRun::new("Then open a pull request against main").note("Saved"),
        QueuedRun::new("Run the visual regression suite")
            .error(true)
            .note("Could not send"),
    ]
}

/// The u0 board `ThreadComposer` queue: Time head, not collapsible,
/// unnumbered rows on one line, Earlier on every row but the first.
fn board(id: &'static str, items: Vec<QueuedRun>) -> QueuedRunsControl {
    let n = items.len();
    QueuedRunsControl::new(id, items)
        .icon(CarbonIcon::Time)
        .title(format!(
            "{n} wait their turn · sent in order when Claude’s turn ends"
        ))
        .collapsible(false)
        .numbered(false)
        .single_line(true)
        .bordered(true)
}

/// The preview's items.
fn reference_items() -> Vec<QueuedRun> {
    vec![
        QueuedRun::new("Also update the README badges"),
        QueuedRun::new("Then open a PR against main").editing(true),
        QueuedRun::new("Run the visual regression suite").error(true),
    ]
}

impl Render for QueuePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let update = |f: fn(&mut QueuePage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(38.75))
            .child(
                QueuedRunsControl::new("queue-live", self.items.clone())
                    .when(self.board, |this| {
                        let n = self.items.len();
                        this.icon(CarbonIcon::Time)
                            .title(format!(
                                "{n} wait their turn · sent in order when Claude’s turn ends"
                            ))
                            .bordered(true)
                    })
                    .collapsible(self.collapsible)
                    .numbered(self.numbered)
                    .single_line(self.single_line)
                    .when(self.earlier, |this| {
                        this.on_earlier(update(|this, i| {
                            this.items.swap(i - 1, i);
                            this.last = format!("Earlier: {i} → {}", i - 1).into();
                        }))
                    })
                    .on_send_now(update(|this, i| {
                        let item = this.items.remove(i);
                        this.last = format!("Sent: {}", item.text).into();
                    }))
                    .on_edit(update(|this, i| {
                        for (k, item) in this.items.iter_mut().enumerate() {
                            item.editing = k == i;
                        }
                        this.last = "Editing in the composer".into();
                    }))
                    .on_remove(update(|this, i| {
                        let item = this.items.remove(i);
                        this.last = format!("Removed: {}", item.text).into();
                    })),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "queue-board",
                "Time head, caller's text, frame",
                self.board,
                controls::setter(&entity, |this: &mut Self, on| this.board = on),
            ))
            .child(controls::toggle(
                "queue-collapsible",
                "collapsible",
                self.collapsible,
                controls::setter(&entity, |this: &mut Self, on| this.collapsible = on),
            ))
            .child(controls::toggle(
                "queue-numbered",
                "numbered",
                self.numbered,
                controls::setter(&entity, |this: &mut Self, on| this.numbered = on),
            ))
            .child(controls::toggle(
                "queue-single",
                "single_line",
                self.single_line,
                controls::setter(&entity, |this: &mut Self, on| this.single_line = on),
            ))
            .child(controls::toggle(
                "queue-earlier",
                "on_earlier",
                self.earlier,
                controls::setter(&entity, |this: &mut Self, on| this.earlier = on),
            ))
            .child(
                Button::new("queue-reset", "Reset queue")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.items = reference_items();
                                this.last = "".into();
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .hint("The header folds the list.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(38.75))
                    .child(QueuedRunsControl::new("queue-ref", reference_items())),
            ))
            .example(controls::block(
                "u0 ThreadComposer (.tc-later)",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(38.75))
                    .child(board("queue-board-ref", board_items()).on_earlier(|_, _, _| {})),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(QueuedRunsControl::new(
            "queue-th",
            vec![QueuedRun::new("Also update the README badges")],
        ))
        .into_any_element()
}
