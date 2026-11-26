//! Chat & AI › Attachment: the reference group (done PDF, uploading image,
//! failed SQL dump, trigger) and a configurable live attachment; the trigger
//! opens the platform file picker and lists what was chosen.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _,
};
use u0ui::components::attachment::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentGroup, AttachmentMedia, AttachmentOrientation, AttachmentSize, AttachmentState,
    AttachmentStatus, AttachmentTitle, AttachmentTrigger,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AttachmentPage {
        state: 1,
        size: 0,
        orientation: 0,
        image: false,
        picked: Vec::new(),
    })
    .into()
}

struct AttachmentPage {
    state: usize,
    size: usize,
    orientation: usize,
    image: bool,
    picked: Vec<SharedString>,
}

fn status_label(state: AttachmentState) -> &'static str {
    match state {
        AttachmentState::Uploading => "Uploading",
        AttachmentState::Processing => "Processing",
        _ => "Upload failed",
    }
}

impl Render for AttachmentPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Attachment/preview.html`.
        let reference = AttachmentGroup::new()
            .child(
                Attachment::new()
                    .media(AttachmentMedia::new())
                    .content(
                        AttachmentContent::new()
                            .child(AttachmentTitle::new("quarterly-report.pdf"))
                            .child(AttachmentDescription::new("2.4 MB")),
                    )
                    .actions(
                        AttachmentActions::new()
                            .child(AttachmentAction::new("att-ref-0", "Remove")),
                    ),
            )
            .child(
                Attachment::new()
                    .state(AttachmentState::Uploading)
                    .media(AttachmentMedia::new().icon(CarbonIcon::Image))
                    .content(
                        AttachmentContent::new()
                            .child(AttachmentTitle::new("diagram.png"))
                            .child(
                                AttachmentStatus::new(
                                    "att-ref-1-status",
                                    AttachmentState::Uploading,
                                )
                                .label("Uploading"),
                            ),
                    )
                    .actions(
                        AttachmentActions::new()
                            .child(AttachmentAction::new("att-ref-1", "Cancel")),
                    ),
            )
            .child(
                Attachment::new()
                    .state(AttachmentState::Error)
                    .media(AttachmentMedia::new())
                    .content(
                        AttachmentContent::new()
                            .child(AttachmentTitle::new("dump.sql"))
                            .child(
                                AttachmentStatus::new("att-ref-2-status", AttachmentState::Error)
                                    .label("Exceeds 50 MB"),
                            ),
                    )
                    .actions(
                        AttachmentActions::new()
                            .child(AttachmentAction::new("att-ref-2", "Remove")),
                    ),
            )
            .child(AttachmentTrigger::new("att-ref-trigger", "Attach file"));

        let state = AttachmentState::ALL[self.state];
        let media = if self.image {
            AttachmentMedia::new().image(Icon::named("image").asset_path().to_string())
        } else {
            AttachmentMedia::new()
        };
        let live = AttachmentGroup::new()
            .child(
                Attachment::new()
                    .state(state)
                    .size(AttachmentSize::ALL[self.size])
                    .orientation(AttachmentOrientation::ALL[self.orientation])
                    .media(media)
                    .content(
                        AttachmentContent::new()
                            .child(AttachmentTitle::new(
                                "a-very-long-file-name-for-the-quarterly-report-2026.pdf",
                            ))
                            .child(AttachmentDescription::new("2.4 MB"))
                            .child(
                                AttachmentStatus::new("att-live-status", state)
                                    .label(status_label(state)),
                            ),
                    )
                    .actions(
                        AttachmentActions::new()
                            .child(AttachmentAction::new("att-live-remove", "Remove")),
                    ),
            )
            .child(
                AttachmentTrigger::new("att-live-trigger", "Attach file")
                    .multiple(true)
                    .on_files({
                        let entity = entity.clone();
                        move |paths, cx| {
                            entity.update(cx, |this, cx| {
                                this.picked = paths
                                    .iter()
                                    .map(|p| SharedString::from(p.display().to_string()))
                                    .collect();
                                cx.notify();
                            })
                        }
                    }),
            );
        let picked = div()
            .flex()
            .flex_col()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .when(self.picked.is_empty(), |this| {
                this.child("Chosen files appear here.")
            })
            .children(self.picked.iter().cloned());

        let states = AttachmentState::ALL.map(|s| s.name());
        let sizes = AttachmentSize::ALL.map(|s| s.name());
        let orientations = AttachmentOrientation::ALL.map(|o| o.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "att-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut AttachmentPage, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "att-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut AttachmentPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "orientation",
                controls::choices(
                    "att-orientation",
                    &orientations,
                    self.orientation,
                    controls::setter(&entity, |this: &mut AttachmentPage, i| this.orientation = i),
                ),
            ))
            .child(controls::toggle(
                "att-image",
                "AttachmentMedia image",
                self.image,
                controls::setter(&entity, |this: &mut AttachmentPage, on| this.image = on),
            ));

        Body::new()
            .playground(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_04)
                    .child(live)
                    .child(picked),
            )
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
