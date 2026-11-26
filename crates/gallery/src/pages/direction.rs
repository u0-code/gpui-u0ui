//! Layout › Direction: the reference RTL breadcrumb and input inside a
//! `DirectionProvider` (mirrored: breadcrumb right to left, label and input
//! at the right edge with right-aligned text), a live readout of
//! `use_direction()`, and a note of which components mirror.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    RenderOnce, Styled, Window, div, rems,
};
use u0ui::components::direction::{Direction, DirectionProvider, use_direction};
use u0ui::components::{Breadcrumb, BreadcrumbItem, Input, InputState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let reference = cx.new(|cx| InputState::new(window, cx));
    let live = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| DirectionPage {
        reference,
        live,
        direction: 1,
    })
    .into()
}

struct DirectionPage {
    reference: Entity<InputState>,
    live: Entity<InputState>,
    direction: usize,
}

/// Reads the provided direction while rendering, as a mirroring component would.
#[derive(IntoElement)]
struct Readout;

impl RenderOnce for Readout {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Code01)
            .text_color(color(ColorRole::TextSecondary))
            .child(format!("use_direction() = \"{}\"", use_direction().name()))
    }
}

/// `preview.html`: the Arabic breadcrumb, then a 300px-wide input 12px below,
/// both direct children of the provider (the reference's `<div dir>`).
fn content(id: &'static str, state: &Entity<InputState>) -> [AnyElement; 2] {
    [
        Breadcrumb::new(id)
            .item(BreadcrumbItem::new("الرئيسية"))
            .item(BreadcrumbItem::new("الإعدادات").current(true))
            .into_any_element(),
        // `style: { marginTop: 12, maxWidth: 300 }` (a block: full width up to 300).
        div()
            .mt(rems(0.75))
            .w_full()
            .max_w(rems(18.75))
            .child(
                Input::new((id, 1usize), state)
                    .label("الاسم")
                    .placeholder("اكتب اسمك"),
            )
            .into_any_element(),
    ]
}

fn note(lines: &[&'static str]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_02)
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextSecondary))
        .children(lines.iter().map(|l| div().child(*l)))
}

impl Render for DirectionPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let direction = Direction::ALL[self.direction];

        let reference = DirectionProvider::new(Direction::Rtl)
            .children(content("dir-ref", &self.reference))
            .child(div().mt(tokens::SPACING_04).child(Readout));

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(
                DirectionProvider::new(direction)
                    .children(content("dir-live", &self.live))
                    .child(div().mt(tokens::SPACING_04).child(Readout)),
            )
            .child(div().child(Readout));

        let honest = note(&[
            "Mirrors under rtl: the provider (children at the right edge, text \
             right-aligned), Breadcrumb (items right to left), form items, Label \
             and Field (labels and messages right-aligned, horizontal rows \
             reversed) and Input (text and placeholder right-aligned, paddings and \
             end icons swapped). Other components read use_direction() to mirror.",
            "GPUI applies no paragraph-level bidi: mixed-direction text keeps its \
             left-to-right run order. A single Arabic run is shaped right to left \
             by the platform.",
        ]);

        let names = Direction::ALL.map(|d| d.name());
        let config = controls::field(
            "direction",
            controls::choices(
                "dir-direction",
                &names,
                self.direction,
                controls::setter(&entity, |this: &mut DirectionPage, i| this.direction = i),
            ),
        );

        Body::new()
            .playground(live)
            .hint("the last readout is outside the provider")
            .config(config)
            .example(controls::block("Reference preview (rtl)", reference))
            .example(controls::block("Which components mirror", honest))
    }
}
