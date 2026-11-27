//! The device list rows (`src/ghost/admin.jsx` "Devices", `.g-dev` and
//! `.g-tunnel` in `src/ghost.css`):
//!
//! - [`DeviceRow`]: a paired device: its machine glyph, name with "This
//!   device" / "Can control" tags, fingerprint, last seen, version, and a
//!   danger-ghost "Revoke" (never on this device).
//! - [`MacRow`]: a Mac seen from a phone or another Mac: its connection
//!   state (dot, or a spinner while reaching), version and latency, "Reach
//!   again" while not online, and "Forget".
//! - [`TunnelRow`]: the cloudflared tunnel: state, uptime, the public
//!   address in a [`CopyField`], and Stop / Start / Install.
use crate::{
    components::{Badge, BadgeSize, BadgeVariant, Button, ButtonSize, ButtonVariant},
    ghost::{primitives::CopyField, rows as row},
    icons::{CarbonIcon, Icon, IconSize},
    t3::{
        ConnectionStatus, ConnectionStatusDot, EnvironmentMachineIcon, MachineKind, Spin,
        craft::rem,
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

fn wire(b: Button, f: Option<Handler>) -> Button {
    match f {
        Some(f) => b.on_click(move |_, w, cx| f(w, cx)),
        None => b,
    }
}

fn text(s: impl Into<SharedString>) -> AnyElement {
    div().whitespace_nowrap().child(s.into()).into_any_element()
}

// ── DeviceRow ────────────────────────────────────────────────────────────

/// [`DeviceRow`]'s user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct DeviceRowTexts {
    /// `"This device"`.
    pub this_device: SharedString,
    /// `"Can control"`.
    pub can_control: SharedString,
    /// `"Revoke"`.
    pub revoke: SharedString,
}

impl Default for DeviceRowTexts {
    fn default() -> Self {
        Self {
            this_device: "This device".into(),
            can_control: "Can control".into(),
            revoke: "Revoke".into(),
        }
    }
}

/// A paired device. See the module docs.
#[derive(IntoElement)]
pub struct DeviceRow {
    id: ElementId,
    name: SharedString,
    kind: MachineKind,
    fingerprint: Option<SharedString>,
    last_seen: Option<SharedString>,
    version: Option<SharedString>,
    current: bool,
    can_control: bool,
    texts: DeviceRowTexts,
    on_revoke: Option<Handler>,
}

impl DeviceRow {
    /// A phone named `name`.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: MachineKind::Phone,
            fingerprint: None,
            last_seen: None,
            version: None,
            current: false,
            can_control: false,
            texts: DeviceRowTexts::default(),
            on_revoke: None,
        }
    }

    /// The machine kind (`kind`, default phone; other names such as
    /// `tablet` fall back to the laptop glyph, as `MACHINE[kind] || Laptop`).
    pub fn kind(mut self, kind: MachineKind) -> Self {
        self.kind = kind;
        self
    }

    /// The key fingerprint (`fingerprint`), mono.
    pub fn fingerprint(mut self, fingerprint: impl Into<SharedString>) -> Self {
        self.fingerprint = Some(fingerprint.into());
        self
    }

    /// When it was last seen (`lastSeen`).
    pub fn last_seen(mut self, last_seen: impl Into<SharedString>) -> Self {
        self.last_seen = Some(last_seen.into());
        self
    }

    /// Its app version (`version`), shown as `v{version}`.
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// This device (`current`): tagged, and it can't be revoked.
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// It may control this Mac (`canControl`).
    pub fn can_control(mut self, can_control: bool) -> Self {
        self.can_control = can_control;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: DeviceRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Revoke pressed (`onRevoke`).
    pub fn on_revoke(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_revoke = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for DeviceRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let title = row::title(color(ColorRole::TextPrimary))
            .child(row::title_text(self.name.clone()))
            .when(self.current, |t| {
                t.child(
                    Badge::new(self.texts.this_device.clone())
                        .variant(BadgeVariant::Blue)
                        .size(BadgeSize::Sm),
                )
            })
            .when(self.can_control, |t| {
                t.child(
                    Badge::new(self.texts.can_control.clone())
                        .variant(BadgeVariant::Purple)
                        .size(BadgeSize::Sm),
                )
            });
        let mut meta = Vec::new();
        if let Some(f) = &self.fingerprint {
            meta.push(row::mono_text(f.clone()).into_any_element());
        }
        if let Some(s) = &self.last_seen {
            meta.push(text(s.clone()));
        }
        if let Some(v) = &self.version {
            meta.push(text(format!("v{v}")));
        }
        row::frame()
            .child(EnvironmentMachineIcon::new(self.kind).size(20.))
            .child(row::slrow_main(title, [row::meta(meta).into_any_element()]))
            .when(!self.current, |this| {
                this.child(row::tail(wire(
                    Button::new((id.clone(), "revoke"), self.texts.revoke.clone())
                        .variant(ButtonVariant::DangerGhost)
                        .size(ButtonSize::Sm),
                    self.on_revoke.clone(),
                )))
            })
    }
}

