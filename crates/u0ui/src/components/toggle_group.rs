//! ToggleGroup: shadcn `ToggleGroup`.
//!
//! - `type="single"` → Carbon `ContentSwitcher` (`.cds--content-switcher`): one
//!   selected option, a square track (`t3.css` global craft pass:
//!   `.cds--content-switcher, .cds--content-switcher-btn,
//!   .cds--content-switcher-btn::after { border-radius: 0 !important }`) with a
//!   1px `border-inverse` outline and centered labels
//!   (`.cds--content-switcher-btn { justify-content: center; text-align: center }`);
//!   the selected button fills `layer-selected-inverse` with `text-inverse`;
//!   others are `text-secondary`, hover `layer-hover`, split by 1×16px
//!   `border-subtle` dividers that hide next to the selected/hovered/focused
//!   button. Focus: inset 2px `focus` + 1px `focus-inset`.
//!   Keyboard: roving focus, ←/→ (and Home/End) move and select (Carbon's
//!   automatic selection), wrapping; Enter/Space select.
//! - [`ToggleGroupFrame::Toolbar`]: a single-select group in a toolbar reads
//!   like a field (`bundle.css` `u0: a toggle group in a toolbar reads like a
//!   field: border-strong-01 frame (not the black border-inverse), square,
//!   inverse fill for the selected segment`).
//! - `type="multiple"` → a row of outline [`Toggle`]s 4px apart
//!   (`.cu-toggle-group { gap: var(--spacing-02) }`).
//!
//! Kit has no content switcher, so the roving focus and arrow keys are written
//! here; each segment is a Kit `base::Button` (focus, Enter/Space, a11y).
//!
//! ```ignore
//! ToggleGroup::new("range")
//!     .default_value(["week"])
//!     .item(ToggleGroupItem::new("day").child("Day"))
//!     .item(ToggleGroupItem::new("week").child("Week"))
//! ```
use super::form::{self, rem};
use super::toggle::{Toggle, ToggleSize, ToggleVariant, toggle_height};
use crate::{
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, FocusHandle, Hsla, InteractiveElement as _,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `index.d.ts` `ToggleGroupProps.type`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleGroupType {
    /// One value (Carbon `ContentSwitcher`).
    #[default]
    Single,
    /// Any number of values (outline toggles).
    Multiple,
}

impl ToggleGroupType {
    /// Both types.
    pub const ALL: [Self; 2] = [Self::Single, Self::Multiple];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multiple => "multiple",
        }
    }
}

/// The frame of a single-select group: the context it sits in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleGroupFrame {
    /// Carbon's content switcher: `border-inverse` outline.
    #[default]
    Default,
    /// In a toolbar: a `border-strong-01` frame like a field, the selected
    /// segment still filled `layer-selected-inverse` (`bundle.css` `u0: a
    /// toggle group in a toolbar`).
    Toolbar,
}

impl ToggleGroupFrame {
    /// Both frames.
    pub const ALL: [Self; 2] = [Self::Default, Self::Toolbar];

    /// A short name for controls.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Toolbar => "toolbar",
        }
    }

    /// The enabled frame color.
    fn border(self) -> ColorRole {
        match self {
            Self::Default => ColorRole::BorderInverse,
            Self::Toolbar => ColorRole::BorderStrong01,
        }
    }
}

/// One option (`ToggleGroupItem`).
pub struct ToggleGroupItem {
    value: SharedString,
    disabled: bool,
    children: Vec<AnyElement>,
}

impl ToggleGroupItem {
    /// An option with its `value`; add its content with `child`.
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            disabled: false,
            children: Vec::new(),
        }
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ParentElement for ToggleGroupItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The state a content-switcher segment look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SegmentState {
    /// The selected segment.
    pub selected: bool,
    /// Pointer over it.
    pub hovered: bool,
    /// Keyboard focus on it.
    pub focused: bool,
    /// Disabled.
    pub disabled: bool,
}

