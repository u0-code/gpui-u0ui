//! Attachment: a file or image attachment with upload state (`.cu-att*` in
//! `custom.css`; the Carbon `FileUploaderItem` anatomy).
//!
//! - **Box**: `.cu-att { display: flex; align-items: center; gap: spacing-04;
//!   min-height: size-lg; padding: 0 spacing-02 0 spacing-04; background:
//!   layer-01; max-width: 20rem }`; sizes `sm` (`min-height: size-md`) and `xs`
//!   (`size-sm`, gap `spacing-03`); `vertical` stacks the parts in a 10rem
//!   column (`padding: 0 0 spacing-03; gap: spacing-03`, content and actions
//!   inset 12px, an image media 6rem tall and full width). `error` adds a 2px
//!   inset `support-error` outline.
//! - **Parts**: [`AttachmentMedia`] (a 20px icon in `icon-primary`, `Document`
//!   by default, or a 2rem cover-fit image); [`AttachmentContent`] (`flex: 1;
//!   gap: 2px`) holding [`AttachmentTitle`] (14px, one line with an ellipsis),
//!   [`AttachmentDescription`] (12px `text-secondary`) and [`AttachmentStatus`]
//!   (uploading/processing: Carbon `InlineLoading`, i.e. the small spinner 8px
//!   before `label-02` text in `text-secondary`, with `.cu-att-status
//!   { min-height: 0 }` replacing its 2rem row; error: the 16px `ErrorFilled`
//!   in `support-error` 4px before `text-error` 12px text);
//!   [`AttachmentActions`] / [`AttachmentAction`] (ghost `sm` icon-only
//!   [`Button`], `Close` by default); [`AttachmentTrigger`] (Carbon
//!   `FileUploaderButton`: a ghost `sm` button that opens the platform file
//!   picker and reports the chosen paths); [`AttachmentGroup`] (wrapping row,
//!   8px gaps).
//! - **Kit**: Kit's `attachment` draws its own card, progress and buttons;
//!   none of it matches `FileUploaderItem`, so this composes u0ui parts. The
//!   file picker is GPUI's `prompt_for_paths` (it has no type filter, so the
//!   reference's `accept` is not offered).
//! - No motion besides the spinner.
use super::button::{Button, ButtonSize, ButtonVariant};
use super::form::rem;
use super::spinner::{Spinner, SpinnerSize};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{ColorRole, SIZE_LG, SIZE_MD, SIZE_SM, SPACING_02, SPACING_03, SPACING_04, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, ImageSource, IntoElement, ObjectFit, ParentElement,
    PathPromptOptions, RenderOnce, SharedString, Styled, StyledImage as _, Window, div, img,
    prelude::FluentBuilder as _, px,
};
use std::{path::PathBuf, rc::Rc};

/// `index.d.ts` `AttachmentProps.state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentState {
    /// Not started.
    Idle,
    /// Uploading (busy).
    Uploading,
    /// Processing (busy).
    Processing,
    /// Failed: 2px `support-error` outline.
    Error,
    /// Done (the default).
    #[default]
    Done,
}

impl AttachmentState {
    /// Every state.
    pub const ALL: [AttachmentState; 5] = [
        Self::Idle,
        Self::Uploading,
        Self::Processing,
        Self::Error,
        Self::Done,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Uploading => "uploading",
            Self::Processing => "processing",
            Self::Error => "error",
            Self::Done => "done",
        }
    }

    /// `aria-busy`: uploading or processing.
    pub const fn is_busy(self) -> bool {
        matches!(self, Self::Uploading | Self::Processing)
    }
}

/// `index.d.ts` `AttachmentProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentSize {
    /// `min-height: size-lg` (48).
    #[default]
    Default,
    /// `min-height: size-md` (40).
    Sm,
    /// `min-height: size-sm` (32), gap 8.
    Xs,
}

impl AttachmentSize {
    /// Every size.
    pub const ALL: [AttachmentSize; 3] = [Self::Default, Self::Sm, Self::Xs];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sm => "sm",
            Self::Xs => "xs",
        }
    }

    /// (min-height, gap) in px for a horizontal attachment.
    pub const fn geometry(self) -> (f32, f32) {
        match self {
            Self::Default => (SIZE_LG.px_value(), SPACING_04.px_value()),
            Self::Sm => (SIZE_MD.px_value(), SPACING_04.px_value()),
            Self::Xs => (SIZE_SM.px_value(), SPACING_03.px_value()),
        }
    }
}

/// `index.d.ts` `AttachmentProps.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentOrientation {
    /// A row (default).
    #[default]
    Horizontal,
    /// A 10rem card.
    Vertical,
}

