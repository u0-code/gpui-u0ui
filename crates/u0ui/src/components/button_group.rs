//! ButtonGroup: shadcn `ButtonGroup` → Carbon `ButtonSet` (`.cds--btn-set`).
//!
//! Buttons sit flush, each filling an equal share up to 12.25rem
//! (`.cds--btn-set .cds--btn { inline-size: 100%; max-inline-size: 12.25rem }`),
//! split by a 1px `button-separator` line (`box-shadow: -1px 0 0 0
//! var(--button-separator)`, on top in `--stacked`). `vertical` is Carbon's
//! `stacked`. [`ButtonGroupSeparator`] and [`ButtonGroupText`] are the reference
//! `cu-btn-group-sep` / `cu-btn-group-text` parts. Secondary actions go left,
//! the primary action right.
//!
//! Behavior is each [`Button`]'s (Kit's `base::Button`); the group only lays
//! them out.
//!
//! ```ignore
//! ButtonGroup::new()
//!     .button(Button::new("cancel", "Cancel").variant(ButtonVariant::Secondary))
//!     .button(Button::new("save", "Save changes"))
//! ```
use super::button::Button;
use super::form::rem;
use crate::{
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::FluentBuilder as _, px,
};

/// `index.d.ts` `ButtonGroupProps.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonGroupOrientation {
    /// Side by side (`.cds--btn-set`).
    #[default]
    Horizontal,
    /// Stacked (`.cds--btn-set--stacked`).
    Vertical,
}

impl ButtonGroupOrientation {
    /// Both orientations.
    pub const ALL: [Self; 2] = [Self::Horizontal, Self::Vertical];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// `max-inline-size: 12.25rem` of a button in a set.
pub const BUTTON_MAX_WIDTH: f32 = 196.;

/// The separator before a button: `button-separator`, or for a disabled
/// button `icon-on-color-disabled` side by side / `layer-selected-disabled`
/// stacked (`.cds--btn-set .cds--btn.cds--btn--disabled`). The first button
/// has none, and it hides next to a focused button so the focus ring shows
/// whole (`.cds--btn-set .cds--btn:not(:focus)`,
/// `.cds--btn-set .cds--btn:focus + .cds--btn { box-shadow: inherit }`).
pub fn separator_color(
    orientation: ButtonGroupOrientation,
    first: bool,
    disabled: bool,
    focused: bool,
    previous_focused: bool,
) -> Option<Hsla> {
    if first || focused || previous_focused {
        return None;
    }
    Some(color(match (disabled, orientation) {
        (false, _) => ColorRole::ButtonSeparator,
        (true, ButtonGroupOrientation::Horizontal) => ColorRole::IconOnColorDisabled,
        (true, ButtonGroupOrientation::Vertical) => ColorRole::LayerSelectedDisabled,
    }))
}

enum Item {
    Button(Box<Button>),
    Other(AnyElement),
}

/// A Carbon button set. See the module docs.
#[derive(IntoElement, Default)]
pub struct ButtonGroup {
    orientation: ButtonGroupOrientation,
    items: Vec<Item>,
}

impl ButtonGroup {
    /// An empty horizontal group.
    pub fn new() -> Self {
        Self::default()
    }

