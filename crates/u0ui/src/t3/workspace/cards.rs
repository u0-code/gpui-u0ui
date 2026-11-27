//! T3 · Settings: [`ProviderInstanceCard`], [`EnvironmentRow`],
//! [`WizardSteps`], [`Wizard`], [`StatBlock`] and [`StatsGrid`]
//! (`src/t3/workspace.jsx`, "Settings" in `src/t3.css`).
use super::parts::{card_sub, child_id, icon_button};
use crate::{
    components::{Badge, BadgeSize, BadgeVariant, Button, ButtonSize, ButtonVariant},
    icons::CarbonIcon,
    t3::{
        craft::{BODY_LINE, mono, sans},
        press::{self, Handler, rem},
        shell::{
            ConnectionStatus, ConnectionStatusDot, EnvironmentMachineIcon, IdentitySize,
            MachineKind, ProviderIcon,
        },
    },
    theme::{color, color_in},
    tokens::{CarbonTheme, ColorRole},
};
use gpui_kit::{
    AnyElement, App, Bounds, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Styled, Window, canvas, div, prelude::FluentBuilder as _, px,
    relative,
};
use std::rc::Rc;

// ── ProviderInstanceCard ────────────────────────────────────────────────

/// `ProviderInstanceCard` `status`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProviderStatus {
    /// `ready` (the default): green Ready.
    #[default]
    Ready,
    /// `signin`: Sign in required, with the sign-in diagnostic.
    Signin,
    /// `error`: Not found, with the PATH diagnostic.
    Error,
    /// `updating`: blue Update available.
    Updating,
}

impl ProviderStatus {
    /// Every status.
    pub const ALL: [ProviderStatus; 4] = [Self::Ready, Self::Signin, Self::Error, Self::Updating];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Signin => "signin",
            Self::Error => "error",
            Self::Updating => "updating",
        }
    }

    /// The default tag label (`st[status][0]`).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Signin => "Sign in required",
            Self::Error => "Not found",
            Self::Updating => "Update available",
        }
    }

    /// The tag variant (`st[status][1]`).
    pub const fn variant(self) -> BadgeVariant {
        match self {
            Self::Ready => BadgeVariant::Green,
            Self::Signin => BadgeVariant::WarmGray,
            Self::Error => BadgeVariant::Red,
            Self::Updating => BadgeVariant::Blue,
        }
    }

    /// A custom tag pair, where `t3.css` repaints the Carbon one:
    /// `.t3-provcard .cds--tag--warm-gray { background:
    /// var(--tag-background-yellow, #fcf4d6); color: var(--text-primary) }`.
    /// There is no `tag-background-yellow` token, so the `#fcf4d6`
    /// fallback (Yellow 10, White's `notification-background-warning`)
    /// paints in every theme. Its ink is White's `text-primary` (`#161616`)
    /// in every theme too: the dark themes' white `text-primary` would
    /// vanish on that yellow.
    pub fn colors(self) -> Option<(Hsla, Hsla)> {
        (self == Self::Signin).then(|| {
            (
                color_in(CarbonTheme::White, ColorRole::NotificationBackgroundWarning),
                crate::t3::base::ink(),
            )
        })
    }
}

/// The subtitle (`{provider !== name ? provider : ''}{… && version ? ' · ' : ''}{version ? `v${version}` : ''}`).
pub fn provider_subtitle(name: &str, provider: &str, version: Option<&str>) -> String {
    let mut s = String::new();
    if provider != name {
        s.push_str(provider);
        if version.is_some() {
            s.push_str(" \u{b7} ");
        }
    }
    if let Some(v) = version {
        s.push('v');
        s.push_str(v);
    }
    s
}

