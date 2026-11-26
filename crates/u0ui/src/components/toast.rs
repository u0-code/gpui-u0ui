//! Toast: shadcn `Toast` / Sonner → Carbon `ToastNotification`
//! (`.cds--toast-notification*`), plus the `Toaster` host (`.cu-toaster*`) and
//! an imperative API ([`toast`], [`success`], [`error`], [`warning`], [`info`],
//! [`dismiss`]).
//!
//! - **Look** reuses [`super::alert::NotificationLook`] (the same kind ×
//!   contrast colors; toasts default to high contrast because `index.jsx`
//!   `Toast` passes no `lowContrast`). Geometry: `inline-size: 18rem` (22rem
//!   from a 99rem viewport), `padding-inline-start: 0.8125rem` after the 3px
//!   start border, `box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.2)`; icon 20px with
//!   `margin-block-start: 1rem; margin-inline-end: 1rem`; details
//!   `margin-block-end: 1rem; margin-inline-end: 1rem` holding the
//!   `heading-compact-01` title (`margin-block-start: 1rem`), the
//!   `body-compact-01` subtitle (`margin-block: 0 1rem`) and caption
//!   (`padding-block-start: 0.5rem`); a 3rem close button at the end
//!   (`margin-inline-start: auto`, `transition: outline 250ms`, no hover fill).
//! - **Timeout**: a [`Toast`] with `timeout` calls `on_close` after it; the toast
//!   hides unless `on_close` returns false (Carbon's `onClose` contract).
//! - **Toaster**: `.cu-toaster { position: fixed; z-index: 9500; display: flex;
//!   flex-direction: column; gap: spacing-03; padding: spacing-05 }` in a
//!   corner, drawn in a deferred top layer over the window. Toasts raised with
//!   [`toast`] live in an app-global queue; `error` toasts never auto-dismiss,
//!   the others after their `duration` or the toaster's (default 5000ms):
//!   `timeout={t.variant === 'error' ? 0 : t.duration ?? duration}`.
//! - **Motion**: none. The reference has no toast keyframes or transitions
//!   (`bundle.css` has none for `.cds--toast-notification`, nor do `custom.css`,
//!   `ghost.css` or `t3.css` for `.cu-toaster`), so toasts appear and disappear
//!   at once and the stack reflows instantly.
//! - **Kit reuse**: Kit's `notification` and Base's `toast` stack toasts
//!   Sonner-style (collapsed layers, springs, hover expansion), which Carbon's
//!   flat 8px-gap column does not do; only the idea (a global queue with
//!   timers) is reused here.
use super::alert::{
    ACCENT_WIDTH, NotificationKind, NotificationLook, close_button, status_icon, viewport_rem,
};
use super::form::rem;
use super::overlay::Callback;
use crate::{tokens::TypeStyle, typography::TypeStyled as _};
use gpui_kit::{
    App, BoxShadow, ElementId, Global, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, anchored, deferred, div, point, prelude::FluentBuilder as _, px,
};
use std::{collections::HashSet, rc::Rc, time::Duration};

/// `inline-size: 18rem`; `@media (min-width: 99rem) { inline-size: 22rem }`.
pub fn toast_width(viewport_rem: f32) -> f32 {
    if viewport_rem >= 99. { 352. } else { 288. }
}

/// `.cu-toaster { gap: var(--spacing-03) }`.
pub const TOASTER_GAP: f32 = 8.;
/// `.cu-toaster { padding: var(--spacing-05) }`.
pub const TOASTER_PADDING: f32 = 16.;
/// `.cu-toaster { z-index: 9500 }` (deferred draw priority).
pub const TOASTER_PRIORITY: usize = 9500;
/// The `Toaster` default `duration = 5000`.
pub const DEFAULT_DURATION_MS: u64 = 5000;

/// The auto-dismiss delay of a toaster toast: `t.variant === 'error' ? 0 :
/// t.duration ?? duration` (`None` = stays until closed).
pub fn effective_timeout(
    kind: NotificationKind,
    duration: Option<u64>,
    toaster_duration: u64,
) -> Option<u64> {
    if kind == NotificationKind::Error {
        return None;
    }
    let ms = duration.unwrap_or(toaster_duration);
    (ms > 0).then_some(ms)
}

