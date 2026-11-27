//! Dialog: shadcn `Dialog` → Carbon `Modal` (`.cds--modal`).
//!
//! - **Behavior** is Kit's: the host is [`gpui_kit::base::Dialog`] (focus
//!   trap, Escape, backdrop press, deferred top layer, dialog role). Kit's
//!   styled `component::dialog` is not used: it is opened imperatively through
//!   `Root` and draws its own header/footer/radius/entrance, none of which can
//!   be made Carbon (64px split footer, translateY(-24px) presence). u0ui adds
//!   what Base leaves to callers: open state (controlled or not), focus moving
//!   in on open and back on close, and the presence motion.
//! - **Appearance**: `.cds--modal-container` on `layer-01` with a 1px
//!   `border-subtle-01` line, Carbon's responsive widths per size ([`ModalGeometry`]),
//!   header/body/footer geometry from `bundle.css`, footer buttons 64px tall and
//!   split 50/50 (`.cds--modal-footer .cds--btn { flex: 0 1 50% }`).
//! - **Motion** (`.cds--modal`, `.cds--modal-container`): the scrim and the
//!   container fade over 240ms; the container also slides from
//!   `translate3d(0, -24px, 0)`. Entrance `cubic-bezier(0, 0, 0.3, 1)`, exit
//!   `cubic-bezier(0.4, 0.14, 1, 1)`. The exit plays before the host unmounts;
//!   reopening mid-exit reverses from the current value. Reduced motion keeps
//!   the fade and drops the slide.
//!
//! ```ignore
//! Dialog::new("rename")
//!     .trigger(Button::new("open", "Open dialog"))
//!     .label("Settings")
//!     .title("Rename project")
//!     .primary_text("Save")
//!     .secondary_text("Cancel")
//!     .child(input)
//! ```
use super::button::{Button, ButtonSize, ButtonVariant};
use super::overlay::{self, OpenChange, Translate, rem};
use crate::{
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// Modal entrance: `.cds--modal.is-visible { transition: opacity 240ms
/// cubic-bezier(0, 0, 0.3, 1) }` and `cds--presence-modal-container__enter`.
pub const MODAL_ENTER: Motion = Motion::new(Curve::ExpressiveEntrance, Speed::Moderate02);
/// Modal exit: `.cds--modal { transition: opacity 240ms cubic-bezier(0.4, 0.14, 1, 1) }`
/// and `cds--presence-modal-container__exit`.
pub const MODAL_EXIT: Motion = Motion::new(Curve::ExpressiveExit, Speed::Moderate02);
/// Container travel: `translate3d(0, -24px, 0)` → `translate3d(0, 0, 0)`.
pub const MODAL_TRAVEL: f32 = -24.;
/// `.cds--modal-close { transition: background-color 110ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
const CLOSE_HOVER: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);

/// `index.d.ts` `DialogProps.size` (`.cds--modal-container--<size>`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DialogSize {
    /// `xs`.
    Xs,
    /// `sm`, the reference default (`index.jsx`: `size = 'sm'`).
    #[default]
    Sm,
    /// `md` (Carbon's unsized container).
    Md,
    /// `lg`.
    Lg,
}

impl DialogSize {
    /// Every size.
    pub const ALL: [DialogSize; 4] = [Self::Xs, Self::Sm, Self::Md, Self::Lg];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// Container size and body text inset for a viewport, from the media queries
/// on `.cds--modal-container` (42rem / 66rem / 82rem).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalGeometry {
    /// `inline-size` as a fraction of the viewport width.
    pub width: f32,
    /// `max-block-size` as a fraction of the viewport height.
    pub max_height: f32,
    /// Below 42rem the container is pinned to the top (`inset-block-start: 0`),
    /// above it is centered by `.cds--modal { align-items: center }`.
    pub centered: bool,
    /// `.cds--modal-content > p { padding-inline-end }` as a fraction of the
    /// content width, minus [`Self::text_inset_px`].
    pub text_inset: f32,
    /// The fixed part of the paragraph inset in px (`calc(20% - 2rem)` → 32).
    pub text_inset_px: f32,
}

