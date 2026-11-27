//! Servers (Macs) in the Ghost sidebar (`workspace.jsx` "Shell"):
//! [`ServerChip`], the small tag that says which Mac a thread, palette
//! result or setting lives on, and [`ServerHeader`], the sidebar group title
//! per server.
//!
//! Both say a server's state with a dot (or spinner) **and** words, never
//! color alone (`SERVER_STATE`: Online, Offline, Turned away, Reaching…).
//!
//! ```ignore
//! ServerChip::new("chip", "build-box").kind(MachineKind::Server).state(ServerState::Offline)
//! ServerHeader::new("head", "build-box")
//!     .kind(MachineKind::Server)
//!     .state(ServerState::Offline)
//!     .reason("Not reachable since 10:42. It may be asleep.")
//!     .count(3)
//!     .on_reach(|_, _| {})
//! ```
use crate::{
    components::button::{Button, ButtonSize, ButtonVariant},
    density,
    icons::{CarbonIcon, Icon},
    t3::{
        Spin,
        base::visually_hidden,
        craft::{focus_handle, focus_ring, focus_visible, press, rem, sans, sized},
        shell::{ConnectionStatus, ConnectionStatusDot, MachineKind},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, radians,
};
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

/// A click handler without the event.
type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A plain hover tooltip standing in for the HTML `title` attribute.
fn title_tooltip(text: SharedString) -> impl Fn(&mut Window, &mut App) -> gpui_kit::AnyView {
    move |window, cx| gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
}

/// A server's state (`state`: `online | offline | turned-away | reaching`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ServerState {
    /// `online` (the default).
    #[default]
    Online,
    /// `offline`: not reachable.
    Offline,
    /// `turned-away`: the server refused this Mac (pairing revoked).
    TurnedAway,
    /// `reaching`: connecting.
    Reaching,
}

impl ServerState {
    /// Every state, in `index.d.ts` order.
    pub const ALL: [ServerState; 4] = [
        Self::Online,
        Self::Offline,
        Self::TurnedAway,
        Self::Reaching,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::TurnedAway => "turned-away",
            Self::Reaching => "reaching",
        }
    }

    /// The reference's words for the state (`SERVER_STATE[state][0]`), the
    /// default label; callers pass their own with `state_label`.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Offline => "Offline",
            Self::TurnedAway => "Turned away",
            Self::Reaching => "Reaching…",
        }
    }

    /// The dot it shows (`SERVER_STATE[state][1]`): connected, offline,
    /// error, connecting.
    pub const fn connection(self) -> ConnectionStatus {
        match self {
            Self::Online => ConnectionStatus::Connected,
            Self::Offline => ConnectionStatus::Offline,
            Self::TurnedAway => ConnectionStatus::Error,
            Self::Reaching => ConnectionStatus::Connecting,
        }
    }

    /// Whether the header explains itself with a reason line and "Reach
    /// again" (`state !== 'online' && state !== 'reaching'`).
    pub const fn has_reason(self) -> bool {
        matches!(self, Self::Offline | Self::TurnedAway)
    }
}

// ── ServerChip ──────────────────────────────────────────────────────────

/// `ServerChip` `size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ServerChipSize {
    /// `md` (the default): 20px tall.
    #[default]
    Md,
    /// `sm`: 16px tall, for thread rows.
    Sm,
}

impl ServerChipSize {
    /// Both sizes.
    pub const ALL: [ServerChipSize; 2] = [Self::Md, Self::Sm];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Md => "md",
            Self::Sm => "sm",
        }
    }

    /// (height, horizontal padding, font size, glyph size) in px:
    /// `.g-server { height: 1.25rem; padding: 0 var(--spacing-03); font-size:
    /// 0.75rem }`, `.g-server--sm { height: 1rem; padding: 0 var(--spacing-02);
    /// font-size: 0.6875rem }`; the machine glyph is 14 / 12
    /// (`size={size === 'sm' ? 12 : 14}`).
    pub fn geometry(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Md => (20., tokens::SPACING_03.px_value(), 12., 14.),
            Self::Sm => (16., tokens::SPACING_02.px_value(), 11., 12.),
        }
    }
}

