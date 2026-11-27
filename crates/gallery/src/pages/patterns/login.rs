//! Carbon · Patterns › PatternLogin: the "Log in" form (title, sign-up
//! link, email first, password on the second step, alternative login),
//! the split layout with marketing on the other side, and the error copy
//! (`components/PatternLogin/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Bounds, Context, Entity, Hsla, IntoElement,
    ParentElement, Pixels, Render, Rgba, SharedString, Styled, Window, canvas,
    component::input::InputState, div, linear_color_stop, linear_gradient,
    prelude::FluentBuilder as _, px, relative, rems,
};
use u0ui::components::{Checkbox, FieldSize, Input};
use u0ui::patterns::Caption;
use u0ui::prelude::*;
use u0ui::typography::paragraph;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let email = crate::stand_in::input_state("", window, cx);
    let password = crate::stand_in::input_state("", window, cx);
    let ref_email = crate::stand_in::input_state("", window, cx);
    // `K.PasswordInput` with no value.
    let ref_password = crate::stand_in::input_state("", window, cx);
    cx.new(|cx| {
        cx.observe(&email, |_, _, cx| cx.notify()).detach();
        cx.observe(&password, |_, _, cx| cx.notify()).detach();
        LoginPage {
            email,
            password,
            ref_email,
            ref_password,
            split: true,
            step: 0,
            error: 0,
            remember: false,
        }
    })
    .into()
}

struct LoginPage {
    email: Entity<InputState>,
    password: Entity<InputState>,
    ref_email: Entity<InputState>,
    ref_password: Entity<InputState>,
    split: bool,
    /// 0: email, 1: password.
    step: usize,
    /// Index into [`ERRORS`].
    error: usize,
    remember: bool,
}

const LAYOUTS: [&str; 2] = ["centered", "split"];
const STEPS: [&str; 2] = ["1 · email", "2 · password"];
const ERRORS: [&str; 3] = ["none", "required", "incorrect"];

/// The error copy: never says which field was wrong.
fn error_text(step: usize, error: usize) -> Option<&'static str> {
    match (error, step) {
        (1, 0) => Some("Email is required"),
        (1, _) => Some("Password is required"),
        (2, _) => Some("Incorrect email or password. Try again."),
        _ => None,
    }
}

/// `hr.cu-phr { border-top: 1px solid var(--border-subtle-01) }`.
fn rule() -> impl IntoElement {
    div()
        .h(px(1.))
        .w_full()
        .bg(color(ColorRole::BorderSubtle01))
}

fn link(id: impl Into<SharedString>, label: &'static str, small: bool) -> Button {
    Button::new(id.into(), label)
        .variant(ButtonVariant::Link)
        .size(if small {
            ButtonSize::Sm
        } else {
            ButtonSize::Md
        })
}

/// `.cu-plogin-sub { font-size: 0.875rem; color: var(--text-primary) }`:
/// a sentence ending in a link. It is a `p`, so the global `p` rule's
/// `line-height: 1.5` stays: 21px lines.
fn sentence(text: &'static str, link: Button) -> impl IntoElement {
    paragraph()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(rems(0.25))
        .text_size(rems(0.875))
        .line_height(rems(0.875 * 1.5))
        .text_color(color(ColorRole::TextPrimary))
        .child(text)
        .child(link)
}

/// A link alone at the row's end (`display: flex; justify-content: flex-end`).
fn end_link(link: Button) -> impl IntoElement {
    div().flex().flex_row().justify_end().child(link)
}

struct Form {
    prefix: &'static str,
    field: AnyElement,
    forgot: &'static str,
    primary: Button,
    remember: Checkbox,
    back: Option<Button>,
    /// The end rule against the marketing side; off once the split row has
    /// wrapped and nothing sits beside the form.
    divider: bool,
}

