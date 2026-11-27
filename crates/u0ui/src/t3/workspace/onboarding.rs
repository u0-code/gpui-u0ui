//! T3 · Onboarding & feedback: [`NoProjectsHero`], [`ComputerSetupSteps`],
//! [`PermissionChecklist`], [`ExpandableToast`], [`MiddleTruncate`] and
//! [`RelativeTime`] (`src/t3/workspace.jsx`, "Onboarding" in `src/t3.css`).
use super::{
    cards::bullet,
    parts::{card_sub, child_id, icon_button, title_tooltip},
};
use crate::{
    components::{Button, ButtonSize, ButtonVariant, CopyButton},
    icons::CarbonIcon,
    t3::{
        base::Spin,
        craft::{mono, sans},
        press::{self, Handler, rem},
        shell::{EnvironmentMachineIcon, MachineKind},
    },
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, ClipboardItem, CursorStyle, ElementId, Hsla, InteractiveElement as _,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px, relative,
};
use std::{
    rc::Rc,
    time::{Duration, SystemTime},
};

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

fn click(b: Button, f: Option<Handler>) -> Button {
    match f {
        Some(f) => b.on_click(move |_, w, cx| f(w, cx)),
        None => b,
    }
}

// ── NoProjectsHero ──────────────────────────────────────────────────────

/// Texts of a [`NoProjectsHero`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct NoProjectsTexts {
    /// `What should we work on?`
    pub title: SharedString,
    /// `Add project`.
    pub add: SharedString,
    /// `Start without a project`.
    pub skip: SharedString,
}

impl Default for NoProjectsTexts {
    fn default() -> Self {
        Self {
            title: "What should we work on?".into(),
            add: "Add project".into(),
            skip: "Start without a project".into(),
        }
    }
}

/// The first screen without projects (`NoProjectsHero`, `.t3-hero`): a
/// 42px light headline, then Add project and Start without a project.
#[derive(IntoElement)]
pub struct NoProjectsHero {
    id: ElementId,
    texts: NoProjectsTexts,
    on_add: Option<Handler>,
    on_skip: Option<Handler>,
}

impl NoProjectsHero {
    /// The hero.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            texts: NoProjectsTexts::default(),
            on_add: None,
            on_skip: None,
        }
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: NoProjectsTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Add project pressed (`onAdd`).
    pub fn on_add(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add = Some(Rc::new(f));
        self
    }

    /// Start without a project pressed (`onSkip`).
    pub fn on_skip(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_skip = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for NoProjectsHero {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // `.t3-hero { display: flex; flex-direction: column; align-items: center;
        // gap: spacing-07; padding: spacing-10 spacing-05; text-align: center }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .items_center()
            .gap(rem(32.))
            .py(rem(64.))
            .px(rem(16.))
            .text_center()
            // `h2 { font-size: 2.625rem; line-height: 1.199; font-weight: 300 }`.
            .child(
                sans(div(), 42., 300)
                    .line_height(relative(1.199))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.texts.title.clone()),
            )
            // `.t3-hero-actions { display: flex; gap: spacing-03 }`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .justify_center()
                    .gap(rem(8.))
                    .child(click(
                        Button::new((id.clone(), "add"), self.texts.add.clone())
                            .icon(CarbonIcon::FolderOpen),
                        self.on_add.clone(),
                    ))
                    .child(click(
                        Button::new((id.clone(), "skip"), self.texts.skip.clone())
                            .variant(ButtonVariant::Ghost),
                        self.on_skip.clone(),
                    )),
            )
    }
}

// ── ComputerSetupSteps ──────────────────────────────────────────────────

/// `ComputerSetupSteps` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SetupState {
    /// `waiting` (the default): no computer connected yet.
    #[default]
    Waiting,
    /// `off`: the paired computer is off.
    Off,
}

impl SetupState {
    /// Both states.
    pub const ALL: [SetupState; 2] = [Self::Waiting, Self::Off];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Off => "off",
        }
    }
}

