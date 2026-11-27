//! T3 · Composer › Composer: an idle composer with three context chips and
//! a running one with dictation (`components/Composer/preview.html`), plus a
//! live composer: type and press Enter to send (Shift+Enter for a new line);
//! running, Enter queues and ⌘↵ steers; the footer controls light up.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rgb,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{
    AttachmentFile, AttachmentImage, AttachmentImageState, Composer, ComposerMenu, ComposerState,
    ContextChip, ContextChipKind, EffortLevel, VoiceInputPill, VoiceState,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ComposerPage {
        state: 0,
        steer: true,
        suggestion: false,
        attachments: false,
        voice: false,
        answering: false,
        open: None,
        log: Vec::new(),
    })
    .into()
}

struct ComposerPage {
    state: usize,
    steer: bool,
    suggestion: bool,
    attachments: bool,
    voice: bool,
    answering: bool,
    open: Option<ComposerMenu>,
    log: Vec<SharedString>,
}

fn contexts(prefix: &str) -> Vec<ContextChip> {
    let key = |s: &str| SharedString::from(format!("{prefix}-{s}"));
    vec![
        ContextChip::new(key("c0"), ContextChipKind::File, "tokens.json").on_remove(|_, _| {}),
        ContextChip::new(key("c1"), ContextChipKind::PrOpen, "#482 Dark theme"),
        ContextChip::new(key("c2"), ContextChipKind::Terminal, "dev · last 40 lines"),
    ]
}

/// The reference preview (`cu-stack`, max 720px).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .w_full()
        .max_w(gpui_kit::rems(45.))
        .child(
            Composer::new("composer-ref-0")
                .model("Opus", "Claude")
                .effort(EffortLevel::High, false)
                .contexts(contexts("composer-ref-0")),
        )
        .child(
            Composer::new("composer-ref-1")
                .value("Run the full test suite")
                .model("GPT", "Codex")
                .state(ComposerState::Running)
                .voice(
                    VoiceInputPill::new("composer-ref-1-voice", VoiceState::Recording).seconds(3),
                ),
        )
}

const STATES: [&str; 4] = ["idle", "running", "answering", "submitting"];
const MENUS: [&str; 4] = ["none", "model", "effort", "permissions"];

impl Render for ComposerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let log = |what: &'static str| {
            let entity = entity.clone();
            move |text: SharedString, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.log.insert(0, format!("{what}: {text}").into());
                    this.log.truncate(4);
                    cx.notify();
                })
            }
        };
        let toggle_menu = |menu: ComposerMenu| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.open = if this.open == Some(menu) {
                        None
                    } else {
                        Some(menu)
                    };
                    cx.notify();
                })
            }
        };
        let state = if self.answering {
            ComposerState::Idle
        } else {
            [
                ComposerState::Idle,
                ComposerState::Running,
                ComposerState::Answering,
                ComposerState::Submitting,
            ][self.state]
        };
        let mut composer = Composer::new("composer-live")
            .model("Opus", "Claude")
            .effort(EffortLevel::High, true)
            .contexts(contexts("composer-live"))
            .state(state)
            .open_control(self.open)
            .on_send(log("onSend"))
            .on_queue(log("onQueue"))
            .on_stop({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.log.insert(0, "onStop".into());
                        this.state = 0;
                        cx.notify();
                    })
                }
            })
            .on_model_click(toggle_menu(ComposerMenu::Model))
            .on_effort_click(toggle_menu(ComposerMenu::Effort))
            .on_permissions_click(toggle_menu(ComposerMenu::Permissions));
        if self.steer {
            composer = composer.on_steer(log("onSteer"));
        }
        if self.suggestion {
            composer = composer.suggestion("Open a PR against main with a short summary");
        }
        if self.attachments {
            composer = composer.attachments(
                vec![
                    AttachmentImage::new("shot.png").color(rgb(0x0f62fe)),
                    AttachmentImage::new("diagram.png")
                        .color(rgb(0x8a3ffc))
                        .state(AttachmentImageState::Uploading),
                ],
                vec![
                    AttachmentFile::new("design-spec.pdf")
                        .kind(ContextChipKind::DocPdf)
                        .size("2.4 MB"),
                ],
            );
        }
        if self.voice {
            composer = composer.voice(VoiceInputPill::new(
                "composer-live-voice",
                VoiceState::Recording,
            ));
        }
        if self.answering {
            composer = composer.send_label("Submit answers");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(45.))
            .child(composer)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .children(self.log.iter().cloned()),
            );
        let open = match self.open {
            None => 0,
            Some(ComposerMenu::Model) => 1,
            Some(ComposerMenu::Effort) => 2,
            Some(ComposerMenu::Permissions) => 3,
        };
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "composer-state",
                    &STATES,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "openControl",
                controls::choices(
                    "composer-open",
                    &MENUS,
                    open,
                    controls::setter(&entity, |this: &mut Self, i| {
                        this.open = [
                            None,
                            Some(ComposerMenu::Model),
                            Some(ComposerMenu::Effort),
                            Some(ComposerMenu::Permissions),
                        ][i];
                    }),
                ),
            ))
            .child(controls::toggle(
                "composer-steer",
                "onSteer",
                self.steer,
                controls::setter(&entity, |this: &mut Self, on| this.steer = on),
            ))
            .child(controls::toggle(
                "composer-suggestion",
                "suggestion",
                self.suggestion,
                controls::setter(&entity, |this: &mut Self, on| this.suggestion = on),
            ))
            .child(controls::toggle(
                "composer-attachments",
                "attachments",
                self.attachments,
                controls::setter(&entity, |this: &mut Self, on| this.attachments = on),
            ))
            .child(controls::toggle(
                "composer-voice",
                "voice",
                self.voice,
                controls::setter(&entity, |this: &mut Self, on| this.voice = on),
            ))
            .child(controls::toggle(
                "composer-answer",
                "sendLabel",
                self.answering,
                controls::setter(&entity, |this: &mut Self, on| this.answering = on),
            ));
        Body::new()
            .playground(live)
            .hint("Enter sends, Shift+Enter adds a line; running, Enter queues and ⌘↵ steers.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(Composer::new("composer-th").model("Opus", "Claude"))
        .into_any_element()
}
