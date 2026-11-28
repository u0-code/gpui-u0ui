//! A KeyRecorder records a shortcut the app has already bound: while it
//! records, ⌘K is its, not the keymap's (a recorder in a keybindings page
//! must take the very keys that conflict), whether it was pressed with the
//! pointer or the keyboard. Escape ends recording without a
//! shortcut, and once it is over the binding runs again.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Point, Render,
    SharedString, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, actions, div,
    px, size,
};
use gpui_kit::{ElementId, InputEvent as _, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent};
use std::time::Duration;
use u0ui::u0::primitives::KeyRecorder;

actions!(recorder_test, [Palette]);

struct Page {
    before: FocusHandle,
    keys: Entity<SharedString>,
    hits: usize,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let keys = self.keys.clone();
        div()
            .id("root")
            .key_context("Page")
            .on_action(cx.listener(|this, _: &Palette, _, cx| {
                this.hits += 1;
                cx.notify();
            }))
            .p(px(20.))
            .w(px(420.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                div()
                    .id("before")
                    .track_focus(&self.before.clone().tab_index(0).tab_stop(true))
                    .size(px(10.)),
            )
            .child(
                KeyRecorder::new("kr")
                    .label("Shortcut")
                    .value(self.keys.read(cx).clone())
                    .on_value_change(move |v, _, cx| keys.update(cx, |s, _| *s = v)),
            )
    }
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

/// A pointer press on the record button, at its centre.
fn click_record(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
        let id = ElementId::from((ElementId::from("kr"), "button"));
        let position = window.find(id).bounds().center();
        let down = MouseDownEvent {
            button: MouseButton::Left,
            position,
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        };
        let up = MouseUpEvent {
            button: MouseButton::Left,
            position,
            modifiers: Modifiers::default(),
            click_count: 1,
        };
        window.dispatch_event(down.to_platform_input(), cx);
        window.dispatch_event(up.to_platform_input(), cx);
    })
    .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn a_shortcut_the_app_has_bound_is_recorded_not_run(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
        cx.bind_keys([KeyBinding::new("cmd-k", Palette, None)]);
    });
    let (before, keys) = cx.update(|cx| (cx.focus_handle(), cx.new(|_| SharedString::default())));
    let page_before = before.clone();
    let page_keys = keys.clone();
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| {
                cx.new(|_| Page {
                    before: page_before,
                    keys: page_keys,
                    hits: 0,
                })
            },
        )
        .expect("open test window")
    });
    settle(window, cx);
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);
    let hits = |cx: &mut TestAppContext| page.read_with(cx, |p, _| p.hits);

    // Not recording, ⌘K is the app's.
    press("cmd-k", window, cx);
    assert_eq!(hits(cx), 1);

    // Press the record button with the pointer: ⌘K is recorded, and not run.
    click_record(window, cx);
    press("cmd-k", window, cx);
    assert_eq!(keys.read_with(cx, |k, _| k.clone()), "⌘+K");
    assert_eq!(hits(cx), 1);

    // Recording is over: ⌘K is the app's again.
    press("cmd-k", window, cx);
    assert_eq!(hits(cx), 2);

    // Escape ends recording with the shortcut as it was.
    press("space", window, cx);
    press("escape", window, cx);
    assert_eq!(keys.read_with(cx, |k, _| k.clone()), "⌘+K");
    press("cmd-k", window, cx);
    assert_eq!(hits(cx), 3);
}