/// Texts of a [`ComputerSetupSteps`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct SetupTexts {
    /// Headings for waiting / off.
    pub title: [SharedString; 2],
    /// Notes for waiting / off.
    pub body: [SharedString; 2],
    /// Step 1 (`Run on your computer:`).
    pub run: SharedString,
    /// The copy button's name (`Copy command`).
    pub copy: SharedString,
    /// The copy feedback (`Copied!`).
    pub copied: SharedString,
    /// Step 2.
    pub sign_in: SharedString,
    /// Step 3.
    pub waiting: SharedString,
    /// `Add pairing link`.
    pub pair: SharedString,
    /// `Or download the desktop app`.
    pub download: SharedString,
}

impl Default for SetupTexts {
    fn default() -> Self {
        Self {
            title: [
                "No computer connected".into(),
                "This computer is off".into(),
            ],
            body: [
                "Your agents run on your computer. Start the app there first.".into(),
                "Turn it back on, or connect a different computer.".into(),
            ],
            run: "Run on your computer:".into(),
            copy: "Copy command".into(),
            copied: "Copied!".into(),
            sign_in: "Sign in with the same account.".into(),
            waiting: "Waiting for your computer\u{2026}".into(),
            pair: "Add pairing link".into(),
            download: "Or download the desktop app".into(),
        }
    }
}

/// Connecting a computer (`ComputerSetupSteps`, `.t3-setup`): a machine
/// glyph, heading and note, three numbered steps (run the command with a
/// copy button, sign in, wait with a spinner), then Add pairing link and
/// Download.
#[derive(IntoElement)]
pub struct ComputerSetupSteps {
    id: ElementId,
    state: SetupState,
    command: SharedString,
    texts: SetupTexts,
    on_pair: Option<Handler>,
    on_download: Option<Handler>,
}

impl ComputerSetupSteps {
    /// The steps (command `npx t3@latest` in the reference).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            state: SetupState::Waiting,
            command: "npx t3@latest".into(),
            texts: SetupTexts::default(),
            on_pair: None,
            on_download: None,
        }
    }

    /// The state (`state`).
    pub fn state(mut self, state: SetupState) -> Self {
        self.state = state;
        self
    }

    /// The command to run (`command`).
    pub fn command(mut self, command: impl Into<SharedString>) -> Self {
        self.command = command.into();
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: SetupTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Add pairing link pressed.
    pub fn on_pair(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_pair = Some(Rc::new(f));
        self
    }

    /// Download pressed.
    pub fn on_download(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_download = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ComputerSetupSteps {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let k = match self.state {
            SetupState::Waiting => 0,
            SetupState::Off => 1,
        };
        let (fill, ink) = (
            color(ColorRole::LayerAccent01),
            color(ColorRole::TextPrimary),
        );
        let number = |n: &'static str| bullet(fill, ink, n.into_any_element());
        // `.t3-setup-steps li { display: flex; gap: spacing-04; align-items: flex-start }`.
        let step = |mark: gpui_kit::Div, content: AnyElement| {
            div()
                .flex()
                .flex_row()
                .items_start()
                .gap(rem(12.))
                .child(mark)
                .child(div().flex_1().min_w_0().child(content))
        };
        // `.t3-setup-cmd { display: flex; align-items: center; justify-content: space-between;
        // gap: spacing-04; margin-top: spacing-03; padding-left: spacing-04; background: layer-01;
        // font-family: mono; font-size: 0.875rem }`.
        let command = mono(div(), 14., 400)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(rem(12.))
            .mt(rem(8.))
            .pl(rem(12.))
            .bg(color(ColorRole::Layer01))
            .child(self.command.clone())
            .child(
                // `.t3-copy.cds--copy-btn { width: size-sm; height: size-sm; background: transparent }`.
                CopyButton::new((id.clone(), "copy"), self.command.clone())
                    .label(self.texts.copy.clone())
                    .feedback(self.texts.copied.clone())
                    .size(32.)
                    .quiet(true),
            );
        // `.t3-setup-steps { display: flex; flex-direction: column; gap: spacing-04;
        // margin: spacing-03 0; font-size: 0.875rem }`.
        let steps = sans(div(), 14., 400)
            .flex()
            .flex_col()
            .gap(rem(12.))
            .my(rem(8.))
            .w_full()
            .child(step(
                number("1"),
                div()
                    .flex()
                    .flex_col()
                    .child(self.texts.run.clone())
                    .child(command)
                    .into_any_element(),
            ))
            .child(step(
                number("2"),
                self.texts.sign_in.clone().into_any_element(),
            ))
            .child(step(
                // `.t3-guide-bullet--wait { background: transparent }` holding a Spin.
                bullet(
                    gpui_kit::transparent_black(),
                    ink,
                    Spin::new((id.clone(), "wait")).into_any_element(),
                ),
                self.texts.waiting.clone().into_any_element(),
            ));
        // `.t3-setup { display: flex; flex-direction: column; align-items: flex-start;
        // gap: spacing-04; max-width: 32rem; padding: spacing-07 spacing-05 }`.
        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .flex_col()
            .items_start()
            .gap(rem(12.))
            .max_w(rem(512.))
            .py(rem(32.))
            .px(rem(16.))
            .text_color(color(ColorRole::TextPrimary))
            .child(EnvironmentMachineIcon::new(MachineKind::Laptop).size(32.))
            // `h3 { font-size: 1.25rem; font-weight: 400 }`.
            .child(sans(div(), 20., 400).child(self.texts.title[k].clone()))
            // `p { font-size: 0.875rem; color: text-secondary }`.
            .child(
                sans(div(), 14., 400)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.texts.body[k].clone()),
            )
            .child(steps)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(rem(8.))
                    .child(click(
                        Button::new((id.clone(), "pair"), self.texts.pair.clone())
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Md)
                            .icon(CarbonIcon::QrCode),
                        self.on_pair.clone(),
                    ))
                    .child(click(
                        Button::new((id.clone(), "download"), self.texts.download.clone())
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Md)
                            .icon(CarbonIcon::Download),
                        self.on_download.clone(),
                    )),
            )
    }
}