/// `.cu-plogin-form`: the form column.
fn form(f: Form) -> impl IntoElement {
    let p = f.prefix;
    div()
        .flex()
        .flex_col()
        // `.cu-plogin-form { padding: var(--spacing-07) var(--spacing-06) }`.
        .py(tokens::SPACING_07)
        .px(tokens::SPACING_06)
        // `background: var(--background); border-right: 1px solid var(--border-subtle-01)`.
        .bg(color(ColorRole::Background))
        .when(f.divider, |d| {
            d.border_r_1()
                .border_color(color(ColorRole::BorderSubtle01))
        })
        .child(
            // `.cu-plogin-form h2 { margin: 0 0 var(--spacing-03); font-size: 1.75rem;
            // font-weight: 400; line-height: 1.28 }`.
            div()
                .mb(tokens::SPACING_03)
                .text_size(rems(1.75))
                .line_height(rems(1.75 * 1.28))
                .text_color(color(ColorRole::TextPrimary))
                .child("Log in to [Product]"),
        )
        .child(
            // `.cu-plogin-sub { margin: 0 0 var(--spacing-06) }`.
            div().mb(tokens::SPACING_06).child(sentence(
                "Don’t have an account?",
                link(format!("{p}-create"), "Create an account", false),
            )),
        )
        // `.cu-plogin-form .cu-phr { margin-bottom: var(--spacing-06) }`.
        .child(div().mb(tokens::SPACING_06).child(rule()))
        .children(f.back.map(|b| div().mb(tokens::SPACING_03).child(b)))
        .child(f.field)
        // `margin: 4px 0 24px`.
        .child(
            div()
                .mt(tokens::SPACING_02)
                .mb(tokens::SPACING_06)
                .child(end_link(link(format!("{p}-forgot"), f.forgot, true))),
        )
        .child(f.primary.full_width(true))
        // `margin: 16px 0`.
        .child(div().my(tokens::SPACING_05).child(f.remember))
        .child(rule())
        // `.cu-pcap` with `margin: 16px 0 8px`.
        .child(
            div()
                .mt(tokens::SPACING_05)
                .child(Caption::new("Alternative logins")),
        )
        .child(
            Button::new(
                SharedString::from(format!("{p}-sso")),
                "Log in with single sign-on",
            )
            .variant(ButtonVariant::Tertiary)
            .icon(CarbonIcon::Launch)
            .full_width(true),
        )
        // `marginTop: 24`; it is a `.cu-plogin-sub`, so its `margin: 0 0
        // var(--spacing-06)` bottom margin stays inside the form's padding
        // (the reference frame is 512px, not the 31rem minimum).
        .child(
            div()
                .mt(tokens::SPACING_06)
                .mb(tokens::SPACING_06)
                .child(sentence(
                    "Need help?",
                    link(format!("{p}-support"), "Contact support", false),
                )),
        )
}

/// `.cu-plogin-art { position: absolute; inset: 0 0 45% 0; background:
/// linear-gradient(135deg, var(--charts-1), var(--charts-2) 60%,
/// transparent); opacity: .85 }` over the side's `background-inverse`.
///
/// GPUI gradients take two stops and its `opacity` does not composite a
/// group, so the three stops are two stacked layers of opaque colors
/// pre-blended at 85% over the background: `P2` solid to 60% then fading
/// to a transparent `P2` at 100%, and over it `P1` fading out by 60%. Over
/// 0–60% that is `P1·(1−s) + P2·s`, from 60% `P2` thinning to the
/// background: the CSS result exactly.
fn art() -> [AnyElement; 2] {
    let bg = color(ColorRole::BackgroundInverse);
    let p1 = over(color(ColorRole::Charts1), bg, 0.85);
    let p2 = over(color(ColorRole::Charts2), bg, 0.85);
    let layer = |from: Hsla, from_at: f32, to: Hsla, to_at: f32| {
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(relative(0.55))
            .bg(linear_gradient(
                135.,
                linear_color_stop(from, from_at),
                linear_color_stop(to, to_at),
            ))
            .into_any_element()
    };
    [
        layer(p2, 0.6, p2.opacity(0.), 1.),
        layer(p1, 0., p1.opacity(0.), 0.6),
    ]
}

/// `top` at `alpha` composited over the opaque `bottom`, in sRGB.
fn over(top: Hsla, bottom: Hsla, alpha: f32) -> Hsla {
    let (t, b) = (top.to_rgb(), bottom.to_rgb());
    let mix = |t: f32, b: f32| t * alpha + b * (1. - alpha);
    Rgba {
        r: mix(t.r, b.r),
        g: mix(t.g, b.g),
        b: mix(t.b, b.b),
        a: 1.,
    }
    .into()
}

