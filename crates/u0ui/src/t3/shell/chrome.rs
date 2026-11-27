//! T3 page chrome (`shell.jsx` "Page chrome"): [`WorkspaceBreadcrumb`],
//! [`WorkspacePageHeader`], [`RightPanelTabs`] and [`PanelLayoutControls`].
use super::super::craft::{press, sans};
use super::super::press::{Handler, focus, hover, icon, rem, ring, track_hover};
use super::IconMenu;
use crate::{
    components::{
        button::{Button, ButtonSize, ButtonVariant},
        menu::MenuItem,
        tooltip::Tooltip,
    },
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::{ColorRole, SIZE_MD},
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

// ── WorkspaceBreadcrumb ─────────────────────────────────────────────────

/// One breadcrumb (`{ label, href, icon }`): with `on_click` (the `href`)
/// it is a link unless it is the last item.
pub struct Crumb {
    label: SharedString,
    icon: Option<Icon>,
    on_click: Option<Handler>,
}

impl Crumb {
    /// A crumb labelled `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            on_click: None,
        }
    }

    /// A leading icon (`icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Make it a link (`href`).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// The workspace breadcrumb (`WorkspaceBreadcrumb`, `.t3-crumbs`): 14px
/// items 4px apart, wrapping, split by a `text-helper` "/"; earlier items
/// with a target are `link-primary` links (underlined on hover), the rest
/// `text-primary`.
#[derive(IntoElement)]
pub struct WorkspaceBreadcrumb {
    id: ElementId,
    items: Vec<Crumb>,
}

impl WorkspaceBreadcrumb {
    /// A breadcrumb of `items`.
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = Crumb>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
        }
    }
}

impl RenderOnce for WorkspaceBreadcrumb {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let last = self.items.len().saturating_sub(1);
        let mut children: Vec<AnyElement> = Vec::new();
        for (i, crumb) in self.items.into_iter().enumerate() {
            if i > 0 {
                // `.t3-crumbs-sep { color: var(--text-helper) }`.
                children.push(
                    div()
                        .text_color(color(ColorRole::TextHelper))
                        .child("/")
                        .into_any_element(),
                );
            }
            let link = crumb.on_click.filter(|_| i < last);
            let item_id: ElementId = (id.clone(), SharedString::from(format!("crumb-{i}"))).into();
            match link {
                Some(f) => {
                    // `.t3-crumbs-link { gap: var(--spacing-02); color: var(--link-primary) }`,
                    // `:hover { text-decoration: underline }`.
                    let (hovered, store) = hover((item_id.clone(), "hover"), window, cx);
                    let (handle, focused) = focus((item_id.clone(), "focus"), window, cx);
                    let ink = color(ColorRole::LinkPrimary);
                    children.push(
                        press(item_id.clone(), &handle)
                            .flex()
                            .items_center()
                            .gap(rem(4.))
                            .text_color(ink)
                            .on_hover(track_hover(store))
                            .on_click(move |_, w, cx| f(w, cx))
                            .children(crumb.icon.map(|i| i.size_length(rem(16.)).color(ink)))
                            .child(div().when(hovered, |d| d.underline()).child(crumb.label))
                            .when(focused, |b| b.child(ring()))
                            .into_any_element(),
                    );
                }
                None => {
                    // `.t3-crumbs-cur { gap: var(--spacing-02); color: var(--text-primary) }`.
                    let ink = color(ColorRole::TextPrimary);
                    children.push(
                        div()
                            .flex()
                            .items_center()
                            .gap(rem(4.))
                            .text_color(ink)
                            .children(crumb.icon.map(|i| i.size_length(rem(16.)).color(ink)))
                            .child(crumb.label)
                            .into_any_element(),
                    );
                }
            }
        }
        // `.t3-crumbs { flex-wrap: wrap; gap: var(--spacing-02); font-size: 0.875rem }`.
        sans(div(), 14., 400)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(4.))
            .children(children)
    }
}

// ── WorkspacePageHeader ─────────────────────────────────────────────────