// ── PermissionChecklist ─────────────────────────────────────────────────

/// One permission (`items[]`: `{ label, why, granted }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permission {
    /// `label`.
    pub label: SharedString,
    /// `why`.
    pub why: SharedString,
    /// `granted`.
    pub granted: bool,
}

impl Permission {
    /// A permission.
    pub fn new(
        label: impl Into<SharedString>,
        why: impl Into<SharedString>,
        granted: bool,
    ) -> Self {
        Self {
            label: label.into(),
            why: why.into(),
            granted,
        }
    }
}

/// Texts of a [`PermissionChecklist`].
#[derive(Clone, Debug)]
pub struct PermissionTexts {
    /// `Granted`.
    pub granted: SharedString,
    /// `Open settings`.
    pub open: SharedString,
}

impl Default for PermissionTexts {
    fn default() -> Self {
        Self {
            granted: "Granted".into(),
            open: "Open settings".into(),
        }
    }
}

/// System permissions (`PermissionChecklist`, `.t3-perms`): each row a
/// check (granted, green) or dashed circle, label and why, then Granted or
/// Open settings. `layer-01` rows 1px apart.
#[derive(IntoElement)]
pub struct PermissionChecklist {
    id: ElementId,
    items: Vec<Permission>,
    texts: PermissionTexts,
    on_open: Option<IndexHandler>,
}

