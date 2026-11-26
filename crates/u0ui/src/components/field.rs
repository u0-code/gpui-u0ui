//! Field: shadcn `Field` and its parts, composing labels, controls,
//! descriptions and errors with Carbon's form structure (`.cds--form-item`,
//! `.cds--label`, `.cds--form__helper-text`, `.cds--form-requirement`,
//! `.cds--fieldset`) plus the reference `cu-field-*` rules.
//!
//! | Part | Look |
//! | --- | --- |
//! | [`Field`] | a form item; `vertical` stacks, `horizontal` is a row 16px apart with label margins dropped |
//! | [`FieldLabel`] | `.cds--label` (see [`super::Label`]) |
//! | [`FieldDescription`] | `.cds--form__helper-text`: `helper-text-01`, `text-helper`, 4px above |
//! | [`FieldError`] | `.cds--form-requirement` in `text-error`, 4px above; nothing without a message |
//! | [`FieldGroup`] | a column 24px apart (`spacing-06`) |
//! | [`FieldSet`] / [`FieldLegend`] | `.cds--fieldset`; the legend is a 14px semibold `text-primary` label |
//! | [`FieldSeparator`] | a 1px `border-subtle` rule, 8px above and below, with optional centered `label-01` text |
//! | [`FieldContent`] / [`FieldTitle`] | a 4px-gap column / 14px semibold `text-primary` |
//!
//! These are layout and type only; behavior lives in the controls they hold.
//! A `Field`'s `invalid` reaches the [`super::InputGroup`]s inside it, which
//! draw the red outline (`.cu-field[data-invalid] .cu-ig { outline: 2px solid
//! var(--support-error) }`): the field marks its subtree while it lays out
//! (GPUI renders components during layout; see [`in_invalid_field`]). Other
//! controls take their own `invalid`, as the reference CSS only styles `.cu-ig`.
use super::form::{self, rem};
use crate::{
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::cell::Cell;

thread_local! {
    /// How many `invalid` [`Field`]s are laying out their children now.
    static INVALID_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Whether the component rendering now sits inside an `invalid` [`Field`]
/// (`.cu-field[data-invalid] …`). Read it in `render`.
pub fn in_invalid_field() -> bool {
    INVALID_DEPTH.with(|d| d.get() > 0)
}

/// Run `f` marked as inside an invalid field when `invalid`.
pub fn with_invalid_field<R>(invalid: bool, f: impl FnOnce() -> R) -> R {
    if !invalid {
        return f();
    }
    INVALID_DEPTH.with(|d| d.set(d.get() + 1));
    struct Pop;
    impl Drop for Pop {
        fn drop(&mut self) {
            INVALID_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        }
    }
    let _pop = Pop;
    f()
}

/// The element a [`Field`] renders: its body, laid out with the field's
/// `invalid` mark in scope.
pub struct FieldScope {
    invalid: bool,
    child: AnyElement,
}

impl Element for FieldScope {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let child = &mut self.child;
        let id = with_invalid_field(self.invalid, || child.request_layout(window, cx));
        (id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        with_invalid_field(self.invalid, || {
            child.prepaint(window, cx);
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        with_invalid_field(self.invalid, || child.paint(window, cx));
    }
}

macro_rules! parent {
    ($name:ident) => {
        impl ParentElement for $name {
            fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
                self.children.extend(elements);
            }
        }
    };
}

/// `index.d.ts` `FieldProps.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FieldOrientation {
    /// Label above the control (`.cu-field--vertical`).
    #[default]
    Vertical,
    /// Label beside the control (`.cu-field--horizontal`).
    Horizontal,
}

impl FieldOrientation {
    /// Both orientations.
    pub const ALL: [Self; 2] = [Self::Vertical, Self::Horizontal];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// A form field (`.cds--form-item.cu-field`).
#[derive(IntoElement, Default)]
pub struct Field {
    orientation: FieldOrientation,
    invalid: bool,
    children: Vec<AnyElement>,
}

impl Field {
    /// A vertical field.
    pub fn new() -> Self {
        Self::default()
    }

    /// `orientation`.
    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `invalid` (`data-invalid`): exposed for callers; see the module docs.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Whether the field is marked invalid.
    pub fn is_invalid(&self) -> bool {
        self.invalid
    }
}
parent!(Field);

impl RenderOnce for Field {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let body = match self.orientation {
            // `.cds--form-item { display: flex; flex-direction: column; align-items: flex-start }`.
            FieldOrientation::Vertical => form::form_item().children(self.children),
            // `.cu-field--horizontal { flex-direction: row; align-items: center; gap: var(--spacing-05) }`.
            FieldOrientation::Horizontal => div()
                .flex()
                .flex_row()
                .items_center()
                .gap(tokens::SPACING_05)
                .w_full()
                .children(self.children),
        }
        .type_style(TypeStyle::BodyCompact01);
        FieldScope {
            invalid: self.invalid,
            child: body.into_any_element(),
        }
    }
}

/// `FieldLabel` (`.cds--label`).
#[derive(IntoElement)]
pub struct FieldLabel {
    text: SharedString,
    disabled: bool,
    no_margin: bool,
}

impl FieldLabel {
    /// A label.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            disabled: false,
            no_margin: false,
        }
    }

    /// `text-disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Drop the 8px margin (`.cu-field--horizontal .cds--label { margin: 0 }`).
    pub fn no_margin(mut self, no_margin: bool) -> Self {
        self.no_margin = no_margin;
        self
    }
}