/// A workspace page header (`WorkspacePageHeader`, `.t3-pagehead`): the
/// optional breadcrumb, then a row with the 28/36 title and its
/// `text-secondary` description on the left and the actions (1px apart)
/// bottom-aligned on the right; 16px block padding over a
/// `border-subtle-01` bottom rule.
#[derive(IntoElement)]
pub struct WorkspacePageHeader {
    id: ElementId,
    crumbs: Option<Vec<Crumb>>,
    title: SharedString,
    description: Option<SharedString>,
    actions: Vec<AnyElement>,
}

impl WorkspacePageHeader {
    /// A header titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            crumbs: None,
            title: title.into(),
            description: None,
            actions: Vec::new(),
        }
    }

    /// The breadcrumb (`crumbs`).
    pub fn crumbs(mut self, crumbs: impl IntoIterator<Item = Crumb>) -> Self {
        self.crumbs = Some(crumbs.into_iter().collect());
        self
    }

    /// The description (`description`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add an action (`actions`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for WorkspacePageHeader {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.t3-pagehead-title { font-size: 1.75rem; line-height: 1.28572; font-weight: 400 }`.
        let title = sans(div(), 28., 400)
            .text_color(color(ColorRole::TextPrimary))
            .child(self.title);
        // `.t3-pagehead-desc { margin: var(--spacing-02) 0 0; font-size: 0.875rem;
        // color: var(--text-secondary) }`.
        let description = self.description.map(|d| {
            sans(div(), 14., 400)
                .mt(rem(4.))
                .text_color(color(ColorRole::TextSecondary))
                .child(d)
        });
        div()
            .flex()
            .flex_col()
            .gap(rem(8.))
            .py(rem(16.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .children(
                self.crumbs
                    .map(|c| WorkspaceBreadcrumb::new((self.id.clone(), "crumbs"), c)),
            )
            .child(
                // `.t3-pagehead-row { align-items: flex-end; justify-content:
                // space-between; gap: var(--spacing-05) }`.
                div()
                    .flex()
                    .items_end()
                    .justify_between()
                    .gap(rem(16.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(title)
                            .children(description),
                    )
                    .child(div().flex().flex_none().gap(px(1.)).children(self.actions)),
            )
    }
}

// ── RightPanelTabs ──────────────────────────────────────────────────────

/// The kind of a right-panel surface (`kind`), which picks its glyph
/// (`SURFACE_ICON`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SurfaceKind {
    /// `browser`: Earth.
    Browser,
    /// `terminal`.
    Terminal,
    /// `files`: Folder.
    Files,
    /// `diff`: Code.
    Diff,
    /// `pr`: PullRequest.
    Pr,
    /// `linked`: Link.
    Linked,
    /// `device`: Mobile.
    Device,
    /// `chat`.
    Chat,
    /// `details`: Information.
    Details,
    /// Anything else: Document.
    #[default]
    Other,
}

impl SurfaceKind {
    /// The glyph.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Browser => CarbonIcon::Earth,
            Self::Terminal => CarbonIcon::Terminal,
            Self::Files => CarbonIcon::Folder,
            Self::Diff => CarbonIcon::Code,
            Self::Pr => CarbonIcon::PullRequest,
            Self::Linked => CarbonIcon::Link,
            Self::Device => CarbonIcon::Mobile,
            Self::Chat => CarbonIcon::Chat,
            Self::Details => CarbonIcon::Information,
            Self::Other => CarbonIcon::Document,
        }
    }
}

/// One right-panel tab (`{ id, label, kind, icon, closable, disabledReason }`).
#[derive(Clone)]
pub struct SurfaceTab {
    /// Stable id.
    pub id: SharedString,
    /// Label.
    pub label: SharedString,
    /// Kind (glyph).
    pub kind: SurfaceKind,
    /// Optional explicit Carbon icon override.
    pub icon: Option<CarbonIcon>,
    /// Shows a close button.
    pub closable: bool,
    /// Why it cannot be opened (disables it; shown as a tooltip).
    pub disabled_reason: Option<SharedString>,
}