/// Texts of a [`ProviderInstanceCard`] (hard-coded in the reference).
#[derive(Clone)]
pub struct ProviderCardTexts {
    /// The status tag label (default [`ProviderStatus::label`]).
    pub status: Option<SharedString>,
    /// `More`.
    pub more: SharedString,
    /// `Authenticated as`.
    pub account: SharedString,
    /// `Models`.
    pub models: SharedString,
    /// `{n} available`.
    pub available: Rc<dyn Fn(usize) -> SharedString>,
    /// `Endpoint`.
    pub endpoint: SharedString,
    /// Before and after the login command (`Sign in by running `, `.`).
    pub sign_in_run: (SharedString, SharedString),
    /// Without a login command (`Sign in with the agent’s login command.`).
    pub sign_in: SharedString,
    /// `Run in terminal`.
    pub run: SharedString,
    /// `I’ve signed in`.
    pub signed_in: SharedString,
    /// `The CLI was not found on PATH.`
    pub not_found: SharedString,
    /// `Set path…`.
    pub set_path: SharedString,
}

impl Default for ProviderCardTexts {
    fn default() -> Self {
        Self {
            status: None,
            more: "More".into(),
            account: "Authenticated as".into(),
            models: "Models".into(),
            available: Rc::new(|n| format!("{n} available").into()),
            endpoint: "Endpoint".into(),
            sign_in_run: ("Sign in by running ".into(), ".".into()),
            sign_in: "Sign in with the agent\u{2019}s login command.".into(),
            run: "Run in terminal".into(),
            signed_in: "I\u{2019}ve signed in".into(),
            not_found: "The CLI was not found on PATH.".into(),
            set_path: "Set path\u{2026}".into(),
        }
    }
}

/// An agent provider instance (`ProviderInstanceCard`, `.t3-provcard`):
/// provider mark, name and `provider · vX`, a status tag and More; the
/// account, model count and endpoint as a key/value list; a sign-in or
/// not-found diagnostic strip with its next steps. `layer-01`.
#[derive(IntoElement)]
pub struct ProviderInstanceCard {
    id: ElementId,
    name: SharedString,
    provider: SharedString,
    status: ProviderStatus,
    account: Option<SharedString>,
    version: Option<SharedString>,
    models: usize,
    env: Option<SharedString>,
    login_command: Option<SharedString>,
    primary: bool,
    texts: ProviderCardTexts,
    on_more: Option<Handler>,
    on_run: Option<Handler>,
    on_signed_in: Option<Handler>,
    on_set_path: Option<Handler>,
}

impl ProviderInstanceCard {
    /// The card for instance `name` of `provider`.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        provider: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
            status: ProviderStatus::Ready,
            account: None,
            version: None,
            models: 0,
            env: None,
            login_command: None,
            primary: false,
            texts: ProviderCardTexts::default(),
            on_more: None,
            on_run: None,
            on_signed_in: None,
            on_set_path: None,
        }
    }

    /// The status (`status`).
    pub fn status(mut self, status: ProviderStatus) -> Self {
        self.status = status;
        self
    }

    /// The signed-in account (`account`).
    pub fn account(mut self, account: impl Into<SharedString>) -> Self {
        self.account = Some(account.into());
        self
    }

    /// The CLI version (`version`).
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Available models (`models`).
    pub fn models(mut self, models: usize) -> Self {
        self.models = models;
        self
    }

    /// The endpoint / environment (`env`).
    pub fn env(mut self, env: impl Into<SharedString>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// The login command named in the sign-in strip (`loginCommand`).
    pub fn login_command(mut self, command: impl Into<SharedString>) -> Self {
        self.login_command = Some(command.into());
        self
    }

    /// Run in terminal is the primary button (`primary`; default tertiary).
    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: ProviderCardTexts) -> Self {
        self.texts = texts;
        self
    }

    /// More pressed.
    pub fn on_more(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_more = Some(Rc::new(f));
        self
    }

    /// Run in terminal pressed.
    pub fn on_run(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_run = Some(Rc::new(f));
        self
    }

    /// I've signed in pressed.
    pub fn on_signed_in(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_signed_in = Some(Rc::new(f));
        self
    }

    /// Set path pressed.
    pub fn on_set_path(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_set_path = Some(Rc::new(f));
        self
    }
}