// ── MacRow ───────────────────────────────────────────────────────────────

/// A Mac's connection state (`state`, as `ServerHeader`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MacState {
    /// `online` (default).
    #[default]
    Online,
    /// `reaching`: a spinner.
    Reaching,
    /// `turned-away`: it refused the connection.
    TurnedAway,
    /// `offline`.
    Offline,
}

impl MacState {
    /// Every state.
    pub const ALL: [Self; 4] = [
        Self::Online,
        Self::Reaching,
        Self::TurnedAway,
        Self::Offline,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Reaching => "reaching",
            Self::TurnedAway => "turned-away",
            Self::Offline => "offline",
        }
    }

    /// The dot's status: online → connected, turned away → error, else
    /// offline (`reaching` draws a spinner instead).
    pub const fn dot(self) -> ConnectionStatus {
        match self {
            Self::Online => ConnectionStatus::Connected,
            Self::TurnedAway => ConnectionStatus::Error,
            Self::Reaching | Self::Offline => ConnectionStatus::Offline,
        }
    }
}

/// [`MacRow`]'s user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct MacRowTexts {
    /// The state words.
    pub state: Rc<dyn Fn(MacState) -> SharedString>,
    /// The latency: `"{n} ms"`.
    pub latency: Rc<dyn Fn(u32) -> SharedString>,
}

impl Default for MacRowTexts {
    fn default() -> Self {
        Self {
            state: Rc::new(|s| {
                match s {
                    MacState::Online => "Online",
                    MacState::Reaching => "Reaching…",
                    MacState::TurnedAway => "Turned away",
                    MacState::Offline => "Offline",
                }
                .into()
            }),
            latency: Rc::new(|n| format!("{n} ms").into()),
        }
    }
}

/// The leading glyph of a [`MacRow`] (`icon`: a machine kind or any icon).
#[derive(Clone)]
pub enum MacIcon {
    /// A machine kind (`'laptop' | 'desktop' | …`).
    Machine(MachineKind),
    /// Any icon (`.t3-machine`: `icon-secondary`).
    Icon(Box<Icon>),
    /// None (`icon={null}`).
    None,
}

/// The `.g-statetext` run: a 12px spinner while reaching, else the dot, then
/// the state's words.
fn state_text(id: &ElementId, state: MacState, words: SharedString) -> AnyElement {
    let mark = if state == MacState::Reaching {
        Spin::new((id.clone(), "spin")).size(12.).into_any_element()
    } else {
        ConnectionStatusDot::new(state.dot())
            .id((id.clone(), "dot"))
            .into_any_element()
    };
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_02)
        .whitespace_nowrap()
        .child(mark)
        .child(words)
        .into_any_element()
}

/// A Mac seen from another device. See the module docs.
#[derive(IntoElement)]
pub struct MacRow {
    id: ElementId,
    name: SharedString,
    state: MacState,
    version: Option<SharedString>,
    latency: Option<u32>,
    icon: MacIcon,
    action_label: Option<SharedString>,
    reach_label: SharedString,
    texts: MacRowTexts,
    on_forget: Option<Handler>,
    on_reach: Option<Handler>,
}