impl SurfaceTab {
    /// A tab.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        kind: SurfaceKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            icon: None,
            closable: false,
            disabled_reason: None,
        }
    }

    /// Override the Carbon icon for this tab.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Resolved glyph for this tab.
    pub const fn resolved_icon(&self) -> CarbonIcon {
        match self.icon {
            Some(i) => i,
            None => self.kind.icon(),
        }
    }

    /// Show a close button (`closable`).
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Disable it, explaining why (`disabledReason`).
    pub fn disabled_reason(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// Which tabs show and which fold into the overflow menu (`maxVisible`):
/// the first `max`, except that an active tab past them swaps with the last
/// shown one, which then leads the hidden list. Returns tab indices.
pub fn split_tabs(
    len: usize,
    active: Option<usize>,
    max: Option<usize>,
) -> (Vec<usize>, Vec<usize>) {
    let all: Vec<usize> = (0..len).collect();
    let Some(max) = max.filter(|m| *m > 0 && len > *m) else {
        return (all, Vec::new());
    };
    let mut shown: Vec<usize> = all[..max].to_vec();
    let mut hidden: Vec<usize> = all[max..].to_vec();
    if let Some(a) = active.filter(|a| hidden.contains(a)) {
        let last = shown[max - 1];
        hidden.retain(|t| *t != a);
        hidden.insert(0, last);
        shown[max - 1] = a;
    }
    (shown, hidden)
}

type ValueChange = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The right panel's tab strip (`RightPanelTabs`, `.t3-rptabs`): a 40px
/// `layer-01` strip with a bottom rule. Each tab (glyph + label, 12px
/// inline padding, `text-secondary`) sits in a cell with a right rule;
/// the active cell takes `background` with a 2px `border-interactive` top
/// line. Closable tabs end with a 14px close button; a disabled tab is
/// `text-disabled` with its reason in a tooltip. Tabs past `max_visible`
/// fold into an overflow [`IconMenu`]; `on_add` shows a ghost "Open
/// surface" button. Value is controlled (`value`) or internal.
#[derive(IntoElement)]
pub struct RightPanelTabs {
    id: ElementId,
    tabs: Vec<SurfaceTab>,
    value: Option<SharedString>,
    on_value_change: Option<ValueChange>,
    on_close: Option<ValueChange>,
    on_add: Option<Handler>,
    max_visible: Option<usize>,
    add_label: SharedString,
    close_label: SharedString,
    more_label: SharedString,
    add_menu: Vec<MenuItem>,
    trailing: Vec<AnyElement>,
}

impl RightPanelTabs {
    /// A tab strip of `tabs`.
    pub fn new(id: impl Into<ElementId>, tabs: impl IntoIterator<Item = SurfaceTab>) -> Self {
        Self {
            id: id.into(),
            tabs: tabs.into_iter().collect(),
            value: None,
            on_value_change: None,
            on_close: None,
            on_add: None,
            max_visible: None,
            add_label: "Open surface".into(),
            close_label: "Close {}".into(),
            more_label: "{} more surfaces".into(),
            add_menu: Vec::new(),
            trailing: Vec::new(),
        }
    }

    /// Items for an `IconMenu` on the `+` button (instead of bare `on_add`).
    pub fn add_menu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.add_menu = items.into_iter().collect();
        self
    }

    /// Append a trailing control at the far right of the strip.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing.push(element.into_any_element());
        self
    }

    /// The active tab id (controlled `value`).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `onValueChange`.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// `onClose`, with the closed tab's id.
    pub fn on_close(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// `onAdd`: shows the add button.
    pub fn on_add(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add = Some(Rc::new(f));
        self
    }

    /// Fold tabs past `max` into the overflow menu (`maxVisible`).
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = Some(max);
        self
    }

    /// Labels: the add button, the close button (`{}` = tab label) and
    /// the overflow menu (`{}` = hidden count).
    pub fn labels(
        mut self,
        add: impl Into<SharedString>,
        close: impl Into<SharedString>,
        more: impl Into<SharedString>,
    ) -> Self {
        self.add_label = add.into();
        self.close_label = close.into();
        self.more_label = more.into();
        self
    }
}

