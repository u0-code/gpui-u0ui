//! Ghost · Shell › RightPanelTabsOverflow: `RightPanelTabs` with
//! `maxVisible`, the tabs past it folded into the trailing "…" menu, the
//! active one always kept in view.
use super::last_action;
use crate::controls;
use crate::page::Body;
use crate::pages::t3_shell::stack;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::shell::{RightPanelTabs, SurfaceKind, SurfaceTab};

/// `components/RightPanelTabsOverflow/preview.html`'s six tabs.
fn reference_tabs() -> Vec<SurfaceTab> {
    vec![
        SurfaceTab::new("d", "Diff", SurfaceKind::Diff),
        SurfaceTab::new("b", "Browser", SurfaceKind::Browser).closable(true),
        SurfaceTab::new("f", "Files", SurfaceKind::Files),
        SurfaceTab::new("t", "Terminal", SurfaceKind::Terminal),
        SurfaceTab::new("p", "PR #482", SurfaceKind::Pr),
        SurfaceTab::new("l", "Linked", SurfaceKind::Linked),
    ]
}

/// `maxVisible` options; 3 is the usual one at a 360px panel.
const MAX_VISIBLE: [&str; 4] = ["2", "3", "4", "all"];

struct OverflowPage {
    tabs: Vec<SurfaceTab>,
    value: SharedString,
    max: usize,
    add: bool,
    action: SharedString,
}

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| OverflowPage {
        tabs: reference_tabs(),
        value: "t".into(),
        max: 1,
        add: true,
        action: "none".into(),
    })
    .into()
}

/// The preview's 420 × 200 box.
fn frame() -> gpui_kit::Div {
    div().w(px(420.)).max_w_full().h(px(200.))
}

/// Overview specimen.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            RightPanelTabs::new("thumb-rpto", reference_tabs())
                .value("t")
                .max_visible(2),
        )
        .into_any_element()
}

impl Render for OverflowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut tabs = RightPanelTabs::new("live-rpto", self.tabs.clone())
            .value(self.value.clone())
            .on_value_change({
                let entity = entity.clone();
                move |v, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.action = format!("Selected {v}").into();
                        this.value = v;
                        cx.notify();
                    })
                }
            })
            .on_close({
                let entity = entity.clone();
                move |v, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.tabs.retain(|t| t.id != v);
                        if this.value == v {
                            this.value =
                                this.tabs.first().map(|t| t.id.clone()).unwrap_or_default();
                        }
                        this.action = format!("Closed {v}").into();
                        cx.notify();
                    })
                }
            });
        if let Ok(max) = MAX_VISIBLE[self.max].parse::<usize>() {
            tabs = tabs.max_visible(max);
        }
        if self.add {
            let entity = entity.clone();
            tabs = tabs.on_add(move |_, cx| {
                entity.update(cx, |this, cx| {
                    let n = this.tabs.len();
                    this.tabs.push(
                        SurfaceTab::new(
                            SharedString::from(format!("chat-{n}")),
                            "Side chat",
                            SurfaceKind::Chat,
                        )
                        .closable(true),
                    );
                    this.action = "Open surface".into();
                    cx.notify();
                })
            });
        }
        let live = stack()
            .w_full()
            .gap(px(16.))
            .child(frame().child(tabs))
            .child(last_action(&self.action));
        let values: Vec<&'static str> = ["d", "b", "f", "t", "p", "l"]
            .into_iter()
            .filter(|id| self.tabs.iter().any(|t| t.id.as_ref() == *id))
            .collect();
        let current = values
            .iter()
            .position(|v| *v == self.value.as_ref())
            .unwrap_or(0);
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "maxVisible",
                controls::choices(
                    "rpto-max",
                    &MAX_VISIBLE,
                    self.max,
                    controls::setter(&entity, |this: &mut OverflowPage, i| this.max = i),
                ),
            ))
            .child(controls::field(
                "value",
                controls::choices("rpto-value", &values, current, {
                    let entity = entity.clone();
                    let values = values.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            if let Some(v) = values.get(i) {
                                this.value = SharedString::from(*v);
                            }
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::toggle(
                "rpto-add",
                "onAdd",
                self.add,
                controls::setter(&entity, |this: &mut OverflowPage, on| this.add = on),
            ))
            .child(
                Button::new("rpto-reset", "Reset tabs")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.tabs = reference_tabs();
                                this.value = "t".into();
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .hint("Pick a tab from the … menu: it swaps into view with the last shown one")
            .config(config)
            .example(controls::block(
                "Reference preview",
                frame().child(
                    RightPanelTabs::new("ref-rpto", reference_tabs())
                        .value("t")
                        .max_visible(3)
                        .on_add(|_, _| {}),
                ),
            ))
    }
}
