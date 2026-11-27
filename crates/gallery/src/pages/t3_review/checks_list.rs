//! T3 · Pull requests & issues › ChecksList: the reference checks (one
//! failing, one running) and a live list whose check states are set here.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{Check, ChecksList, ChecksStatus};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        states: [0, 1, 2, 0],
    })
    .into()
}

struct Page {
    states: [usize; 4],
}

const NAMES: [(&str, &str); 4] = [
    ("build", "2m 10s"),
    ("unit tests", "4m 02s"),
    ("visual regression", "1m"),
    ("lint", "38s"),
];

fn checks(states: [usize; 4]) -> Vec<Check> {
    NAMES
        .iter()
        .zip(states)
        .map(|((name, d), s)| Check::new(*name, ChecksStatus::ALL[s], *d))
        .collect()
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(ChecksList::new("cl-thumb").checks(checks([0, 1, 2, 0]).into_iter().take(3)))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut config = div().flex().flex_col().gap(tokens::SPACING_05);
        for (i, (name, _)) in NAMES.iter().enumerate() {
            config = config.child(controls::field(
                *name,
                controls::choices(
                    ("cl-state", i),
                    &["passed", "failed", "running", "none"],
                    self.states[i],
                    controls::setter(&entity, move |this: &mut Page, v| this.states[i] = v),
                ),
            ));
        }
        // The list is flat and takes its container's width (`app-r3`:
        // `.t3-checklist { width: auto; box-shadow: none }`).
        Body::new()
            .playground(
                div()
                    .w(rems(22.))
                    .child(ChecksList::new("cl-live").checks(checks(self.states))),
            )
            .config(config)
            .example(controls::block(
                "Reference preview",
                ChecksList::new("cl-ref").checks(checks([0, 1, 2, 0])),
            ))
    }
}