impl MacRow {
    /// An online laptop named `name`, with "Forget" and "Reach again".
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state: MacState::Online,
            version: None,
            latency: None,
            icon: MacIcon::Machine(MachineKind::Laptop),
            action_label: Some("Forget".into()),
            reach_label: "Reach again".into(),
            texts: MacRowTexts::default(),
            on_forget: None,
            on_reach: None,
        }
    }

    /// The connection state (`state`).
    pub fn state(mut self, state: MacState) -> Self {
        self.state = state;
        self
    }

    /// Its app version (`version`).
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Round-trip latency in ms (`latency`).
    pub fn latency(mut self, latency: u32) -> Self {
        self.latency = Some(latency);
        self
    }

    /// The leading glyph (`icon`, default laptop).
    pub fn icon(mut self, icon: MacIcon) -> Self {
        self.icon = icon;
        self
    }

    /// The trailing action's label (`actionLabel`, default "Forget"; `None`
    /// hides it).
    pub fn action_label(mut self, label: Option<SharedString>) -> Self {
        self.action_label = label;
        self
    }

    /// The reach button's label (`reachLabel`, default "Reach again").
    pub fn reach_label(mut self, label: impl Into<SharedString>) -> Self {
        self.reach_label = label.into();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: MacRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// The action pressed (`onForget`).
    pub fn on_forget(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_forget = Some(Rc::new(f));
        self
    }

    /// "Reach again" pressed (`onReach`).
    pub fn on_reach(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reach = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for MacRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let icon: Option<AnyElement> = match self.icon.clone() {
            MacIcon::Machine(k) => {
                Some(EnvironmentMachineIcon::new(k).size(20.).into_any_element())
            }
            MacIcon::Icon(i) => Some(
                (*i).size(IconSize::S20)
                    .color(color(ColorRole::IconSecondary))
                    .into_any_element(),
            ),
            MacIcon::None => None,
        };
        let mut meta = vec![state_text(&id, self.state, (self.texts.state)(self.state))];
        if let Some(v) = &self.version {
            meta.push(text(format!("v{v}")));
        }
        if let Some(l) = self.latency {
            meta.push(text((self.texts.latency)(l)));
        }
        row::frame()
            .children(icon.map(|i| div().flex().flex_none().child(i)))
            .child(row::slrow_main(
                row::title(color(ColorRole::TextPrimary)).child(row::title_text(self.name.clone())),
                [row::meta(meta).into_any_element()],
            ))
            .when(self.state != MacState::Online, |this| {
                this.child(row::tail(wire(
                    Button::new((id.clone(), "reach"), self.reach_label.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm),
                    self.on_reach.clone(),
                )))
            })
            .when_some(self.action_label.clone(), |this, label| {
                this.child(row::tail(wire(
                    Button::new((id.clone(), "forget"), label)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm),
                    self.on_forget.clone(),
                )))
            })
    }
}

// ── TunnelRow ────────────────────────────────────────────────────────────

/// The tunnel's state (`state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TunnelState {
    /// `running` (default).
    #[default]
    Running,
    /// `starting`: a spinner, Start disabled.
    Starting,
    /// `stopped`.
    Stopped,
    /// `error`: failed to start.
    Error,
    /// `missing`: cloudflared isn't installed; Install.
    Missing,
}

impl TunnelState {
    /// Every state.
    pub const ALL: [Self; 5] = [
        Self::Running,
        Self::Starting,
        Self::Stopped,
        Self::Error,
        Self::Missing,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Starting => "starting",
            Self::Stopped => "stopped",
            Self::Error => "error",
            Self::Missing => "missing",
        }
    }

    /// The dot's status (`lab[state][1]`).
    pub const fn dot(self) -> ConnectionStatus {
        match self {
            Self::Running => ConnectionStatus::Connected,
            Self::Starting => ConnectionStatus::Connecting,
            Self::Error => ConnectionStatus::Error,
            Self::Stopped | Self::Missing => ConnectionStatus::Offline,
        }
    }
}

/// [`TunnelRow`]'s user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct TunnelRowTexts {
    /// `"Tunnel"`.
    pub title: SharedString,
    /// `"cloudflared"`.
    pub tool: SharedString,
    /// The state words (`lab[state][0]`).
    pub state: Rc<dyn Fn(TunnelState) -> SharedString>,
    /// The uptime: `"up {uptime}"`.
    pub uptime: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Stop"` (`stopLabel`).
    pub stop: SharedString,
    /// `"Start"` (`startLabel`).
    pub start: SharedString,
    /// `"Install"`.
    pub install: SharedString,
}

impl Default for TunnelRowTexts {
    fn default() -> Self {
        Self {
            title: "Tunnel".into(),
            tool: "cloudflared".into(),
            state: Rc::new(|s| {
                match s {
                    TunnelState::Running => "Running",
                    TunnelState::Starting => "Starting…",
                    TunnelState::Stopped => "Stopped",
                    TunnelState::Error => "Failed to start",
                    TunnelState::Missing => "cloudflared not installed",
                }
                .into()
            }),
            uptime: Rc::new(|u| format!("up {u}").into()),
            stop: "Stop".into(),
            start: "Start".into(),
            install: "Install".into(),
        }
    }
}

/// Which button a tunnel row shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelButton {
    /// Secondary "Stop" (running).
    Stop,
    /// Tertiary "Install" (missing).
    Install,
    /// Tertiary "Start" (disabled while starting).
    Start {
        /// `disabled={state === 'starting'}`.
        disabled: bool,
    },
}

impl TunnelButton {
    /// The button for `state`.
    pub const fn of(state: TunnelState) -> Self {
        match state {
            TunnelState::Running => Self::Stop,
            TunnelState::Missing => Self::Install,
            TunnelState::Starting => Self::Start { disabled: true },
            TunnelState::Stopped | TunnelState::Error => Self::Start { disabled: false },
        }
    }
}

/// The tunnel row. See the module docs.
#[derive(IntoElement)]
pub struct TunnelRow {
    id: ElementId,
    state: TunnelState,
    address: Option<SharedString>,
    uptime: Option<SharedString>,
    texts: TunnelRowTexts,
    on_start: Option<Handler>,
    on_stop: Option<Handler>,
    on_install: Option<Handler>,
}

