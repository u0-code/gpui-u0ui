//! FilterMenu: a searchable multi-select filter (`FilterMenu` in
//! `src/ghost/workspace.jsx`, `.g-fmenu*` in `src/ghost.css`), for the
//! sidebar's project, server, agent and tag filters.
//!
//! The trigger is a quiet 12px button (filter glyph, label, a blue count
//! pill once something is chosen, a chevron) that turns `link-primary` while
//! set. It opens a 16rem floating panel under it: a search field (focused on
//! open) filtering the options by substring, the options with their check
//! box, optional icon, fuzzy-highlighted label and count, "No matching
//! projects" when none match, and "Select all" / "Clear" links. Escape
//! closes it and returns focus to the trigger; a press outside closes it,
//! and so does focus leaving the panel and its trigger (the reference's
//! `onBlur`). Tab follows the reference's DOM order, where the panel sits
//! right after its trigger: Tab on the open trigger enters the search
//! field, Shift+Tab from the search field returns to the trigger (the panel
//! stays open), and Tab from "Clear" closes the panel and moves on to the
//! stop after the trigger. These need [`init`]'s bindings, which beat
//! Base's window-level Tab.
//! The panel hangs from the trigger's start edge 4px below it (flipping
//! above it when only that side has room), like a Carbon dropdown
//! ([`crate::components::overlay::DropAnchor`]); positioning, focus return
//! and outside presses are Base's `Popup`.
use super::super::panels::git::check_box;
use crate::{
    components::overlay,
    density,
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::{
        HighlightedText,
        parts::{editor, focus_ring, focus_visible, keyed_focus},
    },
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ClickEvent, CursorStyle, ElementId, Entity, FocusHandle,
    Focusable as _, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent, KeyboardButton,
    KeyboardClickEvent, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, component::input::InputState, div,
    prelude::FluentBuilder as _, px,
};
use gpui_kit::{KeyBinding, TestSupportExt as _, actions};
use std::rc::Rc;

actions!(
    u0ui_filter_menu,
    [
        /// Tab on the open trigger: into the search field.
        IntoPanel,
        /// Shift+Tab from the search field: back to the trigger, still open.
        BackToTrigger,
        /// Tab from the panel's last stop ("Clear"): close the panel and go
        /// on to the stop after the trigger.
        TabOut
    ]
);

/// The key context of an open menu's trigger.
const TRIGGER_CONTEXT: &str = "U0FilterMenuTrigger";
/// The key context around the search field (the panel's first stop).
const FIRST_CONTEXT: &str = "U0FilterMenuFirst";
/// The key context around "Clear" (the panel's last stop).
const LAST_CONTEXT: &str = "U0FilterMenuLast";

/// Bind Tab and Shift+Tab at the panel's edges. A binding in an element's
/// own key context is deeper than Base's window-level `root::Tab`, so it
/// wins. Called by [`crate::init`].
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", IntoPanel, Some(TRIGGER_CONTEXT)),
        KeyBinding::new("shift-tab", BackToTrigger, Some(FIRST_CONTEXT)),
        KeyBinding::new("tab", TabOut, Some(LAST_CONTEXT)),
    ]);
}

/// `.g-fmenu-pop { top: calc(100% + 4px) }`: the gap under the trigger.
pub const POP_GAP: f32 = 4.;

/// One filter option (`items[i]`).
#[derive(Clone, Debug, PartialEq)]
pub struct FilterItem {
    /// `value`.
    pub value: SharedString,
    /// `label`.
    pub label: SharedString,
    /// `count`.
    pub count: Option<usize>,
    /// `icon`.
    pub icon: Option<CarbonIcon>,
}

impl FilterItem {
    /// An option `value` shown as `label`.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            count: None,
            icon: None,
        }
    }

    /// `count`.
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// `icon`.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// The options whose label contains `query`, case-insensitively
/// (`i.label.toLowerCase().includes(q.toLowerCase())`).
pub fn filter_items<'a>(items: &'a [FilterItem], query: &str) -> Vec<&'a FilterItem> {
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|i| i.label.to_lowercase().contains(&q))
        .collect()
}

