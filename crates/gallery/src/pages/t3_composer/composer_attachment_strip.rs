//! T3 · Composer › ComposerAttachmentStrip: a done, an uploading and a
//! failed image, a PDF and a JSON file
//! (`components/ComposerAttachmentStrip/preview.html`). The playground's
//! strip is live: × removes, Retry restarts the failed upload, and the
//! composer variant shows the 32px family.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rgb,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{
    AttachmentFile, AttachmentImage, AttachmentImageState, ComposerAttachmentStrip, ContextChipKind,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StripPage {
        images: images(),
        files: files(),
        in_composer: false,
        last: "".into(),
    })
    .into()
}

struct StripPage {
    images: Vec<AttachmentImage>,
    files: Vec<AttachmentFile>,
    in_composer: bool,
    last: SharedString,
}

/// The preview's images.
fn images() -> Vec<AttachmentImage> {
    vec![
        AttachmentImage::new("a.png").color(rgb(0x0f62fe)),
        AttachmentImage::new("b.png")
            .color(rgb(0x8a3ffc))
            .state(AttachmentImageState::Uploading),
        AttachmentImage::new("c.png")
            .color(rgb(0x007d79))
            .state(AttachmentImageState::Error),
    ]
}

/// The preview's files.
fn files() -> Vec<AttachmentFile> {
    vec![
        AttachmentFile::new("design-spec.pdf")
            .kind(ContextChipKind::DocPdf)
            .size("2.4 MB"),
        AttachmentFile::new("tokens.json").size("60 KB"),
    ]
}

impl Render for StripPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let strip =
            ComposerAttachmentStrip::new("strip-live", self.images.clone(), self.files.clone())
                .in_composer(self.in_composer)
                .on_remove({
                    let entity = entity.clone();
                    move |name, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.images.retain(|i| i.name != name);
                            this.files.retain(|f| f.name != name);
                            this.last = format!("onRemove({name})").into();
                            cx.notify();
                        })
                    }
                })
                .on_retry({
                    let entity = entity.clone();
                    move |name, _, cx| {
                        entity.update(cx, |this, cx| {
                            for image in this.images.iter_mut().filter(|i| i.name == name) {
                                image.state = AttachmentImageState::Uploading;
                            }
                            this.last = format!("onRetry({name})").into();
                            cx.notify();
                        })
                    }
                });
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(strip)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "strip-composer",
                "In the composer",
                self.in_composer,
                controls::setter(&entity, |this: &mut Self, on| this.in_composer = on),
            ))
            .child(
                Button::new("strip-reset", "Reset attachments")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.images = images();
                                this.files = files();
                                this.last = "".into();
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                ComposerAttachmentStrip::new("strip-ref", images(), files()),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(ComposerAttachmentStrip::new(
            "strip-th",
            images().into_iter().take(2).collect(),
            Vec::new(),
        ))
        .into_any_element()
}
