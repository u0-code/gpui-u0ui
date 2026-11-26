//! Forms › DatePicker: the reference Start date and Billing period pickers
//! and a configurable live picker.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::FieldSize;
use u0ui::components::calendar::{CalendarDate, Selection};
use u0ui::components::date_picker::{DatePicker, DatePickerMode};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DatePickerPage {
        mode: 0,
        size: 1,
        value: Selection::default(),
        bounded: false,
        invalid: false,
        warn: false,
        disabled: false,
        readonly: false,
    })
    .into()
}

struct DatePickerPage {
    mode: usize,
    size: usize,
    value: Selection,
    bounded: bool,
    invalid: bool,
    warn: bool,
    disabled: bool,
    readonly: bool,
}

fn describe(selection: Selection) -> String {
    let f = |d: Option<CalendarDate>| d.map_or("—".to_string(), |d| d.format("m/d/Y"));
    format!("start = {}, end = {}", f(selection.start), f(selection.end))
}

impl Render for DatePickerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut DatePickerPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let set = |f: fn(&mut DatePickerPage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };
        let mode = DatePickerMode::ALL[self.mode];

        // `components/DatePicker/preview.html` (`cu-row`, align-items: flex-start).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_start()
                    .gap(tokens::SPACING_05)
                    .child(
                        DatePicker::new("ref-start")
                            .label("Start date")
                            .placeholder("mm/dd/yyyy"),
                    )
                    .child(
                        DatePicker::new("ref-billing")
                            .mode(DatePickerMode::Range)
                            .label("Billing period")
                            .end_label("End date")
                            .placeholder("mm/dd/yyyy"),
                    ),
            );

        let mut picker = DatePicker::new(SharedString::from(format!("live-{}", mode.name())))
            .mode(mode)
            .label(if mode == DatePickerMode::Range {
                "Start date"
            } else {
                "Due date"
            })
            .end_label("End date")
            .placeholder("mm/dd/yyyy")
            .value(self.value)
            .helper_text("Type a date or pick one.")
            .size(FieldSize::ALL[self.size])
            .invalid(self.invalid)
            .invalid_text("Enter a date in mm/dd/yyyy.")
            .warn(self.warn)
            .warn_text("This date falls on a holiday.")
            .disabled(self.disabled)
            .readonly(self.readonly)
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
            picker = picker
                .min_date(CalendarDate::new(2026, 11, 5))
                .max_date(CalendarDate::new(2026, 12, 20));
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(picker)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(describe(self.value))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "mode",
                controls::choices(
                    "dp-mode",
                    &DatePickerMode::ALL.map(|m| m.name()),
                    self.mode,
                    set(|this, i| {
                        this.mode = i;
                        this.value = Selection::default();
                    }),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "dp-size",
                    &FieldSize::ALL.map(|s| s.name()),
                    self.size,
                    set(|this, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "dp-bounded",
                "minDate 11/05 · maxDate 12/20",
                self.bounded,
                toggle(|this, on| this.bounded = on),
            ))
            .child(controls::toggle(
                "dp-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "dp-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "dp-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "dp-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}
