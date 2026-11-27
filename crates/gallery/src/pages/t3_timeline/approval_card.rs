//! T3 · Timeline › ApprovalCard: a pending command with its reason and an
//! approved file change (`components/ApprovalCard/preview.html`), plus a
//! live card: Approve / Decline resolve it, and with shortcuts ⇧⌘↵ / ⇧⌘⌫
//! work while focus is inside. The options variant (u0 board
//! `PermissionCard`) shows the board's cases and a live card whose options
//! answer it (Return takes the default, Esc the first reject).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{
    ApprovalCard, ApprovalKind, ApprovalOption, ApprovalState, PermissionOptionKind,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ApprovalPage {
        kind: 0,
        state: 0,
        shortcuts: true,
        destructive: false,
        notes: true,
        always: true,
        last: "".into(),
        picked: None,
        show_kinds: true,
        extras: true,
    })
    .into()
}

struct ApprovalPage {
    kind: usize,
    state: usize,
    shortcuts: bool,
    destructive: bool,
    notes: bool,
    always: bool,
    last: SharedString,
    /// The live options card's answer.
    picked: Option<usize>,
    show_kinds: bool,
    extras: bool,
}

fn agent_options() -> Vec<ApprovalOption> {
    use PermissionOptionKind::*;
    vec![
        ApprovalOption::new("Allow once", AllowOnce),
        ApprovalOption::new("Allow always", AllowAlways),
        ApprovalOption::new("Reject once", RejectOnce),
        ApprovalOption::new("Reject always", RejectAlways),
    ]
}

/// The board's spoken words for an answer of `kind` (`doneWords`).
fn spoken(kind: PermissionOptionKind) -> &'static str {
    match kind {
        PermissionOptionKind::AllowOnce => "You allowed it once",
        PermissionOptionKind::AllowAlways => "You allowed it always",
        PermissionOptionKind::RejectOnce => "You rejected it",
        PermissionOptionKind::RejectAlways => "You rejected it always",
    }
}

/// The u0 board `PermissionCard` cases.
fn options_reference() -> impl IntoElement {
    use PermissionOptionKind::*;
    let tool = |t: &str| {
        vec![
            ApprovalOption::new("Allow", AllowOnce),
            ApprovalOption::new(t.to_string(), AllowAlways),
            ApprovalOption::new("Reject", RejectOnce),
        ]
    };
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_07)
        .w_full()
        .max_w(gpui_kit::rems(42.))
        .child(
            ApprovalCard::options("pmc-claude", agent_options())
                .title("Run a command")
                .provider("Claude · waiting 1m")
                .detail("pnpm playwright test --project=visual")
                .reason(
                    "Runs the visual tests in Chromium on This Mac. Writes only to test-results/.",
                )
                .show_kinds(true)
                .extra_action("No, and tell it…", CarbonIcon::Chat, |_, _| {})
                .note("The options Claude sent, in its order. Return takes “Allow once”."),
        )
        .child(
            ApprovalCard::options(
                "pmc-codex",
                vec![
                    ApprovalOption::new("Yes", AllowOnce),
                    ApprovalOption::new("Yes, and don’t ask again this session", AllowAlways),
                    ApprovalOption::new("No, and tell Codex what to do differently", RejectOnce),
                ],
            )
            .title("Run a command")
            .provider("Codex · waiting 3m")
            .detail("pnpm db:migrate --env=staging")
            .context("build-box · 1 migration · 0042_redis_sessions.sql")
            .show_kinds(true)
            .note("The options Codex sent, in its order. Return takes “Yes”."),
        )
        .child(
            ApprovalCard::options(
                "pmc-edit",
                vec![
                    ApprovalOption::new("Allow", AllowOnce),
                    ApprovalOption::new("Reject", RejectOnce),
                ],
            )
            .kind(ApprovalKind::File)
            .title("Edit src/tokens/g90.json")
            .provider("Antigravity")
            .detail("+8 −0")
            .extra_action("Revise…", CarbonIcon::Edit, |_, _| {})
            .note("The options Antigravity sent, in its order. Return takes “Allow”."),
        )
        .child(
            ApprovalCard::options("pmc-tool", tool("Always allow"))
                .tool(true)
                .title("Open a pull request")
                .provider("Claude asks to use u0’s open_pr")
                .facts([
                    ("Repository", "acme/carbon-ui"),
                    ("From", "feat/dark-tokens → main"),
                    ("Title", "Add dark theme tokens"),
                    ("Draft", "No"),
                ])
                .note("“Always allow” lets open_pr act from this thread without asking again."),
        )
        .child(
            ApprovalCard::options("pmc-done", agent_options())
                .title("Run a command")
                .provider("Claude")
                .detail("pnpm test tokens")
                .answered(true, "You allowed it once", "“Allow once” · 10:22"),
        )
}