/// `.cu-plogin-side`: marketing, apart from the form.
fn side() -> impl IntoElement {
    div()
        .relative()
        .flex()
        .flex_col()
        .justify_end()
        .flex_1()
        .min_w(rems(15.))
        .min_h(rems(20.))
        // `padding: var(--spacing-07); overflow: hidden`.
        .p(tokens::SPACING_07)
        .overflow_hidden()
        // `background: var(--background-inverse); color: var(--text-inverse)`.
        .bg(color(ColorRole::BackgroundInverse))
        .text_color(color(ColorRole::TextInverse))
        .children(art())
        .child(
            div()
                .relative()
                .child(
                    // `fontSize: 28, lineHeight: 1.25, maxWidth: 280`.
                    div()
                        .max_w(rems(280. / 16.))
                        .text_size(rems(1.75))
                        .line_height(rems(1.75 * 1.25))
                        .child("Your agents, on every Mac you own."),
                )
                .child(
                    // A `<p>` at `fontSize: 14, opacity: .8`: the global `p`
                    // rule's 1.5 line height (21px) and no margin.
                    paragraph()
                        .opacity(0.8)
                        .text_size(rems(0.875))
                        .line_height(rems(0.875 * 1.5))
                        .child("Marketing content stays on this side, separate from the form."),
                ),
        )
}

fn password_field(id: &'static str, state: &Entity<InputState>, error: Option<&str>) -> Input {
    let input = Input::new(id, state)
        .label("Password")
        .password(true)
        .size(FieldSize::Md);
    match error {
        Some(text) => input.invalid(true).invalid_text(text.to_string()),
        None => input,
    }
}

/// `.cu-plogin-err`: the second step with its error.
fn second_step(
    id: &'static str,
    state: &Entity<InputState>,
    prefix: &'static str,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w(rems(15.))
        .flex_none()
        // `padding: var(--spacing-07) var(--spacing-05); background: var(--layer-01)`.
        .py(tokens::SPACING_07)
        .px(tokens::SPACING_05)
        .bg(color(ColorRole::Layer01))
        .child(Caption::new("第二步与错误"))
        // `.cu-plogin-err` sets no `cds--layer`: `field-01` on `layer-01`.
        .child(password_field(id, state, error_text(1, 2)).layer(LayerLevel::One))
        .child(div().mt(tokens::SPACING_02).child(end_link(link(
            format!("{prefix}-forgot-pw"),
            "Forgot password?",
            true,
        ))))
}

/// `components/PatternLogin/preview.html` (`.cu-plogin { grid-template-columns:
/// 22rem 1fr 15rem; min-height: 31rem }`; the columns wrap when narrow).
pub fn reference(email: &Entity<InputState>, password: &Entity<InputState>) -> impl IntoElement {
    let field = Input::new("lg-ref-email", email)
        .label("Email")
        .placeholder("name@example.com")
        .size(FieldSize::Md)
        .into_any_element();
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .min_h(rems(31.))
        .w_full()
        .child(
            div().w(rems(22.)).flex_none().child(form(Form {
                prefix: "lg-ref",
                field,
                forgot: "Forgot email?",
                primary: Button::new("lg-ref-continue", "Continue")
                    .variant(ButtonVariant::Primary)
                    .icon(CarbonIcon::ArrowRight),
                remember: Checkbox::new("lg-ref-remember").label("Remember email"),
                back: None,
                divider: true,
            })),
        )
        .child(side())
        .child(second_step("lg-ref-password", password, "lg-ref"))
}

/// The overview tile: the title and the first step.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .p(tokens::SPACING_05)
        .bg(color(ColorRole::Background))
        .child(
            div()
                .text_size(rems(1.25))
                .text_color(color(ColorRole::TextPrimary))
                .child("Log in to [Product]"),
        )
        .child(rule())
        .child(
            Button::new("lg-thumb-continue", "Continue")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Sm)
                .icon(CarbonIcon::ArrowRight)
                .full_width(true),
        )
        .into_any_element()
}

