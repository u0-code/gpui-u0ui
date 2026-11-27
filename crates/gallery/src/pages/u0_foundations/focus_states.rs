//! u0 · Foundations › FocusStates: focus, current and selected, stacking
//! and distinguishable, with a live row and the reference legend.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::foundations::StateRow;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| FocusStatesPage {
        current: true,
        selected: false,
        focus: false,
    })
    .into()
}

struct FocusStatesPage {
    current: bool,
    selected: bool,
    focus: bool,
}

fn rows(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .bg(color(ColorRole::Layer01))
        .child(StateRow::new((prefix, 0usize), "Default"))
        .child(StateRow::new((prefix, 1usize), "Current (open in the thread view)").current(true))
        .child(StateRow::new((prefix, 2usize), "Selected (multi-select)").selected(true))
        .child(StateRow::new((prefix, 3usize), "Keyboard focus").focus_ring(true))
        .child(
            StateRow::new((prefix, 4usize), "Current + focus")
                .current(true)
                .focus_ring(true),
        )
}

fn legend() -> impl IntoElement {
    // `.g-statelegend dl { grid-template-columns: auto 1fr; gap: 4px 12px; font-size: 0.75rem }`.
    let entries = [
        (
            "state-focus",
            "2px 描边，只在键盘导航（:focus-visible）时出现",
        ),
        (
            "state-current-*",
            "正在打开的那一项：底色 + 左侧 3px 指示条 + 600 字重，aria-current",
        ),
        (
            "state-selected-background",
            "被选中参与批量操作，aria-selected / aria-pressed",
        ),
        ("hover", "layer-hover-01，不改变语义"),
    ];
    let term_w = term_column_px(entries.map(|(t, _)| t));
    // The `dl` stretches to the grid row (the rows' height), and its `auto`
    // tracks share the free space: each row grows by the same amount.
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_02)
        .text_size(rems(0.75))
        .line_height(rems(1.))
        .children(entries.map(|(t, d)| {
            div()
                .flex_grow_1()
                .flex()
                .items_start()
                .gap(tokens::SPACING_04)
                .child(
                    div()
                        .w(rems(term_w / 16.))
                        .flex_none()
                        .type_style(TypeStyle::Code01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(t),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_color(color(ColorRole::TextPrimary))
                        .child(d),
                )
        }))
}

/// The `dt` column's `auto` width: its longest term on one line, in IBM Plex
/// Mono 12px (`code-01`; every glyph advances 600/1000 em).
pub fn term_column_px<'a>(terms: impl IntoIterator<Item = &'a str>) -> f32 {
    let longest = terms
        .into_iter()
        .map(|t| t.chars().count())
        .max()
        .unwrap_or(0);
    longest as f32 * 0.6 * 12.
}

/// `components/FocusStates/preview.html`: rows beside the legend
/// (`.g-statelegend { display: grid; grid-template-columns: 1fr 1fr; gap: 16px }`;
/// both cells stretch to the taller one).
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .items_stretch()
        .gap(tokens::SPACING_05)
        .child(div().flex_1().min_w(rems(16.)).child(rows(prefix)))
        .child(
            div()
                .flex_1()
                .min_w(rems(16.))
                .flex()
                .flex_col()
                .child(legend()),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(rows("thumb-state"))
        .into_any_element()
}

impl Render for FocusStatesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .w(rems(22.))
            .max_w_full()
            .bg(color(ColorRole::Layer01))
            .child(StateRow::new("state-live-0", "Add dark theme tokens"))
            .child(
                StateRow::new("state-live-1", "Fix flaky date picker test")
                    .current(self.current)
                    .selected(self.selected)
                    .focus_ring(self.focus),
            )
            .child(StateRow::new("state-live-2", "Write migration guide"));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "state-current",
                "current",
                self.current,
                controls::setter(&entity, |this: &mut Self, on| this.current = on),
            ))
            .child(controls::toggle(
                "state-selected",
                "selected",
                self.selected,
                controls::setter(&entity, |this: &mut Self, on| this.selected = on),
            ))
            .child(controls::toggle(
                "state-focus",
                "focusRing",
                self.focus,
                controls::setter(&entity, |this: &mut Self, on| this.focus = on),
            ));
        Body::new()
            .playground(live)
            .hint("Tab to the rows for the real focus-visible ring")
            .config(config)
            .example(controls::block("Reference preview", reference("state-ref")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_column_fits_the_longest_term() {
        // "state-selected-background" stays on one line: 25 × 7.2px.
        let w = term_column_px(["state-focus", "state-selected-background", "hover"]);
        assert!((w - 180.).abs() < 0.01, "{w}");
    }
}
