//! Tab or Shift-Tab into any single-line field selects all of its text, as a
//! browser does for `<input>` (design review FG3D: the u0 fields kept the
//! caret, so typing after Shift-Tab appended instead of replacing). The rule
//! hangs off every `InputState`, so the u0 parts get it without opting in:
//! the SettingsDialog search, the DictionaryEditor new-word field and the
//! Stepper (SettingsRowOverridden, ScheduleEditor) are checked here. A
//! `<textarea>` keeps its caret, as in a browser.
use gpui_kit::Focusable as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyElement, AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions,
    component::input::{InputState, TextareaState},
    div, px, size,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use u0ui::components::Textarea;
use u0ui::icons::CarbonIcon;
use u0ui::u0::primitives::Stepper;
use u0ui::u0::settings::{DictionaryEditor, SettingsDialog, SettingsSection};

#[derive(Clone, Copy)]
enum Field {
    SettingsSearch,
    DictionaryWord,
    Stepper,
    Textarea,
}

struct Page {
    field: Field,
    before: FocusHandle,
    after: FocusHandle,
    textarea: Entity<TextareaState>,
}

fn stop(handle: &FocusHandle, id: &'static str) -> impl IntoElement {
    div()
        .id(id)
        .track_focus(&handle.clone().tab_index(0).tab_stop(true))
        .size(px(10.))
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let field: AnyElement = match self.field {
            Field::SettingsSearch => SettingsDialog::new("settings")
                .section(
                    SettingsSection::new("")
                        .item("general", "General", CarbonIcon::Settings)
                        .item("agents", "Agents", CarbonIcon::Bot),
                )
                .into_any_element(),
            Field::DictionaryWord => DictionaryEditor::new("dict")
                .label("Dictation dictionary")
                .words(["Carbon", "pnpm"])
                .into_any_element(),
            Field::Stepper => Stepper::new("stepper")
                .label("Parallel agents")
                .default_value(6.)
                .min(1.)
                .max(8.)
                .into_any_element(),
            Field::Textarea => Textarea::new("notes", &self.textarea)
                .label("Notes")
                .into_any_element(),
        };
        div()
            .w(px(900.))
            .flex()
            .flex_col()
            .child(stop(&self.before, "before"))
            .child(field)
            .child(stop(&self.after, "after"))
    }
}

struct Harness {
    window: AnyWindowHandle,
    before: FocusHandle,
    /// Every single-line field created in the app, in creation order.
    inputs: Rc<RefCell<Vec<Entity<InputState>>>>,
    textarea: Entity<TextareaState>,
}

fn open(field: Field, cx: &mut TestAppContext) -> Harness {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let inputs: Rc<RefCell<Vec<Entity<InputState>>>> = Rc::default();
    cx.update(|cx| {
        let inputs = inputs.clone();
        cx.observe_new(move |_: &mut InputState, _, cx| {
            inputs.borrow_mut().push(cx.entity());
        })
        .detach();
    });
    let (before, after) = cx.update(|cx| (cx.focus_handle(), cx.focus_handle()));
    let (window, textarea) = cx.update(|cx| {
        let (before, after) = (before.clone(), after.clone());
        let (window, page) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(800.)),
                })),
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let textarea = cx.new(|cx| TextareaState::new(window, cx));
                cx.new(|_| Page {
                    field,
                    before,
                    after,
                    textarea,
                })
            },
        )
        .expect("open test window");
        let textarea = page.read(cx).textarea.clone();
        (window, textarea)
    });
    // Focus events (Kit's `InputEvent::Focus`) fire only in an active window.
    cx.update_window(window, |_, window, _| window.activate_window())
        .unwrap();
    settle(window, cx);
    Harness {
        window,
        before,
        inputs,
        textarea,
    }
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(h: &Harness, key: &str, cx: &mut TestAppContext) {
    cx.update_window(h.window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(h.window, cx);
}

/// The single-line field that has focus, if any.
fn focused_input(h: &Harness, cx: &mut TestAppContext) -> Option<Entity<InputState>> {
    let inputs = h.inputs.borrow().clone();
    cx.update_window(h.window, |_, window, cx| {
        inputs
            .into_iter()
            .find(|s| s.read(cx).focus_handle(cx).is_focused(window))
    })
    .unwrap()
}

/// Tab from the stop before the component to its first single-line field.
fn tab_to_field(h: &Harness, cx: &mut TestAppContext) -> Entity<InputState> {
    cx.update_window(h.window, |_, window, cx| h.before.focus(window, cx))
        .unwrap();
    settle(h.window, cx);
    for _ in 0..12 {
        press(h, "tab", cx);
        if let Some(state) = focused_input(h, cx) {
            return state;
        }
    }
    panic!("Tab never reached a text field");
}

/// Give the field `text` (caret at its end), Tab out and Shift-Tab back:
/// the whole value is selected each time Tab brings focus in.
fn assert_tab_in_selects_all(field: Field, text: &str, cx: &mut TestAppContext) {
    let h = open(field, cx);
    let state = tab_to_field(&h, cx);
    cx.update_window(h.window, |_, window, cx| {
        state.update(cx, |s, cx| s.set_value(text.to_string(), window, cx));
    })
    .unwrap();
    settle(h.window, cx);
    let range = |cx: &mut TestAppContext| cx.update(|cx| state.read(cx).selected_range());
    assert!(range(cx).is_empty(), "set_value leaves a caret");
    press(&h, "tab", cx);
    assert!(focused_input(&h, cx).as_ref() != Some(&state), "Tab leaves");
    press(&h, "shift-tab", cx);
    assert!(
        focused_input(&h, cx).as_ref() == Some(&state),
        "Shift-Tab comes back"
    );
    assert_eq!(range(cx), 0..text.len(), "Shift-Tab in selects all");
}

#[gpui_kit::test]
fn settings_dialog_search(cx: &mut TestAppContext) {
    assert_tab_in_selects_all(Field::SettingsSearch, "vd", cx);
}

#[gpui_kit::test]
fn dictionary_editor_new_word(cx: &mut TestAppContext) {
    assert_tab_in_selects_all(Field::DictionaryWord, "ab", cx);
}

#[gpui_kit::test]
fn stepper_field(cx: &mut TestAppContext) {
    let h = open(Field::Stepper, cx);
    let state = tab_to_field(&h, cx);
    // Tab in from before the stepper selects its "6".
    assert_eq!(cx.update(|cx| state.read(cx).selected_range()), 0..1);
}

#[gpui_kit::test]
fn textarea_keeps_its_caret(cx: &mut TestAppContext) {
    let h = open(Field::Textarea, cx);
    cx.update_window(h.window, |_, window, cx| {
        h.textarea
            .update(cx, |s, cx| s.set_value("two\nlines", window, cx));
        h.before.focus(window, cx);
    })
    .unwrap();
    settle(h.window, cx);
    let focused = |h: &Harness, cx: &mut TestAppContext| {
        cx.update_window(h.window, |_, window, cx| {
            h.textarea.read(cx).focus_handle(cx).is_focused(window)
        })
        .unwrap()
    };
    for _ in 0..6 {
        press(&h, "tab", cx);
        if focused(&h, cx) {
            break;
        }
    }
    assert!(focused(&h, cx), "Tab reaches the textarea");
    assert!(
        cx.update(|cx| h.textarea.read(cx).selected_range())
            .is_empty()
    );
}
