//! ActionableToast: Carbon's `ActionableNotification` in its toast form
//! (`.cds--actionable-notification--toast`), the toast that carries one
//! tertiary action. The notifications pattern allows it exactly one
//! tertiary button (the inline form takes a ghost one: [`crate::components::Alert`]
//! with an action).
//!
//! - **Look**: [`NotificationLook`] (high contrast by default, as the
//!   reference passes no `lowContrast`): `background-inverse`, a 3px
//!   `support-*-inverse` start border and icon, `text-inverse`, the toast
//!   shadow `0 2px 6px 0 rgba(0,0,0,.2)`.
//! - **Geometry**: 18rem (`max-inline-size: 18rem; min-inline-size: 18rem`),
//!   wrapping (`flex-wrap: wrap`): the icon (`margin-block-start: 1rem;
//!   margin-inline-end: 1rem`), the details (`margin-inline: 0.8125rem 3rem`
//!   less the icon row) with the text wrapper `padding: 0.9375rem 0
//!   1.4375rem 0`, the 3rem close button in the top-end corner, and the
//!   action on its own line (`margin-block-end: 1rem; margin-inline-start:
//!   calc(2rem + 1.25rem - 0.125rem)`).
//! - **Action colors**: on high contrast the tertiary button is drawn
//!   inverse (`notification-action-tertiary-inverse` border and text); the
//!   action and close render in a [`theme::zone`] of the opposite theme
//!   ([`inverse_theme`]), whose tertiary and ghost buttons are exactly the
//!   light-on-dark (or dark-on-light) pair. Low contrast keeps the theme.
//! - **Behavior**: Base buttons (focus, Enter/Space, pointer); closing calls
//!   `on_close` and hides the toast (no transition, as Carbon).
use crate::components::{
    Button, ButtonSize, ButtonVariant, alert::NotificationKind, alert::NotificationLook,
};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme,
    tokens::{CarbonTheme, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, BoxShadow, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// `.cds--actionable-notification--toast { min-inline-size: 18rem; max-inline-size: 18rem }`.
pub const WIDTH_REM: f32 = 18.;
/// `border-inline-start: 3px solid …`.
pub const ACCENT_PX: f32 = 3.;
/// `.cds--actionable-notification__details { margin-inline-start: 0.8125rem }`.
pub const DETAILS_START_REM: f32 = 0.8125;
/// `.cds--actionable-notification--toast .cds--actionable-notification__text-wrapper
/// { padding: 0.9375rem 0 1.4375rem 0 }`.
pub const TEXT_PADDING_REM: (f32, f32) = (0.9375, 1.4375);
/// `.cds--actionable-notification__action-button.cds--btn--tertiary
/// { margin-inline-start: calc(2rem + 1.25rem - 0.125rem); margin-block-end: 1rem }`.
pub const ACTION_INSET_REM: (f32, f32) = (2. + 1.25 - 0.125, 1.);
/// The close button's 3rem square.
pub const CLOSE_REM: f32 = 3.;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The theme the action and close button render in on a high-contrast toast:
/// light themes take G100, dark ones White.
pub fn inverse_theme(theme: CarbonTheme) -> CarbonTheme {
    match theme {
        CarbonTheme::White | CarbonTheme::G10 => CarbonTheme::G100,
        CarbonTheme::G90 | CarbonTheme::G100 => CarbonTheme::White,
    }
}

/// An actionable toast. Callers pass every string.
#[derive(IntoElement)]
pub struct ActionableToast {
    id: ElementId,
    kind: NotificationKind,
    title: SharedString,
    subtitle: Option<SharedString>,
    action_label: SharedString,
    on_action: Option<Handler>,
    close_label: SharedString,
    on_close: Option<Handler>,
    low_contrast: bool,
}

impl ActionableToast {
    /// A toast with a stable `id`, its `title` and the action's label
    /// (`actionButtonLabel`).
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        action_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: NotificationKind::Info,
            title: title.into(),
            subtitle: None,
            action_label: action_label.into(),
            on_action: None,
            close_label: SharedString::default(),
            on_close: None,
            low_contrast: false,
        }
    }

    /// `kind` (default info).
    pub fn kind(mut self, kind: NotificationKind) -> Self {
        self.kind = kind;
        self
    }

    /// `subtitle`.
    pub fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// `onActionButtonClick`.
    pub fn on_action(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }

    /// The close button's accessible name (`ariaLabel`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// `onClose`, after the toast hides.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// `lowContrast` (default false).
    pub fn low_contrast(mut self, low: bool) -> Self {
        self.low_contrast = low;
        self
    }
}

