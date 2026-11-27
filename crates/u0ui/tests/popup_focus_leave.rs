//! Non-modal popups close when focus leaves them, as their references do
//! (design review FG2B #2, the audit after FilterMenu):
//!
//! - Popover (Carbon `Popover`: `onRequestClose` on `focusout` outside it
//!   and its trigger). Tab follows the DOM, where the content sits right
//!   after the trigger: Tab on the open trigger enters it, Shift+Tab from
//!   its first stop returns to the trigger (still open), Tab from its last
//!   stop closes it and moves to the stop after the trigger.
//! - Select (Carbon `Dropdown`, Downshift `ToggleButtonBlur`): focus leaving
//!   the trigger closes the menu.
//! - DatePicker (Carbon `DatePicker` with flatpickr): Tab from the open
//!   field moves into the calendar, Shift+Tab there returns to the field,
//!   Tab there closes it and moves on; Shift+Tab from the field closes it.
//!
//! Focus moving into a surface nested in the open one (a popover inside a
//! popover) is not leaving it.
//!
//! Before, Base's window-level Tab moved focus away and left each open.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{Button, DatePicker, Popover, Select};

#[derive(Clone, Copy)]
enum Kind {
    Popover,
    Nested,
    DateInPopover,
    Select,
    DatePicker,
}

struct Page {
    kind: Kind,
    before: FocusHandle,
    after: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let popup = match self.kind {
            Kind::Popover => Popover::new("pop", Button::new("pop-trigger", "Dimensions"))
                .child(Button::new("pop-inner", "Apply"))
                .into_any_element(),
            Kind::Nested => Popover::new("pop", Button::new("pop-trigger", "Filters"))
                .child(
                    Popover::new("inner", Button::new("inner-trigger", "More"))
                        .child(Button::new("inner-btn", "Apply")),
                )
                .into_any_element(),
            Kind::DateInPopover => Popover::new("pop", Button::new("pop-trigger", "Schedule"))
                .child(Button::new("pop-inner", "Today"))
                .child(DatePicker::new("dp").label("Start"))
                .into_any_element(),
            Kind::Select => Select::new("sel")
                .items([("a", "Option A"), ("b", "Option B")])
                .into_any_element(),
            Kind::DatePicker => DatePicker::new("dp").label("Start").into_any_element(),
        };
        div()
            .size_full()
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .w(px(40.))
                    .h(px(20.))
                    .track_focus(&self.before.clone().tab_stop(true)),
            )
            .child(popup)
            .child(
                div()
                    .w(px(40.))
                    .h(px(20.))
                    .track_focus(&self.after.clone().tab_stop(true)),
            )
    }
}

struct Harness {
    window: AnyWindowHandle,
    page: gpui_kit::Entity<Page>,
    surface: ElementId,
}

impl Harness {
    fn new(kind: Kind, cx: &mut TestAppContext) -> Self {
        cx.update(|cx| {
            gpui_kit::init(cx);
            u0ui::init(cx);
        });
        let (window, page) = cx.update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::default(),
                        size: size(px(900.), px(700.)),
                    })),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    let before = cx.focus_handle();
                    before.focus(window, cx);
                    cx.new(|cx| Page {
                        kind,
                        before,
                        after: cx.focus_handle(),
                    })
                },
            )
            .expect("open test window")
        });
        let surface: ElementId = match kind {
            // The content's first control shows while it is open.
            Kind::Popover | Kind::DateInPopover => "pop-inner".into(),
            Kind::Nested => "inner-trigger".into(),
            Kind::Select => (ElementId::from("sel"), "menu").into(),
            Kind::DatePicker => {
                let calendar: ElementId = (ElementId::from("dp"), "calendar").into();
                (calendar, "panel").into()
            }
        };
        let h = Self {
            window,
            page,
            surface,
        };
        h.settle(cx);
        h
    }

    fn settle(&self, cx: &mut TestAppContext) {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(self.window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }

    fn press(&self, key: &str, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| window.press(key, cx))
            .unwrap();
        self.settle(cx);
    }

    fn open(&self, cx: &mut TestAppContext) -> bool {
        let surface = self.surface.clone();
        cx.update_window(self.window, |_, window, _| {
            window.try_find(surface.clone()).is_some()
        })
        .unwrap()
    }

    fn on(&self, cx: &mut TestAppContext, which: fn(&Page) -> &FocusHandle) -> bool {
        let page = self.page.clone();
        cx.update_window(self.window, |_, window, cx| {
            which(page.read(cx)).is_focused(window)
        })
        .unwrap()
    }

    fn focus_after(&self, cx: &mut TestAppContext) {
        let page = self.page.clone();
        cx.update_window(self.window, |_, window, cx| {
            page.read(cx).after.clone().focus(window, cx)
        })
        .unwrap();
        self.settle(cx);
    }
}

