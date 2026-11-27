//! Carbon · Patterns › PatternNotifications: the five forms (banner,
//! inline, toast, actionable, callout) in the four statuses
//! (`components/PatternNotifications/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::components::alert::{Alert, AlertAction, AlertVariant, NotificationKind};
use u0ui::components::toast::Toast;
use u0ui::patterns::{ActionableToast, Caption, PatternGrid};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| NotificationsPage {
        form: 1,
        kind: 0,
        low_contrast: true,
        toasts: Vec::new(),
        next: 0,
        generation: 0,
    })
    .into()
}

struct NotificationsPage {
    /// Index into [`FORMS`].
    form: usize,
    /// Index into `NotificationKind::ALL`.
    kind: usize,
    low_contrast: bool,
    /// Raised toasts, newest first: (id, kind).
    toasts: Vec<(usize, NotificationKind)>,
    next: usize,
    /// Bumped by "Show again" so closed notifications come back.
    generation: usize,
}

const FORMS: [&str; 6] = [
    "banner",
    "inline",
    "actionable inline",
    "toast",
    "actionable toast",
    "callout",
];

/// `KINDOF`: the alert variant of a notification kind.
fn variant(kind: NotificationKind) -> AlertVariant {
    match kind {
        NotificationKind::Error => AlertVariant::Error,
        NotificationKind::Success => AlertVariant::Success,
        NotificationKind::Warning => AlertVariant::Warning,
        NotificationKind::Info => AlertVariant::Info,
    }
}

/// The demo copy of each status: (title, subtitle).
fn copy(kind: NotificationKind) -> (&'static str, &'static str) {
    match kind {
        NotificationKind::Error => ("Push failed", "origin rejected the update."),
        NotificationKind::Success => ("Pushed", "3 commits to origin/feat."),
        NotificationKind::Warning => ("Token expires in 3 days", ""),
        NotificationKind::Info => ("Plan mode", "The agent will not edit files."),
    }
}

fn inline(id: impl Into<SharedString>, kind: NotificationKind, low: bool) -> Alert {
    let (title, subtitle) = copy(kind);
    let alert = Alert::new(id.into())
        .variant(variant(kind))
        .title(title)
        .low_contrast(low)
        .on_close(|_, _| {});
    if subtitle.is_empty() {
        alert
    } else {
        alert.description(subtitle)
    }
}

/// The banner: an inline actionable notification across the content.
fn banner(id: impl Into<SharedString>, kind: NotificationKind, low: bool) -> Alert {
    Alert::new(id.into())
        .variant(variant(kind))
        .title("Ghost 1.5 is ready.")
        .description("Restart to update; your threads keep running.")
        .action(AlertAction::new("Restart now"))
        .low_contrast(low)
        .on_close(|_, _| {})
}

/// The callout: never closable, info or warning only.
fn callout(id: impl Into<SharedString>, kind: NotificationKind, low: bool) -> Alert {
    let kind = match kind {
        NotificationKind::Warning => NotificationKind::Warning,
        _ => NotificationKind::Info,
    };
    Alert::new(id.into())
        .variant(variant(kind))
        .title("Agents run on the server you pick.")
        .description("Files never leave that machine.")
        .low_contrast(low)
}

fn approve(id: impl Into<SharedString>, kind: NotificationKind, low: bool) -> ActionableToast {
    ActionableToast::new(id.into(), "Approve command?", "Review")
        .kind(kind)
        .subtitle("claude wants to run pnpm db:migrate")
        .close_label("Close notification")
        .low_contrast(low)
}

fn restored(id: impl Into<SharedString>, kind: NotificationKind, low: bool) -> Toast {
    Toast::new(id.into(), "Checkpoint restored")
        .variant(kind)
        .description("4 files reverted")
        .caption("10:42")
        .low_contrast(low)
}

