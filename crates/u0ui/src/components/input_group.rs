//! InputGroup: shadcn `InputGroup`, addons (icons, text, buttons) around an
//! input or text area. Carbon has no such component; the reference draws the
//! Carbon field anatomy around it (`.cu-ig` in `src/custom.css`).
//!
//! - **Behavior**: the text is Kit's [`InputState`] / [`TextareaState`]
//!   (editing, IME, selection, clipboard); buttons are [`Button`]s.
//! - **Appearance**: `field` fill, 1px `border-strong` bottom rule, hover
//!   `field-hover`, `:focus-within` 2px inset `focus` outline, `invalid` 2px
//!   `support-error` outline. Addons sit by `align`: `inline-start` (16px in,
//!   the input then starts 8px after it), `inline-end` (4px in),
//!   `block-start` (a full-width row above, `8px 16px 0`) and `block-end` (a
//!   full-width row below with a `border-subtle` rule, `4px 8px 4px 16px`,
//!   spread apart). Addon icons are `icon-secondary`; [`InputGroupText`] is
//!   14px `text-secondary`.
//! - **Motion**: background 70ms.
//!
//! ```ignore
//! InputGroup::new("search")
//!     .addon(InputGroupAddon::new().child(Icon::carbon(CarbonIcon::Search)))
//!     .input(InputGroupInput::new(&state).placeholder("Search resources"))
//! ```
use super::button::{Button, ButtonSize, ButtonVariant};
use super::form::{self, FieldSize, rem};
use super::input::{FIELD_MOTION, InputState};
use super::textarea::TextareaState;
use crate::{
    motion,
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _, Size,
        input::{Input as KitInput, Textarea as KitTextarea},
    },
    div,
    prelude::FluentBuilder as _,
};

/// `index.d.ts` `InputGroupAddonProps.align`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AddonAlign {
    /// Before the input (`order: -1; padding-left: var(--spacing-05)`).
    #[default]
    InlineStart,
    /// After the input (`order: 2; padding-right: var(--spacing-02)`).
    InlineEnd,
    /// A row above (`order: -2; flex-basis: 100%`).
    BlockStart,
    /// A row below with a rule (`order: 3; border-top: 1px solid var(--border-subtle)`).
    BlockEnd,
}

impl AddonAlign {
    /// Every alignment.
    pub const ALL: [Self; 4] = [
        Self::InlineStart,
        Self::InlineEnd,
        Self::BlockStart,
        Self::BlockEnd,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
            Self::BlockStart => "block-start",
            Self::BlockEnd => "block-end",
        }
    }

    /// CSS `order` relative to the input (0).
    pub const fn order(self) -> i32 {
        match self {
            Self::BlockStart => -2,
            Self::InlineStart => -1,
            Self::InlineEnd => 2,
            Self::BlockEnd => 3,
        }
    }

    /// Padding (top, right, bottom, left) in px (`.cu-ig-addon--*`).
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::InlineStart => [0., 0., 0., 16.],
            Self::InlineEnd => [0., 4., 0., 0.],
            Self::BlockStart => [8., 16., 0., 16.],
            Self::BlockEnd => [4., 8., 4., 16.],
        }
    }
}

/// The text area's block padding, `0.6875rem` above and below, less Kit's
/// 2px text inset on each side.
const TEXTAREA_PADDING: f32 = 18.;

/// The `.cu-ig-textarea` height for `rows` lines (`padding-block: 0.6875rem`,
/// 20px lines); also its smallest dragged height at one row.
pub fn textarea_height(rows: usize) -> f32 {
    rows.max(1) as f32 * 20. + 22.
}

/// The single-line group height: `size-md` (40px) in every density.
pub fn group_height() -> f32 {
    FieldSize::Md.carbon_height()
}

/// An addon (`InputGroupAddon`).
pub struct InputGroupAddon {
    align: AddonAlign,
    children: Vec<AnyElement>,
}

