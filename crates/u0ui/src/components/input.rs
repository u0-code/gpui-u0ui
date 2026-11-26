//! Input: shadcn `Input` → Carbon `TextInput` / `PasswordInput` (`.cds--text-input`).
//!
//! - **Behavior** is Kit's: [`gpui_kit::component::input::Input`] over the
//!   caller's [`InputState`] owns editing, IME, selection, clipboard, undo, the
//!   caret and the context menu. It renders without chrome (`appearance(false)`).
//! - **Appearance** is Carbon's: `field` fill, 1px `border-strong` bottom rule,
//!   2px inset `focus` outline, 2px `support-error` outline plus the error icon
//!   when invalid, the warning icon when `warn`, the label above and the helper
//!   or requirement text below ([`super::form`]). The placeholder is drawn here
//!   in `text-placeholder` (Kit would paint it `text-secondary`), so give the
//!   placeholder to [`Input::placeholder`], not to the state.
//! - **Motion**: `transition: background-color 70ms, outline 70ms`
//!   (productive standard), via [`FIELD_MOTION`].
//! - `type="password"` ([`Input::password`]) adds the visibility toggle
//!   (`.cds--text-input--password__visibility__toggle`), which flips the
//!   state's masking.
//!
//! ```ignore
//! let state = cx.new(|cx| InputState::new(window, cx));
//! Input::new("name", &state).label("Cluster name").placeholder("e.g. prod-us-east")
//! ```
use super::form::{self, FieldSize, Validation, rem};
use crate::{
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window,
    component::{Sizable as _, Size, input::Input as KitInput},
    div,
    prelude::FluentBuilder as _,
};

pub use gpui_kit::component::input::InputState;

/// `transition: background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9), outline 70ms …`
/// (`.cds--text-input`, `.cds--text-area`).
pub const FIELD_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);

/// The interaction state a field look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FieldState {
    /// The editor has keyboard focus.
    pub focused: bool,
    /// Disabled.
    pub disabled: bool,
    /// Read-only.
    pub readonly: bool,
}

/// The colors of a Carbon text field in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldLook {
    /// Fill (`background-color: var(--field)`; read-only `transparent`).
    pub background: Hsla,
    /// The 1px bottom rule (`border-block-end: 1px solid var(--border-strong)`).
    pub border_bottom: Hsla,
    /// Text color.
    pub text: Hsla,
    /// Placeholder color (`::placeholder`).
    pub placeholder: Hsla,
}

impl FieldLook {
    /// The look of a field at `level` in `state`.
    pub fn of(level: LayerLevel, state: FieldState) -> Self {
        let clear = Hsla::transparent_black();
        if state.disabled {
            // `.cds--text-input:disabled { background: var(--field); color: text-disabled }`.
            return Self {
                background: color(level.field()),
                border_bottom: color(level.border_strong()),
                text: color(ColorRole::TextDisabled),
                placeholder: color(ColorRole::TextDisabled),
            };
        }
        if state.readonly {
            // `.cds--text-input-wrapper--readonly .cds--text-input { background:
            // transparent; border-block-end-color: var(--border-subtle) }`.
            return Self {
                background: clear,
                border_bottom: color(level.border_subtle()),
                text: color(ColorRole::TextPrimary),
                placeholder: color(ColorRole::TextPlaceholder),
            };
        }
        Self {
            background: color(level.field()),
            border_bottom: color(level.border_strong()),
            text: color(ColorRole::TextPrimary),
            placeholder: color(ColorRole::TextPlaceholder),
        }
    }
}

/// `padding-inline-end` of the text input: 16px (`--layout-density-padding-inline`),
/// 40px with a status icon (`.cds--text-input--invalid { padding-inline-end: 2.5rem }`),
/// the field height for a password (`.cds--password-input { padding-inline-end:
/// var(--layout-size-height-local) }`), plus 24px when it also shows an icon.
pub fn padding_end(height: f32, password: bool, has_icon: bool) -> f32 {
    match (password, has_icon) {
        (false, false) => 16.,
        (false, true) => 40.,
        (true, false) => height,
        (true, true) => height + 24.,
    }
}

/// A Carbon text input. See the module docs.
#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    state: Entity<InputState>,
    label: Option<SharedString>,
    hide_label: bool,
    helper_text: Option<SharedString>,
    placeholder: Option<SharedString>,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    size: FieldSize,
    density: Option<Density>,
    disabled: bool,
    readonly: bool,
    password: bool,
    password_labels: Option<(SharedString, SharedString)>,
    level: LayerLevel,
}

