//! Carbon · Patterns › PatternDisabledStates: disabled while a condition is
//! unmet (with the inline warning that says how to unlock it), disabled
//! menu items, and hidden when there is no permission. The live example
//! flips the condition: sign in, become the owner.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::alert::{Alert, AlertVariant};
use u0ui::patterns::{Caption, PatternGrid, PatternMenu, PatternMenuItem, PatternRow};
use u0ui::prelude::*;

const EXAMPLES: [&str; 3] = ["disabled", "menu", "hidden"];

const CAPTIONS: [&str; 3] = [
    "Disabled：临时不可用，保持可见；上方说明如何启用",
    "菜单项禁用：没有悬停、光标为禁止",
    "Hidden：没有权限时直接不显示（如非所有者看不到 “Add member”）",
];

/// The reference thread menu (`[label, reason when disabled]`).
const MENU: [(&str, Option<&str>); 4] = [
    ("Rename", None),
    ("Fork from here", None),
    ("Create pull request", Some("Needs a pushed branch")),
    ("Archive", None),
];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        example: 0,
        signed_in: false,
        pushed: false,
        owner: false,
        last: None,
    })
    .into()
}

struct Page {
    example: usize,
    signed_in: bool,
    pushed: bool,
    owner: bool,
    last: Option<&'static str>,
}

/// The conditions the examples read.
#[derive(Clone, Copy)]
struct State {
    signed_in: bool,
    pushed: bool,
    owner: bool,
}

impl State {
    const REFERENCE: Self = Self {
        signed_in: false,
        pushed: false,
        owner: false,
    };
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// Disabled: the warning, then the disabled primary 16px below.
fn disabled(p: &str, s: State) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .when(!s.signed_in, |d| {
            d.child(
                Alert::new(id(p, "alert"))
                    .variant(AlertVariant::Warning)
                    .low_contrast(true)
                    .title("Sign in to Codex")
                    .description("to start a thread with it."),
            )
        })
        .child(
            // `.cds--btn` is `inline-flex`: its natural width, not the
            // column's (a block child would stretch to `max-inline-size: 20rem`).
            div().mt(tokens::SPACING_05).flex().child(
                Button::new(id(p, "new"), "New Codex thread")
                    .variant(ButtonVariant::Primary)
                    .icon(Icon::carbon(CarbonIcon::Add))
                    .disabled(!s.signed_in),
            ),
        )
        .into_any_element()
}

/// The menu with its disabled item.
fn menu(p: &str, s: State, on_select: Option<Entity<Page>>) -> AnyElement {
    let mut menu = PatternMenu::new(id(p, "menu"));
    for (label, reason) in MENU {
        let item = PatternMenuItem::new(label);
        menu = menu.item(match reason {
            Some(reason) if !s.pushed => item.disabled(reason),
            _ => item,
        });
    }
    if let Some(entity) = on_select {
        menu = menu.on_select(move |i, _, cx| {
            entity.update(cx, |this, cx| {
                this.last = Some(MENU[i].0);
                cx.notify();
            })
        });
    }
    menu.into_any_element()
}

/// Hidden: the members list; the owner sees "Add member", others see the
/// dashed note where it would be (`.cu-pghost`).
fn hidden(p: &str, s: State) -> AnyElement {
    let action = if s.owner {
        Button::new(id(p, "add"), "Add member")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .icon(Icon::carbon(CarbonIcon::Add))
            .into_any_element()
    } else {
        // `.cu-pghost { padding: 0 var(--spacing-03); border: 1px dashed
        // var(--border-strong-01); font-size: 0.75rem; color: var(--text-helper);
        // line-height: 1.5rem }`.
        div()
            .px(tokens::SPACING_03)
            .border_1()
            .border_dashed()
            .border_color(color(ColorRole::BorderStrong01))
            .type_style(TypeStyle::Label01)
            .line_height(rems(1.5))
            .text_color(color(ColorRole::TextHelper))
            .child("Add member — hidden")
            .into_any_element()
    };
    div()
        .bg(color(ColorRole::Layer01))
        .p(tokens::SPACING_05)
        .child(
            // `marginBottom: 12`.
            div()
                .flex()
                .justify_between()
                .items_center()
                .mb(tokens::SPACING_04)
                .child(
                    div()
                        .type_style(TypeStyle::HeadingCompact01)
                        .text_color(color(ColorRole::TextPrimary))
                        .child("Members"),
                )
                .child(action),
        )
        .children(["Ana · Owner", "Ben · Editor"].map(|m| PatternRow::new().child(div().child(m))))
        .into_any_element()
}

fn example(i: usize, p: &str, s: State, entity: Option<Entity<Page>>) -> AnyElement {
    let body = match i {
        0 => disabled(p, s),
        1 => menu(p, s, entity),
        _ => hidden(p, s),
    };
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[i.min(2)]))
        .child(body)
        .into_any_element()
}

/// The overview tile: the menu.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(menu("dis-thumb", State::REFERENCE, None))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let state = State {
            signed_in: self.signed_in,
            pushed: self.pushed,
            owner: self.owner,
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(rems(24.))
            .child(example(
                self.example,
                "dis-live",
                state,
                Some(entity.clone()),
            ))
            .when_some(self.last.filter(|_| self.example == 1), |d, last| {
                d.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("Selected: {last}")),
                )
            });

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "example",
                controls::choices(
                    "dis-example",
                    &EXAMPLES,
                    self.example,
                    controls::setter(&entity, |this: &mut Page, i| this.example = i),
                ),
            ))
            .child(controls::toggle(
                "dis-signed-in",
                "signed in",
                self.signed_in,
                controls::setter(&entity, |this: &mut Page, v| this.signed_in = v),
            ))
            .child(controls::toggle(
                "dis-pushed",
                "branch pushed",
                self.pushed,
                controls::setter(&entity, |this: &mut Page, v| this.pushed = v),
            ))
            .child(controls::toggle(
                "dis-owner",
                "owner",
                self.owner,
                controls::setter(&entity, |this: &mut Page, v| this.owner = v),
            ));

        let mut grid = PatternGrid::new(3);
        for i in 0..EXAMPLES.len() {
            grid = grid.cell(example(i, &format!("dis-ref{i}"), State::REFERENCE, None));
        }

        Body::new()
            .playground(live)
            .hint("Flip the conditions: the disabled button, item and hidden action follow")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
