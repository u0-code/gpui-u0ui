//! Alert: shadcn `Alert` → Carbon `InlineNotification`, or the inline
//! `ActionableNotification` when an `action` is given (`.cds--inline-notification*`,
//! `.cds--actionable-notification*`).
//!
//! - **Kinds**: `index.jsx` `KINDOF` maps `default` → info, `destructive` →
//!   error, and `success` / `warning` / `info` to themselves ([`AlertVariant::kind`]).
//! - **Colors** ([`NotificationLook`], shared with [`super::toast`]): low contrast
//!   (the default, `lowContrast = true`) paints `notification-background-*`,
//!   a 3px `support-*` start border, `support-*` icon, `text-primary`, and a
//!   1px top/end/bottom outline in the support color at 40% (`::before { border-width:
//!   1px 1px 1px 0; filter: opacity(0.4) }`); high contrast paints
//!   `background-inverse`, `support-*-inverse` border and icon, `text-inverse`.
//!   The warning icon's "!" is black (`path[opacity="0"] { fill: #000; opacity: 1 }`).
//! - **Geometry**: `min-block-size: 3rem`, `min-inline-size: 18rem`, full width
//!   up to `38rem` (`46rem` from a 66rem-wide viewport, `52rem` from 99rem).
//!   From a 42rem-wide viewport the row does not wrap, the details have
//!   `margin: 0 0.8125rem` and the 48px close button sits in the flow; below it
//!   the row wraps, the details keep `margin-inline-end: 3rem` and the close
//!   button is absolute at the top end. Icon 20px with `margin-block-start:
//!   0.875rem; margin-inline-end: 1rem`; text wrapper `padding: 0.9375rem 0`
//!   with the `heading-compact-01` title (4px end margin) and the
//!   `body-compact-01` subtitle on one wrapping line.
//! - **Action** (`.cds--actionable-notification__action-button.cds--btn--ghost`):
//!   a 2rem ghost button (`margin: 0.5rem 0`; narrow `margin-inline-start: 2.5rem`),
//!   `link-inverse` text on high contrast, hover `background-inverse-hover` /
//!   `notification-action-hover`, a 2px `focus-inverse` / `focus` outline.
//! - **Close**: a 3rem square button with the 20px `Close` icon (`icon-inverse`
//!   / `icon-primary`), no hover fill, a 2px `focus-inverse` / `focus` inset
//!   outline (`transition: outline 110ms`). Closing unmounts the notification
//!   (Carbon has no hide transition) after calling `on_close`.
//! - **Behavior**: the buttons are Base buttons (focus, Enter/Space, pointer).
use super::form::{inset_outline, rem};
use super::overlay::Callback;
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// A Carbon notification kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NotificationKind {
    /// `error`.
    Error,
    /// `success`.
    Success,
    /// `warning`.
    Warning,
    /// `info` (the default).
    #[default]
    Info,
}

impl NotificationKind {
    /// Every kind.
    pub const ALL: [NotificationKind; 4] = [Self::Error, Self::Success, Self::Warning, Self::Info];

    /// Carbon's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }

    /// Carbon's `NotificationIcon`: `ErrorFilled`, `CheckmarkFilled`,
    /// `WarningFilled`, `InformationFilled` (20px).
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Error => CarbonIcon::ErrorFilled,
            Self::Success => CarbonIcon::CheckmarkFilled,
            Self::Warning => CarbonIcon::WarningFilled,
            Self::Info => CarbonIcon::InformationFilled,
        }
    }
}

/// `index.d.ts` `AlertProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlertVariant {
    /// `default` → info.
    #[default]
    Default,
    /// `destructive` → error.
    Destructive,
    /// `success`.
    Success,
    /// `warning`.
    Warning,
    /// `info`.
    Info,
    /// `error`.
    Error,
}

impl AlertVariant {
    /// Every variant.
    pub const ALL: [AlertVariant; 6] = [
        Self::Default,
        Self::Destructive,
        Self::Success,
        Self::Warning,
        Self::Info,
        Self::Error,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Destructive => "destructive",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Error => "error",
        }
    }

    /// `KINDOF = { default: 'info', destructive: 'error', error: 'error', … }`.
    pub const fn kind(self) -> NotificationKind {
        match self {
            Self::Default | Self::Info => NotificationKind::Info,
            Self::Destructive | Self::Error => NotificationKind::Error,
            Self::Success => NotificationKind::Success,
            Self::Warning => NotificationKind::Warning,
        }
    }
}

