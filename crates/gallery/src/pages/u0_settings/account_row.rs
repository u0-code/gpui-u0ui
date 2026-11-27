//! u0 · Settings › AccountRow: a live row for every token state, and the
//! four reference accounts.
use super::{list, note};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::u0::settings::{AccountRow, TokenState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        token: 0,
        forge: 0,
        last: SharedString::from("—"),
    })
    .into()
}

struct Page {
    token: usize,
    forge: usize,
    last: SharedString,
}

const TOKENS: [&str; 6] = ["ok", "cli", "expiring", "expired", "missing", "scopes"];
const FORGES: [&str; 3] = ["GitHub", "GitLab", "Gitea"];

/// `components/AccountRow/preview.html`: four accounts, 600px on `layer-01`.
fn reference() -> impl IntoElement {
    list(600.)
        .child(
            AccountRow::new("acct-0", "ana-dev")
                .host("github.com")
                .token(TokenState::Ok),
        )
        .child(
            AccountRow::new("acct-1", "ana")
                .forge("GitLab")
                .host("gitlab.example.com")
                .token(TokenState::Expiring),
        )
        .child(
            AccountRow::new("acct-2", "ana-bot")
                .host("github.com")
                .token(TokenState::Scopes),
        )
        .child(
            AccountRow::new("acct-3", "old-account")
                .forge("Gitea")
                .host("git.internal")
                .token(TokenState::Expired),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(
            AccountRow::new("acct-thumb", "ana-dev")
                .host("github.com")
                .token(TokenState::Ok),
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
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                list(600.).child(
                    AccountRow::new("acct-live", "ana-dev")
                        .forge(FORGES[self.forge])
                        .host("github.com")
                        .token(TokenState::ALL[self.token])
                        .on_replace(fired("onReplace"))
                        .on_open(fired("onOpen"))
                        .on_remove(fired("onRemove")),
                ),
            )
            .child(note(format!("Last: {}", self.last)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "token",
                controls::choices(
                    "acct-token",
                    &TOKENS,
                    self.token,
                    controls::setter(&entity, |p: &mut Page, i| p.token = i),
                ),
            ))
            .child(controls::field(
                "forge",
                controls::choices(
                    "acct-forge",
                    &FORGES,
                    self.forge,
                    controls::setter(&entity, |p: &mut Page, i| p.forge = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("The ⋮ menu opens Open / Replace token / Remove account")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