fn click(b: Button, f: Option<Handler>) -> Button {
    match f {
        Some(f) => b.on_click(move |_, w, cx| f(w, cx)),
        None => b,
    }
}

/// `.t3-kv { display: grid; grid-template-columns: max-content 1fr;
/// gap: spacing-02 spacing-05; font-size: 0.875rem }`, `dt { color:
/// text-secondary }`, `dd { min-width: 0; overflow: hidden; text-overflow:
/// ellipsis }`. Single-line rows, so two stacked columns line up as the grid.
fn key_values(pairs: Vec<(SharedString, SharedString)>) -> impl IntoElement {
    let column = || div().flex().flex_col().gap(rem(4.));
    sans(div(), 14., 400)
        .flex()
        .flex_row()
        .gap(rem(16.))
        .min_w_0()
        .child(column().flex_none().children(pairs.iter().map(|(k, _)| {
            div()
                .text_color(color(ColorRole::TextSecondary))
                .child(k.clone())
        })))
        .child(
            column()
                .flex_1()
                .min_w_0()
                .children(pairs.iter().map(|(_, v)| {
                    div()
                        .min_w_0()
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .text_color(color(ColorRole::TextPrimary))
                        .child(v.clone())
                })),
        )
}

impl RenderOnce for ProviderInstanceCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let texts = self.texts.clone();
        let status = self.status;
        let subtitle = provider_subtitle(&self.name, &self.provider, self.version.as_deref());
        let tag = {
            let label = texts
                .status
                .clone()
                .unwrap_or_else(|| status.label().into());
            let badge = Badge::new(label)
                .variant(status.variant())
                .size(BadgeSize::Sm);
            match status.colors() {
                Some((bg, fg)) => badge.colors(bg, fg),
                None => badge,
            }
        };
        // `.t3-provcard-head { display: flex; align-items: center; gap: spacing-04 }`.
        let head = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(12.))
            .child(ProviderIcon::new(self.provider.clone()).size(IdentitySize::Lg))
            .child(
                // `.t3-provcard-title { display: flex; flex: 1; flex-direction: column; min-width: 0 }`,
                // `b { font-size: 1rem; font-weight: 600 }`.
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        sans(div(), 16., 600)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(self.name.clone()),
                    )
                    // An empty subtitle span takes no height.
                    .when(!subtitle.is_empty(), |this| this.child(card_sub(subtitle))),
            )
            .child(tag)
            .child(click(
                icon_button(
                    (id.clone(), "more"),
                    texts.more.clone(),
                    CarbonIcon::OverflowMenuVertical,
                ),
                self.on_more.clone(),
            ));
        let mut pairs = Vec::new();
        if let Some(a) = self.account.clone() {
            pairs.push((texts.account.clone(), a));
        }
        pairs.push((texts.models.clone(), (texts.available)(self.models)));
        if let Some(e) = self.env.clone() {
            pairs.push((texts.endpoint.clone(), e));
        }
        // `.t3-provcard-diag { display: flex; flex-wrap: wrap; align-items: center;
        // gap: spacing-03; padding: spacing-04; background: notification-background-warning;
        // font-size: 0.875rem }`, `> svg { fill: support-warning }`, `> span { flex: 1 }`;
        // `--err { background: notification-background-error }`, `> svg { fill: support-error }`.
        let strip = |bg: ColorRole| {
            sans(div(), 14., 400)
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(rem(8.))
                .p(rem(12.))
                .bg(color(bg))
                .text_color(color(ColorRole::TextPrimary))
        };
        let diagnostic = match status {
            ProviderStatus::Signin => {
                let message = match self.login_command.clone() {
                    Some(cmd) => div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .items_baseline()
                        .child(texts.sign_in_run.0.clone())
                        // `.t3-provcard-diag code { white-space: nowrap }`.
                        .child(mono(div(), 14., 400).whitespace_nowrap().child(cmd))
                        .child(texts.sign_in_run.1.clone()),
                    None => div().child(texts.sign_in.clone()),
                };
                Some(
                    strip(ColorRole::NotificationBackgroundWarning)
                        .child(crate::t3::base::warning_glyph(16.))
                        .child(message.flex_1().min_w_0())
                        .child(
                            // `.t3-provcard-diag-acts { display: inline-flex; gap: spacing-03;
                            // margin-left: auto; flex: none }`.
                            div()
                                .flex()
                                .flex_row()
                                .flex_none()
                                .ml_auto()
                                .gap(rem(8.))
                                .child(click(
                                    Button::new((id.clone(), "run"), texts.run.clone())
                                        .size(ButtonSize::Sm)
                                        .variant(if self.primary {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Tertiary
                                        })
                                        .balanced(true),
                                    self.on_run.clone(),
                                ))
                                .child(click(
                                    Button::new((id.clone(), "signed-in"), texts.signed_in.clone())
                                        .size(ButtonSize::Sm)
                                        .variant(ButtonVariant::Ghost),
                                    self.on_signed_in.clone(),
                                )),
                        ),
                )
            }
            ProviderStatus::Error => Some(
                strip(ColorRole::NotificationBackgroundError)
                    .child(press::icon(
                        CarbonIcon::ErrorFilled,
                        16.,
                        color(ColorRole::SupportError),
                    ))
                    .child(div().flex_1().min_w_0().child(texts.not_found.clone()))
                    .child(click(
                        Button::new((id.clone(), "set-path"), texts.set_path.clone())
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost),
                        self.on_set_path.clone(),
                    )),
            ),
            _ => None,
        };
        // `.t3-provcard { display: flex; flex-direction: column; gap: spacing-04;
        // max-width: none; padding: spacing-05; background: layer-01 }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .gap(rem(12.))
            .p(rem(16.))
            .bg(color(ColorRole::Layer01))
            .child(head)
            .child(key_values(pairs))
            .children(diagnostic)
    }
}