/// A server chip's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerChipLook {
    /// `.g-server { background: var(--layer-accent-01) }`;
    /// `button.g-server:hover { background: var(--layer-accent-hover-01) }`.
    pub background: Hsla,
    /// Text and glyph (`.g-server svg { fill: currentColor }`):
    /// `text-secondary`, `text-helper` when offline or turned away
    /// (`.g-server--offline, .g-server--turned-away`).
    pub text: Hsla,
}

impl ServerChipLook {
    /// The look of a chip in `state`; only a clickable chip hovers.
    pub fn of(state: ServerState, clickable: bool, hovered: bool) -> Self {
        let background = if clickable && hovered {
            color(ColorRole::LayerAccentHover01)
        } else {
            color(ColorRole::LayerAccent01)
        };
        let text = match state {
            ServerState::Offline | ServerState::TurnedAway => color(ColorRole::TextHelper),
            _ => color(ColorRole::TextSecondary),
        };
        Self { background, text }
    }
}

/// `.g-server { max-width: 10rem }`.
pub const CHIP_MAX_WIDTH: f32 = 160.;

/// A small server tag (`ServerChip`, `.g-server`): a pill holding the
/// machine glyph, the name (ellipsized past 10rem) and the state dot, titled
/// `name · state`. With [`ServerChip::on_click`] it is a button (hover
/// `layer-accent-hover-01`, focus ring); otherwise a plain tag.
#[derive(IntoElement)]
pub struct ServerChip {
    id: ElementId,
    name: SharedString,
    kind: MachineKind,
    state: ServerState,
    size: ServerChipSize,
    state_label: Option<SharedString>,
    on_click: Option<Handler>,
}

impl ServerChip {
    /// A chip for server `name` (`name`).
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: MachineKind::Laptop,
            state: ServerState::Online,
            size: ServerChipSize::Md,
            state_label: None,
            on_click: None,
        }
    }

    /// The machine (`kind`, default laptop).
    pub fn kind(mut self, kind: MachineKind) -> Self {
        self.kind = kind;
        self
    }

    /// The state (`state`, default online).
    pub fn state(mut self, state: ServerState) -> Self {
        self.state = state;
        self
    }

    /// The size (`size`, default md).
    pub fn size(mut self, size: ServerChipSize) -> Self {
        self.size = size;
        self
    }

    /// The words for the state (default [`ServerState::label`]).
    pub fn state_label(mut self, label: impl Into<SharedString>) -> Self {
        self.state_label = Some(label.into());
        self
    }

    /// `onClick`: given, the chip renders as a button.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ServerChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let (_, _, _, glyph) = self.size.geometry();
        let label = self
            .state_label
            .unwrap_or_else(|| self.state.label().into());
        let clickable = self.on_click.is_some();
        let rest = ServerChipLook::of(self.state, clickable, false);
        let hover = ServerChipLook::of(self.state, clickable, true);
        let title: SharedString = format!("{} · {}", self.name, label).into();
        let parts = vec![
            self.kind
                .icon()
                .size_length(rem(glyph))
                .color(rest.text)
                .flex_none()
                .into_any_element(),
            // `.g-server-name { overflow: hidden; white-space: nowrap; text-overflow: ellipsis }`.
            div()
                .min_w_0()
                .truncate()
                .child(self.name.clone())
                .into_any_element(),
            ConnectionStatusDot::new(self.state.connection())
                .id((id.clone(), "dot"))
                .label(label.clone())
                .into_any_element(),
            visually_hidden(label.clone()).into_any_element(),
        ];
        fn pill<E: Styled + ParentElement>(el: E, size: ServerChipSize, look: ServerChipLook) -> E {
            let (height, pad, font, _) = size.geometry();
            // `.g-server { font: inherit }` sets no weight: semibold inside an
            // unread `.g-trow2--unread` row.
            sized(el, font)
                .flex()
                .flex_none()
                .items_center()
                // `.g-server { gap: var(--spacing-02) }`.
                .gap(tokens::SPACING_02)
                .min_w_0()
                .h(rem(height))
                .px(rem(pad))
                // `border-radius: var(--border-radius-16)`.
                .rounded(tokens::BORDER_RADIUS_16.rems())
                .bg(look.background)
                .text_color(look.text)
        }
        let body = match self.on_click {
            Some(on_click) => {
                let focus = focus_handle(&id, window, cx);
                let focused = focus_visible(&focus, window);
                pill(press((id.clone(), "btn"), &focus), self.size, rest)
                    .hover(move |s| s.bg(hover.background))
                    .on_click(move |_, w, cx| on_click(w, cx))
                    .children(parts)
                    // `button` focus: the 2px `focus` ring, following the pill.
                    .when(focused, |b| {
                        b.child(
                            div()
                                .absolute()
                                .inset_0()
                                .rounded(tokens::BORDER_RADIUS_16.rems())
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    })
                    .into_any_element()
            }
            None => pill(div(), self.size, rest)
                .children(parts)
                .into_any_element(),
        };
        // `.g-server { max-width: 10rem; flex: none }`, titled `name · state`.
        div()
            .id(id)
            .flex()
            .flex_none()
            .max_w(rem(CHIP_MAX_WIDTH))
            .tooltip(title_tooltip(title))
            .child(body)
    }
}