/// The colors of one content-switcher segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentLook {
    /// Fill (the `::after` indicator when selected, `layer-hover` on hover).
    pub background: Hsla,
    /// Label color.
    pub text: Hsla,
    /// Top/bottom 1px border (`border-block-*: 1px solid var(--border-inverse)`;
    /// `border-strong-01` in a toolbar).
    pub border: Hsla,
    /// Focus ring (`inset 0 0 0 2px var(--focus)`).
    pub focus: Option<Hsla>,
    /// Inner ring or fill inside the focus ring (`focus-inset`).
    pub focus_inset: Option<Hsla>,
}

impl SegmentLook {
    /// The look of a segment (`.cds--content-switcher-btn*` rules).
    pub fn of(state: SegmentState) -> Self {
        Self::framed(state, ToggleGroupFrame::Default)
    }

    /// The look of a segment in `frame`.
    pub fn framed(state: SegmentState, frame: ToggleGroupFrame) -> Self {
        let clear = Hsla::transparent_black();
        if state.disabled {
            // `.cds--content-switcher-btn:disabled { border-color: var(--border-disabled);
            // color: var(--text-disabled) }`; selected adds `button-disabled`.
            return Self {
                background: if state.selected {
                    color(ColorRole::ButtonDisabled)
                } else {
                    clear
                },
                text: color(ColorRole::TextDisabled),
                border: color(ColorRole::BorderDisabled),
                focus: None,
                focus_inset: None,
            };
        }
        let border = color(frame.border());
        if state.selected {
            // `.cds--content-switcher--selected { color: var(--text-inverse) }`,
            // `::after { background: var(--layer-selected-inverse); transform: scaleY(1) }`;
            // `:focus::after { box-shadow: inset 0 0 0 2px focus, inset 0 0 0 3px focus-inset }`.
            return Self {
                background: color(ColorRole::LayerSelectedInverse),
                text: color(ColorRole::TextInverse),
                border,
                focus: state.focused.then(|| color(ColorRole::Focus)),
                focus_inset: state.focused.then(|| color(ColorRole::FocusInset)),
            };
        }
        if state.focused && !state.hovered {
            // `:not(--selected):focus::after { background: var(--focus-inset) }` inside
            // `::before { box-shadow: inset 0 0 0 2px var(--focus) }`.
            return Self {
                background: color(ColorRole::FocusInset),
                text: color(ColorRole::TextPrimary),
                border,
                focus: Some(color(ColorRole::Focus)),
                focus_inset: None,
            };
        }
        if state.hovered {
            // `:hover { background-color: var(--layer-hover); color: var(--text-primary) }`.
            return Self {
                background: color(ColorRole::LayerHover01),
                text: color(ColorRole::TextPrimary),
                border,
                focus: None,
                focus_inset: None,
            };
        }
        Self {
            background: clear,
            text: color(ColorRole::TextSecondary),
            border,
            focus: None,
            focus_inset: None,
        }
    }
}

/// The track outline (`.cds--content-switcher { outline: 0.0625rem solid
/// var(--border-inverse) }`), dimmed for the whole track when any segment is
/// disabled (`:has(.cds--content-switcher-btn:disabled) { outline-color:
/// var(--border-disabled) }`); `border-strong-01` in a toolbar.
pub fn track_outline(any_disabled: bool, frame: ToggleGroupFrame) -> Hsla {
    color(if any_disabled {
        ColorRole::BorderDisabled
    } else {
        frame.border()
    })
}

/// Whether the 1×16px divider before segment `index` shows
/// (`.cds--content-switcher-btn::before`): hidden on the first segment and
/// next to a selected, hovered or focused one
/// (`.cds--content-switcher--selected::before, … + .cds--content-switcher-btn::before`).
pub fn divider_visible(index: usize, states: &[SegmentState]) -> bool {
    if index == 0 {
        return false;
    }
    let busy = |s: &SegmentState| s.selected || s.hovered || s.focused;
    !(busy(&states[index]) || busy(&states[index - 1]))
}

/// The next selectable index for a key, wrapping and skipping disabled
/// segments (Carbon `ContentSwitcher` keyboard handling).
pub fn step(current: usize, delta: isize, disabled: &[bool]) -> Option<usize> {
    let n = disabled.len() as isize;
    if n == 0 {
        return None;
    }
    let mut i = current as isize;
    for _ in 0..n {
        i = (i + delta).rem_euclid(n);
        if !disabled[i as usize] {
            return Some(i as usize);
        }
    }
    None
}