// ── EnvironmentRow ──────────────────────────────────────────────────────

/// Texts of an [`EnvironmentRow`].
#[derive(Clone, Debug)]
pub struct EnvironmentRowTexts {
    /// The current-machine badge (`This computer`).
    pub current: SharedString,
    /// The default action (`Options`).
    pub options: SharedString,
}

impl Default for EnvironmentRowTexts {
    fn default() -> Self {
        Self {
            current: "This computer".into(),
            options: "Options".into(),
        }
    }
}

/// A machine (`EnvironmentRow`, `.t3-envrow`): a 20px machine glyph with
/// its connection dot cut into the corner, the name (and This computer),
/// `transport · detail`, and actions (default an Options button). 64px on
/// `layer-01` over a bottom hairline.
#[derive(IntoElement)]
pub struct EnvironmentRow {
    id: ElementId,
    name: SharedString,
    kind: MachineKind,
    transport: SharedString,
    status: ConnectionStatus,
    detail: Option<SharedString>,
    current: bool,
    actions: Option<AnyElement>,
    texts: EnvironmentRowTexts,
    on_options: Option<Handler>,
}

impl EnvironmentRow {
    /// A row for the machine `name`.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: MachineKind::Laptop,
            transport: "Local".into(),
            status: ConnectionStatus::Connected,
            detail: None,
            current: false,
            actions: None,
            texts: EnvironmentRowTexts::default(),
            on_options: None,
        }
    }

    /// The machine kind (`kind`).
    pub fn kind(mut self, kind: MachineKind) -> Self {
        self.kind = kind;
        self
    }

    /// How it connects (`transport`, default `Local`).
    pub fn transport(mut self, transport: impl Into<SharedString>) -> Self {
        self.transport = transport.into();
        self
    }

    /// The connection status (`status`).
    pub fn status(mut self, status: ConnectionStatus) -> Self {
        self.status = status;
        self
    }

    /// The detail after the transport (`detail`).
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// This computer (`current`).
    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Replace the Options button (`actions`).
    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: EnvironmentRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// The default Options button pressed.
    pub fn on_options(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_options = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for EnvironmentRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let sub = match &self.detail {
            Some(d) => format!("{} \u{b7} {d}", self.transport),
            None => self.transport.to_string(),
        };
        let actions = self.actions.unwrap_or_else(|| {
            click(
                icon_button(
                    (id.clone(), "options"),
                    self.texts.options.clone(),
                    CarbonIcon::OverflowMenuVertical,
                ),
                self.on_options.clone(),
            )
            .into_any_element()
        });
        let surface = color(ColorRole::Layer01);
        // `.t3-envrow { display: flex; align-items: center; gap: spacing-04; min-height: size-xl;
        // padding: spacing-03 spacing-02 spacing-03 spacing-05; border-bottom: 1px solid
        // border-subtle-01; background: layer-01 }`.
        div()
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(12.))
            .min_h(rem(64.))
            .py(rem(8.))
            .pl(rem(16.))
            .pr(rem(4.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .bg(surface)
            .child(
                // `.t3-envrow-icon { position: relative; display: flex }`, `.t3-dot { position:
                // absolute; right: -3px; bottom: -2px; box-shadow: 0 0 0 2px layer-01 }`.
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .child(EnvironmentMachineIcon::new(self.kind).size(20.))
                    .child(
                        div().absolute().right(px(-3.)).bottom(px(-2.)).child(
                            ConnectionStatusDot::new(self.status)
                                .id((id.clone(), "dot"))
                                .cutout(surface),
                        ),
                    ),
            )
            .child(
                // `.t3-envrow-text { display: flex; flex: 1; flex-direction: column; gap: 2px }`,
                // `b { display: flex; gap: spacing-03; align-items: center; 14px 600 }`.
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        sans(div(), 14., 600)
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(rem(8.))
                            .text_color(color(ColorRole::TextPrimary))
                            .child(self.name.clone())
                            .when(self.current, |this| {
                                this.child(
                                    Badge::new(self.texts.current.clone())
                                        .variant(BadgeVariant::Blue)
                                        .size(BadgeSize::Sm),
                                )
                            }),
                    )
                    .child(card_sub(sub)),
            )
            .child(actions)
    }
}

