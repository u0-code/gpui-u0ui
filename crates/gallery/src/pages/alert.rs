//! Feedback › Alert: the reference four notifications (destructive, success,
//! warning with close, info with an action) and a live alert with every
//! `AlertProps` prop.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AlertPage {
        variant: 1,
        low_contrast: true,
        title: true,
        closable: true,
        action: false,
        generation: 0,
        log: None,
    })
    .into()
}

struct AlertPage {
    variant: usize,
    low_contrast: bool,
    title: bool,
    closable: bool,
    action: bool,
    /// Bumped to remount closed alerts.
    generation: usize,
    log: Option<SharedString>,
}

impl Render for AlertPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let g = self.generation;

        // `components/Alert/preview.html`: a `cu-stack` with an 8px gap.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                Alert::new(("ref-alert-1", g))
                    .variant(AlertVariant::Destructive)
                    .alert_title(AlertTitle::new("Deployment failed"))
                    .alert_description(AlertDescription::new("Image could not be pulled.")),
            )
            .child(
                Alert::new(("ref-alert-2", g))
                    .variant(AlertVariant::Success)
                    .title("Saved")
                    .description("Your changes are live."),
            )
            .child(
                Alert::new(("ref-alert-3", g))
                    .variant(AlertVariant::Warning)
                    .title("Quota at 90%")
                    .description("Remove unused volumes.")
                    .on_close(|_, _| {}),
            )
            .child(
                Alert::new(("ref-alert-4", g))
                    .variant(AlertVariant::Info)
                    .title("Maintenance")
                    .description("Sunday 02:00 UTC.")
                    .action(AlertAction::new("Details")),
            );

        let variant = AlertVariant::ALL[self.variant];
        let mut live = Alert::new(("live-alert", g))
            .variant(variant)
            .low_contrast(self.low_contrast)
            .description("Remove unused volumes to free up space.");
        if self.title {
            live = live.title("Quota at 90%");
        }
        if self.closable {
            let e = entity.clone();
            live = live.on_close(move |_, cx| {
                e.update(cx, |this, cx| {
                    this.log = Some("onClose".into());
                    cx.notify();
                })
            });
        }
        if self.action {
            let e = entity.clone();
            live = live.action(AlertAction::new("Details").on_click(move |_, cx| {
                e.update(cx, |this, cx| {
                    this.log = Some("action.onClick".into());
                    cx.notify();
                })
            }));
        }

        let names = AlertVariant::ALL.map(|v| v.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "alert-variant",
                    &names,
                    self.variant,
                    controls::setter(&entity, |this: &mut AlertPage, i| this.variant = i),
                ),
            ))
            .child(controls::toggle(
                "alert-low",
                "lowContrast",
                self.low_contrast,
                controls::setter(&entity, |this: &mut AlertPage, on| this.low_contrast = on),
            ))
            .child(controls::toggle(
                "alert-title",
                "title",
                self.title,
                controls::setter(&entity, |this: &mut AlertPage, on| this.title = on),
            ))
            .child(controls::toggle(
                "alert-close",
                "onClose",
                self.closable,
                controls::setter(&entity, |this: &mut AlertPage, on| this.closable = on),
            ))
            .child(controls::toggle(
                "alert-action",
                "action",
                self.action,
                controls::setter(&entity, |this: &mut AlertPage, on| this.action = on),
            ))
            .child(
                Button::new("alert-reset", "Show closed alerts again")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let e = entity.clone();
                        move |_, _, cx| {
                            e.update(cx, |this, cx| {
                                this.generation += 1;
                                cx.notify();
                            })
                        }
                    }),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.log.clone().unwrap_or_else(|| "No events yet.".into())),
            );

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
