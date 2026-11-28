//! Data display › Avatar: the reference avatars (JD, AK lg, MR xl, a group
//! with +3), a live avatar with every size and color, and the image paths
//! (loaded image, broken image → fallback), the icon fallback (a gray
//! User icon, u0's pending review draft); marks overlapped as a group
//! (`MarkGroup`) at 12, 16 and 24px.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::components::avatar::{
    Avatar, AvatarColor, AvatarFallback, AvatarGroup, AvatarSize, MarkGroup,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AvatarPage {
        size: 1,
        color: 0,
        image: 0,
    })
    .into()
}

struct AvatarPage {
    size: usize,
    color: usize,
    image: usize,
}

const IMAGES: [&str; 3] = ["none", "image", "broken image"];

fn fallback(text: &'static str, color: AvatarColor) -> Avatar {
    Avatar::new().fallback(AvatarFallback::new(text).color(color))
}

impl Render for AvatarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/Avatar/preview.html` (`cu-row`).
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_05)
            .child(fallback("JD", AvatarColor::Blue))
            .child(fallback("AK", AvatarColor::Teal).size(AvatarSize::Lg))
            .child(fallback("MR", AvatarColor::Purple).size(AvatarSize::Xl))
            .child(
                AvatarGroup::new()
                    .avatar(fallback("LS", AvatarColor::Magenta))
                    .avatar(fallback("TP", AvatarColor::Blue))
                    .avatar(fallback("+3", AvatarColor::Gray)),
            );

        let size = AvatarSize::ALL[self.size];
        let color = AvatarColor::ALL[self.color];
        let mut live = fallback("JD", color).size(size);
        match self.image {
            // An image served by the app's asset source (a Material symbol).
            1 => live = live.image(Icon::named("person").asset_path().to_string()),
            2 => live = live.image("missing/avatar.png"),
            _ => {}
        }
        let all = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .children(AvatarSize::ALL.map(|size| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .children(AvatarColor::ALL.map(|c| fallback("JD", c).size(size)))
            }));

        // Marks that are not people (here three Material symbols), overlapped
        // as avatars are.
        let marks = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(tokens::SPACING_05)
            .children([12., 16., 24.].map(|size| {
                MarkGroup::new(size).marks(["smart_toy", "terminal", "code"].map(|name| {
                    Icon::named(name)
                        .size_length(gpui_kit::px(size))
                        .color(theme::color(ColorRole::IconPrimary))
                }))
            }));

        let sizes = AvatarSize::ALL.map(|s| s.name());
        let colors = AvatarColor::ALL.map(|c| c.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "avatar-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut AvatarPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "AvatarFallback color",
                controls::choices(
                    "avatar-color",
                    &colors,
                    self.color,
                    controls::setter(&entity, |this: &mut AvatarPage, i| this.color = i),
                ),
            ))
            .child(controls::field(
                "AvatarImage",
                controls::choices(
                    "avatar-image",
                    &IMAGES,
                    self.image,
                    controls::setter(&entity, |this: &mut AvatarPage, i| this.image = i),
                ),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block("MarkGroup", marks))
            .example(controls::block("Sizes × colors", all))
            .example(controls::block(
                "Icon fallback",
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .children(AvatarSize::ALL.map(|size| {
                        Avatar::new().size(size).fallback(
                            AvatarFallback::icon(CarbonIcon::User).color(AvatarColor::Gray),
                        )
                    })),
            ))
    }
}
