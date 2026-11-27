//! Compact u0 form parts the admin cards share (`src/ghost.css` "Fields
//! (compact)" and friends): the `.g-in` / `.g-ta` text boxes, the
//! `.g-inline-check` checkbox and the `.g-seg` segmented radio group.
//! Text editing is Kit's ([`InputState`], [`TextareaState`]); these draw the
//! u0 field around it.
use crate::{
    components::form,
    density,
    icons::{CarbonIcon, Icon},
    t3::base::radio_step,
    theme::color,
    tokens::{self, ColorRole, FontFamilyKey},
    typography::font,
    u0::primitives::parts::{self, rem},
};
use gpui_kit::{
    AnyElement, App, CursorStyle, Div, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _, Size,
        input::{InputState, Textarea as KitTextarea, TextareaState},
    },
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// `.g-in, .g-select, .g-ta { … background: var(--field-01) }`; inside a
/// workflow step card `field-02` (`.g-wfe-card .g-in { background:
/// var(--field-02, var(--field-01)) }`).
pub fn field_fill(on_layer: bool) -> Hsla {
    color(if on_layer {
        ColorRole::Field02
    } else {
        ColorRole::Field01
    })
}

/// `.g-ta { padding: var(--spacing-03) var(--spacing-04); line-height:
/// 1.42857 }` at 14px: 20px lines.
pub const TA_LINE: f32 = 20.;

/// The `.g-ta` box height for `rows` lines: rows × 20 + 8 + 8 + the 1px rule.
pub fn ta_height(rows: usize) -> f32 {
    rows as f32 * TA_LINE + 16. + 1.
}

/// `.g-in`: a `density-field-md` (32px compact) single-line box, `padding: 0
/// var(--spacing-04)`, filled `fill`, a 1px `border-strong-01` bottom rule,
/// 14px text (12px mono with `.g-mono`), and the 2px `focus` outline while
/// focused (`.g-in:focus { outline-color: var(--focus) }`). `placeholder`
/// draws in `text-placeholder` while empty.
pub fn line_input(
    state: &Entity<InputState>,
    mono: bool,
    fill: Hsla,
    placeholder: Option<SharedString>,
    window: &Window,
    cx: &App,
) -> Div {
    let focused = state.read(cx).focus_handle(cx).is_focused(window);
    let empty = state.read(cx).value().is_empty();
    let editor = if mono {
        parts::mono_editor(state)
    } else {
        parts::editor(state)
    };
    div()
        .relative()
        .flex()
        .items_center()
        .w_full()
        .min_w_0()
        .h(density::current().field())
        .px(tokens::SPACING_04)
        .bg(fill)
        .border_b_1()
        .border_color(color(ColorRole::BorderStrong01))
        .text_color(color(ColorRole::TextPrimary))
        .child(editor.text_color(color(ColorRole::TextPrimary)))
        .when_some(placeholder.filter(|_| empty), |this, p| {
            this.child(
                div()
                    .absolute()
                    .left(tokens::SPACING_04)
                    .text_size(rem(if mono { 12. } else { 14. }))
                    .when(mono, |d| d.font(font(FontFamilyKey::Mono, 400)))
                    .text_color(color(ColorRole::TextPlaceholder))
                    .whitespace_nowrap()
                    .child(p),
            )
        })
        .when(focused, |this| this.child(parts::focus_ring()))
}

/// `.g-ta`: a `rows`-line text box (`padding: var(--spacing-03)
/// var(--spacing-04)`, 20px lines), otherwise as [`line_input`].
pub fn text_area(
    state: &Entity<TextareaState>,
    rows: usize,
    fill: Hsla,
    label: Option<SharedString>,
    window: &Window,
    cx: &App,
) -> Div {
    let focused = state.read(cx).focus_handle(cx).is_focused(window);
    // Kit insets multi-line text by its XSmall padding (2px block, 4px inline).
    let editor = KitTextarea::new(state)
        .appearance(false)
        .with_size(Size::XSmall)
        .when_some(label, |t, l| t.aria_label(l))
        .w_full()
        .h(rem(rows as f32 * TA_LINE + 4.))
        .text_size(rem(14.))
        .line_height(rem(TA_LINE))
        .text_color(color(ColorRole::TextPrimary));
    // Tab and Shift+Tab move focus, as in the reference's `<textarea>`.
    crate::components::textarea::tab_moves_focus(div())
        .relative()
        .w_full()
        .min_w_0()
        .h(rem(ta_height(rows)))
        .pt(px(6.))
        .pb(px(6.))
        .px(px(8.))
        .bg(fill)
        .border_b_1()
        .border_color(color(ColorRole::BorderStrong01))
        .child(editor)
        .when(focused, |this| this.child(parts::focus_ring()))
}

type CheckHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// `.g-check` colors for `checked`: (fill, border, glyph). `{ border: 1px
/// solid var(--icon-primary); background: transparent }`, `input:checked +
/// .g-check { background: var(--icon-primary) }`, the check
/// `fill: var(--icon-inverse)`, shown only when checked.
pub fn check_colors(checked: bool) -> (Hsla, Hsla, Option<Hsla>) {
    let primary = color(ColorRole::IconPrimary);
    if checked {
        (primary, primary, Some(color(ColorRole::IconInverse)))
    } else {
        (Hsla::transparent_black(), primary, None)
    }
}

/// `.g-inline-check`: the compact checkbox row (a 16px `.g-check` box with a
/// 2px radius, then its label), `gap: var(--spacing-03)`, `min-height:
/// 1.5rem`, 14px text. The 2px `focus` ring sits 1px outside the box
/// (`input:focus-visible + .g-check { outline-offset: 1px }`). Controlled.
#[derive(IntoElement)]
pub struct InlineCheck {
    id: ElementId,
    checked: bool,
    label: Option<AnyElement>,
    top: bool,
    on_change: Option<CheckHandler>,
}

impl InlineCheck {
    /// An unchecked box.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            label: None,
            top: false,
            on_change: None,
        }
    }

    /// Checked.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// The label (text, or a title + description column).
    pub fn label(mut self, label: impl IntoElement) -> Self {
        self.label = Some(label.into_any_element());
        self
    }

    /// Top-align the box with a 2px nudge (`.g-sheet-check { align-items:
    /// flex-start } .g-sheet-check .g-check { margin-top: 2px }`).
    pub fn align_top(mut self, top: bool) -> Self {
        self.top = top;
        self
    }

    /// Flipped.
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for InlineCheck {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = parts::keyed_focus((self.id.clone(), "focus"), window, cx);
        let ring = parts::focus_visible(&focus, window);
        let (fill, border, glyph) = check_colors(self.checked);
        let checked = self.checked;
        let on_change = self.on_change.clone();
        gpui_kit::base::Button::new(self.id.clone())
            .track_focus(&focus)
            .flex()
            .when(self.top, |b| b.items_start())
            .when(!self.top, |b| b.items_center())
            .justify_start()
            .gap(tokens::SPACING_03)
            .min_h(rem(24.))
            .text_size(rem(14.))
            // The inherited body `line-height: 1.28572` (18px at 14px): the
            // label's first line sits level with the 2px-nudged box.
            .line_height(gpui_kit::relative(crate::t3::craft::BODY_LINE))
            .text_color(color(ColorRole::TextPrimary))
            .cursor(CursorStyle::PointingHand)
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(16.))
                    .when(self.top, |d| d.mt(px(2.)))
                    .rounded(px(2.))
                    .border_1()
                    .border_color(border)
                    .bg(fill)
                    .when_some(glyph, |d, g| {
                        d.child(
                            Icon::carbon(CarbonIcon::Checkmark)
                                .size_length(rem(12.))
                                .color(g),
                        )
                    })
                    .when(ring, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top(px(-4.))
                                .left(px(-4.))
                                .right(px(-4.))
                                .bottom(px(-4.))
                                .rounded(px(2.))
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    }),
            )
            .children(self.label)
            .on_click(move |_, w, cx| {
                if let Some(f) = &on_change {
                    f(!checked, w, cx);
                }
            })
    }
}

type SegHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// `.g-seg-btn` colors (bg, text): transparent / `text-secondary`; hover
/// `layer-hover-01` / `text-primary`; `--on` `layer-selected-inverse` /
/// `text-inverse`.
pub fn seg_colors(on: bool, hovered: bool) -> (Hsla, Hsla) {
    if on {
        (
            color(ColorRole::LayerSelectedInverse),
            color(ColorRole::TextInverse),
        )
    } else if hovered {
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::TextPrimary),
        )
    } else {
        (Hsla::transparent_black(), color(ColorRole::TextSecondary))
    }
}

