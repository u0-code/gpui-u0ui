//! AgentRow: one coding agent in Settings › Agents (`AgentRow` in
//! `src/ghost/admin.jsx`, `.g-agent` in `src/ghost.css`): its provider mark,
//! name and version, then the install / update progress, "Not installed", or
//! the sign-in state with the account count and a newer version; Install,
//! Update and Sign in buttons as the state calls for, and an on/off switch.
//! While it installs or updates, a row given [`AgentRow::on_cancel`] shows
//! Cancel in the switch's place (`Welcome`'s `.wl-ag-inst`).
//!
//! The switch keeps its own state seeded from [`AgentRow::enabled`] (the
//! reference's `useState(enabled)`); a switched-off agent greys its name.
//!
//! ```ignore
//! AgentRow::new("agent-claude", "Claude")
//!     .version("2.1.0")
//!     .signed_in(true)
//!     .accounts(["work", "personal"])
//!     .accent(1)
//! ```
use crate::{
    components::{Button, ButtonSize, ButtonVariant, Switch, SwitchSize},
    motion::{self, Curve, Motion, Speed},
    t3::{
        IdentitySize, ProviderIcon,
        craft::{rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
    u0::{rows as row, rows::StateTone, tasks::EnabledState},
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, TestSupportExt as _, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// The agent's install state (`state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AgentState {
    /// `installed` (default).
    #[default]
    Installed,
    /// `installing`: a progress bar.
    Installing,
    /// `updating`: a progress bar.
    Updating,
    /// `missing`: not installed; an Install button, no switch.
    Missing,
}

impl AgentState {
    /// Every state, in `index.d.ts` order.
    pub const ALL: [Self; 4] = [
        Self::Installed,
        Self::Installing,
        Self::Updating,
        Self::Missing,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Installing => "installing",
            Self::Updating => "updating",
            Self::Missing => "missing",
        }
    }

    /// Whether a progress bar shows.
    pub const fn in_progress(self) -> bool {
        matches!(self, Self::Installing | Self::Updating)
    }
}

/// The row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct AgentRowTexts {
    /// `"Installing"`.
    pub installing: SharedString,
    /// `"Updating"`.
    pub updating: SharedString,
    /// `"Not installed"`.
    pub not_installed: SharedString,
    /// `"Signed in"`.
    pub signed_in: SharedString,
    /// The account count after "Signed in": `" · {n} account(s)"`.
    pub accounts: Rc<dyn Fn(usize) -> SharedString>,
    /// `"Not signed in"`.
    pub not_signed_in: SharedString,
    /// A newer version: `"· {latest} available"` (the span's leading space
    /// collapses at the start of its flex item).
    pub available: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Install"`.
    pub install: SharedString,
    /// `"Update"`.
    pub update: SharedString,
    /// `"Sign in"`.
    pub sign_in: SharedString,
    /// `"Cancel"`.
    pub cancel: SharedString,
    /// The switch's accessible name: `"{name} enabled"`.
    pub enabled: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for AgentRowTexts {
    fn default() -> Self {
        Self {
            installing: "Installing".into(),
            updating: "Updating".into(),
            not_installed: "Not installed".into(),
            signed_in: "Signed in".into(),
            accounts: Rc::new(|n| format!(" · {n} account{}", if n > 1 { "s" } else { "" }).into()),
            not_signed_in: "Not signed in".into(),
            available: Rc::new(|v| format!("· {v} available").into()),
            install: "Install".into(),
            update: "Update".into(),
            sign_in: "Sign in".into(),
            cancel: "Cancel".into(),
            enabled: Rc::new(|name| format!("{name} enabled").into()),
        }
    }
}

/// `.g-bar { display: inline-block; width: 8rem; height: 4px; background:
/// var(--layer-accent-01) }`.
pub const BAR_WIDTH: f32 = 128.;
/// The text column's least width in px, when buttons crowd a narrow row.
pub const MIN_TEXT_WIDTH: f32 = 56.;
/// `.g-bar { height: 4px }`.
pub const BAR_HEIGHT: f32 = 4.;
/// `.g-bar i { transition: width 240ms cubic-bezier(0.2,0,0.38,0.9) }`.
pub const BAR_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Moderate02);

