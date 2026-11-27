//! u0 · Thread › NoteLine, CheckpointMarker, ContinueRow, PlanChecklist
//! and MessageImage: each live with its configuration, plus the reference
//! previews.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::{Button, ButtonSize, ButtonVariant};
use u0ui::prelude::*;
use u0ui::u0::thread::{
    CheckpointMarker, ContinueRow, ContinueStatus, MessageImage, NoteKind, NoteLine, PlanChecklist,
    PlanItem, PlanStatus,
};

fn log(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

fn width(px: f32) -> gpui_kit::Div {
    div().w_full().max_w(rems(px / 16.))
}

// ── NoteLine ────────────────────────────────────────────────────────────

fn retry(id: &'static str) -> Button {
    Button::new(id, "Retry")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
}

/// `components/NoteLine/preview.html`.
fn note_reference() -> impl IntoElement {
    width(520.)
        .child(NoteLine::new(NoteKind::Stopped, "You stopped the agent."))
        .child(NoteLine::new(NoteKind::Note, "Switched to Plan mode."))
        .child(NoteLine::new(NoteKind::Limit, "Usage limit reached.").resumes_at("14:00"))
        .child(
            NoteLine::new(NoteKind::Error, "The agent process exited unexpectedly.")
                .action(retry("nl-retry")),
        )
}

/// The NoteLine overview tile.
pub fn note_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(NoteLine::new(NoteKind::Stopped, "You stopped the agent."))
        .child(NoteLine::new(NoteKind::Limit, "Usage limit reached.").resumes_at("14:00"))
        .into_any_element()
}

struct NotePage {
    kind: usize,
    action: bool,
}

/// Build the NoteLine page.
pub fn build_note_line(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| NotePage {
        kind: 2,
        action: false,
    })
    .into()
}

impl Render for NotePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let kinds: Vec<&'static str> = NoteKind::ALL.iter().map(|k| k.name()).collect();
        let kind = NoteKind::ALL[self.kind];
        let live = width(520.).child(
            NoteLine::new(kind, "Usage limit reached.")
                .when(kind == NoteKind::Limit, |n| n.resumes_at("14:00"))
                .when(self.action, |n| n.action(retry("nl-live-retry"))),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "kind",
                controls::choices(
                    "nl-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&e, |p: &mut NotePage, v| p.kind = v),
                ),
            ))
            .child(controls::toggle(
                "nl-action",
                "action",
                self.action,
                controls::setter(&e, |p: &mut NotePage, v| p.action = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", note_reference()))
    }
}

// ── CheckpointMarker ────────────────────────────────────────────────────

/// `components/CheckpointMarker/preview.html`: 560px.
fn checkpoint_reference() -> impl IntoElement {
    width(560.).child(
        CheckpointMarker::new("ck-ref", "Checkpoint")
            .time("10:41")
            .files(4),
    )
}

/// The CheckpointMarker overview tile.
pub fn checkpoint_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(CheckpointMarker::new("ck-thumb", "Checkpoint").time("10:41"))
        .into_any_element()
}

struct CheckpointPage {
    files: bool,
    restored: usize,
}

/// Build the CheckpointMarker page.
pub fn build_checkpoint_marker(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CheckpointPage {
        files: true,
        restored: 0,
    })
    .into()
}

impl Render for CheckpointPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let restore = e.clone();
        let live = width(560.)
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                CheckpointMarker::new("ck-live", "Checkpoint")
                    .time("10:41")
                    .when(self.files, |c| c.files(4))
                    .on_restore(move |_, cx| {
                        restore.update(cx, |p, cx| {
                            p.restored += 1;
                            cx.notify();
                        })
                    }),
            )
            .child(log(format!("onRestore × {}", self.restored)));
        let config = div().child(controls::toggle(
            "ck-files",
            "files",
            self.files,
            controls::setter(&e, |p: &mut CheckpointPage, v| p.files = v),
        ));
        Body::new()
            .playground(live)
            .hint("Hover the marker (or Tab to it) to reveal Restore")
            .config(config)
            .example(controls::block("Reference preview", checkpoint_reference()))
    }
}

// ── ContinueRow ─────────────────────────────────────────────────────────

/// `components/ContinueRow/preview.html`: three rows, 520px on `layer-01`.
fn continue_reference() -> impl IntoElement {
    width(520.)
        .bg(color(ColorRole::Layer01))
        .child(
            ContinueRow::new("cn-0", "Add dark theme tokens")
                .project("carbon-ui")
                .agent("Claude")
                .accent(1)
                .time("2m ago")
                .status(ContinueStatus::NeedsYou),
        )
        .child(
            ContinueRow::new("cn-1", "Write migration guide")
                .project("docs")
                .agent("Codex")
                .accent(2)
                .time("yesterday"),
        )
        .child(
            ContinueRow::new("cn-2", "Investigate memory spike")
                .project("api-gateway")
                .agent("Gemini")
                .accent(3)
                .time("3d ago"),
        )
}

/// The ContinueRow overview tile.
pub fn continue_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            ContinueRow::new("cn-thumb", "Add dark theme tokens")
                .project("carbon-ui")
                .agent("Claude")
                .accent(1)
                .time("2m"),
        )
        .into_any_element()
}

struct ContinuePage {
    status: usize,
    accent: usize,
    picked: usize,
}

