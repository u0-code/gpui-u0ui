//! Textarea: shadcn `Textarea` → Carbon `TextArea` (`.cds--text-area`).
//!
//! - **Behavior** is Kit's [`gpui_kit::component::input::Textarea`] over the
//!   caller's [`TextareaState`] (multi-line editing, IME, selection, clipboard,
//!   undo, scrolling), rendered without chrome.
//! - **Appearance**: the Carbon field ([`super::input::FieldLook`]) with
//!   `body-01` text (14/20), `padding-block: 0.6875rem 0.5rem`, 16px sides,
//!   `min-block-size: 2.5rem`; focus/invalid outlines and the error/warn icon
//!   at the top right (`inset-block-start: 0.75rem`); disabled drops the
//!   bottom rule (`border-block-end: 1px solid transparent`). With
//!   `enableCounter` the label row shows `count/maxCount` at its end
//!   (`.cds--text-area__label-wrapper { justify-content: space-between }`).
//!   The placeholder is drawn here in `text-placeholder`.
//! - **Motion**: background and outline 70ms ([`super::input::FIELD_MOTION`]).
//!
//! Deviation: Carbon's `resize: vertical` handle is not drawn; the height
//! follows `rows`.
//!
//! ```ignore
//! let state = cx.new(|cx| TextareaState::new(window, cx));
//! Textarea::new("desc", &state).label("Description").enable_counter(true).max_count(200)
//! ```
use super::form::{self, Validation, rem};
use super::input::{FIELD_MOTION, FieldLook, FieldState};
use crate::{motion, theme::LayerLevel, tokens::TypeStyle, typography::TypeStyled as _};
use gpui_kit::{
    App, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window,
    component::{Sizable as _, Size, input::Textarea as KitTextarea},
    div,
    prelude::FluentBuilder as _,
};

pub use gpui_kit::component::input::TextareaState;

/// `.cds--text-area { padding-block: 0.6875rem 0.5rem }`.
pub const PADDING_TOP: f32 = 11.;
/// See [`PADDING_TOP`].
pub const PADDING_BOTTOM: f32 = 8.;
/// `body-01` line height (14px × 1.42857).
pub const LINE: f32 = 20.;
/// Kit insets multi-line text by its XSmall input padding (2px block, 4px
/// inline); the field padding subtracts it.
const KIT_INSET: (f32, f32) = (2., 4.);

/// Field height for `rows` lines: rows × 20 + 11 + 8 (+1 bottom rule),
/// at least `min-block-size: 2.5rem`.
pub fn field_height(rows: usize) -> f32 {
    (rows as f32 * LINE + PADDING_TOP + PADDING_BOTTOM + 1.).max(40.)
}

/// The counter text (`3/200`), or `None` without `enableCounter`/`maxCount`.
pub fn counter(enabled: bool, count: usize, max: Option<usize>) -> Option<String> {
    match (enabled, max) {
        (true, Some(max)) => Some(format!("{count}/{max}")),
        _ => None,
    }
}

/// A Carbon text area. See the module docs.
#[derive(IntoElement)]
pub struct Textarea {
    id: ElementId,
    state: Entity<TextareaState>,
    label: Option<SharedString>,
    hide_label: bool,
    helper_text: Option<SharedString>,
    placeholder: Option<SharedString>,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    rows: usize,
    enable_counter: bool,
    max_count: Option<usize>,
    disabled: bool,
    readonly: bool,
    level: LayerLevel,
}

