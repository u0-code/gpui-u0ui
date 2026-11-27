//! T3 · Composer › ComposerPrimaryActions: idle, idle disabled, running,
//! answering ("Submit answers") and submitting
//! (`components/ComposerPrimaryActions/preview.html`), plus a live set with
//! every state and the Steer / Queue / Stop layout.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{ComposerPrimaryActions, ComposerState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ActionsPage {
        state: 1,
        steer: true,
        empty: false,
        disabled: false,
        last: "".into(),
    })
    .into()
}

struct ActionsPage {
    state: usize,
    steer: bool,
    empty: bool,
    disabled: bool,
    last: SharedString,
}

/// The reference preview (`cu-row`).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
        .child(ComposerPrimaryActions::new("pa-ref-0", ComposerState::Idle))
        .child(ComposerPrimaryActions::new("pa-ref-1", ComposerState::Idle).disabled(true))
        .child(ComposerPrimaryActions::new(
            "pa-ref-2",
            ComposerState::Running,
        ))
        .child(
            ComposerPrimaryActions::new("pa-ref-3", ComposerState::Answering)
                .answer_label("Submit answers"),
        )
        .child(ComposerPrimaryActions::new(
            "pa-ref-4",
            ComposerState::Submitting,
        ))
}

impl Render for ActionsPage {
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
        let mut actions = ComposerPrimaryActions::new("pa-live", ComposerState::ALL[self.state])
            .disabled(self.disabled)
            .empty(self.empty)
            .on_send(note("onSend"))
            .on_stop(note("onStop"))
            .on_queue(note("onQueue"));
        if self.steer {
            actions = actions.on_steer(note("onSteer"));
        }
        let live = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(tokens::SPACING_05)
            .child(actions)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let states = ComposerState::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "pa-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ))
            .child(controls::toggle(
                "pa-steer",
                "onSteer (running)",
                self.steer,
                controls::setter(&entity, |this: &mut Self, on| this.steer = on),
            ))
            .child(controls::toggle(
                "pa-empty",
                "empty",
                self.empty,
                controls::setter(&entity, |this: &mut Self, on| this.empty = on),
            ))
            .child(controls::toggle(
                "pa-disabled",
                "disabled",
                self.disabled,
                controls::setter(&entity, |this: &mut Self, on| this.disabled = on),
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
        .flex()
        .flex_row()
        .gap(tokens::SPACING_05)
        .child(ComposerPrimaryActions::new("pa-th-0", ComposerState::Idle))
        .child(ComposerPrimaryActions::new(
            "pa-th-1",
            ComposerState::Running,
        ))
        .into_any_element()
}
