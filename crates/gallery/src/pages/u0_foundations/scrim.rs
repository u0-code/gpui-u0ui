//! u0 · Foundations › Scrim: the dialog and palette veils over a mock
//! page, live (press the scrim to dismiss it).
//!
//! The palette inside is the u0 · Palette `PalettePage`.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::foundations::{Scrim, ScrimAlign, ScrimVariant};
use u0ui::u0::palette::{PaletteItem, PalettePage};

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

/// `h(C.PalettePage, null, h(C.PaletteItem, {icon: Add, title: 'New thread',
/// shortcut: '⌘ N', active: true}), h(C.PaletteItem, {icon: Settings, title:
/// 'Open settings', shortcut: '⌘ ,'}))` in a 360px box.
fn palette(id: &'static str) -> impl IntoElement {
    div()
        .w(rems(22.5))
        .max_w_full()
        .child(PalettePage::new(id).content(move |_, _, _| {
            vec![
                PaletteItem::new((id, 0usize), "New thread")
                    .icon(CarbonIcon::Add)
                    .shortcut("⌘ N")
                    .active(true)
                    .into_any_element(),
                PaletteItem::new((id, 1usize), "Open settings")
                    .icon(CarbonIcon::Settings)
                    .shortcut("⌘ ,")
                    .into_any_element(),
            ]
        }))
}

/// The reference card's viewport (`@dsCard height=300`): its `12vh` is 36px.
const CARD_VIEWPORT: f32 = 300.;

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
                .viewport_height(rems(CARD_VIEWPORT / 16.))
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
                    .viewport_height(rems(CARD_VIEWPORT / 16.))
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