impl Textarea {
    /// A text area with a stable `id`, editing the caller's `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextareaState>) -> Self {
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
            rows: 4,
            enable_counter: false,
            max_count: None,
            disabled: false,
            readonly: false,
            level: LayerLevel::One,
        }
    }

    /// `label`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Keep the label for accessibility only.
    pub fn hide_label(mut self, hide: bool) -> Self {
        self.hide_label = hide;
        self
    }

    /// `helperText`.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Placeholder in `text-placeholder`.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// `invalid`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// `invalidText`.
    pub fn invalid_text(mut self, text: impl Into<SharedString>) -> Self {
        self.invalid_text = Some(text.into());
        self
    }

    /// `warn`.
    pub fn warn(mut self, warn: bool) -> Self {
        self.warn = warn;
        self
    }

    /// `warnText`.
    pub fn warn_text(mut self, text: impl Into<SharedString>) -> Self {
        self.warn_text = Some(text.into());
        self
    }

    /// `rows` (default 4, Carbon's default).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// `enableCounter`.
    pub fn enable_counter(mut self, enable: bool) -> Self {
        self.enable_counter = enable;
        self
    }

    /// `maxCount`.
    pub fn max_count(mut self, max: usize) -> Self {
        self.max_count = Some(max);
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// The layer the field sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for Textarea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = self.state.clone();
        let focus = state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        let value = state.read(cx).value();
        let count = value.chars().count();
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
        let mut look = FieldLook::of(
            self.level,
            FieldState {
                focused,
                disabled: self.disabled,
                readonly: self.readonly,
            },
        );
        if self.disabled {
            // `.cds--text-area:disabled { border-block-end: 1px solid transparent }`.
            look.border_bottom = Hsla::transparent_black();
        }
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
        let pad_end = if validation.has_icon() { 40. } else { 16. };
        let editor_height = self.rows as f32 * LINE;

        let editor = KitTextarea::new(&state)
            .appearance(false)
            .with_size(Size::XSmall)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .when_some(self.label.clone(), |this, label| this.aria_label(label))
            .h(rem(editor_height + 2. * KIT_INSET.0))
            .w_full()
            .text_size(rem(14.))
            .line_height(rem(LINE))
            .text_color(look.text);

        let placeholder = self
            .placeholder
            .clone()
            .filter(|_| value.is_empty())
            .map(|text| {
                div()
                    .absolute()
                    .top(rem(PADDING_TOP))
                    .left(rem(16.))
                    .right(rem(pad_end))
                    .type_style(TypeStyle::Body01)
                    .text_color(look.placeholder)
                    .child(text)
            });
        let icon = form::status_icon(&validation)
            .map(|icon| div().absolute().top(rem(12.)).right(rem(16.)).child(icon));

        let field = div()
            .id((id.clone(), "field"))
            .relative()
            .w_full()
            .min_h(rem(40.))
            .pt(rem(PADDING_TOP - KIT_INSET.0))
            .pb(rem(PADDING_BOTTOM - KIT_INSET.0))
            .pl(rem(16. - KIT_INSET.1))
            .pr(rem(pad_end - KIT_INSET.1))
            .bg(background)
            .border_b_1()
            .border_color(look.border_bottom)
            .type_style(TypeStyle::Body01)
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
            .child(form::inset_outline(2., outline));

        let counter_text = counter(self.enable_counter, count, self.max_count);
        let label = self
            .label
            .clone()
            .filter(|l| !self.hide_label && !l.is_empty());
        let label_row = (label.is_some() || counter_text.is_some()).then(|| {
            div()
                .flex()
                .flex_row()
                .justify_between()
                .w_full()
                .child(form::label(
                    label.clone().unwrap_or_default(),
                    self.disabled,
                ))
                .children(counter_text.map(|text| {
                    form::label(text, self.disabled).text_color(form::label_color(self.disabled))
                }))
        });
        form::form_item()
            .children(label_row)
            .child(field)
            .children(form::message(
                &validation,
                self.helper_text.as_ref(),
                self.disabled,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_follows_rows_and_min_block_size() {
        // 4 rows × 20 + 11 + 8 + 1 rule.
        assert_eq!(field_height(4), 100.);
        // `min-block-size: 2.5rem`.
        assert_eq!(field_height(1), 40.);
    }

    #[test]
    fn counter_needs_both_props() {
        assert_eq!(counter(true, 3, Some(200)).as_deref(), Some("3/200"));
        assert_eq!(counter(true, 3, None), None);
        assert_eq!(counter(false, 3, Some(200)), None);
    }
}
