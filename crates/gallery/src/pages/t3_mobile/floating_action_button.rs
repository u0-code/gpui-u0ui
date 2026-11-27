//! T3 · Mobile › FloatingActionButton: the reference pair (icon-only and
//! extended "New thread") and a live FAB.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::mobile::FloatingActionButton;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { extended: false }).into()
}

struct Page {
    extended: bool,
}

fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .child(FloatingActionButton::new("fab-thumb-0", "New thread"))
        .child(FloatingActionButton::new("fab-thumb-1", "New thread").extended(true))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        Body::new()
            .playground(FloatingActionButton::new("fab-live", "New thread").extended(self.extended))
            .config(controls::toggle(
                "fab-extended",
                "extended",
                self.extended,
                controls::setter(&entity, |this: &mut Page, v| this.extended = v),
            ))
            .example(controls::block(
                "Reference preview",
                row()
                    .child(FloatingActionButton::new("fab-ref-0", "New thread"))
                    .child(FloatingActionButton::new("fab-ref-1", "New thread").extended(true)),
            ))
    }
}