/// The `.g-bar` progress track: `layer-accent-01` with an `interactive` fill
/// whose width animates (240ms productive). `id` keys the transition;
/// `progress` is 0–100.
pub fn progress_bar(
    id: impl Into<ElementId>,
    progress: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let target = (progress / 100.).clamp(0., 1.);
    let fraction = motion::animate(
        (id.into(), "g-bar"),
        target,
        BAR_MOTION.spatial(true),
        window,
        cx,
    );
    // An `inline-block` flex item of `.g-agent-prog`: 8rem, giving way
    // when the status beside it needs the room.
    div()
        .flex_shrink(1.)
        .min_w_0()
        .w(rem(BAR_WIDTH))
        .h(px(BAR_HEIGHT))
        .bg(color(ColorRole::LayerAccent01))
        .child(
            div()
                .h_full()
                .w(relative(fraction))
                .bg(color(ColorRole::Interactive)),
        )
        .into_any_element()
}

/// The name's ink: `text-primary`, `text-secondary` while off
/// (`.g-agent--off .g-slrow-main b { color: var(--text-secondary) }`).
pub fn name_ink(on: bool) -> Hsla {
    color(if on {
        ColorRole::TextPrimary
    } else {
        ColorRole::TextSecondary
    })
}

/// The buttons a row shows, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentButton {
    /// Tertiary "Install" (`state === 'missing'`).
    Install,
    /// Ghost "Update" (a newer version, installed).
    Update,
    /// Ghost "Sign in" (installed, not signed in).
    SignIn,
    /// Ghost "Cancel" (installing or updating, with [`AgentRow::on_cancel`]).
    Cancel,
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// An agent row. See the module docs.
#[derive(IntoElement)]
pub struct AgentRow {
    id: ElementId,
    name: SharedString,
    version: Option<SharedString>,
    state: AgentState,
    progress: f32,
    enabled: bool,
    signed_in: bool,
    accounts: Vec<SharedString>,
    latest: Option<SharedString>,
    accent: Option<u8>,
    texts: AgentRowTexts,
    on_install: Option<Handler>,
    on_update: Option<Handler>,
    on_sign_in: Option<Handler>,
    on_cancel: Option<Handler>,
    on_toggle: Option<ToggleHandler>,
}

impl AgentRow {
    /// A row for agent `name` (`name`), installed and enabled.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: None,
            state: AgentState::Installed,
            progress: 0.,
            enabled: true,
            signed_in: false,
            accounts: Vec::new(),
            latest: None,
            accent: None,
            texts: AgentRowTexts::default(),
            on_install: None,
            on_update: None,
            on_sign_in: None,
            on_cancel: None,
            on_toggle: None,
        }
    }

    /// The installed version (`version`), mono after the name.
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// The install state (`state`).
    pub fn state(mut self, state: AgentState) -> Self {
        self.state = state;
        self
    }

    /// Install / update progress, 0–100 (`progress`).
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    /// Whether the agent is switched on (`enabled`, default true).
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Signed in (`signedIn`).
    pub fn signed_in(mut self, signed_in: bool) -> Self {
        self.signed_in = signed_in;
        self
    }

    /// The signed-in accounts (`accounts`); only the count shows.
    pub fn accounts(mut self, accounts: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.accounts = accounts.into_iter().map(Into::into).collect();
        self
    }

    /// The newest version (`latest`); Update shows when it differs.
    pub fn latest(mut self, latest: impl Into<SharedString>) -> Self {
        self.latest = Some(latest.into());
        self
    }

    /// The provider mark's accent slot (`accent`, `agent-accent-N`).
    pub fn accent(mut self, accent: u8) -> Self {
        self.accent = Some(accent);
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: AgentRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Install pressed (`onInstall`).
    pub fn on_install(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_install = Some(Rc::new(f));
        self
    }

    /// Update pressed (`onUpdate`).
    pub fn on_update(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_update = Some(Rc::new(f));
        self
    }

    /// Sign in pressed (the reference leaves it unwired).
    pub fn on_sign_in(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_sign_in = Some(Rc::new(f));
        self
    }

    /// Cancel pressed: while installing or updating, Cancel takes the
    /// switch's place.
    pub fn on_cancel(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(f));
        self
    }

    /// Whether Cancel shows in the switch's place.
    fn cancels(&self) -> bool {
        self.state.in_progress() && self.on_cancel.is_some()
    }

    /// The switch was flipped.
    pub fn on_toggle(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// Whether a newer version is on offer (`latest && latest !== version`).
    pub fn has_update(&self) -> bool {
        self.latest
            .as_ref()
            .is_some_and(|l| Some(l) != self.version.as_ref())
    }

    /// The buttons, in the reference's order.
    pub fn buttons(&self) -> Vec<AgentButton> {
        let mut out = Vec::new();
        if self.state == AgentState::Missing {
            out.push(AgentButton::Install);
        } else if self.has_update() && self.state == AgentState::Installed {
            out.push(AgentButton::Update);
        }
        if !self.signed_in && self.state == AgentState::Installed {
            out.push(AgentButton::SignIn);
        }
        if self.cancels() {
            out.push(AgentButton::Cancel);
        }
        out
    }

    /// The second line's sign-in state, `None` while in progress or missing.
    fn sub_line(&self) -> Vec<AnyElement> {
        let mut runs = Vec::new();
        if self.signed_in {
            let mut t = self.texts.signed_in.to_string();
            if !self.accounts.is_empty() {
                t.push_str(&(self.texts.accounts)(self.accounts.len()));
            }
            runs.push(row::state_text_wrapping(StateTone::Ok, t).into_any_element());
        } else {
            runs.push(
                row::state_text_wrapping(StateTone::Warn, self.texts.not_signed_in.clone())
                    .into_any_element(),
            );
        }
        if self.has_update()
            && let Some(latest) = &self.latest
        {
            runs.push(
                row::dim((self.texts.available)(latest), 12.)
                    .min_w_0()
                    .whitespace_normal()
                    .into_any_element(),
            );
        }
        runs
    }
}

