//! AlertDialog: an interrupting confirmation → Carbon `Modal` in `alert` mode,
//! `danger` by default, not dismissed by a press on the scrim
//! (`index.jsx`: `<Dialog alert danger primaryText={actionText}
//! secondaryText={cancelText} preventCloseOnClickOutside />`).
//!
//! Everything else (host, focus, motion, geometry) is [`Dialog`]'s. A danger
//! alert starts with focus on Cancel, so Enter never confirms by accident; its
//! primary button should repeat the verb of the title ("Delete bucket").
//!
//! ```ignore
//! AlertDialog::new("delete")
//!     .trigger(Button::new("t", "Delete bucket").variant(ButtonVariant::Destructive))
//!     .title("Delete logs-2026?")
//!     .description("This permanently deletes 12,408 objects.")
//!     .action_text("Delete bucket")
//!     .cancel_text("Cancel")
//! ```
use super::dialog::{Dialog, DialogSize};
use gpui_kit::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, SharedString,
    Window,
};
use std::rc::Rc;

type Action = Rc<dyn Fn(&mut Window, &mut App)>;

/// A Carbon alert modal. See the module docs.
#[derive(IntoElement)]
pub struct AlertDialog {
    dialog: Dialog,
    action_text: SharedString,
    cancel_text: SharedString,
    on_action: Option<Action>,
    danger: bool,
}

impl AlertDialog {
    /// An alert dialog with a stable `id`. Pass the visible texts with
    /// [`Self::action_text`] and [`Self::cancel_text`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            dialog: Dialog::new(id).alert(true).close_on_backdrop(false),
            action_text: SharedString::default(),
            cancel_text: SharedString::default(),
            on_action: None,
            danger: true,
        }
    }

    /// The primary (consequence) action's label; repeat the title's verb.
    pub fn action_text(mut self, text: impl Into<SharedString>) -> Self {
        self.action_text = text.into();
        self
    }

    /// The cancel button's label.
    pub fn cancel_text(mut self, text: impl Into<SharedString>) -> Self {
        self.cancel_text = text.into();
        self
    }

    /// Run the action (the dialog stays open unless the handler closes it).
    pub fn on_action(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }

    /// Danger styling (default true).
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Controlled open state.
    pub fn open(mut self, open: bool) -> Self {
        self.dialog = self.dialog.open(open);
        self
    }

    /// Initial open state when uncontrolled.
    pub fn default_open(mut self, open: bool) -> Self {
        self.dialog = self.dialog.default_open(open);
        self
    }

    /// Called whenever the dialog asks to open or close.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.dialog = self.dialog.on_open_change(f);
        self
    }

    /// An element that opens the dialog when pressed.
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.dialog = self.dialog.trigger(trigger);
        self
    }

    /// The heading.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.dialog = self.dialog.title(title);
        self
    }

    /// The small label above the heading.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.dialog = self.dialog.label(label);
        self
    }

    /// The consequence, as the body paragraph.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.dialog = self.dialog.description(description);
        self
    }

    /// Container size (default `sm`).
    pub fn size(mut self, size: DialogSize) -> Self {
        self.dialog = self.dialog.size(size);
        self
    }

    /// Accessible name of the close button.
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.dialog = self.dialog.close_label(label);
        self
    }

    /// Focus this element on open instead of Cancel.
    pub fn initial_focus(mut self, handle: &FocusHandle) -> Self {
        self.dialog = self.dialog.initial_focus(handle);
        self
    }

    /// The configured dialog.
    fn into_dialog(self) -> Dialog {
        let mut dialog = self
            .dialog
            .danger(self.danger)
            .primary_text(self.action_text)
            .secondary_text(self.cancel_text);
        if let Some(action) = self.on_action {
            dialog = dialog.on_primary(move |window, cx| action(window, cx));
        }
        dialog
    }
}

impl ParentElement for AlertDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.dialog.extend(elements);
    }
}

impl RenderOnce for AlertDialog {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.into_dialog()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_defaults_follow_index_jsx() {
        let alert = AlertDialog::new("a").action_text("Delete bucket");
        assert!(alert.danger);
        let dialog = alert.into_dialog();
        assert!(dialog.is_danger());
        assert!(dialog.is_alert());
        assert!(!dialog.closes_on_backdrop());
    }
}
