//! Ghost · Foundations › Icon: Material Symbols Sharp under Carbon names,
//! any Material name, filled variants, status icons and the four sizes.
use crate::page::Body;
use crate::{controls, stand_in};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, component::input::InputState, div, rems,
};
use u0ui::prelude::*;

const NAMES: [CarbonIcon; 18] = [
    CarbonIcon::Add,
    CarbonIcon::Search,
    CarbonIcon::Settings,
    CarbonIcon::Terminal,
    CarbonIcon::Branch,
    CarbonIcon::PullRequest,
    CarbonIcon::Commit,
    CarbonIcon::Folder,
    CarbonIcon::Bot,
    CarbonIcon::Time,
    CarbonIcon::Locked,
    CarbonIcon::Copy,
    CarbonIcon::TrashCan,
    CarbonIcon::Launch,
    CarbonIcon::Pin,
    CarbonIcon::Mobile,
    CarbonIcon::Laptop,
    CarbonIcon::Notification,
];

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let name = stand_in::input_state("rocket_launch", window, cx);
    cx.new(|cx| {
        cx.observe(&name, |_, _, cx| cx.notify()).detach();
        IconPage {
            name,
            filled: false,
            size: 1,
        }
    })
    .into()
}

struct IconPage {
    name: Entity<InputState>,
    filled: bool,
    size: usize,
}

/// The status warning: `support-warning` with its "!" drawn dark
/// (`.t3-warnicon path[data-icon-path="inner-path"] { fill: #161616 }`).
fn warning(size: f32) -> impl IntoElement {
    let s = size / 16.;
    div()
        .relative()
        .size(rems(size / 16.))
        .child(
            div()
                .absolute()
                .left(rems(7. * s / 16.))
                .top(rems(6. * s / 16.))
                .w(rems(2. * s / 16.))
                .h(rems(7.5 * s / 16.))
                .bg(u0ui::theme::color_in(
                    CarbonTheme::White,
                    ColorRole::TextPrimary,
                )),
        )
        .child(
            Icon::carbon(CarbonIcon::WarningAltFilled)
                .size_length(gpui_kit::px(size))
                .color(color(ColorRole::SupportWarning))
                .absolute()
                .top_0()
                .left_0(),
        )
}

/// `components/Icon/preview.html`.
pub fn reference() -> impl IntoElement {
    let grid = div()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_05)
        .items_center()
        .children(NAMES.map(|n| {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(tokens::SPACING_02)
                .w(rems(4.))
                .text_size(rems(0.6875))
                .text_color(color(ColorRole::TextSecondary))
                .child(
                    n.icon()
                        .size(IconSize::S20)
                        .color(color(ColorRole::IconPrimary)),
                )
                .child(n.name())
        }));
    let row = div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
        .child(
            Icon::named("terminal")
                .size(IconSize::S24)
                .color(color(ColorRole::IconPrimary)),
        )
        .child(
            Icon::named("keep")
                .filled(true)
                .size(IconSize::S24)
                .color(color(ColorRole::IconPrimary)),
        )
        .child(
            Icon::named("rocket_launch")
                .size(IconSize::S24)
                .color(color(ColorRole::IconPrimary)),
        )
        .child(
            Icon::carbon(CarbonIcon::CheckmarkFilled)
                .size(IconSize::S20)
                .color(color(ColorRole::SupportSuccess)),
        )
        .child(warning(20.))
        .child(
            Icon::carbon(CarbonIcon::ErrorFilled)
                .size(IconSize::S20)
                .color(color(ColorRole::SupportError)),
        )
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child("16 · 20 · 24 · 32 px, Sharp, weight 400"),
        );
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(grid)
        .child(row)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .gap(tokens::SPACING_04)
        .children(NAMES[..6].iter().map(|n| {
            n.icon()
                .size(IconSize::S24)
                .color(color(ColorRole::IconPrimary))
        }))
        .into_any_element()
}

impl Render for IconPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let name = self.name.read(cx).value().to_string();
        let size = IconSize::ALL[self.size];
        let live = div()
            .flex()
            .items_center()
            .gap(tokens::SPACING_05)
            .child(
                Icon::named(&name)
                    .filled(self.filled)
                    .size(size)
                    .color(color(ColorRole::IconPrimary)),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("<Icon name=\"{name}\" size={{{}}} />", size.px())),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .max_w(rems(18.))
            .child(stand_in::text_field(
                "icon-name",
                "name",
                &self.name,
                LayerLevel::Two,
            ))
            .child(controls::toggle(
                "icon-filled",
                "filled",
                self.filled,
                controls::setter(&entity, |this: &mut Self, on| this.filled = on),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "icon-size",
                    &["16", "20", "24", "32"],
                    self.size,
                    controls::setter(&entity, |this: &mut Self, i| this.size = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Any Material Symbols Sharp name")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
