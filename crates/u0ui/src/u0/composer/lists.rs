//! ModeMenu, ModeList and MentionList (`src/ghost/workspace.jsx`; `.g-pop`,
//! `.g-pop-h`, `.g-opt*`, `.g-modes`, `.g-mentions` in `src/ghost.css`).
//!
//! - **ModeMenu**: a ghost `sm` menu button named after the current mode
//!   whose menu is one radio group of the modes (Carbon `MenuButton` +
//!   `MenuItemRadioGroup`, here u0ui's [`DropdownMenu`]). As Carbon's
//!   `MenuButton` the menu is centred under the trigger and as wide as it,
//!   at least 10rem, so a long mode ("Auto-accept edits") ellipsizes.
//! - **ModeList**: the expanded form, a 20rem floating list: each option
//!   has its glyph, label and description, an optional shortcut, and a
//!   checkmark when selected (`layer-selected-01` with a 2px
//!   `border-interactive` start bar). Options take focus; Enter / Space pick.
//! - **MentionList**: the `@` popup, 22rem: grouped files and sessions with
//!   their glyph, the fuzzy-highlighted label and a detail; the host's input
//!   drives the active row (↑/↓, Enter, `aria-activedescendant`). With no
//!   match it says so.
use crate::{
    components::{
        DropdownMenu, DropdownMenuKind, DropdownMenuSize, Kbd, KbdGroup, MenuItem,
        overlay::floating_shadow,
    },
    density,
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::{
        HighlightedText,
        parts::{focus_ring, focus_visible, keyed_focus},
    },
};
use gpui_kit::{
    App, CursorStyle, Div, ElementId, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// One agent mode (`modes[i]`).
#[derive(Clone, Debug, PartialEq)]
pub struct Mode {
    /// `id`.
    pub id: SharedString,
    /// `label`.
    pub label: SharedString,
    /// `description` (ModeList only).
    pub description: Option<SharedString>,
    /// `icon` (ModeList only).
    pub icon: Option<CarbonIcon>,
    /// `shortcut`, space-separated keys (`"⇧ ⇥"`).
    pub shortcut: Option<SharedString>,
}

impl Mode {
    /// A mode `id` labelled `label`.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            icon: None,
            shortcut: None,
        }
    }

    /// `description`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `icon`.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// `shortcut`.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

type ValueHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The `.g-opt` row's (background, start bar): `layer-selected-01` with an
/// inset 2px `border-interactive` bar when on (`.g-opt--on`), else
/// `layer-hover-01` under the pointer.
pub fn option_look(on: bool, hovered: bool) -> (Option<Hsla>, bool) {
    if on {
        (Some(color(ColorRole::LayerSelected01)), true)
    } else if hovered {
        (Some(color(ColorRole::LayerHover01)), false)
    } else {
        (None, false)
    }
}

/// `.g-pop { background: var(--layer-01); box-shadow: 0 2px 6px var(--shadow) }`.
fn pop() -> Div {
    div()
        .flex()
        .flex_col()
        .bg(color(ColorRole::Layer01))
        .shadow(floating_shadow())
}

/// `.g-opt { display: flex; align-items: center; gap: var(--spacing-03);
/// min-height: var(--density-row-md); padding: 0 var(--spacing-04);
/// font-size: 0.875rem; color: var(--text-primary) }`.
fn option(on: bool, hovered: bool) -> Div {
    let (bg, bar) = option_look(on, hovered);
    sans(div(), 14., 400)
        .relative()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .min_h(density::current().row())
        .px(tokens::SPACING_04)
        .text_color(color(ColorRole::TextPrimary))
        .when_some(bg, |d, c| d.bg(c))
        .when(bar, |d| {
            d.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(px(2.))
                    .bg(color(ColorRole::BorderInteractive)),
            )
        })
}

/// `.g-kbds { display: inline-flex; gap: 2px }`: one key cap per key.
fn kbds(shortcut: &str) -> impl IntoElement {
    KbdGroup::new().children(shortcut.split(' ').map(|k| Kbd::new(k.to_string())))
}

