//! Pairing (`src/ghost/admin.jsx` "Devices", `.g-pair` and `.g-sheet` in
//! `src/ghost.css`):
//!
//! - [`PairingCard`]: the QR code of a pairing link, the link in a
//!   [`CopyField`], a "m:ss" countdown and "New link". At zero the QR code
//!   becomes "Link expired" and the countdown "Expired" in `text-error`. The
//!   countdown restarts whenever the link or `expires_in` changes.
//! - [`PairConfirmSheet`]: the confirmation shown on this Mac when another
//!   device asks to pair: its fingerprint to compare, an "Also let it control
//!   this Mac" box (off by default) and "Don't pair" / "Pair".
use crate::{
    components::{Button, ButtonSize, ButtonVariant, overlay::floating_shadow},
    icons::{CarbonIcon, Icon},
    t3::{
        base::use_ticker,
        craft::{mono, rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
    u0::{
        fields::InlineCheck,
        primitives::{CopyField, QrCode, parts},
        rows as row,
    },
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// `Expires in ${Math.floor(left / 60)}:${String(left % 60).padStart(2, '0')}`'s clock.
pub fn countdown(left: u64) -> String {
    format!("{}:{:02}", left / 60, left % 60)
}

/// `.g-pair-qr` / `.g-pair-expired { width: 168px; height: 168px }`.
pub const QR_SIZE: f32 = 168.;

/// [`PairingCard`]'s user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct PairingCardTexts {
    /// `"Pair a phone or another Mac"`.
    pub title: SharedString,
    /// The instruction line.
    pub body: SharedString,
    /// `"Pairing link"`.
    pub link: SharedString,
    /// `"Pairing QR code"`.
    pub qr: SharedString,
    /// `"Link expired"` (over the QR code).
    pub link_expired: SharedString,
    /// `"Expired"`.
    pub expired: SharedString,
    /// `"Expires in {m:ss}"`.
    pub expires_in: Rc<dyn Fn(&str) -> SharedString>,
    /// `"New link"`.
    pub new_link: SharedString,
}

impl Default for PairingCardTexts {
    fn default() -> Self {
        Self {
            title: "Pair a phone or another Mac".into(),
            body: "Scan with the app on your phone, or open the link on the other device.".into(),
            link: "Pairing link".into(),
            qr: "Pairing QR code".into(),
            link_expired: "Link expired".into(),
            expired: "Expired".into(),
            expires_in: Rc::new(|t| format!("Expires in {t}").into()),
            new_link: "New link".into(),
        }
    }
}

/// The pairing card. See the module docs.
#[derive(IntoElement)]
pub struct PairingCard {
    id: ElementId,
    link: SharedString,
    expires_in: u64,
    texts: PairingCardTexts,
    on_new_link: Option<Handler>,
}

impl PairingCard {
    /// A card for the reference link, expiring in 10 minutes.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            link: "u0://pair/7f3a-9c2e-41b0".into(),
            expires_in: 600,
            texts: PairingCardTexts::default(),
            on_new_link: None,
        }
    }

    /// The pairing link (`link`).
    pub fn link(mut self, link: impl Into<SharedString>) -> Self {
        self.link = link.into();
        self
    }

    /// Seconds until it expires (`expiresIn`, default 600).
    pub fn expires_in(mut self, secs: u64) -> Self {
        self.expires_in = secs;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: PairingCardTexts) -> Self {
        self.texts = texts;
        self
    }

    /// "New link" pressed (`onNewLink`).
    pub fn on_new_link(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new_link = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PairingCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // The countdown restarts when the link or the expiry changes
        // (`useEffect(…, [expiresIn, link])`): key the ticker by both.
        let key = SharedString::from(format!("countdown-{}-{}", self.link, self.expires_in));
        let running_state = window.use_keyed_state((id.clone(), key.clone()), cx, |_, _| true);
        let running = *running_state.read(cx);
        let elapsed = use_ticker((id.clone(), key), 0, running, window, cx);
        let left = self.expires_in.saturating_sub(elapsed);
        if running && left == 0 {
            running_state.update(cx, |r, _| *r = false);
        }
        let expired = left == 0;
        let on_new = self.on_new_link.clone();

        let qr = if expired {
            // `.g-pair-expired { width/height: 168px; flex-direction: column;
            // gap: var(--spacing-03); background: var(--layer-accent-01); color:
            // var(--text-secondary); font-size: 0.75rem; line-height: 1.3 }`.
            sans(div(), 12., 400)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(tokens::SPACING_03)
                .size(gpui_kit::px(QR_SIZE))
                .bg(color(ColorRole::LayerAccent01))
                .text_color(color(ColorRole::TextSecondary))
                .child(
                    Icon::carbon(CarbonIcon::Time)
                        .size_length(rem(24.))
                        .color(color(ColorRole::TextSecondary)),
                )
                .child(self.texts.link_expired.clone())
                .into_any_element()
        } else {
            // `.g-pair-qr .g-qr { box-shadow: 0 0 0 1px var(--border-subtle-01) }`.
            div()
                .p(gpui_kit::px(1.))
                .bg(color(ColorRole::BorderSubtle01))
                .m(gpui_kit::px(-1.))
                .child(
                    QrCode::new((id.clone(), "qr"), self.link.clone())
                        .size(QR_SIZE)
                        .label(self.texts.qr.clone()),
                )
                .into_any_element()
        };

        let status = if expired {
            sans(div(), 12., 400)
                .text_color(color(ColorRole::TextError))
                .child(self.texts.expired.clone())
        } else {
            row::dim((self.texts.expires_in)(&countdown(left)), 12.)
        };

        // `.g-pair { display: flex; gap: var(--spacing-06); padding:
        // var(--spacing-05); background: var(--layer-01) }`.
        div()
            .flex()
            .gap(tokens::SPACING_06)
            .p(tokens::SPACING_05)
            .bg(color(ColorRole::Layer01))
            .child(div().flex_none().child(qr))
            .child(
                // `.g-pair-side { display: flex; flex: 1; min-width: 0;
                // flex-direction: column; gap: var(--spacing-04); font-size: 0.875rem }`.
                sans(div(), 14., 400)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(tokens::SPACING_04)
                    .text_color(color(ColorRole::TextPrimary))
                    // `.g-pair-side b { font-size: 1rem; font-weight: 600 }`.
                    .child(sans(div(), 16., 600).child(self.texts.title.clone()))
                    // `.g-pair-side p { margin: calc(-1 * var(--spacing-03)) 0 0;
                    // font-size: 0.75rem }` in `.g-dim`; Carbon's global `p {
                    // line-height: 1.5 }` makes the lines 18px (reference render).
                    .child(
                        sans(div(), 12., 400)
                            .line_height(rem(18.))
                            .mt(rem(-8.))
                            .text_color(color(ColorRole::TextSecondary))
                            .child(self.texts.body.clone()),
                    )
                    .child(
                        CopyField::new((id.clone(), "link"), self.link.clone())
                            .label(self.texts.link.clone()),
                    )
                    .child(
                        // `.g-pair-foot { display: flex; align-items: center;
                        // justify-content: space-between; margin-top: auto;
                        // font-size: 0.75rem }`.
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .mt_auto()
                            .child(status)
                            .child(
                                Button::new((id.clone(), "new"), self.texts.new_link.clone())
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Sm)
                                    .icon(CarbonIcon::Renew)
                                    .when_some(on_new, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                            ),
                    ),
            )
    }
}