impl RenderOnce for FieldLabel {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        form::label(self.text, self.disabled).when(self.no_margin, |this| this.mb_0())
    }
}

/// `FieldDescription` (`.cds--form__helper-text`).
#[derive(IntoElement)]
pub struct FieldDescription {
    text: SharedString,
    disabled: bool,
}

impl FieldDescription {
    /// A description.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            disabled: false,
        }
    }

    /// `text-disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for FieldDescription {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        form::helper_text(self.text, self.disabled)
    }
}

/// `FieldError` (`.cds--form-requirement.cu-field-error`): the message, or
/// the `errors` joined by spaces; renders nothing when both are empty.
#[derive(IntoElement, Default)]
pub struct FieldError {
    text: Option<SharedString>,
    errors: Vec<SharedString>,
}

impl FieldError {
    /// An error with its message.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: Some(text.into()),
            errors: Vec::new(),
        }
    }

    /// An error built from an `errors` list (empty entries are skipped).
    pub fn errors<S: Into<SharedString>>(errors: impl IntoIterator<Item = S>) -> Self {
        Self {
            text: None,
            errors: errors.into_iter().map(Into::into).collect(),
        }
    }

    /// The message shown (`children || errors.filter(Boolean).join(' ')`).
    pub fn message(&self) -> Option<SharedString> {
        if let Some(text) = self.text.clone().filter(|t| !t.is_empty()) {
            return Some(text);
        }
        let joined = self
            .errors
            .iter()
            .filter(|e| !e.is_empty())
            .map(|e| e.as_ref())
            .collect::<Vec<_>>()
            .join(" ");
        (!joined.is_empty()).then(|| joined.into())
    }
}

impl RenderOnce for FieldError {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let message = self.message();
        div().when_some(message, |_, text| {
            form::requirement(&form::Validation::Invalid(text)).unwrap_or_else(div)
        })
    }
}

/// `FieldGroup` (`.cu-field-group`): fields 24px apart.
#[derive(IntoElement, Default)]
pub struct FieldGroup {
    children: Vec<AnyElement>,
}

impl FieldGroup {
    /// A group.
    pub fn new() -> Self {
        Self::default()
    }
}
parent!(FieldGroup);

impl RenderOnce for FieldGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .w_full()
            .children(self.children)
    }
}

