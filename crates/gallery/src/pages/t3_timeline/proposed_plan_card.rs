//! T3 · Timeline › ProposedPlanCard: the three-step token plan
//! (`components/ProposedPlanCard/preview.html`); the live card reports its
//! actions, and can show where each step stands, how far the plan has got
//! and a note before its actions.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{PlanStep, ProposedPlanCard};
use u0ui::u0::thread::PlanStatus;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PlanPage {
        details: true,
        states: false,
        last: "".into(),
    })
    .into()
}

struct PlanPage {
    details: bool,
    states: bool,
    last: SharedString,
}

fn steps(details: bool) -> Vec<PlanStep> {
    with_states(details, false)
}

/// The steps, each saying where it stands when `states`: the first done,
/// the second under way, the third not begun.
fn with_states(details: bool, states: bool) -> Vec<PlanStep> {
    let marks = [
        PlanStatus::Done,
        PlanStatus::InProgress,
        PlanStatus::Pending,
    ];
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
    .zip(marks)
    .map(|((t, d), mark)| {
        let step = PlanStep::new(t);
        let step = if details { step.detail(d) } else { step };
        if states { step.status(mark) } else { step }
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
                ProposedPlanCard::new(
                    "plan-live",
                    "Proposed plan",
                    with_states(self.details, self.states),
                )
                .when(self.states, |c| {
                    c.progress("1 of 3 done")
                        .note("Implementing switches the agent to Default")
                })
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
            ))
            .child(controls::toggle(
                "plan-states",
                "Step states, progress and note",
                self.states,
                controls::setter(&entity, |this: &mut Self, on| this.states = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().max_w(gpui_kit::rems(40.)).child(
                    ProposedPlanCard::new("plan-ref", "Proposed plan", steps(true))
                        .on_implement(|_, _| {})
                        .on_keep_planning(|_, _| {})
                        .on_copy(|_, _| {})
                        .on_download(|_, _| {})
                        .on_save(|_, _| {}),
                ),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            ProposedPlanCard::new(
                "plan-th",
                "Proposed plan",
                steps(false).into_iter().take(2).collect(),
            )
            .on_implement(|_, _| {})
            .on_keep_planning(|_, _| {})
            .on_copy(|_, _| {})
            .on_download(|_, _| {})
            .on_save(|_, _| {}),
        )
        .into_any_element()
}