impl InputGroupAddon {
    /// An `inline-start` addon.
    pub fn new() -> Self {
        Self {
            align: AddonAlign::InlineStart,
            children: Vec::new(),
        }
    }

    /// `align`.
    pub fn align(mut self, align: AddonAlign) -> Self {
        self.align = align;
        self
    }
}

impl Default for InputGroupAddon {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for InputGroupAddon {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// `InputGroupText` (`.cu-ig-text`): 14px `text-secondary`.
#[derive(IntoElement)]
pub struct InputGroupText {
    text: SharedString,
}

impl InputGroupText {
    /// A text addon part.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for InputGroupText {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextSecondary))
            .whitespace_nowrap()
            .child(self.text)
    }
}

/// `InputGroupButton`: a [`Button`] that defaults to `ghost`, size `sm`.
pub fn input_group_button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
}

/// `InputGroupInput`: the single-line editor (`.cu-ig-input`).
pub struct InputGroupInput {
    state: Entity<InputState>,
    placeholder: Option<SharedString>,
}

impl InputGroupInput {
    /// An input over the caller's `state`.
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            placeholder: None,
        }
    }

    /// Placeholder in `text-placeholder`.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }
}

/// `InputGroupTextarea`: the multi-line editor (`.cu-ig-textarea`, 3 rows).
pub struct InputGroupTextarea {
    state: Entity<TextareaState>,
    placeholder: Option<SharedString>,
    rows: usize,
}

impl InputGroupTextarea {
    /// A text area over the caller's `state`.
    pub fn new(state: &Entity<TextareaState>) -> Self {
        Self {
            state: state.clone(),
            placeholder: None,
            rows: 3,
        }
    }

    /// Placeholder in `text-placeholder`.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// `rows` (default 3).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }
}

enum Editor {
    Input(InputGroupInput),
    Textarea(InputGroupTextarea),
}

/// The colors of the group frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupLook {
    /// Fill (`field`, hover `field-hover`).
    pub background: Hsla,
    /// Bottom rule (`border-strong`).
    pub border: Hsla,
    /// Outline: `focus` within, else `support-error` when invalid.
    pub outline: Option<Hsla>,
}

impl GroupLook {
    /// The frame look (`.cu-ig*`, `.cu-ig[data-invalid]`).
    pub fn of(
        level: LayerLevel,
        hovered: bool,
        focused: bool,
        invalid: bool,
        disabled: bool,
    ) -> Self {
        let background = if hovered && !disabled {
            color(level.field_hover())
        } else {
            color(level.field())
        };
        let outline = if disabled {
            None
        } else if focused {
            Some(color(ColorRole::Focus))
        } else if invalid {
            Some(color(ColorRole::SupportError))
        } else {
            None
        };
        Self {
            background,
            border: color(level.border_strong()),
            outline,
        }
    }
}

/// A Carbon-field input group. See the module docs.
#[derive(IntoElement)]
pub struct InputGroup {
    id: ElementId,
    invalid: bool,
    disabled: bool,
    level: LayerLevel,
    addons: Vec<InputGroupAddon>,
    editor: Option<Editor>,
}

