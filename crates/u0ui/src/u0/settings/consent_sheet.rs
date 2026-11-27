//! ConsentSheet: the first-run telemetry consent (`ConsentSheet` in
//! `src/ghost/admin.jsx`, `.g-consent` in `src/ghost.css`): a welcome title,
//! a stepped slider over Off / Basic / Usage / Detailed with each level's
//! description, the live [`TelemetryPreview`] of what that level sends, and
//! two equal full-bleed buttons, "Share nothing" and "Continue" (refusing is
//! never harder than agreeing).
//!
//! The [`ConsentActions::Dialog`] variant is the sheet as u0's Dialogs board
//! draws it in a modal: the slider starts at the first level that shares
//! (level 0 is the refusing button alone), and the two buttons are 32px,
//! at the right of a dialog footer, the proceeding one naming the level
//! (`Continue with Basic`).
//!
//! ```ignore
//! ConsentSheet::new("consent").app_name("u0").on_continue(|level, _, _| {})
//! ```
use super::telemetry_preview::{Payload, TelemetryPreview};
use crate::{
    components::{Button, ButtonSize, ButtonVariant, overlay::floating_shadow},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::SteppedSlider,
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
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
    /// The proceeding button, given the chosen level's name: `"Continue"`.
    pub proceed: Rc<dyn Fn(&str) -> SharedString>,
}

/// Where the sheet's two buttons stand, and how the slider begins.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConsentActions {
    /// `.g-consent-actions`: two equal full-bleed 64px halves under a
    /// slider over every level.
    #[default]
    FullBleed,
    /// `.dlg-consent .g-consent-actions.dlg-foot` (u0's Dialogs board):
    /// 32px buttons at the right (`justify-content: flex-end; gap: 8px;
    /// padding: 0 16px 16px`), and a slider over the levels above 0, which
    /// the refusing button alone chooses.
    Dialog,
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
            proceed: Rc::new(|_| "Continue".into()),
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
    actions: ConsentActions,
    on_continue: Option<ContinueHandler>,
}

impl ConsentSheet {
    /// The sheet for "u0", starting at Usage (level 2).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            app_name: "u0".into(),
            default_level: 2,
            payload: Rc::new(sample_payload),
            texts: ConsentTexts::default(),
            actions: ConsentActions::default(),
            on_continue: None,
        }
    }

    /// Where the buttons stand (default [`ConsentActions::FullBleed`]).
    pub fn actions(mut self, actions: ConsentActions) -> Self {
        self.actions = actions;
        self
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
        let dialog = self.actions == ConsentActions::Dialog;
        // The first level the slider offers: with the dialog's footer, level
        // 0 is the refusing button's alone.
        let first = usize::from(dialog);
        let start = self.default_level.max(first);
        let level = window.use_keyed_state((id.clone(), "level"), cx, |_, _| start);
        let lvl = *level.read(cx);
        let describe = self.texts.describe.clone();
        let steps = self.texts.levels[first..].to_vec();
        let proceed_label = (self.texts.proceed)(&self.texts.levels[lvl]);
        let set_level = level.clone();
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
            // `.dlg-consent .g-consent { padding: 24px 16px 0 }`, its
            // shadow the dialog's (`box-shadow: none`).
            .pt(if dialog {
                tokens::SPACING_06
            } else {
                tokens::SPACING_07
            })
            .px(if dialog {
                tokens::SPACING_05
            } else {
                tokens::SPACING_06
            })
            .bg(color(ColorRole::Layer01))
            .when(!dialog, |d| d.shadow(floating_shadow()))
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
                SteppedSlider::new((id.clone(), "slider"), steps)
                    .label(self.texts.slider.clone())
                    .value(lvl - first)
                    .describe(move |i| describe.get(i + first).cloned().unwrap_or_default())
                    .on_value_change(move |v, _, cx| {
                        set_level.update(cx, |l, cx| {
                            *l = v + first;
                            cx.notify();
                        })
                    }),
            )
            .child(TelemetryPreview::new(
                (id.clone(), "tele"),
                lvl as u32,
                (self.payload)(lvl),
            ))
            .child(actions(
                &id,
                self.actions,
                self.texts.share_nothing.clone(),
                proceed_label,
                lvl,
                self.on_continue.clone(),
            ))
    }
}

/// The two buttons, as `variant` sets them: "Share nothing" passes 0,
/// the proceeding one `lvl`.
fn actions(
    id: &ElementId,
    variant: ConsentActions,
    nothing: SharedString,
    proceed: SharedString,
    lvl: usize,
    on_continue: Option<ContinueHandler>,
) -> gpui_kit::Div {
    let (on_nothing, on_proceed) = (on_continue.clone(), on_continue);
    let dialog = variant == ConsentActions::Dialog;
    let size = if dialog {
        ButtonSize::Sm
    } else {
        ButtonSize::Xl
    };
    let nothing = Button::new((id.clone(), "nothing"), nothing)
        .variant(ButtonVariant::Secondary)
        .size(size)
        .full_width(!dialog)
        .on_click(move |_, w, cx| {
            if let Some(f) = &on_nothing {
                f(0, w, cx);
            }
        });
    let proceed = Button::new((id.clone(), "continue"), proceed)
        .variant(ButtonVariant::Primary)
        .size(size)
        .full_width(!dialog)
        .on_click(move |_, w, cx| {
            if let Some(f) = &on_proceed {
                f(lvl, w, cx);
            }
        });
    if dialog {
        // `.dlg-foot { display: flex; justify-content: flex-end; gap: 8px;
        // padding: 0 16px 16px }`, in `.dlg-consent .g-consent-actions {
        // margin: 8px -16px 0 }` under the sheet's 16px sides.
        return div()
            .flex()
            .justify_end()
            .gap(tokens::SPACING_03)
            .mt(tokens::SPACING_03)
            .mx(rem(-16.))
            .px(tokens::SPACING_05)
            .pb(tokens::SPACING_05)
            .child(nothing)
            .child(proceed);
    }
    // `.g-consent-actions { display: grid; grid-template-columns: 1fr
    // 1fr; margin: var(--spacing-03) calc(-1 * var(--spacing-06)) 0 }`.
    div()
        .flex()
        .mt(tokens::SPACING_03)
        .mx(rem(-24.))
        .child(div().flex_1().min_w_0().child(nothing))
        .child(div().flex_1().min_w_0().child(proceed))
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
        assert_eq!((t.title)("u0"), SharedString::from("Welcome to u0"));
        assert_eq!(t.levels.len(), 4);
        assert_eq!((t.proceed)("Basic"), SharedString::from("Continue"));
    }
}