    /// `orientation` (`vertical` = Carbon `stacked`).
    pub fn orientation(mut self, orientation: ButtonGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Append a button; it fills its share of the set.
    pub fn button(mut self, button: Button) -> Self {
        self.items
            .push(Item::Button(Box::new(button.full_width(true))));
        self
    }

    /// Append any other part ([`ButtonGroupSeparator`], [`ButtonGroupText`], …)
    /// at its own size.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.items.push(Item::Other(child.into_any_element()));
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let orientation = self.orientation;
        let vertical = orientation == ButtonGroupOrientation::Vertical;
        let mut seen_button = false;
        let mut previous_focused = false;
        // The group hands each button the handle it reads, so both agree on
        // focus whatever element-id scope the button renders in.
        let mut focused = Vec::with_capacity(self.items.len());
        let items: Vec<Item> = self
            .items
            .into_iter()
            .map(|item| match item {
                Item::Button(button) => {
                    let handle = button.focus_handle(window, cx);
                    focused.push(handle.is_focused(window));
                    Item::Button(Box::new(button.track_focus(&handle)))
                }
                other => {
                    focused.push(false);
                    other
                }
            })
            .collect();
        div()
            .flex()
            .when(vertical, |this| this.flex_col())
            .when(!vertical, |this| this.flex_row())
            .children(
                items
                    .into_iter()
                    .zip(focused)
                    .map(|(item, focused)| match item {
                        Item::Button(button) => {
                            let line = separator_color(
                                orientation,
                                !seen_button,
                                button.is_disabled(),
                                focused,
                                previous_focused,
                            );
                            seen_button = true;
                            previous_focused = focused;
                            div()
                                .relative()
                                .min_w_0()
                                // `.cds--btn-set .cds--btn { inline-size: 100%; max-inline-size:
                                // 12.25rem }` holds stacked too.
                                .max_w(rem(BUTTON_MAX_WIDTH))
                                .when(!vertical, |this| this.flex_1())
                                .when(vertical, |this| this.w_full())
                                .child(*button)
                                .when_some(line, |this, line| {
                                    this.child(div().absolute().bg(line).map(|this| {
                                        if vertical {
                                            this.top(px(-1.)).left_0().right_0().h(px(1.))
                                        } else {
                                            this.left(px(-1.)).top_0().bottom_0().w(px(1.))
                                        }
                                    }))
                                })
                                .into_any_element()
                        }
                        Item::Other(element) => {
                            previous_focused = false;
                            element
                        }
                    }),
            )
    }
}

/// `ButtonGroupSeparator` (`.cu-btn-group-sep`): a 1px `button-separator`
/// rule stretched across the group.
#[derive(IntoElement, Default)]
pub struct ButtonGroupSeparator;

impl ButtonGroupSeparator {
    /// A separator.
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for ButtonGroupSeparator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex_shrink_0()
            .self_stretch()
            .min_w(px(1.))
            .min_h(px(1.))
            .bg(color(ColorRole::ButtonSeparator))
    }
}

/// `ButtonGroupText` (`.cu-btn-group-text`): inline text on `layer-01` in
/// `text-secondary`, 16px side padding.
#[derive(IntoElement)]
pub struct ButtonGroupText {
    text: SharedString,
}

impl ButtonGroupText {
    /// A text part.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for ButtonGroupText {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .px(tokens::SPACING_05)
            .bg(color(ColorRole::Layer01))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextSecondary))
            .whitespace_nowrap()
            .child(self.text)
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
    fn separators_follow_btn_set_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let h = ButtonGroupOrientation::Horizontal;
        // `.cds--btn-set .cds--btn:first-of-type:not(:focus) { box-shadow: inherit }`.
        assert_eq!(separator_color(h, true, false, false, false), None);
        // `box-shadow: -0.0625rem 0 0 0 var(--button-separator, #e0e0e0)`.
        assert_eq!(
            separator_color(h, false, false, false, false)
                .map(hex)
                .as_deref(),
            Some("#e0e0e0")
        );
        // Hidden on a focused button and the one after it.
        assert_eq!(separator_color(h, false, false, true, false), None);
        assert_eq!(separator_color(h, false, false, false, true), None);
        assert_eq!(
            separator_color(ButtonGroupOrientation::Vertical, false, false, false, true),
            None
        );
        // Disabled: `var(--icon-on-color-disabled, #8d8d8d)`.
        assert_eq!(
            separator_color(h, false, true, false, false)
                .map(hex)
                .as_deref(),
            Some("#8d8d8d")
        );
        assert_eq!(BUTTON_MAX_WIDTH, 12.25 * 16.);
    }
}
