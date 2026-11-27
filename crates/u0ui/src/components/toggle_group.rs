//! ToggleGroup: shadcn `ToggleGroup`.
//!
//! - `type="single"` → Carbon `ContentSwitcher` (`.cds--content-switcher`): one
//!   selected option, a square track (`t3.css` global craft pass:
//!   `.cds--content-switcher, .cds--content-switcher-btn,
//!   .cds--content-switcher-btn::after { border-radius: 0 !important }`) with a
//!   1px `border-strong-01` outline, segment lines and side lines
//!   (`bundle.css` `.cds--content-switcher { outline: … var(--border-strong-01) }`,
//!   `.cds--content-switcher-btn { border-block-*: … var(--border-strong-01) }`)
//!   and centered labels
//!   (`.cds--content-switcher-btn { justify-content: center; text-align: center }`);
//!   the selected button fills `layer-selected-inverse` with `text-inverse`
//!   edge to edge (its `border: 0` lets the fill cover the outline);
//!   others are `text-secondary`, hover `layer-hover`, split by 1×16px
//!   `border-subtle` dividers that hide next to the selected/hovered/focused
//!   button. Focus: inset 2px `focus` + 1px `focus-inset`. Disabled segments
//!   draw `border-disabled` lines and dividers.
//!   Keyboard: roving focus, ←/→ (and Home/End) move and select (Carbon's
//!   automatic selection), wrapping; Enter/Space select.
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

/// The frame of a single-select group. Kept for downstream code: since the
/// design moved every content-switcher line to `border-strong-01`, the toolbar
/// frame is identical to the default and every value draws the same.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleGroupFrame {
    /// Carbon's content switcher.
    #[default]
    Default,
    /// Same as [`ToggleGroupFrame::Default`].
    #[deprecated(
        note = "identical to `ToggleGroupFrame::Default`; the content switcher \
                         now always draws `border-strong-01` lines"
    )]
    Toolbar,
}

impl ToggleGroupFrame {
    /// The one distinct frame.
    pub const ALL: [Self; 1] = [Self::Default];

    /// A short name for controls.
    #[allow(deprecated)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Toolbar => "toolbar",
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
    /// The 1px edge color (top/bottom, plus the outer side of the first and
    /// last segment): what shows on the track outline over this segment.
    pub border: Hsla,
    /// Focus ring (`inset 0 0 0 2px var(--focus)`).
    pub focus: Option<Hsla>,
    /// Inner ring or fill inside the focus ring (`focus-inset`).
    pub focus_inset: Option<Hsla>,
}

impl SegmentLook {
    /// The look of a segment (`.cds--content-switcher-btn*` rules).
    pub fn of(state: SegmentState) -> Self {
        let clear = Hsla::transparent_black();
        if state.disabled {
            // `.cds--content-switcher-btn:disabled { border-color: var(--border-disabled);
            // color: var(--text-disabled) }`; the first/last side line drops its
            // `box-shadow`, leaving the outline, which `:has(:disabled)` turns
            // `border-disabled`. Selected adds `button-disabled` with `border: 0`,
            // so its fill reaches the edge.
            let fill = color(ColorRole::ButtonDisabled);
            return Self {
                background: if state.selected { fill } else { clear },
                text: color(ColorRole::TextDisabled),
                border: if state.selected {
                    fill
                } else {
                    color(ColorRole::BorderDisabled)
                },
                focus: None,
                focus_inset: None,
            };
        }
        // `.cds--content-switcher-btn { border-block-*: 1px solid var(--border-strong-01) }`,
        // `:first-child`/`:last-child { box-shadow: inset ±1px 0 0 0 var(--border-strong-01) }`,
        // `:focus:hover::after { border-block-*: … var(--border-strong-01) }`. These sit
        // over the track outline, so an enabled segment keeps `border-strong-01` even
        // when another segment is disabled.
        let strong = color(ColorRole::BorderStrong01);
        let focus = color(ColorRole::Focus);
        if state.selected {
            // `.cds--content-switcher--selected { border: 0; color: var(--text-inverse) }`,
            // `::after { background: var(--layer-selected-inverse); transform: scaleY(1) }`
            // over the outline (z-index 3);
            // `:focus::after { box-shadow: inset 0 0 0 2px focus, inset 0 0 0 3px focus-inset }`.
            let fill = color(ColorRole::LayerSelectedInverse);
            return Self {
                background: fill,
                text: color(ColorRole::TextInverse),
                border: if state.focused { focus } else { fill },
                focus: state.focused.then_some(focus),
                focus_inset: state.focused.then(|| color(ColorRole::FocusInset)),
            };
        }
        if state.focused {
            // `:focus { border: 0 }`; `:not(--selected):focus::before { box-shadow: inset 0 0 0
            // 0.125rem var(--focus) }` (z-index 2) over `::after { background: var(--focus-inset) }`,
            // which turns `layer-hover` on hover (`:focus:hover::after`).
            return Self {
                background: color(if state.hovered {
                    ColorRole::LayerHover01
                } else {
                    ColorRole::FocusInset
                }),
                text: color(ColorRole::TextPrimary),
                border: focus,
                focus: Some(focus),
                focus_inset: None,
            };
        }
        if state.hovered {
            // `:hover { background-color: var(--layer-hover); color: var(--text-primary) }`.
            return Self {
                background: color(ColorRole::LayerHover01),
                text: color(ColorRole::TextPrimary),
                border: strong,
                focus: None,
                focus_inset: None,
            };
        }
        Self {
            background: clear,
            text: color(ColorRole::TextSecondary),
            border: strong,
            focus: None,
            focus_inset: None,
        }
    }
}