/// The reference preview (`cu-stack`, gap 12, max 640px).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .w_full()
        .max_w(gpui_kit::rems(40.))
        .child(
            ApprovalCard::new(
                "appr-ref-0",
                ApprovalKind::Command,
                "pnpm prisma migrate deploy --schema ./db/schema.prisma",
            )
            .provider("Claude")
            .reason("Applies 2 pending migrations to the development database."),
        )
        .child(
            ApprovalCard::new(
                "appr-ref-1",
                ApprovalKind::File,
                "src/server/auth.ts  (+12 −4)",
            )
            .state(ApprovalState::Approved),
        )
}

impl Render for ApprovalPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let resolve = |state: ApprovalState, what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    if let Some(i) = ApprovalState::ALL.iter().position(|s| *s == state) {
                        this.state = i;
                    }
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let kind = ApprovalKind::ALL[self.kind];
        let detail = match kind {
            ApprovalKind::Command => "rm -rf node_modules && pnpm install",
            ApprovalKind::File => "src/server/auth.ts  (+12 −4)",
            ApprovalKind::Read => "~/.config/gh/hosts.yml",
            ApprovalKind::Permission => "Notifications: show desktop alerts",
            ApprovalKind::Access => "Figma: read the open file",
        };
        let mut card = ApprovalCard::new("appr-live", kind, detail)
            .provider("Claude · waiting 2m")
            .state(ApprovalState::ALL[self.state])
            .shortcuts(self.shortcuts)
            .destructive(self.destructive)
            .on_approve(resolve(ApprovalState::Approved, "onApprove"))
            .on_decline(resolve(ApprovalState::Declined, "onDecline"))
            .on_always(resolve(ApprovalState::Approved, "onAlways"));
        if self.notes {
            card = card
                .context("Runs in ~/code/web on build-box")
                .reason("Reinstalls dependencies after the lockfile changed.")
                .always_hint("Always allow covers pnpm install in this thread.");
        }
        if self.destructive {
            card = card.approve_label("Delete and reinstall");
        }
        if !self.always {
            card = card.always_label(None);
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(40.))
            .child(card)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let options = agent_options();
        let mut live_options = ApprovalCard::options("pmc-live", options.clone())
            .title("Run a command")
            .provider("Claude · waiting 1m")
            .detail("pnpm playwright test --project=visual")
            .reason("Runs the visual tests in Chromium on This Mac.")
            .show_kinds(self.show_kinds)
            .note("The options Claude sent, in its order. Return takes “Allow once”.")
            .on_pick({
                let entity = entity.clone();
                move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.picked = Some(i);
                        this.last = format!("onPick({i})").into();
                        cx.notify();
                    })
                }
            });
        if self.extras {
            live_options = live_options
                .extra_action("No, and tell it…", CarbonIcon::Chat, |_, _| {})
                .extra_action("Revise…", CarbonIcon::Edit, |_, _| {});
        }
        if let Some(i) = self.picked {
            let o = &options[i];
            live_options =
                live_options.answered(o.kind.allows(), spoken(o.kind), format!("“{}”", o.label));
        }
        let live = live.child(live_options);
        let kinds = ApprovalKind::ALL.map(|k| k.name());
        let states = ApprovalState::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "kind",
                controls::choices(
                    "appr-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "state",
                controls::choices(
                    "appr-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ))
            .child(controls::toggle(
                "appr-shortcuts",
                "shortcuts",
                self.shortcuts,
                controls::setter(&entity, |this: &mut Self, on| this.shortcuts = on),
            ))
            .child(controls::toggle(
                "appr-destructive",
                "destructive",
                self.destructive,
                controls::setter(&entity, |this: &mut Self, on| this.destructive = on),
            ))
            .child(controls::toggle(
                "appr-notes",
                "context / reason / alwaysHint",
                self.notes,
                controls::setter(&entity, |this: &mut Self, on| this.notes = on),
            ))
            .child(controls::toggle(
                "appr-always",
                "Always allow",
                self.always,
                controls::setter(&entity, |this: &mut Self, on| this.always = on),
            ))
            .child(controls::section("Options variant"))
            .child(controls::toggle(
                "pmc-kinds",
                "showKinds",
                self.show_kinds,
                controls::setter(&entity, |this: &mut Self, on| this.show_kinds = on),
            ))
            .child(controls::toggle(
                "pmc-extras",
                "extra actions",
                self.extras,
                controls::setter(&entity, |this: &mut Self, on| this.extras = on),
            ))
            .child(
                Button::new("pmc-reset", "Ask again")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.picked = None;
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .hint("Tab into the card: ⇧⌘↵ approves, ⇧⌘⌫ declines.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
            .example(controls::block(
                "Options (u0 PermissionCard)",
                options_reference(),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            ApprovalCard::new("appr-th", ApprovalKind::File, "src/server/auth.ts")
                .state(ApprovalState::Approved),
        )
        .into_any_element()
}
