//! T3 · Timeline › WorkingIndicator: working, thinking, compacting and
//! setup, each ticking from its start (`components/WorkingIndicator/preview.html`).
//! The shimmer stops under reduced motion.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{WorkingIndicator, WorkingState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| WorkingPage {
        state: 0,
        label: false,
    })
    .into()
}

struct WorkingPage {
    state: usize,
    label: bool,
}

/// The reference preview (`cu-stack`, gap 12).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .child(WorkingIndicator::new("wi-ref-0", WorkingState::Working, 74))
        .child(WorkingIndicator::new("wi-ref-1", WorkingState::Thinking, 8))
        .child(WorkingIndicator::new(
            "wi-ref-2",
            WorkingState::Compacting,
            21,
        ))
        .child(WorkingIndicator::new("wi-ref-3", WorkingState::Setup, 5))
}

impl Render for WorkingPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let state = WorkingState::ALL[self.state];
        let mut indicator = WorkingIndicator::new(
            SharedString::from(format!("wi-live-{}", self.state)),
            state,
            0,
        );
        if self.label {
            indicator = indicator.label("Reviewing 14 files");
        }
        let states = WorkingState::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "wi-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ))
            .child(controls::toggle(
                "wi-label",
                "label",
                self.label,
                controls::setter(&entity, |this: &mut Self, on| this.label = on),
            ));
        Body::new()
            .playground(indicator)
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    WorkingIndicator::new("wi-th", WorkingState::Working, 74).into_any_element()
}
