//! T3 · Timeline › ProposedPlanCard: the three-step token plan
//! (`components/ProposedPlanCard/preview.html`); the live card reports its
//! actions.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{PlanStep, ProposedPlanCard};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PlanPage {
        details: true,
        last: "".into(),
    })
    .into()
}

struct PlanPage {
    details: bool,
    last: SharedString,
}

fn steps(details: bool) -> Vec<PlanStep> {
    [
        (
            "Extract tokens",
            "Read @carbon/themes and write all four themes to tokens.json.",
        ),
        (
            "Retarget CSS",
            "Rename --cds-* to token names in bundle.css.",
        ),
        (
            "Verify contrast",
            "Check every text token against its documented grounds.",
        ),
    ]
    .into_iter()
    .map(|(t, d)| {
        let step = PlanStep::new(t);
        if details { step.detail(d) } else { step }
    })
    .collect()
}

impl Render for PlanPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let note = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(40.))
            .child(
                ProposedPlanCard::new("plan-live", "Proposed plan", steps(self.details))
                    .on_implement(note("onImplement"))
                    .on_keep_planning(note("Keep planning"))
                    .on_copy(note("Copy plan"))
                    .on_download(note("Download as Markdown"))
                    .on_save(note("Save to workspace")),
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
                "plan-details",
                "Step details",
                self.details,
                controls::setter(&entity, |this: &mut Self, on| this.details = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(40.))
                    .child(ProposedPlanCard::new(
                        "plan-ref",
                        "Proposed plan",
                        steps(true),
                    )),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(ProposedPlanCard::new(
            "plan-th",
            "Proposed plan",
            steps(false).into_iter().take(2).collect(),
        ))
        .into_any_element()
}
