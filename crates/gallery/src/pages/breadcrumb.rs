//! Navigation › Breadcrumb: the reference preview and a configurable live
//! breadcrumb that navigates.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::components::{Breadcrumb, BreadcrumbItem, BreadcrumbSize};
use u0ui::prelude::*;

const PATH: [&str; 4] = ["Resources", "Storage", "Buckets", "logs-2026"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| BreadcrumbPage {
        depth: PATH.len(),
        size: 1,
        trailing: false,
    })
    .into()
}

struct BreadcrumbPage {
    /// How many path steps are shown (the last is current).
    depth: usize,
    size: usize,
    trailing: bool,
}

impl Render for BreadcrumbPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/Breadcrumb/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                Breadcrumb::new("ref-breadcrumb")
                    .item(BreadcrumbItem::new("Resources"))
                    .item(BreadcrumbItem::new("Storage"))
                    .item(BreadcrumbItem::new("logs-2026").current(true)),
            );

        let mut live_crumb = Breadcrumb::new("live-breadcrumb")
            .size(BreadcrumbSize::ALL[self.size])
            .no_trailing_slash(!self.trailing);
        for (index, label) in PATH.iter().take(self.depth).enumerate() {
            let entity = entity.clone();
            live_crumb = live_crumb.item(
                BreadcrumbItem::new(*label)
                    .current(index + 1 == self.depth)
                    .on_click(move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.depth = index + 1;
                            cx.notify();
                        })
                    }),
            );
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(live_crumb)
            .child(
                div()
                    .type_style(TypeStyle::Heading04)
                    .child(PATH[self.depth - 1]),
            )
            .child(
                Button::new("breadcrumb-reset", "Reset path")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.depth = PATH.len();
                                cx.notify();
                            })
                        }
                    }),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "bc-size",
                    &BreadcrumbSize::ALL.map(|s| s.name()),
                    self.size,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.size = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "bc-trailing",
                "trailing slash (noTrailingSlash off)",
                self.trailing,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.trailing = on;
                        cx.notify();
                    })
                },
            ));

        Body::new()
            .playground(live)
            .hint("click an ancestor to go up")
            .config(config)
            .example(reference)
    }
}
