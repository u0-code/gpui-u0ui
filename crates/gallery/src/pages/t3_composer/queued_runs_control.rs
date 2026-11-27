//! T3 · Composer › QueuedRunsControl: three queued messages, one being
//! edited and one that failed (`components/QueuedRunsControl/preview.html`).
//! The playground's queue is live: Send now and Remove take the message
//! out, Edit marks it as being edited.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{QueuedRun, QueuedRunsControl};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| QueuePage {
        items: reference_items(),
        last: "".into(),
    })
    .into()
}

struct QueuePage {
    items: Vec<QueuedRun>,
    last: SharedString,
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
        let config = div().flex().flex_col().gap(tokens::SPACING_06).child(
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
