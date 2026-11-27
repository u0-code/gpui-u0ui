//! Tree: a tree whose top level can be reordered by drag and pinned
//! (`Tree` in `src/ghost/primitives.jsx`). For the sidebar project list and
//! workspace folders.
//!
//! - **Behavior**: pinned items sort first. A row press focuses it, toggles
//!   it open if it has children and selects it. Keyboard (roving focus):
//!   ↑/↓ move, →/← open/close (← on a leaf goes to its parent), Enter/Space
//!   select, **Alt+↑/↓** move a top-level item among items of the same pinned
//!   state (the keyboard alternative to dragging), **P** pins/unpins. A
//!   top-level row drags onto another top-level row of the same pinned state
//!   (GPUI drag and drop); the pin glyph is a pointer shortcut only.
//! - **Appearance** (`.g-trow`): `density-row-md` rows indented 16px per
//!   level from 4px; hover `layer-hover-01`; selected `layer-selected-01`
//!   with a 2px `border-interactive` start bar; the drop target a 2px
//!   `border-interactive` top line; the dragged row at 40%. The grip and pin
//!   appear on hover (the pin stays while pinned, `icon-primary`); the
//!   chevron (`icon-secondary`) turns 90° over 110ms productive (`.t3-chev`).
use super::parts::{self, rem};
use crate::{
    density,
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, AppContext as _, BoxShadow, Context, CursorStyle, ElementId, FocusHandle,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Render, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::{collections::HashSet, rc::Rc};

/// `.t3-chev { transition: transform 110ms cubic-bezier(0.2,0,0.38,0.9) }`; spatial.
pub const CHEVRON_MOTION: Motion =
    Motion::new(Curve::ProductiveStandard, Speed::Fast02).spatial(true);

/// One tree node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeItem {
    /// Stable id.
    pub id: SharedString,
    /// Visible label.
    pub label: SharedString,
    /// Leading icon.
    pub icon: Option<CarbonIcon>,
    /// Trailing meta text (`2m`).
    pub meta: Option<SharedString>,
    /// Pinned (sorted first).
    pub pinned: bool,
    /// Initially open.
    pub open: bool,
    /// Children (`None` for a leaf).
    pub children: Option<Vec<TreeItem>>,
}

impl TreeItem {
    /// A leaf.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            ..Default::default()
        }
    }

    /// Leading icon.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Trailing meta text.
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Pinned.
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    /// Initially open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Children (makes it a branch).
    pub fn children(mut self, children: impl IntoIterator<Item = TreeItem>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }
}

/// A visible row (`flat`): the item, its depth and its parent's id.
#[derive(Clone, Debug, PartialEq)]
pub struct FlatRow {
    /// The item.
    pub item: TreeItem,
    /// Depth (0 = top level).
    pub depth: usize,
    /// Parent id.
    pub parent: Option<SharedString>,
}

/// `sorted(ns)`: pinned first, otherwise in order.
pub fn sorted(items: &[TreeItem]) -> Vec<TreeItem> {
    items
        .iter()
        .filter(|n| n.pinned)
        .chain(items.iter().filter(|n| !n.pinned))
        .cloned()
        .collect()
}

