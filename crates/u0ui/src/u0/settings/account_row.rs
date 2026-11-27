//! AccountRow: a code-host account and its token (`AccountRow` in
//! `src/ghost/admin.jsx`, `.g-acct` in `src/ghost.css`): an initials disc
//! (no forge logo), login · forge, the host in mono, the token state as glyph
//! and words, one ghost action that fixes the state ("Replace token", "Renew
//! token", "Fix scopes"…) and an overflow menu (Open on {forge}, Replace
//! token, Remove account).
//!
//! ```ignore
//! AccountRow::new("acct-ana", "ana-dev").host("github.com").token(TokenState::Ok)
//! ```
use crate::{
    components::Button,
    components::{ButtonSize, ButtonVariant, MenuItem},
    icons::CarbonIcon,
    t3::{
        IconMenu,
        craft::{rem, sans},
    },
    theme::color,
    tokens::ColorRole,
    u0::{rows as row, rows::StateTone},
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The token's state (`token`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TokenState {
    /// `ok`: "Token valid" (default).
    #[default]
    Ok,
    /// `cli`: signed in through the forge's CLI; no token to replace.
    Cli,
    /// `expiring`: "Expires in 3 days".
    Expiring,
    /// `expired`: "Token expired".
    Expired,
    /// `missing`: "No token".
    Missing,
    /// `scopes`: "Missing repo scope".
    Scopes,
}

impl TokenState {
    /// Every state, in the reference's `TOKEN` order.
    pub const ALL: [Self; 6] = [
        Self::Ok,
        Self::Cli,
        Self::Expiring,
        Self::Expired,
        Self::Missing,
        Self::Scopes,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Cli => "cli",
            Self::Expiring => "expiring",
            Self::Expired => "expired",
            Self::Missing => "missing",
            Self::Scopes => "scopes",
        }
    }

    /// The tone (`TOKEN[token][1]`).
    pub const fn tone(self) -> StateTone {
        match self {
            Self::Ok | Self::Cli => StateTone::Ok,
            Self::Expired => StateTone::Err,
            Self::Expiring | Self::Missing | Self::Scopes => StateTone::Warn,
        }
    }
}

/// The row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct AccountRowTexts {
    /// The state words (`TOKEN[token][0]`).
    pub state: Rc<dyn Fn(TokenState) -> SharedString>,
    /// The fixing action's label for a state.
    pub action: Rc<dyn Fn(TokenState) -> SharedString>,
    /// The overflow menu's accessible name: `"More actions for {login}"`.
    pub more: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Open on {forge}"`.
    pub open_on: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Replace token"`.
    pub replace: SharedString,
    /// `"Remove account"`.
    pub remove: SharedString,
}

impl Default for AccountRowTexts {
    fn default() -> Self {
        Self {
            state: Rc::new(|t| {
                match t {
                    TokenState::Ok => "Token valid",
                    TokenState::Cli => "Signed in",
                    TokenState::Expiring => "Expires in 3 days",
                    TokenState::Expired => "Token expired",
                    TokenState::Missing => "No token",
                    TokenState::Scopes => "Missing repo scope",
                }
                .into()
            }),
            action: Rc::new(|t| {
                match t {
                    TokenState::Ok => "Replace token",
                    TokenState::Cli => "Check sign-in",
                    TokenState::Expiring | TokenState::Expired => "Renew token",
                    TokenState::Missing => "Add token",
                    TokenState::Scopes => "Fix scopes",
                }
                .into()
            }),
            more: Rc::new(|login| format!("More actions for {login}").into()),
            open_on: Rc::new(|forge| format!("Open on {forge}").into()),
            replace: "Replace token".into(),
            remove: "Remove account".into(),
        }
    }
}

