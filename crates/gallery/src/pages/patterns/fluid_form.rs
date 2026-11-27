//! Carbon · Patterns › PatternFluidForm: a fluid form (labels inside 64px
//! fields, no gaps, full-bleed buttons at the bottom) and the mixed use in a
//! dialog (regular fields, full-bleed buttons). Live: the fields edit, the
//! dropdowns choose, Pair server reports what it would pair, Cancel resets.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, prelude::FluentBuilder as _,
    rems,
};
use u0ui::components::{Input, Select};
use u0ui::patterns::{
    Caption, FluidDropdown, FluidTextInput, FrameButton, PatternFrame, PatternGrid,
};
use u0ui::prelude::*;

const EXAMPLES: [&str; 2] = ["fluid", "mixed"];
const CAPTIONS: [&str; 2] = [
    "Fluid 表单：标签在框内，64px 高，字段之间 0 间距，按钮贴底",
    "混合：普通字段 + 贴底满出血按钮（对话框、侧面板）",
];
const CONNECT: [&str; 2] = ["SSH", "Tunnel"];
const BASES: [&str; 2] = ["main", "develop"];
const LAYERS: [&str; 2] = ["01", "02"];
const BUTTONS: [&str; 2] = ["50% + 50%", "100%"];

/// The fields of one form.
struct Fields {
    name: Entity<InputState>,
    host: Entity<InputState>,
    code: Entity<InputState>,
    branch: Entity<InputState>,
}

impl Fields {
    fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("build-box")),
            host: cx.new(|cx| InputState::new(window, cx)),
            code: cx.new(|cx| InputState::new(window, cx)),
            branch: cx.new(|cx| InputState::new(window, cx).default_value("feat/side-chat")),
        }
    }
}

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let live = Fields::new(window, cx);
    let reference = Fields::new(window, cx);
    cx.new(|cx| {
        for field in [&live.name, &live.host, &live.code, &live.branch] {
            cx.observe(field, |_, _, cx| cx.notify()).detach();
        }
        Page {
            live,
            reference,
            example: 0,
            connect: Some(0),
            base: Some(0),
            layer: 0,
            buttons: 0,
            result: None,
        }
    })
    .into()
}

struct Page {
    live: Fields,
    reference: Fields,
    example: usize,
    connect: Option<usize>,
    base: Option<usize>,
    layer: usize,
    buttons: usize,
    result: Option<SharedString>,
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// The options an example is drawn with.
#[derive(Clone, Copy)]
struct Knobs {
    level: LayerLevel,
    connect: Option<usize>,
    base: Option<usize>,
    full_button: bool,
}

impl Knobs {
    const REFERENCE: Self = Self {
        level: LayerLevel::One,
        connect: Some(0),
        base: Some(0),
        full_button: false,
    };
}

/// The fluid form: Server name | Host, Connect through, Pairing code, then
/// Cancel | Pair server at `xl` sharing the width.
fn fluid(p: &str, f: &Fields, k: Knobs, live: Option<&Entity<Page>>) -> AnyElement {
    let mut connect = FluidDropdown::new(id(p, "connect"), "Connect through")
        .placeholder("Choose")
        .items(CONNECT)
        .value(k.connect)
        .layer(k.level);
    let mut cancel = Button::new(id(p, "cancel"), "Cancel")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Xl)
        .full_width(true);
    let mut pair = Button::new(id(p, "pair"), "Pair server")
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Xl)
        .full_width(true);
    if let Some(e) = live {
        let (e1, e2, e3) = (e.clone(), e.clone(), e.clone());
        connect = connect.on_change(move |i, _, cx| {
            e1.update(cx, |this, cx| {
                this.connect = Some(i);
                cx.notify();
            })
        });
        cancel = cancel.on_click(move |_, window, cx| {
            e2.update(cx, |this, cx| {
                this.live
                    .name
                    .update(cx, |s, cx| s.set_value("build-box", window, cx));
                this.live
                    .host
                    .update(cx, |s, cx| s.set_value("", window, cx));
                this.live
                    .code
                    .update(cx, |s, cx| s.set_value("", window, cx));
                this.connect = Some(0);
                this.result = None;
                cx.notify();
            })
        });
        pair = pair.on_click(move |_, _, cx| {
            e3.update(cx, |this, cx| {
                let name = this.live.name.read(cx).value().to_string();
                let host = this.live.host.read(cx).value().to_string();
                let via = this.connect.map_or("—", |i| CONNECT[i]);
                this.result = Some(if host.is_empty() {
                    "Add the host to pair".into()
                } else {
                    format!("Pairing {name} at {host} over {via}").into()
                });
                cx.notify();
            })
        });
    }
    let buttons = if k.full_button {
        div().flex().child(div().flex_1().child(pair))
    } else {
        div()
            .flex()
            .child(div().flex_1().child(cancel))
            .child(div().flex_1().child(pair))
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .child(
            // `display: grid; gridTemplateColumns: 1fr 1fr`, no gap.
            div()
                .flex()
                .child(div().flex_1().min_w_0().child(
                    FluidTextInput::new(id(p, "name"), &f.name, "Server name").layer(k.level),
                ))
                .child(
                    div().flex_1().min_w_0().child(
                        FluidTextInput::new(id(p, "host"), &f.host, "Host")
                            .placeholder("192.168.1.20")
                            .layer(k.level),
                    ),
                ),
        )
        .child(connect)
        .child(
            FluidTextInput::new(id(p, "code"), &f.code, "Pairing code")
                .password(true)
                .layer(k.level),
        )
        .child(buttons)
        .into_any_element()
}