impl Input {
    /// An input with a stable `id`, editing the caller's `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: None,
            hide_label: false,
            helper_text: None,
            placeholder: None,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            size: FieldSize::Md,
            density: None,
            disabled: false,
            readonly: false,
            password: false,
            password_labels: None,
            level: LayerLevel::One,
        }
    }

    /// The label (required by Carbon; hide it with [`Self::hide_label`]).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Keep the label for accessibility only (`hideLabel`).
    pub fn hide_label(mut self, hide: bool) -> Self {
        self.hide_label = hide;
        self
    }

    /// Helper text under the field (`helperText`).
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Placeholder, drawn in `text-placeholder` while the field is empty.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// `invalid`: error outline, icon and message.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// `invalidText`.
    pub fn invalid_text(mut self, text: impl Into<SharedString>) -> Self {
        self.invalid_text = Some(text.into());
        self
    }

    /// `warn`: warning icon and message.
    pub fn warn(mut self, warn: bool) -> Self {
        self.warn = warn;
        self
    }

    /// `warnText`.
    pub fn warn_text(mut self, text: impl Into<SharedString>) -> Self {
        self.warn_text = Some(text.into());
        self
    }

    /// `size` (default `md`).
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Override the app density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Disabled (`cursor: not-allowed`, `text-disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only (`readOnly`).
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// `type="password"` → Carbon `PasswordInput`: masked, with a visibility toggle.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Tooltips of the visibility toggle (`showPasswordLabel`, `hidePasswordLabel`).
    pub fn password_labels(
        mut self,
        show: impl Into<SharedString>,
        hide: impl Into<SharedString>,
    ) -> Self {
        self.password_labels = Some((show.into(), hide.into()));
        self
    }

    /// The layer the field sits on (`field-01/02/03`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let density = self.density.unwrap_or_else(density::current);
        let height = self.size.height(density);
        let state = self.state.clone();
        let focus = state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        let empty = state.read(cx).value().is_empty();
        // Carbon drops invalid/warn on disabled and read-only fields.
        let validation = if self.disabled || self.readonly {
            Validation::None
        } else {
            Validation::from_flags(
                self.invalid,
                self.invalid_text.clone(),
                self.warn,
                self.warn_text.clone(),
            )
        };

        // A password field starts masked; apply it once per state.
        if self.password {
            let applied = window.use_keyed_state((id.clone(), "masked-init"), cx, |_, _| {
                None::<gpui_kit::EntityId>
            });
            if *applied.read(cx) != Some(state.entity_id()) {
                applied.update(cx, |a, _| *a = Some(state.entity_id()));
                state.update(cx, |s, cx| s.set_masked(true, window, cx));
            }
        }
        let masked = state.read(cx).presentation().is_masked();

        let look = FieldLook::of(
            self.level,
            FieldState {
                focused,
                disabled: self.disabled,
                readonly: self.readonly,
            },
        );
        let outline_target = form::field_outline(focused && !self.disabled, &validation)
            .unwrap_or(Hsla::transparent_black());
        let background = motion::animate_color(
            (id.clone(), "bg"),
            look.background,
            FIELD_MOTION,
            window,
            cx,
        );
        let outline = motion::animate_color(
            (id.clone(), "outline"),
            outline_target,
            FIELD_MOTION,
            window,
            cx,
        );

        let pad_end = padding_end(height, self.password, validation.has_icon());
        let aria = self.label.clone().or(self.placeholder.clone());
        let editor = KitInput::new(&state)
            .appearance(false)
            .with_size(Size::XSmall)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .when_some(aria, |this, label| this.aria_label(label))
            .p_0()
            .h_full()
            .flex_1()
            .min_w_0()
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(look.text);

        let placeholder = self.placeholder.clone().filter(|_| empty).map(|text| {
            div()
                .absolute()
                .left(rem(16.))
                .right(rem(pad_end))
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_color(look.placeholder)
                .child(text)
        });

        let icon = form::status_icon(&validation).map(|icon| {
            // `.cds--text-input__invalid-icon { inset-inline-end: 1rem }`; in a
            // password field it sits left of the toggle (`inset-inline-end:
            // var(--layout-size-height-local)`).
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .right(rem(if self.password { height } else { 16. }))
                .flex()
                .items_center()
                .child(icon)
        });

        let toggle = self.password.then(|| {
            let state = state.clone();
            let labels = self.password_labels.clone();
            let toggle_focus = window
                .use_keyed_state((id.clone(), "toggle-focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            let toggle_focused = toggle_focus.is_focused(window);
            let mut button = gpui_kit::base::Button::new((id.clone(), "visibility"))
                .track_focus(&toggle_focus)
                .disabled(self.disabled)
                .absolute()
                .top_0()
                .right_0()
                .size(rem(height))
                .flex()
                .items_center()
                .justify_center()
                .cursor(if self.disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::PointingHand
                })
                .child(
                    Icon::carbon(if masked {
                        CarbonIcon::View
                    } else {
                        CarbonIcon::ViewOff
                    })
                    .size(IconSize::S16)
                    // `svg { fill: var(--icon-primary) }`; disabled `icon-disabled`.
                    .color(color(if self.disabled {
                        ColorRole::IconDisabled
                    } else {
                        ColorRole::IconPrimary
                    })),
                )
                // `:focus { outline: 2px solid var(--focus); outline-offset: -2px }`.
                .when(toggle_focused, |this| {
                    this.child(form::inset_outline(2., color(ColorRole::Focus)))
                })
                .on_click(move |_, window, cx| {
                    state.update(cx, |s, cx| {
                        let next = !s.presentation().is_masked();
                        s.set_masked(next, window, cx);
                    })
                });
            if let Some((show, hide)) = labels {
                let tip = if masked { show } else { hide };
                button = button
                    .accessibility_label(tip.clone())
                    .tooltip(move |window, cx| {
                        gpui_kit::component::tooltip::Tooltip::new(tip.clone()).build(window, cx)
                    });
            }
            button
        });

        let field = div()
            .id((id.clone(), "field"))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(rem(height))
            .pl(rem(16.))
            .pr(rem(pad_end))
            .bg(background)
            .pb(gpui_kit::px(form::FIELD_RULE))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(look.text)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::IBeam
            })
            .when(!self.disabled, |this| {
                let focus = focus.clone();
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    focus.focus(window, cx);
                })
            })
            .child(editor)
            .children(placeholder)
            .children(icon)
            .children(toggle)
            .child(form::bottom_rule(look.border_bottom))
            .child(form::inset_outline(2., outline));

        let label = self.label.clone().filter(|_| !self.hide_label);
        form::labeled(
            label.as_ref(),
            self.disabled,
            field,
            form::message(&validation, self.helper_text.as_ref(), self.disabled),
        )
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
    fn field_colors_follow_text_input_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = FieldLook::of(LayerLevel::One, FieldState::default());
        // `.cds--text-input { background-color: var(--field); border-block-end: 1px solid var(--border-strong) }`.
        assert_eq!(hex(rest.background), "#f4f4f4");
        assert_eq!(hex(rest.border_bottom), "#8d8d8d");
        assert_eq!(hex(rest.text), "#161616");
        // `::placeholder { color: var(--text-placeholder) }` is translucent.
        assert!(rest.placeholder.a < 1.);
        // On layer-01 the field is field-02 (white).
        let two = FieldLook::of(LayerLevel::Two, FieldState::default());
        assert_eq!(hex(two.background), "#ffffff");
        let readonly = FieldLook::of(
            LayerLevel::One,
            FieldState {
                readonly: true,
                ..Default::default()
            },
        );
        assert_eq!(readonly.background.a, 0.);
        assert_eq!(hex(readonly.border_bottom), "#c6c6c6");
        let disabled = FieldLook::of(
            LayerLevel::One,
            FieldState {
                disabled: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(disabled.background), "#f4f4f4");
        assert!(disabled.text.a < 1.);
    }

    #[test]
    fn end_padding_follows_css() {
        assert_eq!(padding_end(40., false, false), 16.);
        // `.cds--text-input--invalid { padding-inline-end: 2.5rem }`.
        assert_eq!(padding_end(40., false, true), 40.);
        // `.cds--password-input { padding-inline-end: var(--layout-size-height-local) }`.
        assert_eq!(padding_end(40., true, false), 40.);
        // `.cds--text-input--invalid.cds--password-input { calc(height + 1.5rem) }`.
        assert_eq!(padding_end(48., true, true), 72.);
    }
}