/// `walk(sorted(items), 0, null)`: the visible rows.
pub fn flatten(items: &[TreeItem], open: &HashSet<SharedString>) -> Vec<FlatRow> {
    fn walk(
        ns: &[TreeItem],
        d: usize,
        parent: Option<&SharedString>,
        open: &HashSet<SharedString>,
        out: &mut Vec<FlatRow>,
    ) {
        for n in ns {
            out.push(FlatRow {
                item: n.clone(),
                depth: d,
                parent: parent.cloned(),
            });
            if let Some(children) = &n.children
                && open.contains(&n.id)
            {
                walk(children, d + 1, Some(&n.id), open, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(&sorted(items), 0, None, open, &mut out);
    out
}

/// `togglePin(id)` anywhere in the tree.
pub fn toggle_pin(items: &[TreeItem], id: &str) -> Vec<TreeItem> {
    items
        .iter()
        .map(|n| {
            let mut n = n.clone();
            if n.id.as_ref() == id {
                n.pinned = !n.pinned;
            } else if let Some(c) = &n.children {
                n.children = Some(toggle_pin(c, id));
            }
            n
        })
        .collect()
}

/// `move(from, to)` among top-level items (in sorted order); `None` when
/// either is missing, they are the same, or their pinned states differ.
pub fn move_item(items: &[TreeItem], from: &str, to: &str) -> Option<Vec<TreeItem>> {
    if from == to {
        return None;
    }
    let mut n = sorted(items);
    let a = n.iter().position(|x| x.id.as_ref() == from)?;
    let b = n.iter().position(|x| x.id.as_ref() == to)?;
    if n[a].pinned != n[b].pinned {
        return None;
    }
    let m = n.remove(a);
    n.insert(b, m);
    Some(n)
}

/// The neighbour a top-level item swaps with on Alt+↑ (`-1`) / Alt+↓ (`1`).
pub fn alt_target(items: &[TreeItem], id: &str, dir: isize) -> Option<SharedString> {
    let top: Vec<_> = sorted(items).into_iter().map(|x| x.id).collect();
    let k = top.iter().position(|x| x.as_ref() == id)? as isize;
    top.get(usize::try_from(k + dir).ok()?).cloned()
}

/// The dragged value: which tree and which top-level item.
#[derive(Clone, Debug)]
pub struct TreeDrag {
    tree: ElementId,
    id: SharedString,
    label: SharedString,
    pinned: bool,
}

/// The drag preview: the row's label on `layer-01` (the native drag image).
struct DragPreview(SharedString);

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(rem(8.))
            .h(rem(density::current().row().px_value()))
            .flex()
            .items_center()
            .bg(color(ColorRole::Layer01))
            .opacity(0.8)
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .child(self.0.clone())
    }
}

type ChangeHandler = Rc<dyn Fn(Vec<TreeItem>, &mut Window, &mut App)>;
type SelectHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

struct Model {
    items: Vec<TreeItem>,
    open: HashSet<SharedString>,
    focus: Option<SharedString>,
    dragging: Option<SharedString>,
}

/// A reorderable, pinnable tree. See the module docs.
#[derive(IntoElement)]
pub struct Tree {
    id: ElementId,
    items: Vec<TreeItem>,
    selected: Option<SharedString>,
    label: SharedString,
    draggable: bool,
    pin_titles: (SharedString, SharedString, SharedString, SharedString),
    on_select: Option<SelectHandler>,
    on_change: Option<ChangeHandler>,
}

impl Tree {
    /// A tree with a stable `id`, seeded with `items` (it then owns the order).
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = TreeItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            selected: None,
            // The reference's texts; override per locale.
            label: "Tree".into(),
            draggable: true,
            pin_titles: (
                "Pin (P)".into(),
                "Unpin (P)".into(),
                ", pinned".into(),
                "Alt plus arrow keys reorders top-level items. P pins or unpins.".into(),
            ),
            on_select: None,
            on_change: None,
        }
    }

    /// The selected item's id.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected = Some(id.into());
        self
    }

    /// The accessible name (`role="tree"`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Whether top-level rows drag to reorder (default true).
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Texts: pin title, unpin title, the ", pinned" suffix and the keyboard hint.
    pub fn texts(
        mut self,
        pin: impl Into<SharedString>,
        unpin: impl Into<SharedString>,
        pinned: impl Into<SharedString>,
        hint: impl Into<SharedString>,
    ) -> Self {
        self.pin_titles = (pin.into(), unpin.into(), pinned.into(), hint.into());
        self
    }

    /// Called with a row's id on press, Enter or Space.
    pub fn on_select(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Called with the new items after a reorder or pin.
    pub fn on_change(mut self, f: impl Fn(Vec<TreeItem>, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

fn row_focus(
    tree: &ElementId,
    id: &SharedString,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    parts::keyed_focus(
        (tree.clone(), SharedString::from(format!("row-{id}"))),
        window,
        cx,
    )
}

impl RenderOnce for Tree {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tid = self.id.clone();
        let init = self.items.clone();
        let first = self
            .selected
            .clone()
            .or_else(|| init.first().map(|n| n.id.clone()));
        let model = window.use_keyed_state((tid.clone(), "tree"), cx, move |_, _| {
            fn opened(ns: &[TreeItem], out: &mut HashSet<SharedString>) {
                for n in ns {
                    if n.open {
                        out.insert(n.id.clone());
                    }
                    if let Some(c) = &n.children {
                        opened(c, out);
                    }
                }
            }
            let mut open = HashSet::new();
            opened(&init, &mut open);
            Model {
                items: init,
                open,
                focus: first,
                dragging: None,
            }
        });
        if !cx.has_active_drag() && model.read(cx).dragging.is_some() {
            model.update(cx, |m, _| m.dragging = None);
        }
        let (items, open, focus_id, dragging) = {
            let m = model.read(cx);
            (
                m.items.clone(),
                m.open.clone(),
                m.focus.clone(),
                m.dragging.clone(),
            )
        };
        let flat = Rc::new(flatten(&items, &open));
        let (pin_title, unpin_title, pinned_suffix, hint) = self.pin_titles.clone();
        let row_h = density::current().row().px_value();
        let on_change = self.on_change.clone();
        let on_select = self.on_select.clone();

        let update: ChangeHandler = {
            let model = model.clone();
            let on_change = on_change.clone();
            Rc::new(move |n, window, cx| {
                model.update(cx, |m, cx| {
                    m.items = n.clone();
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(n, window, cx);
                }
            })
        };
        let focus_handles: Vec<FocusHandle> = flat
            .iter()
            .map(|r| row_focus(&tid, &r.item.id, window, cx))
            .collect();
        let focus_handles = Rc::new(focus_handles);

        let rows = flat.iter().enumerate().map(|(i, r)| {
            let n = &r.item;
            let has_children = n.children.is_some();
            let is_open = open.contains(&n.id);
            let selected = self.selected.as_ref() == Some(&n.id);
            let focus = focus_handles[i].clone();
            let ring = parts::focus_visible(&focus, window);
            let hover = window.use_keyed_state(
                (tid.clone(), SharedString::from(format!("hover-{}", n.id))),
                cx,
                |_, _| false,
            );
            let hovered = *hover.read(cx);
            let top_drag = self.draggable && r.depth == 0;
            let is_dragging = dragging.as_ref() == Some(&n.id);
            let chevron = motion::animate(
                (tid.clone(), SharedString::from(format!("chev-{}", n.id))),
                if is_open {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.
                },
                CHEVRON_MOTION,
                window,
                cx,
            );

            let on_key = {
                let model = model.clone();
                let flat = flat.clone();
                let handles = focus_handles.clone();
                let update = update.clone();
                let on_select = on_select.clone();
                let row = r.clone();
                move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                    let ks = &e.keystroke;
                    let go = |j: isize, window: &mut Window, cx: &mut App| {
                        let j = j.clamp(0, flat.len() as isize - 1) as usize;
                        let id = flat[j].item.id.clone();
                        model.update(cx, |m, cx| {
                            m.focus = Some(id);
                            cx.notify();
                        });
                        handles[j].focus(window, cx);
                    };
                    let key = ks.key.as_str();
                    if ks.modifiers.alt && (key == "up" || key == "down") {
                        cx.stop_propagation();
                        if row.depth == 0 {
                            let items = model.read(cx).items.clone();
                            let dir = if key == "up" { -1 } else { 1 };
                            if let Some(t) = alt_target(&items, &row.item.id, dir)
                                && let Some(n) = move_item(&items, &row.item.id, &t)
                            {
                                update(n, window, cx);
                            }
                        }
                        return;
                    }
                    match key {
                        "down" => go(i as isize + 1, window, cx),
                        "up" => go(i as isize - 1, window, cx),
                        "p" if !ks.modifiers.platform && !ks.modifiers.control => {
                            let items = model.read(cx).items.clone();
                            update(toggle_pin(&items, &row.item.id), window, cx);
                        }
                        "right" if row.item.children.is_some() => model.update(cx, |m, cx| {
                            m.open.insert(row.item.id.clone());
                            cx.notify();
                        }),
                        "left" => {
                            let is_open = model.read(cx).open.contains(&row.item.id);
                            if row.item.children.is_some() && is_open {
                                model.update(cx, |m, cx| {
                                    m.open.remove(&row.item.id);
                                    cx.notify();
                                });
                            } else if let Some(p) = &row.parent
                                && let Some(j) = flat.iter().position(|x| &x.item.id == p)
                            {
                                go(j as isize, window, cx);
                            }
                        }
                        "enter" | "space" => {
                            if let Some(f) = &on_select {
                                f(row.item.id.clone(), window, cx);
                            }
                        }
                        _ => return,
                    }
                    cx.stop_propagation();
                }
            };
            let on_click = {
                let model = model.clone();
                let on_select = on_select.clone();
                let id = n.id.clone();
                let focus = focus.clone();
                move |_: &gpui_kit::ClickEvent, window: &mut Window, cx: &mut App| {
                    focus.focus(window, cx);
                    model.update(cx, |m, cx| {
                        m.focus = Some(id.clone());
                        if has_children && !m.open.remove(&id) {
                            m.open.insert(id.clone());
                        }
                        cx.notify();
                    });
                    if let Some(f) = &on_select {
                        f(id.clone(), window, cx);
                    }
                }
            };
            let pin = {
                let update = update.clone();
                let model = model.clone();
                let id = n.id.clone();
                let visible = hovered || n.pinned || ring;
                div()
                    .id((tid.clone(), SharedString::from(format!("pin-{}", n.id))))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(24.))
                    .opacity(if visible { 1. } else { 0. })
                    .cursor(CursorStyle::PointingHand)
                    .child(
                        Icon::carbon(if n.pinned {
                            CarbonIcon::PinFilled
                        } else {
                            CarbonIcon::Pin
                        })
                        .size_length(px(14.))
                        .color(color(if n.pinned {
                            ColorRole::IconPrimary
                        } else {
                            ColorRole::IconSecondary
                        })),
                    )
                    .tooltip({
                        let t = if n.pinned {
                            unpin_title.clone()
                        } else {
                            pin_title.clone()
                        };
                        move |window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(t.clone()).build(window, cx)
                        }
                    })
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        let items = model.read(cx).items.clone();
                        update(toggle_pin(&items, &id), window, cx);
                    })
            };
            let base_shadow = selected.then(|| BoxShadow {
                color: color(ColorRole::BorderInteractive),
                offset: gpui_kit::point(px(2.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(0.),
                inset: true,
            });
            let tid2 = tid.clone();
            let pinned = n.pinned;
            let accessible = if n.pinned {
                format!("{}{}", n.label, pinned_suffix)
            } else {
                n.label.to_string()
            };
            div()
                .id((tid.clone(), SharedString::from(format!("trow-{}", n.id))))
                .track_focus(&focus.clone().tab_stop(focus_id.as_ref() == Some(&n.id)))
                .role(gpui_kit::Role::TreeItem)
                .aria_label(accessible)
                .relative()
                .flex()
                .items_center()
                .gap(rem(4.))
                .h(rem(row_h))
                .pr(rem(4.))
                .pl(rem(4. + r.depth as f32 * 16.))
                .cursor(CursorStyle::PointingHand)
                .when(hovered && !selected, |d| {
                    d.bg(color(ColorRole::LayerHover01))
                })
                .when(selected, |d| d.bg(color(ColorRole::LayerSelected01)))
                .when_some(base_shadow, |d, s| d.shadow(vec![s]))
                .when(is_dragging, |d| d.opacity(0.4))
                .on_hover(move |h, _, cx| {
                    hover.update(cx, |v, cx| {
                        *v = *h;
                        cx.notify();
                    })
                })
                .on_key_down(on_key)
                .on_click(on_click)
                .when(top_drag, |d| {
                    let model = model.clone();
                    let update = update.clone();
                    let my_id = n.id.clone();
                    let drag_model = model.clone();
                    d.on_drag(
                        TreeDrag {
                            tree: tid2.clone(),
                            id: n.id.clone(),
                            label: n.label.clone(),
                            pinned,
                        },
                        move |drag: &TreeDrag, _, _, cx| {
                            let id = drag.id.clone();
                            drag_model.update(cx, |m, cx| {
                                m.dragging = Some(id);
                                cx.notify();
                            });
                            cx.new(|_| DragPreview(drag.label.clone()))
                        },
                    )
                    .drag_over::<TreeDrag>({
                        let tid = tid2.clone();
                        move |style, drag, _, _| {
                            // `.g-trow--over { box-shadow: inset 0 2px 0 var(--border-interactive) }`.
                            if drag.tree == tid && drag.pinned == pinned {
                                style.shadow(vec![BoxShadow {
                                    color: color(ColorRole::BorderInteractive),
                                    offset: gpui_kit::point(px(0.), px(2.)),
                                    blur_radius: px(0.),
                                    spread_radius: px(0.),
                                    inset: true,
                                }])
                            } else {
                                style
                            }
                        }
                    })
                    .on_drop({
                        let tid = tid2.clone();
                        let model = model.clone();
                        move |drag: &TreeDrag, window, cx| {
                            if drag.tree != tid {
                                return;
                            }
                            model.update(cx, |m, _| m.dragging = None);
                            let items = model.read(cx).items.clone();
                            if let Some(n) = move_item(&items, &drag.id, &my_id) {
                                update(n, window, cx);
                            }
                        }
                    })
                })
                // `.g-trow-grip`: Draggable 16px, `icon-secondary`, shown on hover.
                .child(if top_drag {
                    Icon::carbon(CarbonIcon::Draggable)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconSecondary))
                        .opacity(if hovered { 1. } else { 0. })
                        .cursor(CursorStyle::OpenHand)
                        .into_any_element()
                } else {
                    div().w(rem(16.)).flex_none().into_any_element()
                })
                .child(if has_children {
                    Icon::carbon(CarbonIcon::ChevronRight)
                        .size_length(px(14.))
                        .color(color(ColorRole::IconSecondary))
                        .rotate(gpui_kit::radians(chevron))
                        .into_any_element()
                } else {
                    div().w(rem(14.)).flex_none().into_any_element()
                })
                .children(n.icon.map(|icon| {
                    Icon::carbon(icon)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconSecondary))
                }))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(n.label.clone()),
                )
                .children(n.meta.clone().map(|m| {
                    div()
                        .text_size(rem(12.))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(m)
                }))
                .child(pin)
                .when(ring, |d| d.child(parts::focus_ring()))
        });

        div()
            .id(tid.clone())
            .role(gpui_kit::Role::Tree)
            .aria_label(self.label.clone())
            .aria_description(hint)
            .flex()
            .flex_col()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .children(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<TreeItem> {
        vec![
            TreeItem::new("a", "carbon-ui")
                .pinned(true)
                .open(true)
                .children([
                    TreeItem::new("a1", "Add dark theme tokens").meta("2m"),
                    TreeItem::new("a2", "Fix flaky test").meta("1h"),
                ]),
            TreeItem::new("b", "marketing-site").children([TreeItem::new("b1", "Hero copy")]),
            TreeItem::new("c", "api-gateway"),
            TreeItem::new("d", "scratch"),
        ]
    }

    fn ids(rows: &[FlatRow]) -> Vec<&str> {
        rows.iter().map(|r| r.item.id.as_ref()).collect()
    }

    #[test]
    fn flatten_sorts_pinned_first_and_walks_open_branches() {
        let items = sample();
        let open: HashSet<SharedString> = ["a".into()].into();
        let rows = flatten(&items, &open);
        assert_eq!(ids(&rows), ["a", "a1", "a2", "b", "c", "d"]);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[1].parent.as_deref(), Some("a"));
        let pinned = toggle_pin(&items, "d");
        assert_eq!(
            ids(&flatten(&pinned, &HashSet::new())),
            ["a", "d", "b", "c"]
        );
    }

    #[test]
    fn moves_stay_within_the_pinned_state() {
        let items = sample();
        // Pinned "a" cannot swap with unpinned "b".
        assert_eq!(move_item(&items, "b", "a"), None);
        let moved = move_item(&items, "d", "b").unwrap();
        assert_eq!(
            moved.iter().map(|x| x.id.as_ref()).collect::<Vec<_>>(),
            ["a", "d", "b", "c"]
        );
        assert_eq!(alt_target(&items, "c", -1).as_deref(), Some("b"));
        assert_eq!(alt_target(&items, "d", 1), None);
        assert_eq!(alt_target(&items, "a", -1), None);
    }
}