// ── Wizard ──────────────────────────────────────────────────────────────

/// A step's place relative to the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    /// Before the current step: a green check.
    Done,
    /// The current step: a blue number, semibold.
    Current,
    /// After it: a neutral number.
    Upcoming,
}

impl StepState {
    /// Step `i` while `current` is shown.
    pub fn of(i: usize, current: usize) -> Self {
        match i.cmp(&current) {
            std::cmp::Ordering::Less => Self::Done,
            std::cmp::Ordering::Equal => Self::Current,
            std::cmp::Ordering::Greater => Self::Upcoming,
        }
    }

    /// (bullet fill, bullet ink): `.t3-guide-bullet { background: layer-accent-01;
    /// color: text-primary }`, `.t3-wsteps--on .t3-guide-bullet { background:
    /// interactive; color: text-on-color }`, `--done` `support-success`.
    pub fn bullet(self) -> (Hsla, Hsla) {
        match self {
            Self::Done => (
                color(ColorRole::SupportSuccess),
                color(ColorRole::TextOnColor),
            ),
            Self::Current => (color(ColorRole::Interactive), color(ColorRole::TextOnColor)),
            Self::Upcoming => (
                color(ColorRole::LayerAccent01),
                color(ColorRole::TextPrimary),
            ),
        }
    }
}