/// A keyed controlled-or-not value (`useControlled`).
fn value_state(
    id: &ElementId,
    value: Option<SharedString>,
    default: Option<SharedString>,
    on_change: Option<ValueHandler>,
    window: &mut Window,
    cx: &mut App,
) -> (Option<SharedString>, ValueHandler) {
    let store = window.use_keyed_state((id.clone(), "value"), cx, move |_, _| default);
    let current = value.clone().or_else(|| store.read(cx).clone());
    let set: ValueHandler = Rc::new(move |v, window, cx| {
        store.update(cx, |s, cx| {
            *s = Some(v.clone());
            cx.notify();
        });
        if let Some(f) = &on_change {
            f(v, window, cx);
        }
    });
    (current, set)
}

// ── ModeMenu ────────────────────────────────────────────────────────────

/// The toolbar mode picker. See the module docs.
#[derive(IntoElement)]
pub struct ModeMenu {
    id: ElementId,
    modes: Vec<Mode>,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    on_value_change: Option<ValueHandler>,
    group_label: SharedString,
    fallback_label: SharedString,
}

impl ModeMenu {
    /// A picker over `modes`, the first selected by default.
    pub fn new(id: impl Into<ElementId>, modes: impl IntoIterator<Item = Mode>) -> Self {
        Self {
            id: id.into(),
            modes: modes.into_iter().collect(),
            value: None,
            default_value: None,
            on_value_change: None,
            group_label: "Mode".into(),
            fallback_label: "Mode".into(),
        }
    }

    /// The selected mode id, controlled (`value`).
    pub fn value(mut self, id: impl Into<SharedString>) -> Self {
        self.value = Some(id.into());
        self
    }

    /// The first selected mode when uncontrolled (`defaultValue`).
    pub fn default_value(mut self, id: impl Into<SharedString>) -> Self {
        self.default_value = Some(id.into());
        self
    }

    /// Called with the picked mode id (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// The radio group's name and the label with no mode: `"Mode"`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.group_label = label.clone();
        self.fallback_label = label;
        self
    }
}

impl RenderOnce for ModeMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let default = self
            .default_value
            .clone()
            .or_else(|| self.modes.first().map(|m| m.id.clone()));
        let (current, set) = value_state(
            &self.id,
            self.value.clone(),
            default,
            self.on_value_change.clone(),
            window,
            cx,
        );
        let index = self
            .modes
            .iter()
            .position(|m| Some(&m.id) == current.as_ref());
        let label = index
            .map(|i| self.modes[i].label.clone())
            .unwrap_or(self.fallback_label.clone());
        let ids: Vec<SharedString> = self.modes.iter().map(|m| m.id.clone()).collect();
        DropdownMenu::new(self.id.clone())
            .label(label)
            .kind(DropdownMenuKind::Ghost)
            .size(DropdownMenuSize::Sm)
            .item(
                MenuItem::radio_group(
                    self.group_label.clone(),
                    self.modes.iter().map(|m| m.label.clone()),
                )
                .selected_item(index)
                .on_select_item(move |i, w, cx| {
                    if let Some(id) = ids.get(i) {
                        set(id.clone(), w, cx);
                    }
                }),
            )
    }
}

// ── ModeList ────────────────────────────────────────────────────────────

/// The expanded mode list. See the module docs.
#[derive(IntoElement)]
pub struct ModeList {
    id: ElementId,
    modes: Vec<Mode>,
    value: Option<SharedString>,
    on_value_change: Option<ValueHandler>,
}

impl ModeList {
    /// A list over `modes`, the first selected by default.
    pub fn new(id: impl Into<ElementId>, modes: impl IntoIterator<Item = Mode>) -> Self {
        Self {
            id: id.into(),
            modes: modes.into_iter().collect(),
            value: None,
            on_value_change: None,
        }
    }

    /// The selected mode id, controlled (`value`).
    pub fn value(mut self, id: impl Into<SharedString>) -> Self {
        self.value = Some(id.into());
        self
    }