type ChangeHandler = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
type CommitFn = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

/// A toggle group. See the module docs.
#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    kind: ToggleGroupType,
    value: Option<Vec<SharedString>>,
    default_value: Option<Vec<SharedString>>,
    size: ToggleSize,
    frame: ToggleGroupFrame,
    items: Vec<ToggleGroupItem>,
    on_value_change: Option<ChangeHandler>,
}

impl ToggleGroup {
    /// A single-select group with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            kind: ToggleGroupType::Single,
            value: None,
            default_value: None,
            size: ToggleSize::Md,
            frame: ToggleGroupFrame::Default,
            items: Vec::new(),
            on_value_change: None,
        }
    }

    /// `type`.
    pub fn kind(mut self, kind: ToggleGroupType) -> Self {
        self.kind = kind;
        self
    }

    /// Controlled `value` (single: one entry; multiple: any).
    pub fn value<S: Into<SharedString>>(mut self, value: impl IntoIterator<Item = S>) -> Self {
        self.value = Some(value.into_iter().map(Into::into).collect());
        self
    }

    /// `defaultValue` (single defaults to the first item).
    pub fn default_value<S: Into<SharedString>>(
        mut self,
        value: impl IntoIterator<Item = S>,
    ) -> Self {
        self.default_value = Some(value.into_iter().map(Into::into).collect());
        self
    }

    /// `onValueChange`, with the selected values.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }

    /// `size` (default `md`).
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// The frame of a single-select group (default Carbon's; `Toolbar` in a
    /// toolbar). No effect on `type="multiple"`.
    pub fn frame(mut self, frame: ToggleGroupFrame) -> Self {
        self.frame = frame;
        self
    }

    /// Append an option.
    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self
    }
}

#[derive(Default)]
struct Segments {
    hovered: Option<usize>,
    focus: Vec<FocusHandle>,
}

