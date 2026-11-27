//! u0 · Settings › AgentRow: a live row driven by the configuration
//! (state, progress, sign-in, update), and the five reference agents.
use super::{list, note};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use u0ui::prelude::*;
use u0ui::u0::settings::{AgentRow, AgentState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        state: 0,
        progress: 2,
        signed_in: true,
        accounts: 2,
        update: false,
        enabled: true,
        last: "—".into(),
    })
    .into()
}

struct Page {
    state: usize,
    progress: usize,
    signed_in: bool,
    accounts: usize,
    update: bool,
    enabled: bool,
    last: SharedString,
}

const STATES: [&str; 4] = ["installed", "installing", "updating", "missing"];
const PROGRESS: [&str; 4] = ["0", "30", "64", "100"];
const ACCOUNTS: [&str; 3] = ["0", "1", "2"];

/// `components/AgentRow/preview.html`: five agents, 600px on `layer-01`.
fn reference() -> impl IntoElement {
    list(600.)
        .child(
            AgentRow::new("agent-0", "Claude")
                .version("2.1.0")
                .signed_in(true)
                .accounts(["work", "personal"])
                .accent(1),
        )
        .child(
            AgentRow::new("agent-1", "Codex")
                .version("0.9.2")
                .latest("0.10.0")
                .signed_in(true),
        )
        .child(
            AgentRow::new("agent-2", "Gemini")
                .state(AgentState::Installing)
                .progress(64.),
        )
        .child(AgentRow::new("agent-3", "Aider").version("0.62"))
        .child(AgentRow::new("agent-4", "Cursor").state(AgentState::Missing))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(
            AgentRow::new("agent-thumb", "Claude")
                .version("2.1.0")
                .signed_in(true),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let fired = |what: &'static str| {
            let e = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                e.update(cx, |p, cx| {
                    p.last = what.into();
                    cx.notify();
                })
            }
        };
        let accounts: Vec<&str> = ["work", "personal"][..self.accounts].to_vec();
        let toggled = entity.clone();
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                list(600.).child(
                    AgentRow::new("agent-live", "Codex")
                        .version("0.9.2")
                        .when(self.update, |r| r.latest("0.10.0"))
                        .state(AgentState::ALL[self.state])
                        .progress(PROGRESS[self.progress].parse().unwrap_or(0.))
                        .signed_in(self.signed_in)
                        .accounts(accounts)
                        .enabled(self.enabled)
                        .on_install(fired("onInstall"))
                        .on_update(fired("onUpdate"))
                        .on_sign_in(fired("Sign in"))
                        .on_cancel(fired("onCancel"))
                        .on_toggle(move |on, _, cx| {
                            toggled.update(cx, |p, cx| {
                                p.last = format!("switch → {on}").into();
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(note(format!("Last: {}", self.last)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "state",
                controls::choices(
                    "agent-state",
                    &STATES,
                    self.state,
                    controls::setter(&entity, |p: &mut Page, i| p.state = i),
                ),
            ))
            .child(controls::field(
                "progress",
                controls::choices(
                    "agent-progress",
                    &PROGRESS,
                    self.progress,
                    controls::setter(&entity, |p: &mut Page, i| p.progress = i),
                ),
            ))
            .child(controls::field(
                "accounts",
                controls::choices(
                    "agent-accounts",
                    &ACCOUNTS,
                    self.accounts,
                    controls::setter(&entity, |p: &mut Page, i| p.accounts = i),
                ),
            ))
            .child(controls::toggle(
                "agent-signed",
                "signedIn",
                self.signed_in,
                controls::setter(&entity, |p: &mut Page, v| p.signed_in = v),
            ))
            .child(controls::toggle(
                "agent-latest",
                "latest (0.10.0)",
                self.update,
                controls::setter(&entity, |p: &mut Page, v| p.update = v),
            ))
            .child(controls::toggle(
                "agent-enabled",
                "enabled",
                self.enabled,
                controls::setter(&entity, |p: &mut Page, v| p.enabled = v),
            ));
        Body::new()
            .playground(live)
            .hint("Change progress to watch the bar's 240ms width transition")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
