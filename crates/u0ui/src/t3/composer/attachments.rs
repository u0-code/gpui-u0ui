//! [`ComposerAttachmentStrip`] (`.t3-attstrip`, "Attachments" in `t3.css`):
//! image thumbnails and file cards above the composer input.
//!
//! - **Thumbnail** (`.t3-thumb`): 4rem square on `layer-accent-01`, the image
//!   cover-fit or a colored placeholder with a 20px image glyph; while
//!   uploading a 50% scrim holds a white [`Spin`]; the remove × sits in the
//!   corner on `background-inverse` (always shown, d-fix).
//! - **Failed upload** (`.t3-attfail`, r2-gal): a 32px readable chip with the
//!   error icon, name, message, Retry and ×, on
//!   `notification-background-error` framed by `support-error`.
//! - **Files**: [`ContextChip`] bubbles with their size.
//! - **Opening** ([`ComposerAttachmentStrip::on_open`]): a done image's
//!   thumbnail is a button that opens it, named by `texts.open`.
//! - **In the composer** ([`ComposerAttachmentStrip::in_composer`], gemini-A
//!   and r2-gal): 2rem thumbs with a hairline, the × hung off the corner,
//!   a 55% upload scrim, and 32px file chips.
use super::chip::{ContextChip, ContextChipKind, ContextChipShape};
use crate::t3::base::Spin;
use crate::t3::press::{self, rem};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, BoxShadow, CursorStyle, ElementId, Hsla, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledImage as _, Window, div, img,
    prelude::FluentBuilder as _, px, rgba,
};
use std::rc::Rc;

/// An image's upload state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentImageState {
    /// Uploaded.
    #[default]
    Done,
    /// Uploading: scrim and spinner.
    Uploading,
    /// Failed: the readable error chip.
    Error,
}

/// An image attachment (`{ name, src | color, state, error, onRetry }`).
#[derive(Clone)]
pub struct AttachmentImage {
    /// `name`: the key, tooltip and remove label.
    pub name: SharedString,
    /// `src`.
    pub source: Option<ImageSource>,
    /// `color`: the placeholder fill without `src`.
    pub color: Option<Hsla>,
    /// `state`.
    pub state: AttachmentImageState,
    /// `error`: the failure message (default "Upload failed").
    pub error: Option<SharedString>,
}

impl AttachmentImage {
    /// An image named `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            source: None,
            color: None,
            state: AttachmentImageState::Done,
            error: None,
        }
    }

    /// `src`.
    pub fn source(mut self, source: impl Into<ImageSource>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// `color`: placeholder fill.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// `state`.
    pub fn state(mut self, state: AttachmentImageState) -> Self {
        self.state = state;
        self
    }

    /// `error`.
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }
}

/// A file attachment (`{ name, kind, size }`).
#[derive(Clone)]
pub struct AttachmentFile {
    /// `name`.
    pub name: SharedString,
    /// `kind` (default `file`).
    pub kind: ContextChipKind,
    /// `size`, shown as the chip's meta.
    pub size: Option<SharedString>,
}

impl AttachmentFile {
    /// A file named `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            kind: ContextChipKind::File,
            size: None,
        }
    }

    /// `kind`.
    pub fn kind(mut self, kind: ContextChipKind) -> Self {
        self.kind = kind;
        self
    }

    /// `size`.
    pub fn size(mut self, size: impl Into<SharedString>) -> Self {
        self.size = Some(size.into());
        self
    }
}