impl RenderOnce for ToggleGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let default = self
            .default_value
            .clone()
            .unwrap_or_else(|| match self.kind {
                ToggleGroupType::Single => self
                    .items
                    .first()
                    .map(|i| i.value.clone())
                    .into_iter()
                    .collect(),
                ToggleGroupType::Multiple => Vec::new(),
            });
        let (value, store) = form::controlled(
            (id.clone(), "value"),
            self.value.clone(),
            default,
            window,
            cx,
        );
        let on_change = self.on_value_change.clone();
        let commit: CommitFn = Rc::new({
            let store = store.clone();
            move |next, window, cx| {
                store.update(cx, |v, cx| {
                    *v = next.clone();
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(&next, window, cx);
                }
            }
        });

        if self.kind == ToggleGroupType::Multiple {
            return div()
                .id(id.clone())
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(tokens::SPACING_02)
                .children(self.items.into_iter().map(|item| {
                    let on = value.contains(&item.value);
                    let commit = commit.clone();
                    let value = value.clone();
                    let item_value = item.value.clone();
                    let mut toggle = Toggle::new((id.clone(), item.value.clone()))
                        .variant(ToggleVariant::Outline)
                        .size(self.size)
                        .disabled(item.disabled)
                        .pressed(on)
                        .on_pressed_change(move |pressed, window, cx| {
                            let mut next: Vec<SharedString> = value
                                .iter()
                                .filter(|v| **v != item_value)
                                .cloned()
                                .collect();
                            if pressed {
                                next.push(item_value.clone());
                            }
                            commit(next, window, cx)
                        });
                    toggle.extend(item.children);
                    toggle
                }))
                .into_any_element();
        }

        // Single: Carbon content switcher, `block-size: var(--layout-size-height-local)`
        // (32/40/48 by size; no density rule in the reference).
        let height = toggle_height(self.size);
        let selected = self
            .items
            .iter()
            .position(|i| value.first() == Some(&i.value))
            .unwrap_or(0);
        let count = self.items.len();
        let segments =
            window.use_keyed_state((id.clone(), "segments"), cx, |_, _| Segments::default());
        segments.update(cx, |s, cx| {
            while s.focus.len() < count {
                s.focus.push(cx.focus_handle());
            }
        });
        let (hovered, focus) = {
            let s = segments.read(cx);
            (s.hovered, s.focus.clone())
        };
        let disabled: Vec<bool> = self.items.iter().map(|i| i.disabled).collect();
        let frame = self.frame;
        let outline = track_outline(disabled.iter().any(|d| *d), frame);
        let states: Vec<SegmentState> = (0..count)
            .map(|i| SegmentState {
                selected: i == selected,
                hovered: hovered == Some(i) && !disabled[i],
                focused: focus[i].is_focused(window),
                disabled: disabled[i],
            })
            .collect();
        let values: Vec<SharedString> = self.items.iter().map(|i| i.value.clone()).collect();
        // `.cds--content-switcher--lg .cds--content-switcher-btn { padding-inline: 0.875rem }`.
        let pad_x = if self.size == ToggleSize::Lg {
            14.
        } else {
            16.
        };

        let on_key = {
            let commit = commit.clone();
            let focus = focus.clone();
            let disabled = disabled.clone();
            let values = values.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let target = match event.keystroke.key.as_str() {
                    "left" => step(selected, -1, &disabled),
                    "right" => step(selected, 1, &disabled),
                    "home" => step(count.saturating_sub(1), 1, &disabled),
                    "end" => step(0, -1, &disabled),
                    _ => return,
                };
                cx.stop_propagation();
                if let Some(next) = target {
                    focus[next].focus(window, cx);
                    if next != selected {
                        commit(vec![values[next].clone()], window, cx);
                    }
                }
            }
        };

        div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_row()
            .w_full()
            .h(rem(height))
            .on_key_down(on_key)
            .children(self.items.into_iter().enumerate().map(|(i, item)| {
                let state = states[i];
                // No transition: `.cds--content-switcher-btn` has none in `bundle.css`.
                let look = SegmentLook::framed(state, frame);
                let (bg, text) = (look.background, look.text);
                let first = i == 0;
                let last = i + 1 == count;
                let commit = commit.clone();
                let value = item.value.clone();
                let segments = segments.clone();
                let divider = divider_visible(i, &states);
                gpui_kit::base::Button::new((id.clone(), format!("seg-{i}")))
                    .track_focus(&focus[i])
                    .tab_stop(i == selected)
                    .disabled(state.disabled)
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .items_center()
                    // `t3.css` craft pass: `.cds--content-switcher-btn
                    // { justify-content: center; text-align: center }`.
                    .justify_center()
                    .px(rem(pad_x))
                    .border_t_1()
                    .border_b_1()
                    // The track outline (`outline: 1px solid; outline-offset: -1px`) is
                    // painted over each segment's own border edge, so the edge takes the
                    // outline's color.
                    .border_color(outline)
                    .when(first, |this| this.border_l_1())
                    .when(last, |this| this.border_r_1())
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(text)
                    .cursor(if state.disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::PointingHand
                    })
                    .on_hover(move |over, _, cx| {
                        segments.update(cx, |s, cx| {
                            let next = if *over {
                                Some(i)
                            } else if s.hovered == Some(i) {
                                None
                            } else {
                                s.hovered
                            };
                            if next != s.hovered {
                                s.hovered = next;
                                cx.notify();
                            }
                        })
                    })
                    .on_click(move |_, window, cx| {
                        if i != selected {
                            commit(vec![value.clone()], window, cx);
                        }
                    })
                    // The selected/focus fill (`::after`, square per the craft pass,
                    // over the borders).
                    .child(
                        div()
                            .absolute()
                            .top(px(-1.))
                            .bottom(px(-1.))
                            .left(px(if first { -1. } else { 0. }))
                            .right(px(if last { -1. } else { 0. }))
                            .bg(bg)
                            .when_some(look.focus, |this, ring| {
                                this.border_2().border_color(ring).child(
                                    div()
                                        .size_full()
                                        .when_some(look.focus_inset, |this, inset| {
                                            this.border_1().border_color(inset)
                                        }),
                                )
                            }),
                    )
                    .when(divider, |this| {
                        this.child(
                            div()
                                .absolute()
                                .left_0()
                                .top(rem((height - 2. - 16.) / 2.))
                                .w(px(1.))
                                .h(rem(16.))
                                .bg(color(ColorRole::BorderSubtle01)),
                        )
                    })
                    .child(
                        div()
                            .relative()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .flex()
                            .items_center()
                            .gap(tokens::SPACING_03)
                            .children(item.children),
                    )
            }))
            // The track outline is drawn by each segment's borders (see above).
            .into_any_element()
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
    fn segment_colors_follow_content_switcher_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = SegmentLook::of(SegmentState::default());
        assert_eq!(rest.background.a, 0.);
        assert_eq!(hex(rest.text), "#525252");
        assert_eq!(hex(rest.border), "#161616");
        let selected = SegmentLook::of(SegmentState {
            selected: true,
            ..Default::default()
        });
        assert_eq!(hex(selected.background), "#161616");
        assert_eq!(hex(selected.text), "#ffffff");
        let hover = SegmentLook::of(SegmentState {
            hovered: true,
            ..Default::default()
        });
        assert_eq!(hex(hover.background), "#e8e8e8");
        assert_eq!(hex(hover.text), "#161616");
        let focus = SegmentLook::of(SegmentState {
            focused: true,
            selected: true,
            ..Default::default()
        });
        assert_eq!(focus.focus.map(hex).as_deref(), Some("#0f62fe"));
        assert_eq!(focus.focus_inset.map(hex).as_deref(), Some("#ffffff"));
        let disabled = SegmentLook::of(SegmentState {
            disabled: true,
            selected: true,
            ..Default::default()
        });
        assert_eq!(hex(disabled.background), "#c6c6c6");
        assert_eq!(hex(disabled.border), "#c6c6c6");
    }

    #[test]
    fn track_outline_dims_when_any_segment_is_disabled() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            hex(track_outline(false, ToggleGroupFrame::Default)),
            "#161616"
        );
        assert_eq!(
            hex(track_outline(true, ToggleGroupFrame::Default)),
            "#c6c6c6"
        );
        // `u0: a toggle group in a toolbar`: border-strong-01 frame.
        assert_eq!(
            hex(track_outline(false, ToggleGroupFrame::Toolbar)),
            "#8d8d8d"
        );
        assert_eq!(
            hex(track_outline(true, ToggleGroupFrame::Toolbar)),
            "#c6c6c6"
        );
        // `--layout-size-height-local`: 32/40/48 by size, whatever the density.
        assert_eq!(ToggleSize::ALL.map(toggle_height), [32., 40., 48.]);
    }

    #[test]
    fn toolbar_frame_keeps_the_inverse_selection() {
        crate::theme::tests_set(CarbonTheme::G100);
        let selected = SegmentState {
            selected: true,
            ..Default::default()
        };
        let toolbar = SegmentLook::framed(selected, ToggleGroupFrame::Toolbar);
        // G100 border-strong-01, layer-selected-inverse, text-inverse.
        assert_eq!(hex(toolbar.border), "#6f6f6f");
        assert_eq!(hex(toolbar.background), "#f4f4f4");
        assert_eq!(hex(toolbar.text), "#161616");
        let rest = SegmentLook::framed(SegmentState::default(), ToggleGroupFrame::Toolbar);
        assert_eq!(hex(rest.border), "#6f6f6f");
        assert_eq!(
            hex(SegmentLook::of(SegmentState::default()).border),
            "#f4f4f4"
        );
    }

    #[test]
    fn dividers_hide_next_to_busy_segments() {
        let mut states = vec![SegmentState::default(); 3];
        assert!(!divider_visible(0, &states));
        assert!(divider_visible(1, &states));
        states[1].selected = true;
        assert!(!divider_visible(1, &states));
        assert!(!divider_visible(2, &states));
    }

    #[test]
    fn arrow_keys_wrap_and_skip_disabled() {
        let disabled = [false, true, false];
        assert_eq!(step(0, 1, &disabled), Some(2));
        assert_eq!(step(2, 1, &disabled), Some(0));
        assert_eq!(step(0, -1, &disabled), Some(2));
        assert_eq!(step(0, 1, &[true]), None);
    }
}
