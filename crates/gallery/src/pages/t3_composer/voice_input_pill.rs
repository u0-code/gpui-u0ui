//! T3 · Composer › VoiceInputPill: recording (12s and ticking),
//! transcribing, downloading 42% and an error
//! (`components/VoiceInputPill/preview.html`), plus a live pill whose
//! Discard / Done / dismiss move it through the dictation states.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{VoiceInputPill, VoiceState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| VoicePage {
        state: 0,
        generation: 0,
        last: "".into(),
    })
    .into()
}

struct VoicePage {
    state: usize,
    generation: usize,
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
        .child(VoiceInputPill::new("voice-ref-0", VoiceState::Recording).seconds(12))
        .child(VoiceInputPill::new("voice-ref-1", VoiceState::Transcribing))
        .child(VoiceInputPill::new("voice-ref-2", VoiceState::Downloading).progress(42))
        .child(
            VoiceInputPill::new("voice-ref-3", VoiceState::Error).error("Microphone access denied"),
        )
}

impl Render for VoicePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let go = |state: VoiceState, what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    if let Some(i) = VoiceState::ALL.iter().position(|s| *s == state) {
                        this.state = i;
                    }
                    this.generation += 1;
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let pill = VoiceInputPill::new(
            SharedString::from(format!("voice-live-{}", self.generation)),
            VoiceState::ALL[self.state],
        )
        .progress(42)
        .on_stop(go(VoiceState::Transcribing, "onStop: transcribing"))
        .on_cancel(go(VoiceState::Recording, "onCancel"));
        let live = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(tokens::SPACING_05)
            .child(pill)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let states = VoiceState::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "voice-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("The waveform holds still under reduced motion.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    VoiceInputPill::new("voice-th", VoiceState::Recording)
        .seconds(12)
        .into_any_element()
}