/// (cell background, tab text) of a right-panel tab: active `background`
/// and `text-primary`; hover `layer-hover-01` and `text-primary`; disabled
/// `text-disabled` with no hover; else `text-secondary`.
pub fn tab_colors(active: bool, hovered: bool, disabled: bool) -> (Hsla, Hsla) {
    let clear = gpui_kit::transparent_black();
    if disabled {
        (clear, color(ColorRole::TextDisabled))
    } else if hovered {
        // `.t3-rptab:hover { background: var(--layer-hover-01) }` over the cell.
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::TextPrimary),
        )
    } else if active {
        (clear, color(ColorRole::TextPrimary))
    } else {
        (clear, color(ColorRole::TextSecondary))
    }
}

impl RenderOnce for RightPanelTabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let first = self.tabs.first().map(|t| t.id.clone()).unwrap_or_default();
        let state = window.use_keyed_state((id.clone(), "value"), cx, |_, _| first);
        let value = self.value.clone().unwrap_or_else(|| state.read(cx).clone());
        let controlled = self.value.is_some();
        let set: ValueChange = {
            let on_change = self.on_value_change.clone();
            Rc::new(move |v: SharedString, window: &mut Window, cx: &mut App| {
                if !controlled {
                    state.update(cx, |s, cx| {
                        *s = v.clone();
                        cx.notify();
                    });
                }
                if let Some(f) = &on_change {
                    f(v, window, cx);
                }
            })
        };
        let active = self.tabs.iter().position(|t| t.id == value);
        let (shown, hidden) = split_tabs(self.tabs.len(), active, self.max_visible);

        let mut cells: Vec<AnyElement> = Vec::new();
        for i in shown {
            let tab = self.tabs[i].clone();
            let tab_id: ElementId =
                (id.clone(), SharedString::from(format!("tab-{}", tab.id))).into();
            let on = tab.id == value;
            let disabled = tab.disabled_reason.is_some();
            let (hovered, store) = hover((tab_id.clone(), "hover"), window, cx);
            let (handle, focused) = focus((tab_id.clone(), "focus"), window, cx);
            let (bg, fg) = tab_colors(on, hovered, disabled);
            let select = {
                let set = set.clone();
                let tid = tab.id.clone();
                move |_: &gpui_kit::ClickEvent, w: &mut Window, cx: &mut App| {
                    if !disabled {
                        set(tid.clone(), w, cx)
                    }
                }
            };
            // `.t3-rptab { gap: var(--spacing-03); padding: 0 var(--spacing-04);
            // font-size: 0.875rem; white-space: nowrap }`, `--closable { padding-right: spacing-02 }`.
            let button = sans(press((tab_id.clone(), "btn"), &handle), 14., 400)
                .flex()
                .h(rem(SIZE_MD.px_value() - 1.)) // the strip less its bottom rule (a tooltip wrapper cannot stretch)
                .items_center()
                .gap(rem(8.))
                .pl(rem(12.))
                .pr(rem(if tab.closable { 4. } else { 12. }))
                .whitespace_nowrap()
                .bg(bg)
                .text_color(fg)
                .when(disabled, |b| b.cursor_not_allowed())
                .on_hover(track_hover(store))
                .on_click(select)
                .child(
                    Icon::carbon(tab.resolved_icon())
                        .size_length(rem(16.))
                        .color(fg),
                )
                .child(tab.label.clone())
                .when(focused, |b| b.child(ring()));
            let button: AnyElement = match &tab.disabled_reason {
                Some(reason) => Tooltip::new((tab_id.clone(), "tip"), button)
                    .content(reason.clone())
                    .into_any_element(),
                None => button.into_any_element(),
            };
            let close = tab.closable.then(|| {
                let (x_hovered, x_store) = hover((tab_id.clone(), "x-hover"), window, cx);
                let (x_handle, x_focused) = focus((tab_id.clone(), "x-focus"), window, cx);
                let on_close = self.on_close.clone();
                let tid = tab.id.clone();
                // `.t3-rptab-x { margin-right: spacing-03; padding: 2px; border-radius: 2px;
                // color: icon-secondary }`, `:hover { background: background-hover; color: icon-primary }`.
                let ink = color(if x_hovered {
                    ColorRole::IconPrimary
                } else {
                    ColorRole::IconSecondary
                });
                press((tab_id.clone(), "x"), &x_handle)
                    .flex()
                    .flex_none()
                    .self_center()
                    .mr(rem(8.))
                    .p(px(2.))
                    .rounded(px(2.))
                    .when(x_hovered, |b| b.bg(color(ColorRole::BackgroundHover)))
                    .accessibility_label(SharedString::from(
                        self.close_label.replace("{}", &tab.label),
                    ))
                    .on_hover(track_hover(x_store))
                    .on_click(move |_, w, cx| {
                        if let Some(f) = &on_close {
                            f(tid.clone(), w, cx)
                        }
                    })
                    .child(icon(CarbonIcon::Close, 14., ink))
                    .when(x_focused, |b| b.child(ring()))
            });
            // `.t3-rptab-wrap { border-right: 1px solid var(--border-subtle-01) }`;
            // `--on { background: var(--background); box-shadow: inset 0 2px 0 var(--border-interactive) }`.
            cells.push(
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .h_full()
                    .border_r_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .when(on, |d| d.bg(color(ColorRole::Background)))
                    .child(button)
                    .children(close)
                    .when(on, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .h(px(2.))
                                .bg(color(ColorRole::BorderInteractive)),
                        )
                    })
                    .into_any_element(),
            );
        }
        let overflow = (!hidden.is_empty()).then(|| {
            let label = self.more_label.replace("{}", &hidden.len().to_string());
            IconMenu::new((id.clone(), "more"), label)
                .icon(CarbonIcon::OverflowMenuHorizontal)
                .variant(ButtonVariant::Ghost)
                .items(hidden.into_iter().map(|i| {
                    let tab = self.tabs[i].clone();
                    let set = set.clone();
                    MenuItem::item(tab.label.clone())
                        .icon(tab.resolved_icon())
                        .on_select(move |w, cx| set(tab.id.clone(), w, cx))
                }))
        });
        let add: Option<AnyElement> = if !self.add_menu.is_empty() {
            Some(
                IconMenu::new((id.clone(), "add-menu"), self.add_label.clone())
                    .icon(CarbonIcon::Add)
                    .variant(ButtonVariant::Ghost)
                    .items(self.add_menu)
                    .into_any_element(),
            )
        } else {
            self.on_add.map(|f| {
                Button::new((id.clone(), "add"), self.add_label.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Md)
                    .icon(CarbonIcon::Add)
                    .icon_only(true)
                    .on_click(move |_, w, cx| f(w, cx))
                    .into_any_element()
            })
        };
        let has_trailing = !self.trailing.is_empty();
        // `.t3-rptabs { height: var(--size-md); background: var(--layer-01);
        // border-bottom: 1px solid var(--border-subtle-01) }`.
        div()
            .flex()
            .w_full()
            .items_stretch()
            .h(SIZE_MD.rems())
            .bg(color(ColorRole::Layer01))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(
                div()
                    .id((id.clone(), "list"))
                    .flex()
                    .flex_shrink(1.)
                    .min_w_0()
                    .overflow_x_scroll()
                    .children(cells),
            )
            .children(overflow.map(|m| div().flex().items_center().child(m)))
            .children(add.map(|b| div().flex().items_center().child(b)))
            .when(has_trailing, |d| {
                d.child(div().flex_1()).child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .children(self.trailing),
                )
            })
    }
}