/// `.t3-guide-bullet { width: 1.25rem; height: 1.25rem; border-radius: 50%;
/// font-size: 0.6875rem }` holding `content`.
pub(crate) fn bullet(fill: Hsla, ink: Hsla, content: AnyElement) -> gpui_kit::Div {
    sans(div(), 11., 400)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(rem(20.))
        .rounded_full()
        .bg(fill)
        .text_color(ink)
        .child(content)
}

/// The step indicator (`WizardSteps`, `.t3-wsteps`): numbered bullets, a
/// check for finished steps, the current one in blue.
#[derive(IntoElement, Clone, Debug)]
pub struct WizardSteps {
    steps: Vec<SharedString>,
    current: usize,
}

impl WizardSteps {
    /// `steps` with step `current` (0-based) shown.
    pub fn new<S: Into<SharedString>>(steps: impl IntoIterator<Item = S>, current: usize) -> Self {
        Self {
            steps: steps.into_iter().map(Into::into).collect(),
            current,
        }
    }
}

impl RenderOnce for WizardSteps {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.t3-wsteps { display: flex; gap: spacing-05; font-size: 0.75rem; color: text-secondary }`,
        // `li { display: flex; align-items: center; gap: spacing-02 }`.
        sans(div(), 12., 400)
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(rem(16.))
            .text_color(color(ColorRole::TextSecondary))
            .children(self.steps.iter().enumerate().map(|(i, s)| {
                let state = StepState::of(i, self.current);
                let (fill, ink) = state.bullet();
                let content = if state == StepState::Done {
                    press::icon(CarbonIcon::Checkmark, 12., ink).into_any_element()
                } else {
                    SharedString::from((i + 1).to_string()).into_any_element()
                };
                sans(
                    div(),
                    12.,
                    if state == StepState::Current {
                        600
                    } else {
                        400
                    },
                )
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(4.))
                .when(state == StepState::Current, |this| {
                    this.text_color(color(ColorRole::TextPrimary))
                })
                .child(bullet(fill, ink, content))
                .child(s.clone())
            }))
    }
}

/// Texts of a [`Wizard`].
#[derive(Clone, Debug)]
pub struct WizardTexts {
    /// `Back`.
    pub back: SharedString,
    /// The next button (`nextLabel`, default `Next`).
    pub next: SharedString,
}

impl Default for WizardTexts {
    fn default() -> Self {
        Self {
            back: "Back".into(),
            next: "Next".into(),
        }
    }
}

/// A stepped dialog panel (`Wizard`, `.t3-wizard`): title and
/// [`WizardSteps`], the content, then Back and Next filling the footer
/// half and half (64px, as a Carbon modal). Back is disabled on the first
/// step. `layer-01` with the floating shadow, at most 36rem.
#[derive(IntoElement)]
pub struct Wizard {
    id: ElementId,
    title: SharedString,
    steps: Vec<SharedString>,
    current: usize,
    children: Vec<AnyElement>,
    texts: WizardTexts,
    on_back: Option<Handler>,
    on_next: Option<Handler>,
}

impl Wizard {
    /// A wizard titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            steps: Vec::new(),
            current: 0,
            children: Vec::new(),
            texts: WizardTexts::default(),
            on_back: None,
            on_next: None,
        }
    }

    /// The step names (`steps`).
    pub fn steps<S: Into<SharedString>>(mut self, steps: impl IntoIterator<Item = S>) -> Self {
        self.steps = steps.into_iter().map(Into::into).collect();
        self
    }

    /// The current step (`current`, 0-based).
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    /// The next button's label (`nextLabel`).
    pub fn next_label(mut self, label: impl Into<SharedString>) -> Self {
        self.texts.next = label.into();
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: WizardTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Back pressed (`onBack`).
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }

    /// Next pressed (`onNext`).
    pub fn on_next(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_next = Some(Rc::new(f));
        self
    }
}

impl ParentElement for Wizard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Wizard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // `.t3-wizard-foot { display: flex }`, `.cds--btn { flex: 1; max-width: none }`.
        let half = |b: Button| div().flex_1().min_w_0().child(b.full_width(true));
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(576.))
            .bg(color(ColorRole::Layer01))
            .shadow(crate::components::overlay::floating_shadow())
            .child(
                // `.t3-wizard-head { display: flex; flex-direction: column; gap: spacing-04;
                // padding: spacing-05 }`, `h3 { font-size: 1.25rem; font-weight: 400 }`.
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(12.))
                    .p(rem(16.))
                    .child(
                        sans(div(), 20., 400)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(self.title.clone()),
                    )
                    .child(WizardSteps::new(self.steps.clone(), self.current)),
            )
            // `.t3-wizard-body { padding: 0 spacing-05 spacing-07 }`.
            .child(div().px(rem(16.)).pb(rem(32.)).children(self.children))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .child(half(click(
                        Button::new((id.clone(), "back"), self.texts.back.clone())
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Xl)
                            .balanced(true)
                            .disabled(self.current == 0),
                        self.on_back.clone(),
                    )))
                    .child(half(click(
                        Button::new((id.clone(), "next"), self.texts.next.clone())
                            .size(ButtonSize::Xl)
                            .balanced(true),
                        self.on_next.clone(),
                    ))),
            )
    }
}

