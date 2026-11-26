//! Forms › Calendar: the reference inline calendar (11/26/2026) and a
//! configurable live one (single/range, min/max).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::calendar::{Calendar, CalendarDate, CalendarMode, Selection};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CalendarPage {
        mode: 0,
        bounded: false,
        value: Selection::single(CalendarDate::new(2026, 11, 26)),
    })
    .into()
}

struct CalendarPage {
    mode: usize,
    bounded: bool,
    value: Selection,
}

fn describe(selection: Selection) -> String {
    let f = |d: Option<CalendarDate>| d.map_or("—".to_string(), |d| d.format("m/d/Y"));
    match selection.end {
        Some(_) => format!("{} – {}", f(selection.start), f(selection.end)),
        None => f(selection.start),
    }
}

impl Render for CalendarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mode = CalendarMode::ALL[self.mode];

        // `components/Calendar/preview.html`: `value: '11/26/2026'`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                Calendar::new("ref-calendar")
                    .default_value(Selection::single(CalendarDate::new(2026, 11, 26)))
                    .nav_labels("Previous month", "Next month"),
            );

        let mut live_calendar = Calendar::new(SharedString::from(format!("live-{}", mode.name())))
            .mode(mode)
            .value(self.value)
            .nav_labels("Previous month", "Next month")
            .on_value_change({
                let entity = entity.clone();
                move |value, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.value = value;
                        cx.notify();
                    })
                }
            });
        if self.bounded {
            live_calendar = live_calendar
                .min_date(CalendarDate::new(2026, 11, 5))
                .max_date(CalendarDate::new(2026, 12, 20));
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(live_calendar)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!(
                        "value = {}",
                        describe(self.value)
                    ))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "mode",
                controls::choices(
                    "cal-mode",
                    &CalendarMode::ALL.map(|m| m.name()),
                    self.mode,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.mode = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "cal-bounded",
                "minDate 11/05 · maxDate 12/20",
                self.bounded,
                {
                    let entity = entity.clone();
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.bounded = on;
                            cx.notify();
                        })
                    }
                },
            ));

        Body::new()
            .playground(live)
            .hint("Tab into the grid: arrows, PageUp/PageDown, Enter")
            .config(config)
            .example(reference)
    }
}