impl InputGroup {
    /// A group with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            invalid: false,
            disabled: false,
            level: LayerLevel::One,
            addons: Vec::new(),
            editor: None,
        }
    }

    /// `invalid`: 2px `support-error` outline.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disable the editor.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The layer the group sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Add an addon.
    pub fn addon(mut self, addon: InputGroupAddon) -> Self {
        self.addons.push(addon);
        self
    }

    /// The single-line editor.
    pub fn input(mut self, input: InputGroupInput) -> Self {
        self.editor = Some(Editor::Input(input));
        self
    }

    /// The multi-line editor.
    pub fn textarea(mut self, textarea: InputGroupTextarea) -> Self {
        self.editor = Some(Editor::Textarea(textarea));
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // `.cu-ig { min-height: var(--size-md) }`, `.cu-ig-input { height:
        // var(--size-md) }`: 40px in every density (the reference's compact rules
        // in `src/ghost.css` do not touch `.cu-ig`).
        let height = group_height();
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let (focus, empty) = match &self.editor {
            Some(Editor::Input(i)) => (
                Some(i.state.read(cx).focus_handle(cx)),
                i.state.read(cx).value().is_empty(),
            ),
            Some(Editor::Textarea(t)) => (
                Some(t.state.read(cx).focus_handle(cx)),
                t.state.read(cx).value().is_empty(),
            ),
            None => (None, true),
        };
        let focused = focus.as_ref().is_some_and(|f| f.is_focused(window));
        // `.cu-field[data-invalid] .cu-ig, .cu-ig[data-invalid]`.
        let invalid = self.invalid || super::field::in_invalid_field();
        let look = GroupLook::of(
            self.level,
            *hovered.read(cx),
            focused,
            invalid,
            self.disabled,
        );
        let background = motion::animate_color(
            (id.clone(), "bg"),
            look.background,
            FIELD_MOTION,
            window,
            cx,
        );
        let outline = motion::animate_color(
            (id.clone(), "outline"),
            look.outline.unwrap_or(Hsla::transparent_black()),
            FIELD_MOTION,
            window,
            cx,
        );
        let text_color = color(if self.disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextPrimary
        });
        let placeholder_color = color(if self.disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextPlaceholder
        });
        let has_start = self
            .addons
            .iter()
            .any(|a| a.align == AddonAlign::InlineStart);

        let editor: Option<(i32, AnyElement)> = self.editor.map(|editor| match editor {
            Editor::Input(input) => {
                // `.cu-ig-input { flex: 1; height: var(--size-md); padding: 0 var(--spacing-05) }`,
                // `.cu-ig-addon--inline-start ~ .cu-ig-input { padding-left: var(--spacing-03) }`.
                let pad_left = if has_start { 8. } else { 16. };
                let element = div()
                    .relative()
                    .flex()
                    .items_center()
                    .flex_1()
                    .min_w_0()
                    .h(rem(height))
                    .pl(rem(pad_left))
                    .pr(rem(16.))
                    .child(
                        KitInput::new(&input.state)
                            .appearance(false)
                            .with_size(Size::XSmall)
                            .disabled(self.disabled)
                            .p_0()
                            .h_full()
                            .flex_1()
                            .min_w_0()
                            .text_size(rem(14.))
                            .line_height(rem(18.))
                            .text_color(text_color),
                    )
                    .when_some(input.placeholder.filter(|_| empty), |this, text| {
                        this.child(
                            div()
                                .absolute()
                                .top_0()
                                .bottom_0()
                                .left(rem(pad_left))
                                .right(rem(16.))
                                .flex()
                                .items_center()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_color(placeholder_color)
                                .child(text),
                        )
                    });
                (0, element.into_any_element())
            }
            Editor::Textarea(area) => {
                // `.cu-ig-textarea { padding: 0.6875rem var(--spacing-05); line-height:
                // 1.42857; flex-basis: 100% }` (Kit insets its text by 2px/4px); no
                // resize grip (`t3.css` craft pass: `.cu-ig-textarea { resize: none }`).
                let height = textarea_height(area.rows);
                // Tab and Shift+Tab move focus, as in a `<textarea>`.
                let element = super::textarea::tab_moves_focus(div())
                    .relative()
                    .w_full()
                    .h(rem(height))
                    .overflow_hidden()
                    .pt(rem(9.))
                    .pb(rem(9.))
                    .px(rem(12.))
                    .child(
                        KitTextarea::new(&area.state)
                            .appearance(false)
                            .with_size(Size::XSmall)
                            .disabled(self.disabled)
                            .h(rem(height - TEXTAREA_PADDING))
                            .w_full()
                            .text_size(rem(14.))
                            .line_height(rem(20.))
                            .text_color(text_color),
                    )
                    .when_some(area.placeholder.filter(|_| empty), |this, text| {
                        this.child(
                            div()
                                .absolute()
                                .top(rem(11.))
                                .left(rem(16.))
                                .right(rem(16.))
                                .type_style(TypeStyle::Body01)
                                .text_color(placeholder_color)
                                .child(text),
                        )
                    });
                (0, element.into_any_element())
            }
        });

        let mut parts: Vec<(i32, AnyElement)> = self
            .addons
            .into_iter()
            .map(|addon| {
                let [top, right, bottom, left] = addon.align.padding();
                let full_row = matches!(addon.align, AddonAlign::BlockStart | AddonAlign::BlockEnd);
                let element = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .flex_shrink_0()
                    .pt(rem(top))
                    .pr(rem(right))
                    .pb(rem(bottom))
                    .pl(rem(left))
                    // `.cu-ig-addon { color: var(--icon-secondary) }`.
                    .text_color(color(ColorRole::IconSecondary))
                    .when(full_row, |this| this.w_full())
                    .when(addon.align == AddonAlign::BlockEnd, |this| {
                        this.justify_between()
                            .border_t_1()
                            .border_color(color(self.level.border_subtle()))
                    })
                    .children(addon.children);
                (addon.align.order(), element.into_any_element())
            })
            .collect();
        parts.extend(editor);
        parts.sort_by_key(|(order, _)| *order);

        div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_stretch()
            .w_full()
            .min_h(rem(height))
            .bg(background)
            .pb(gpui_kit::px(form::FIELD_RULE))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(text_color)
            .when(self.disabled, |this| {
                this.cursor(CursorStyle::OperationNotAllowed)
            })
            .on_hover(move |over, _, cx| {
                hovered.update(cx, |h, cx| {
                    if *h != *over {
                        *h = *over;
                        cx.notify();
                    }
                })
            })
            .when_some(focus.filter(|_| !self.disabled), |this, focus| {
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if !focus.contains_focused(window, cx) {
                        focus.focus(window, cx);
                    }
                })
            })
            .children(parts.into_iter().map(|(_, e)| e))
            .child(form::bottom_rule(look.border))
            .child(form::inset_outline(2., outline))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn frame_follows_cu_ig_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = GroupLook::of(LayerLevel::One, false, false, false, false);
        assert_eq!(hex(rest.background), "#f4f4f4");
        assert_eq!(hex(rest.border), "#8d8d8d");
        assert_eq!(rest.outline, None);
        // `.cu-ig:hover { background: var(--field-hover) }`.
        let hover = GroupLook::of(LayerLevel::One, true, false, false, false);
        assert_eq!(hex(hover.background), "#e8e8e8");
        // `:focus-within` wins over `[data-invalid]`.
        let both = GroupLook::of(LayerLevel::One, false, true, true, false);
        assert_eq!(both.outline.map(hex).as_deref(), Some("#0f62fe"));
        let invalid = GroupLook::of(LayerLevel::One, false, false, true, false);
        assert_eq!(invalid.outline.map(hex).as_deref(), Some("#da1e28"));
    }

    #[test]
    fn group_is_size_md_in_every_density() {
        assert_eq!(group_height(), 40.);
        // 3 rows × 20 + 2 × 11.
        assert_eq!(textarea_height(3), 82.);
        assert_eq!(textarea_height(3) - TEXTAREA_PADDING, 64.);
    }

    #[test]
    fn addons_order_and_pad_like_the_reference() {
        let orders = AddonAlign::ALL.map(|a| a.order());
        assert_eq!(orders, [-1, 2, -2, 3]);
        assert_eq!(AddonAlign::InlineStart.padding(), [0., 0., 0., 16.]);
        assert_eq!(AddonAlign::BlockEnd.padding(), [4., 8., 4., 16.]);
    }
}