/// [`PairConfirmSheet`]'s user-facing strings (the reference's English
/// defaults; `{device}` is the requesting device).
#[derive(Clone)]
pub struct PairConfirmTexts {
    /// `"Pair “{device}”?"`.
    pub title: Rc<dyn Fn(&str) -> SharedString>,
    /// The consequence line.
    pub body: SharedString,
    /// `"Fingerprint"`.
    pub fingerprint: SharedString,
    /// `"Check it matches the one shown on {device}."`.
    pub check: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Also let it control this Mac"`.
    pub control: SharedString,
    /// `"Start sessions, approve commands and run terminals from {device}."`.
    pub control_detail: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Don’t pair"`.
    pub deny: SharedString,
    /// `"Pair"`.
    pub allow: SharedString,
}

impl Default for PairConfirmTexts {
    fn default() -> Self {
        Self {
            title: Rc::new(|d| format!("Pair “{d}”?").into()),
            body: "It will see your sessions and can send messages to agents on this Mac.".into(),
            fingerprint: "Fingerprint".into(),
            check: Rc::new(|d| format!("Check it matches the one shown on {d}.").into()),
            control: "Also let it control this Mac".into(),
            control_detail: Rc::new(|d| {
                format!("Start sessions, approve commands and run terminals from {d}.").into()
            }),
            deny: "Don’t pair".into(),
            allow: "Pair".into(),
        }
    }
}

type AllowHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// The pairing confirmation. See the module docs.
#[derive(IntoElement)]
pub struct PairConfirmSheet {
    id: ElementId,
    device: SharedString,
    fingerprint: SharedString,
    texts: PairConfirmTexts,
    on_allow: Option<AllowHandler>,
    on_deny: Option<Handler>,
}

