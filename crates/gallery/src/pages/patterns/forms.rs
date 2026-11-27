//! Carbon · Patterns › PatternForms: a page form (fields 32px apart, short
//! related fields side by side, buttons at the start 48px below) and a
//! dialog form (fields 24px apart, full-bleed buttons with the primary at
//! the end, an error that says how to fix it). Live: the fields validate as
//! you type and the actions say what they would do.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, rems,
};
use u0ui::components::{Input, RadioGroup, RadioGroupItem};
use u0ui::patterns::{Caption, FrameButton, PatternFrame, PatternGrid};
use u0ui::prelude::*;

const EXAMPLES: [&str; 2] = ["page", "dialog"];
const CAPTIONS: [&str; 2] = [
    "页面表单：字段间距 32px，按钮左对齐，距最后一个字段 48px",
    "对话框 / 面板里：字段间距 24px，按钮贴底满出血、主按钮在右",
];
const CONNECT: [(&str, &str); 3] = [("ssh", "SSH"), ("tun", "Tunnel"), ("none", "None")];
const SPACINGS: [&str; 2] = ["regular", "compact"];

/// `.cu-pcols { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 6rem }`, in rem.
const PORT_COLUMN: f32 = 6.;
/// `marginTop: 48` above the page form's buttons.
const BUTTONS_GAP: tokens::TokenLength = tokens::SPACING_09;

/// The branch rule the dialog checks.
const BRANCH_ERROR: &str = "Branch names can’t contain spaces. Use - instead.";

struct Fields {
    name: Entity<InputState>,
    host: Entity<InputState>,
    port: Entity<InputState>,
    user: Entity<InputState>,
    branch: Entity<InputState>,
    bad_branch: Entity<InputState>,
}

impl Fields {
    fn new(window: &mut Window, cx: &mut App) -> Self {
        let field = |value: &'static str, window: &mut Window, cx: &mut App| {
            cx.new(|cx| InputState::new(window, cx).default_value(value))
        };
        Self {
            name: field("build-box", window, cx),
            host: field("", window, cx),
            port: field("22", window, cx),
            user: field("", window, cx),
            branch: field("feat/dark-tokens", window, cx),
            bad_branch: field("feat/dark tokens", window, cx),
        }
    }
}

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let live = Fields::new(window, cx);
    let reference = Fields::new(window, cx);
    cx.new(|cx| {
        for f in [&live.name, &live.host, &live.port, &live.user, &live.branch] {
            cx.observe(f, |_, _, cx| cx.notify()).detach();
        }
        Page {
            live,
            reference,
            example: 0,
            connect: "ssh".into(),
            spacing: 0,
            checked: false,
            result: None,
        }
    })
    .into()
}

struct Page {
    live: Fields,
    reference: Fields,
    example: usize,
    connect: SharedString,
    /// Dialog field spacing: regular 24px or compact 16px.
    spacing: usize,
    /// Add server was pressed: empty required fields show their errors.
    checked: bool,
    result: Option<SharedString>,
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// A port is a number from 1 to 65535 (`min: 1, max: 65535`).
fn port_error(port: &str) -> Option<&'static str> {
    match port.trim().parse::<u32>() {
        Ok(1..=65535) => None,
        _ => Some("Use a port from 1 to 65535"),
    }
}

/// A branch name may not contain whitespace.
fn branch_error(name: &str) -> Option<&'static str> {
    name.chars()
        .any(char::is_whitespace)
        .then_some(BRANCH_ERROR)
}

