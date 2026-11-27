//! T3 · Pull requests & issues: the PR detail page parts —
//! [`PullRequestDetailHeader`] and [`MergeControls`].
use super::pull_request::{PullRequestState, PullRequestStateGlyph, start_bar};
use crate::{
    components::{Badge, BadgeSize, Button, ButtonSize, ButtonVariant, MenuItem},
    icons::{CarbonIcon, Icon},
    t3::craft::{self, focus_handle, focus_visible, mono, press, rem, sans},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Texts of a [`PullRequestDetailHeader`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct DetailHeaderTexts {
    /// `Back`.
    pub back: SharedString,
    /// `Edit title`.
    pub edit_title: SharedString,
    /// ` wants to merge `.
    pub wants_to_merge: SharedString,
    /// ` into `.
    pub into: SharedString,
}

impl Default for DetailHeaderTexts {
    fn default() -> Self {
        Self {
            back: "Back".into(),
            edit_title: "Edit title".into(),
            wants_to_merge: "wants to merge".into(),
            into: "into".into(),
        }
    }
}

/// `.t3-ref`: mono 12px on `layer-accent-01`, `1px 4px` padding. It sets no
/// colour, so it keeps its parent's (`.t3-prhead-meta`'s `text-secondary`).
pub(crate) fn code_ref(text: impl Into<SharedString>) -> gpui_kit::Div {
    mono(div(), 12., 400)
        .flex_none()
        .px(rem(4.))
        .py(px(1.))
        .bg(color(ColorRole::LayerAccent01))
        .child(text.into())
}

type TabHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The PR detail page header (`.t3-prhead`): Back + repo / `#number`
/// crumbs, state glyph + title + Edit, state tag + "author wants to merge
/// head into base" + primary actions, and the page tabs.
#[derive(IntoElement)]
pub struct PullRequestDetailHeader {
    id: ElementId,
    number: u32,
    title: SharedString,
    state: PullRequestState,
    repo: SharedString,
    author: SharedString,
    head: SharedString,
    base: SharedString,
    tabs: Vec<SharedString>,
    tab: Option<SharedString>,
    primary: Option<AnyElement>,
    top_action: Option<AnyElement>,
    show_crumbs: bool,
    title_large: bool,
    divided: bool,
    on_tab: Option<TabHandler>,
    on_back: Option<Handler>,
    on_edit: Option<Handler>,
    on_number: Option<Handler>,
    texts: DetailHeaderTexts,
}

impl PullRequestDetailHeader {
    /// A header for PR `number` titled `title`.
    pub fn new(id: impl Into<ElementId>, number: u32, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            number,
            title: title.into(),
            state: PullRequestState::Open,
            repo: SharedString::default(),
            author: SharedString::default(),
            head: SharedString::default(),
            base: SharedString::default(),
            tabs: ["Summary", "Code", "Timeline", "Guide"]
                .map(SharedString::from)
                .to_vec(),
            tab: None,
            primary: None,
            top_action: None,
            show_crumbs: true,
            title_large: false,
            divided: true,
            on_tab: None,
            on_back: None,
            on_edit: None,
            on_number: None,
            texts: DetailHeaderTexts::default(),
        }
    }

    /// The state.
    pub fn state(mut self, state: PullRequestState) -> Self {
        self.state = state;
        self
    }

    /// The repository.
    pub fn repo(mut self, repo: impl Into<SharedString>) -> Self {
        self.repo = repo.into();
        self
    }

    /// The author.
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = author.into();
        self
    }

    /// Head and base branches.
    pub fn branches(
        mut self,
        head: impl Into<SharedString>,
        base: impl Into<SharedString>,
    ) -> Self {
        self.head = head.into();
        self.base = base.into();
        self
    }

    /// The tabs (default Summary, Code, Timeline, Guide).
    pub fn tabs<S: Into<SharedString>>(mut self, tabs: impl IntoIterator<Item = S>) -> Self {
        self.tabs = tabs.into_iter().map(Into::into).collect();
        self
    }

    /// The selected tab (default the first).
    pub fn tab(mut self, tab: impl Into<SharedString>) -> Self {
        self.tab = Some(tab.into());
        self
    }

    /// Primary actions at the meta row's end.
    pub fn primary(mut self, primary: impl IntoElement) -> Self {
        self.primary = Some(primary.into_any_element());
        self
    }

    /// Action aligned at the top-right of the title row (`Check out in a worktree`).
    pub fn top_action(mut self, action: impl IntoElement) -> Self {
        self.top_action = Some(action.into_any_element());
        self
    }

    /// Show the built-in crumb row (`true` by default; `false` when the page bar shows `WorkspaceBreadcrumb`).
    pub fn show_crumbs(mut self, show: bool) -> Self {
        self.show_crumbs = show;
        self
    }

    /// Use the 28px/36px workspace heading scale (`false` by default).
    pub fn title_large(mut self, large: bool) -> Self {
        self.title_large = large;
        self
    }

    /// Show the bottom border (`true` by default; `false` when custom tabs follow).
    pub fn divided(mut self, divided: bool) -> Self {
        self.divided = divided;
        self
    }

    /// A tab was chosen (`onTab`).
    pub fn on_tab(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_tab = Some(Rc::new(f));
        self
    }

    /// Back pressed.
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }

    /// Edit title pressed.
    pub fn on_edit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }

    /// The `#number` link was activated.
    pub fn on_number(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_number = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: DetailHeaderTexts) -> Self {
        self.texts = texts;
        self
    }
}

pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    label: SharedString,
    icon: CarbonIcon,
    handler: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .icon(icon)
        .icon_only(true)
        .when_some(handler, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}

impl RenderOnce for PullRequestDetailHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let secondary = color(ColorRole::TextSecondary);
        let current = self
            .tab
            .clone()
            .or_else(|| self.tabs.first().cloned())
            .unwrap_or_default();
        let crumbs = sans(div(), 14., 400)
            .flex()
            .items_center()
            .gap(rem(4.))
            .text_color(secondary)
            .child(icon_button(
                (id.clone(), "back"),
                self.texts.back.clone(),
                CarbonIcon::ArrowLeft,
                self.on_back.clone(),
            ))
            .child(self.repo.clone())
            .child(div().text_color(color(ColorRole::TextHelper)).child("/"))
            .child({
                let f = self.on_number.clone();
                Button::new((id.clone(), "number"), format!("#{}", self.number))
                    .variant(ButtonVariant::Link)
                    .when_some(f, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            });
        let title_size = if self.title_large { 28. } else { 20. };
        let title_lh = if self.title_large { rem(36.) } else { rem(28.) };
        let title = div()
            .flex()
            .items_center()
            .gap(rem(8.))
            .child(PullRequestStateGlyph::new(self.state).size(20.))
            .child(
                sans(div(), title_size, 300)
                    .font_weight(gpui_kit::FontWeight(400.))
                    .line_height(title_lh)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.title.clone()),
            )
            .child(icon_button(
                (id.clone(), "edit"),
                self.texts.edit_title.clone(),
                CarbonIcon::Edit,
                self.on_edit.clone(),
            ))
            .when_some(self.top_action, |row, act| {
                row.child(div().flex_1()).child(act)
            });
        let meta = sans(div(), 14., 400)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(8.))
            .text_color(secondary)
            .child(
                Badge::new(self.state.label())
                    .variant(self.state.badge())
                    .size(BadgeSize::Sm),
            )
            // `.t3-prhead-meta b { color: text-primary }`, no weight: Carbon's
            // `b { font: inherit }` keeps it 400.
            .child(
                sans(div(), 14., 400)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.author.clone()),
            )
            .child(self.texts.wants_to_merge.clone())
            .child(code_ref(self.head.clone()))
            .child(self.texts.into.clone())
            .child(code_ref(self.base.clone()))
            .child(div().flex_1())
            .children(self.primary);
        let tabs = div()
            .flex()
            .children(self.tabs.iter().enumerate().map(|(i, t)| {
                let on = *t == current;
                let tab_id = craft::sub(&id, "tab", i);
                let focus = focus_handle(&tab_id, window, cx);
                let ring = focus_visible(&focus, window);
                let on_tab = self.on_tab.clone();
                let name = t.clone();
                // `.t3-prtab { height: 40px; padding: 0 16px; border-bottom: 2px }`.
                sans(press(tab_id, &focus), 14., if on { 600 } else { 400 })
                    .role(gpui_kit::Role::Tab)
                    .h(rem(40.))
                    .px(rem(16.))
                    .border_b_2()
                    .text_color(color(if on {
                        ColorRole::TextPrimary
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .border_color(if on {
                        color(ColorRole::BorderInteractive)
                    } else {
                        Hsla::transparent_black()
                    })
                    .when(!on, |this| {
                        this.hover(|s| {
                            s.text_color(color(ColorRole::TextPrimary))
                                .border_color(color(ColorRole::BorderStrong01))
                        })
                    })
                    .when_some(on_tab, |this, f| {
                        this.on_click(move |_, w, cx| f(name.clone(), w, cx))
                    })
                    .child(t.clone())
                    .when(ring, |this| this.child(craft::focus_ring()))
            }));
        let has_tabs = !self.tabs.is_empty();
        div()
            .id(id)
            .flex()
            .flex_col()
            .gap(rem(if self.title_large { 12. } else { 8. }))
            .when(self.divided, |d| {
                d.border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
            })
            .when(self.show_crumbs, |d| d.child(crumbs))
            .child(title)
            .child(meta)
            .when(has_tabs, |d| d.child(tabs))
    }
}

/// A merge method (`method`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MergeMethod {
    /// `squash`, the default.
    #[default]
    Squash,
    /// `merge`.
    Merge,
    /// `rebase`.
    Rebase,
}

impl MergeMethod {
    /// Every method, in menu order.
    pub const ALL: [MergeMethod; 3] = [Self::Squash, Self::Merge, Self::Rebase];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Squash => "squash",
            Self::Merge => "merge",
            Self::Rebase => "rebase",
        }
    }

    /// The primary button text (also the menu option).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Squash => "Squash and merge",
            Self::Merge => "Create merge commit",
            Self::Rebase => "Rebase and merge",
        }
    }
}