/// `.g-seg`: a segmented radio group (`role="radiogroup"`): `field-sm`-high
/// buttons, `padding: 0 var(--spacing-04)`, 12px text, a 1px
/// `border-inverse` box with shared inner rules; arrows / Home / End move the
/// choice (`radioKeys`), and only the chosen button is a Tab stop.
#[derive(IntoElement)]
pub struct Segmented {
    id: ElementId,
    options: Vec<SharedString>,
    selected: usize,
    on_select: Option<SegHandler>,
}

impl Segmented {
    /// A group of `options` with `selected` chosen.
    pub fn new(
        id: impl Into<ElementId>,
        options: impl IntoIterator<Item = impl Into<SharedString>>,
        selected: usize,
    ) -> Self {
        Self {
            id: id.into(),
            options: options.into_iter().map(Into::into).collect(),
            selected,
            on_select: None,
        }
    }

    /// A choice was made.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Segmented {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let n = self.options.len();
        let selected = self.selected;
        let focuses: Vec<_> = (0..n)
            .map(|i| {
                parts::keyed_focus(
                    (self.id.clone(), SharedString::from(format!("focus-{i}"))),
                    window,
                    cx,
                )
            })
            .collect();
        let on_select = self.on_select.clone();
        let border = color(ColorRole::BorderInverse);
        let height = density::current().field_sm();
        let buttons = self.options.into_iter().enumerate().map(|(i, label)| {
            let hover = window.use_keyed_state(
                (self.id.clone(), SharedString::from(format!("hover-{i}"))),
                cx,
                |_, _| false,
            );
            let on = i == selected;
            let (bg, fg) = seg_colors(on, *hover.read(cx));
            let ring = parts::focus_visible(&focuses[i], window);
            let pick = on_select.clone();
            gpui_kit::base::Button::new((self.id.clone(), SharedString::from(format!("seg-{i}"))))
                .track_focus(&focuses[i])
                .tab_stop(on)
                .relative()
                .flex()
                .items_center()
                .h(height)
                .px(tokens::SPACING_04)
                .border_t_1()
                .border_b_1()
                .border_r_1()
                .when(i == 0, |b| b.border_l_1())
                .border_color(border)
                .bg(bg)
                .text_color(fg)
                .text_size(rem(12.))
                .whitespace_nowrap()
                .cursor(CursorStyle::PointingHand)
                .on_hover(move |h, _, cx| {
                    hover.update(cx, |v, cx| {
                        if *v != *h {
                            *v = *h;
                            cx.notify();
                        }
                    })
                })
                .child(label)
                .when(ring, |b| b.child(parts::focus_ring()))
                .on_click(move |_, w, cx| {
                    if let Some(f) = &pick {
                        f(i, w, cx);
                    }
                })
        });
        let focuses_k = focuses.clone();
        let on_key = self.on_select.clone();
        div()
            .id(self.id.clone())
            .flex()
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                if let Some(next) = radio_step(&e.keystroke.key, n, Some(selected)) {
                    cx.stop_propagation();
                    focuses_k[next].focus(window, cx);
                    if let Some(f) = &on_key {
                        f(next, window, cx);
                    }
                }
            })
            .children(buttons)
    }
}

/// The outline a box shows (`form::inset_outline`), re-exported for parts
/// that draw their own field.
pub fn outline(width: f32, c: Hsla) -> Div {
    form::inset_outline(width, c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn check_follows_g_check() {
        crate::theme::tests_set(CarbonTheme::White);
        let (fill, border, glyph) = check_colors(true);
        assert_eq!(fill, color(ColorRole::IconPrimary));
        assert_eq!(border, color(ColorRole::IconPrimary));
        assert_eq!(glyph, Some(color(ColorRole::IconInverse)));
        let (fill, _, glyph) = check_colors(false);
        assert_eq!(fill, Hsla::transparent_black());
        assert_eq!(glyph, None);
    }

    #[test]
    fn segments_follow_g_seg_btn() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(
            seg_colors(true, false),
            (
                color(ColorRole::LayerSelectedInverse),
                color(ColorRole::TextInverse)
            )
        );
        assert_eq!(
            seg_colors(false, true),
            (
                color(ColorRole::LayerHover01),
                color(ColorRole::TextPrimary)
            )
        );
        assert_eq!(seg_colors(false, false).1, color(ColorRole::TextSecondary));
    }

    #[test]
    fn textarea_height_follows_g_ta() {
        assert_eq!(ta_height(2), 57.);
        assert_eq!(
            field_fill(true),
            color(ColorRole::Field02),
            "inside a step card"
        );
    }
}