impl PermissionChecklist {
    /// A checklist of `items`.
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = Permission>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            texts: PermissionTexts::default(),
            on_open: None,
        }
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: PermissionTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Open settings pressed, with the item's index.
    pub fn on_open_settings(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PermissionChecklist {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let rows = self.items.iter().enumerate().map(|(i, p)| {
            // `.t3-perm > svg { fill: icon-secondary }`, `.t3-perm--ok > svg { fill: support-success }`.
            let (glyph, fill) = if p.granted {
                (CarbonIcon::CheckmarkFilled, ColorRole::SupportSuccess)
            } else {
                (CarbonIcon::CircleDash, ColorRole::IconSecondary)
            };
            let end = if p.granted {
                card_sub(self.texts.granted.clone()).into_any_element()
            } else {
                let f = self.on_open.clone();
                Button::new(child_id(&id, "open", i), self.texts.open.clone())
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .when_some(f, |b, f| b.on_click(move |_, w, cx| f(i, w, cx)))
                    .into_any_element()
            };
            // `.t3-perm { display: flex; align-items: center; gap: spacing-04; min-height: size-xl;
            // padding: spacing-03 spacing-04; background: layer-01 }`.
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(12.))
                .min_h(rem(64.))
                .py(rem(8.))
                .px(rem(12.))
                .bg(color(ColorRole::Layer01))
                .child(press::icon(glyph, 20., color(fill)).flex_none())
                .child(
                    // `> span:nth-of-type(1) { display: flex; flex: 1; flex-direction: column;
                    // gap: 2px; font-size: 0.875rem }`, `b { font-weight: 600 }`.
                    sans(div(), 14., 400)
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .gap(px(2.))
                        .text_color(color(ColorRole::TextPrimary))
                        .child(sans(div(), 14., 600).child(p.label.clone()))
                        .child(card_sub(p.why.clone())),
                )
                .child(end)
        });
        // `.t3-perms { display: flex; flex-direction: column; gap: 1px; max-width: none }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .gap(px(1.))
            .w_full()
            .children(rows)
    }
}

// ── ExpandableToast ─────────────────────────────────────────────────────

/// `ExpandableToast` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToastKind {
    /// `error` (the default): `support-error`.
    #[default]
    Error,
    /// `success`: `support-success`.
    Success,
    /// `info`: `support-info`.
    Info,
}

impl ToastKind {
    /// Every kind.
    pub const ALL: [ToastKind; 3] = [Self::Error, Self::Success, Self::Info];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Success => "success",
            Self::Info => "info",
        }
    }

    /// The accent of the start bar and glyph (`--t3-accent`).
    pub fn accent(self) -> Hsla {
        color(match self {
            Self::Error => ColorRole::SupportError,
            Self::Success => ColorRole::SupportSuccess,
            Self::Info => ColorRole::SupportInfo,
        })
    }

    /// The glyph (`ErrorFilled`, `CheckmarkFilled`, `InformationFilled`).
    pub const fn glyph(self) -> CarbonIcon {
        match self {
            Self::Error => CarbonIcon::ErrorFilled,
            Self::Success => CarbonIcon::CheckmarkFilled,
            Self::Info => CarbonIcon::InformationFilled,
        }
    }
}

/// Texts of an [`ExpandableToast`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct ExpandableToastTexts {
    /// `Show details`.
    pub show: SharedString,
    /// `Hide details`.
    pub hide: SharedString,
    /// `Copy error`.
    pub copy: SharedString,
    /// `Close`.
    pub close: SharedString,
}

impl Default for ExpandableToastTexts {
    fn default() -> Self {
        Self {
            show: "Show details".into(),
            hide: "Hide details".into(),
            copy: "Copy error".into(),
            close: "Close".into(),
        }
    }
}

/// A notification whose raw details expand (`ExpandableToast`,
/// `.t3-xtoast`): a 3px accent start bar and 20px glyph, title,
/// description, a Show details link opening the monospace details, Copy
/// error (with details) and Close. 24rem, `layer-01`, floating shadow.
#[derive(IntoElement)]
pub struct ExpandableToast {
    id: ElementId,
    kind: ToastKind,
    title: SharedString,
    description: Option<SharedString>,
    details: Option<SharedString>,
    texts: ExpandableToastTexts,
    on_close: Option<Handler>,
}