/// `.cu-toaster--top-right` etc.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToasterPosition {
    /// Top right (the default).
    #[default]
    TopRight,
    /// Top left.
    TopLeft,
    /// Bottom right.
    BottomRight,
    /// Bottom left.
    BottomLeft,
}

impl ToasterPosition {
    /// Every position.
    pub const ALL: [ToasterPosition; 4] = [
        Self::TopRight,
        Self::TopLeft,
        Self::BottomRight,
        Self::BottomLeft,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::TopRight => "top-right",
            Self::TopLeft => "top-left",
            Self::BottomRight => "bottom-right",
            Self::BottomLeft => "bottom-left",
        }
    }

    fn right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }

    fn bottom(self) -> bool {
        matches!(self, Self::BottomRight | Self::BottomLeft)
    }
}

type CloseHandler = Rc<dyn Fn(&mut App) -> bool>;

/// A Carbon toast notification. See the module docs.
#[derive(IntoElement)]
pub struct Toast {
    id: ElementId,
    kind: NotificationKind,
    title: SharedString,
    description: Option<SharedString>,
    caption: Option<SharedString>,
    on_close: Option<CloseHandler>,
    timeout: u64,
    low_contrast: bool,
    close_label: SharedString,
}

impl Toast {
    /// An info toast titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind: NotificationKind::Info,
            title: title.into(),
            description: None,
            caption: None,
            on_close: None,
            timeout: 0,
            low_contrast: false,
            close_label: "Close notification".into(),
        }
    }

    /// `variant`.
    pub fn variant(mut self, kind: NotificationKind) -> Self {
        self.kind = kind;
        self
    }

    /// `description` (Carbon `subtitle`).
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// `caption`.
    pub fn caption(mut self, text: impl Into<SharedString>) -> Self {
        self.caption = Some(text.into());
        self
    }

    /// `onClose`: return false to keep the toast open.
    pub fn on_close(mut self, f: impl Fn(&mut App) -> bool + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// `timeout` in ms (0 = never).
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout = ms;
        self
    }

    /// Carbon `lowContrast` (default false).
    pub fn low_contrast(mut self, low: bool) -> Self {
        self.low_contrast = low;
        self
    }

    /// The close button's accessible label.
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }
}

/// The toast card (no open state).
#[allow(clippy::too_many_arguments)]
fn card(
    id: ElementId,
    kind: NotificationKind,
    low_contrast: bool,
    title: SharedString,
    description: Option<SharedString>,
    caption: Option<SharedString>,
    close_label: SharedString,
    on_close: Callback,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::Div {
    let look = NotificationLook::of(kind, low_contrast);
    let width = toast_width(viewport_rem(window));
    div()
        .relative()
        .flex()
        .flex_row()
        .flex_none()
        .w(rem(width))
        .bg(look.background)
        .border_l(rem(ACCENT_WIDTH))
        .border_color(look.accent)
        .pl(rem(13.))
        .text_color(look.text)
        // `box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.2)`.
        .shadow(vec![
            BoxShadow::new(px(0.), px(2.), gpui_kit::black().opacity(0.2)).blur_radius(px(6.)),
        ])
        .child(
            div()
                .flex_none()
                .mt(rem(16.))
                .mr(rem(16.))
                .child(status_icon(kind, look.accent)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .mb(rem(16.))
                .mr(rem(16.))
                .child(
                    div()
                        .type_style(TypeStyle::HeadingCompact01)
                        .mt(rem(16.))
                        .child(title),
                )
                .when_some(description, |d, t| {
                    d.child(
                        div()
                            .type_style(TypeStyle::BodyCompact01)
                            .mb(rem(16.))
                            .child(t),
                    )
                })
                .when_some(caption, |d, t| {
                    d.child(
                        div()
                            .type_style(TypeStyle::BodyCompact01)
                            .pt(rem(8.))
                            .child(t),
                    )
                }),
        )
        .child(close_button(
            (id, "close").into(),
            look,
            close_label,
            on_close,
            window,
            cx,
        ))
}

impl RenderOnce for Toast {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open = window.use_keyed_state((id.clone(), "open"), cx, |_, _| true);
        let scheduled = window.use_keyed_state((id.clone(), "timer"), cx, |_, _| false);
        let close: Rc<dyn Fn(&mut App)> = {
            let open = open.clone();
            let on_close = self.on_close.clone();
            Rc::new(move |cx: &mut App| {
                let keep = on_close.as_ref().is_some_and(|f| !f(cx));
                if !keep {
                    open.update(cx, |o, cx| {
                        *o = false;
                        cx.notify();
                    });
                }
            })
        };
        if self.timeout > 0 && !*scheduled.read(cx) {
            scheduled.update(cx, |s, _| *s = true);
            let close = close.clone();
            let delay = Duration::from_millis(self.timeout);
            cx.spawn(async move |cx| {
                cx.background_executor().timer(delay).await;
                cx.update(|cx| close(cx));
            })
            .detach();
        }
        if !*open.read(cx) {
            return div().into_any_element();
        }
        card(
            id,
            self.kind,
            self.low_contrast,
            self.title,
            self.description,
            self.caption,
            self.close_label,
            Rc::new(move |_, cx| close(cx)),
            window,
            cx,
        )
        .into_any_element()
    }
}

/// A toast id from [`toast`].
pub type ToastId = u64;

/// Options of [`toast`]: `Omit<ToastProps, 'title'> & { duration }`.
#[derive(Clone, Debug, Default)]
pub struct ToastOptions {
    /// `variant` (default info).
    pub variant: Option<NotificationKind>,
    /// `description`.
    pub description: Option<SharedString>,
    /// `caption`.
    pub caption: Option<SharedString>,
    /// `duration` in ms (overrides the toaster's; ignored for errors).
    pub duration: Option<u64>,
}

impl ToastOptions {
    /// With a description.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// With a caption.
    pub fn caption(mut self, text: impl Into<SharedString>) -> Self {
        self.caption = Some(text.into());
        self
    }

    /// With a duration.
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration = Some(ms);
        self
    }
}

