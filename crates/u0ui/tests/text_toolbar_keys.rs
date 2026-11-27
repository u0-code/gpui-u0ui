//! `role="toolbar"` roving keys in TextToolbar: ←/→ visit every enabled
//! button, the inline dropdowns and the overflow button, wrapping, and
//! never land on a disabled button (Undo), which would show no focus
//! (design review FP2, item 11). A dropdown without its own focus handle
//! (as the gallery builds them) is a stop too: Enter/Space/Down open it,
//! Escape keeps focus on it, and the arrows go on from there (FP3, item 2).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use u0ui::components::{FieldSize, MenuItem, Select, SelectType};
use u0ui::icons::CarbonIcon;
use u0ui::patterns::{TextToolbar, ToolbarAction, ToolbarItem};

struct Page {
    sentinel: FocusHandle,
    font: FocusHandle,
    picked: Rc<RefCell<Vec<String>>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let picked = self.picked.clone();
        div()
            .w(px(800.))
            .flex()
            .flex_col()
            .child(
                div()
                    .id("sentinel")
                    .track_focus(&self.sentinel.clone().tab_index(0).tab_stop(true))
                    .size(px(4.)),
            )
            .child(
                TextToolbar::new("tb")
                    .overflow_label("More")
                    .item(ToolbarItem::action(
                        ToolbarAction::new("undo", "Undo", CarbonIcon::Undo).disabled(true),
                    ))
                    .item(ToolbarItem::action(ToolbarAction::new(
                        "redo",
                        "Redo",
                        CarbonIcon::Redo,
                    )))
                    .item(ToolbarItem::Separator)
                    .item(ToolbarItem::dropdown(
                        140.,
                        Select::new("font")
                            .items(["IBM Plex Sans", "IBM Plex Mono"])
                            .default_value("IBM Plex Sans")
                            .kind(SelectType::Inline)
                            .size(FieldSize::Md)
                            .track_focus(&self.font),
                    ))
                    // No `track_focus`: the toolbar supplies the handle.
                    .item(ToolbarItem::dropdown(
                        64.,
                        Select::new("size")
                            .items(["12", "14", "16", "20"])
                            .default_value("14")
                            .kind(SelectType::Inline)
                            .size(FieldSize::Md)
                            .on_value_change(move |v, _, _| {
                                picked
                                    .borrow_mut()
                                    .push(v.map(|v| v.to_string()).unwrap_or_default())
                            }),
                    ))
                    .item(ToolbarItem::Separator)
                    .item(ToolbarItem::action(ToolbarAction::new(
                        "bold",
                        "Bold (⌘B)",
                        CarbonIcon::TextBold,
                    )))
                    .overflow_items([MenuItem::item("Clear formatting")]),
            )
    }
}

type Picked = Rc<RefCell<Vec<String>>>;

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, FocusHandle, FocusHandle, Picked) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (sentinel, font) = cx.update(|cx| (cx.focus_handle(), cx.focus_handle()));
    let picked: Picked = Rc::default();
    let page = Page {
        sentinel: sentinel.clone(),
        font: font.clone(),
        picked: picked.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(900.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| page),
        )
        .expect("open test window")
        .0
    });
    (window, sentinel, font, picked)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) -> FocusHandle {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
    cx.update_window(window, |_, window, cx| window.focused(cx))
        .unwrap()
        .expect("focus stays in the window")
}

#[gpui_kit::test]
fn arrows_visit_buttons_dropdowns_and_more_skipping_disabled(cx: &mut TestAppContext) {
    let (window, sentinel, font, _) = open(cx);
    settle(window, cx);
    cx.update_window(window, |_, window, cx| sentinel.focus(window, cx))
        .unwrap();
    settle(window, cx);
    // Tab skips the disabled Undo: Redo.
    let redo = press("tab", window, cx);
    // → the font dropdown, → the size dropdown, → Bold, → More, → back
    // to Redo (not Undo).
    assert_eq!(press("right", window, cx), font);
    let size = press("right", window, cx);
    assert!(size != font && size != redo);
    let bold = press("right", window, cx);
    assert!(bold != font && bold != redo && bold != size);
    let more = press("right", window, cx);
    assert!(more != bold && more != font && more != redo);
    assert_eq!(press("right", window, cx), redo);
    // ← wraps from Redo to More, then walks back.
    assert_eq!(press("left", window, cx), more);
    assert_eq!(press("left", window, cx), bold);
    assert_eq!(press("left", window, cx), size);
    assert_eq!(press("left", window, cx), font);
    assert_eq!(press("left", window, cx), redo);
    // Home / End: the first and last stops.
    assert_eq!(press("end", window, cx), more);
    assert_eq!(press("home", window, cx), redo);
}

#[gpui_kit::test]
fn a_dropdown_without_its_own_handle_opens_and_keeps_roving(cx: &mut TestAppContext) {
    let (window, sentinel, font, picked) = open(cx);
    settle(window, cx);
    cx.update_window(window, |_, window, cx| sentinel.focus(window, cx))
        .unwrap();
    settle(window, cx);
    press("tab", window, cx);
    press("end", window, cx);
    let bold = press("left", window, cx);
    // ← from Bold: the size dropdown, a real stop (← goes on to the font).
    let size = press("left", window, cx);
    assert!(size != bold && size != font);
    // Enter opens it; ↓ moves to 16; Enter picks it and focus stays.
    press("enter", window, cx);
    press("down", window, cx);
    assert_eq!(press("enter", window, cx), size);
    assert_eq!(picked.borrow().as_slice(), ["16"]);
    // Space opens it, Escape closes it with focus on the trigger, and the
    // arrows rove on from there.
    press("space", window, cx);
    assert_eq!(press("escape", window, cx), size);
    assert!(picked.borrow().len() == 1);
    assert_eq!(press("right", window, cx), bold);
    assert_eq!(press("left", window, cx), size);
    // ↓ opens too: Enter then picks the highlighted (current) value.
    press("down", window, cx);
    assert_eq!(press("enter", window, cx), size);
    assert_eq!(picked.borrow().as_slice(), ["16", "16"]);
    assert_eq!(press("left", window, cx), font);
}