impl ExpandableToast {
    /// A `kind` toast titled `title`.
    pub fn new(id: impl Into<ElementId>, kind: ToastKind, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind,
            title: title.into(),
            description: None,
            details: None,
            texts: ExpandableToastTexts::default(),
            on_close: None,
        }
    }

    /// The description (`description`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The raw details (`details`).
    pub fn details(mut self, details: impl Into<SharedString>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: ExpandableToastTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Close pressed (`onClose`).
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ExpandableToast {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, |_, _| false);
        let open = *open_state.read(cx);
        let accent = self.kind.accent();
        let link = self.details.clone().map(|_| {
            let link_id = ElementId::from((id.clone(), "details-link"));
            let (hovered, hover) = press::hover((link_id.clone(), "hover"), window, cx);
            let (focus, ring) = press::focus((link_id.clone(), "focus"), window, cx);
            let label = if open {
                self.texts.hide.clone()
            } else {
                self.texts.show.clone()
            };
            // `.t3-linkbtn { padding: 0; color: link-primary; font-size: 0.75rem }`,
            // `:hover { text-decoration: underline }`; `align-self: flex-start`.
            sans(
                gpui_kit::base::Button::new(link_id)
                    .track_focus(&focus)
                    .accessibility_label(label.clone()),
                12.,
                400,
            )
            .relative()
            .self_start()
            .text_color(color(ColorRole::LinkPrimary))
            .when(hovered, |this| this.underline())
            .cursor(CursorStyle::PointingHand)
            .on_hover(press::track_hover(hover))
            .on_click(move |_, _, cx| {
                open_state.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                })
            })
            .child(label)
            .when(ring, |this| this.child(press::ring()))
        });
        let details = self.details.clone().filter(|_| open).map(|d| {
            // `.t3-xtoast-pre { margin: 0; max-height: 8rem; overflow: auto; padding: spacing-03;
            // background: layer-02; font-family: mono; font-size: 0.75rem; white-space: pre-wrap }`.
            crate::components::ScrollArea::new((id.clone(), "pre"))
                .max_h(rem(128.))
                .bg(color(ColorRole::Layer02))
                .child(
                    mono(div(), 12., 400)
                        .p(rem(8.))
                        .text_color(color(ColorRole::TextPrimary))
                        .children(d.split('\n').map(|l| {
                            div().child(SharedString::from(if l.is_empty() {
                                " ".to_string()
                            } else {
                                l.to_string()
                            }))
                        })),
                )
        });
        let copy = self.details.clone().map(|d| {
            icon_button(
                (id.clone(), "copy"),
                self.texts.copy.clone(),
                CarbonIcon::Copy,
            )
            .on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(d.to_string()))
            })
        });
        let on_close = self.on_close.clone();
        let mut shadows = crate::components::overlay::floating_shadow();
        // `box-shadow: 0 2px 6px shadow, inset 3px 0 0 var(--t3-accent)`.
        shadows.push(gpui_kit::BoxShadow::new(px(3.), px(0.), accent).inset());
        // `.t3-xtoast { display: flex; align-items: flex-start; gap: spacing-04; width: 24rem;
        // padding: spacing-04 spacing-02 spacing-04 spacing-04; background: layer-01 }`.
        div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_start()
            .gap(rem(12.))
            .w(rem(384.))
            .max_w_full()
            .py(rem(12.))
            .pl(rem(12.))
            .pr(rem(4.))
            .bg(color(ColorRole::Layer01))
            .shadow(shadows)
            .child(press::icon(self.kind.glyph(), 20., accent).flex_none())
            .child(
                // `.t3-xtoast-body { display: flex; flex: 1; min-width: 0; flex-direction: column;
                // gap: spacing-02; font-size: 0.875rem }`, `b { 600 }`, `> span { text-secondary }`.
                sans(div(), 14., 400)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(rem(4.))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(sans(div(), 14., 600).child(self.title.clone()))
                    .when_some(self.description.clone(), |this, d| {
                        this.child(div().text_color(color(ColorRole::TextSecondary)).child(d))
                    })
                    .children(link)
                    .children(details),
            )
            // `.t3-xtoast-acts { display: flex }`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .children(copy)
                    .child(click(
                        icon_button(
                            (id.clone(), "close"),
                            self.texts.close.clone(),
                            CarbonIcon::Close,
                        ),
                        on_close,
                    )),
            )
    }
}

// ── MiddleTruncate / RelativeTime ───────────────────────────────────────

/// `text` with its middle cut (`MiddleTruncate`): `None` when it already
/// fits (`length <= keep × 2 + 1`), else the first and last `keep`
/// characters around `…`.
pub fn middle_truncate(text: &str, keep: usize) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= keep * 2 + 1 {
        return None;
    }
    let head: String = chars[..keep].iter().collect();
    let tail: String = chars[chars.len() - keep..].iter().collect();
    Some(format!("{head}\u{2026}{tail}"))
}

