//! T3 · Composer › ReadOnlyComposerBar: the agent-started thread bar
//! (`components/ReadOnlyComposerBar/preview.html`) and a configurable one.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::ReadOnlyComposerBar;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ReadOnlyPage {
        status: true,
        action: true,
        clicks: 0,
    })
    .into()
}

struct ReadOnlyPage {
    status: bool,
    action: bool,
    clicks: usize,
}

fn open_parent(id: &'static str) -> Button {
    Button::new(id, "Open parent thread")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
}

/// The reference preview.
fn reference() -> ReadOnlyComposerBar {
    ReadOnlyComposerBar::new("This thread was started by an agent")
        .status("Read-only")
        .action(open_parent("ro-ref-action"))
}

impl Render for ReadOnlyPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut bar = ReadOnlyComposerBar::new("This thread is read-only");
        if self.status {
            bar = bar.status("Archived");
        }
        if self.action {
            let entity = entity.clone();
            bar = bar.action(open_parent("ro-live-action").on_click(move |_, _, cx| {
                entity.update(cx, |this, cx| {
                    this.clicks += 1;
                    cx.notify();
                })
            }));
        }
        let note: SharedString = format!("action clicked {} times", self.clicks).into();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(bar)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(note),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "ro-status",
                "status",
                self.status,
                controls::setter(&entity, |this: &mut Self, on| this.status = on),
            ))
            .child(controls::toggle(
                "ro-action",
                "action",
                self.action,
                controls::setter(&entity, |this: &mut Self, on| this.action = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(ReadOnlyComposerBar::new("This thread is read-only").status("Read-only"))
        .into_any_element()
}