/// The page form on `layer-01`, 24px padding: the title, the fields 32px
/// apart, then Add server and Cancel 48px below.
fn page_form(p: &str, f: &Fields, live: Option<(&Page, &Entity<Page>)>, cx: &App) -> AnyElement {
    let checked = live.is_some_and(|(pg, _)| pg.checked);
    let host_empty = f.host.read(cx).value().trim().is_empty();
    let port = f.port.read(cx).value().to_string();
    let port_err = live.and(port_error(&port));
    let mut radios = RadioGroup::new(id(p, "connect")).legend("Connect through");
    for (value, label) in CONNECT {
        radios = radios.item(RadioGroupItem::new(value).label(label));
    }
    radios = match live {
        Some((pg, e)) => {
            let e = e.clone();
            radios
                .value(pg.connect.clone())
                .on_value_change(move |v, _, cx| {
                    let v = v.clone();
                    e.update(cx, |this, cx| {
                        this.connect = v;
                        cx.notify();
                    })
                })
        }
        None => radios.default_value("ssh"),
    };
    let mut add = Button::new(id(p, "add"), "Add server").variant(ButtonVariant::Primary);
    let mut cancel = Button::new(id(p, "cancel"), "Cancel").variant(ButtonVariant::Ghost);
    if let Some((_, e)) = live {
        let (e1, e2) = (e.clone(), e.clone());
        add = add.on_click(move |_, _, cx| {
            e1.update(cx, |this, cx| {
                this.checked = true;
                let host = this.live.host.read(cx).value().trim().to_string();
                let port = this.live.port.read(cx).value().to_string();
                this.result = (!host.is_empty() && port_error(&port).is_none()).then(|| {
                    let name = this.live.name.read(cx).value().to_string();
                    format!("Adding {name} at {host}:{}", port.trim()).into()
                });
                cx.notify();
            })
        });
        cancel = cancel.on_click(move |_, window, cx| {
            e2.update(cx, |this, cx| {
                let f = &this.live;
                for (state, value) in [
                    (&f.name, "build-box"),
                    (&f.host, ""),
                    (&f.port, "22"),
                    (&f.user, ""),
                ] {
                    state.update(cx, |s, cx| s.set_value(value, window, cx));
                }
                this.connect = "ssh".into();
                this.checked = false;
                this.result = None;
                cx.notify();
            })
        });
    }
    div()
        .w_full()
        .bg(color(ColorRole::Layer01))
        .p(tokens::SPACING_06)
        .child(
            // `h3 { fontSize: 20; fontWeight: 400; margin: 0 0 24px }`.
            div()
                .mb(tokens::SPACING_06)
                .type_style(TypeStyle::Heading03)
                .text_color(color(ColorRole::TextPrimary))
                .child("Add a server"),
        )
        .child(
            // `Stack gap={7}`: 32px.
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(
                    // `.cu-pcols { gap: var(--spacing-05) }`.
                    div()
                        .flex()
                        .items_start()
                        .gap(tokens::SPACING_05)
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(Input::new(id(p, "name"), &f.name).label("Name")),
                        )
                        .child(
                            div().flex_1().min_w_0().child(
                                Input::new(id(p, "host"), &f.host)
                                    .label("Host")
                                    .placeholder("192.168.1.20")
                                    .invalid(checked && host_empty)
                                    .invalid_text("Enter the server’s address, e.g. 192.168.1.20"),
                            ),
                        )
                        .child(
                            // `NumberInput hideSteppers`: a plain field.
                            div().w(rems(PORT_COLUMN)).flex_none().child(
                                Input::new(id(p, "port"), &f.port)
                                    .label("Port")
                                    .invalid(port_err.is_some())
                                    .invalid_text(port_err.unwrap_or_default()),
                            ),
                        ),
                )
                .child(
                    Input::new(id(p, "user"), &f.user)
                        .label("User (optional)")
                        .helper_text("Defaults to your Mac user name"),
                )
                .child(radios),
        )
        .child(div().flex().mt(BUTTONS_GAP).child(add).child(cancel))
        .into_any_element()
}

