//! T3 · Pull requests & issues › PullRequestStack: the reference stack of
//! four layers (#482 current) and a live stack whose current layer is picked.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::t3::review::{ChecksStatus, PullRequestStack, PullRequestState, StackLayer};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { current: 2 }).into()
}

struct Page {
    current: usize,
}

/// `components/PullRequestStack/preview.html`.
fn layers() -> [StackLayer; 4] {
    [
        StackLayer::new(470, "Upgrade Carbon", PullRequestState::Merged)
            .checks(ChecksStatus::Passed),
        StackLayer::new(479, "Theme plumbing", PullRequestState::Open).checks(ChecksStatus::Passed),
        StackLayer::new(482, "Dark theme tokens", PullRequestState::Open)
            .checks(ChecksStatus::Running),
        StackLayer::new(485, "Cover update", PullRequestState::Draft),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(PullRequestStack::new().layers(layers()).current(482))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let numbers = [470, 479, 482, 485];
        let config = controls::field(
            "current",
            controls::choices(
                "prs-current",
                &["#470", "#479", "#482", "#485"],
                self.current,
                controls::setter(&entity, |this: &mut Page, i| this.current = i),
            ),
        );
        Body::new()
            .playground(
                PullRequestStack::new()
                    .layers(layers())
                    .current(numbers[self.current]),
            )
            .config(config)
            .example(controls::block(
                "Reference preview",
                PullRequestStack::new().layers(layers()).current(482),
            ))
    }
}
