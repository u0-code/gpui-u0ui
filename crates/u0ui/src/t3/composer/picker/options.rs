//! [`super::ModelPicker`]'s options column (`.t3-mpick-effort`): a level
//! option's rows, Speed's switch, and each other option as a row opening a
//! Carbon menu of its choices.
use super::{MODEL_PICKER_GEOMETRY, ModelPicker, ModelPickerTexts, PickerOption, row_fill};
use crate::components::menu::{self, MenuItem, MenuSize};
use crate::components::overlay;
use crate::components::switch::{Switch, SwitchSize};
use crate::t3::composer::controls::{EffortGlyph, EffortLevel};
use crate::t3::press::{self, rem};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.t3-mpick-h { padding: spacing-03 spacing-04 spacing-02; font-size: 0.75rem; color: text-secondary }`.
fn header(text: SharedString) -> gpui_kit::Div {
    div()
        .pt(rem(8.))
        .px(rem(12.))
        .pb(rem(4.))
        .text_size(rem(12.))
        .line_height(rem(16.))
        .text_color(color(ColorRole::TextSecondary))
        .child(text)
}

/// gemini-B: `.t3-erow + .t3-mpick-h { margin-top: spacing-04; padding-top:
/// spacing-04; border-top: 1px solid border-subtle-01 }`, a group after another.
fn header_after(text: SharedString) -> gpui_kit::Div {
    header(text)
        .mt(rem(12.))
        .pt(rem(12.))
        .border_t_1()
        .border_color(color(ColorRole::BorderSubtle01))
}

impl ModelPicker {
    /// The options column: the levels, Speed, then every other option.
    pub(super) fn options_column(
        &mut self,
        texts: &ModelPickerTexts,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let id = self.id.clone();
        let mut column = div()
            .id((id.clone(), "options"))
            .flex()
            .flex_col()
            .flex_none()
            .w(rem(MODEL_PICKER_GEOMETRY.1))
            .py(rem(8.))
            .border_l_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .overflow_y_scroll();
        let mut groups = 0;
        if let Some(levels) = self.levels.take() {
            let of = levels.choices.len();
            column = column.child(header(levels.name.clone()));
            for (at, c) in levels.choices.into_iter().enumerate() {
                let key = |s: &str| -> ElementId {
                    (id.clone(), SharedString::from(format!("{s}-{at}"))).into()
                };
                let (hovered, hover) = press::hover(key("erow-hover"), window, cx);
                let (focus, ring) = press::focus(key("erow-focus"), window, cx);
                let f = self.on_option_change.clone();
                let (option, value) = (levels.id.clone(), c.value.clone());
                // `.t3-erow { display: flex; align-items: center; gap: spacing-03; height: size-sm;
                // padding: 0 spacing-04; color: text-primary; font-size: 0.875rem }`.
                column = column.child(
                    gpui_kit::base::Button::new(key("erow"))
                        .track_focus(&focus)
                        .selected(c.on)
                        .relative()
                        .flex()
                        .flex_row()
                        .flex_none()
                        .items_center()
                        .justify_start()
                        .w_full()
                        .gap(rem(8.))
                        .h(rem(32.))
                        .px(rem(12.))
                        .text_size(rem(14.))
                        .line_height(rem(18.))
                        .text_color(color(ColorRole::TextPrimary))
                        .cursor_pointer()
                        .when_some(row_fill(c.on, hovered), |this, bg| this.bg(bg))
                        .on_hover(press::track_hover(hover))
                        .when_some(f, |this, f| {
                            this.on_click(move |_, w, cx| f(option.clone(), value.clone(), w, cx))
                        })
                        .child(EffortGlyph::new(EffortLevel::among(at, of)))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .child(c.name),
                        )
                        // `.t3-erow > svg { margin-left: auto; fill: interactive }`.
                        .when(c.on, |this| {
                            this.child(press::icon(
                                CarbonIcon::Checkmark,
                                16.,
                                color(ColorRole::Interactive),
                            ))
                        })
                        .when(ring, |this| this.child(press::ring())),
                );
            }
            groups += 1;
        }
        if let Some(speed) = self.speed.take() {
            let turn = self.on_speed_change.clone();
            let head = texts.speed.clone();
            column = column
                .child(if groups > 0 {
                    header_after(head)
                } else {
                    header(head)
                })
                // `.t3-mpick-effort .cds--toggle { padding: 0 spacing-04 }`, `size="sm"`.
                .child(
                    div().px(rem(12.)).child(
                        // `<Cb.Toggle labelText=… hideLabel>`: with `hideLabel`
                        // Carbon shows `labelText` beside the track in both
                        // states; the label is the option's own name.
                        Switch::new((id.clone(), "speed"))
                            .size(SwitchSize::Sm)
                            .checked(speed.on)
                            .label_off(speed.label.clone())
                            .label_on(speed.label)
                            .on_checked_change(move |on, w, cx| {
                                if let Some(f) = turn.as_ref() {
                                    f(on, w, cx)
                                }
                            }),
                    ),
                );
            groups += 1;
        }
        for (n, o) in std::mem::take(&mut self.options).into_iter().enumerate() {
            let row = self.option_row(n, o, window, cx);
            column = column.child(if groups > 0 && n == 0 {
                div()
                    .mt(rem(12.))
                    .pt(rem(4.))
                    .border_t_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(row)
            } else {
                div().child(row)
            });
        }
        column.into_any_element()
    }

    /// An option beyond the levels and Speed: its name over its value, a
    /// Carbon menu of its choices (a radio group) hanging from it.
    fn option_row(
        &self,
        n: usize,
        o: PickerOption,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let key: ElementId = (self.id.clone(), SharedString::from(format!("option-{n}"))).into();
        let open_change: overlay::OpenChange = {
            let (f, oid) = (self.on_option_open.clone(), o.id.clone());
            Rc::new(move |open, w, cx| {
                if let Some(f) = f.as_ref() {
                    f(open.then(|| oid.clone()), w, cx)
                }
            })
        };
        let openness = overlay::openness(
            &key,
            Some(self.open_option.as_ref() == Some(&o.id)),
            false,
            Some(open_change),
            window,
            cx,
        );
        let open = openness.open;
        let set_open = openness.setter();
        let (hovered, hover) = press::hover((key.clone(), "hover"), window, cx);
        let current = o.current().unwrap_or_default();
        // `.t3-erow`'s row, two lines: the option's name in `label-01`
        // `text-secondary` over its value.
        let line = div()
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(8.))
            .w_full()
            .h(rem(44.))
            .px(rem(12.))
            .cursor_pointer()
            .when_some(row_fill(open, hovered), |this, bg| this.bg(bg))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(rem(12.))
                            .line_height(rem(16.))
                            .text_color(color(ColorRole::TextSecondary))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(o.name.clone()),
                    )
                    .child(
                        div()
                            .text_size(rem(14.))
                            .line_height(rem(18.))
                            .text_color(color(ColorRole::TextPrimary))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(current),
                    ),
            )
            .child(press::icon(
                CarbonIcon::ChevronRight,
                16.,
                color(ColorRole::IconSecondary),
            ));
        let trigger = overlay::trigger_wrapper(
            (key.clone(), "trigger"),
            div()
                .id((key.clone(), "line"))
                .w_full()
                .on_hover(press::track_hover(hover))
                .child(line)
                .into_any_element(),
            {
                let set_open = set_open.clone();
                Rc::new(move |w, cx| set_open(!open, w, cx))
            },
        )
        .w_full()
        .into_any_element();
        let picked = o.choices.iter().position(|c| c.on);
        let values: Vec<SharedString> = o.choices.iter().map(|c| c.value.clone()).collect();
        let (f, oid) = (self.on_option_change.clone(), o.id.clone());
        let spec = menu::MenuSpec {
            items: vec![
                MenuItem::radio_group(o.name.clone(), o.choices.iter().map(|c| c.name.clone()))
                    .selected_item(picked)
                    .on_select_item(move |i, w, cx| {
                        if let (Some(f), Some(v)) = (f.as_ref(), values.get(i)) {
                            f(oid.clone(), v.clone(), w, cx)
                        }
                    }),
            ],
            size: MenuSize::Sm,
            label: o.name,
            open,
            set_open,
            min_width: None,
            on_unhandled_key: None,
        };
        menu::anchored_menu(&key, trigger, spec, window, cx)
    }
}