impl TunnelRow {
    /// A running tunnel.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            state: TunnelState::Running,
            address: None,
            uptime: None,
            texts: TunnelRowTexts::default(),
            on_start: None,
            on_stop: None,
            on_install: None,
        }
    }

    /// The state (`state`).
    pub fn state(mut self, state: TunnelState) -> Self {
        self.state = state;
        self
    }

    /// The public address (`address`), shown while running.
    pub fn address(mut self, address: impl Into<SharedString>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// How long it has run (`uptime`), shown while running.
    pub fn uptime(mut self, uptime: impl Into<SharedString>) -> Self {
        self.uptime = Some(uptime.into());
        self
    }

    /// The user-facing strings (`stopLabel` / `startLabel` live here).
    pub fn texts(mut self, texts: TunnelRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Start pressed (`onStart`).
    pub fn on_start(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_start = Some(Rc::new(f));
        self
    }

    /// Stop pressed (`onStop`).
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }

    /// Install pressed (the reference leaves it unwired).
    pub fn on_install(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_install = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for TunnelRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let running = self.state == TunnelState::Running;
        let mark = if self.state == TunnelState::Starting {
            Spin::new((id.clone(), "spin")).size(12.).into_any_element()
        } else {
            ConnectionStatusDot::new(self.state.dot())
                .id((id.clone(), "dot"))
                .into_any_element()
        };
        let mut meta = vec![mark, text((self.texts.state)(self.state))];
        if running && let Some(u) = &self.uptime {
            meta.push(text((self.texts.uptime)(u)));
        }
        let address = self.address.clone().filter(|_| running);
        let button = match TunnelButton::of(self.state) {
            TunnelButton::Stop => wire(
                Button::new((id.clone(), "stop"), self.texts.stop.clone())
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm),
                self.on_stop.clone(),
            ),
            TunnelButton::Install => wire(
                Button::new((id.clone(), "install"), self.texts.install.clone())
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm),
                self.on_install.clone(),
            ),
            TunnelButton::Start { disabled } => wire(
                Button::new((id.clone(), "start"), self.texts.start.clone())
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .disabled(disabled),
                self.on_start.clone(),
            ),
        };
        // `.g-tunnel .g-slrow-main { gap: var(--spacing-02) }`.
        let main = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(tokens::SPACING_02)
            .child(
                row::title(color(ColorRole::TextPrimary))
                    .child(self.texts.title.clone())
                    .child(row::dim(self.texts.tool.clone(), 14.)),
            )
            .child(row::meta(meta))
            .when_some(address, |m, a| {
                // `.g-tunnel .g-field { max-width: 20rem }`.
                m.child(
                    div()
                        .w_full()
                        .max_w(rem(320.))
                        .child(CopyField::new((id.clone(), "addr"), a)),
                )
            });
        row::frame()
            .child(
                // `<I.Network_3 size={20} />`, `.g-tunnel > svg { fill: var(--icon-secondary) }`.
                div().flex().flex_none().child(
                    Icon::carbon(CarbonIcon::Network3)
                        .size(IconSize::S20)
                        .color(color(ColorRole::IconSecondary)),
                ),
            )
            .child(main)
            .child(row::tail(button))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_states_map_to_dots() {
        assert_eq!(MacState::Online.dot(), ConnectionStatus::Connected);
        assert_eq!(MacState::TurnedAway.dot(), ConnectionStatus::Error);
        assert_eq!(MacState::Offline.dot(), ConnectionStatus::Offline);
        let t = MacRowTexts::default();
        assert_eq!(
            (t.state)(MacState::Reaching),
            SharedString::from("Reaching…")
        );
        assert_eq!((t.latency)(12), SharedString::from("12 ms"));
    }

    #[test]
    fn tunnel_states_map_to_dot_and_button() {
        assert_eq!(TunnelState::Starting.dot(), ConnectionStatus::Connecting);
        assert_eq!(TunnelState::Error.dot(), ConnectionStatus::Error);
        assert_eq!(TunnelButton::of(TunnelState::Running), TunnelButton::Stop);
        assert_eq!(
            TunnelButton::of(TunnelState::Missing),
            TunnelButton::Install
        );
        assert_eq!(
            TunnelButton::of(TunnelState::Starting),
            TunnelButton::Start { disabled: true }
        );
        assert_eq!(
            TunnelButton::of(TunnelState::Error),
            TunnelButton::Start { disabled: false }
        );
        let t = TunnelRowTexts::default();
        assert_eq!(
            (t.state)(TunnelState::Missing),
            SharedString::from("cloudflared not installed")
        );
    }
}