// ── PanelLayoutControls ─────────────────────────────────────────────────

/// A panel [`PanelLayoutControls`] toggles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutPanel {
    /// `sidebar` (⌘B).
    Sidebar,
    /// `terminal` (⌘J).
    Terminal,
    /// `right` (⌘⌥B).
    Right,
}

impl LayoutPanel {
    /// The reference id.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sidebar => "sidebar",
            Self::Terminal => "terminal",
            Self::Right => "right",
        }
    }
}

type PanelToggle = Rc<dyn Fn(LayoutPanel, &mut Window, &mut App)>;

/// The header's layout toggles (`PanelLayoutControls`, `.t3-layoutctl`):
/// three ghost `sm` icon buttons 4px apart (SidePanelOpen, Terminal,
/// RightPanelOpen), selected while their panel shows, with the shortcut in
/// the tooltip.
#[derive(IntoElement)]
pub struct PanelLayoutControls {
    id: ElementId,
    sidebar: bool,
    right_panel: bool,
    terminal: bool,
    labels: [SharedString; 3],
    on_toggle: Option<PanelToggle>,
}

impl PanelLayoutControls {
    /// The controls (sidebar shown, terminal and right panel hidden).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sidebar: true,
            right_panel: false,
            terminal: false,
            labels: [
                "Toggle sidebar (⌘B)".into(),
                "Toggle terminal (⌘J)".into(),
                "Toggle right panel (⌘⌥B)".into(),
            ],
            on_toggle: None,
        }
    }

    /// The sidebar shows (`sidebar`).
    pub fn sidebar(mut self, on: bool) -> Self {
        self.sidebar = on;
        self
    }

    /// The right panel shows (`rightPanel`).
    pub fn right_panel(mut self, on: bool) -> Self {
        self.right_panel = on;
        self
    }

    /// The terminal shows (`terminal`).
    pub fn terminal(mut self, on: bool) -> Self {
        self.terminal = on;
        self
    }

    /// The three tooltips: sidebar, terminal, right panel.
    pub fn labels(mut self, labels: [SharedString; 3]) -> Self {
        self.labels = labels;
        self
    }

    /// `onToggle`.
    pub fn on_toggle(mut self, f: impl Fn(LayoutPanel, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PanelLayoutControls {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let [sidebar, terminal, right] = self.labels;
        let toggles = [
            (
                LayoutPanel::Sidebar,
                self.sidebar,
                sidebar,
                CarbonIcon::SidePanelOpen,
            ),
            (
                LayoutPanel::Terminal,
                self.terminal,
                terminal,
                CarbonIcon::Terminal,
            ),
            (
                LayoutPanel::Right,
                self.right_panel,
                right,
                CarbonIcon::RightPanelOpen,
            ),
        ];
        // `.t3-layoutctl { gap: var(--spacing-02) }` (gemini-A).
        div()
            .flex()
            .gap(rem(4.))
            .children(toggles.into_iter().map(|(panel, on, label, glyph)| {
                let f = self.on_toggle.clone();
                Button::new((self.id.clone(), panel.name()), label)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(glyph)
                    .icon_only(true)
                    .selected(on)
                    .on_click(move |_, w, cx| {
                        if let Some(f) = &f {
                            f(panel, w, cx)
                        }
                    })
            }))
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
    fn overflow_keeps_the_active_tab_visible() {
        assert_eq!(split_tabs(4, Some(0), None), (vec![0, 1, 2, 3], vec![]));
        assert_eq!(split_tabs(4, Some(0), Some(2)), (vec![0, 1], vec![2, 3]));
        // Active tab 3 swaps with the last shown (1), which leads the hidden list.
        assert_eq!(split_tabs(5, Some(3), Some(2)), (vec![0, 3], vec![1, 2, 4]));
        assert_eq!(split_tabs(2, Some(1), Some(2)), (vec![0, 1], vec![]));
    }

    #[test]
    fn tabs_follow_t3_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, fg) = tab_colors(false, false, false);
        assert_eq!((bg.a, hex(fg)), (0., "#525252".into()));
        let (bg, fg) = tab_colors(false, true, false);
        assert_eq!((hex(bg), hex(fg)), ("#e8e8e8".into(), "#161616".into()));
        let (bg, fg) = tab_colors(false, true, true);
        assert_eq!((bg.a, fg), (0., color(ColorRole::TextDisabled)));
        assert_eq!(SurfaceKind::Device.icon(), CarbonIcon::Mobile);
        assert_eq!(SurfaceKind::Details.icon(), CarbonIcon::Information);
        assert_eq!(
            SurfaceTab::new("x", "X", SurfaceKind::Other)
                .icon(CarbonIcon::Code)
                .resolved_icon(),
            CarbonIcon::Code
        );
        assert_eq!(SurfaceKind::Other.icon(), CarbonIcon::Document);
    }
}
