//! A label-less switch keeps the 56px `.cds--toggle` box (the 48px track
//! and the 8px column gap), so a `SettingsRow` toggle ends 28px from the
//! row's right edge; `hide_label` (`.cu-switch--bare`) is the track alone
//! (design review FT4B, item 2).
use gpui_kit::test::{TestQueryExt as _, TestWindowExt as _};
use gpui_kit::{
    AnyElement, AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point,
    Render, Role, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div,
    px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::components::{Switch, SwitchSize};
use u0ui::t3::workspace::SettingsRow;

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

/// `child` with its bounds recorded into `slot`.
fn measured(slot: &Slot, child: impl IntoElement) -> gpui_kit::Div {
    let slot = slot.clone();
    div().relative().child(child).child(
        canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
    )
}

struct Page(Box<dyn Fn() -> AnyElement>);

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        (self.0)()
    }
}

fn render(
    cx: &mut TestAppContext,
    build: impl Fn() -> AnyElement + 'static,
) -> gpui_kit::AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(800.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page(Box::new(build))),
        )
        .expect("open test window")
        .0
    });
    for _ in 0..2 {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }
    window
}

fn near(a: Pixels, b: f32) -> bool {
    (f32::from(a) - b).abs() < 0.5
}

#[gpui_kit::test]
fn a_settings_row_toggle_sits_28px_in(cx: &mut TestAppContext) {
    let row: Slot = Rc::default();
    let slot = row.clone();
    let window = render(cx, move || {
        div()
            .w(px(800.))
            .child(measured(
                &slot,
                SettingsRow::new("row", "Auto-update")
                    .control(Switch::new("toggle").default_checked(true)),
            ))
            .into_any_element()
    });
    let switch = cx
        .update_window(window, |_, window, _| window.find_all_by_role(Role::Switch))
        .unwrap();
    let switch = switch.first().expect("a switch").bounds();
    let row = row.get().expect("measured");
    // `.cds--toggle` is 56px (computed): the 48px track and the 8px gap.
    assert!(near(switch.size.width, 56.), "{switch:?}");
    // `.t3-srow` ends in 20px of padding; the track ends 28px in.
    assert!(
        near(row.right() - switch.right(), 20.),
        "{row:?} {switch:?}"
    );
    assert!(
        near(row.right() - (switch.origin.x + px(48.)), 28.),
        "{row:?} {switch:?}"
    );
}

#[gpui_kit::test]
fn a_bare_switch_is_its_track(cx: &mut TestAppContext) {
    // `hide_label` (`.cu-switch--bare`, the `g-switch` stand-in) drops the gap.
    let window = render(cx, || {
        div()
            .flex()
            .child(
                Switch::new("bare")
                    .size(SwitchSize::Sm)
                    .label("Enabled")
                    .hide_label(true),
            )
            .into_any_element()
    });
    let switch = cx
        .update_window(window, |_, window, _| window.find_all_by_role(Role::Switch))
        .unwrap();
    assert!(near(switch[0].bounds().size.width, 32.), "{:?}", switch[0]);
}
