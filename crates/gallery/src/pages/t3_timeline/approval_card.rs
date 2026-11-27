//! T3 · Timeline › ApprovalCard: a pending command with its reason and an
//! approved file change (`components/ApprovalCard/preview.html`), plus a
//! live card: Approve / Decline resolve it, and with shortcuts ⇧⌘↵ / ⇧⌘⌫
//! work while focus is inside.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{ApprovalCard, ApprovalKind, ApprovalState};

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
            ));
        Body::new()
            .playground(live)
            .hint("Tab into the card: ⇧⌘↵ approves, ⇧⌘⌫ declines.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
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