impl ModalGeometry {
    /// The geometry of `size` in a viewport `viewport_rem` wide (in rems).
    pub fn of(size: DialogSize, viewport_rem: f32) -> Self {
        // Columns: < 42rem, ≥ 42rem, ≥ 66rem, ≥ 82rem.
        let (widths, max_lg) = match size {
            // `.cds--modal-container--xs`: 48% / 32% (max 48%) / 24%.
            DialogSize::Xs => ([1., 0.48, 0.32, 0.24], 0.48),
            // `.cds--modal-container--sm`: 60% / 42% (max 72%) / 36%.
            DialogSize::Sm => ([1., 0.60, 0.42, 0.36], 0.72),
            // `.cds--modal-container`: 84% (max 90%) / 60% (max 84%) / 48%.
            DialogSize::Md => ([1., 0.84, 0.60, 0.48], 0.84),
            // `.cds--modal-container--lg`: 96% / 84% (max 96%) / 72%.
            DialogSize::Lg => ([1., 0.96, 0.84, 0.72], 0.96),
        };
        let column = if viewport_rem >= 82. {
            3
        } else if viewport_rem >= 66. {
            2
        } else if viewport_rem >= 42. {
            1
        } else {
            0
        };
        let max_height = match column {
            0 => 1.,
            1 => 0.90,
            _ => max_lg,
        };
        let (text_inset, text_inset_px) = match (size, column) {
            // `.cds--modal-container--xs .cds--modal-content > p { padding-inline-end: 0 }`.
            (DialogSize::Xs, _) => (0., 0.),
            // `--sm … > p { padding-inline-end: 0 }`, `20%` from 66rem.
            (DialogSize::Sm, 0 | 1) => (0., 0.),
            (DialogSize::Sm, _) => (0.2, 0.),
            // `.cds--modal-content > p { padding-inline-end: calc(20% - 2rem) }`.
            _ => (0.2, 32.),
        };
        Self {
            width: widths[column],
            max_height,
            centered: column > 0,
            text_inset,
            text_inset_px,
        }
    }

    /// The paragraph's right padding in px for a content box `width` px wide.
    pub fn text_padding(&self, width: f32) -> f32 {
        (width * self.text_inset - self.text_inset_px).max(0.)
    }
}

/// The colors of the modal surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalLook {
    /// `.cds--modal { background-color: var(--overlay) }`.
    pub scrim: Hsla,
    /// `.cds--modal-container { background-color: var(--layer) }` (layer-01).
    pub container: Hsla,
    /// `border: 1px solid var(--border-subtle-01)`.
    pub border: Hsla,
    /// `.cds--modal-header__label { color: var(--text-secondary) }`.
    pub label: Hsla,
    /// `.cds--modal-header__heading`, `.cds--modal-content { color: var(--text-primary) }`.
    pub text: Hsla,
    /// `.cds--modal-close__icon { fill: var(--icon-primary) }`.
    pub close_icon: Hsla,
    /// `.cds--modal-close:hover { background-color: var(--layer-hover) }`.
    pub close_hover: Hsla,
}

impl ModalLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        use ColorRole::*;
        Self {
            scrim: color(Overlay),
            container: color(Layer01),
            border: color(BorderSubtle01),
            label: color(TextSecondary),
            text: color(TextPrimary),
            close_icon: color(IconPrimary),
            close_hover: color(LayerHover01),
        }
    }
}

type Action = Rc<dyn Fn(&mut Window, &mut App)>;

/// A Carbon modal dialog. See the module docs.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    trigger: Option<AnyElement>,
    title: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    children: Vec<AnyElement>,
    primary_text: Option<SharedString>,
    secondary_text: Option<SharedString>,
    on_primary: Option<Action>,
    on_secondary: Option<Action>,
    danger: bool,
    passive: bool,
    size: DialogSize,
    primary_disabled: bool,
    alert: bool,
    close_on_backdrop: bool,
    close_label: SharedString,
    initial_focus: Option<FocusHandle>,
}

