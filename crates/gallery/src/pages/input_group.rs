//! Forms › InputGroup: the reference groups (search with ⌘K, URL prefix with
//! a Verify button, a composer with a counter and Send) and a live group.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::input_group::{
    AddonAlign, InputGroup, InputGroupAddon, InputGroupInput, InputGroupText, InputGroupTextarea,
    input_group_button,
};
use u0ui::components::{InputState, Kbd, TextareaState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let search = cx.new(|cx| InputState::new(window, cx));
    let url = cx.new(|cx| InputState::new(window, cx).default_value("example.com"));
    let prompt = cx.new(|cx| TextareaState::new(window, cx));
    let live = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| InputGroupPage {
        search,
        url,
        prompt,
        live,
        align: 0,
        invalid: false,
        disabled: false,
        sent: 0,
    })
    .into()
}

struct InputGroupPage {
    search: Entity<InputState>,
    url: Entity<InputState>,
    prompt: Entity<TextareaState>,
    live: Entity<InputState>,
    align: usize,
    invalid: bool,
    disabled: bool,
    sent: usize,
}

impl Render for InputGroupPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut InputGroupPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let count = self.prompt.read(cx).value().chars().count();

        // `components/InputGroup/preview.html` (`cu-stack`, max-width 420).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .max_w(rems(26.25))
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_06)
                    .child(
                        InputGroup::new("ref-search")
                            .addon(
                                InputGroupAddon::new()
                                    .child(Icon::carbon(CarbonIcon::Search).size(IconSize::S16)),
                            )
                            .input(
                                InputGroupInput::new(&self.search).placeholder("Search resources"),
                            )
                            .addon(
                                InputGroupAddon::new()
                                    .align(AddonAlign::InlineEnd)
                                    .child(Kbd::new("⌘K")),
                            ),
                    )
                    .child(
                        InputGroup::new("ref-url")
                            .addon(InputGroupAddon::new().child(InputGroupText::new("https://")))
                            .input(InputGroupInput::new(&self.url))
                            .addon(
                                InputGroupAddon::new()
                                    .align(AddonAlign::InlineEnd)
                                    .child(input_group_button("ref-verify", "Verify")),
                            ),
                    )
                    .child(
                        InputGroup::new("ref-composer")
                            .textarea(
                                InputGroupTextarea::new(&self.prompt).placeholder("Ask anything…"),
                            )
                            .addon(
                                InputGroupAddon::new()
                                    .align(AddonAlign::BlockEnd)
                                    .child(InputGroupText::new(format!("{count} / 500")))
                                    .child(
                                        input_group_button("ref-send", "Send")
                                            .variant(ButtonVariant::Default)
                                            .on_click({
                                                let entity = entity.clone();
                                                move |_, _, cx| {
                                                    entity.update(cx, |this, cx| {
                                                        this.sent += 1;
                                                        cx.notify();
                                                    })
                                                }
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(SharedString::from(format!("Sent: {}", self.sent))),
                    ),
            );

        let align = AddonAlign::ALL[self.align];
        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            div().max_w(rems(26.25)).child(
                InputGroup::new("live")
                    .invalid(self.invalid)
                    .disabled(self.disabled)
                    .input(InputGroupInput::new(&self.live).placeholder("Amount"))
                    .addon(
                        InputGroupAddon::new()
                            .align(align)
                            .child(Icon::carbon(CarbonIcon::Calculator).size(IconSize::S16))
                            .child(InputGroupText::new("USD")),
                    ),
            ),
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "addon align",
                controls::choices(
                    "ig-align",
                    &AddonAlign::ALL.map(|a| a.name()),
                    self.align,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.align = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "ig-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "ig-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}