/// Same as [`SegmentLook::of`]; every frame draws the same.
impl SegmentLook {
    /// The look of a segment; `frame` no longer changes it.
    #[deprecated(note = "use `SegmentLook::of`; every frame draws the same")]
    pub fn framed(state: SegmentState, _frame: ToggleGroupFrame) -> Self {
        Self::of(state)
    }
}

/// The track outline (`.cds--content-switcher { outline: 0.0625rem solid
/// var(--border-strong-01) }`), dimmed when any segment is disabled
/// (`:has(.cds--content-switcher-btn:disabled) { outline-color:
/// var(--border-disabled) }`); `frame` no longer changes it. Each segment
/// paints its own edge over it ([`SegmentLook::border`]), so it only shows
/// beside a disabled segment, where that edge is `border-disabled` too.
pub fn track_outline(any_disabled: bool, _frame: ToggleGroupFrame) -> Hsla {
    color(if any_disabled {
        ColorRole::BorderDisabled
    } else {
        ColorRole::BorderStrong01
    })
}

/// Whether the divider before segment `index` shows (see [`divider`]).
#[deprecated(note = "use `divider`, which also gives its color")]
pub fn divider_visible(index: usize, states: &[SegmentState]) -> bool {
    divider(index, states).is_some()
}

/// The 1×16px divider before segment `index`
/// (`.cds--content-switcher-btn::before { background-color: var(--border-subtle) }`),
/// or `None` where it hides: on the first segment and next to a selected,
/// hovered or focused one
/// (`.cds--content-switcher--selected::before, … + .cds--content-switcher-btn::before`).
/// Before a disabled segment it turns `border-disabled`
/// (`.cds--content-switcher-btn:disabled::before`, which wins over the
/// selected-neighbour rule by source order but not over the `:focus +` and
/// `:hover +` rules) and hides on a disabled selected one or after it
/// (`--selected:disabled::before { display: none }`,
/// `--selected:disabled + .cds--content-switcher-btn::before`).
pub fn divider(index: usize, states: &[SegmentState]) -> Option<Hsla> {
    if index == 0 {
        return None;
    }
    let (prev, this) = (states[index - 1], states[index]);
    if this.disabled {
        let hidden =
            this.selected || prev.focused || prev.hovered || (prev.selected && prev.disabled);
        return (!hidden).then(|| color(ColorRole::BorderDisabled));
    }
    let busy = |s: SegmentState| s.selected || s.hovered || s.focused;
    (!(busy(this) || busy(prev))).then(|| color(ColorRole::BorderSubtle01))
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

    /// The frame of a single-select group; every frame now draws the same
    /// `border-strong-01` lines, so this has no effect.
    #[deprecated(note = "every frame draws the same; drop the call")]
    pub fn frame(self, _frame: ToggleGroupFrame) -> Self {
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
                let look = SegmentLook::of(state);
                let (bg, text) = (look.background, look.text);
                let first = i == 0;
                let last = i + 1 == count;
                let commit = commit.clone();
                let value = item.value.clone();
                let segments = segments.clone();
                let divider = divider(i, &states);
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
                    // The track outline (`outline: 1px solid; outline-offset: -1px`)
                    // lies under each segment's edge; the segment's own lines, fill or
                    // focus ring decide the color there (GPUI paints borders over the
                    // children, so the fill below never hides them).
                    .border_color(look.border)
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
                    .when_some(divider, |this, line| {
                        this.child(
                            div()
                                .absolute()
                                .left_0()
                                .top(rem((height - 2. - 16.) / 2.))
                                .w(px(1.))
                                .h(rem(16.))
                                .bg(line),
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
        // `border-block-*: 1px solid var(--border-strong-01)`.
        assert_eq!(hex(rest.border), "#8d8d8d");
        let selected = SegmentLook::of(SegmentState {
            selected: true,
            ..Default::default()
        });
        assert_eq!(hex(selected.background), "#161616");
        assert_eq!(hex(selected.text), "#ffffff");
        // `border: 0`: the fill covers the outline.
        assert_eq!(hex(selected.border), "#161616");
        let hover = SegmentLook::of(SegmentState {
            hovered: true,
            ..Default::default()
        });
        assert_eq!(hex(hover.background), "#e8e8e8");
        assert_eq!(hex(hover.text), "#161616");
        assert_eq!(hex(hover.border), "#8d8d8d");
        let focus = SegmentLook::of(SegmentState {
            focused: true,
            selected: true,
            ..Default::default()
        });
        assert_eq!(focus.focus.map(hex).as_deref(), Some("#0f62fe"));
        assert_eq!(focus.focus_inset.map(hex).as_deref(), Some("#ffffff"));
        assert_eq!(hex(focus.border), "#0f62fe");
        // `:focus:hover::after { background: var(--layer-hover) }` under the ring.
        let focus_hover = SegmentLook::of(SegmentState {
            focused: true,
            hovered: true,
            ..Default::default()
        });
        assert_eq!(hex(focus_hover.background), "#e8e8e8");
        assert_eq!(focus_hover.focus.map(hex).as_deref(), Some("#0f62fe"));
        let disabled = SegmentLook::of(SegmentState {
            disabled: true,
            ..Default::default()
        });
        assert_eq!(hex(disabled.border), "#c6c6c6");
        let disabled_selected = SegmentLook::of(SegmentState {
            disabled: true,
            selected: true,
            ..Default::default()
        });
        assert_eq!(hex(disabled_selected.background), "#c6c6c6");
        assert_eq!(hex(disabled_selected.border), "#c6c6c6");
        // `--layout-size-height-local`: 32/40/48 by size, whatever the density.
        assert_eq!(ToggleSize::ALL.map(toggle_height), [32., 40., 48.]);
    }

    #[test]
    fn g100_lines_are_border_strong_01() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(
            hex(SegmentLook::of(SegmentState::default()).border),
            "#6f6f6f"
        );
        let selected = SegmentLook::of(SegmentState {
            selected: true,
            ..Default::default()
        });
        // G100 layer-selected-inverse, text-inverse.
        assert_eq!(hex(selected.background), "#f4f4f4");
        assert_eq!(hex(selected.text), "#161616");
    }

    #[test]
    fn dividers_hide_next_to_busy_segments() {
        crate::theme::tests_set(CarbonTheme::White);
        // `.cds--content-switcher { outline: … var(--border-strong-01) }`.
        assert_eq!(
            hex(track_outline(false, ToggleGroupFrame::Default)),
            "#8d8d8d"
        );
        assert_eq!(
            hex(track_outline(true, ToggleGroupFrame::Default)),
            "#c6c6c6"
        );
        // G100, where border-subtle and border-disabled differ.
        crate::theme::tests_set(CarbonTheme::G100);
        let subtle = Some(color(ColorRole::BorderSubtle01));
        let dim = Some(color(ColorRole::BorderDisabled));
        assert_ne!(subtle, dim);
        let mut states = vec![SegmentState::default(); 3];
        assert_eq!(divider(0, &states), None);
        assert_eq!(divider(1, &states), subtle);
        states[1].selected = true;
        assert_eq!(divider(1, &states), None);
        assert_eq!(divider(2, &states), None);
        // `:disabled::before { background-color: var(--border-disabled) }` wins
        // over the selected neighbour.
        states[2].disabled = true;
        assert_eq!(divider(2, &states), dim);
        states[1].disabled = true;
        assert_eq!(divider(2, &states), None);
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
