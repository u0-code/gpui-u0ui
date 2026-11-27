//! Carbon · Patterns › PatternReadOnlyStates: fields the user can see,
//! focus and copy but not change (transparent fill, a subtle rule, text at
//! full contrast), next to the editable and disabled states
//! (`components/PatternReadOnlyStates/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, component::input::InputState, div,
};
use u0ui::components::slider::{Slider, SliderState};
use u0ui::components::{
    Checkbox, FieldSize, Input, RadioGroup, RadioGroupItem, Select, Switch, SwitchSize,
};
use u0ui::ghost::primitives::Stepper;
use u0ui::patterns::{Caption, PatternGrid};
use u0ui::prelude::*;

/// The page's field states, one set per preview.
struct Fields {
    server: Entity<InputState>,
    editable: Entity<InputState>,
    disabled: Entity<InputState>,
    level: Entity<SliderState>,
}

impl Fields {
    fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            server: crate::stand_in::input_state("build-box.local", window, cx),
            editable: crate::stand_in::input_state("build-box.local", window, cx),
            disabled: crate::stand_in::input_state("build-box.local", window, cx),
            level: cx.new(|_| {
                SliderState::new()
                    .min(0.)
                    .max(3.)
                    .step(1.)
                    .default_value(1.)
            }),
        }
    }
}

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let live = Fields::new(window, cx);
    let reference = Fields::new(window, cx);
    cx.new(|_| ReadOnlyPage {
        live,
        reference,
        readonly: true,
        empty: false,
    })
    .into()
}

struct ReadOnlyPage {
    live: Fields,
    reference: Fields,
    /// The left column read-only (off: editable, to compare).
    readonly: bool,
    /// The dropdown has no value (it then shows its helper text).
    empty: bool,
}

/// The read-only column: text input, dropdown, number input, checkbox and
/// toggle, radio group (`display: grid; gap: 16`).
fn read_only_column(prefix: &'static str, f: &Fields, readonly: bool, empty: bool) -> AnyElement {
    let id = |s: &str| gpui_kit::SharedString::from(format!("{prefix}-{s}"));
    let mut agent = Select::new(id("agent"))
        .label("Default agent")
        .items(["Claude", "Codex"])
        .size(FieldSize::Md)
        .readonly(readonly);
    agent = if empty {
        agent
            .placeholder("No agent")
            .helper_text("Set by your workspace admin")
    } else {
        agent.default_value("Claude")
    };
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            Input::new(id("server"), &f.server)
                .label("Server")
                .size(FieldSize::Md)
                .readonly(readonly),
        )
        .child(agent)
        .child(
            Stepper::new(id("parallel"))
                .label("Max parallel agents")
                .default_value(3.)
                .min(1.)
                .max(8.)
                .readonly(readonly),
        )
        .child(
            // `display: flex; gap: 24`.
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(tokens::SPACING_06)
                .child(
                    Checkbox::new(id("accept"))
                        .label("Auto-accept edits")
                        .default_checked(true)
                        .readonly(readonly),
                )
                .child(
                    Switch::new(id("telemetry"))
                        .label("Telemetry")
                        .default_checked(false)
                        .size(SwitchSize::Sm)
                        .readonly(readonly),
                ),
        )
        .child(
            RadioGroup::new(id("connect"))
                .legend("Connect through")
                .default_value("ssh")
                .readonly(readonly)
                .item(RadioGroupItem::new("ssh").label("SSH"))
                .item(RadioGroupItem::new("tun").label("Tunnel")),
        )
        .into_any_element()
}

/// The comparison column: editable, disabled, and a read-only slider.
fn compare_column(prefix: &'static str, f: &Fields) -> AnyElement {
    let id = |s: &str| gpui_kit::SharedString::from(format!("{prefix}-{s}"));
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            Input::new(id("editable"), &f.editable)
                .label("Server (editable)")
                .size(FieldSize::Md),
        )
        .child(
            Input::new(id("disabled"), &f.disabled)
                .label("Server (disabled — waiting for pairing)")
                .size(FieldSize::Md)
                .disabled(true),
        )
        .child(
            Slider::new(id("level"), &f.level)
                .label("Telemetry level (read-only)")
                .hide_input(true)
                .readonly(true),
        )
        .into_any_element()
}

fn layout(left: AnyElement, right: AnyElement) -> impl IntoElement {
    PatternGrid::new(2)
        .align_start(true)
        .cell(
            div()
                .child(Caption::new(
                    "Read-only：底色透明、边框弱化、文字保持 4.5:1，可聚焦、可复制",
                ))
                .child(left),
        )
        .cell(div().child(Caption::new("对比：可编辑与禁用")).child(right))
}

/// The overview tile: one read-only field.
pub fn thumbnail(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = crate::stand_in::input_state("build-box.local", window, cx);
    div()
        .w(gpui_kit::rems(15.))
        .child(
            Input::new("ro-thumb", &state)
                .label("Server")
                .size(FieldSize::Md)
                .readonly(true),
        )
        .into_any_element()
}

impl Render for ReadOnlyPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = layout(
            read_only_column("ro-live", &self.live, self.readonly, self.empty),
            compare_column("ro-live", &self.live),
        );
        // `components/PatternReadOnlyStates/preview.html`.
        let reference = layout(
            read_only_column("ro-ref", &self.reference, true, false),
            compare_column("ro-ref", &self.reference),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "ro-readonly",
                "read-only (off: editable)",
                self.readonly,
                controls::setter(&entity, |this: &mut Self, on| this.readonly = on),
            ))
            .child(controls::toggle(
                "ro-empty",
                "dropdown without a value",
                self.empty,
                controls::setter(&entity, |this: &mut Self, on| this.empty = on),
            ));

        Body::new()
            .playground(live)
            .hint(
                "read-only fields still take focus (Tab) and their text can be selected and copied",
            )
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
