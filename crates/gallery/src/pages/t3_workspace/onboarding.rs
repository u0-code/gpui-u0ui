//! T3 · Onboarding & feedback: ComputerSetupSteps, ExpandableToast,
//! NoProjectsHero, PermissionChecklist and TruncatedText (MiddleTruncate,
//! StartTruncatedPath, RelativeTime).
use super::{note, stack, text};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::time::{Duration, SystemTime};
use u0ui::prelude::*;
use u0ui::t3::review::StartTruncatedPath;
use u0ui::t3::workspace::{
    ComputerSetupSteps, ExpandableToast, MiddleTruncate, NoProjectsHero, Permission,
    PermissionChecklist, RelativeTime, SetupState, ToastKind,
};

// ── ComputerSetupSteps ──────────────────────────────────────────────────

struct SetupPage {
    state: usize,
    log: Option<SharedString>,
}

/// Build the ComputerSetupSteps page.
pub fn build_computer_setup_steps(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SetupPage {
        state: 0,
        log: None,
    })
    .into()
}

/// The overview tile.
pub fn computer_setup_steps_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .h(px(160.))
        .overflow_hidden()
        .child(ComputerSetupSteps::new("setup-thumb"))
        .into_any_element()
}

impl Render for SetupPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let say = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.log = Some(what.into());
                    cx.notify();
                })
            }
        };
        let live = stack(8.)
            .child(
                ComputerSetupSteps::new("setup-live")
                    .state(SetupState::ALL[self.state])
                    .on_pair(say("Add pairing link"))
                    .on_download(say("Download the desktop app")),
            )
            .when_some(self.log.clone(), |this, l| this.child(note(l)));
        let states = SetupState::ALL.map(|s| s.name());
        let config = controls::field(
            "state",
            controls::choices(
                "setup-state",
                &states,
                self.state,
                controls::setter(&entity, |this: &mut SetupPage, i| this.state = i),
            ),
        );
        Body::new()
            .playground(live)
            .hint("The copy button copies the command")
            .config(config)
            .example(controls::block(
                "Reference preview",
                ComputerSetupSteps::new("setup-ref").state(SetupState::Waiting),
            ))
    }
}

// ── ExpandableToast ─────────────────────────────────────────────────────

const DETAILS: &str = "! [rejected] feat/dark-tokens -> feat/dark-tokens (fetch first)\nerror: failed to push some refs";

struct ToastPage {
    kind: usize,
    description: bool,
    details: bool,
    closed: bool,
}

/// Build the ExpandableToast page.
pub fn build_expandable_toast(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ToastPage {
        kind: 0,
        description: true,
        details: true,
        closed: false,
    })
    .into()
}

/// The overview tile.
pub fn expandable_toast_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            ExpandableToast::new("xtoast-thumb", ToastKind::Error, "Could not push to origin")
                .details(DETAILS),
        )
        .into_any_element()
}

impl Render for ToastPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = ToastKind::ALL[self.kind];
        let live = if self.closed {
            stack(12.)
                .child(note("Closed."))
                .child(
                    Button::new("xtoast-reopen", "Show again")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Tertiary)
                        .balanced(true)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.closed = false;
                                    cx.notify();
                                })
                            }
                        }),
                )
                .into_any_element()
        } else {
            let title = match kind {
                ToastKind::Error => "Could not push to origin",
                ToastKind::Success => "Thread archived",
                ToastKind::Info => "Update available",
            };
            let mut toast = ExpandableToast::new("xtoast-live", kind, title).on_close({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.closed = true;
                        cx.notify();
                    })
                }
            });
            if self.description {
                toast = toast.description(match kind {
                    ToastKind::Error => "The remote rejected the update.",
                    ToastKind::Success => "Undo from the sidebar within 10 seconds.",
                    ToastKind::Info => "Restart to install version 2.1.",
                });
            }
            if self.details {
                toast = toast.details(DETAILS);
            }
            toast.into_any_element()
        };
        let kinds = ToastKind::ALL.map(|k| k.name());
        let config = stack(24.)
            .child(controls::field(
                "kind",
                controls::choices(
                    "xtoast-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut ToastPage, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "xtoast-description",
                "description",
                self.description,
                controls::setter(&entity, |this: &mut ToastPage, v| this.description = v),
            ))
            .child(controls::toggle(
                "xtoast-details",
                "details",
                self.details,
                controls::setter(&entity, |this: &mut ToastPage, v| this.details = v),
            ));
        // `components/ExpandableToast/preview.html` (`cu-stack`, gap 12).
        let reference = stack(12.)
            .child(
                ExpandableToast::new("xtoast-ref-0", ToastKind::Error, "Could not push to origin")
                    .description("The remote rejected the update.")
                    .details(DETAILS),
            )
            .child(
                ExpandableToast::new("xtoast-ref-1", ToastKind::Success, "Thread archived")
                    .description("Undo from the sidebar within 10 seconds."),
            );
        Body::new()
            .playground(live)
            .hint("Show details expands the raw error; Copy error copies it")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── NoProjectsHero ──────────────────────────────────────────────────────

struct HeroPage {
    log: Option<SharedString>,
}

/// Build the NoProjectsHero page.
pub fn build_no_projects_hero(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| HeroPage { log: None }).into()
}

/// The overview tile.
pub fn no_projects_hero_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .type_style(TypeStyle::Heading03)
        .text_color(color(ColorRole::TextPrimary))
        .child("What should we work on?")
        .into_any_element()
}