/// Texts of [`MergeControls`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct MergeTexts {
    /// `Ready to merge`.
    pub ready: SharedString,
    /// The menu trigger's name (`Merge method`).
    pub menu: SharedString,
    /// The radio group's name (`Method`).
    pub method: SharedString,
    /// `Enable auto-merge`.
    pub auto_merge: SharedString,
}

impl Default for MergeTexts {
    fn default() -> Self {
        Self {
            ready: "Ready to merge".into(),
            menu: "Merge method".into(),
            method: "Method".into(),
            auto_merge: "Enable auto-merge".into(),
        }
    }
}

type MethodHandler = Rc<dyn Fn(MergeMethod, &mut Window, &mut App)>;

/// The merge box (`.t3-merge` on `layer-01` with a 3px start bar): green
/// "Ready to merge" or the yellow blocking reason, then the merge button
/// (primary; secondary and disabled while blocked) and, 1px apart, the
/// method menu (Squash / Merge commit / Rebase, Enable auto-merge).
#[derive(IntoElement)]
pub struct MergeControls {
    id: ElementId,
    method: MergeMethod,
    blocked: Option<SharedString>,
    on_merge: Option<Handler>,
    on_method: Option<MethodHandler>,
    on_auto_merge: Option<Handler>,
    texts: MergeTexts,
}

impl MergeControls {
    /// Merge controls.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            method: MergeMethod::Squash,
            blocked: None,
            on_merge: None,
            on_method: None,
            on_auto_merge: None,
            texts: MergeTexts::default(),
        }
    }

    /// The merge method (default squash).
    pub fn method(mut self, method: MergeMethod) -> Self {
        self.method = method;
        self
    }

    /// Why merging is blocked (shows the warning and disables merging).
    pub fn blocked(mut self, reason: impl Into<SharedString>) -> Self {
        self.blocked = Some(reason.into());
        self
    }

    /// Merge pressed (`onMerge`).
    pub fn on_merge(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_merge = Some(Rc::new(f));
        self
    }

    /// A method was chosen in the menu.
    pub fn on_method(mut self, f: impl Fn(MergeMethod, &mut Window, &mut App) + 'static) -> Self {
        self.on_method = Some(Rc::new(f));
        self
    }

    /// Enable auto-merge chosen.
    pub fn on_auto_merge(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_auto_merge = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: MergeTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for MergeControls {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let blocked = self.blocked.is_some();
        let kind = if blocked {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Primary
        };
        let status_icon = if blocked {
            crate::t3::base::warning_glyph(16.).into_any_element()
        } else {
            Icon::carbon(CarbonIcon::CheckmarkFilled)
                .size_length(rem(16.))
                .color(color(ColorRole::SupportSuccess))
                .into_any_element()
        };
        let on_method = self.on_method.clone();
        let on_merge = self.on_merge.clone();
        let on_auto = self.on_auto_merge.clone();
        let mut radio = MenuItem::radio_group(
            self.texts.method.clone(),
            MergeMethod::ALL.map(|m| m.label()),
        )
        .default_selected_item(
            MergeMethod::ALL
                .iter()
                .position(|m| *m == self.method)
                .unwrap_or(0),
        );
        if let Some(f) = on_method {
            radio = radio.on_select_item(move |i, w, cx| f(MergeMethod::ALL[i], w, cx));
        }
        let mut auto = MenuItem::item(self.texts.auto_merge.clone());
        if let Some(f) = on_auto {
            auto = auto.on_select(move |w, cx| f(w, cx));
        }
        sans(div(), 14., 400)
            .id(id.clone())
            .relative()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(rem(12.))
            .py(rem(12.))
            .pr(rem(12.))
            .pl(rem(16.))
            .bg(color(ColorRole::Layer01))
            .child(start_bar(color(if blocked {
                ColorRole::SupportWarning
            } else {
                ColorRole::SupportSuccess
            })))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(div().flex_none().child(status_icon))
                    .child(
                        self.blocked
                            .clone()
                            .unwrap_or_else(|| self.texts.ready.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(1.))
                    .child(
                        Button::new((id.clone(), "merge"), self.method.label())
                            .variant(kind)
                            .size(ButtonSize::Md)
                            .icon(CarbonIcon::Merge)
                            .disabled(blocked)
                            .when_some(on_merge, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                    )
                    .child(
                        crate::t3::shell::IconMenu::new((id, "menu"), self.texts.menu.clone())
                            .variant(kind)
                            .size(ButtonSize::Md)
                            .disabled(blocked)
                            .item(radio)
                            .item(MenuItem::divider())
                            .item(auto),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_label_the_button() {
        assert_eq!(MergeMethod::Squash.label(), "Squash and merge");
        assert_eq!(MergeMethod::Rebase.name(), "rebase");
        assert_eq!(
            PullRequestState::Merged.badge(),
            crate::components::BadgeVariant::Purple
        );
    }
}