/// The strip's strings (the reference's English defaults).
#[derive(Clone)]
pub struct AttachmentTexts {
    /// The failed chip's message without `error`.
    pub upload_failed: SharedString,
    /// Retry.
    pub retry: SharedString,
    /// The spinner's hidden label.
    pub uploading: SharedString,
    /// The remove ×'s name: `"Remove {name}"`.
    pub remove: Rc<dyn Fn(&str) -> SharedString>,
    /// An openable thumbnail's name: `"Open {name}"`.
    pub open: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for AttachmentTexts {
    fn default() -> Self {
        Self {
            upload_failed: "Upload failed".into(),
            retry: "Retry".into(),
            uploading: "Uploading".into(),
            remove: Rc::new(|name| format!("Remove {name}").into()),
            open: Rc::new(|name| format!("Open {name}").into()),
        }
    }
}

/// Thumbnail geometry in px: (side, × offset from the corner, × padding).
pub const fn thumb_geometry(in_composer: bool) -> (f32, f32, f32) {
    if in_composer {
        // `.t3-composer-ctx .t3-thumb { width: 2rem; height: 2rem }`,
        // `.t3-thumb-x { top: -6px; right: -6px; padding: 1px }` (r2-gal).
        (32., -6., 1.)
    } else {
        // `.t3-thumb { width: 4rem; height: 4rem }`, `.t3-thumb-x { top: 2px; right: 2px; padding: 2px }`.
        (64., 2., 2.)
    }
}

/// The upload scrim: `rgba(22, 22, 22, 0.5)`, `.55` in the composer.
pub fn upload_scrim(in_composer: bool) -> Hsla {
    rgba(if in_composer { 0x1616168c } else { 0x16161680 }).into()
}

type NameHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The attachment strip. See the module docs.
#[derive(IntoElement)]
pub struct ComposerAttachmentStrip {
    id: ElementId,
    images: Vec<AttachmentImage>,
    files: Vec<AttachmentFile>,
    in_composer: bool,
    texts: AttachmentTexts,
    on_remove: Option<NameHandler>,
    on_retry: Option<NameHandler>,
    on_open: Option<NameHandler>,
}

impl ComposerAttachmentStrip {
    /// A strip of `images` then `files`.
    pub fn new(
        id: impl Into<ElementId>,
        images: Vec<AttachmentImage>,
        files: Vec<AttachmentFile>,
    ) -> Self {
        Self {
            id: id.into(),
            images,
            files,
            in_composer: false,
            texts: AttachmentTexts::default(),
            on_remove: None,
            on_retry: None,
            on_open: None,
        }
    }

