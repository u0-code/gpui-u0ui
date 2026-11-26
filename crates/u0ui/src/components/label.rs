//! Label: shadcn `Label` → Carbon `FormLabel` (`.cds--label`).
//!
//! `label-01` in `text-secondary` with 8px below. Carbon inputs draw their own
//! label; use this one for custom controls. `htmlFor` becomes
//! [`Label::target`]: clicking the label focuses that control.
//!
//! ```ignore
//! Label::new("Region").target(&region_focus)
//! ```
use super::form;
use gpui_kit::{
    App, CursorStyle, FocusHandle, InteractiveElement, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, prelude::FluentBuilder as _,
};

/// A Carbon form label. See the module docs.
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    disabled: bool,
    no_margin: bool,
    target: Option<FocusHandle>,
}

impl Label {
    /// A label with its text.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            disabled: false,
            no_margin: false,
            target: None,
        }
    }

    /// `text-disabled` (`.cds--label--disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Drop the 8px bottom margin (`.cds--label--no-margin`).
    pub fn no_margin(mut self, no_margin: bool) -> Self {
        self.no_margin = no_margin;
        self
    }

    /// `htmlFor`: the control a click on the label focuses.
    pub fn target(mut self, focus: &FocusHandle) -> Self {
        self.target = Some(focus.clone());
        self
    }
}

impl RenderOnce for Label {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let label = form::label(self.text.clone(), self.disabled)
            .when(self.no_margin, |this| this.mb_0())
            // `text-align: start` under `dir="rtl"`.
            .when(super::direction::is_rtl(), |this| this.text_right());
        match self.target.filter(|_| !self.disabled) {
            Some(target) => label
                .id(gpui_kit::ElementId::Name(self.text))
                .cursor(CursorStyle::Arrow)
                .on_click(move |_, window, cx| target.focus(window, cx))
                .into_any_element(),
            None => label.into_any_element(),
        }
    }
}