impl EffortLevel {
    /// The glyph for a level at `at` among `of` an agent states, lowest
    /// first, whatever it names them: the lowest lights one bar, the
    /// highest five, the rest spread between; a lone level the middle.
    pub fn among(at: usize, of: usize) -> Self {
        if of < 2 {
            return Self::Medium;
        }
        let bars = Self::ALL.len() - 1;
        Self::ALL[(at.min(of - 1) * bars + (of - 1) / 2) / (of - 1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effort_bars_follow_where_a_level_stands_among_the_agents_own() {
        let three: Vec<_> = (0..3).map(|at| EffortLevel::among(at, 3)).collect();
        assert_eq!(
            three,
            [EffortLevel::Minimal, EffortLevel::Medium, EffortLevel::Max]
        );
        let five: Vec<_> = (0..5).map(|at| EffortLevel::among(at, 5)).collect();
        assert_eq!(five, EffortLevel::ALL);
        assert_eq!(EffortLevel::among(1, 4), EffortLevel::Low);
        assert_eq!(EffortLevel::among(2, 4), EffortLevel::High);
        assert_eq!(EffortLevel::among(0, 1), EffortLevel::Medium);
        // Codex's six, Low to Ultra: the lowest one bar, the highest five.
        assert_eq!(EffortLevel::among(0, 6), EffortLevel::Minimal);
        assert_eq!(EffortLevel::among(5, 6), EffortLevel::Max);
    }
}