impl PairConfirmSheet {
    /// The reference request from "iPhone 16".
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            device: "iPhone 16".into(),
            fingerprint: "A1F3 9C2E 7B04 51D8".into(),
            texts: PairConfirmTexts::default(),
            on_allow: None,
            on_deny: None,
        }
    }

    /// The requesting device's name (`device`).
    pub fn device(mut self, device: impl Into<SharedString>) -> Self {
        self.device = device.into();
        self
    }

    /// The fingerprint both sides show (`fingerprint`).
    pub fn fingerprint(mut self, fingerprint: impl Into<SharedString>) -> Self {
        self.fingerprint = fingerprint.into();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: PairConfirmTexts) -> Self {
        self.texts = texts;
        self
    }

    /// "Pair" pressed (`onAllow(canControl)`).
    pub fn on_allow(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_allow = Some(Rc::new(f));
        self
    }

    /// "Don't pair" pressed (`onDeny`).
    pub fn on_deny(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_deny = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PairConfirmSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let ctrl = window.use_keyed_state((id.clone(), "control"), cx, |_, _| false);
        let on = *ctrl.read(cx);
        let (on_allow, on_deny) = (self.on_allow.clone(), self.on_deny.clone());
        let d = self.device.clone();
        // `.g-sheet { display: flex; flex-direction: column; gap:
        // var(--spacing-03); max-width: 26rem; padding: var(--spacing-06)
        // var(--spacing-06) 0; background: var(--layer-01); box-shadow: 0 2px
        // 6px var(--shadow); font-size: 0.875rem }`.
        sans(div(), 14., 400)
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .w_full()
            .max_w(rem(416.))
            .pt(tokens::SPACING_06)
            .px(tokens::SPACING_06)
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .text_color(color(ColorRole::TextPrimary))
            .child(
                // `.g-sheet-icon { width/height: 3rem; background:
                // var(--layer-accent-01) }`, `svg { fill: var(--icon-primary) }`.
                div().flex().child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(rem(48.))
                        .bg(color(ColorRole::LayerAccent01))
                        .child(
                            Icon::carbon(CarbonIcon::Mobile)
                                .size_length(rem(24.))
                                .color(color(ColorRole::IconPrimary)),
                        ),
                ),
            )
            // `.g-sheet h3 { margin: 0; font-size: 1.25rem; font-weight: 400 }`.
            .child(sans(div(), 20., 400).child((self.texts.title)(&d)))
            // `.g-sheet > p { margin: 0 }` in `.g-dim`: Carbon's global `p {
            // font-size: 1rem; line-height: 1.5; letter-spacing: 0 }` (reference
            // render: 16/24).
            .child(
                sans(div(), 16., 400)
                    .line_height(rem(24.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.texts.body.clone()),
            )
            .child(
                // `.g-fp { display: flex; justify-content: space-between;
                // align-items: center; margin-top: var(--spacing-03); padding:
                // var(--spacing-03) var(--spacing-04); background:
                // var(--background); font-size: 0.75rem }`, `.g-fp .g-mono {
                // font-size: 0.875rem }`.
                sans(div(), 12., 400)
                    .flex()
                    .items_center()
                    .justify_between()
                    .mt(tokens::SPACING_03)
                    .py(tokens::SPACING_03)
                    .px(tokens::SPACING_04)
                    .bg(color(ColorRole::Background))
                    .child(row::dim(self.texts.fingerprint.clone(), 12.))
                    .child(
                        mono(div(), 14., 400)
                            .whitespace_nowrap()
                            .child(self.fingerprint.clone()),
                    ),
            )
            .child(parts::helper((self.texts.check)(&d)))
            .child(
                // `.g-sheet-check { margin: var(--spacing-03) 0; align-items: flex-start }`.
                // The label column takes the row's remaining width and wraps
                // in it (reference render: 344px in the 416px sheet).
                div().flex().flex_col().my(tokens::SPACING_03).child(
                    InlineCheck::new((id.clone(), "check"))
                        .checked(on)
                        .align_top(true)
                        .label(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .gap(gpui_kit::px(2.))
                                .child(self.texts.control.clone())
                                .child(
                                    row::dim((self.texts.control_detail)(&d), 12.)
                                        .whitespace_normal(),
                                ),
                        )
                        .on_change(move |v, _, cx| {
                            ctrl.update(cx, |c, cx| {
                                *c = v;
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(
                // `.g-sheet-actions { display: grid; grid-template-columns: 1fr
                // 1fr; margin: var(--spacing-05) calc(-1 * var(--spacing-06)) 0 }`.
                div()
                    .flex()
                    .mt(tokens::SPACING_05)
                    .mx(rem(-24.))
                    .child(
                        div().flex_1().min_w_0().child(
                            Button::new((id.clone(), "deny"), self.texts.deny.clone())
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Lg)
                                .full_width(true)
                                .when_some(on_deny, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                        ),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            Button::new((id.clone(), "allow"), self.texts.allow.clone())
                                .variant(ButtonVariant::Primary)
                                .size(ButtonSize::Lg)
                                .full_width(true)
                                .when_some(on_allow, |b, f| {
                                    b.on_click(move |_, w, cx| f(on, w, cx))
                                }),
                        ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_is_m_ss() {
        assert_eq!(countdown(600), "10:00");
        assert_eq!(countdown(61), "1:01");
        assert_eq!(countdown(9), "0:09");
        assert_eq!(QR_SIZE, 168.);
    }

    #[test]
    fn texts_name_the_device() {
        let t = PairConfirmTexts::default();
        assert_eq!(
            (t.title)("iPhone 16"),
            SharedString::from("Pair “iPhone 16”?")
        );
        assert_eq!(
            (t.check)("iPhone 16"),
            SharedString::from("Check it matches the one shown on iPhone 16.")
        );
    }
}
