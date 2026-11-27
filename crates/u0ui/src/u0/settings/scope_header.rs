//! ScopeHeader: the "Applying to" bar over a settings page (`ScopeHeader` in
//! `src/ghost/admin.jsx`, `.g-scope` in `src/ghost.css`): a ghost menu button
//! choosing all projects (global) or one project; with a project chosen, a
//! note says the values here override the global ones for that project only.
//!
//! Controlled: the caller keeps the scope (`None` = global) and updates it in
//! [`ScopeHeader::on_scope_change`].
//!
//! ```ignore
//! ScopeHeader::new("scope")
//!     .scope(Some("carbon-ui".into()))
//!     .projects(["carbon-ui", "api-gateway"])
//!     .on_scope_change(|scope, _, _| {})
//! ```
use crate::{
    components::{DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem},
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
    u0::rows as row,
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The bar's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ScopeHeaderTexts {
    /// `"Applying to"`.
    pub applying_to: SharedString,
    /// `"All projects (global)"`.
    pub global: SharedString,
    /// The project note: `"Values you change here override the global
    /// setting for {scope} only."`.
    pub note: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for ScopeHeaderTexts {
    fn default() -> Self {
        Self {
            applying_to: "Applying to".into(),
            global: "All projects (global)".into(),
            note: Rc::new(|s| {
                format!("Values you change here override the global setting for {s} only.").into()
            }),
        }
    }
}

type ScopeHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;

/// The "Applying to" bar. See the module docs.
#[derive(IntoElement)]
pub struct ScopeHeader {
    id: ElementId,
    scope: Option<SharedString>,
    projects: Vec<SharedString>,
    texts: ScopeHeaderTexts,
    on_scope_change: Option<ScopeHandler>,
}

impl ScopeHeader {
    /// A bar applying to all projects.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            scope: None,
            projects: Vec::new(),
            texts: ScopeHeaderTexts::default(),
            on_scope_change: None,
        }
    }

    /// The scope (`scope`): `None` for `'global'`, or a project name.
    pub fn scope(mut self, scope: Option<SharedString>) -> Self {
        self.scope = scope;
        self
    }

    /// The projects to choose from (`projects`).
    pub fn projects(mut self, projects: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.projects = projects.into_iter().map(Into::into).collect();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ScopeHeaderTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A scope was picked (`onScopeChange`; `None` = global).
    pub fn on_scope_change(
        mut self,
        f: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_scope_change = Some(Rc::new(f));
        self
    }

    /// The menu button's label.
    pub fn button_label(&self) -> SharedString {
        self.scope
            .clone()
            .unwrap_or_else(|| self.texts.global.clone())
    }
}

impl RenderOnce for ScopeHeader {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let pick = |scope: Option<SharedString>, f: Option<ScopeHandler>| {
            move |w: &mut Window, cx: &mut App| {
                if let Some(f) = &f {
                    f(scope.clone(), w, cx);
                }
            }
        };
        let mut items = vec![
            MenuItem::item(self.texts.global.clone())
                .on_select(pick(None, self.on_scope_change.clone())),
            MenuItem::divider(),
        ];
        items.extend(self.projects.iter().map(|p| {
            MenuItem::item(p.clone()).on_select(pick(Some(p.clone()), self.on_scope_change.clone()))
        }));
        let label = self.button_label();
        // `.g-scope { display: flex; align-items: center; flex-wrap: wrap; gap:
        // var(--spacing-03); padding: var(--spacing-03) var(--spacing-04);
        // background: var(--layer-01); border-bottom: 1px solid
        // var(--border-subtle-01); font-size: 0.75rem }`.
        sans(div(), 12., 400)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_03)
            .py(tokens::SPACING_03)
            .px(tokens::SPACING_04)
            .bg(color(ColorRole::Layer01))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextPrimary))
            .child(row::dim(self.texts.applying_to.clone(), 12.))
            .child(
                DropdownMenu::new((self.id.clone(), "menu"))
                    .label(label)
                    .kind(DropdownMenuKind::Ghost)
                    .size(DropdownMenuSize::Sm)
                    .items(items),
            )
            .when_some(self.scope.clone(), |this, scope| {
                // `.g-scope-note { flex-basis: 100%; display: flex; align-items:
                // center; gap: var(--spacing-02); color: var(--text-secondary) }`,
                // `.g-scope-note svg { fill: var(--support-info) }`.
                this.child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap(tokens::SPACING_02)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(
                            Icon::carbon(CarbonIcon::InformationFilled)
                                .size_length(rem(14.))
                                .color(color(ColorRole::SupportInfo)),
                        )
                        .child((self.texts.note)(&scope)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_and_note_follow_the_scope() {
        let h = ScopeHeader::new("s");
        assert_eq!(
            h.button_label(),
            SharedString::from("All projects (global)")
        );
        let h = ScopeHeader::new("s").scope(Some("carbon-ui".into()));
        assert_eq!(h.button_label(), SharedString::from("carbon-ui"));
        assert_eq!(
            (h.texts.note)("carbon-ui"),
            SharedString::from(
                "Values you change here override the global setting for carbon-ui only."
            )
        );
    }
}
