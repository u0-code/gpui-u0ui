//! Foundations › Icons: the reference's Carbon-named set and a searchable
//! sample of the full Material Symbols catalog.
use crate::controls;
use crate::page::Body;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window, div, rems,
};
use u0ui::icons::Symbol;
use u0ui::prelude::*;

/// At most this many catalog matches are drawn.
const LIMIT: usize = 240;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search symbols"));
        cx.subscribe(&search, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();
        IconsPage {
            search,
            size: 1,
            filled: false,
        }
    })
    .into()
}

struct IconsPage {
    search: Entity<InputState>,
    /// Index into [`IconSize::ALL`].
    size: usize,
    filled: bool,
}

fn cell(id: SharedString, name: SharedString, icon: Icon, size: IconSize) -> impl IntoElement {
    let side = rems((size.px() + 24.) / 16.);
    div()
        .id(id)
        .size(side)
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::Layer01))
        .text_color(color(ColorRole::IconPrimary))
        .hover(|s| s.bg(color(ColorRole::LayerHover01)))
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(name.clone()).build(window, cx)
        })
        .child(icon.size(size))
}

impl Render for IconsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search.read(cx).value().to_lowercase();
        let size = IconSize::ALL[self.size];
        let filled = self.filled;
        let matches: Vec<Symbol> = Symbol::all()
            .filter(|s| query.is_empty() || s.name().contains(query.trim()))
            .take(LIMIT)
            .collect();
        let entity = cx.entity();
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "Size",
                controls::choices("icon-size", &["16", "20", "24", "32"], self.size, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.size = i;
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::toggle(
                "icon-filled",
                "Filled (FILL 1)",
                filled,
                {
                    let entity = entity.clone();
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.filled = on;
                            cx.notify();
                        })
                    }
                },
            ));
        let preview = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section(format!(
                "Carbon names ({})",
                CarbonIcon::ALL.len()
            )))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_01)
                    .children(CarbonIcon::ALL.iter().map(|icon| {
                        cell(
                            format!("carbon-{}", icon.name()).into(),
                            format!("{} → {}", icon.name(), icon.material_name()).into(),
                            Icon::carbon(*icon),
                            size,
                        )
                    })),
            )
            .child(controls::section("Material Symbols catalog"))
            .child(div().max_w(rems(24.)).child(Input::new(&self.search)))
            .child(
                div()
                    .type_style(TypeStyle::HelperText01)
                    .text_color(color(ColorRole::TextHelper))
                    .child(format!(
                        "Showing {} of {} symbols{}",
                        matches.len(),
                        Symbol::all().len(),
                        if matches.len() == LIMIT {
                            " (refine the search to see more)"
                        } else {
                            ""
                        }
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_01)
                    .children(matches.into_iter().map(|symbol| {
                        cell(
                            format!("symbol-{}", symbol.name()).into(),
                            symbol.name().into(),
                            Icon::symbol(symbol).filled(filled),
                            size,
                        )
                    })),
            );
        Body::new().playground(preview).config(config)
    }
}