impl Render for HeroPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let say = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.log = Some(what.into());
                    cx.notify();
                })
            }
        };
        let live = stack(8.)
            .w_full()
            .child(
                NoProjectsHero::new("hero-live")
                    .on_add(say("Add project"))
                    .on_skip(say("Start without a project")),
            )
            .when_some(self.log.clone(), |this, l| this.child(note(l)));
        Body::new().playground(live).example(controls::block(
            "Reference preview",
            div().w_full().child(NoProjectsHero::new("hero-ref")),
        ))
    }
}

// ── PermissionChecklist ─────────────────────────────────────────────────

struct PermsPage {
    granted: [bool; 3],
}

/// Build the PermissionChecklist page.
pub fn build_permission_checklist(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PermsPage {
        granted: [true, true, false],
    })
    .into()
}

fn permissions(granted: [bool; 3]) -> [Permission; 3] {
    [
        Permission::new(
            "Accessibility",
            "Needed to type into other apps.",
            granted[0],
        ),
        Permission::new("Microphone", "For voice input.", granted[1]),
        Permission::new(
            "Screen recording",
            "For device and browser capture.",
            granted[2],
        ),
    ]
}

/// The overview tile.
pub fn permission_checklist_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(PermissionChecklist::new(
            "perms-thumb",
            permissions([true, false, false]).into_iter().take(2),
        ))
        .into_any_element()
}

impl Render for PermsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = PermissionChecklist::new("perms-live", permissions(self.granted))
            .on_open_settings({
                let entity = entity.clone();
                move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.granted[i] = true;
                        cx.notify();
                    })
                }
            });
        let names = ["Accessibility", "Microphone", "Screen recording"];
        let mut config = stack(24.);
        for (i, n) in names.into_iter().enumerate() {
            config = config.child(controls::toggle(
                SharedString::from(format!("perms-{i}")),
                format!("{n} granted"),
                self.granted[i],
                controls::setter(&entity, move |this: &mut PermsPage, v| this.granted[i] = v),
            ));
        }
        Body::new()
            .playground(div().w(px(512.)).max_w_full().child(live))
            .hint("Open settings grants the permission in this demo")
            .config(config)
            .example(controls::block(
                "Reference preview",
                PermissionChecklist::new("perms-ref", permissions([true, true, false])),
            ))
    }
}

// ── TruncatedText ───────────────────────────────────────────────────────

const HASH: &str = "sha256:9f2c1e0a7b4d5e6f8a9b0c1d2e3f4a5b";
const PATH: &str = "packages/ui/src/components/date-picker/DatePicker.tsx";

struct TruncPage {
    keep: usize,
    width: usize,
    opened: SystemTime,
}

const KEEPS: [&str; 4] = ["4", "8", "10", "20"];
const WIDTHS: [&str; 3] = ["180", "260", "420"];

/// Build the TruncatedText page.
pub fn build_truncated_text(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TruncPage {
        keep: 2,
        width: 1,
        opened: SystemTime::now(),
    })
    .into()
}

/// The overview tile.
pub fn truncated_text_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .text_color(color(ColorRole::TextPrimary))
        .child(MiddleTruncate::new("trunc-thumb", HASH).keep(6))
        .into_any_element()
}

fn ago(secs: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(secs)
}

/// `components/TruncatedText/preview.html` (`cu-stack`, gap 12).
fn trunc_reference(
    prefix: &str,
    keep: usize,
    width: f32,
    opened: Option<SystemTime>,
) -> impl IntoElement {
    let id = |s: &str| SharedString::from(format!("{prefix}-{s}"));
    let line = || {
        div()
            .flex()
            .flex_row()
            .items_center()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
    };
    stack(12.)
        .child(
            line()
                .child("Middle:\u{a0}")
                .child(MiddleTruncate::new(id("hash"), HASH).keep(keep)),
        )
        .child(
            // `width: 260; display: flex; gap: 4; overflow: hidden`.
            line()
                .w(px(width))
                .gap(px(4.))
                .overflow_hidden()
                .child(div().flex_none().child("Start:"))
                .child(StartTruncatedPath::new(PATH)),
        )
        .child(
            line()
                .child("Relative:\u{a0}")
                .child(RelativeTime::new(id("t0"), ago(180)))
                .child("\u{a0}\u{b7}\u{a0}")
                .child(RelativeTime::new(id("t1"), ago(7200)))
                .when_some(opened, |this, t| {
                    this.child("\u{a0}\u{b7}\u{a0}")
                        .child(RelativeTime::new(id("t2"), t))
                }),
        )
}

impl Render for TruncPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let keep = KEEPS[self.keep].parse().unwrap_or(8);
        let width = WIDTHS[self.width].parse().unwrap_or(260.);
        let config = stack(24.)
            .child(controls::field(
                "MiddleTruncate keep",
                controls::choices(
                    "trunc-keep",
                    &KEEPS,
                    self.keep,
                    controls::setter(&entity, |this: &mut TruncPage, i| this.keep = i),
                ),
            ))
            .child(controls::field(
                "StartTruncatedPath width",
                controls::choices(
                    "trunc-width",
                    &WIDTHS,
                    self.width,
                    controls::setter(&entity, |this: &mut TruncPage, i| this.width = i),
                ),
            ));
        Body::new()
            .playground(
                stack(12.)
                    .child(trunc_reference(
                        "trunc-live",
                        keep,
                        width,
                        Some(self.opened),
                    ))
                    .child(note("The third time is when this page opened.")),
            )
            .hint("Hover the hash or a time for the full value")
            .config(config)
            .example(controls::block(
                "Reference preview",
                trunc_reference("trunc-ref", 10, 260., None),
            ))
            .example(controls::block(
                "Short text stays whole",
                text("Commit\u{a0}")
                    .flex()
                    .flex_row()
                    .child(MiddleTruncate::new("trunc-short", "a1b2c3d")),
            ))
    }
}
