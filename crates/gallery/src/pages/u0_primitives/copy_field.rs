//! u0 · Primitives › CopyField: a live copy field (mono, masked, open
//! link) plus the reference webhook and tunnel fields.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::CopyField;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CopyFieldPage {
        mono: true,
        masked: false,
        dashed: false,
        href: true,
        opened: 0,
    })
    .into()
}

struct CopyFieldPage {
    mono: bool,
    masked: bool,
    dashed: bool,
    href: bool,
    opened: usize,
}

/// `components/CopyField/preview.html`: 420px wide, 16px apart.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .w(rems(26.25))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            CopyField::new((prefix, 0usize), "https://u0.local/hooks/7f3a9c2e41b0")
                .label("Webhook URL"),
        )
        .child(
            CopyField::new(
                (prefix, 1usize),
                "https://quiet-river-1234.trycloudflare.com",
            )
            .label("Tunnel address")
            .on_open(|_, _, _| {}),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            CopyField::new("thumb-copy", "https://u0.local/hooks/7f3a9c2e41b0")
                .label("Webhook URL"),
        )
        .into_any_element()
}

impl Render for CopyFieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut field = CopyField::new("copy-live", "https://quiet-river-1234.trycloudflare.com")
            .label("Tunnel address")
            .mono(self.mono)
            .masked(self.masked)
            .dashed(self.dashed);
        if self.href {
            let e = entity.clone();
            field = field.on_open(move |_, _, cx| {
                e.update(cx, |this, cx| {
                    this.opened += 1;
                    cx.notify();
                })
            });
        }
        let live = div()
            .w(rems(26.25))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(field)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("Open pressed {} times", self.opened)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "copy-mono",
                "mono",
                self.mono,
                controls::setter(&entity, |this: &mut Self, on| this.mono = on),
            ))
            .child(controls::toggle(
                "copy-masked",
                "masked",
                self.masked,
                controls::setter(&entity, |this: &mut Self, on| this.masked = on),
            ))
            .child(controls::toggle(
                "copy-dashed",
                "dashed",
                self.dashed,
                controls::setter(&entity, |this: &mut Self, on| this.dashed = on),
            ))
            .child(controls::toggle(
                "copy-href",
                "href (open button)",
                self.href,
                controls::setter(&entity, |this: &mut Self, on| this.href = on),
            ));
        Body::new()
            .playground(live)
            .hint("Copy turns into a checkmark for 1.5s")
            .config(config)
            .example(controls::block("Reference preview", reference("copy-ref")))
    }
}
