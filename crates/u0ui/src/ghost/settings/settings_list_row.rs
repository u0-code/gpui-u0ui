//! Ghost · Settings › [`SettingsListRow`] (`.g-slrow`): the list row the
//! settings pages share (projects, servers, hooks…): a leading icon or
//! element, title over subtitle, an optional status, and actions. An action
//! with an icon draws as a ghost icon button (its label the tooltip and
//! accessible name), one without as a ghost text button.
use super::row::{self, Handler, ghost_button, icon_button};
use crate::{icons::CarbonIcon, t3::press::icon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The leading icon: `React.createElement(icon, { size: 20, className: 'g-dim-icon' })`.
const LEADING_ICON: f32 = 20.;

/// One action at the row's end (`actions: {label, icon?, onClick}[]`).
#[derive(Clone)]
pub struct SettingsAction {
    /// The label: the button text, or the icon button's tooltip.
    pub label: SharedString,
    /// The icon; with one the action is an icon button.
    pub icon: Option<CarbonIcon>,
    on_click: Option<Handler>,
}

impl SettingsAction {
    /// A text action.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            on_click: None,
        }
    }

    /// An icon action (`{label, icon}`).
    pub fn icon(label: impl Into<SharedString>, glyph: CarbonIcon) -> Self {
        Self {
            icon: Some(glyph),
            ..Self::new(label)
        }
    }

    /// `onClick`.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// How the action draws (`a.icon ? Cb.IconButton : Button`).
    pub fn kind(&self) -> ActionKind {
        if self.icon.is_some() {
            ActionKind::IconButton
        } else {
            ActionKind::TextButton
        }
    }
}

/// How a [`SettingsAction`] draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    /// `Cb.IconButton kind="ghost" size="sm"`.
    IconButton,
    /// `Button variant="ghost" size="sm"`.
    TextButton,
}

/// The settings list row (`SettingsListRow`). See the module docs.
#[derive(IntoElement)]
pub struct SettingsListRow {
    id: ElementId,
    icon: Option<CarbonIcon>,
    leading: Option<AnyElement>,
    title: SharedString,
    subtitle: Option<SharedString>,
    status: Option<AnyElement>,
    actions: Vec<SettingsAction>,
    on_click: Option<Handler>,
}

impl SettingsListRow {
    /// A row titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            leading: None,
            title: title.into(),
            subtitle: None,
            status: None,
            actions: Vec::new(),
            on_click: None,
        }
    }

    /// The leading icon (`icon`), 20px in `icon-secondary`.
    pub fn icon(mut self, glyph: CarbonIcon) -> Self {
        self.icon = Some(glyph);
        self
    }

    /// A leading element in place of the icon (`leading`).
    pub fn leading(mut self, el: impl IntoElement) -> Self {
        self.leading = Some(el.into_any_element());
        self
    }

    /// The secondary line (`subtitle`).
    pub fn subtitle(mut self, s: impl Into<SharedString>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    /// A status element before the actions (`status`).
    pub fn status(mut self, el: impl IntoElement) -> Self {
        self.status = Some(el.into_any_element());
        self
    }

    /// Add an action.
    pub fn action(mut self, action: SettingsAction) -> Self {
        self.actions.push(action);
        self
    }

    /// The actions (`actions`).
    pub fn actions(mut self, actions: impl IntoIterator<Item = SettingsAction>) -> Self {
        self.actions.extend(actions);
        self
    }

    /// `onClick`: pressing the row.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SettingsListRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id;
        let leading = self.leading.or_else(|| {
            self.icon.map(|g| {
                // `.g-dim-icon { fill: var(--icon-secondary); flex: none }`.
                div()
                    .flex_none()
                    .child(icon(g, LEADING_ICON, color(ColorRole::IconSecondary)))
                    .into_any_element()
            })
        });
        let actions = (!self.actions.is_empty()).then(|| {
            // `.g-slrow-acts { display: flex; align-items: center; flex: none }`.
            div().flex().flex_none().items_center().children(
                self.actions.into_iter().enumerate().map(|(i, a)| {
                    let aid: ElementId =
                        (id.clone(), SharedString::from(format!("act-{i}"))).into();
                    match a.icon {
                        Some(g) => icon_button(aid, a.label, g, a.on_click).into_any_element(),
                        None => ghost_button(aid, a.label, None, a.on_click).into_any_element(),
                    }
                }),
            )
        });
        let on_click = self.on_click;
        row::frame()
            .id(id.clone())
            .when_some(on_click, |this, f| {
                this.cursor_pointer().on_click(move |_, w, cx| f(w, cx))
            })
            .when_some(leading, |this, l| this.child(l))
            .child(row::main(self.title, self.subtitle.map(row::subtitle)))
            .when_some(self.status, |this, s| this.child(s))
            .when_some(actions, |this, a| this.child(a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_with_an_icon_are_icon_buttons() {
        assert_eq!(
            SettingsAction::icon("Open", CarbonIcon::Launch).kind(),
            ActionKind::IconButton
        );
        assert_eq!(SettingsAction::new("Open").kind(), ActionKind::TextButton);
    }

    #[test]
    fn builders_keep_order() {
        let r = SettingsListRow::new("r", "carbon-ui")
            .subtitle("~/code/carbon-ui")
            .action(SettingsAction::icon("Open", CarbonIcon::Launch))
            .actions([SettingsAction::icon("Remove", CarbonIcon::TrashCan)]);
        let labels: Vec<_> = r.actions.iter().map(|a| a.label.as_ref()).collect();
        assert_eq!(labels, ["Open", "Remove"]);
        assert_eq!(r.subtitle.as_deref(), Some("~/code/carbon-ui"));
    }
}