/// `tog(k)`: add or remove `value`, keeping the order chosen.
pub fn toggle_value(values: &[SharedString], value: &SharedString) -> Vec<SharedString> {
    if values.contains(value) {
        values.iter().filter(|v| *v != value).cloned().collect()
    } else {
        let mut out = values.to_vec();
        out.push(value.clone());
        out
    }
}

/// The trigger's ink: `text-secondary`; hover `text-primary`; set
/// `link-primary` (`.g-fmenu-trigger--set`, later in the cascade).
pub fn trigger_ink(set: bool, hovered: bool) -> Hsla {
    color(if set {
        ColorRole::LinkPrimary
    } else if hovered {
        ColorRole::TextPrimary
    } else {
        ColorRole::TextSecondary
    })
}

/// An option's focus handle as a tab stop in the panel's tab group, between
/// the search field and the "Select all" / "Clear" links.
pub fn option_focus(focus: FocusHandle) -> FocusHandle {
    focus.tab_index(0).tab_stop(true)
}

/// Whether a click on a checkbox row toggles it: a pointer click or Space,
/// as on `<input type="checkbox">`, but not Enter. GPUI already turns
/// Space and Enter on a focused clickable element into a click (on key up),
/// so a separate Space handler would toggle twice.
pub fn toggles_checkbox(e: &ClickEvent) -> bool {
    !matches!(
        e,
        ClickEvent::Keyboard(KeyboardClickEvent {
            button: KeyboardButton::Enter,
            ..
        })
    )
}

/// The menu's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct FilterMenuTexts {
    /// The panel's name: `"Filter {label}"`.
    pub dialog: Rc<dyn Fn(&str) -> SharedString>,
    /// `"No matching projects"`.
    pub empty: SharedString,
    /// `"Select all"`.
    pub select_all: SharedString,
    /// `"Clear"`.
    pub clear: SharedString,
}

impl Default for FilterMenuTexts {
    fn default() -> Self {
        Self {
            dialog: Rc::new(|l| format!("Filter {l}").into()),
            empty: "No matching projects".into(),
            select_all: "Select all".into(),
            clear: "Clear".into(),
        }
    }
}

type ValuesHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

/// A searchable multi-select filter. See the module docs.
#[derive(IntoElement)]
pub struct FilterMenu {
    id: ElementId,
    label: SharedString,
    items: Vec<FilterItem>,
    value: Option<Vec<SharedString>>,
    default_value: Vec<SharedString>,
    on_value_change: Option<ValuesHandler>,
    search_placeholder: SharedString,
    texts: FilterMenuTexts,
}