    /// Drawn inside the composer's context row (`.t3-composer-ctx`).
    pub fn in_composer(mut self, in_composer: bool) -> Self {
        self.in_composer = in_composer;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: AttachmentTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onRemove(name)`: adds the × to files (images always have one).
    pub fn on_remove(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }

    /// A done image's thumbnail opened (`onOpen(name)`): the thumbnail is a
    /// button, the × above it still removes.
    pub fn on_open(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// A failed image's Retry (`image.onRetry(name)`).
    pub fn on_retry(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_retry = Some(Rc::new(f));
        self
    }
}

/// How a bare button fills: on hover and/or under keyboard focus.
#[derive(Clone, Copy)]
struct Fill {
    hover: Option<Hsla>,
    focus: Option<Hsla>,
}

/// A bare button with its fills and the focus ring; returns it and whether
/// the pointer is over it.
fn bare_button(
    id: ElementId,
    label: SharedString,
    fill: Fill,
    content: impl FnOnce(bool) -> AnyElement,
    on_click: Option<press::Handler>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::base::Button {
    let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
    let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
    let bg = if ring {
        fill.focus.or(fill.hover.filter(|_| hovered))
    } else {
        fill.hover.filter(|_| hovered)
    };
    gpui_kit::base::Button::new(id)
        .track_focus(&focus)
        .accessibility_label(label)
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .cursor(CursorStyle::PointingHand)
        .on_hover(press::track_hover(hover))
        .when_some(bg, |this, bg| this.bg(bg))
        .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
        .child(content(hovered))
        .when(ring, |this| this.child(press::ring()))
}

impl RenderOnce for ComposerAttachmentStrip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let texts = self.texts.clone();
        let (side, x_at, x_pad) = thumb_geometry(self.in_composer);
        let mut children: Vec<AnyElement> = Vec::new();
        for (i, image) in self.images.into_iter().enumerate() {
            let key = |part: &str| -> ElementId {
                (id.clone(), SharedString::from(format!("{part}-{i}"))).into()
            };
            let remove_name = (texts.remove)(&image.name);
            let remove = self.on_remove.clone().map(|f| {
                let name = image.name.clone();
                Rc::new(move |w: &mut Window, cx: &mut App| f(name.clone(), w, cx))
                    as Rc<dyn Fn(&mut Window, &mut App)>
            });
            if image.state == AttachmentImageState::Error {
                // `.t3-attfail { display: inline-flex; align-items: center; gap: spacing-03; height: 2rem;
                // padding: 0 spacing-02 0 spacing-03; background: notification-background-error;
                // box-shadow: inset 0 0 0 1px support-error; font-size: 0.875rem; white-space: nowrap }`.
                let retry = self.on_retry.clone().map(|f| {
                    let name = image.name.clone();
                    Rc::new(move |w: &mut Window, cx: &mut App| f(name.clone(), w, cx))
                        as Rc<dyn Fn(&mut Window, &mut App)>
                });
                children.push(
                    div()
                        .flex()
                        .flex_row()
                        .flex_none()
                        .items_center()
                        .gap(rem(8.))
                        .h(rem(32.))
                        .pl(rem(8.))
                        .pr(rem(4.))
                        .bg(color(ColorRole::NotificationBackgroundError))
                        .shadow(press::inset_frame(color(ColorRole::SupportError)))
                        .text_size(rem(14.))
                        .line_height(rem(18.))
                        .whitespace_nowrap()
                        .child(press::icon(
                            CarbonIcon::ErrorFilled,
                            16.,
                            color(ColorRole::SupportError),
                        ))
                        // `.t3-attfail-name { color: text-primary }`.
                        .child(
                            div()
                                .text_color(color(ColorRole::TextPrimary))
                                .child(image.name.clone()),
                        )
                        // `.t3-attfail-msg { color: text-error; font-size: 0.75rem }`.
                        .child(
                            div()
                                .text_size(rem(12.))
                                .text_color(color(ColorRole::TextError))
                                .child(image.error.clone().unwrap_or(texts.upload_failed.clone())),
                        )
                        // `.t3-attfail-retry { padding: 0 spacing-02; height: 1.5rem; color: link-primary;
                        // font-size: 0.875rem }`, `:hover { text-decoration: underline }`,
                        // `:focus-visible { background: layer-hover-01; outline: 2px solid focus }`.
                        .child(
                            bare_button(
                                key("retry"),
                                texts.retry.clone(),
                                Fill {
                                    hover: None,
                                    focus: Some(color(ColorRole::LayerHover01)),
                                },
                                |hovered| {
                                    div()
                                        .text_color(color(ColorRole::LinkPrimary))
                                        .when(hovered, |this| this.underline())
                                        .child(texts.retry.clone())
                                        .into_any_element()
                                },
                                retry,
                                window,
                                cx,
                            )
                            .h(rem(24.))
                            .px(rem(4.)),
                        )
                        // `.t3-attfail-x { width: 1.5rem; height: 1.5rem; color: icon-primary }`,
                        // `:hover { background: layer-hover-01 }`.
                        .child(
                            bare_button(
                                key("x"),
                                remove_name,
                                Fill {
                                    hover: Some(color(ColorRole::LayerHover01)),
                                    focus: Some(color(ColorRole::LayerHover01)),
                                },
                                |_| {
                                    press::icon(
                                        CarbonIcon::Close,
                                        12.,
                                        color(ColorRole::IconPrimary),
                                    )
                                    .into_any_element()
                                },
                                remove,
                                window,
                                cx,
                            )
                            .size(rem(24.)),
                        )
                        .into_any_element(),
                );
                continue;
            }
            let uploading = image.state == AttachmentImageState::Uploading;
            let open = self.on_open.clone().filter(|_| !uploading).map(|f| {
                let name = image.name.clone();
                Rc::new(move |w: &mut Window, cx: &mut App| f(name.clone(), w, cx))
                    as Rc<dyn Fn(&mut Window, &mut App)>
            });
            let picture: AnyElement = match image.source.clone() {
                Some(source) => img(source)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .into_any_element(),
                // `.t3-thumb-ph { background: <color> }`, `svg { fill: text-on-color; opacity: .8 }`.
                None => div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when_some(image.color, |this, c| this.bg(c))
                    .child(press::icon(
                        CarbonIcon::Image,
                        20.,
                        color(ColorRole::TextOnColor).opacity(0.8),
                    ))
                    .into_any_element(),
            };
            // `.t3-thumb-x { padding: 2px; background: background-inverse; color: text-inverse }`;
            // in the composer `box-shadow: 0 0 0 1px layer-01`.
            let x = bare_button(
                key("thumb-x"),
                remove_name,
                Fill {
                    hover: None,
                    focus: None,
                },
                |_| {
                    press::icon(CarbonIcon::Close, 12., color(ColorRole::TextInverse))
                        .into_any_element()
                },
                remove,
                window,
                cx,
            )
            .p(px(x_pad))
            .bg(color(ColorRole::BackgroundInverse))
            .when(self.in_composer, |this| {
                this.shadow(vec![
                    BoxShadow::new(px(0.), px(0.), color(ColorRole::Layer01)).spread_radius(px(1.)),
                ])
            });
            children.push(
                div()
                    .relative()
                    .flex_none()
                    .size(rem(side))
                    .bg(color(ColorRole::LayerAccent01))
                    // `.t3-thumb { overflow: hidden }`; the composer's thumb lets the × hang out.
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .overflow_hidden()
                            .child(match open {
                                Some(f) => bare_button(
                                    key("open"),
                                    (texts.open)(&image.name),
                                    Fill {
                                        hover: None,
                                        focus: None,
                                    },
                                    |_| picture,
                                    Some(f),
                                    window,
                                    cx,
                                )
                                .size_full()
                                .into_any_element(),
                                None => picture,
                            })
                            .when(uploading, |this| {
                                // `.t3-thumb-over { background: rgba(22,22,22,.5) }` with a white spinner
                                // on a 30% white track (`.t3-thumb-over .t3-spin`).
                                this.child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(upload_scrim(self.in_composer))
                                        .child(
                                            Spin::new(key("spin"))
                                                .label(texts.uploading.clone())
                                                .color(gpui_kit::white())
                                                .track(gpui_kit::white().opacity(0.3)),
                                        ),
                                )
                            }),
                    )
                    // `.t3-composer-ctx .t3-thumb { box-shadow: inset 0 0 0 1px border-subtle-01 }`.
                    .when(self.in_composer, |this| {
                        this.child(
                            div()
                                .absolute()
                                .inset_0()
                                .border_1()
                                .border_color(color(ColorRole::BorderSubtle01)),
                        )
                    })
                    // The × blocks the pointer, so pressing it never opens the thumbnail under it.
                    .child(
                        div()
                            .absolute()
                            .top(px(x_at))
                            .right(px(x_at))
                            .occlude()
                            .child(x),
                    )
                    .into_any_element(),
            );
        }
        for (i, file) in self.files.into_iter().enumerate() {
            let mut chip = ContextChip::new(
                (id.clone(), SharedString::from(format!("file-{i}"))),
                file.kind,
                file.name.clone(),
            )
            .shape(ContextChipShape::Bubble)
            .in_composer(self.in_composer)
            .remove_label((texts.remove)(&file.name));
            if let Some(size) = file.size {
                chip = chip.meta(size);
            }
            // `onRemove={() => onRemove && onRemove(f.name)}`: every file chip
            // has its ×, with or without a handler (as the image thumbs do).
            let remove = self.on_remove.clone();
            let name = file.name.clone();
            chip = chip.on_remove(move |w, cx| {
                if let Some(f) = remove.as_ref() {
                    f(name.clone(), w, cx)
                }
            });
            children.push(chip.into_any_element());
        }
        // `.t3-attstrip { display: flex; flex-wrap: wrap; gap: spacing-03 }`; in the composer
        // `align-items: center`.
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(rem(8.))
            .when(self.in_composer, |this| this.items_center())
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_thumbs_are_32px_with_a_hanging_x() {
        assert_eq!(thumb_geometry(false), (64., 2., 2.));
        assert_eq!(thumb_geometry(true), (32., -6., 1.));
    }

    /// What a strip's handlers were called with, in order.
    struct Strip {
        heard: Rc<std::cell::RefCell<Vec<String>>>,
    }

    impl gpui_kit::Render for Strip {
        fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
            let (opened, removed) = (self.heard.clone(), self.heard.clone());
            div().size_full().child(
                ComposerAttachmentStrip::new("strip", vec![AttachmentImage::new("a.png")], vec![])
                    .on_open(move |name, _, _| opened.borrow_mut().push(format!("open {name}")))
                    .on_remove(move |name, _, _| {
                        removed.borrow_mut().push(format!("remove {name}"));
                    }),
            )
        }
    }

    #[gpui_kit::test]
    fn a_thumbnail_opens_and_its_x_only_removes(cx: &mut gpui_kit::TestAppContext) {
        let heard = Rc::new(std::cell::RefCell::new(Vec::new()));
        let (_, window) = cx.add_window_view({
            let heard = heard.clone();
            |_, _| Strip { heard }
        });
        // The 64px thumb at the origin; its × 2px in from the top right corner.
        window.simulate_click(
            gpui_kit::point(px(16.), px(48.)),
            gpui_kit::Modifiers::none(),
        );
        window.simulate_click(
            gpui_kit::point(px(56.), px(8.)),
            gpui_kit::Modifiers::none(),
        );
        assert_eq!(*heard.borrow(), ["open a.png", "remove a.png"]);
    }

    #[test]
    fn upload_scrims_follow_the_css() {
        assert!((upload_scrim(false).a - 0.5).abs() < 0.01);
        assert!((upload_scrim(true).a - 0.55).abs() < 0.01);
    }
}