#[derive(Clone, Debug)]
struct Item {
    id: ToastId,
    kind: NotificationKind,
    title: SharedString,
    options: ToastOptions,
}

/// The app-global toast queue (`toastBus`).
#[derive(Default)]
struct ToastBus {
    items: Vec<Item>,
    next: ToastId,
    scheduled: HashSet<ToastId>,
}

impl Global for ToastBus {}

/// `toast(title, opts)`: raise a toast on every [`Toaster`]; returns its id.
pub fn toast(cx: &mut App, title: impl Into<SharedString>, options: ToastOptions) -> ToastId {
    let bus = cx.default_global::<ToastBus>();
    bus.next += 1;
    let id = bus.next;
    bus.items.push(Item {
        id,
        kind: options.variant.unwrap_or_default(),
        title: title.into(),
        options,
    });
    cx.refresh_windows();
    id
}

fn with_kind(
    cx: &mut App,
    kind: NotificationKind,
    title: impl Into<SharedString>,
    options: ToastOptions,
) -> ToastId {
    toast(
        cx,
        title,
        ToastOptions {
            variant: Some(kind),
            ..options
        },
    )
}

/// `toast.success`.
pub fn success(cx: &mut App, title: impl Into<SharedString>, options: ToastOptions) -> ToastId {
    with_kind(cx, NotificationKind::Success, title, options)
}

/// `toast.error` (never auto-dismisses).
pub fn error(cx: &mut App, title: impl Into<SharedString>, options: ToastOptions) -> ToastId {
    with_kind(cx, NotificationKind::Error, title, options)
}

/// `toast.warning`.
pub fn warning(cx: &mut App, title: impl Into<SharedString>, options: ToastOptions) -> ToastId {
    with_kind(cx, NotificationKind::Warning, title, options)
}

/// `toast.info`.
pub fn info(cx: &mut App, title: impl Into<SharedString>, options: ToastOptions) -> ToastId {
    with_kind(cx, NotificationKind::Info, title, options)
}

/// `toast.dismiss(id)`: the toast leaves the queue at once (no exit motion).
pub fn dismiss(cx: &mut App, id: ToastId) {
    let bus = cx.default_global::<ToastBus>();
    let before = bus.items.len();
    bus.items.retain(|t| t.id != id);
    bus.scheduled.remove(&id);
    if bus.items.len() != before {
        cx.refresh_windows();
    }
}

/// Dismiss every toast.
pub fn dismiss_all(cx: &mut App) {
    let ids: Vec<ToastId> = cx
        .default_global::<ToastBus>()
        .items
        .iter()
        .map(|t| t.id)
        .collect();
    for id in ids {
        dismiss(cx, id);
    }
}