// ── ServerHeader ────────────────────────────────────────────────────────

/// The reason line's start rule: `support-warning` when turned away,
/// `border-strong-01` when offline (`.g-shead-reason { border-left: 2px solid
/// var(--support-warning) }`, `.g-shead--offline .g-shead-reason {
/// border-left-color: var(--border-strong-01) }`).
pub fn reason_rule(state: ServerState) -> Hsla {
    match state {
        ServerState::Offline => color(ColorRole::BorderStrong01),
        _ => color(ColorRole::SupportWarning),
    }
}

/// A server's group title in the sidebar (`ServerHeader`, `.g-shead`): a
/// disclosure button (chevron, machine glyph, semibold name, count pill) and
/// the state (dot or spinner, words, the version when online). Offline and
/// turned-away servers add a reason line with a "Reach again" button.
#[derive(IntoElement)]
pub struct ServerHeader {
    id: ElementId,
    name: SharedString,
    kind: MachineKind,
    state: ServerState,
    state_label: Option<SharedString>,
    reason: Option<SharedString>,
    version: Option<SharedString>,
    count: Option<usize>,
    expanded: bool,
    reach_label: SharedString,
    on_toggle: Option<Handler>,
    on_reach: Option<Handler>,
}

impl ServerHeader {
    /// The header for server `name` (`name`).
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: MachineKind::Laptop,
            state: ServerState::Online,
            state_label: None,
            reason: None,
            version: None,
            count: None,
            expanded: true,
            reach_label: "Reach again".into(),
            on_toggle: None,
            on_reach: None,
        }
    }

    /// The machine (`kind`, default laptop).
    pub fn kind(mut self, kind: MachineKind) -> Self {
        self.kind = kind;
        self
    }

    /// The state (`state`, default online).
    pub fn state(mut self, state: ServerState) -> Self {
        self.state = state;
        self
    }

    /// The words for the state (default [`ServerState::label`]).
    pub fn state_label(mut self, label: impl Into<SharedString>) -> Self {
        self.state_label = Some(label.into());
        self
    }

    /// Why it is offline or turned away (`reason`).
    pub fn reason(mut self, reason: impl Into<SharedString>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Ghost's version on it, shown while online (`version`).
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// The thread count pill (`count`).
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Whether the group is open (`expanded`, default true).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// The reach button's label (default `"Reach again"`).
    pub fn reach_label(mut self, label: impl Into<SharedString>) -> Self {
        self.reach_label = label.into();
        self
    }

    /// `onToggle`.
    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// `onReach`.
    pub fn on_reach(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reach = Some(Rc::new(f));
        self
    }
}

/// The disclosure chevron's turn: `.t3-rot { transform: rotate(-90deg) }`
/// when collapsed. `.g-shead-chev` has no transition, so it snaps.
pub fn header_chevron_angle(expanded: bool) -> f32 {
    if expanded { 0. } else { -FRAC_PI_2 }
}

/// The name as drawn: `.g-shead-main { text-transform: uppercase }`.
pub fn shead_name(name: &str) -> String {
    name.to_uppercase()
}