/// The colors of a notification (inline, actionable or toast).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NotificationLook {
    /// Fill.
    pub background: Hsla,
    /// The 3px start border and the status icon.
    pub accent: Hsla,
    /// The low-contrast 1px outline (`::before`, already at 40%), if any.
    pub outline: Option<Hsla>,
    /// Title and subtitle.
    pub text: Hsla,
    /// The close icon.
    pub close_icon: Hsla,
    /// Focus outlines.
    pub focus: Hsla,
}

impl NotificationLook {
    /// The look of `kind`, low or high contrast.
    pub fn of(kind: NotificationKind, low_contrast: bool) -> Self {
        use NotificationKind::*;
        if low_contrast {
            // `.cds--inline-notification--low-contrast.cds--inline-notification--error
            // { background: notification-background-error; border-inline-start: 3px
            // solid support-error }`, `::before { border-color: support-error }`.
            let (bg, accent) = match kind {
                Error => (
                    ColorRole::NotificationBackgroundError,
                    ColorRole::SupportError,
                ),
                Success => (
                    ColorRole::NotificationBackgroundSuccess,
                    ColorRole::SupportSuccess,
                ),
                Warning => (
                    ColorRole::NotificationBackgroundWarning,
                    ColorRole::SupportWarning,
                ),
                Info => (
                    ColorRole::NotificationBackgroundInfo,
                    ColorRole::SupportInfo,
                ),
            };
            Self {
                background: color(bg),
                accent: color(accent),
                // `filter: opacity(0.4)`.
                outline: Some(color(accent).opacity(0.4)),
                // `.cds--inline-notification--low-contrast { color: text-primary }`.
                text: color(ColorRole::TextPrimary),
                close_icon: color(ColorRole::IconPrimary),
                focus: color(ColorRole::Focus),
            }
        } else {
            // `.cds--inline-notification--error { background: background-inverse;
            // border-inline-start: 3px solid support-error-inverse }`.
            let accent = match kind {
                Error => ColorRole::SupportErrorInverse,
                Success => ColorRole::SupportSuccessInverse,
                Warning => ColorRole::SupportWarningInverse,
                Info => ColorRole::SupportInfoInverse,
            };
            Self {
                background: color(ColorRole::BackgroundInverse),
                accent: color(accent),
                outline: None,
                text: color(ColorRole::TextInverse),
                close_icon: color(ColorRole::IconInverse),
                focus: color(ColorRole::FocusInverse),
            }
        }
    }
}

/// `border-inline-start: 3px solid …`.
pub const ACCENT_WIDTH: f32 = 3.;
/// `min-block-size: 3rem` and the close button's 3rem square.
pub const ROW_HEIGHT: f32 = 48.;
/// `.cds--inline-notification__details { margin: 0 0.8125rem }`.
pub const DETAILS_MARGIN: f32 = 13.;

/// `max-inline-size` for a viewport `width_rem` wide: `18rem`, `38rem` from
/// 42rem, `46rem` from 66rem, `52rem` from 99rem.
pub fn max_width(width_rem: f32) -> f32 {
    if width_rem >= 99. {
        52. * 16.
    } else if width_rem >= 66. {
        46. * 16.
    } else if width_rem >= 42. {
        38. * 16.
    } else {
        18. * 16.
    }
}

/// `@media (min-width: 42rem)`: the wide (non-wrapping) layout.
pub fn is_wide(width_rem: f32) -> bool {
    width_rem >= 42.
}

/// The viewport width in rems.
pub(crate) fn viewport_rem(window: &Window) -> f32 {
    f32::from(window.viewport_size().width) / f32::from(window.rem_size())
}