/// The progress status's least width: its longest word (12px Plex Sans,
/// `.g-agent-sub { font-size: 0.75rem }`), its CSS min-content.
pub fn status_min_width(status: &str, window: &Window) -> gpui_kit::Pixels {
    crate::typography::longest_word_width(status, 12., tokens::FontFamilyKey::Sans, 400, window)
}

impl RenderOnce for AgentRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let enabled = self.enabled;
        let store = window.use_keyed_state((id.clone(), "enabled"), cx, |_, _| {
            EnabledState::new(enabled)
        });
        let on = store.update(cx, |s, _| {
            s.sync(enabled);
            s.on()
        });

        // In a narrow column the version gives way first; the name keeps
        // its width up to the whole line, then ends in an ellipsis.
        let mut title = row::title(name_ink(on)).child(
            row::title_text(self.name.clone())
                .flex_shrink_0()
                .max_w_full(),
        );
        if let Some(v) = &self.version {
            // `<span className="g-dim g-mono"> {version}</span>`.
            title = title.child(
                row::mono_text(v.clone())
                    .min_w_0()
                    .overflow_hidden()
                    .text_color(color(ColorRole::TextSecondary)),
            );
        }

        // `.g-agent-sub { display: flex; align-items: center; gap:
        // var(--spacing-02); font-size: 0.75rem; color: var(--text-secondary) }`.
        // In a narrow column the runs wrap (as the reference's inline runs
        // do) instead of running under the buttons.
        let sub = sans(div(), 12., 400)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_02)
            .min_w_0()
            .text_color(color(ColorRole::TextSecondary));
        // The text column's least width: a few letters of the name, and the
        // progress status's longest word beside a collapsed bar.
        let mut text_min = rem(MIN_TEXT_WIDTH).to_pixels(window.rem_size());
        let sub = if self.state.in_progress() {
            let word = if self.state == AgentState::Installing {
                self.texts.installing.clone()
            } else {
                self.texts.updating.clone()
            };
            let status = format!("{word} {}%", self.progress.round() as i64);
            // The status is an anonymous flex item of `.g-agent-prog`: its
            // `min-width: auto` is its min-content, the longest word, so it
            // wraps between words only ("Installing" / "64%") and the bar
            // (min-content 0) is what gives way.
            let status_min = status_min_width(&status, window);
            text_min = text_min
                .max(status_min + rem(tokens::SPACING_03.px_value()).to_pixels(window.rem_size()));
            // `.g-agent-prog { display: inline-flex; align-items: center; gap:
            // var(--spacing-03); font-variant-numeric: tabular-nums }`.
            sub.child(
                div()
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .min_w_0()
                    .child(progress_bar(id.clone(), self.progress, window, cx))
                    .child(
                        div()
                            .id((id.clone(), "status"))
                            .test_support()
                            .flex_shrink(1.)
                            .min_w(status_min)
                            .child(status),
                    ),
            )
        } else if self.state == AgentState::Missing {
            sub.child(self.texts.not_installed.clone())
        } else {
            sub.children(self.sub_line())
        };

        let buttons = self.buttons();
        let texts = self.texts.clone();
        let (on_install, on_update, on_sign_in, on_cancel) = (
            self.on_install.clone(),
            self.on_update.clone(),
            self.on_sign_in.clone(),
            self.on_cancel.clone(),
        );
        let on_toggle = self.on_toggle.clone();
        let switch_label = (self.texts.enabled)(&self.name);
        let switched = self.state != AgentState::Missing && !self.cancels();

        // When even the text column's least width does not fit beside the
        // buttons and the switch, they wrap to a second line at the end
        // instead of running out of the row (the reference overflows).
        row::frame()
            .flex_wrap()
            .justify_end()
            .child(
                ProviderIcon::new(self.name.clone())
                    .id((id.clone(), "mark"))
                    .size(IdentitySize::Md)
                    .accent(self.accent),
            )
            .child(
                // The text column takes what the buttons leave and never
                // paints under them.
                div()
                    .flex()
                    .flex_col()
                    // `flex: 1` from a zero basis, so the row wraps only
                    // once the text is down to its least width.
                    .flex_grow(1.)
                    .flex_shrink(1.)
                    .flex_basis(px(0.))
                    // Never squeezed to a sliver: room for a few letters of
                    // the name ("Co…") and a word of the status per line.
                    .min_w(text_min)
                    .overflow_hidden()
                    .gap(px(2.))
                    .child(title)
                    .child(sub),
            )
            .children(buttons.into_iter().map(|b| {
                let (label, variant, handler, key) = match b {
                    AgentButton::Install => (
                        texts.install.clone(),
                        ButtonVariant::Tertiary,
                        on_install.clone(),
                        "install",
                    ),
                    AgentButton::Update => (
                        texts.update.clone(),
                        ButtonVariant::Ghost,
                        on_update.clone(),
                        "update",
                    ),
                    AgentButton::SignIn => (
                        texts.sign_in.clone(),
                        ButtonVariant::Ghost,
                        on_sign_in.clone(),
                        "sign-in",
                    ),
                    AgentButton::Cancel => (
                        texts.cancel.clone(),
                        ButtonVariant::Ghost,
                        on_cancel.clone(),
                        "cancel",
                    ),
                };
                row::tail(
                    Button::new((id.clone(), key), label)
                        .variant(variant)
                        .size(ButtonSize::Sm)
                        .when_some(handler, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                )
            }))
            .when(switched, |this| {
                this.child(row::tail(
                    Switch::new((id.clone(), "toggle"))
                        .size(SwitchSize::Sm)
                        .label(switch_label)
                        .hide_label(true)
                        .checked(on)
                        .on_checked_change(move |v, w, cx| {
                            store.update(cx, |s, cx| {
                                s.set(v);
                                cx.notify();
                            });
                            if let Some(f) = &on_toggle {
                                f(v, w, cx);
                            }
                        }),
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn buttons_follow_the_state() {
        let r = AgentRow::new("a", "Cursor").state(AgentState::Missing);
        assert_eq!(r.buttons(), vec![AgentButton::Install]);
        let r = AgentRow::new("a", "Codex")
            .version("0.9.2")
            .latest("0.10.0")
            .signed_in(true);
        assert_eq!(r.buttons(), vec![AgentButton::Update]);
        let r = AgentRow::new("a", "Aider").version("0.62");
        assert_eq!(r.buttons(), vec![AgentButton::SignIn]);
        let r = AgentRow::new("a", "Gemini")
            .state(AgentState::Installing)
            .progress(64.);
        assert!(r.buttons().is_empty());
        let r = AgentRow::new("a", "Gemini")
            .state(AgentState::Installing)
            .on_cancel(|_, _| {});
        assert_eq!(r.buttons(), vec![AgentButton::Cancel]);
        let r = AgentRow::new("a", "Gemini")
            .signed_in(true)
            .on_cancel(|_, _| {});
        assert!(r.buttons().is_empty());
        let r = AgentRow::new("a", "Claude")
            .version("2.1.0")
            .latest("2.1.0")
            .signed_in(true);
        assert!(!r.has_update());
        assert!(r.buttons().is_empty());
    }

    #[test]
    fn account_count_is_pluralised() {
        let t = AgentRowTexts::default();
        assert_eq!((t.accounts)(1), SharedString::from(" · 1 account"));
        assert_eq!((t.accounts)(2), SharedString::from(" · 2 accounts"));
        assert_eq!(
            (t.available)("0.10.0"),
            SharedString::from("· 0.10.0 available")
        );
    }

    #[test]
    fn bar_geometry_and_motion_follow_g_bar() {
        assert_eq!(BAR_WIDTH, 128.);
        assert_eq!(BAR_HEIGHT, 4.);
        assert_eq!(BAR_MOTION.speed, Speed::Moderate02);
        assert_eq!(BAR_MOTION.curve, Curve::ProductiveStandard);
    }

    #[test]
    fn switched_off_greys_the_name() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(name_ink(false), color(ColorRole::TextSecondary));
        assert_eq!(name_ink(true), color(ColorRole::TextPrimary));
    }
}
