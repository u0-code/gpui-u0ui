//! Feedback › Toast: the reference static success toast next to a "Trigger
//! toast()" button raising an error toast, a live `Toast` with every
//! `ToastProps` prop, and the `Toaster` (position, duration) with buttons for
//! every `toast.*` call.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::alert::NotificationKind;
use u0ui::components::toast::{self, Toast, ToastOptions, Toaster, ToasterPosition};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ToastPage {
        kind: 1,
        low_contrast: false,
        description: true,
        caption: true,
        closable: true,
        timeout: 0,
        position: 0,
        duration: 1,
        generation: 0,
    })
    .into()
}

struct ToastPage {
    kind: usize,
    low_contrast: bool,
    description: bool,
    caption: bool,
    closable: bool,
    timeout: usize,
    position: usize,
    duration: usize,
    generation: usize,
}

const TIMEOUTS: [(&str, u64); 3] = [("0 (never)", 0), ("3000", 3000), ("5000", 5000)];
const DURATIONS: [(&str, u64); 3] = [("2000", 2000), ("5000", 5000), ("10000", 10000)];

/// `new Date().toLocaleTimeString()`: the wall-clock time (UTC; std has no
/// time-zone database).
fn now() -> SharedString {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        % 86_400;
    format!(
        "{:02}:{:02}:{:02} UTC",
        secs / 3600,
        (secs / 60) % 60,
        secs % 60
    )
    .into()
}

impl Render for ToastPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Toast/preview.html`: a top-aligned `cu-row`.
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_start()
            .gap(tokens::SPACING_05)
            .child(
                Toast::new(("ref-toast", self.generation), "Backup complete")
                    .variant(NotificationKind::Success)
                    .description("db-main snapshot saved.")
                    .caption("Just now"),
            )
            .child(
                Button::new("ref-trigger-toast", "Trigger toast()")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Md)
                    .on_click(|_, _, cx| {
                        toast::error(
                            cx,
                            "Upload failed",
                            ToastOptions::default()
                                .description("2 files exceeded 5 GB.")
                                .caption(now()),
                        );
                    }),
            );

        let kind = NotificationKind::ALL[self.kind];
        let mut live = Toast::new(
            SharedString::from(format!("live-toast-{}", self.generation)),
            "Backup complete",
        )
        .variant(kind)
        .low_contrast(self.low_contrast)
        .timeout(TIMEOUTS[self.timeout].1);
        if self.description {
            live = live.description("db-main snapshot saved.");
        }
        if self.caption {
            live = live.caption("Just now");
        }
        if !self.closable {
            live = live.on_close(|_| false);
        }

        let raise = |label: &'static str, kind: Option<NotificationKind>| {
            Button::new(SharedString::from(format!("raise-{label}")), label)
                .variant(ButtonVariant::Tertiary)
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| {
                    let options = ToastOptions::default()
                        .description("Raised from the gallery.")
                        .caption(now());
                    match kind {
                        None => toast::toast(cx, "Notification", options),
                        Some(NotificationKind::Success) => toast::success(cx, "Saved", options),
                        Some(NotificationKind::Error) => toast::error(cx, "Upload failed", options),
                        Some(NotificationKind::Warning) => {
                            toast::warning(cx, "Quota at 90%", options)
                        }
                        Some(NotificationKind::Info) => toast::info(cx, "Maintenance", options),
                    };
                })
        };
        let toaster_controls = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(tokens::SPACING_03)
            .child(raise("toast()", None))
            .child(raise("toast.success", Some(NotificationKind::Success)))
            .child(raise("toast.error", Some(NotificationKind::Error)))
            .child(raise("toast.warning", Some(NotificationKind::Warning)))
            .child(raise("toast.info", Some(NotificationKind::Info)))
            .child(
                Button::new("dismiss-all", "Dismiss all")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .on_click(|_, _, cx| toast::dismiss_all(cx)),
            );

        let kinds = NotificationKind::ALL.map(|k| k.name());
        let timeouts = TIMEOUTS.map(|t| t.0);
        let positions = ToasterPosition::ALL.map(|p| p.name());
        let durations = DURATIONS.map(|d| d.0);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::section("Toast"))
            .child(controls::field(
                "variant",
                controls::choices(
                    "toast-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut ToastPage, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "timeout",
                controls::choices(
                    "toast-timeout",
                    &timeouts,
                    self.timeout,
                    controls::setter(&entity, |this: &mut ToastPage, i| {
                        this.timeout = i;
                        this.generation += 1;
                    }),
                ),
            ))
            .child(controls::toggle(
                "toast-low",
                "lowContrast",
                self.low_contrast,
                controls::setter(&entity, |this: &mut ToastPage, on| this.low_contrast = on),
            ))
            .child(controls::toggle(
                "toast-desc",
                "description",
                self.description,
                controls::setter(&entity, |this: &mut ToastPage, on| this.description = on),
            ))
            .child(controls::toggle(
                "toast-caption",
                "caption",
                self.caption,
                controls::setter(&entity, |this: &mut ToastPage, on| this.caption = on),
            ))
            .child(controls::toggle(
                "toast-closable",
                "onClose allows closing",
                self.closable,
                controls::setter(&entity, |this: &mut ToastPage, on| this.closable = on),
            ))
            .child(
                Button::new("toast-reset", "Show closed toasts again")
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
            .child(controls::section("Toaster"))
            .child(controls::field(
                "position",
                controls::choices(
                    "toaster-position",
                    &positions,
                    self.position,
                    controls::setter(&entity, |this: &mut ToastPage, i| this.position = i),
                ),
            ))
            .child(controls::field(
                "duration",
                controls::choices(
                    "toaster-duration",
                    &durations,
                    self.duration,
                    controls::setter(&entity, |this: &mut ToastPage, i| this.duration = i),
                ),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block("Toaster", toaster_controls))
            .example(
                Toaster::new("gallery-toaster")
                    .position(ToasterPosition::ALL[self.position])
                    .duration(DURATIONS[self.duration].1),
            )
    }
}