/// The 20px status icon; the warning glyph's "!" filled black
/// (`.cds--inline-notification--warning .cds--inline-notification__icon
/// path[opacity="0"] { fill: #000000; opacity: 1 }`).
pub(crate) fn status_icon(kind: NotificationKind, accent: Hsla) -> AnyElement {
    let icon = Icon::carbon(kind.icon()).size(IconSize::S20).color(accent);
    if kind != NotificationKind::Warning {
        return icon.into_any_element();
    }
    div()
        .relative()
        .size(rem(20.))
        .flex_none()
        .child(
            div()
                .absolute()
                .left(rem(8.5))
                .top(rem(5.))
                .w(rem(3.))
                .h(rem(10.5))
                .bg(gpui_kit::black()),
        )
        .child(icon.absolute().top_0().left_0())
        .into_any_element()
}

/// `transition: outline 110ms cubic-bezier(0.2, 0, 0.38, 0.9)` (close button).
pub const CLOSE_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);

/// The 3rem close button.
pub(crate) fn close_button(
    id: ElementId,
    look: NotificationLook,
    label: SharedString,
    on_click: Callback,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::base::Button {
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let outline = motion::animate_color(
        (id.clone(), "outline"),
        if focus.is_focused(window) {
            look.focus
        } else {
            look.focus.opacity(0.)
        },
        CLOSE_MOTION,
        window,
        cx,
    );
    gpui_kit::base::Button::new(id)
        .track_focus(&focus)
        .accessibility_label(label)
        .relative()
        .flex_none()
        .size(rem(ROW_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .cursor(CursorStyle::PointingHand)
        .child(
            Icon::carbon(CarbonIcon::Close)
                .size(IconSize::S20)
                .color(look.close_icon),
        )
        .child(inset_outline(2., outline))
        .on_click({
            let focus = focus.clone();
            move |_, window, cx| {
                // Closing removes the button: keep the Tab starting point.
                crate::u0::primitives::parts::vacate_focus(&focus, window, cx);
                on_click(window, cx)
            }
        })
}

/// `AlertTitle`.
pub struct AlertTitle(pub SharedString);

impl AlertTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

/// `AlertDescription`.
pub struct AlertDescription(pub AnyElement);

impl AlertDescription {
    /// A description.
    pub fn new(content: impl IntoElement) -> Self {
        Self(content.into_any_element())
    }
}

/// `AlertProps.action`: `{ label, onClick }`.
pub struct AlertAction {
    label: SharedString,
    on_click: Option<Callback>,
}

impl AlertAction {
    /// An action button labeled `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }

    /// `onClick`.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// An inline notification. See the module docs.
#[derive(IntoElement)]
pub struct Alert {
    id: ElementId,
    variant: AlertVariant,
    title: Option<SharedString>,
    description: Vec<AnyElement>,
    action: Option<AlertAction>,
    full_width: bool,
    on_close: Option<Callback>,
    close_label: SharedString,
    low_contrast: bool,
}

impl Alert {
    /// A low-contrast info alert.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: AlertVariant::Default,
            title: None,
            description: Vec::new(),
            action: None,
            full_width: false,
            on_close: None,
            close_label: "Close notification".into(),
            low_contrast: true,
        }
    }

    /// `variant`.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `title`.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The description (`children`).
    pub fn description(mut self, content: impl IntoElement) -> Self {
        self.description.push(content.into_any_element());
        self
    }

    /// `<AlertTitle>` child.
    pub fn alert_title(mut self, title: AlertTitle) -> Self {
        self.title = Some(title.0);
        self
    }

    /// `<AlertDescription>` child.
    pub fn alert_description(mut self, description: AlertDescription) -> Self {
        self.description.push(description.0);
        self
    }

    /// Drop the `max-inline-size` cap (`style: { maxWidth: 'none' }`), as
    /// a banner across the content does. Off by default.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// `action`: makes it an `ActionableNotification`.
    pub fn action(mut self, action: AlertAction) -> Self {
        self.action = Some(action);
        self
    }

    /// `onClose`: shows the close button.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// The close button's accessible label (Carbon `ariaLabel`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// `lowContrast` (default true).
    pub fn low_contrast(mut self, low: bool) -> Self {
        self.low_contrast = low;
        self
    }
}

/// The ghost action button (`.cds--actionable-notification__action-button.cds--btn--ghost`).
fn action_button(
    id: ElementId,
    action: AlertAction,
    low_contrast: bool,
    look: NotificationLook,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let pointer = window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| (false, false));
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let (hovered, pressed) = *pointer.read(cx);
    // `:hover, :active { background-color: background-inverse-hover }`;
    // low contrast `notification-action-hover`.
    let hover_bg = color(if low_contrast {
        ColorRole::NotificationActionHover
    } else {
        ColorRole::BackgroundInverseHover
    });
    let bg = motion::animate_color(
        (id.clone(), "bg"),
        if hovered || pressed {
            hover_bg
        } else {
            hover_bg.opacity(0.)
        },
        Motion::HOVER,
        window,
        cx,
    );
    // `.cds--btn--ghost { color: var(--link-primary) }`; high contrast `link-inverse`.
    let text = color(if low_contrast {
        ColorRole::LinkPrimary
    } else {
        ColorRole::LinkInverse
    });
    let on_click = action.on_click.clone();
    gpui_kit::base::Button::new(id)
        .track_focus(&focus)
        .accessibility_label(action.label.clone())
        .relative()
        .flex_none()
        .h(rem(32.))
        // `.cds--actionable-notification .cds--actionable-notification__action-button.cds--btn
        // { block-size: 2rem; padding: 0 var(--spacing-04) }` (the bundle's r3-set rule).
        .px(rem(12.))
        .flex()
        .items_center()
        .whitespace_nowrap()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(text)
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .on_hover({
            let pointer = pointer.clone();
            move |h, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.0 = *h;
                    if !h {
                        p.1 = false;
                    }
                    cx.notify();
                })
            }
        })
        .on_mouse_down(MouseButton::Left, {
            let pointer = pointer.clone();
            move |_, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.1 = true;
                    cx.notify();
                })
            }
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            pointer.update(cx, |p, cx| {
                p.1 = false;
                cx.notify();
            })
        })
        .when_some(on_click, |d, f| {
            let focus = focus.clone();
            d.on_click(move |_, window, cx| {
                crate::u0::primitives::parts::vacate_focus(&focus, window, cx);
                f(window, cx)
            })
        })
        .child(action.label)
        // `:focus { outline: 2px solid var(--focus-inverse) }`; low contrast `focus`.
        .when(focus.is_focused(window), |d| {
            d.child(inset_outline(2., look.focus))
        })
}

