//! Carbon · Patterns › PatternDialogs: button counts and widths, and the four
//! dialog types (passive, acknowledgment, transactional, progress), drawn as
//! static frames side by side. The live frame switches type, shows the
//! loading state and walks the progress steps.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, prelude::FluentBuilder as _,
    rems,
};
use u0ui::components::{Input, Spinner};
use u0ui::patterns::{
    Caption, FrameButton, PatternFrame, PatternGrid, ProgressIndicator, ProgressStep,
};
use u0ui::prelude::*;
use u0ui::typography::paragraph;

/// The dialog kinds the reference shows, in preview order.
const KINDS: [&str; 6] = [
    "passive",
    "acknowledgment",
    "transactional",
    "danger",
    "three",
    "progress",
];

/// The progress dialog's steps.
const STEPS: [&str; 3] = ["Folder", "Agents", "Review"];

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let name = cx.new(|cx| InputState::new(window, cx).default_value("carbon-ui"));
    let live_name = cx.new(|cx| InputState::new(window, cx).default_value("carbon-ui"));
    cx.new(|_| Page {
        name,
        live_name,
        kind: 2,
        step: 1,
        loading: false,
        label: true,
        closed: false,
    })
    .into()
}

struct Page {
    name: Entity<InputState>,
    live_name: Entity<InputState>,
    kind: usize,
    step: usize,
    loading: bool,
    label: bool,
    closed: bool,
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// A bare `<p>` in `.cu-pframe-body`: `.cu-pframe-body p` only resets the
/// margin, so Carbon's global `p` rule makes it 16px/24px.
fn para(text: &'static str) -> AnyElement {
    paragraph().child(text).into_any_element()
}

/// Live state a frame reacts to: the progress step, loading, the label.
#[derive(Clone, Copy)]
struct State {
    step: usize,
    loading: bool,
    label: bool,
}

impl State {
    const REFERENCE: Self = Self {
        step: 1,
        loading: false,
        label: true,
    };
}

/// The reference frame of `kind` (`Frame` calls in preview.html).
fn frame(kind: usize, p: &str, name: Option<&Entity<InputState>>, s: State) -> PatternFrame {
    let frame = PatternFrame::new(id(p, "frame")).close_label("Close");
    let label = |f: PatternFrame, text: &'static str| if s.label { f.label(text) } else { f };
    match kind {
        0 => label(frame, "Usage")
            .title("Your plan renews on 1 Dec")
            .child(para("Nothing to do now. We will email you a receipt.")),
        1 => frame
            .title("Pairing complete")
            .lead_gap(1.)
            .button(FrameButton::new("OK", ButtonVariant::Primary))
            .child(para("iPhone 16 can now see your sessions.")),
        2 => label(frame, "Project")
            .title("Rename project")
            .button(FrameButton::new("Cancel", ButtonVariant::Secondary))
            .button(FrameButton::new("Rename", ButtonVariant::Primary).disabled(s.loading))
            .when_some(name, |f, name| {
                f.child(
                    Input::new(id(p, "name"), name)
                        .label("Project name")
                        // `.cu-pframe` sets no `cds--layer`, so the field is
                        // `field-01`, the same gray as the frame.
                        .layer(LayerLevel::One),
                )
            }),
        3 => label(frame, "carbon-ui")
            .danger(true)
            .title("Delete 3 worktrees?")
            .button(FrameButton::new("Cancel", ButtonVariant::Secondary))
            .button(
                FrameButton::new("Delete 3 worktrees", ButtonVariant::Danger).disabled(s.loading),
            )
            .child(para("Uncommitted changes in these worktrees are lost.")),
        4 => frame
            .width(Some(420.))
            .title("Unsaved changes")
            .lead_gap(1.)
            .button(FrameButton::new("Discard", ButtonVariant::Ghost))
            .button(FrameButton::new("Cancel", ButtonVariant::Secondary))
            .button(FrameButton::new("Save", ButtonVariant::Primary).disabled(s.loading))
            .child(para("Save your changes to settings.json before closing?")),
        _ => {
            let last = s.step + 1 >= STEPS.len();
            let mut steps = ProgressIndicator::new(id(p, "steps"))
                .current_index(s.step)
                .space_equally(true);
            for step in STEPS {
                steps = steps.step(ProgressStep::new(step));
            }
            label(
                frame.width(Some(420.)),
                ["Step 1 of 3", "Step 2 of 3", "Step 3 of 3"][s.step.min(2)],
            )
            .title("Choose agents")
            .lead_gap(1.)
            .button(FrameButton::new("Cancel", ButtonVariant::Ghost))
            .button(FrameButton::new("Back", ButtonVariant::Secondary).disabled(s.step == 0))
            .button(
                // The last step names the action instead of "Next".
                FrameButton::new(
                    if last { "Start threads" } else { "Next" },
                    ButtonVariant::Primary,
                )
                .disabled(s.loading),
            )
            .child(steps)
        }
    }
}

const CAPTIONS: [&str; 6] = [
    "Passive：没有操作按钮，只用 × 关闭",
    "Acknowledgment：单按钮，右侧 50%，贴底满出血",
    "Transactional：次要在左、主要在右，各 50%",
    "Danger：主按钮用 danger，复述动作",
    "三个按钮：各 25%，靠右，只有最右是主按钮",
    "Progress：Cancel（ghost，左 25%）+ 右半边 Back / Next",
];

fn captioned(kind: usize, frame: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[kind]))
        .child(frame)
        .into_any_element()
}