    /// Called with the picked mode id (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ModeList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (current, set) = value_state(
            &id,
            self.value.clone(),
            self.modes.first().map(|m| m.id.clone()),
            self.on_value_change.clone(),
            window,
            cx,
        );
        let rows: Vec<_> = self
            .modes
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let rid: ElementId = (id.clone(), SharedString::from(format!("mode-{i}"))).into();
                let focus = keyed_focus((rid.clone(), "focus"), window, cx);
                let ring = focus_visible(&focus, window);
                let hovered = window.use_keyed_state((rid.clone(), "hover"), cx, |_, _| false);
                let over = *hovered.read(cx);
                let on = current.as_ref() == Some(&m.id);
                let (pick, key_pick) = (set.clone(), set.clone());
                let (mid, key_id) = (m.id.clone(), m.id.clone());
                // `.g-opt--2 { align-items: flex-start; padding: var(--spacing-03)
                // var(--spacing-04) }`.
                option(on, over)
                    .id(rid)
                    // A tracked handle carries its own tab-stop flag; the div's
                    // `.tab_stop` does not reach it.
                    .track_focus(&focus.clone().tab_index(0).tab_stop(true))
                    .items_start()
                    .py(tokens::SPACING_03)
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |h, _, cx| {
                        hovered.update(cx, |v, cx| {
                            if *v != *h {
                                *v = *h;
                                cx.notify();
                            }
                        })
                    })
                    // GPUI already turns Enter/Space on a focused clickable element into a
                    // click (on key up); the key handler below acts on the key itself, so a
                    // keyboard click is ignored here or the action would run twice.
                    .on_click(move |e, w, cx| {
                        if !e.is_keyboard() {
                            pick(mid.clone(), w, cx)
                        }
                    })
                    .on_key_down(move |e: &KeyDownEvent, w, cx| {
                        if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                            key_pick(key_id.clone(), w, cx);
                            cx.stop_propagation();
                        }
                    })
                    .when_some(m.icon, |d, glyph| {
                        d.child(
                            Icon::carbon(glyph)
                                .size_length(rem(16.))
                                .color(color(ColorRole::TextPrimary))
                                .flex_none(),
                        )
                    })
                    .child(
                        // `.g-opt--2 > span { flex: 1; flex-direction: column;
                        // gap: 2px } b { font-weight: 400 } .g-dim { 0.75rem }`.
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(m.label.clone())
                            .when_some(m.description.clone(), |d, t| {
                                d.child(
                                    sans(div(), 12., 400)
                                        .text_color(color(ColorRole::TextSecondary))
                                        .child(t),
                                )
                            }),
                    )
                    .when_some(m.shortcut.clone(), |d, s| d.child(kbds(&s)))
                    .when(on, |d| {
                        // `.g-check-on { fill: var(--interactive); margin-left: auto }`.
                        d.child(
                            Icon::carbon(CarbonIcon::Checkmark)
                                .size_length(rem(16.))
                                .color(color(ColorRole::Interactive))
                                .flex_none(),
                        )
                    })
                    .when(ring, |d| d.child(focus_ring()))
            })
            .collect();
        // `.g-modes { width: 20rem; padding: var(--spacing-02) 0 }`.
        pop()
            .id(id)
            .w(rem(320.))
            .py(tokens::SPACING_02)
            .children(rows)
    }
}

// ── MentionList ─────────────────────────────────────────────────────────

/// A mention group's kind (`kind`): sessions default to the chat glyph,
/// everything else to the document glyph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MentionKind {
    /// Files (the default).
    #[default]
    File,
    /// `session`.
    Session,
}

/// One mention candidate (`items[i]`).
#[derive(Clone, Debug, PartialEq)]
pub struct MentionItem {
    /// `label`.
    pub label: SharedString,
    /// `detail`.
    pub detail: Option<SharedString>,
    /// `icon`.
    pub icon: Option<CarbonIcon>,
}

impl MentionItem {
    /// A candidate labelled `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
        }
    }

    /// `detail`.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// `icon`.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A group of candidates (`groups[i]`).
#[derive(Clone, Debug, PartialEq)]
pub struct MentionGroup {
    /// `label`.
    pub label: SharedString,
    /// `kind`.
    pub kind: MentionKind,
    /// `items`.
    pub items: Vec<MentionItem>,
}

impl MentionGroup {
    /// A group `label` of `kind`.
    pub fn new(
        label: impl Into<SharedString>,
        kind: MentionKind,
        items: impl IntoIterator<Item = MentionItem>,
    ) -> Self {
        Self {
            label: label.into(),
            kind,
            items: items.into_iter().collect(),
        }
    }
}

/// The mention list's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct MentionListTexts {
    /// `"No files or sessions match “{query}”"`.
    pub empty: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for MentionListTexts {
    fn default() -> Self {
        Self {
            empty: Rc::new(|q| format!("No files or sessions match “{q}”").into()),
        }
    }
}

