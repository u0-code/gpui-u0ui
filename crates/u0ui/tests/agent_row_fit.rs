//! An AgentRow's switch never runs out of the row (design review FG3C: in a
//! SettingsDialog about 572px wide the switch overflowed its row by ~24px
//! because the text column kept its least width). Once the text column is
//! down to that width, the buttons and the switch wrap to a second line at
//! the row's end.
use gpui_kit::test::{TestQueryExt as _, TestWindowExt as _};
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::u0::settings::{AgentRow, AgentState};

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

struct Page {
    width: f32,
    rows: Slot,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let slot = self.rows.clone();
        div().w(px(self.width)).overflow_hidden().child(
            div()
                .relative()
                .flex()
                .flex_col()
                .child(
                    AgentRow::new("claude", "Claude")
                        .version("2.1.0")
                        .signed_in(true)
                        .accounts(["work", "personal"]),
                )
                .child(
                    AgentRow::new("codex", "Codex")
                        .version("0.9.2")
                        .latest("0.10.0")
                        .signed_in(false),
                )
                .child(
                    AgentRow::new("gemini", "Gemini")
                        .state(AgentState::Installing)
                        .progress(64.),
                )
                .child(
                    canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                ),
        )
    }
}

/// The rows' bounds and each switch's.
fn measure(cx: &mut TestAppContext, width: f32) -> (Bounds<Pixels>, Vec<Bounds<Pixels>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let rows: Slot = Rc::default();
    let window = cx.update(|cx| {
        let rows = rows.clone();
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(700.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { width, rows }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    let switches = cx
        .update_window(window, |_, window, _| {
            ["Claude", "Codex", "Gemini"]
                .map(|n| window.find_by_label(&format!("{n} enabled")).bounds())
                .to_vec()
        })
        .unwrap();
    (rows.get().expect("rows painted"), switches)
}

fn assert_inside(width: f32, cx: &mut TestAppContext) {
    let (rows, switches) = measure(cx, width);
    for s in switches {
        assert!(
            s.right() <= rows.right() + px(0.5) && s.left() >= rows.left(),
            "at {width}px a switch {s:?} runs out of the rows {rows:?}"
        );
    }
}

#[gpui_kit::test]
fn switch_stays_inside_at_572(cx: &mut TestAppContext) {
    assert_inside(572., cx);
}

#[gpui_kit::test]
fn switch_stays_inside_at_300(cx: &mut TestAppContext) {
    assert_inside(300., cx);
}

#[gpui_kit::test]
fn switch_stays_inside_at_160(cx: &mut TestAppContext) {
    assert_inside(160., cx);
}

/// The progress status's bounds and its longest word's width.
fn status(cx: &mut TestAppContext, width: f32) -> (Bounds<Pixels>, Pixels) {
    let _ = measure(cx, width);
    let window = cx.windows()[0];
    cx.update_window(window, |_, window, _| {
        let status = window
            .find((gpui_kit::ElementId::from("gemini"), "status"))
            .bounds();
        let word = u0ui::u0::settings::status_min_width("Installing 64%", window);
        (status, word)
    })
    .unwrap()
}

/// The status wraps between words only ("Installing" / "64%"), never inside
/// "Installing" (design review FG3D); the bar is what gives way.
fn assert_no_mid_word_break(width: f32, cx: &mut TestAppContext) {
    let (status, word) = status(cx, width);
    assert!(
        status.size.width + px(0.5) >= word,
        "at {width}px the status is {:?} wide, under its longest word {word:?}",
        status.size.width
    );
}

#[gpui_kit::test]
fn status_keeps_its_words_at_510(cx: &mut TestAppContext) {
    assert_no_mid_word_break(510., cx);
}

#[gpui_kit::test]
fn status_keeps_its_words_at_400(cx: &mut TestAppContext) {
    assert_no_mid_word_break(400., cx);
}

#[gpui_kit::test]
fn status_keeps_its_words_at_160(cx: &mut TestAppContext) {
    assert_no_mid_word_break(160., cx);
}
