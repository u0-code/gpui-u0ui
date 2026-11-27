//! T3 · Mobile › ControlPill: the reference toolbar (Opus active, main,
//! Ask first, and an All / Mine segmented ToggleGroup) and a live pill.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::toggle::ToggleSize;
use u0ui::components::{ToggleGroup, ToggleGroupItem};
use u0ui::prelude::*;
use u0ui::t3::mobile::ControlPill;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        active: true,
        icon: true,
        presses: 0,
    })
    .into()
}

struct Page {
    active: bool,
    icon: bool,
    presses: usize,
}

fn row() -> gpui_kit::Div {
    // `.cu-row { flex-wrap: wrap; gap: 16px; align-items: center }`.
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .child(
            ControlPill::new("cp-thumb-0", "Opus")
                .icon(CarbonIcon::Bot)
                .active(true),
        )
        .child(ControlPill::new("cp-thumb-1", "main").icon(CarbonIcon::Branch))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let reference = row()
            .child(
                ControlPill::new("cp-ref-0", "Opus")
                    .icon(CarbonIcon::Bot)
                    .active(true),
            )
            .child(ControlPill::new("cp-ref-1", "main").icon(CarbonIcon::Branch))
            .child(ControlPill::new("cp-ref-2", "Ask first").icon(CarbonIcon::Locked))
            .child(
                div().w(rems(12.5)).child(
                    ToggleGroup::new("cp-ref-seg")
                        .default_value(["all"])
                        .size(ToggleSize::Sm)
                        .item(ToggleGroupItem::new("all").child("All"))
                        .item(ToggleGroupItem::new("mine").child("Mine")),
                ),
            );
        let mut pill = ControlPill::new("cp-live", "Opus")
            .active(self.active)
            .on_press({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.presses += 1;
                        cx.notify();
                    })
                }
            });
        if self.icon {
            pill = pill.icon(CarbonIcon::Bot);
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(pill)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("onPress × {}", self.presses)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "cp-active",
                "active",
                self.active,
                controls::setter(&entity, |this: &mut Page, v| this.active = v),
            ))
            .child(controls::toggle(
                "cp-icon",
                "icon",
                self.icon,
                controls::setter(&entity, |this: &mut Page, v| this.icon = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
