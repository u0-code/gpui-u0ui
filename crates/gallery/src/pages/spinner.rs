//! Feedback › Spinner (Carbon Loading / InlineLoading): the reference row
//! (large, small, "Loading data…", "Saved" finished), a live spinner with
//! every size and status, the stop sequence and the window overlay.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use std::time::Duration;
use u0ui::components::spinner::{Spinner, SpinnerSize, SpinnerStatus};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SpinnerPage {
        size: 1,
        status: 0,
        description: false,
        overlay: false,
    })
    .into()
}

struct SpinnerPage {
    size: usize,
    status: usize,
    description: bool,
    overlay: bool,
}

fn caption(text: &'static str) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text)
}

impl Render for SpinnerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/Spinner/preview.html` (`cu-row`, gap 40).
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_08)
            .child(Spinner::new("ref-md").label("Loading"))
            .child(
                Spinner::new("ref-sm")
                    .size(SpinnerSize::Sm)
                    .label("Loading"),
            )
            .child(Spinner::new("ref-inline").description("Loading data…"))
            .child(
                Spinner::new("ref-saved")
                    .description("Saved")
                    .status(SpinnerStatus::Finished),
            );

        let size = SpinnerSize::ALL[self.size];
        let status = SpinnerStatus::ALL[self.status];
        let mut live = Spinner::new("live")
            .size(size)
            .status(status)
            .label("Loading");
        if self.description {
            live = live.description(match status {
                SpinnerStatus::Active => "Loading data…",
                SpinnerStatus::Finished => "Saved",
                SpinnerStatus::Error => "Failed to load",
                SpinnerStatus::Inactive => "Paused",
            });
        }

        let states = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_start()
            .gap(tokens::SPACING_08)
            .children(SpinnerStatus::ALL.map(|status| {
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_03)
                    .child(caption(status.name()))
                    .child(
                        Spinner::new(("matrix-inline", status as usize))
                            .size(SpinnerSize::Inline)
                            .status(status)
                            .description(status.name()),
                    )
                    .child(
                        Spinner::new(("matrix-sm", status as usize))
                            .size(SpinnerSize::Sm)
                            .status(status),
                    )
            }));

        let overlay = self
            .overlay
            .then(|| Spinner::new("overlay").with_overlay(true).label("Loading"));

        let sizes = SpinnerSize::ALL.map(|s| s.name());
        let statuses = SpinnerStatus::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "spinner-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut SpinnerPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "status (inactive stops the spinner)",
                controls::choices(
                    "spinner-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut SpinnerPage, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "spinner-description",
                "description",
                self.description,
                controls::setter(&entity, |this: &mut SpinnerPage, on| this.description = on),
            ))
            .child(
                Button::new("spinner-overlay", "withOverlay for 3 s")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, window, cx| {
                            entity.update(cx, |this, cx| {
                                this.overlay = true;
                                cx.notify();
                            });
                            let entity = entity.clone();
                            window
                                .spawn(cx, async move |cx| {
                                    cx.background_executor().timer(Duration::from_secs(3)).await;
                                    entity.update(cx, |this, cx| {
                                        this.overlay = false;
                                        cx.notify();
                                    });
                                })
                                .detach();
                        }
                    }),
            );

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live))
                .child(controls::block("Statuses (inline and small)", states))
                .children(overlay),
            Some(config.into_any_element()),
        )
    }
}