/// A hash or id with its middle cut (`MiddleTruncate`, `.t3-mtrunc`: mono
/// 12px, titled with the full text). Short text renders unchanged.
#[derive(IntoElement, Clone, Debug)]
pub struct MiddleTruncate {
    id: ElementId,
    text: SharedString,
    keep: usize,
}

impl MiddleTruncate {
    /// `text`, keeping 8 characters at each end.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            keep: 8,
        }
    }

    /// Characters kept at each end (`keep`, default 8).
    pub fn keep(mut self, keep: usize) -> Self {
        self.keep = keep;
        self
    }
}

impl RenderOnce for MiddleTruncate {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match middle_truncate(&self.text, self.keep) {
            None => div().id(self.id).child(self.text),
            Some(short) => mono(div(), 12., 400)
                .id(self.id)
                .tooltip(title_tooltip(self.text.clone()))
                .child(short),
        }
    }
}

/// The compact age of something `secs` seconds old (`RelativeTime`):
/// under 45s `now`, then whole minutes, hours and days, rounded.
pub fn relative_label(secs: f64) -> String {
    let round = |x: f64| (x + 0.5).floor() as i64;
    if secs < 45. {
        "now".into()
    } else if secs < 3600. {
        format!("{}m", round(secs / 60.))
    } else if secs < 86400. {
        format!("{}h", round(secs / 3600.))
    } else {
        format!("{}d", round(secs / 86400.))
    }
}

/// `toLocaleString()` in the reference's `en-US` locale, local time:
/// `11/27/2026, 2:05:09 PM`.
pub fn locale_string(date: SystemTime) -> String {
    let t: chrono::DateTime<chrono::Local> = date.into();
    t.format("%-m/%-d/%Y, %-I:%M:%S %p").to_string()
}

/// A compact age (`RelativeTime`, `.t3-reltime`: 12px `text-helper`),
/// titled with the full local date and time, computed when it renders
/// (as the reference's `Date.now()`).
#[derive(IntoElement, Clone, Debug)]
pub struct RelativeTime {
    id: ElementId,
    date: SystemTime,
}

impl RelativeTime {
    /// The age of `date`.
    pub fn new(id: impl Into<ElementId>, date: SystemTime) -> Self {
        Self {
            id: id.into(),
            date,
        }
    }
}

impl RenderOnce for RelativeTime {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let age = SystemTime::now()
            .duration_since(self.date)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();
        sans(div(), 12., 400)
            .id(self.id)
            .text_color(color(ColorRole::TextHelper))
            .tooltip(title_tooltip(locale_string(self.date).into()))
            .child(relative_label(age))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn middle_truncate_keeps_both_ends() {
        assert_eq!(
            middle_truncate("sha256:9f2c1e0a7b4d5e6f8a9b0c1d2e3f4a5b", 10).as_deref(),
            Some("sha256:9f2\u{2026}1d2e3f4a5b")
        );
        assert_eq!(middle_truncate("short", 8), None);
        // `length <= keep * 2 + 1` still fits.
        assert_eq!(middle_truncate("abcdefghi", 4), None);
        assert_eq!(
            middle_truncate("abcdefghij", 4).as_deref(),
            Some("abcd\u{2026}ghij")
        );
    }

    #[test]
    fn relative_labels_round_to_the_unit() {
        assert_eq!(relative_label(10.), "now");
        assert_eq!(relative_label(44.9), "now");
        assert_eq!(relative_label(45.), "1m");
        assert_eq!(relative_label(180.), "3m");
        assert_eq!(relative_label(7200.), "2h");
        assert_eq!(relative_label(86400. * 5.), "5d");
    }

    #[test]
    fn toast_accents_follow_the_kind() {
        assert_eq!(ToastKind::Error.accent(), color(ColorRole::SupportError));
        assert_eq!(ToastKind::Info.glyph(), CarbonIcon::InformationFilled);
    }
}