/// The `@` mention popup. See the module docs.
#[derive(IntoElement)]
pub struct MentionList {
    query: SharedString,
    groups: Vec<MentionGroup>,
    active_index: usize,
    texts: MentionListTexts,
}

impl MentionList {
    /// The candidates in `groups`, matched against `query`.
    pub fn new(
        query: impl Into<SharedString>,
        groups: impl IntoIterator<Item = MentionGroup>,
    ) -> Self {
        Self {
            query: query.into(),
            groups: groups.into_iter().collect(),
            active_index: 0,
            texts: MentionListTexts::default(),
        }
    }

    /// The highlighted row, counted across groups (`activeIndex`).
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: MentionListTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for MentionList {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut i = 0usize;
        let any = self.groups.iter().any(|g| !g.items.is_empty());
        let groups: Vec<_> = self
            .groups
            .iter()
            .map(|g| {
                let rows: Vec<_> = g
                    .items
                    .iter()
                    .map(|it| {
                        let on = i == self.active_index;
                        i += 1;
                        let glyph = it.icon.unwrap_or(match g.kind {
                            MentionKind::Session => CarbonIcon::Chat,
                            MentionKind::File => CarbonIcon::Document,
                        });
                        option(on, false)
                            .child(
                                Icon::carbon(glyph)
                                    .size_length(rem(16.))
                                    .color(color(ColorRole::IconSecondary))
                                    .flex_none(),
                            )
                            .child(div().min_w_0().truncate().child(
                                HighlightedText::new(it.label.clone()).query(self.query.clone()),
                            ))
                            .when_some(it.detail.clone(), |d, t| {
                                // `.g-opt-detail { margin-left: auto; font-size: 0.75rem }`.
                                d.child(
                                    sans(div(), 12., 400)
                                        .ml_auto()
                                        .flex_none()
                                        .text_color(color(ColorRole::TextSecondary))
                                        .child(t),
                                )
                            })
                    })
                    .collect();
                div()
                    .flex()
                    .flex_col()
                    .child(
                        // `.g-pop-h { padding: var(--spacing-03) var(--spacing-04)
                        // var(--spacing-02); font-size: 0.75rem; color: var(--text-secondary) }`.
                        sans(div(), 12., 400)
                            .pt(tokens::SPACING_03)
                            .px(tokens::SPACING_04)
                            .pb(tokens::SPACING_02)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(g.label.clone()),
                    )
                    .children(rows)
            })
            .collect();
        // `.g-mentions { width: 22rem; padding-bottom: var(--spacing-02) }`.
        pop()
            .w(rem(352.))
            .pb(tokens::SPACING_02)
            .children(groups)
            .when(!any, |d| {
                // `.g-empty-line { padding: var(--spacing-04) var(--spacing-05);
                // font-size: 0.75rem; color: var(--text-secondary) }`.
                d.child(
                    sans(div(), 12., 400)
                        .py(tokens::SPACING_04)
                        .px(tokens::SPACING_05)
                        .text_color(color(ColorRole::TextSecondary))
                        .child((self.texts.empty)(&self.query)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn the_mode_menu_is_a_10rem_menu_button_menu() {
        // A radio group has no icons: under the reference's 103px trigger
        // the menu is `.cds--menu`'s 10rem, centred (x 3.5 under x 32).
        let items = [MenuItem::radio_group(
            "Mode",
            ["Default", "Plan", "Auto-accept edits"].map(SharedString::from),
        )];
        assert_eq!(
            crate::components::menu::button_menu_width(&items, 103.),
            160.
        );
    }

    #[test]
    fn options_follow_g_opt() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        // `.g-opt--on { background: layer-selected-01; box-shadow: inset 2px 0 0 border-interactive }`.
        assert_eq!(
            option_look(true, true),
            (Some(c(ColorRole::LayerSelected01)), true)
        );
        // `.g-opt:hover { background: layer-hover-01 }`.
        assert_eq!(
            option_look(false, true),
            (Some(c(ColorRole::LayerHover01)), false)
        );
        assert_eq!(option_look(false, false), (None, false));
        assert_eq!(
            (MentionListTexts::default().empty)("zz"),
            "No files or sessions match “zz”"
        );
    }
}
