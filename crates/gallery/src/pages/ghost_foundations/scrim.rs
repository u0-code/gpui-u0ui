//! Ghost · Foundations › Scrim: the dialog and palette veils over a mock
//! page, live (press the scrim to dismiss it).
//!
//! The palette inside is the u0ui `Command` (the Ghost · Palette cards'
//! `PalettePage` is drawn by that group).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::{Command, CommandGroup, CommandItem};
use u0ui::ghost::foundations::{Scrim, ScrimAlign, ScrimVariant};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ScrimPage {
        open: true,
        variant: 1,
        align: 1,
    })
    .into()
}

struct ScrimPage {
    open: bool,
    variant: usize,
    align: usize,
}

/// The page behind: four `layer-01` bars 24px high, 8px apart, 16px in.
fn backdrop() -> impl IntoElement {
    div()
        .p(tokens::SPACING_05)
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .children((0..4).map(|_| div().h(rems(1.5)).bg(color(ColorRole::Layer01))))
}

fn palette(id: &'static str) -> impl IntoElement {
    div().w(rems(22.5)).child(
        Command::new(id).group(
            CommandGroup::new("")
                .item(
                    CommandItem::new("New thread")
                        .icon(CarbonIcon::Add)
                        .shortcut("⌘ N"),
                )
                .item(
                    CommandItem::new("Open settings")
                        .icon(CarbonIcon::Settings)
                        .shortcut("⌘ ,"),
                ),
        ),
    )
}

/// `components/Scrim/preview.html`: a 270px page under a palette scrim.
pub fn reference() -> impl IntoElement {
    div()
        .relative()
        .h(rems(270. / 16.))
        .bg(color(ColorRole::Background))
        .overflow_hidden()
        .child(backdrop())
        .child(
            Scrim::new("scrim-ref")
                .variant(ScrimVariant::Palette)
                .align(ScrimAlign::Top)
                .child(palette("scrim-ref-palette")),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .relative()
        .w(rems(15.))
        .h(rems(6.))
        .overflow_hidden()
        .child(backdrop())
        .child(Scrim::new("scrim-thumb"))
        .into_any_element()
}

impl Render for ScrimPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut area = div()
            .relative()
            .w_full()
            .h(rems(270. / 16.))
            .bg(color(ColorRole::Background))
            .overflow_hidden()
            .child(backdrop());
        if self.open {
            let e = entity.clone();
            area = area.child(
                Scrim::new("scrim-live")
                    .variant(ScrimVariant::ALL[self.variant])
                    .align(ScrimAlign::ALL[self.align])
                    .on_dismiss(move |_, cx| {
                        e.update(cx, |this, cx| {
                            this.open = false;
                            cx.notify();
                        })
                    })
                    .child(palette("scrim-live-palette")),
            );
        }
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "scrim-open",
                "open",
                self.open,
                controls::setter(&entity, |this: &mut Self, on| this.open = on),
            ))
            .child(controls::field(
                "variant",
                controls::choices(
                    "scrim-variant",
                    &["dialog", "palette"],
                    self.variant,
                    controls::setter(&entity, |this: &mut Self, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "align",
                controls::choices(
                    "scrim-align",
                    &["center", "top"],
                    self.align,
                    controls::setter(&entity, |this: &mut Self, i| this.align = i),
                ),
            ));
        Body::new()
            .playground(area)
            .hint("Press the scrim (not the palette) to dismiss it")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