impl RenderOnce for ActionableToast {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open = window.use_keyed_state((id.clone(), "open"), cx, |_, _| true);
        if !*open.read(cx) {
            return div().into_any_element();
        }
        let look = NotificationLook::of(self.kind, self.low_contrast);
        let zone = if self.low_contrast {
            theme::current()
        } else {
            inverse_theme(theme::current())
        };

        let action_id = ElementId::from((id.clone(), "action"));
        let label = self.action_label.clone();
        let on_action = self.on_action.clone();
        let action = theme::zone(zone, move |_, _| {
            Button::new(action_id, label)
                .variant(ButtonVariant::Tertiary)
                .size(ButtonSize::Sm)
                // Computed: `padding-block: 0`, the label at the top.
                .label_top(true)
                .when_some(on_action, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
                .into_any_element()
        });

        let close_id = ElementId::from((id.clone(), "close"));
        let close_label = self.close_label.clone();
        let on_close = self.on_close.clone();
        let close = theme::zone(zone, move |_, _| {
            Button::new(close_id, close_label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Lg)
                .density(crate::density::Density::Comfortable)
                .icon(CarbonIcon::Close)
                .icon_only(true)
                .tooltip(false)
                .on_click(move |_, w, cx| {
                    open.update(cx, |o, cx| {
                        *o = false;
                        cx.notify();
                    });
                    if let Some(f) = &on_close {
                        f(w, cx);
                    }
                })
                .into_any_element()
        });

        div()
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .w(rems(WIDTH_REM))
            .bg(look.background)
            .border_l(px(ACCENT_PX))
            .border_color(look.accent)
            .text_color(look.text)
            // `.cds--actionable-notification--toast { box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.2) }`.
            .shadow(vec![
                BoxShadow::new(px(0.), px(2.), gpui_kit::black().opacity(0.2)).blur_radius(px(6.)),
            ])
            // No 1px outline: `.cds--actionable-notification--low-contrast
            // :not(.cds--actionable-notification--toast)::before`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .pl(rems(DETAILS_START_REM))
                    .pr(rems(CLOSE_REM))
                    .child(
                        // `.cds--toast-notification__icon { margin-block-start: 1rem;
                        // margin-inline-end: 1rem }`, 20px.
                        div().flex_none().mt(rems(1.)).mr(rems(1.)).child(
                            Icon::carbon(self.kind.icon())
                                .size(IconSize::S20)
                                .color(look.accent),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .pt(rems(TEXT_PADDING_REM.0))
                            .pb(rems(TEXT_PADDING_REM.1))
                            // `.cds--actionable-notification__title`: heading-compact-01.
                            .child(
                                div()
                                    .type_style(TypeStyle::HeadingCompact01)
                                    .child(self.title),
                            )
                            // `__subtitle`: body-compact-01.
                            .when_some(self.subtitle, |d, t| {
                                d.child(div().type_style(TypeStyle::BodyCompact01).child(t))
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .ml(rems(ACTION_INSET_REM.0))
                    .mb(rems(ACTION_INSET_REM.1))
                    .child(action),
            )
            .child(div().absolute().top_0().right_0().child(close))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_follows_the_css() {
        assert_eq!(WIDTH_REM * 16., 288.);
        assert_eq!(ACTION_INSET_REM.0 * 16., 50.);
        assert_eq!(TEXT_PADDING_REM, (0.9375, 1.4375));
    }

    #[test]
    fn actions_render_in_the_opposite_theme() {
        assert_eq!(inverse_theme(CarbonTheme::White), CarbonTheme::G100);
        assert_eq!(inverse_theme(CarbonTheme::G10), CarbonTheme::G100);
        assert_eq!(inverse_theme(CarbonTheme::G90), CarbonTheme::White);
        assert_eq!(inverse_theme(CarbonTheme::G100), CarbonTheme::White);
    }
}