impl Dialog {
    /// A dialog with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            open: None,
            default_open: false,
            on_open_change: None,
            trigger: None,
            title: None,
            label: None,
            description: None,
            children: Vec::new(),
            primary_text: None,
            secondary_text: None,
            on_primary: None,
            on_secondary: None,
            danger: false,
            passive: false,
            size: DialogSize::Sm,
            primary_disabled: false,
            alert: false,
            close_on_backdrop: true,
            // Carbon's `closeButtonLabel` default; override per locale.
            close_label: "Close".into(),
            initial_focus: None,
        }
    }

    /// Controlled open state.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Initial open state when uncontrolled.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Called whenever the dialog asks to open or close.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// An element that opens the dialog when pressed.
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    /// The heading (`.cds--modal-header__heading`, heading-03).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The small label above the heading (`.cds--modal-header__label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// A paragraph at the top of the body (`.cu-dialog-desc`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The primary footer button's label; without it the dialog is passive.
    pub fn primary_text(mut self, text: impl Into<SharedString>) -> Self {
        self.primary_text = Some(text.into());
        self
    }

    /// The secondary footer button's label.
    pub fn secondary_text(mut self, text: impl Into<SharedString>) -> Self {
        self.secondary_text = Some(text.into());
        self
    }

    /// Primary action (`onRequestSubmit`). The dialog stays open unless the
    /// handler closes it.
    pub fn on_primary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_primary = Some(Rc::new(f));
        self
    }

    /// Secondary action; by default it closes the dialog.
    pub fn on_secondary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_secondary = Some(Rc::new(f));
        self
    }

    /// Danger modal: the primary button is `danger` and focus starts on the
    /// secondary button.
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Passive modal: no footer.
    pub fn passive(mut self, passive: bool) -> Self {
        self.passive = passive;
        self
    }

    /// Container size (default `sm`).
    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    /// Disable the primary button.
    pub fn primary_disabled(mut self, disabled: bool) -> Self {
        self.primary_disabled = disabled;
        self
    }

    /// `alertdialog` role (AlertDialog).
    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    /// Whether a press on the scrim closes the dialog (default true;
    /// `preventCloseOnClickOutside` turns it off).
    pub fn close_on_backdrop(mut self, close: bool) -> Self {
        self.close_on_backdrop = close;
        self
    }

    /// Accessible name of the close button.
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// Focus this element when the dialog opens (Carbon's
    /// `data-modal-primary-focus`).
    pub fn initial_focus(mut self, handle: &FocusHandle) -> Self {
        self.initial_focus = Some(handle.clone());
        self
    }

    fn is_passive(&self) -> bool {
        self.passive || self.primary_text.is_none()
    }

    #[cfg(test)]
    pub(crate) fn is_danger(&self) -> bool {
        self.danger
    }

    #[cfg(test)]
    pub(crate) fn is_alert(&self) -> bool {
        self.alert
    }

    #[cfg(test)]
    pub(crate) fn closes_on_backdrop(&self) -> bool {
        self.close_on_backdrop
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Focus handles of the dialog's own buttons, kept across frames.
struct Handles {
    primary: FocusHandle,
    secondary: FocusHandle,
    close: FocusHandle,
}

/// The footer buttons' size. Comfortable: `.cds--modal-footer .cds--btn {
/// height: 4rem }`, the xl (64px) button. Compact: the compact button rule
/// (`[data-density="compact"] .cds--btn:not(.cds--btn--sm):not(.cds--btn--xl)
/// :not(.cds--btn--2xl)`, five classes) beats the modal-footer one
/// (`[data-density="compact"] .cds--modal-footer .cds--btn:not(.cds--btn--sm)`,
/// four), so the buttons are `density-field-md` (32px, padding-block 0, text
/// centered) at the top of the 64px footer, as the reference computes.
pub fn footer_button_size(density: Density) -> ButtonSize {
    match density {
        Density::Compact => ButtonSize::Lg,
        Density::Comfortable => ButtonSize::Xl,
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(
            &id,
            self.open,
            self.default_open,
            self.on_open_change.clone(),
            window,
            cx,
        );
        let open = openness.open;
        let progress = overlay::presence(id.clone(), open, MODAL_ENTER, MODAL_EXIT, window, cx);
        let set_open = openness.setter();
        let handles = window
            .use_keyed_state((id.clone(), "handles"), cx, |_, cx| Handles {
                primary: cx.focus_handle(),
                secondary: cx.focus_handle(),
                close: cx.focus_handle(),
            })
            .read(cx);
        let (primary_focus, secondary_focus, close_focus) = (
            handles.primary.clone(),
            handles.secondary.clone(),
            handles.close.clone(),
        );
        let passive = self.is_passive();
        // Carbon: `[data-modal-primary-focus]` first; a danger modal starts on
        // its secondary button; otherwise the primary button.
        let initial = self.initial_focus.clone().or_else(|| {
            if passive {
                Some(close_focus.clone())
            } else if self.danger && self.secondary_text.is_some() {
                Some(secondary_focus.clone())
            } else if self.primary_disabled {
                Some(close_focus.clone())
            } else {
                Some(primary_focus.clone())
            }
        });
        let focus = overlay::surface_focus(&id, open, initial, window, cx);

        let trigger = self.trigger.map(|trigger| {
            let set_open = set_open.clone();
            overlay::trigger_wrapper(
                (id.clone(), "trigger"),
                trigger,
                Rc::new(move |window, cx| set_open(true, window, cx)),
            )
        });
        let root = div().flex().flex_shrink_0().children(trigger);
        if !open && progress <= 0. {
            return root.into_any_element();
        }

        let look = ModalLook::current();
        let viewport = window.viewport_size();
        let rem_size = f32::from(window.rem_size());
        let viewport_rem = f32::from(viewport.width) / rem_size;
        let geometry = ModalGeometry::of(self.size, viewport_rem);
        let scale = rem_size / crate::tokens::REM_PX;
        let width_px = f32::from(viewport.width) * geometry.width / scale;
        let max_height_px = f32::from(viewport.height) * geometry.max_height / scale;
        let reduced = motion::reduced_motion(cx);
        let travel = if reduced {
            0.
        } else {
            MODAL_TRAVEL * (1. - progress)
        };

        let close_hover = window.use_keyed_state((id.clone(), "close-hover"), cx, |_, _| false);
        let close_bg = motion::animate_color(
            (id.clone(), "close-bg"),
            if *close_hover.read(cx) {
                look.close_hover
            } else {
                Hsla::transparent_black()
            },
            CLOSE_HOVER,
            window,
            cx,
        );
        let close = {
            let set_open = set_open.clone();
            gpui_kit::base::Button::new((id.clone(), "close"))
                .track_focus(&close_focus)
                .accessibility_label(self.close_label.clone())
                .absolute()
                .top_0()
                .right_0()
                // `.cds--modal-close`: 3rem square, 0.75rem padding, 2px transparent border.
                .size(rem(48.))
                .flex()
                .items_center()
                .justify_center()
                .bg(close_bg)
                .border_2()
                .border_color(if close_focus.is_focused(window) {
                    color(ColorRole::Focus)
                } else {
                    Hsla::transparent_black()
                })
                .cursor(CursorStyle::PointingHand)
                .on_hover(move |hovered, _, cx| {
                    close_hover.update(cx, |h, cx| {
                        *h = *hovered;
                        cx.notify();
                    })
                })
                .on_click(move |_, window, cx| set_open(false, window, cx))
                .child(
                    Icon::carbon(CarbonIcon::Close)
                        .size(IconSize::S20)
                        .color(look.close_icon),
                )
        };

        // `.cds--modal-header`: padding 1rem 3rem 0 1rem, margin-bottom 0.5rem.
        let header = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .pt(rem(16.))
            .pl(rem(16.))
            .pr(rem(48.))
            .mb(rem(8.))
            .when_some(self.label.clone(), |this, label| {
                this.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(look.label)
                        .mb(rem(4.))
                        .child(label),
                )
            })
            .when_some(self.title.clone(), |this, title| {
                this.child(
                    div()
                        .type_style(TypeStyle::Heading03)
                        .text_color(look.text)
                        // `padding-inline-end: calc(20% - 3rem)`.
                        .pr(rem((width_px * 0.2 - 48.).max(0.)))
                        .child(title),
                )
            });

        // `.cds--modal-content`: body-01, padding 0.5rem 1rem 3rem 1rem.
        let text_padding = geometry.text_padding(width_px - 32.);
        let body = div()
            .id((id.clone(), "body"))
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pt(rem(8.))
            .pb(rem(48.))
            .px(rem(16.))
            .type_style(TypeStyle::Body01)
            .text_color(look.text)
            .flex()
            .flex_col()
            .when_some(self.description.clone(), |this, description| {
                // `.cu-dialog-desc { margin: 0 0 var(--spacing-05) }`.
                this.child(div().mb(rem(16.)).pr(rem(text_padding)).child(description))
            })
            .children(self.children);

        let footer_button = footer_button_size(density::current());
        let footer = (!passive).then(|| {
            let secondary = self.secondary_text.clone().map(|text| {
                let on_secondary = self.on_secondary.clone();
                let set_open = set_open.clone();
                div().w_1_2().child(
                    Button::new((id.clone(), "secondary"), text)
                        .variant(ButtonVariant::Secondary)
                        .size(footer_button)
                        .full_width(true)
                        .track_focus(&secondary_focus)
                        .on_click(move |_, window, cx| match &on_secondary {
                            Some(f) => f(window, cx),
                            None => set_open(false, window, cx),
                        }),
                )
            });
            let primary = self.primary_text.clone().map(|text| {
                let on_primary = self.on_primary.clone();
                div().w_1_2().child(
                    Button::new((id.clone(), "primary"), text)
                        .variant(if self.danger {
                            ButtonVariant::Danger
                        } else {
                            ButtonVariant::Primary
                        })
                        .size(footer_button)
                        .full_width(true)
                        .disabled(self.primary_disabled)
                        .track_focus(&primary_focus)
                        .on_click(move |_, window, cx| {
                            if let Some(f) = &on_primary {
                                f(window, cx)
                            }
                        }),
                )
            });
            // `.cds--modal-footer`: 4rem, flex-end; compact buttons sit at its top.
            div()
                .flex()
                .flex_row()
                .flex_shrink_0()
                .justify_end()
                .items_start()
                .h(rem(64.))
                .children(secondary)
                .children(primary)
        });

        let container = div()
            .id((id.clone(), "container"))
            .relative()
            .flex()
            .flex_col()
            .w(rem(width_px))
            .max_h(rem(max_height_px))
            .bg(look.container)
            .border_1()
            .border_color(look.border)
            .occlude()
            .child(header)
            .child(body)
            .children(footer)
            .child(close);

        let surface = div()
            .opacity(progress)
            .when(!geometry.centered, |this| this.absolute().top_0().left_0())
            .child(Translate::y(container, travel));

        let set_cancel = set_open.clone();
        let set_scrim = set_open.clone();
        let close_on_scrim = self.close_on_backdrop && !self.alert;
        // `.cds--modal` covers the page, so the page behind gets no hover:
        // the scrim occludes it. Occluding also hides Base's backdrop
        // listener, so the scrim press is handled here.
        let scrim = div()
            .id((id.clone(), "scrim"))
            .size_full()
            .bg(look.scrim)
            .opacity(progress)
            .occlude()
            .on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| {
                cx.stop_propagation();
                if open && close_on_scrim {
                    set_scrim(false, window, cx);
                }
            });
        // Base's dialog and alert-dialog hosts share every builder; the alert
        // host carries the `alertdialog` role.
        macro_rules! host {
            ($host:expr) => {
                $host
                    .open(true)
                    .focus_handle(focus)
                    // Enter belongs to the focused control (Carbon's
                    // `shouldSubmitOnEnter` is off by default).
                    .on_ok(|_, _, cx| {
                        cx.propagate();
                        false
                    })
                    .on_cancel(move |_, window, cx| {
                        if open {
                            set_cancel(false, window, cx);
                        }
                        false
                    })
                    .backdrop(scrim)
                    .popup(surface)
            };
        }
        let host = if self.alert {
            // `preventCloseOnClickOutside`: the alert host ignores scrim presses.
            host!(gpui_kit::base::AlertDialog::new(cx)).into_any_element()
        } else {
            host!(
                gpui_kit::base::Dialog::new(cx)
                    .when_some(self.title.clone(), |this, title| {
                        this.accessibility_label(title)
                    })
                    .close_on_backdrop_press(self.close_on_backdrop)
            )
            .into_any_element()
        };
        root.child(host).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn widths_follow_modal_container_media_queries() {
        // 1280px window = 80rem: the 66rem column.
        let sm = ModalGeometry::of(DialogSize::Sm, 80.);
        assert_eq!((sm.width, sm.max_height, sm.centered), (0.42, 0.72, true));
        assert_eq!(ModalGeometry::of(DialogSize::Sm, 90.).width, 0.36);
        assert_eq!(ModalGeometry::of(DialogSize::Sm, 50.).width, 0.60);
        assert_eq!(ModalGeometry::of(DialogSize::Xs, 70.).max_height, 0.48);
        assert_eq!(ModalGeometry::of(DialogSize::Md, 50.).max_height, 0.90);
        assert_eq!(ModalGeometry::of(DialogSize::Lg, 85.).width, 0.72);
        let narrow = ModalGeometry::of(DialogSize::Md, 40.);
        assert_eq!((narrow.width, narrow.centered), (1., false));
    }

    #[test]
    fn footer_buttons_are_32px_in_compact_and_64px_in_comfortable() {
        let compact = footer_button_size(Density::Compact);
        let comfortable = footer_button_size(Density::Comfortable);
        assert_eq!(compact.height(Density::Compact), 32.);
        assert_eq!(comfortable.height(Density::Comfortable), 64.);
    }

    #[test]
    fn paragraph_inset_follows_modal_content_p() {
        // `calc(20% - 2rem)` for md.
        assert_eq!(
            ModalGeometry::of(DialogSize::Md, 80.).text_padding(500.),
            68.
        );
        // xs: 0; sm: 20% from 66rem only.
        assert_eq!(
            ModalGeometry::of(DialogSize::Xs, 80.).text_padding(500.),
            0.
        );
        assert_eq!(
            ModalGeometry::of(DialogSize::Sm, 50.).text_padding(500.),
            0.
        );
        assert_eq!(
            ModalGeometry::of(DialogSize::Sm, 80.).text_padding(500.),
            100.
        );
    }

    #[test]
    fn motion_follows_modal_presence_keyframes() {
        assert_eq!(MODAL_ENTER.speed.millis(), 240);
        assert_eq!(MODAL_ENTER.curve.points(), (0., 0., 0.3, 1.));
        assert_eq!(MODAL_EXIT.speed.millis(), 240);
        assert_eq!(MODAL_EXIT.curve.points(), (0.4, 0.14, 1., 1.));
        assert_eq!(MODAL_TRAVEL, -24.);
        assert_eq!(CLOSE_HOVER.speed.millis(), 110);
    }

    #[test]
    fn look_follows_bundle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let look = ModalLook::current();
        assert_eq!(hex(look.container), "#f4f4f4");
        assert_eq!(hex(look.border), "#c6c6c6");
        assert_eq!(hex(look.label), "#525252");
        assert_eq!(hex(look.close_hover), "#e8e8e8");
        // `--overlay: rgba(0, 0, 0, 0.6)` on White.
        assert!((look.scrim.a - 0.6).abs() < 0.01);
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(hex(ModalLook::current().container), "#262626");
    }

    #[test]
    fn passive_without_primary_text() {
        assert!(Dialog::new("d").is_passive());
        assert!(!Dialog::new("d").primary_text("Save").is_passive());
        assert!(
            Dialog::new("d")
                .primary_text("Save")
                .passive(true)
                .is_passive()
        );
        let plain = Dialog::new("d");
        assert!(!plain.is_danger() && !plain.is_alert() && plain.closes_on_backdrop());
    }
}
