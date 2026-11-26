//! Layout › Direction: the reference RTL breadcrumb and input inside a
//! `DirectionProvider`, a live readout of `use_direction()`, and an honest note
//! of what GPUI does and does not change.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, RenderOnce,
    Styled, Window, div, rems,
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

/// `preview.html`: the Arabic breadcrumb, then a 300px-wide input 12px below.
fn content(id: &'static str, state: &Entity<InputState>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .child(
            Breadcrumb::new(id)
                .item(BreadcrumbItem::new("الرئيسية"))
                .item(BreadcrumbItem::new("الإعدادات").current(true)),
        )
        .child(
            div().mt(rems(0.75)).max_w(rems(18.75)).child(
                Input::new((id, 1usize), state)
                    .label("الاسم")
                    .placeholder("اكتب اسمك"),
            ),
        )
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
            .child(content("dir-ref", &self.reference))
            .child(div().mt(tokens::SPACING_04).child(Readout));

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(
                DirectionProvider::new(direction)
                    .child(content("dir-live", &self.live))
                    .child(div().mt(tokens::SPACING_04).child(Readout)),
            )
            .child(div().child(Readout));

        let honest = note(&[
            "Changes: use_direction() returns the provided value to every component \
             rendered inside the provider, so a component can mirror itself.",
            "Does not change: GPUI lays out left to right only. Rows are not reversed \
             (the breadcrumb still reads left to right), text stays left-aligned, \
             and mixed-direction text gets no paragraph-level bidi reordering. \
             A single Arabic run is still shaped right to left by the platform.",
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

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview (rtl)", reference))
                .child(controls::block(
                    "Live (the last readout is outside the provider)",
                    live,
                ))
                .child(controls::block("In GPUI", honest)),
            Some(config.into_any_element()),
        )
    }
}
