//! Data display › Item: the reference group (an outline folder item with an
//! Open action, a linked Jane Doe item with an avatar, a muted small item
//! with a switch) and a live item with every variant, size and media.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::avatar::{Avatar, AvatarFallback};
use u0ui::components::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemSeparator, ItemSize, ItemTitle, ItemVariant,
};
use u0ui::components::switch::Switch;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ItemPage {
        variant: 1,
        size: 0,
        media: 1,
        link: false,
        header: false,
    })
    .into()
}

struct ItemPage {
    variant: usize,
    size: usize,
    media: usize,
    link: bool,
    header: bool,
}

fn folder(variant: ItemMediaVariant) -> ItemMedia {
    ItemMedia::new().variant(variant).child(
        Icon::carbon(CarbonIcon::Folder)
            .size(IconSize::S16)
            .color(ItemMedia::icon_color()),
    )
}

impl Render for ItemPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/Item/preview.html`: an ItemGroup, max-width 460.
        let reference = ItemGroup::new()
            .child(
                Item::new("ref-logs")
                    .variant(ItemVariant::Outline)
                    .child(folder(ItemMediaVariant::Icon))
                    .child(
                        ItemContent::new()
                            .child(ItemTitle::new("logs-2026"))
                            .child(ItemDescription::new("12,408 objects · eu-de")),
                    )
                    .child(
                        ItemActions::new().child(
                            Button::new("ref-open", "Open")
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm),
                        ),
                    ),
            )
            .child(ItemSeparator::new())
            .child(
                Item::new("ref-jane")
                    .href("https://carbondesignsystem.com")
                    .label("Jane Doe")
                    .child(
                        ItemMedia::new().child(Avatar::new().fallback(AvatarFallback::new("JD"))),
                    )
                    .child(
                        ItemContent::new()
                            .child(ItemTitle::new("Jane Doe"))
                            .child(ItemDescription::new("Owner")),
                    )
                    .child(
                        ItemActions::new().child(
                            Icon::carbon(CarbonIcon::ChevronRight)
                                .size(IconSize::S16)
                                .color(color(ColorRole::IconPrimary)),
                        ),
                    ),
            )
            .child(ItemSeparator::new())
            .child(
                Item::new("ref-2fa")
                    .variant(ItemVariant::Muted)
                    .size(ItemSize::Sm)
                    .child(
                        ItemContent::new()
                            .child(ItemTitle::new("Two-factor authentication"))
                            .child(ItemDescription::new("Required for admins.")),
                    )
                    .child(
                        ItemActions::new()
                            .child(Switch::new("ref-2fa-switch").default_checked(true)),
                    ),
            );

        let media = match ItemMediaVariant::ALL[self.media] {
            ItemMediaVariant::Default => ItemMedia::new()
                .child(Avatar::new().fallback(AvatarFallback::new("JD")))
                .into_any_element(),
            v => folder(v).into_any_element(),
        };
        let mut live = Item::new("live")
            .variant(ItemVariant::ALL[self.variant])
            .size(ItemSize::ALL[self.size]);
        if self.link {
            live = live.on_click(|_, _, _| {}).label("logs-2026");
        }
        if self.header {
            live = live.child(
                ItemHeader::new().child("Header").child(
                    div()
                        .text_color(color(ColorRole::TextSecondary))
                        .child("2 min ago"),
                ),
            );
        }
        live = live
            .child(media)
            .child(
                ItemContent::new()
                    .child(ItemTitle::new("logs-2026"))
                    .child(ItemDescription::new("12,408 objects · eu-de")),
            )
            .child(
                ItemActions::new().child(
                    Button::new("live-open", "Open")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm),
                ),
            );
        if self.header {
            live = live.child(
                ItemFooter::new()
                    .child(
                        div()
                            .text_color(color(ColorRole::TextSecondary))
                            .child("Footer"),
                    )
                    .child("Owner: Jane Doe"),
            );
        }

        let variants = ItemVariant::ALL.map(|v| v.name());
        let sizes = ItemSize::ALL.map(|v| v.name());
        let medias = ItemMediaVariant::ALL.map(|v| v.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "item-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut ItemPage, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "item-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut ItemPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "ItemMedia variant",
                controls::choices(
                    "item-media",
                    &medias,
                    self.media,
                    controls::setter(&entity, |this: &mut ItemPage, i| this.media = i),
                ),
            ))
            .child(controls::toggle(
                "item-link",
                "href (link)",
                self.link,
                controls::setter(&entity, |this: &mut ItemPage, on| this.link = on),
            ))
            .child(controls::toggle(
                "item-header",
                "ItemHeader / ItemFooter",
                self.header,
                controls::setter(&entity, |this: &mut ItemPage, on| this.header = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block(
                    "Reference preview",
                    div().max_w(rems(28.75)).child(reference),
                ))
                .child(controls::block(
                    "Live",
                    div().max_w(rems(28.75)).child(live),
                )),
            Some(config.into_any_element()),
        )
    }
}