/// `components/PatternNotifications/preview.html`.
pub fn reference() -> impl IntoElement {
    use NotificationKind::*;
    PatternGrid::new(2)
        .align_start(true)
        .full(
            div()
                .child(Caption::new("Banner：内容区顶部，一次一条，不吸顶"))
                .child(banner("nt-ref-banner", Info, true)),
        )
        .cell(
            div()
                .child(Caption::new("Inline：紧挨相关内容，最多两行"))
                // `display: grid; gap: 8`.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(tokens::SPACING_03)
                        .child(inline("nt-ref-error", Error, true))
                        .child(inline("nt-ref-success", Success, true))
                        .child(inline("nt-ref-warning", Warning, true))
                        .child(inline("nt-ref-info", Info, true)),
                ),
        )
        .cell(
            div()
                .child(Caption::new(
                    "Toast：右上角，最新的在最上，固定宽度，最多三行；可操作的 toast 只放一个 tertiary 按钮",
                ))
                // `display: grid; gap: 8; justify-items: end`.
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_end()
                        .gap(tokens::SPACING_03)
                        .child(approve("nt-ref-approve", Warning, false))
                        .child(restored("nt-ref-toast", Success, false)),
                ),
        )
        .full(
            div()
                .child(Caption::new("Callout：不可关闭，不用于成功或错误，只做提示"))
                .child(callout("nt-ref-callout", Info, true)),
        )
}

/// The overview tile: an inline error and an inline success.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            Alert::new("nt-thumb-error")
                .variant(AlertVariant::Error)
                .title("Push failed"),
        )
        .child(
            Alert::new("nt-thumb-success")
                .variant(AlertVariant::Success)
                .title("Pushed"),
        )
        .into_any_element()
}

impl NotificationsPage {
    fn raise(entity: &Entity<Self>, cx: &mut App) {
        entity.update(cx, |this, cx| {
            let kind = NotificationKind::ALL[this.kind];
            this.toasts.insert(0, (this.next, kind));
            this.next += 1;
            cx.notify();
        });
    }
}

impl Render for NotificationsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = NotificationKind::ALL[self.kind];
        let low = self.low_contrast;
        let g = self.generation;
        let id = |s: &str| SharedString::from(format!("nt-live-{s}-{g}"));
        let shown: AnyElement = match self.form {
            0 => banner(id("banner"), kind, low).into_any_element(),
            1 => inline(id("inline"), kind, low).into_any_element(),
            2 => inline(id("actionable"), kind, low)
                .action(AlertAction::new("Retry"))
                .into_any_element(),
            3 => restored(id("toast"), kind, low).into_any_element(),
            4 => approve(id("approve"), kind, low).into_any_element(),
            _ => callout(id("callout"), kind, low).into_any_element(),
        };

        // Toasts raised from the button: newest on top, at the end of the
        // row; errors stay until closed, the rest leave after 5 s.
        let now = u0ui::clock::now();
        let stamp: SharedString = format!("{}:{:02}", now.hour, now.minute).into();
        let stack = self.toasts.iter().map(|&(n, kind)| {
            let (title, subtitle) = copy(kind);
            let entity = entity.clone();
            let toast = Toast::new(SharedString::from(format!("nt-live-raised-{n}")), title)
                .variant(kind)
                .low_contrast(low)
                .caption(stamp.clone())
                .on_close(move |cx| {
                    entity.update(cx, |this, cx| {
                        this.toasts.retain(|t| t.0 != n);
                        cx.notify();
                    });
                    true
                });
            let toast = if kind == NotificationKind::Error {
                toast
            } else {
                toast.timeout(5000)
            };
            if subtitle.is_empty() {
                toast.into_any_element()
            } else {
                toast.description(subtitle).into_any_element()
            }
        });

        let again = {
            let entity = entity.clone();
            Button::new("nt-live-again", "Show again")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.generation += 1;
                        cx.notify();
                    })
                })
        };
        let raise = {
            let entity = entity.clone();
            Button::new("nt-live-raise", "Raise a toast")
                .variant(ButtonVariant::Tertiary)
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| Self::raise(&entity, cx))
        };

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .min_w_0()
            .child(shown)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(tokens::SPACING_03)
                    .child(raise)
                    .child(again),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap(tokens::SPACING_03)
                    .children(stack),
            );

        let kinds = NotificationKind::ALL.map(|k| k.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "form",
                controls::choices(
                    "nt-form",
                    &FORMS,
                    self.form,
                    controls::setter(&entity, |this: &mut Self, i| this.form = i),
                ),
            ))
            .child(controls::field(
                "status",
                controls::choices(
                    "nt-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "nt-low",
                "low contrast",
                self.low_contrast,
                controls::setter(&entity, |this: &mut Self, on| this.low_contrast = on),
            ));

        Body::new()
            .playground(live)
            .hint("× closes; Show again brings it back; raised toasts stack newest first, errors stay until closed")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_map_to_their_alert_variants() {
        for kind in NotificationKind::ALL {
            assert_eq!(variant(kind).name(), kind.name());
        }
    }
}