const STATUSES: [&str; 4] = ["none", "needs-you", "Failed", "paused"];
const ACCENTS: [&str; 4] = ["none", "1", "4", "8"];

/// Build the ContinueRow page.
pub fn build_continue_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ContinuePage {
        status: 1,
        accent: 1,
        picked: 0,
    })
    .into()
}

impl Render for ContinuePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let pick = e.clone();
        let status = match self.status {
            1 => Some(ContinueStatus::NeedsYou),
            2 => Some(ContinueStatus::Failed),
            3 => Some(ContinueStatus::Other("paused".into())),
            _ => None,
        };
        let live = width(520.)
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                div().bg(color(ColorRole::Layer01)).child(
                    ContinueRow::new("cn-live", "Add dark theme tokens")
                        .project("carbon-ui")
                        .agent("Claude")
                        .when_some(ACCENTS[self.accent].parse().ok(), |r, a| r.accent(a))
                        .time("2m ago")
                        .when_some(status, |r, s| r.status(s))
                        .on_pick_up(move |_, cx| {
                            pick.update(cx, |p, cx| {
                                p.picked += 1;
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(log(format!("onPickUp × {}", self.picked)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "status",
                controls::choices(
                    "cn-status",
                    &STATUSES,
                    self.status,
                    controls::setter(&e, |p: &mut ContinuePage, v| p.status = v),
                ),
            ))
            .child(controls::field(
                "accent",
                controls::choices(
                    "cn-accent",
                    &ACCENTS,
                    self.accent,
                    controls::setter(&e, |p: &mut ContinuePage, v| p.accent = v),
                ),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", continue_reference()))
    }
}

// ── PlanChecklist ───────────────────────────────────────────────────────

fn plan_items() -> Vec<PlanItem> {
    vec![
        PlanItem::new("Read the current token files", PlanStatus::Done),
        PlanItem::new("Add G90 layer-accent tokens", PlanStatus::Done),
        PlanItem::new(
            "Update components that reach layer 3",
            PlanStatus::InProgress,
        ),
        PlanItem::new("Regenerate the cover page", PlanStatus::Pending),
        PlanItem::new("Run visual tests", PlanStatus::Pending),
    ]
}

/// `components/PlanChecklist/preview.html`: 480px.
fn plan_reference() -> impl IntoElement {
    width(480.).child(PlanChecklist::new("pl-ref", "Plan").items(plan_items()))
}

/// The PlanChecklist overview tile.
pub fn plan_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(PlanChecklist::new("pl-thumb", "Plan").items(plan_items().into_iter().take(3)))
        .into_any_element()
}

struct PlanPage {
    collapsed: bool,
    extra: bool,
}

/// Build the PlanChecklist page.
pub fn build_plan_checklist(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PlanPage {
        collapsed: false,
        extra: true,
    })
    .into()
}

impl Render for PlanPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let mut items = plan_items();
        if self.extra {
            items.push(PlanItem::new("Approve the migration", PlanStatus::NeedsYou));
            items.push(PlanItem::new("Update the changelog", PlanStatus::Skipped));
        }
        let live = width(480.).child(
            PlanChecklist::new(("pl-live", self.collapsed as usize), "Plan")
                .items(items)
                .collapsed(self.collapsed),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "pl-col",
                "collapsed",
                self.collapsed,
                controls::setter(&e, |p: &mut PlanPage, v| p.collapsed = v),
            ))
            .child(controls::toggle(
                "pl-extra",
                "needs-you + skipped",
                self.extra,
                controls::setter(&e, |p: &mut PlanPage, v| p.extra = v),
            ));
        Body::new()
            .playground(live)
            .hint("Click the head to fold the list")
            .config(config)
            .example(controls::block("Reference preview", plan_reference()))
    }
}

// ── MessageImage ────────────────────────────────────────────────────────

/// `components/MessageImage/preview.html`: two placeholders, 16px apart.
fn image_reference() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_05)
        .child(
            MessageImage::new("mi-0", "Screenshot of the settings page")
                .dimensions(240., 150.)
                .caption("settings.png · 1440×900"),
        )
        .child(MessageImage::new("mi-1", "Diagram").dimensions(120., 150.))
}

/// The MessageImage overview tile.
pub fn image_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    MessageImage::new("mi-thumb", "Screenshot")
        .dimensions(160., 100.)
        .caption("settings.png")
        .into_any_element()
}

struct ImagePage {
    caption: bool,
    opened: usize,
}

/// Build the MessageImage page.
pub fn build_message_image(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ImagePage {
        caption: true,
        opened: 0,
    })
    .into()
}

impl Render for ImagePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let open = e.clone();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                MessageImage::new("mi-live", "Screenshot of the settings page")
                    .dimensions(240., 150.)
                    .when(self.caption, |m| m.caption("settings.png · 1440×900"))
                    .on_open(move |_, cx| {
                        open.update(cx, |p, cx| {
                            p.opened += 1;
                            cx.notify();
                        })
                    }),
            )
            .child(log(format!("onOpen × {}", self.opened)));
        let config = div().child(controls::toggle(
            "mi-cap",
            "caption",
            self.caption,
            controls::setter(&e, |p: &mut ImagePage, v| p.caption = v),
        ));
        Body::new()
            .playground(live)
            .hint("Hover or Tab to the image to show its zoom badge")
            .config(config)
            .example(controls::block("Reference preview", image_reference()))
    }
}