/// The disc's initials: `(login || '?').slice(0, 2).toUpperCase()`.
pub fn initials(login: &str) -> String {
    let src = if login.is_empty() { "?" } else { login };
    src.chars().take(2).collect::<String>().to_uppercase()
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An account row. See the module docs.
#[derive(IntoElement)]
pub struct AccountRow {
    id: ElementId,
    forge: SharedString,
    login: SharedString,
    host: Option<SharedString>,
    token: TokenState,
    avatar: Option<SharedString>,
    action_label: Option<SharedString>,
    texts: AccountRowTexts,
    on_replace: Option<Handler>,
    on_open: Option<Handler>,
    on_remove: Option<Handler>,
}

impl AccountRow {
    /// A row for account `login` (`login`) on GitHub with a valid token.
    pub fn new(id: impl Into<ElementId>, login: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            forge: "GitHub".into(),
            login: login.into(),
            host: None,
            token: TokenState::Ok,
            avatar: None,
            action_label: None,
            texts: AccountRowTexts::default(),
            on_replace: None,
            on_open: None,
            on_remove: None,
        }
    }

    /// The forge's name (`forge`, default "GitHub").
    pub fn forge(mut self, forge: impl Into<SharedString>) -> Self {
        self.forge = forge.into();
        self
    }

    /// The host (`host`), mono on the second line.
    pub fn host(mut self, host: impl Into<SharedString>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// The token state (`token`).
    pub fn token(mut self, token: TokenState) -> Self {
        self.token = token;
        self
    }

    /// The disc's text (`avatar`), instead of the login's first two letters.
    pub fn avatar(mut self, avatar: impl Into<SharedString>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    /// Override the action's label (`actionLabel`).
    pub fn action_label(mut self, label: impl Into<SharedString>) -> Self {
        self.action_label = Some(label.into());
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: AccountRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// The action button and "Replace token" (`onReplace`).
    pub fn on_replace(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_replace = Some(Rc::new(f));
        self
    }

    /// "Open on {forge}" (`onOpen`).
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// "Remove account" (`onRemove`).
    pub fn on_remove(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }

    /// The action button's label.
    pub fn action(&self) -> SharedString {
        self.action_label
            .clone()
            .unwrap_or_else(|| (self.texts.action)(self.token))
    }

    /// The overflow menu's labels, `None` for the divider.
    pub fn menu_labels(&self) -> Vec<Option<SharedString>> {
        let mut out = vec![Some((self.texts.open_on)(&self.forge))];
        if self.token != TokenState::Cli {
            out.push(Some(self.texts.replace.clone()));
        }
        out.push(None);
        out.push(Some(self.texts.remove.clone()));
        out
    }
}

fn wire(item: MenuItem, f: Option<Handler>) -> MenuItem {
    match f {
        Some(f) => item.on_select(move |w, cx| f(w, cx)),
        None => item,
    }
}

impl RenderOnce for AccountRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let tone = self.token.tone();
        let disc = self
            .avatar
            .clone()
            .unwrap_or_else(|| initials(&self.login).into());
        let mut items = vec![wire(
            MenuItem::item((self.texts.open_on)(&self.forge)),
            self.on_open.clone(),
        )];
        if self.token != TokenState::Cli {
            items.push(wire(
                MenuItem::item(self.texts.replace.clone()),
                self.on_replace.clone(),
            ));
        }
        items.push(MenuItem::divider());
        items.push(wire(
            MenuItem::item(self.texts.remove.clone()).danger(true),
            self.on_remove.clone(),
        ));
        let on_replace = self.on_replace.clone();

        row::frame()
            .child(
                // `.g-acct-av { width: 2rem; height: 2rem; background:
                // var(--layer-accent-01); font-size: 0.75rem; font-weight: 600;
                // border-radius: 50% }`.
                sans(div(), 12., 600)
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(32.))
                    .rounded_full()
                    .bg(color(ColorRole::LayerAccent01))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(disc),
            )
            .child(row::slrow_main(
                row::title(color(ColorRole::TextPrimary))
                    .child(row::title_text(self.login.clone()))
                    // `<span className="g-dim"> · {forge}</span>` in the `b` flex
                    // (gap 8px): its leading space collapses.
                    .child(row::dim(format!("· {}", self.forge), 14.)),
                self.host.clone().map(|h| {
                    row::mono_text(h)
                        .text_color(color(ColorRole::TextSecondary))
                        .into_any_element()
                }),
            ))
            .child(row::state_text(tone, (self.texts.state)(self.token)))
            .child(row::tail(
                Button::new((id.clone(), "action"), self.action())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .when_some(on_replace, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            ))
            .child(row::tail(
                IconMenu::new((id.clone(), "more"), (self.texts.more)(&self.login))
                    .icon(CarbonIcon::OverflowMenuVertical)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .items(items),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_states_map_to_tone_words_and_action() {
        let t = AccountRowTexts::default();
        assert_eq!(TokenState::Ok.tone(), StateTone::Ok);
        assert_eq!(TokenState::Cli.tone(), StateTone::Ok);
        assert_eq!(TokenState::Expired.tone(), StateTone::Err);
        assert_eq!(TokenState::Scopes.tone(), StateTone::Warn);
        assert_eq!(
            (t.state)(TokenState::Expiring),
            SharedString::from("Expires in 3 days")
        );
        assert_eq!(
            (t.action)(TokenState::Expired),
            SharedString::from("Renew token")
        );
        assert_eq!(
            (t.action)(TokenState::Scopes),
            SharedString::from("Fix scopes")
        );
        let row = AccountRow::new("a", "ana").action_label("Fix");
        assert_eq!(row.action(), SharedString::from("Fix"));
    }

    #[test]
    fn cli_accounts_have_no_replace_item() {
        let row = AccountRow::new("a", "ana").forge("GitLab");
        assert_eq!(
            row.menu_labels(),
            vec![
                Some("Open on GitLab".into()),
                Some("Replace token".into()),
                None,
                Some("Remove account".into())
            ]
        );
        let row = AccountRow::new("a", "ana").token(TokenState::Cli);
        assert_eq!(row.menu_labels().len(), 3);
    }

    #[test]
    fn initials_are_the_first_two_letters() {
        assert_eq!(initials("ana-dev"), "AN");
        assert_eq!(initials(""), "?");
        assert_eq!(initials("o"), "O");
    }
}