impl RenderOnce for ServerHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let focus = focus_handle(&id, window, cx);
        let focused = focus_visible(&focus, window);
        let turn = header_chevron_angle(self.expanded);
        let label = self
            .state_label
            .unwrap_or_else(|| self.state.label().into());
        let secondary = color(ColorRole::TextSecondary);
        let glyph = color(ColorRole::IconSecondary);
        // `.g-shead-main { display: flex; gap: var(--spacing-03); min-width: 0;
        // height: var(--density-row-md); padding: 0 var(--spacing-03); color:
        // var(--text-primary); font-size: 0.75rem; font-weight: 600 }`;
        // `.g-shead-main svg { fill: var(--icon-secondary) }`;
        // `.g-shead-main:hover { background: var(--layer-hover-01) }`; then
        // `.g-shead-main { text-transform: uppercase; letter-spacing: 0.32px;
        // color: var(--text-secondary) }` (`src/t3.css`, later in cascade
        // order). GPUI has no tracking control, so the 0.32px is not applied.
        let main = sans(press((id.clone(), "main"), &focus), 12., 600)
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(tokens::SPACING_03)
            .h(density::current().row().rems())
            .px(tokens::SPACING_03)
            .text_color(secondary)
            .hover(|s| s.bg(color(ColorRole::LayerHover01)))
            .when_some(self.on_toggle, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(
                Icon::carbon(CarbonIcon::ChevronDown)
                    .size_length(rem(14.))
                    .color(glyph)
                    .rotate(radians(turn))
                    .flex_none(),
            )
            .child(
                self.kind
                    .icon()
                    .size_length(rem(16.))
                    .color(glyph)
                    .flex_none(),
            )
            // `.g-shead-name { overflow: hidden; white-space: nowrap; text-overflow: ellipsis }`.
            .child(div().min_w_0().truncate().child(shead_name(&self.name)))
            .children(self.count.map(|n| {
                // `.t3-shelf-count { padding: 0 var(--spacing-02); background:
                // var(--layer-accent-01); color: var(--text-secondary);
                // font-size: 0.6875rem; line-height: 1rem; border-radius: 16 }`,
                // no weight of its own: `.g-shead-main`'s 600.
                sized(div(), 11.)
                    .flex_none()
                    .line_height(rem(16.))
                    .px(tokens::SPACING_02)
                    .rounded(tokens::BORDER_RADIUS_16.rems())
                    .bg(color(ColorRole::LayerAccent01))
                    .text_color(secondary)
                    .child(n.to_string())
            }))
            .when(focused, |b| b.child(focus_ring()));
        let indicator = if self.state == ServerState::Reaching {
            Spin::new((id.clone(), "spin")).size(12.).into_any_element()
        } else {
            ConnectionStatusDot::new(self.state.connection())
                .id((id.clone(), "dot"))
                .label(label.clone())
                .into_any_element()
        };
        // `.g-shead-state { display: inline-flex; gap: var(--spacing-02);
        // font-size: 0.75rem; color: var(--text-secondary) }`.
        let state = sans(div(), 12., 400)
            .flex()
            .flex_none()
            .items_center()
            .gap(tokens::SPACING_02)
            .text_color(secondary)
            .child(indicator)
            .child(label)
            .when_some(
                self.version.filter(|_| self.state == ServerState::Online),
                // `<span className="g-dim"> · {version}</span>`.
                |d, v| d.child(div().text_color(secondary).child(format!(" · {v}"))),
            );
        let reason = self.state.has_reason().then(|| {
            let on_reach = self.on_reach;
            // `.g-shead-reason { display: flex; gap: var(--spacing-03); margin: 0
            // 0 var(--spacing-03) var(--spacing-07); padding: var(--spacing-02)
            // var(--spacing-03); background: var(--layer-01); font-size: 0.75rem;
            // color: var(--text-secondary); border-left: 2px solid … }`.
            sans(div(), 12., 400)
                .flex()
                .items_center()
                .gap(tokens::SPACING_03)
                .ml(tokens::SPACING_07)
                .mb(tokens::SPACING_03)
                .py(tokens::SPACING_02)
                .px(tokens::SPACING_03)
                .bg(color(ColorRole::Layer01))
                .text_color(secondary)
                .border_l(px(2.))
                .border_color(reason_rule(self.state))
                // `.g-shead-reason span { flex: 1 }`, then `{ min-width: 0;
                // overflow: hidden; white-space: nowrap; text-overflow:
                // ellipsis }` (`src/t3.css`): one line, cut with an ellipsis.
                .child(div().flex_1().min_w_0().truncate().children(self.reason))
                .child(
                    Button::new((id.clone(), "reach"), self.reach_label)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .when_some(on_reach, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                )
        });
        // `.g-shead { display: grid; grid-template-columns: 1fr auto; padding: 0
        // var(--spacing-03) 0 0 }`; the reason spans both columns.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .pr(tokens::SPACING_03)
                    .child(main)
                    .child(state),
            )
            .children(reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn states_say_words_and_dots_like_server_state() {
        let names: Vec<_> = ServerState::ALL.iter().map(|s| s.name()).collect();
        assert_eq!(names, ["online", "offline", "turned-away", "reaching"]);
        assert_eq!(ServerState::TurnedAway.label(), "Turned away");
        assert_eq!(ServerState::Reaching.label(), "Reaching…");
        assert_eq!(
            ServerState::ALL.map(|s| s.connection()),
            [
                ConnectionStatus::Connected,
                ConnectionStatus::Offline,
                ConnectionStatus::Error,
                ConnectionStatus::Connecting,
            ]
        );
        assert_eq!(
            ServerState::ALL.map(|s| s.has_reason()),
            [false, true, true, false]
        );
    }

    #[test]
    fn chip_geometry_follows_g_server() {
        assert_eq!(ServerChipSize::Md.geometry(), (20., 8., 12., 14.));
        assert_eq!(ServerChipSize::Sm.geometry(), (16., 4., 11., 12.));
        assert_eq!(CHIP_MAX_WIDTH, 160.);
        assert_eq!(ServerChipSize::Sm.name(), "sm");
    }

    #[test]
    fn chip_look_follows_g_server_states() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = ServerChipLook::of(ServerState::Online, true, false);
        assert_eq!(hex(rest.background), hex(color(ColorRole::LayerAccent01)));
        assert_eq!(hex(rest.text), hex(color(ColorRole::TextSecondary)));
        // Only a button hovers.
        let tag = ServerChipLook::of(ServerState::Online, false, true);
        assert_eq!(tag.background, rest.background);
        let hover = ServerChipLook::of(ServerState::Online, true, true);
        assert_eq!(
            hex(hover.background),
            hex(color(ColorRole::LayerAccentHover01))
        );
        for s in [ServerState::Offline, ServerState::TurnedAway] {
            assert_eq!(
                hex(ServerChipLook::of(s, false, false).text),
                hex(color(ColorRole::TextHelper))
            );
        }
        assert_eq!(
            hex(ServerChipLook::of(ServerState::Reaching, false, false).text),
            hex(color(ColorRole::TextSecondary))
        );
    }

    #[test]
    fn reason_rule_is_warning_unless_offline() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(
            hex(reason_rule(ServerState::TurnedAway)),
            hex(color(ColorRole::SupportWarning))
        );
        assert_eq!(
            hex(reason_rule(ServerState::Offline)),
            hex(color(ColorRole::BorderStrong01))
        );
    }

    #[test]
    fn header_defaults_follow_the_jsx() {
        let h = ServerHeader::new("h", "mini");
        assert_eq!(
            (h.kind, h.state, h.expanded, h.reach_label.as_ref()),
            (
                MachineKind::Laptop,
                ServerState::Online,
                true,
                "Reach again"
            )
        );
    }

    #[test]
    fn header_name_is_uppercase_and_the_chevron_snaps() {
        assert_eq!(shead_name("MacBook Pro"), "MACBOOK PRO");
        assert_eq!(shead_name("build-box"), "BUILD-BOX");
        assert_eq!(header_chevron_angle(true), 0.);
        assert_eq!(header_chevron_angle(false), -FRAC_PI_2);
    }
}
