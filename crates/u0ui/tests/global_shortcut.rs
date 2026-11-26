//! A window-level shortcut (the gallery's ⌘K palette, bound without a key
//! context) still reaches the root while any u0ui text field has focus: Input,
//! Textarea, Combobox and InputOtp must not swallow it (design review A3, S2).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, Entity, FocusHandle, Focusable as _, InteractiveElement as _,
    IntoElement, KeyBinding, ParentElement as _, Point, Render, Styled as _, TestAppContext,
    Window, WindowBounds, WindowOptions, actions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{Combobox, Input, InputOtp, InputState, Textarea, TextareaState};

actions!(shortcut_test, [OpenSearch]);

struct Page {
    input: Entity<InputState>,
    area: Entity<TextareaState>,
    combo: Entity<InputState>,
    otp: Entity<InputState>,
    hits: usize,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("root")
            .key_context("Gallery")
            .on_action(cx.listener(|this, _: &OpenSearch, _, cx| {
                this.hits += 1;
                cx.notify();
            }))
            .size_full()
            .flex()
            .flex_col()
            .child(Input::new("input", &self.input))
            .child(Textarea::new("textarea", &self.area))
            .child(Combobox::new("combobox", &self.combo))
            .child(InputOtp::new("otp", &self.otp))
    }
}

#[gpui_kit::test]
fn cmd_k_reaches_the_root_from_every_text_field(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
        cx.bind_keys([
            KeyBinding::new("cmd-k", OpenSearch, None),
            KeyBinding::new("ctrl-k", OpenSearch, None),
        ]);
    });
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let input = cx.new(|cx| InputState::new(window, cx));
                let area = cx.new(|cx| TextareaState::new(window, cx));
                let combo = cx.new(|cx| InputState::new(window, cx));
                let otp = cx.new(|cx| InputState::new(window, cx));
                cx.new(|_| Page {
                    input,
                    area,
                    combo,
                    otp,
                    hits: 0,
                })
            },
        )
        .expect("open test window")
    });
    let settle = |cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    settle(cx);
    type Pick = fn(&Page, &gpui_kit::App) -> FocusHandle;
    let fields: [(&str, Pick); 4] = [
        ("Input", |p, cx| p.input.read(cx).focus_handle(cx)),
        ("Textarea", |p, cx| p.area.read(cx).focus_handle(cx)),
        ("Combobox", |p, cx| p.combo.read(cx).focus_handle(cx)),
        ("InputOtp", |p, cx| p.otp.read(cx).focus_handle(cx)),
    ];
    for (name, pick) in fields {
        for key in ["cmd-k", "ctrl-k"] {
            cx.update_window(window, |_, window, cx| {
                let focus = pick(page.read(cx), cx);
                focus.focus(window, cx);
            })
            .unwrap();
            settle(cx);
            let before = cx.update(|cx| page.read(cx).hits);
            cx.update_window(window, |_, window, cx| window.press(key, cx))
                .unwrap();
            settle(cx);
            let after = cx.update(|cx| page.read(cx).hits);
            assert_eq!(
                after,
                before + 1,
                "{key} from {name} did not reach the root"
            );
        }
    }
}