/// The dialog form: Rename branch, two New name fields (the second shows
/// the error), Cancel / Rename branch.
fn dialog_form(
    p: &str,
    f: &Fields,
    gap: tokens::TokenLength,
    live: Option<&Entity<Page>>,
    cx: &App,
) -> AnyElement {
    let branch = f.branch.read(cx).value().to_string();
    let live_err = branch_error(&branch);
    let mut frame = PatternFrame::new(id(p, "frame"))
        .title("Rename branch")
        .close_label("Close")
        .width(None)
        .button(FrameButton::new("Cancel", ButtonVariant::Secondary))
        .button(
            FrameButton::new("Rename branch", ButtonVariant::Primary)
                .disabled(live.is_some() && live_err.is_some()),
        );
    if let Some(e) = live {
        let e = e.clone();
        frame = frame.on_button(move |i, _, cx| {
            e.update(cx, |this, cx| {
                let name = this.live.branch.read(cx).value().to_string();
                this.result = Some(if i == 1 {
                    format!("Renaming to {name}").into()
                } else {
                    "Cancelled".into()
                });
                cx.notify();
            })
        });
    }
    let fields = div().flex().flex_col().gap(gap).child(
        Input::new(id(p, "branch"), &f.branch)
            .label("New name")
            .invalid(live.is_some() && live_err.is_some())
            .invalid_text(live_err.unwrap_or_default()),
    );
    // The reference shows the error on a second, already-invalid field.
    let fields = if live.is_none() {
        fields.child(
            Input::new(id(p, "bad"), &f.bad_branch)
                .label("New name")
                .invalid(true)
                .invalid_text(BRANCH_ERROR),
        )
    } else {
        fields
    };
    frame.child(fields).into_any_element()
}

fn captioned(i: usize, body: AnyElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[i]))
        .child(body)
        .into_any_element()
}

/// The overview tile: the invalid branch field.
pub fn thumbnail(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("forms-thumb-branch", cx, |window, cx| {
        InputState::new(window, cx).default_value("feat/dark tokens")
    });
    div()
        .w(rems(15.))
        .child(
            Input::new("forms-thumb", &state)
                .label("New name")
                .invalid(true)
                .invalid_text(BRANCH_ERROR),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let gap = if self.spacing == 0 {
            tokens::SPACING_06
        } else {
            tokens::SPACING_05
        };
        let body = if self.example == 0 {
            page_form("forms-live", &self.live, Some((self, &entity)), cx)
        } else {
            dialog_form("forms-live", &self.live, gap, Some(&entity), cx)
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(rems(36.))
            .child(captioned(self.example, body))
            .children(self.result.clone().map(|r| {
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(r)
            }));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "example",
                controls::choices(
                    "forms-example",
                    &EXAMPLES,
                    self.example,
                    controls::setter(&entity, |this: &mut Page, i| this.example = i),
                ),
            ))
            .child(controls::field(
                "dialog spacing",
                controls::choices(
                    "forms-spacing",
                    &SPACINGS,
                    self.spacing,
                    controls::setter(&entity, |this: &mut Page, i| this.spacing = i),
                ),
            ))
            .child(controls::toggle(
                "forms-checked",
                "show required errors",
                self.checked,
                controls::setter(&entity, |this: &mut Page, v| this.checked = v),
            ));

        let grid = PatternGrid::tracks(vec![1.3, 1.])
            .align_start(true)
            .cell(captioned(
                0,
                page_form("forms-ref0", &self.reference, None, cx),
            ))
            .cell(captioned(
                1,
                dialog_form("forms-ref1", &self.reference, tokens::SPACING_06, None, cx),
            ));

        Body::new()
            .playground(live)
            .hint("Leave Host empty and press Add server; type a space in the branch name")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_say_how_to_fix() {
        assert_eq!(port_error("22"), None);
        assert!(port_error("0").is_some());
        assert!(port_error("70000").is_some());
        assert!(port_error("ssh").is_some());
        assert_eq!(branch_error("feat/dark-tokens"), None);
        assert_eq!(branch_error("feat/dark tokens"), Some(BRANCH_ERROR));
    }

    #[test]
    fn spacing_matches_the_readme() {
        assert_eq!(BUTTONS_GAP.px_value(), 48.);
        assert_eq!(tokens::SPACING_07.px_value(), 32.);
        assert_eq!(PORT_COLUMN, 6.);
    }
}
