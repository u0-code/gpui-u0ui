//! T3 · Pull requests & issues › ReviewForm: the reference form and a live
//! one that reports the submitted verdict.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::ReviewForm;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        disabled: false,
        flat: false,
        submitted: None,
    })
    .into()
}

struct Page {
    disabled: bool,
    flat: bool,
    submitted: Option<SharedString>,
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .h(rems(10.))
        .overflow_hidden()
        .child(ReviewForm::new("rf-thumb"))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(
                ReviewForm::new("rf-live")
                    .submit_disabled(self.disabled)
                    .flat(self.flat)
                    .on_submit(controls::setter(
                        &entity,
                        |this: &mut Page, v: u0ui::t3::review::ReviewVerdict| {
                            this.submitted = Some(v.name().into())
                        },
                    )),
            )
            .when_some(self.submitted.clone(), |this, v| {
                this.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("onSubmit(\"{v}\")")),
                )
            });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "rf-disabled",
                "submitDisabled",
                self.disabled,
                controls::setter(&entity, |this: &mut Page, v| this.disabled = v),
            ))
            .child(controls::toggle(
                "rf-flat",
                "flat",
                self.flat,
                controls::setter(&entity, |this: &mut Page, v| this.flat = v),
            ));
        Body::new()
            .playground(live)
            .hint("Arrow keys move between the verdicts; Submit reports the choice")
            .config(config)
            .example(controls::block(
                "Reference preview",
                ReviewForm::new("rf-ref"),
            ))
    }
}