/// The mixed form: a dialog frame with Branch and Base 24px apart and its
/// full-bleed Cancel / Create worktree.
fn mixed(p: &str, f: &Fields, k: Knobs, live: Option<&Entity<Page>>) -> AnyElement {
    let mut base = Select::new(id(p, "base"))
        .label("Base")
        .items(BASES)
        .layer(k.level)
        .value(k.base.map(|i| BASES[i]));
    let mut frame = PatternFrame::new(id(p, "frame"))
        .title("New worktree")
        .close_label("Close")
        .width(None);
    if !k.full_button {
        frame = frame.button(FrameButton::new("Cancel", ButtonVariant::Secondary));
    }
    frame = frame.button(FrameButton::new("Create worktree", ButtonVariant::Primary));
    if let Some(e) = live {
        let (e1, e2) = (e.clone(), e.clone());
        base = base.on_value_change(move |v, _, cx| {
            let i = v.and_then(|v| BASES.iter().position(|b| *b == v.as_ref()));
            e1.update(cx, |this, cx| {
                this.base = i;
                cx.notify();
            })
        });
        let full = k.full_button;
        frame = frame.on_button(move |i, _, cx| {
            e2.update(cx, |this, cx| {
                let create = full || i == 1;
                this.result = Some(if create {
                    let branch = this.live.branch.read(cx).value().to_string();
                    let base = this.base.map_or("—", |i| BASES[i]);
                    format!("Creating {branch} from {base}").into()
                } else {
                    "Cancelled".into()
                });
                cx.notify();
            })
        });
    }
    frame
        .child(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_06)
                .child(
                    Input::new(id(p, "branch"), &f.branch)
                        .label("Branch")
                        .layer(k.level),
                )
                .child(base),
        )
        .into_any_element()
}

fn captioned(i: usize, body: AnyElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[i]))
        .child(body)
        .into_any_element()
}

/// The overview tile: the name and host fields side by side.
pub fn thumbnail(window: &mut Window, cx: &mut App) -> AnyElement {
    let name = window.use_keyed_state("fluid-thumb-name", cx, |window, cx| {
        InputState::new(window, cx).default_value("build-box")
    });
    div()
        .w(rems(15.))
        .child(FluidTextInput::new("fluid-thumb", &name, "Server name"))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let knobs = Knobs {
            level: if self.layer == 0 {
                LayerLevel::One
            } else {
                LayerLevel::Two
            },
            connect: self.connect,
            base: self.base,
            full_button: self.buttons == 1,
        };
        let body = if self.example == 0 {
            fluid("fluid-live", &self.live, knobs, Some(&entity))
        } else {
            mixed("fluid-live", &self.live, knobs, Some(&entity))
        };
        let surface = div()
            .w_full()
            // On layer 02 the form sits on a `layer-01` surface.
            .when(knobs.level == LayerLevel::Two, |d| {
                d.bg(color(ColorRole::Layer01)).p(tokens::SPACING_05)
            })
            .child(body);
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(rems(28.))
            .child(captioned(self.example, surface.into_any_element()))
            .children(self.result.clone().map(|r| {
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(r)
            }));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "example",
                controls::choices(
                    "fluid-example",
                    &EXAMPLES,
                    self.example,
                    controls::setter(&entity, |this: &mut Page, i| this.example = i),
                ),
            ))
            .child(controls::field(
                "layer",
                controls::choices(
                    "fluid-layer",
                    &LAYERS,
                    self.layer,
                    controls::setter(&entity, |this: &mut Page, i| this.layer = i),
                ),
            ))
            .child(controls::field(
                "buttons",
                controls::choices(
                    "fluid-buttons",
                    &BUTTONS,
                    self.buttons,
                    controls::setter(&entity, |this: &mut Page, i| this.buttons = i),
                ),
            ));

        let grid = PatternGrid::new(2)
            .align_start(true)
            .cell(captioned(
                0,
                fluid("fluid-ref0", &self.reference, Knobs::REFERENCE, None),
            ))
            .cell(captioned(
                1,
                mixed("fluid-ref1", &self.reference, Knobs::REFERENCE, None),
            ));

        Body::new()
            .playground(live)
            .hint("Tab through the fields; Enter or Space opens Connect through")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