#[gpui_kit::test]
fn popover_tab_runs_through_and_out(cx: &mut TestAppContext) {
    let h = Harness::new(Kind::Popover, cx);
    h.press("tab", cx);
    h.press("enter", cx);
    assert!(h.open(cx), "Enter on the trigger opens the popover");
    // Into the content, back to the trigger (still open), in again.
    h.press("tab", cx);
    assert!(h.open(cx));
    h.press("shift-tab", cx);
    assert!(h.open(cx), "Shift+Tab to the trigger keeps it open");
    assert!(!h.on(cx, |p| &p.before));
    h.press("tab", cx);
    assert!(h.open(cx), "Tab from the open trigger enters the content");
    // Out past the last stop: closed, on the control after the popover.
    h.press("tab", cx);
    assert!(!h.open(cx), "Tab out of the content closes the popover");
    assert!(h.on(cx, |p| &p.after), "focus moves on past the trigger");
}

#[gpui_kit::test]
fn popover_closes_when_focus_moves_away(cx: &mut TestAppContext) {
    let h = Harness::new(Kind::Popover, cx);
    h.press("tab", cx);
    h.press("enter", cx);
    h.press("tab", cx);
    assert!(h.open(cx));
    h.focus_after(cx);
    assert!(!h.open(cx), "focus moved away: the popover closes");
    assert!(h.on(cx, |p| &p.after), "and focus stays where it went");
}

#[gpui_kit::test]
fn select_closes_when_its_trigger_loses_focus(cx: &mut TestAppContext) {
    let h = Harness::new(Kind::Select, cx);
    h.press("tab", cx);
    h.press("enter", cx);
    assert!(h.open(cx), "Enter opens the menu");
    h.press("tab", cx);
    assert!(!h.open(cx), "Tab away closes the menu");
    assert!(h.on(cx, |p| &p.after));
}

#[gpui_kit::test]
fn date_picker_tabs_through_its_calendar(cx: &mut TestAppContext) {
    let h = Harness::new(Kind::DatePicker, cx);
    h.press("tab", cx);
    assert!(h.open(cx), "focusing the field opens the calendar");
    h.press("tab", cx);
    assert!(h.open(cx), "Tab moves into the calendar");
    h.press("shift-tab", cx);
    assert!(h.open(cx), "Shift+Tab returns to the field");
    h.press("tab", cx);
    h.press("tab", cx);
    assert!(!h.open(cx), "Tab out of the calendar closes it");
    assert!(h.on(cx, |p| &p.after), "and moves past the field");
    // Shift+Tab from the open field closes it on the way out.
    h.press("shift-tab", cx);
    assert!(h.open(cx), "back in the field: open again");
    h.press("shift-tab", cx);
    assert!(!h.open(cx), "Shift+Tab out of the field closes it");
    assert!(h.on(cx, |p| &p.before));
}

#[gpui_kit::test]
fn focus_in_a_nested_popover_is_still_inside(cx: &mut TestAppContext) {
    let h = Harness::new(Kind::Nested, cx);
    let inner: ElementId = "inner-btn".into();
    let inner_open = |cx: &mut TestAppContext| {
        let inner = inner.clone();
        cx.update_window(h.window, |_, window, _| window.try_find(inner).is_some())
            .unwrap()
    };
    h.press("tab", cx);
    h.press("enter", cx);
    assert!(h.open(cx));
    // Into the outer content (the inner trigger), open the inner popover
    // (focus moves into it) and Tab to its button.
    h.press("tab", cx);
    h.press("enter", cx);
    assert!(inner_open(cx), "the inner popover opens");
    h.press("tab", cx);
    assert!(inner_open(cx), "Tab inside the inner popover keeps it open");
    assert!(
        h.open(cx),
        "focus in the nested popover is inside the outer one"
    );
    // Escape closes only the inner one; focus returns to its trigger.
    h.press("escape", cx);
    assert!(!inner_open(cx));
    assert!(h.open(cx), "the outer popover stays open");
    // Focus leaving both closes both.
    h.focus_after(cx);
    assert!(!h.open(cx));
}

#[gpui_kit::test]
fn picking_a_date_inside_a_popover_keeps_the_popover(cx: &mut TestAppContext) {
    let h = Harness::new(Kind::DateInPopover, cx);
    let calendar: ElementId = {
        let id: ElementId = (ElementId::from("dp"), "calendar").into();
        (id, "panel").into()
    };
    let calendar_open = |cx: &mut TestAppContext| {
        let calendar = calendar.clone();
        cx.update_window(h.window, |_, window, _| window.try_find(calendar).is_some())
            .unwrap()
    };
    h.press("tab", cx);
    h.press("enter", cx);
    assert!(h.open(cx));
    // "Today", then the date field (its calendar opens), then the calendar.
    h.press("tab", cx);
    h.press("tab", cx);
    assert!(calendar_open(cx), "the field opens its calendar");
    h.press("tab", cx);
    assert!(
        calendar_open(cx) && h.open(cx),
        "focus in the calendar is inside"
    );
    // Enter picks the focused day: the calendar closes, the popover stays,
    // and focus returns to the field without reopening the calendar.
    h.press("enter", cx);
    assert!(!calendar_open(cx), "a pick closes the calendar");
    assert!(h.open(cx), "the popover holding the picker stays open");
}