/// The toast host (`<Toaster position duration>`). Place one per window.
#[derive(IntoElement)]
pub struct Toaster {
    id: ElementId,
    position: ToasterPosition,
    duration: u64,
    close_label: SharedString,
}

impl Toaster {
    /// A top-right toaster with the 5000ms default duration.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            position: ToasterPosition::TopRight,
            duration: DEFAULT_DURATION_MS,
            close_label: "Close notification".into(),
        }
    }

    /// `position`.
    pub fn position(mut self, position: ToasterPosition) -> Self {
        self.position = position;
        self
    }

    /// `duration` in ms.
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration = ms;
        self
    }

    /// The close buttons' accessible label.
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }
}

impl RenderOnce for Toaster {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items = cx.default_global::<ToastBus>().items.clone();
        // Start the auto-dismiss timers of new toasts.
        for item in &items {
            let timeout = effective_timeout(item.kind, item.options.duration, self.duration);
            let bus = cx.default_global::<ToastBus>();
            if bus.scheduled.contains(&item.id) {
                continue;
            }
            bus.scheduled.insert(item.id);
            if let Some(ms) = timeout {
                let id = item.id;
                cx.spawn(async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(ms))
                        .await;
                    cx.update(|cx| dismiss(cx, id));
                })
                .detach();
            }
        }

        let width = toast_width(viewport_rem(window));
        let position = self.position;
        let mut stack = div()
            .flex()
            .flex_col()
            .w(rem(width + 2. * TOASTER_PADDING))
            .px(rem(TOASTER_PADDING))
            // The last slot carries an 8px gap; trim it from the padding.
            .pt(rem(TOASTER_PADDING))
            .pb(rem(TOASTER_PADDING - TOASTER_GAP));
        for item in items {
            let key = ElementId::NamedChild(
                std::sync::Arc::new(self.id.clone()),
                SharedString::from(format!("toast-{}", item.id)),
            );
            let id = item.id;
            let toast_card = card(
                key.clone(),
                item.kind,
                false,
                item.title.clone(),
                item.options.description.clone(),
                item.options.caption.clone(),
                self.close_label.clone(),
                Rc::new(move |_, cx| dismiss(cx, id)),
                window,
                cx,
            );
            stack = stack.child(div().flex_none().pb(rem(TOASTER_GAP)).child(toast_card));
        }

        let viewport = window.viewport_size();
        deferred(
            anchored().position(point(px(0.), px(0.))).child(
                div()
                    .w(viewport.width)
                    .h(viewport.height)
                    .flex()
                    .flex_col()
                    .map(|d| {
                        if position.bottom() {
                            d.justify_end()
                        } else {
                            d.justify_start()
                        }
                    })
                    .map(|d| {
                        if position.right() {
                            d.items_end()
                        } else {
                            d.items_start()
                        }
                    })
                    .child(stack),
            ),
        )
        .with_priority(TOASTER_PRIORITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeouts_follow_toaster_rules() {
        // `t.variant === 'error' ? 0 : t.duration ?? duration`.
        assert_eq!(
            effective_timeout(NotificationKind::Error, Some(100), 5000),
            None
        );
        assert_eq!(
            effective_timeout(NotificationKind::Info, None, 5000),
            Some(5000)
        );
        assert_eq!(
            effective_timeout(NotificationKind::Success, Some(800), 5000),
            Some(800)
        );
        assert_eq!(effective_timeout(NotificationKind::Warning, None, 0), None);
    }

    #[test]
    fn geometry() {
        assert_eq!(toast_width(80.), 288.);
        assert_eq!(toast_width(100.), 352.);
        assert_eq!(TOASTER_GAP, 8.);
        assert_eq!(TOASTER_PADDING, 16.);
        assert_eq!(ToasterPosition::BottomLeft.name(), "bottom-left");
    }

    /// No toast motion in the reference: dismissing removes the toast at once.
    #[gpui_kit::test]
    fn dismiss_is_instant(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            let first = toast(cx, "One", ToastOptions::default());
            let second = toast(cx, "Two", ToastOptions::default());
            dismiss(cx, first);
            let ids: Vec<ToastId> = cx
                .default_global::<ToastBus>()
                .items
                .iter()
                .map(|t| t.id)
                .collect();
            assert_eq!(ids, vec![second]);
        });
    }
}