// ── StatsGrid ───────────────────────────────────────────────────────────

/// `StatBlock` `tone`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatTone {
    /// `error`: the value in `text-error`.
    Error,
    /// `success`: the value in `support-success`.
    Success,
}

impl StatTone {
    /// The value's color (`.t3-stat--error .t3-stat-value { color: text-error }`,
    /// `--success` `support-success`).
    pub fn color(self) -> Hsla {
        color(match self {
            Self::Error => ColorRole::TextError,
            Self::Success => ColorRole::SupportSuccess,
        })
    }
}

/// One figure (`StatBlock`, `.t3-stat`): a label, the value in 28px light
/// tabular figures, an optional sub line. `layer-01`.
#[derive(IntoElement, Clone, Debug)]
pub struct StatBlock {
    label: SharedString,
    value: SharedString,
    sub: Option<SharedString>,
    tone: Option<StatTone>,
}

impl StatBlock {
    /// A figure.
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            sub: None,
            tone: None,
        }
    }

    /// The line under the value (`sub`, the `index.d.ts` name).
    #[allow(clippy::should_implement_trait)]
    pub fn sub(mut self, sub: impl Into<SharedString>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    /// The tone (`tone`).
    pub fn tone(mut self, tone: StatTone) -> Self {
        self.tone = Some(tone);
        self
    }
}

impl RenderOnce for StatBlock {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.t3-stat { display: flex; flex-direction: column; gap: spacing-02;
        // padding: spacing-04 spacing-05; background: layer-01 }`.
        div()
            .flex()
            .flex_col()
            .gap(rem(4.))
            .py(rem(12.))
            .px(rem(16.))
            .size_full()
            .bg(color(ColorRole::Layer01))
            // `.t3-stat-label { font-size: 0.75rem; color: text-secondary }`.
            .child(
                sans(div(), 12., 400)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.label),
            )
            // `.t3-stat-value { font-size: 1.75rem; line-height: 1.28572; font-weight: 300;
            // font-variant-numeric: tabular-nums }` (IBM Plex figures are tabular).
            .child(
                sans(div(), 28., 300)
                    .line_height(relative(BODY_LINE))
                    .text_color(
                        self.tone
                            .map(StatTone::color)
                            .unwrap_or(color(ColorRole::TextPrimary)),
                    )
                    .child(self.value),
            )
            // `.t3-stat-sub { font-size: 0.75rem; color: text-helper }`.
            .when_some(self.sub, |this, sub| {
                this.child(
                    sans(div(), 12., 400)
                        .text_color(color(ColorRole::TextHelper))
                        .child(sub),
                )
            })
    }
}

