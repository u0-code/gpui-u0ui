//! ConsentSheet: the first-run telemetry consent (`ConsentSheet` in
//! `src/ghost/admin.jsx`, `.g-consent` in `src/ghost.css`): a welcome title,
//! a stepped slider over Off / Basic / Usage / Detailed with each level's
//! description, the live [`TelemetryPreview`] of what that level sends, and
//! two equal full-bleed buttons, "Share nothing" and "Continue" (refusing is
//! never harder than agreeing).
//!
//! ```ignore
//! ConsentSheet::new("consent").app_name("Ghost").on_continue(|level, _, _| {})
//! ```
use super::telemetry_preview::{Payload, TelemetryPreview};
use crate::{
    components::{Button, ButtonSize, ButtonVariant, overlay::floating_shadow},
    ghost::primitives::SteppedSlider,
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};
use std::rc::Rc;

/// The sheet's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ConsentTexts {
    /// `"Welcome to {appName}"`.
    pub title: Rc<dyn Fn(&str) -> SharedString>,
    /// The intro line.
    pub intro: Rc<dyn Fn(&str) -> SharedString>,
    /// The slider's label: `"Telemetry"`.
    pub slider: SharedString,
    /// The four level names.
    pub levels: [SharedString; 4],
    /// The four level descriptions.
    pub describe: [SharedString; 4],
    /// `"Share nothing"`.
    pub share_nothing: SharedString,
    /// `"Continue"`.
    pub proceed: SharedString,
}

impl Default for ConsentTexts {
    fn default() -> Self {
        Self {
            title: Rc::new(|a| format!("Welcome to {a}").into()),
            intro: Rc::new(|a| {
                format!(
                    "Choose what you share to help improve {a}. You can change this any time in Settings › About."
                )
                .into()
            }),
            slider: "Telemetry".into(),
            levels: [
                "Off".into(),
                "Basic".into(),
                "Usage".into(),
                "Detailed".into(),
            ],
            describe: [
                "Nothing leaves this Mac.".into(),
                "An install ID, the app version and which agents are switched on.".into(),
                "Basic, plus anonymous counts of which features are used.".into(),
                "Usage, plus crash reports and performance traces to help fix slow screens.".into(),
            ],
            share_nothing: "Share nothing".into(),
            proceed: "Continue".into(),
        }
    }
}

/// The reference's sample payload for `level` (`{}` at 0; then the app and
/// OS, a crash at ≥ 1, feature counts at ≥ 2, traces at 3).
pub fn sample_payload(level: usize) -> Payload {
    if level == 0 {
        return Payload::Object(Vec::new());
    }
    let mut pairs = vec![
        ("app", Payload::str("1.4.2")),
        ("os", Payload::str("macOS 15.2")),
    ];
    pairs.push((
        "crash",
        Payload::object([
            ("signal", Payload::str("SIGABRT")),
            ("frames", Payload::Number(12.)),
        ]),
    ));
    if level >= 2 {
        pairs.push((
            "features",
            Payload::object([
                ("palette", Payload::Number(14.)),
                ("sideChat", Payload::Number(3.)),
            ]),
        ));
    }
    if level >= 3 {
        pairs.push((
            "traces",
            Payload::Array(vec![Payload::object([
                ("screen", Payload::str("thread")),
                ("ms", Payload::Number(182.)),
            ])]),
        ));
    }
    Payload::object(pairs)
}

type ContinueHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type PayloadFn = Rc<dyn Fn(usize) -> Payload>;

/// The consent sheet. See the module docs.
#[derive(IntoElement)]
pub struct ConsentSheet {
    id: ElementId,
    app_name: SharedString,
    default_level: usize,
    payload: PayloadFn,
    texts: ConsentTexts,
    on_continue: Option<ContinueHandler>,
}