impl Render for LoginPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let error = error_text(self.step, self.error);
        let field = if self.step == 0 {
            let input = Input::new("lg-live-email", &self.email)
                .label("Email")
                .placeholder("name@example.com")
                .size(FieldSize::Md);
            match error {
                Some(text) => input.invalid(true).invalid_text(text),
                None => input,
            }
            .into_any_element()
        } else {
            password_field("lg-live-password", &self.password, error).into_any_element()
        };
        let primary = {
            let entity = entity.clone();
            let (label, icon) = if self.step == 0 {
                ("Continue", CarbonIcon::ArrowRight)
            } else {
                ("Log in", CarbonIcon::Login)
            };
            Button::new("lg-live-primary", label)
                .variant(ButtonVariant::Primary)
                .icon(icon)
                .on_click(move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        if this.step == 0 {
                            if this.email.read(cx).value().trim().is_empty() {
                                this.error = 1;
                            } else {
                                this.error = 0;
                                this.step = 1;
                            }
                        } else if this.password.read(cx).value().is_empty() {
                            this.error = 1;
                        } else {
                            // The demo has no account: every password is wrong.
                            this.error = 2;
                        }
                        cx.notify();
                    })
                })
        };
        let remember = {
            let entity = entity.clone();
            Checkbox::new("lg-live-remember")
                .label(if self.step == 0 {
                    "Remember email"
                } else {
                    "Remember me"
                })
                .checked(self.remember)
                .on_checked_change(move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.remember = on;
                        cx.notify();
                    })
                })
        };
        let back = (self.step == 1).then(|| {
            let entity = entity.clone();
            let email: SharedString = self.email.read(cx).value().to_string().into();
            Button::new("lg-live-back", SharedString::from(format!("Not {email}?")))
                .variant(ButtonVariant::Link)
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.step = 0;
                        this.error = 0;
                        cx.notify();
                    })
                })
        });
        let forgot = if self.step == 0 {
            "Forgot email?"
        } else {
            "Forgot password?"
        };
        // The split row wraps when it is narrower than the form (22rem) and
        // the side's 15rem minimum: the side drops under the form, and the
        // form's end rule would face empty space, so it goes.
        let bounds =
            window.use_keyed_state("lg-live-bounds", cx, |_, _| Bounds::<Pixels>::default());
        let width = f32::from(bounds.read(cx).size.width);
        let wrapped = width > 0. && width < f32::from(window.rem_size()) * (22. + 15.);
        let the_form = form(Form {
            prefix: "lg-live",
            field,
            forgot,
            primary,
            remember,
            back,
            divider: !(self.split && wrapped),
        });
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
        let live = div()
            .relative()
            .flex()
            .flex_row()
            .flex_wrap()
            .w_full()
            .min_h(rems(31.))
            .child(measure)
            .when(!self.split, |d| {
                // Centered on the background.
                d.justify_center()
                    .items_center()
                    .py(tokens::SPACING_07)
                    .bg(color(ColorRole::Layer01))
            })
            .child(div().w(rems(22.)).flex_none().child(the_form))
            .when(self.split, |d| d.child(side()));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "layout",
                controls::choices(
                    "lg-layout",
                    &LAYOUTS,
                    usize::from(self.split),
                    controls::setter(&entity, |this: &mut Self, i: usize| this.split = i == 1),
                ),
            ))
            .child(controls::field(
                "step",
                controls::choices(
                    "lg-step",
                    &STEPS,
                    self.step,
                    controls::setter(&entity, |this: &mut Self, i| this.step = i),
                ),
            ))
            .child(controls::field(
                "error",
                controls::choices(
                    "lg-error",
                    &ERRORS,
                    self.error,
                    controls::setter(&entity, |this: &mut Self, i| this.error = i),
                ),
            ));

        Body::new()
            .playground(live)
            .hint("Continue with an empty email shows the required error; any password is rejected with the generic copy")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference(&self.ref_email, &self.ref_password),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_never_say_which_field_is_wrong() {
        assert_eq!(error_text(0, 1), Some("Email is required"));
        assert_eq!(error_text(1, 1), Some("Password is required"));
        assert_eq!(
            error_text(1, 2),
            Some("Incorrect email or password. Try again.")
        );
        assert_eq!(error_text(0, 0), None);
    }
}