/// `FieldSet` (`.cds--fieldset`: no padding, border or margin).
#[derive(IntoElement, Default)]
pub struct FieldSet {
    children: Vec<AnyElement>,
}

impl FieldSet {
    /// A field set.
    pub fn new() -> Self {
        Self::default()
    }
}
parent!(FieldSet);

impl RenderOnce for FieldSet {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_col().w_full().children(self.children)
    }
}

/// `index.d.ts` `FieldLegend.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FieldLegendVariant {
    /// 14px semibold `text-primary` (`.cu-field-legend`).
    #[default]
    Legend,
    /// A plain `.cds--label`.
    Label,
}

/// `FieldLegend` (`legend.cds--label`, `.cu-field-legend`).
#[derive(IntoElement)]
pub struct FieldLegend {
    text: SharedString,
    variant: FieldLegendVariant,
}

impl FieldLegend {
    /// A legend.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            variant: FieldLegendVariant::Legend,
        }
    }

    /// `variant`.
    pub fn variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for FieldLegend {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let label = form::label(self.text, false);
        match self.variant {
            FieldLegendVariant::Label => label,
            // `.cu-field-legend { font-size: 0.875rem; font-weight: 600; color: var(--text-primary) }`.
            FieldLegendVariant::Legend => label
                .type_style(TypeStyle::HeadingCompact01)
                .text_color(color(ColorRole::TextPrimary)),
        }
    }
}

/// `FieldSeparator` (`.cu-field-sep`).
#[derive(IntoElement, Default)]
pub struct FieldSeparator {
    text: Option<SharedString>,
}

impl FieldSeparator {
    /// A plain rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Centered text on the rule (`.cu-field-sep span`).
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }
}

impl RenderOnce for FieldSeparator {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.cu-field-sep { height: 1px; margin: var(--spacing-03) 0; background: var(--border-subtle-01) }`;
        // the span is `label-01` in `text-secondary` on `background`, 8px side padding.
        div()
            .relative()
            .w_full()
            .my(tokens::SPACING_03)
            .h(px(1.))
            .bg(color(ColorRole::BorderSubtle01))
            .when_some(self.text, |this, text| {
                this.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top(rem(-8.))
                        .h(rem(16.))
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .px(tokens::SPACING_03)
                                .bg(color(ColorRole::Background))
                                .type_style(TypeStyle::Label01)
                                .text_color(color(ColorRole::TextSecondary))
                                .child(text),
                        ),
                )
            })
    }
}

/// `FieldContent` (`.cu-field-content`): a 4px-gap column that fills the row.
#[derive(IntoElement, Default)]
pub struct FieldContent {
    children: Vec<AnyElement>,
}

impl FieldContent {
    /// Content.
    pub fn new() -> Self {
        Self::default()
    }
}
parent!(FieldContent);

impl RenderOnce for FieldContent {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(tokens::SPACING_02)
            .children(self.children)
    }
}

/// `FieldTitle` (`.cu-field-title`): 14px semibold `text-primary`.
#[derive(IntoElement)]
pub struct FieldTitle {
    text: SharedString,
}

impl FieldTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for FieldTitle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::HeadingCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .child(self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_mark_nests_and_clears() {
        assert!(!in_invalid_field());
        with_invalid_field(true, || {
            assert!(in_invalid_field());
            // A valid field inside an invalid one stays inside it.
            with_invalid_field(false, || assert!(in_invalid_field()));
        });
        assert!(!in_invalid_field());
        with_invalid_field(false, || assert!(!in_invalid_field()));
    }

    #[test]
    fn field_error_joins_errors_like_the_reference() {
        assert_eq!(
            FieldError::new("Required.").message().as_deref(),
            Some("Required.")
        );
        let joined = FieldError::errors(["Too short.", "", "Use digits."]);
        assert_eq!(joined.message().as_deref(), Some("Too short. Use digits."));
        assert_eq!(FieldError::errors(Vec::<String>::new()).message(), None);
    }
}
