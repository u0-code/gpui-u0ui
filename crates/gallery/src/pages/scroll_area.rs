//! Layout › ScrollArea: the reference release list (280×180, 30 versions)
//! and a configurable live area (axes, focusability, wide content).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::scroll_area::{ScrollArea, ScrollAxes, ScrollBar, ScrollBarOrientation};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ScrollAreaPage {
        axes: 0,
        focusable: true,
        wide: false,
        rows: 1,
        scrollbar: 0,
    })
    .into()
}

struct ScrollAreaPage {
    axes: usize,
    focusable: bool,
    wide: bool,
    rows: usize,
    scrollbar: usize,
}

const ROWS: [usize; 3] = [3, 30, 100];
const ROW_NAMES: [&str; 3] = ["3", "30", "100"];

/// `preview.html`: a 16px-padded column of `v1.N.0` rows, each `padding: 6px 0`
/// over a 1px `border-subtle-01` rule.
fn releases(count: usize, wide: bool) -> impl IntoElement {
    div()
        .p(tokens::SPACING_05)
        .when(wide, |this| this.w(rems(40.)))
        .children((0..count).map(|i| {
            div()
                .py(rems(0.375))
                .border_b_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child(format!("v1.{}.0", count - i))
        }))
}

/// The preview's `outline: 1px solid var(--border-subtle-01)` (outside the box).
fn outlined(area: impl IntoElement) -> impl IntoElement {
    div()
        .flex_none()
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(area)
}

impl Render for ScrollAreaPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        let reference = div().flex().child(outlined(
            ScrollArea::new("scroll-area-ref")
                .h(rems(11.25))
                .w(rems(17.5))
                .child(releases(30, false)),
        ));

        let axes = ScrollAxes::ALL[self.axes];
        let live = div().flex().child(outlined(
            ScrollArea::new(("scroll-area-live", self.axes * 10 + self.wide as usize))
                .axes(axes)
                .focusable(self.focusable)
                .h(rems(11.25))
                .w(rems(17.5))
                .child(releases(ROWS[self.rows], self.wide))
                // shadcn markup ports as is; the bar is drawn by the area.
                .child(ScrollBar::new().orientation(ScrollBarOrientation::ALL[self.scrollbar])),
        ));

        let axes_names = ScrollAxes::ALL.map(|a| a.name());
        let bar_names = ScrollBarOrientation::ALL.map(|o| o.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "axes (overflow)",
                controls::choices(
                    "scroll-axes",
                    &axes_names,
                    self.axes,
                    controls::setter(&entity, |this: &mut ScrollAreaPage, i| this.axes = i),
                ),
            ))
            .child(controls::field(
                "rows",
                controls::choices(
                    "scroll-rows",
                    &ROW_NAMES,
                    self.rows,
                    controls::setter(&entity, |this: &mut ScrollAreaPage, i| this.rows = i),
                ),
            ))
            .child(controls::field(
                "ScrollBar orientation (no-op)",
                controls::choices(
                    "scroll-bar",
                    &bar_names,
                    self.scrollbar,
                    controls::setter(&entity, |this: &mut ScrollAreaPage, i| this.scrollbar = i),
                ),
            ))
            .child(controls::toggle(
                "scroll-focusable",
                "focusable",
                self.focusable,
                controls::setter(&entity, |this: &mut ScrollAreaPage, on| this.focusable = on),
            ))
            .child(controls::toggle(
                "scroll-wide",
                "wide content (640px)",
                self.wide,
                controls::setter(&entity, |this: &mut ScrollAreaPage, on| this.wide = on),
            ));

        let note = div()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .child(
                "Tab into the area for the 2px focus outline; arrows, Page Up/Down, Space, \
                 Home and End scroll it.",
            );

        Body::new()
            .playground(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_04)
                    .child(live)
                    .child(note),
            )
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