impl RenderOnce for Alert {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open = window.use_keyed_state((id.clone(), "open"), cx, |_, _| true);
        if !*open.read(cx) {
            return div().into_any_element();
        }
        let kind = self.variant.kind();
        let look = NotificationLook::of(kind, self.low_contrast);
        let vw = viewport_rem(window);
        let wide = is_wide(vw);
        let closable = self.on_close.is_some();

        // `.cds--inline-notification__text-wrapper { display: flex; flex-wrap: wrap;
        // padding: 0.9375rem 0 }`.
        let text = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .flex_1()
            .min_w_0()
            .py(rem(15.))
            .text_color(look.text)
            .when_some(self.title.clone(), |d, t| {
                d.child(
                    div()
                        .flex_shrink_1()
                        .min_w_0()
                        .type_style(TypeStyle::HeadingCompact01)
                        .mr(rem(4.))
                        .child(t),
                )
            })
            .children(self.description.into_iter().map(|c| {
                div()
                    .flex_shrink_1()
                    .min_w_0()
                    .type_style(TypeStyle::BodyCompact01)
                    .child(c)
            }));
        let details = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_w_0()
            .ml(rem(DETAILS_MARGIN))
            .mr(rem(if wide || !closable {
                DETAILS_MARGIN
            } else {
                48.
            }))
            .when(!wide, |d| d.w_full())
            .child(
                div()
                    .flex_none()
                    .mt(rem(14.))
                    .mr(rem(16.))
                    .child(status_icon(kind, look.accent)),
            )
            .child(text);

        let actionable = self.action.is_some();
        let action = self.action.map(|a| {
            div()
                .flex()
                .flex_none()
                .map(|d| {
                    if wide {
                        d.my(rem(8.))
                    } else {
                        d.ml(rem(40.)).mb(rem(8.))
                    }
                })
                // `--hide-close-button … { margin-inline-end: 0.5rem }`.
                .when(!closable, |d| d.mr(rem(8.)))
                .child(action_button(
                    (id.clone(), "action").into(),
                    a,
                    self.low_contrast,
                    look,
                    window,
                    cx,
                ))
        });

