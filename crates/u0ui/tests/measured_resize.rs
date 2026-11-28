//! Layouts that depend on a width measured while the frame is laid out
//! follow a live window resize (design review FINAL2): after a resize from
//! 1280 to 800 at launch, ThemeEditor kept its 3-column swatch grid and an
//! open-at-load ContextMenu kept its old position until the next input.
//!
//! A measurement only changes during prepaint, and a notify from there is
//! dropped when the frame ends, so nothing drew the frame that uses it. The
//! tests draw only the frames the window asks for: the resize's, then the
//! next-frame callbacks it scheduled (as the platform's frame source does),
//! never a forced `render_frame`.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, IntoElement, ParentElement as _,
    Pixels, Point, Render, SharedString, Styled as _, TestAppContext, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{ContextMenu, MenuItem};
use u0ui::u0::settings::{ThemeEditor, ThemeToken, grid_columns};

const TOKENS: usize = 8;

struct Editor;

impl Render for Editor {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(ThemeEditor::new(
            "te",
            (0..TOKENS).map(|i| ThemeToken::hex(format!("token-{i}"), "#4589ff").unwrap()),
        ))
    }
}

struct Menu;

impl Render for Menu {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // The area is centered, so a narrower window moves it.
        div().size_full().flex().justify_center().child(
            div().w(px(300.)).child(
                ContextMenu::new("cm", div().w(px(300.)).h(px(200.)))
                    .item(MenuItem::item("Copy"))
                    .item(MenuItem::item("Paste"))
                    .default_open_at(40., 30.),
            ),
        )
    }
}

fn open<V: Render + 'static>(view: V, cx: &mut TestAppContext) -> AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1280.), px(700.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| view),
        )
        .expect("open test window")
        .0
    });
    frames(window, cx);
    window
}

/// Lets the window draw the frames it asked for, as the platform would.
fn frames(window: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..4 {
        cx.executor().advance_clock(Duration::from_millis(50));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| {
            window.simulate_next_frame(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn resize(window: AnyWindowHandle, width: f32, cx: &mut TestAppContext) {
    cx.simulate_window_resize(window, size(px(width), px(700.)));
    frames(window, cx);
}

fn text(i: usize) -> ElementId {
    let tile: ElementId = (ElementId::from("te"), SharedString::from(format!("sw-{i}"))).into();
    (tile, "text").into()
}

/// The swatches on the grid's first row.
fn first_row(window: AnyWindowHandle, cx: &mut TestAppContext) -> usize {
    cx.update_window(window, |_, window, _| {
        let top = window.find(text(0)).bounds().top();
        (0..TOKENS)
            .filter(|&i| window.find(text(i)).bounds().top() == top)
            .count()
    })
    .unwrap()
}

#[gpui_kit::test]
fn theme_editor_grid_follows_a_resize(cx: &mut TestAppContext) {
    let window = open(Editor, cx);
    assert_eq!(first_row(window, cx), grid_columns(1280.).min(TOKENS));
    resize(window, 800., cx);
    assert_eq!(
        first_row(window, cx),
        grid_columns(800.),
        "the grid has the columns of its new width"
    );
    resize(window, 1280., cx);
    assert_eq!(first_row(window, cx), grid_columns(1280.).min(TOKENS));
}

/// The open menu, relative to the area's top-left.
fn menu_offset(window: AnyWindowHandle, cx: &mut TestAppContext) -> Point<Pixels> {
    cx.update_window(window, |_, window, _| {
        let menu = window.find((ElementId::from("cm"), "menu")).bounds();
        let area = window.find((ElementId::from("cm"), "area")).bounds();
        menu.origin - area.origin
    })
    .unwrap()
}

#[gpui_kit::test]
fn default_open_context_menu_follows_its_area(cx: &mut TestAppContext) {
    let window = open(Menu, cx);
    let before = menu_offset(window, cx);
    // `default_open_at(40, 30)`, at most Positioner's 8px margin off.
    assert!(
        before.x >= px(32.) && before.y >= px(22.),
        "opened at {before:?}"
    );
    resize(window, 800., cx);
    assert_eq!(
        menu_offset(window, cx),
        before,
        "the menu stays at its point in the area"
    );
}
