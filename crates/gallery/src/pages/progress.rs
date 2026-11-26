//! Feedback › Progress (Carbon ProgressBar): the reference bars (uploading
//! 62%, finished migration, indeterminate provisioning) and a live bar with
//! every prop; the value buttons show the 110ms bar transition.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::progress::{Progress, ProgressSize, ProgressStatus, ProgressType};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ProgressPage {
        value: 2,
        status: 0,
        size: 0,
        kind: 0,
        label: true,
        helper: true,
    })
    .into()
}

struct ProgressPage {
    value: usize,
    status: usize,
    size: usize,
    kind: usize,
    label: bool,
    helper: bool,
}

const VALUES: [&str; 6] = ["none", "0", "25", "62", "90", "100"];

impl Render for ProgressPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/Progress/preview.html`: `cu-stack`, max-width 420.
        let reference = div()
            .max_w(rems(26.25))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(
                Progress::new("ref-upload")
                    .label("Uploading backup.tar")
                    .value(62.)
                    .helper_text("620 MB of 1 GB"),
            )
            .child(
                Progress::new("ref-migration")
                    .label("Migration")
                    .value(100.)
                    .status(ProgressStatus::Finished)
                    .helper_text("Done"),
            )
            .child(
                Progress::new("ref-provision")
                    .label("Provisioning")
                    .helper_text("Estimating time…"),
            );

        let value = VALUES[self.value].parse::<f32>().ok();
        let status = ProgressStatus::ALL[self.status];
        let mut live = Progress::new("live")
            .value_opt(value)
            .status(status)
            .size(ProgressSize::ALL[self.size])
            .kind(ProgressType::ALL[self.kind]);
        if self.label {
            live = live.label("Uploading backup.tar");
        }
        if self.helper {
            live = live.helper_text(match status {
                ProgressStatus::Error => "Upload failed",
                ProgressStatus::Finished => "Done",
                ProgressStatus::Active => "620 MB of 1 GB",
            });
        }
        let on_layer = div()
            .bg(color(ColorRole::Layer01))
            .p(tokens::SPACING_05)
            .max_w(rems(26.25))
            .child(
                Progress::new("on-layer")
                    .label("On layer-01")
                    .value(40.)
                    .layer(LayerLevel::Two),
            );

        let statuses = ProgressStatus::ALL.map(|s| s.name());
        let sizes = ProgressSize::ALL.map(|s| s.name());
        let kinds = ProgressType::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "value (none = indeterminate)",
                controls::choices(
                    "progress-value",
                    &VALUES,
                    self.value,
                    controls::setter(&entity, |this: &mut ProgressPage, i| this.value = i),
                ),
            ))
            .child(controls::field(
                "status",
                controls::choices(
                    "progress-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut ProgressPage, i| this.status = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "progress-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut ProgressPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "type",
                controls::choices(
                    "progress-type",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut ProgressPage, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "progress-label",
                "label",
                self.label,
                controls::setter(&entity, |this: &mut ProgressPage, on| this.label = on),
            ))
            .child(controls::toggle(
                "progress-helper",
                "helperText",
                self.helper,
                controls::setter(&entity, |this: &mut ProgressPage, on| this.helper = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block(
                    "Live",
                    div().max_w(rems(26.25)).child(live),
                ))
                .child(controls::block("On layer-01", on_layer)),
            Some(config.into_any_element()),
        )
    }
}