        let close = self.on_close.clone().map(|f| {
            let open = open.clone();
            let button = close_button(
                (id.clone(), "close").into(),
                look,
                self.close_label.clone(),
                Rc::new(move |window, cx| {
                    f(window, cx);
                    open.update(cx, |o, cx| {
                        *o = false;
                        cx.notify();
                    });
                }),
                window,
                cx,
            );
            if wide {
                button
            } else {
                button.absolute().top_0().right_0()
            }
        });

        div()
            .id(id)
            .relative()
            .flex()
            .flex_row()
            .when(!wide, |d| d.flex_wrap())
            .w_full()
            .when(self.full_width, |d| d.min_w_0())
            .when(!self.full_width, |d| {
                d.min_w(rem(288.)).max_w(rem(max_width(vw)))
            })
            .min_h(rem(ROW_HEIGHT))
            .bg(look.background)
            .border_l(rem(ACCENT_WIDTH))
            .border_color(look.accent)
            .when_some(look.outline, |d, outline| {
                // `::before { border-width: 1px 1px 1px 0; filter: opacity(0.4) }`.
                d.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .border_t(px(1.))
                        .border_r(px(1.))
                        .border_b(px(1.))
                        .border_color(outline),
                )
            })
            .child(details)
            .map(|d| {
                if actionable && wide {
                    // The bundle's r3-set rules for a (non-toast) actionable
                    // notification: `align-items: flex-start`, and the
                    // `__button-wrapper` (action + close) `align-self:
                    // flex-start; margin: 10px 16px 16px 16px`.
                    d.items_start().child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_none()
                            .mt(rem(10.))
                            .mx(rem(16.))
                            .mb(rem(16.))
                            .children(action)
                            .children(close),
                    )
                } else {
                    d.children(action).children(close)
                }
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn variants_map_like_kindof() {
        assert_eq!(AlertVariant::Default.kind(), NotificationKind::Info);
        assert_eq!(AlertVariant::Destructive.kind(), NotificationKind::Error);
        assert_eq!(AlertVariant::Warning.kind(), NotificationKind::Warning);
        assert_eq!(
            NotificationKind::Success.icon(),
            CarbonIcon::CheckmarkFilled
        );
    }

    #[test]
    fn looks_match_bundle() {
        crate::theme::tests_set(CarbonTheme::White);
        let low = NotificationLook::of(NotificationKind::Error, true);
        assert_eq!(
            low.background,
            color(ColorRole::NotificationBackgroundError)
        );
        assert_eq!(low.accent, color(ColorRole::SupportError));
        assert_eq!(
            low.outline,
            Some(color(ColorRole::SupportError).opacity(0.4))
        );
        assert_eq!(low.text, color(ColorRole::TextPrimary));
        assert_eq!(low.close_icon, color(ColorRole::IconPrimary));
        let high = NotificationLook::of(NotificationKind::Success, false);
        assert_eq!(high.background, color(ColorRole::BackgroundInverse));
        assert_eq!(high.accent, color(ColorRole::SupportSuccessInverse));
        assert_eq!(high.outline, None);
        assert_eq!(high.text, color(ColorRole::TextInverse));
        assert_eq!(high.focus, color(ColorRole::FocusInverse));
        // Low-contrast warning keeps `support-warning` for border and icon.
        assert_eq!(
            NotificationLook::of(NotificationKind::Warning, true).accent,
            color(ColorRole::SupportWarning)
        );
    }

    #[test]
    fn widths_follow_media_queries() {
        assert_eq!(max_width(30.), 288.);
        assert_eq!(max_width(42.), 608.);
        assert_eq!(max_width(70.), 736.);
        assert_eq!(max_width(100.), 832.);
        assert!(!is_wide(41.9) && is_wide(42.));
        assert_eq!(ROW_HEIGHT, 48.);
        assert_eq!(ACCENT_WIDTH, 3.);
    }
}