impl ConsentSheet {
    /// The sheet for "Ghost", starting at Usage (level 2).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            app_name: "Ghost".into(),
            default_level: 2,
            payload: Rc::new(sample_payload),
            texts: ConsentTexts::default(),
            on_continue: None,
        }
    }

    /// The app's name (`appName`).
    pub fn app_name(mut self, name: impl Into<SharedString>) -> Self {
        self.app_name = name.into();
        self
    }

    /// The level the slider starts at (the reference: 2).
    pub fn default_level(mut self, level: usize) -> Self {
        self.default_level = level.min(3);
        self
    }

    /// What each level sends (default: [`sample_payload`]).
    pub fn payload(mut self, f: impl Fn(usize) -> Payload + 'static) -> Self {
        self.payload = Rc::new(f);
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ConsentTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A button was pressed (`onContinue(level)`; "Share nothing" passes 0).
    pub fn on_continue(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_continue = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ConsentSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let start = self.default_level;
        let level = window.use_keyed_state((id.clone(), "level"), cx, |_, _| start);
        let lvl = *level.read(cx);
        let describe = self.texts.describe.clone();
        let set_level = level.clone();
        let (nothing, proceed) = (self.on_continue.clone(), self.on_continue.clone());
        // `.g-consent { display: flex; flex-direction: column; gap:
        // var(--spacing-05); max-width: 32rem; padding: var(--spacing-07)
        // var(--spacing-06) 0; background: var(--layer-01); box-shadow: 0 2px
        // 6px var(--shadow) }`.
        sans(div(), 14., 400)
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(rem(512.))
            .pt(tokens::SPACING_07)
            .px(tokens::SPACING_06)
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .text_color(color(ColorRole::TextPrimary))
            .child(
                // `.g-consent h2 { margin: 0; font-size: 1.5rem; font-weight: 400 }`.
                sans(div(), 24., 400).child((self.texts.title)(&self.app_name)),
            )
            .child(
                // `.g-consent > p { margin: calc(-1 * var(--spacing-03)) 0 0;
                // font-size: 0.875rem }` in `.g-dim`; Carbon's global `p {
                // line-height: 1.5 }` makes the lines 21px (reference render).
                sans(div(), 14., 400)
                    .line_height(rem(21.))
                    .mt(rem(-8.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child((self.texts.intro)(&self.app_name)),
            )
            .child(
                SteppedSlider::new((id.clone(), "slider"), self.texts.levels.clone())
                    .label(self.texts.slider.clone())
                    .value(lvl)
                    .describe(move |i| describe.get(i).cloned().unwrap_or_default())
                    .on_value_change(move |v, _, cx| {
                        set_level.update(cx, |l, cx| {
                            *l = v;
                            cx.notify();
                        })
                    }),
            )
            .child(TelemetryPreview::new(
                (id.clone(), "tele"),
                lvl as u32,
                (self.payload)(lvl),
            ))
            .child(
                // `.g-consent-actions { display: grid; grid-template-columns: 1fr
                // 1fr; margin: var(--spacing-03) calc(-1 * var(--spacing-06)) 0 }`.
                div()
                    .flex()
                    .mt(tokens::SPACING_03)
                    .mx(rem(-24.))
                    .child(
                        div().flex_1().min_w_0().child(
                            Button::new((id.clone(), "nothing"), self.texts.share_nothing.clone())
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Xl)
                                .full_width(true)
                                .on_click(move |_, w, cx| {
                                    if let Some(f) = &nothing {
                                        f(0, w, cx);
                                    }
                                }),
                        ),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            Button::new((id.clone(), "continue"), self.texts.proceed.clone())
                                .variant(ButtonVariant::Primary)
                                .size(ButtonSize::Xl)
                                .full_width(true)
                                .on_click(move |_, w, cx| {
                                    if let Some(f) = &proceed {
                                        f(lvl, w, cx);
                                    }
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
    fn payload_grows_with_the_level() {
        assert_eq!(sample_payload(0).to_pretty(), "{}");
        let keys = |v: Payload| match v {
            Payload::Object(p) => p
                .into_iter()
                .map(|(k, _)| k.to_string())
                .collect::<Vec<_>>(),
            _ => vec![],
        };
        assert_eq!(keys(sample_payload(1)), ["app", "os", "crash"]);
        assert_eq!(keys(sample_payload(2)), ["app", "os", "crash", "features"]);
        assert_eq!(
            keys(sample_payload(3)),
            ["app", "os", "crash", "features", "traces"]
        );
    }

    #[test]
    fn texts_name_the_app() {
        let t = ConsentTexts::default();
        assert_eq!((t.title)("Ghost"), SharedString::from("Welcome to Ghost"));
        assert_eq!(t.levels.len(), 4);
    }
}
