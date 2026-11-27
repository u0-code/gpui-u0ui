//! T3 · Pull requests & issues › PullRequestDetailHeader: the reference
//! header (open #482 with Check out / Open in thread) whose tabs switch, and
//! a live header for every state.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::{Button, ButtonSize, ButtonVariant};
use u0ui::prelude::*;
use u0ui::t3::review::{PullRequestDetailHeader, PullRequestState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        state: 0,
        tab: "Summary".into(),
        ref_tab: "Summary".into(),
    })
    .into()
}

struct Page {
    state: usize,
    tab: SharedString,
    ref_tab: SharedString,
}

fn primary(id: &'static str) -> impl IntoElement {
    // A fragment in the reference: the buttons sit in the meta row's 8px gap.
    div()
        .flex()
        .items_center()
        .gap(rems(0.5))
        .child(
            Button::new(SharedString::from(format!("{id}-checkout")), "Check out")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .icon(CarbonIcon::Branch),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-open")), "Open in thread")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm),
        )
}

fn header(id: &'static str, state: PullRequestState, tab: SharedString) -> PullRequestDetailHeader {
    PullRequestDetailHeader::new(id, 482, "Dark theme tokens")
        .state(state)
        .repo("acme/carbon-ui")
        .author("jd")
        .branches("feat/dark-tokens", "main")
        .tab(tab)
        .primary(primary(id))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .h(rems(10.))
        .overflow_hidden()
        .child(
            PullRequestDetailHeader::new("pdh-thumb", 482, "Dark theme tokens")
                .repo("acme/carbon-ui")
                .author("jd")
                .branches("feat/dark-tokens", "main"),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = header(
            "pdh-live",
            PullRequestState::ALL[self.state],
            self.tab.clone(),
        )
        .on_tab(controls::setter(
            &entity,
            |this: &mut Page, t: SharedString| this.tab = t,
        ));
        let config = controls::field(
            "state",
            controls::choices(
                "pdh-state",
                &["open", "draft", "merged", "closed"],
                self.state,
                controls::setter(&entity, |this: &mut Page, i| this.state = i),
            ),
        );
        Body::new()
            .playground(div().w_full().child(live))
            .hint("The tabs switch; Back, Edit title and #482 are buttons")
            .config(config)
            .example(controls::block(
                "Reference preview",
                header("pdh-ref", PullRequestState::Open, self.ref_tab.clone()).on_tab(
                    controls::setter(&entity, |this: &mut Page, t: SharedString| this.ref_tab = t),
                ),
            ))
    }
}
