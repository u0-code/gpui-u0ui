//! T3 · Pull requests & issues › GuideRail: the reference route (stop 2
//! current) and a live rail whose stops are clickable.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::t3::review::{GuideRail, GuideStop};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { current: 1 }).into()
}

struct Page {
    current: usize,
}

/// `components/GuideRail/preview.html`.
fn stops() -> [GuideStop; 4] {
    [
        GuideStop::new("Token source", 2, "Where values come from"),
        GuideStop::new("Theme switching", 3, "data-theme plumbing"),
        GuideStop::new("Contrast fixes", 4, "Two helper text tokens"),
        GuideStop::new("Tests", 2, "New contrast assertions"),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            GuideRail::new("gr-thumb")
                .stops(stops().into_iter().take(3))
                .current(1),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let config = controls::field(
            "current",
            controls::choices(
                "gr-current",
                &["1", "2", "3", "4"],
                self.current,
                controls::setter(&entity, |this: &mut Page, i| this.current = i),
            ),
        );
        Body::new()
            .playground(
                GuideRail::new("gr-live")
                    .stops(stops())
                    .current(self.current)
                    .on_select(controls::setter(&entity, |this: &mut Page, i| {
                        this.current = i
                    })),
            )
            .hint("Click a stop or press Enter on it to make it current")
            .config(config)
            .example(controls::block(
                "Reference preview",
                GuideRail::new("gr-ref").stops(stops()).current(1),
            ))
    }
}