impl AttachmentOrientation {
    /// Both orientations.
    pub const ALL: [AttachmentOrientation; 2] = [Self::Horizontal, Self::Vertical];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// `.cu-att { max-width: 20rem }` in px.
pub const MAX_WIDTH: f32 = 320.;
/// `.cu-att--vertical { width: 10rem }` in px.
pub const VERTICAL_WIDTH: f32 = 160.;
/// `.cu-att--vertical .cu-att-media--image { height: 6rem }` in px.
pub const VERTICAL_IMAGE: f32 = 96.;
/// `.cu-att-media--image { width: 2rem; height: 2rem }` in px.
pub const IMAGE: f32 = 32.;
/// `.cu-att--error { outline: 2px solid var(--support-error); outline-offset: -2px }`.
pub const ERROR_OUTLINE: f32 = 2.;

/// The box fill and the error outline.
pub fn colors(state: AttachmentState) -> (Hsla, Option<Hsla>) {
    (
        color(ColorRole::Layer01),
        (state == AttachmentState::Error).then(|| color(ColorRole::SupportError)),
    )
}

/// `AttachmentMedia`.
pub struct AttachmentMedia {
    icon: Icon,
    image: Option<ImageSource>,
}

impl Default for AttachmentMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl AttachmentMedia {
    /// The default `Document` icon.
    pub fn new() -> Self {
        Self {
            icon: Icon::carbon(CarbonIcon::Document),
            image: None,
        }
    }

    /// Another icon (`icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    /// An image (`variant="image"`, `src`).
    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }

    fn render(self, vertical: bool) -> AnyElement {
        let base = div().flex().flex_none().items_center().justify_center();
        match self.image {
            Some(source) => base
                .overflow_hidden()
                .map(|this| {
                    if vertical {
                        this.w_full().h(rem(VERTICAL_IMAGE))
                    } else {
                        this.size(rem(IMAGE))
                    }
                })
                .child(img(source).size_full().object_fit(ObjectFit::Cover))
                .into_any_element(),
            None => base
                .child(
                    self.icon
                        .size(IconSize::S20)
                        .color(color(ColorRole::IconPrimary)),
                )
                .into_any_element(),
        }
    }
}

/// `AttachmentContent`: title, description and status.
#[derive(IntoElement, Default)]
pub struct AttachmentContent {
    children: Vec<AnyElement>,
}

impl AttachmentContent {
    /// Empty content.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for AttachmentContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AttachmentContent {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_1()
            .flex_col()
            .min_w_0()
            .gap(px(2.))
            .children(self.children)
    }
}

/// `AttachmentTitle`: one line, ellipsis.
#[derive(IntoElement)]
pub struct AttachmentTitle(SharedString);

impl AttachmentTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for AttachmentTitle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .child(self.0)
    }
}

/// `AttachmentDescription`: 12px `text-secondary`.
#[derive(IntoElement)]
pub struct AttachmentDescription(SharedString);

impl AttachmentDescription {
    /// A description.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for AttachmentDescription {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .child(self.0)
    }
}

/// `AttachmentStatus`: renders for uploading, processing and error only.
#[derive(IntoElement)]
pub struct AttachmentStatus {
    id: ElementId,
    state: AttachmentState,
    label: Option<SharedString>,
}

impl AttachmentStatus {
    /// The status for `state`; the spinner is keyed by `id`.
    pub fn new(id: impl Into<ElementId>, state: AttachmentState) -> Self {
        Self {
            id: id.into(),
            state,
            label: None,
        }
    }

    /// The text ("Uploading", "Exceeds 50 MB", …).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for AttachmentStatus {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self.state {
            s if s.is_busy() => div()
                .flex()
                .flex_row()
                .items_center()
                .text_color(color(ColorRole::TextSecondary))
                // `.cds--inline-loading__animation { margin-inline-end: 0.5rem }`.
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .mr(SPACING_03)
                        .child(Spinner::new(self.id).size(SpinnerSize::Sm)),
                )
                .when_some(self.label, |this, text| {
                    this.child(div().type_style(TypeStyle::Label02).child(text))
                })
                .into_any_element(),
            AttachmentState::Error => div()
                .flex()
                .flex_row()
                .items_center()
                .gap(SPACING_02)
                .text_size(rem(12.))
                .text_color(color(ColorRole::TextError))
                .child(
                    Icon::carbon(CarbonIcon::ErrorFilled)
                        .size(IconSize::S16)
                        .color(color(ColorRole::SupportError)),
                )
                .children(self.label)
                .into_any_element(),
            _ => gpui_kit::Empty.into_any_element(),
        }
    }
}

type ClickHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// `AttachmentAction`: a ghost `sm` icon-only button (`Close` by default).
pub struct AttachmentAction {
    id: ElementId,
    label: SharedString,
    icon: Icon,
    on_click: Option<ClickHandler>,
}

impl AttachmentAction {
    /// An action named `label` (its tooltip, e.g. "Remove").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Icon::carbon(CarbonIcon::Close),
            on_click: None,
        }
    }

    /// Another icon.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    /// The handler.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl IntoElement for AttachmentAction {
    type Element = <Button as IntoElement>::Element;

    fn into_element(self) -> Self::Element {
        let on_click = self.on_click;
        Button::new(self.id, self.label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .icon(self.icon)
            .icon_only(true)
            .when_some(on_click, |b, f| {
                b.on_click(move |_, window, cx| f(window, cx))
            })
            .into_element()
    }
}

/// `AttachmentActions`: a row of actions.
#[derive(Default)]
pub struct AttachmentActions {
    children: Vec<AnyElement>,
}