/// The overview tile: the acknowledgment frame.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(frame(1, "dlg-thumb", None, State::REFERENCE).width(None))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let state = State {
            step: self.step,
            loading: self.loading,
            label: self.label,
        };
        let kind = self.kind;
        let buttons = entity.clone();
        let close = entity.clone();
        let live_frame = frame(kind, "dlg-live", Some(&self.live_name), state)
            .on_button(move |i, _, cx| {
                buttons.update(cx, |this, cx| {
                    if this.kind == 5 {
                        // Cancel, Back, Next / the action.
                        match i {
                            0 => this.step = 0,
                            1 => this.step = this.step.saturating_sub(1),
                            _ => this.step = (this.step + 1).min(STEPS.len() - 1),
                        }
                    } else {
                        this.closed = true;
                    }
                    cx.notify();
                })
            })
            .on_close(move |_, cx| {
                close.update(cx, |this, cx| {
                    this.closed = true;
                    cx.notify();
                })
            });
        // Loading: a spinner over the body, the primary button disabled.
        let live = if self.closed {
            let reopen = entity.clone();
            div()
                .child(
                    Button::new("dlg-reopen", "Show the dialog again")
                        .variant(ButtonVariant::Tertiary)
                        .on_click(move |_, _, cx| {
                            reopen.update(cx, |this, cx| {
                                this.closed = false;
                                cx.notify();
                            })
                        }),
                )
                .into_any_element()
        } else {
            div()
                .relative()
                .child(live_frame)
                .when(self.loading, |d| {
                    d.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Spinner::new("dlg-spinner").label("Loading")),
                    )
                })
                .into_any_element()
        };

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "type",
                controls::choices(
                    "dlg-kind",
                    &KINDS,
                    self.kind,
                    controls::setter(&entity, |this: &mut Page, i| {
                        this.kind = i;
                        this.closed = false;
                    }),
                ),
            ))
            .child(controls::field(
                "step",
                controls::choices(
                    "dlg-step",
                    &STEPS,
                    self.step,
                    controls::setter(&entity, |this: &mut Page, i| this.step = i),
                ),
            ))
            .child(controls::toggle(
                "dlg-loading",
                "loading",
                self.loading,
                controls::setter(&entity, |this: &mut Page, v| this.loading = v),
            ))
            .child(controls::toggle(
                "dlg-label",
                "label",
                self.label,
                controls::setter(&entity, |this: &mut Page, v| this.label = v),
            ));

        let mut grid = PatternGrid::new(2);
        for k in 0..KINDS.len() {
            grid = grid.cell(captioned(
                k,
                frame(
                    k,
                    &format!("dlg-ref{k}"),
                    Some(&self.name),
                    State::REFERENCE,
                ),
            ));
        }

        Body::new()
            .playground(captioned(kind, live))
            .hint("Press the buttons: progress walks the steps, the others close the frame")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
