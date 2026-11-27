//! A component page's body, in md3.io's order: the **Playground** (the live,
//! configurable component on its stage beside the configuration panel), then
//! the **Examples** (the reference `preview.html` content and any matrices).
//! The shell draws the title, the description and the Overview / Specs tabs
//! above it.
//!
//! # Writing a page
//!
//! A page is one file, `pages/<name>.rs`, with a state entity whose `render`
//! returns a [`Body`]:
//!
//! ```ignore
//! pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
//!     cx.new(|_| SwitchPage { checked: true, disabled: false }).into()
//! }
//!
//! impl Render for SwitchPage {
//!     fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         let entity = cx.entity();
//!         Body::new()
//!             .playground(Switch::new("live").checked(self.checked).on_checked_change(
//!                 controls::setter(&entity, |this, on| this.checked = on),
//!             ))
//!             .config(controls::toggle("disabled", "disabled", self.disabled,
//!                 controls::setter(&entity, |this, on| this.disabled = on)))
//!             .example(controls::block("Reference preview", reference()))
//!     }
//! }
//! ```
//!
//! The config panel takes the controls in `crate::controls` (u0ui's own
//! Switch, ToggleGroup and Select): one for every variant, size, state and
//! content option in `index.d.ts`. Pages without a single live component
//! (foundations) return any element instead of a [`Body`].
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;

/// The stage's width before the configuration panel wraps below it.
pub const STAGE_MIN_WIDTH: f32 = 24.;
/// The configuration panel's width.
pub const PANEL_WIDTH: f32 = 20.;

/// A component page's body. See the module docs.
#[derive(IntoElement, Default)]
pub struct Body {
    playground: Option<AnyElement>,
    hint: Option<SharedString>,
    config: Option<AnyElement>,
    stage_min_width: Option<f32>,
    examples: Vec<AnyElement>,
}

impl Body {
    /// An empty body.
    pub fn new() -> Self {
        Self::default()
    }

    /// The live, configurable component (drawn on the stage).
    pub fn playground(mut self, element: impl IntoElement) -> Self {
        self.playground = Some(element.into_any_element());
        self
    }

    /// A line over the stage telling what to try (`↑/↓, Enter`).
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        // Sentence case: the first letter up.
        let hint: SharedString = hint.into();
        let mut chars = hint.chars();
        self.hint = chars
            .next()
            .map(|first| format!("{}{}", first.to_uppercase(), chars.as_str()).into());
        self
    }

    /// A wider stage minimum (rems, padding included) for a component whose
    /// own minimum row is wider than [`STAGE_MIN_WIDTH`] leaves room for: the
    /// panel wraps below the stage sooner instead of the component overflowing.
    pub fn stage_min_width(mut self, rems: f32) -> Self {
        self.stage_min_width = Some(rems.max(STAGE_MIN_WIDTH));
        self
    }

    /// The configuration panel's controls.
    pub fn config(mut self, controls: impl IntoElement) -> Self {
        self.config = Some(controls.into_any_element());
        self
    }

    /// One example, usually a [`crate::controls::block`] with its own heading.
    pub fn example(mut self, element: impl IntoElement) -> Self {
        self.examples.push(element.into_any_element());
        self
    }
}

/// The stage and the panel, side by side when there is room, stacked
/// otherwise (no horizontal scrolling).
fn playground(
    stage: AnyElement,
    hint: Option<SharedString>,
    config: Option<AnyElement>,
    stage_min_width: f32,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(
            // The minimum is a flex basis, not a min-width: past it the panel
            // wraps below, and a row narrower than it (a small window) shrinks
            // the stage instead of scrolling sideways.
            div()
                .flex_basis(rems(stage_min_width))
                .flex_grow(1000.)
                .flex_shrink(1.)
                .min_w_0()
                .min_h(rems(16.))
                .p(tokens::SPACING_07)
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .bg(color(ColorRole::Background))
                .when_some(hint, |this, hint| {
                    this.child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(tokens::SPACING_03)
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(
                                Icon::named("touch_app")
                                    .size(IconSize::S16)
                                    .color(color(ColorRole::IconSecondary)),
                            )
                            .child(hint),
                    )
                })
                // Top-aligned: a component that grows (a Textarea dragged by
                // its grip, an opened accordion) keeps its top edge still, so a
                // resize tracks the pointer instead of moving by half of it.
                .child(div().flex_1().flex().flex_col().child(stage)),
        )
        .when_some(config, |this, config| {
            this.child(
                // The panel is a `layer-01` surface: its controls sit on layer two.
                // It takes its 20rem beside the stage, and the full width
                // once it wraps below it.
                div()
                    .flex_basis(rems(PANEL_WIDTH))
                    .flex_grow(1.)
                    .bg(color(ColorRole::Layer01))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(tokens::SPACING_05)
                            .h(rems(3.))
                            .flex()
                            .items_center()
                            .border_b_1()
                            .border_color(color(ColorRole::BorderSubtle01))
                            .type_style(TypeStyle::HeadingCompact01)
                            .text_color(color(ColorRole::TextPrimary))
                            .child("Configuration"),
                    )
                    .child(
                        div()
                            .p(tokens::SPACING_05)
                            .flex()
                            .flex_col()
                            .gap(tokens::SPACING_06)
                            .child(config),
                    ),
            )
        })
}

impl RenderOnce for Body {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let has_examples = !self.examples.is_empty();
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_09)
            .when_some(self.playground, |this, stage| {
                this.child(playground(
                    stage,
                    self.hint,
                    self.config,
                    self.stage_min_width.unwrap_or(STAGE_MIN_WIDTH),
                ))
            })
            .when(has_examples, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(tokens::SPACING_07)
                        .child(
                            div()
                                .type_style(TypeStyle::Heading03)
                                .text_color(color(ColorRole::TextPrimary))
                                .child("Examples"),
                        )
                        .children(self.examples),
                )
            })
    }
}