impl AttachmentActions {
    /// No actions yet.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for AttachmentActions {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The attachment. See the module docs.
#[derive(IntoElement, Default)]
pub struct Attachment {
    state: AttachmentState,
    size: AttachmentSize,
    orientation: AttachmentOrientation,
    media: Option<AttachmentMedia>,
    content: Option<AttachmentContent>,
    actions: Option<AttachmentActions>,
}

impl Attachment {
    /// A done, default-size, horizontal attachment.
    pub fn new() -> Self {
        Self::default()
    }

    /// The state.
    pub fn state(mut self, state: AttachmentState) -> Self {
        self.state = state;
        self
    }

    /// The size.
    pub fn size(mut self, size: AttachmentSize) -> Self {
        self.size = size;
        self
    }

    /// The orientation.
    pub fn orientation(mut self, orientation: AttachmentOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The media.
    pub fn media(mut self, media: AttachmentMedia) -> Self {
        self.media = Some(media);
        self
    }

    /// The content.
    pub fn content(mut self, content: AttachmentContent) -> Self {
        self.content = Some(content);
        self
    }

    /// The actions.
    pub fn actions(mut self, actions: AttachmentActions) -> Self {
        self.actions = Some(actions);
        self
    }
}

impl RenderOnce for Attachment {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let vertical = self.orientation == AttachmentOrientation::Vertical;
        let (fill, outline) = colors(self.state);
        let (min_height, gap) = self.size.geometry();
        let inset = |e: gpui_kit::Div| if vertical { e.px(SPACING_04) } else { e };
        div()
            .relative()
            .flex()
            .bg(fill)
            .map(|this| {
                if vertical {
                    this.flex_col()
                        .w(rem(VERTICAL_WIDTH))
                        .pb(SPACING_03)
                        .gap(SPACING_03)
                } else {
                    this.flex_row()
                        .items_center()
                        .max_w(rem(MAX_WIDTH))
                        .gap(rem(gap))
                        .pl(SPACING_04)
                        .pr(SPACING_02)
                }
            })
            .min_h(rem(min_height))
            .children(self.media.map(|m| m.render(vertical)))
            .children(
                self.content
                    .map(|c| inset(div().flex().min_w_0().flex_1().child(c))),
            )
            .children(
                self.actions
                    .map(|a| inset(div().flex().flex_none().children(a.children))),
            )
            .when_some(outline, |this, line| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border(px(ERROR_OUTLINE))
                        .border_color(line),
                )
            })
    }
}

/// `AttachmentGroup`: a wrapping row, 8px gaps.
#[derive(IntoElement, Default)]
pub struct AttachmentGroup {
    children: Vec<AnyElement>,
}

impl AttachmentGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for AttachmentGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AttachmentGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_start()
            .gap(SPACING_03)
            .children(self.children)
    }
}

type FilesHandler = Rc<dyn Fn(Vec<PathBuf>, &mut App)>;

/// `AttachmentTrigger`: a ghost `sm` button that opens the file picker.
pub struct AttachmentTrigger {
    id: ElementId,
    label: SharedString,
    multiple: bool,
    on_files: Option<FilesHandler>,
}

impl AttachmentTrigger {
    /// A trigger labelled `label` (e.g. "Attach file").
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            multiple: false,
            on_files: None,
        }
    }

    /// Allow several files.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Called with the chosen paths (not called when the picker is cancelled).
    pub fn on_files(mut self, f: impl Fn(Vec<PathBuf>, &mut App) + 'static) -> Self {
        self.on_files = Some(Rc::new(f));
        self
    }
}

impl IntoElement for AttachmentTrigger {
    type Element = <Button as IntoElement>::Element;

    fn into_element(self) -> Self::Element {
        let on_files = self.on_files;
        let multiple = self.multiple;
        let prompt = self.label.clone();
        Button::new(self.id, self.label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .on_click(move |_, _, cx| {
                let receiver = cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple,
                    prompt: Some(prompt.clone()),
                });
                let on_files = on_files.clone();
                cx.spawn(async move |cx| {
                    if let Ok(Ok(Some(paths))) = receiver.await
                        && let Some(f) = on_files
                    {
                        cx.update(|cx| f(paths, cx));
                    }
                })
                .detach();
            })
            .into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn geometry_follows_custom_css() {
        assert_eq!(AttachmentSize::Default.geometry(), (48., 12.));
        assert_eq!(AttachmentSize::Sm.geometry(), (40., 12.));
        assert_eq!(AttachmentSize::Xs.geometry(), (32., 8.));
        assert_eq!(
            (MAX_WIDTH, VERTICAL_WIDTH, VERTICAL_IMAGE, IMAGE),
            (320., 160., 96., 32.)
        );
        assert_eq!(ERROR_OUTLINE, 2.);
    }

    #[test]
    fn colors_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            colors(AttachmentState::Done),
            (color(ColorRole::Layer01), None)
        );
        assert_eq!(
            colors(AttachmentState::Error).1,
            Some(color(ColorRole::SupportError))
        );
        assert!(AttachmentState::Uploading.is_busy());
        assert!(!AttachmentState::Error.is_busy());
    }
}