/// `.t3-statgrid { grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr)) }`.
pub const STAT_MIN_WIDTH: f32 = 144.;

/// The column count of an auto-fit grid `width` px wide holding `count`
/// blocks: as many 9rem tracks (1px gaps) as fit, at least one, and no
/// more than there are blocks (auto-fit collapses empty tracks).
pub fn stat_columns(width: f32, count: usize) -> usize {
    let fit = ((width + 1.) / (STAT_MIN_WIDTH + 1.)).floor().max(1.) as usize;
    fit.min(count.max(1))
}

/// A grid of [`StatBlock`]s (`StatsGrid`, `.t3-statgrid`): auto-fit 9rem
/// columns, 1px `border-subtle-01` gaps.
#[derive(IntoElement)]
pub struct StatsGrid {
    id: ElementId,
    children: Vec<AnyElement>,
}

impl StatsGrid {
    /// An empty grid.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }
}

impl ParentElement for StatsGrid {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for StatsGrid {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let bounds = window.use_keyed_state((id.clone(), "bounds"), cx, |_, _| {
            Bounds::<Pixels>::default()
        });
        let width = f32::from(bounds.read(cx).size.width)
            / (f32::from(window.rem_size()) / crate::tokens::REM_PX);
        let count = self.children.len();
        let columns = if width > 0. {
            stat_columns(width, count)
        } else {
            count.max(1)
        };
        let measure = canvas(
            move |b, _, cx| {
                if *bounds.read(cx) != b {
                    bounds.update(cx, |s, cx| {
                        *s = b;
                        cx.notify();
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let mut rows = Vec::new();
        let mut cells = self.children.into_iter().peekable();
        let mut r = 0;
        while cells.peek().is_some() {
            let mut row = div().flex().flex_row().gap(px(1.));
            for _ in 0..columns {
                // An empty track shows the grid's 1px-gap background.
                row = row.child(div().flex_1().min_w_0().children(cells.next()));
            }
            rows.push(row.id(child_id(&id, "row", r)));
            r += 1;
        }
        // `.t3-statgrid { display: grid; gap: 1px; background: border-subtle-01 }`.
        div()
            .id(id)
            .relative()
            .flex()
            .flex_col()
            .gap(px(1.))
            .w_full()
            .bg(color(ColorRole::BorderSubtle01))
            .children(rows)
            .child(measure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_joins_provider_and_version() {
        assert_eq!(
            provider_subtitle("Claude (work)", "Claude", Some("2.1.4")),
            "Claude \u{b7} v2.1.4"
        );
        assert_eq!(provider_subtitle("Codex", "Codex", None), "");
        assert_eq!(provider_subtitle("Codex", "Codex", Some("1.0")), "v1.0");
        assert_eq!(provider_subtitle("Work", "Kiro", None), "Kiro");
    }

    #[test]
    fn provider_tags_follow_the_status_map() {
        assert_eq!(ProviderStatus::Ready.variant(), BadgeVariant::Green);
        assert_eq!(ProviderStatus::Signin.variant(), BadgeVariant::WarmGray);
        assert_eq!(ProviderStatus::Error.label(), "Not found");
        assert!(ProviderStatus::Signin.colors().is_some());
        assert!(ProviderStatus::Ready.colors().is_none());
    }

    #[test]
    fn steps_before_current_are_done() {
        assert_eq!(StepState::of(0, 1), StepState::Done);
        assert_eq!(StepState::of(1, 1), StepState::Current);
        assert_eq!(StepState::of(2, 1), StepState::Upcoming);
    }

    #[test]
    fn stat_grid_fits_9rem_tracks() {
        assert_eq!(stat_columns(720., 5), 4);
        assert_eq!(stat_columns(1000., 5), 5);
        assert_eq!(stat_columns(100., 5), 1);
        assert_eq!(stat_columns(289., 3), 2);
        assert_eq!(stat_columns(288., 3), 1);
    }
}
