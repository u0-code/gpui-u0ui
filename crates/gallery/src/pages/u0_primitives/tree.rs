//! u0 · Primitives › Tree: the reference project tree, live (select,
//! expand, drag to reorder, pin), reporting the selection and order.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::{Tree, TreeItem};

/// The reference items.
pub fn items() -> Vec<TreeItem> {
    vec![
        TreeItem::new("a", "carbon-ui")
            .icon(CarbonIcon::Folder)
            .pinned(true)
            .open(true)
            .children([
                TreeItem::new("a1", "Add dark theme tokens")
                    .icon(CarbonIcon::Chat)
                    .meta("2m"),
                TreeItem::new("a2", "Fix flaky test")
                    .icon(CarbonIcon::Chat)
                    .meta("1h"),
            ]),
        TreeItem::new("b", "marketing-site")
            .icon(CarbonIcon::Folder)
            .children([TreeItem::new("b1", "Hero copy").icon(CarbonIcon::Chat)]),
        TreeItem::new("c", "api-gateway").icon(CarbonIcon::Folder),
        TreeItem::new("d", "scratch").icon(CarbonIcon::Folder),
    ]
}

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TreePage {
        selected: "b".into(),
        order: "carbon-ui, marketing-site, api-gateway, scratch".into(),
        draggable: true,
    })
    .into()
}

struct TreePage {
    selected: SharedString,
    order: SharedString,
    draggable: bool,
}

/// `components/Tree/preview.html`: 320px on `layer-01`, `b` selected.
pub fn reference(id: &'static str) -> impl IntoElement {
    let selected = SharedString::from("b");
    div()
        .w(rems(20.))
        .max_w_full()
        .bg(color(ColorRole::Layer01))
        .child(Tree::new(id, items()).label("Projects").selected(selected))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(
            Tree::new("thumb-tree", items())
                .label("Projects")
                .selected("b"),
        )
        .into_any_element()
}

impl Render for TreePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let tree = Tree::new("tree-live", items())
            .label("Projects")
            .selected(self.selected.clone())
            .draggable(self.draggable)
            .on_select(controls::setter(&entity, |this: &mut Self, id| {
                this.selected = id
            }))
            .on_change(controls::setter(
                &entity,
                |this: &mut Self, items: Vec<TreeItem>| {
                    this.order = items
                        .iter()
                        .map(|n| {
                            if n.pinned {
                                format!("{} (pinned)", n.label)
                            } else {
                                n.label.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                        .into();
                },
            ));
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .w(rems(20.))
                    .max_w_full()
                    .bg(color(ColorRole::Layer01))
                    .child(tree),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("selected: {}", self.selected))
                    .child(format!("order: {}", self.order)),
            );
        let config = controls::toggle(
            "tree-draggable",
            "draggable",
            self.draggable,
            controls::setter(&entity, |this: &mut Self, on| this.draggable = on),
        );
        Body::new()
            .playground(live)
            .hint("↑/↓ move, →/← expand, Enter selects, Alt+↑/↓ reorders, P pins; drag top-level rows")
            .config(config)
            .example(controls::block("Reference preview", reference("tree-ref")))
    }
}