impl FilterMenu {
    /// A filter called `label` (the reference's "Projects") over `items`.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = FilterItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            value: None,
            default_value: Vec::new(),
            on_value_change: None,
            search_placeholder: "Filter projects".into(),
            texts: FilterMenuTexts::default(),
        }
    }

    /// The chosen values, controlled (`value`).
    pub fn value(mut self, values: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.value = Some(values.into_iter().map(Into::into).collect());
        self
    }

    /// The first chosen values when uncontrolled (`defaultValue`).
    pub fn default_value(
        mut self,
        values: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = values.into_iter().map(Into::into).collect();
        self
    }

    /// Called with the new choice (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// The search field's placeholder and name (`searchPlaceholder`).
    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: FilterMenuTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.t3-linkbtn { color: var(--link-primary); font-size: 0.75rem }`,
/// underlined on hover.
fn link_button(
    id: ElementId,
    label: SharedString,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let focus = keyed_focus((id.clone(), "focus"), window, cx);
    let ring = focus_visible(&focus, window);
    let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
    let over = *hovered.read(cx);
    sans(gpui_kit::base::Button::new(id), 12., 400)
        .track_focus(&focus)
        .relative()
        .text_color(color(ColorRole::LinkPrimary))
        .when(over, |b| b.underline())
        .cursor(CursorStyle::PointingHand)
        .on_hover(move |h, _, cx| {
            hovered.update(cx, |v, cx| {
                if *v != *h {
                    *v = *h;
                    cx.notify();
                }
            })
        })
        .on_click(move |_, w, cx| on_click(w, cx))
        .child(label)
        .when(ring, |b| b.child(focus_ring()))
}

/// Tab from "Clear": close the panel and move to the stop after the
/// trigger, where the reference's DOM order (panel right after its trigger)
/// takes focus. Base's `root::Tab` does each step, so a focus trap around
/// the menu still holds. GPUI orders the floating panel's stops apart from
/// the page's (last, as it paints last), so steps that land in the closing
/// panel go on through it.
fn tab_out(
    trigger: FocusHandle,
    surface: FocusHandle,
    set_open: overlay::OpenChange,
) -> impl Fn(&TabOut, &mut Window, &mut App) + 'static {
    move |_, window, cx| {
        cx.stop_propagation();
        set_open(false, window, cx);
        trigger.focus(window, cx);
        let Ok(tab) = cx.build_action("root::Tab", None) else {
            return;
        };
        let mut seen = Vec::new();
        loop {
            let Some(from) = window.focused(cx) else {
                break;
            };
            if seen.contains(&from) {
                break;
            }
            from.dispatch_action(&*tab, window, cx);
            seen.push(from);
            if !surface.contains_focused(window, cx) {
                break;
            }
        }
    }
}

impl RenderOnce for FilterMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let default = self.default_value.clone();
        let store = window.use_keyed_state((id.clone(), "value"), cx, move |_, _| default);
        let values = self.value.clone().unwrap_or_else(|| store.read(cx).clone());
        let set: ValuesHandler = {
            let store = store.clone();
            let on_change = self.on_value_change.clone();
            Rc::new(move |v: Vec<SharedString>, window, cx| {
                store.update(cx, |s, cx| {
                    *s = v.clone();
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(v, window, cx);
                }
            })
        };
        let openness = overlay::openness(&id, None, false, None, window, cx);
        let open = openness.open;
        let set_open = openness.setter();
        let search: Entity<InputState> = window
            .use_keyed_state((id.clone(), "search"), cx, |window, cx| {
                cx.new(|cx| InputState::new(window, cx))
            })
            .read(cx)
            .clone();
        let search_focus = search.read(cx).focus_handle(cx);
        let surface_focus =
            overlay::surface_focus(&id, open, Some(search_focus.clone()), window, cx);

        // The trigger.
        let tfocus = keyed_focus((id.clone(), "trigger-focus"), window, cx);
        let tring = focus_visible(&tfocus, window);
        let thover = window.use_keyed_state((id.clone(), "trigger-hover"), cx, |_, _| false);
        let ink = trigger_ink(!values.is_empty(), *thover.read(cx));
        let count = (!values.is_empty()).then(|| {
            // `.g-fmenu-count { padding: 0 5px; background: var(--interactive);
            // color: var(--text-on-color); font-size: 0.6875rem; line-height:
            // 1rem; border-radius: 16 }`.
            sans(div(), 12., 400)
                .line_height(rem(16.))
                .px(px(5.))
                .rounded(tokens::BORDER_RADIUS_16.rems())
                .bg(color(ColorRole::Interactive))
                .text_color(color(ColorRole::TextOnColor))
                .child(values.len().to_string())
        });
        // Focus leaving the panel and its trigger closes the panel without
        // moving focus back (the reference's `onBlur` on `.g-fmenu`, whose
        // `contains(relatedTarget)` counts the trigger as inside). Focus
        // enters a frame after the open, so only a panel focus has had
        // focus can lose it.
        let had_focus = window.use_keyed_state((id.clone(), "had-focus"), cx, |_, _| false);
        if !open {
            if *had_focus.read(cx) {
                had_focus.update(cx, |v, _| *v = false);
            }
        } else if surface_focus.contains_focused(window, cx) {
            if !*had_focus.read(cx) {
                had_focus.update(cx, |v, _| *v = true);
            }
        } else if *had_focus.read(cx) && !tfocus.is_focused(window) {
            had_focus.update(cx, |v, _| *v = false);
            let set_open = set_open.clone();
            window.defer(cx, move |window, cx| set_open(false, window, cx));
        }
        let toggle = set_open.clone();
        let into_panel = {
            let search_focus = search_focus.clone();
            move |_: &IntoPanel, window: &mut Window, cx: &mut App| {
                search_focus.focus(window, cx);
                cx.stop_propagation();
            }
        };
        // `.g-fmenu-trigger { display: inline-flex; align-items: center; gap:
        // var(--spacing-02); height: var(--density-field-sm); padding: 0
        // var(--spacing-03); font-size: 0.75rem }`, hover `layer-hover-01`.
        let trigger = sans(
            gpui_kit::base::Button::new((id.clone(), "trigger")),
            12.,
            400,
        )
        .track_focus(&tfocus)
        .relative()
        .flex()
        .items_center()
        .gap(tokens::SPACING_02)
        .h(density::current().field_sm())
        .px(tokens::SPACING_03)
        .text_color(ink)
        .when(*thover.read(cx), |b| b.bg(color(ColorRole::LayerHover01)))
        .cursor(CursorStyle::PointingHand)
        .on_hover(move |h, _, cx| {
            thover.update(cx, |v, cx| {
                if *v != *h {
                    *v = *h;
                    cx.notify();
                }
            })
        })
        .on_click(move |_, w, cx| toggle(!open, w, cx))
        .when(open, |b| {
            b.key_context(TRIGGER_CONTEXT).on_action(into_panel)
        })
        .child(
            Icon::carbon(CarbonIcon::Filter)
                .size_length(rem(16.))
                .color(ink),
        )
        .child(self.label.clone())
        .children(count)
        .child(
            Icon::carbon(CarbonIcon::ChevronDown)
                .size_length(rem(12.))
                .color(ink),
        )
        .when(tring, |b| b.child(focus_ring()))
        .into_any_element();

        let surface = open.then(|| {
            let query = search.read(cx).value().to_string();
            let shown = filter_items(&self.items, &query);
            let secondary = color(ColorRole::TextSecondary);
            let options: Vec<AnyElement> = shown
                .iter()
                .enumerate()
                .map(|(i, it)| {
                    let oid: ElementId =
                        (id.clone(), SharedString::from(format!("opt-{i}"))).into();
                    let focus = keyed_focus((oid.clone(), "focus"), window, cx);
                    let ring = focus_visible(&focus, window);
                    let hovered = window.use_keyed_state((oid.clone(), "hover"), cx, |_, _| false);
                    let over = *hovered.read(cx);
                    let on = values.contains(&it.value);
                    let flip = {
                        let set = set.clone();
                        let values = values.clone();
                        let value = it.value.clone();
                        move |w: &mut Window, cx: &mut App| {
                            set(toggle_value(&values, &value), w, cx)
                        }
                    };
                    // `.g-fmenu-opt { display: flex; align-items: center; gap:
                    // var(--spacing-03); min-height: var(--density-row-md); padding:
                    // 0 var(--spacing-04); font-size: 0.875rem }`, hover `layer-hover-01`.
                    // Each option is a tab stop, toggled by Space, like the
                    // reference's `<label><input type="checkbox">`. The handle
                    // itself carries the stop: a tracked handle ignores the
                    // div's own `tab_stop`.
                    sans(div(), 14., 400)
                        .id(oid)
                        .track_focus(&option_focus(focus))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(tokens::SPACING_03)
                        .min_h(density::current().row())
                        .px(tokens::SPACING_04)
                        .text_color(color(ColorRole::TextPrimary))
                        .when(over, |d| d.bg(color(ColorRole::LayerHover01)))
                        .cursor(CursorStyle::PointingHand)
                        .on_hover(move |h, _, cx| {
                            hovered.update(cx, |v, cx| {
                                if *v != *h {
                                    *v = *h;
                                    cx.notify();
                                }
                            })
                        })
                        .on_click(move |e, w, cx| {
                            if toggles_checkbox(e) {
                                flip(w, cx)
                            }
                        })
                        .child(check_box(on, ring))
                        .when_some(it.icon, |d, g| {
                            d.child(
                                Icon::carbon(g)
                                    .size_length(rem(16.))
                                    .color(color(ColorRole::IconSecondary)),
                            )
                        })
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .child(HighlightedText::new(it.label.clone()).query(query.clone())),
                        )
                        .when_some(it.count, |d, n| {
                            // `.g-fmenu-n { margin-left: auto; font-size: 0.75rem }`.
                            d.child(
                                sans(div(), 12., 400)
                                    .ml_auto()
                                    .text_color(secondary)
                                    .child(n.to_string()),
                            )
                        })
                        .into_any_element()
                })
                .collect();
            let all: Vec<SharedString> = self.items.iter().map(|i| i.value.clone()).collect();
            let (set_all, set_none) = (set.clone(), set.clone());
            let close = set_open.clone();
            let outside = set_open.clone();
            let search_ring = search_focus.is_focused(window);
            let placeholder = query.is_empty().then(|| {
                sans(div(), 14., 400)
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .text_color(color(ColorRole::TextPlaceholder))
                    .child(self.search_placeholder.clone())
            });
            // `.g-pop.g-fmenu-pop { width: 16rem; background: var(--layer-01);
            // box-shadow: 0 2px 6px var(--shadow) }`.
            div()
                .id((id.clone(), "surface"))
                .test_support()
                .track_focus(&surface_focus)
                .tab_group()
                .flex()
                .flex_col()
                .w(rem(256.))
                .bg(color(ColorRole::Layer01))
                .shadow(overlay::floating_shadow())
                .on_key_down(move |e: &KeyDownEvent, w, cx| {
                    if e.keystroke.key == "escape" {
                        close(false, w, cx);
                        cx.stop_propagation();
                    }
                })
                .on_mouse_down_out(move |_, w, cx| outside(false, w, cx))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(
                    // `.g-fmenu-search { display: flex; align-items: center; gap:
                    // var(--spacing-03); height: var(--density-field-md); padding: 0
                    // var(--spacing-04); border-bottom: 1px solid var(--border-subtle-01) }`,
                    // `:focus-within` outline.
                    div()
                        .key_context(FIRST_CONTEXT)
                        .on_action({
                            let tfocus = tfocus.clone();
                            move |_: &BackToTrigger, window: &mut Window, cx: &mut App| {
                                tfocus.focus(window, cx);
                                cx.stop_propagation();
                            }
                        })
                        .relative()
                        .flex()
                        .items_center()
                        .gap(tokens::SPACING_03)
                        .h(density::current().field())
                        .px(tokens::SPACING_04)
                        .border_b_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .child(
                            Icon::carbon(CarbonIcon::Search)
                                .size_length(rem(16.))
                                .color(color(ColorRole::IconSecondary)),
                        )
                        .child(
                            div()
                                .relative()
                                .flex()
                                .items_center()
                                .flex_1()
                                .min_w_0()
                                .h_full()
                                .children(placeholder)
                                .child(editor(&search)),
                        )
                        .when(search_ring, |d| d.child(focus_ring())),
                )
                .child(
                    // `.g-fmenu-list { max-height: 14rem; overflow: auto; padding:
                    // var(--spacing-02) 0 }`.
                    div()
                        .id((id.clone(), "list"))
                        .flex()
                        .flex_col()
                        .max_h(rem(224.))
                        .overflow_y_scroll()
                        .py(tokens::SPACING_02)
                        .children(options)
                        .when(shown.is_empty(), |d| {
                            d.child(
                                sans(div(), 12., 400)
                                    .py(tokens::SPACING_04)
                                    .px(tokens::SPACING_05)
                                    .text_color(secondary)
                                    .child(self.texts.empty.clone()),
                            )
                        }),
                )
                .child(
                    // `.g-fmenu-foot { display: flex; justify-content: space-between;
                    // padding: var(--spacing-03) var(--spacing-04); border-top: 1px
                    // solid var(--border-subtle-01) }`.
                    div()
                        .flex()
                        .justify_between()
                        .py(tokens::SPACING_03)
                        .px(tokens::SPACING_04)
                        .border_t_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .child(link_button(
                            (id.clone(), "all").into(),
                            self.texts.select_all.clone(),
                            move |w, cx| set_all(all.clone(), w, cx),
                            window,
                            cx,
                        ))
                        .child(
                            div()
                                .key_context(LAST_CONTEXT)
                                .on_action(tab_out(
                                    tfocus.clone(),
                                    surface_focus.clone(),
                                    set_open.clone(),
                                ))
                                .child(link_button(
                                    (id.clone(), "clear").into(),
                                    self.texts.clear.clone(),
                                    move |w, cx| set_none(Vec::new(), w, cx),
                                    window,
                                    cx,
                                )),
                        ),
                )
        });
        // `.g-fmenu-pop { position: absolute; top: calc(100% + 4px); left: 0 }`:
        // the panel's start edge on the trigger's, not a popover's
        // `bottom-start` (which starts 1rem before the trigger's center).
        let anchor = overlay::drop_anchor((id.clone(), "drop"), window, cx);
        let trigger = div()
            .relative()
            .child(trigger)
            .child(anchor.measure_trigger());
        let side = anchor.place_measured(window);
        let popup = gpui_kit::base::Popup::new((id, "popup"), trigger)
            .anchor(side.anchor())
            .offset(px(POP_GAP));
        match surface {
            Some(surface) => popup.content(anchor.surface(surface)).into_any_element(),
            None => popup.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    fn items() -> Vec<FilterItem> {
        vec![
            FilterItem::new("carbon-ui", "carbon-ui").count(6),
            FilterItem::new("marketing-site", "marketing-site").count(2),
            FilterItem::new("api-gateway", "api-gateway").count(9),
        ]
    }

    #[test]
    fn filters_by_substring() {
        let items = items();
        let shown: Vec<&str> = filter_items(&items, "GAT")
            .iter()
            .map(|i| i.label.as_ref())
            .collect();
        assert_eq!(shown, vec!["api-gateway"]);
        assert_eq!(filter_items(&items, "").len(), 3);
        assert!(filter_items(&items, "zz").is_empty());
    }

    #[test]
    fn toggles_and_trigger_ink() {
        let v: Vec<SharedString> = vec!["a".into()];
        assert_eq!(
            toggle_value(&v, &"b".into()),
            vec![SharedString::from("a"), "b".into()]
        );
        assert!(toggle_value(&v, &"a".into()).is_empty());
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(trigger_ink(false, false), c(ColorRole::TextSecondary));
        assert_eq!(trigger_ink(false, true), c(ColorRole::TextPrimary));
        assert_eq!(trigger_ink(true, true), c(ColorRole::LinkPrimary));
    }

    #[test]
    fn checkbox_rows_toggle_on_space_not_enter() {
        let key = |button| {
            ClickEvent::Keyboard(KeyboardClickEvent {
                button,
                bounds: Default::default(),
            })
        };
        assert!(toggles_checkbox(&key(KeyboardButton::Space)));
        assert!(!toggles_checkbox(&key(KeyboardButton::Enter)));
    }
}
