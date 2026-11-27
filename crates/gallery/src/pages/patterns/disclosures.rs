//! Carbon · Patterns › PatternDisclosures: the profile menu (avatar button →
//! panel), the settings / filter menu (checkboxes, radios, Reset / Apply) and
//! the combo button. Live: the triggers open and close their panels (one at a
//! time), the filters reset and apply, the combo menu runs its actions.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, CursorStyle, Entity,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement, Render, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::form::{self, inset_outline};
use u0ui::components::{
    Avatar, AvatarColor, AvatarFallback, Checkbox, MenuItem, RadioGroup, RadioGroupItem,
    RadioOrientation,
};
use u0ui::patterns::{
    Caption, ComboButton, ComboButtonSize, PatternGrid, PatternLink, PatternPopover, PatternRule,
};
use u0ui::prelude::*;

const EXAMPLES: [&str; 3] = ["profile", "settings", "combo"];

const CAPTIONS: [&str; 3] = [
    "Profile 菜单：头像按钮 → 弹层",
    "设置 / 筛选菜单：复选、单选，底部 Reset / Apply",
    "Combo button：文字主操作 + 打开相关操作菜单的图标按钮",
];

const LINKS: [&str; 3] = ["Profile", "Settings", "Keyboard shortcuts"];
const STATUS: [&str; 3] = ["Working", "Needs you", "Done"];
/// `defaultChecked` of the status boxes.
const STATUS_DEFAULT: [bool; 3] = [true, true, false];
const SORT: [(&str, &str); 2] = [("r", "Recent"), ("n", "Name")];
const COMBO: [&str; 3] = ["Commit & push", "Commit & create PR", "Amend last commit"];
const SIZES: [&str; 3] = ["sm", "md", "lg"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        example: 0,
        open: Some(0),
        status: STATUS_DEFAULT,
        sort: "r".into(),
        applied: None,
        size: 2,
        disabled: false,
        last: None,
    })
    .into()
}

struct Page {
    example: usize,
    /// The one open panel (`一次只开一个`).
    open: Option<usize>,
    status: [bool; 3],
    sort: SharedString,
    applied: Option<SharedString>,
    size: usize,
    disabled: bool,
    last: Option<SharedString>,
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// The profile panel (`.cu-ppop`): avatar and name, a rule, the links, a
/// rule, Log out.
fn profile(p: &str, entity: Option<&Entity<Page>>) -> AnyElement {
    let link = |name: &'static str| {
        let link = PatternLink::new(id(p, name), name);
        match entity {
            Some(e) => {
                let e = e.clone();
                link.on_click(move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.last = Some(name.into());
                        this.open = None;
                        cx.notify();
                    })
                })
            }
            None => link,
        }
    };
    PatternPopover::new()
        .child(
            // `display: flex; gap: 12; align-items: center; padding: 16`.
            div()
                .flex()
                .items_center()
                .gap(tokens::SPACING_04)
                .p(tokens::SPACING_05)
                .child(Avatar::new().fallback(AvatarFallback::new("AN").color(AvatarColor::Purple)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .child(
                            div()
                                .type_style(TypeStyle::HeadingCompact01)
                                .child("Ana Ng"),
                        )
                        .child(
                            // `.g-dim { color: var(--text-secondary) }`, `fontSize: 12`.
                            div()
                                .type_style(TypeStyle::Label01)
                                .text_color(color(ColorRole::TextSecondary))
                                .child("ana@example.com"),
                        ),
                ),
        )
        .child(PatternRule::new())
        .children(LINKS.map(link))
        .child(PatternRule::new())
        .child(link("Log out"))
        .into_any_element()
}

/// The settings / filter panel: a checkbox group and a radio group 16px
/// apart in 16px padding, then Reset / Apply sharing the width.
fn settings(p: &str, page: Option<(&Page, &Entity<Page>)>) -> AnyElement {
    let status = page.map_or(STATUS_DEFAULT, |(pg, _)| pg.status);
    let sort: SharedString = page.map_or("r".into(), |(pg, _)| pg.sort.clone());
    let boxes = STATUS.iter().enumerate().map(|(i, label)| {
        let b = Checkbox::new(id(p, &format!("s{i}"))).label(*label);
        match page {
            Some((_, e)) => {
                let e = e.clone();
                b.checked(status[i]).on_checked_change(move |v, _, cx| {
                    e.update(cx, |this, cx| {
                        this.status[i] = v;
                        cx.notify();
                    })
                })
            }
            None => b.default_checked(status[i]),
        }
    });
    let mut radios = RadioGroup::new(id(p, "sort"))
        .legend("Sort by")
        .orientation(RadioOrientation::Vertical);
    for (value, label) in SORT {
        radios = radios.item(RadioGroupItem::new(value).label(label));
    }
    radios = match page {
        Some((_, e)) => {
            let e = e.clone();
            radios.value(sort).on_value_change(move |v, _, cx| {
                let v = v.clone();
                e.update(cx, |this, cx| {
                    this.sort = v;
                    cx.notify();
                })
            })
        }
        None => radios.default_value(sort),
    };
    let reset = Button::new(id(p, "reset"), "Reset")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Lg)
        .full_width(true);
    let apply = Button::new(id(p, "apply"), "Apply")
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Lg)
        .full_width(true);
    let (reset, apply) = match page {
        Some((_, e)) => {
            let (r, a) = (e.clone(), e.clone());
            (
                reset.on_click(move |_, _, cx| {
                    r.update(cx, |this, cx| {
                        this.status = STATUS_DEFAULT;
                        this.sort = "r".into();
                        this.applied = None;
                        cx.notify();
                    })
                }),
                apply.on_click(move |_, _, cx| {
                    a.update(cx, |this, cx| {
                        let picked: Vec<&str> = STATUS
                            .iter()
                            .zip(this.status)
                            .filter_map(|(l, on)| on.then_some(*l))
                            .collect();
                        let sort = SORT
                            .iter()
                            .find(|(v, _)| *v == this.sort.as_ref())
                            .map_or("", |(_, l)| l);
                        this.applied = Some(format!("{} · {sort}", picked.join(", ")).into());
                        this.open = None;
                        cx.notify();
                    })
                }),
            )
        }
        None => (reset, apply),
    };
    PatternPopover::new()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .p(tokens::SPACING_05)
                .child(
                    // `CheckboxGroup legendText="Status"`: a `.cds--label` legend.
                    div()
                        .flex()
                        .flex_col()
                        .child(form::label("Status", false))
                        .children(boxes.map(|b| {
                            // `.cds--form-item.cds--checkbox-wrapper { margin-block-end: 0.375rem }`.
                            div().mb(rems(0.375)).child(b)
                        })),
                )
                .child(radios),
        )
        .child(
            div()
                .flex()
                .child(div().flex_1().child(reset))
                .child(div().flex_1().child(apply)),
        )
        .into_any_element()
}

fn combo(
    p: &str,
    size: ComboButtonSize,
    disabled: bool,
    entity: Option<&Entity<Page>>,
) -> AnyElement {
    let pick = |label: &'static str| {
        let item = MenuItem::item(label);
        match entity {
            Some(e) => {
                let e = e.clone();
                item.on_select(move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.last = Some(label.into());
                        cx.notify();
                    })
                })
            }
            None => item,
        }
    };
    let mut button = ComboButton::new(id(p, "combo"), "Commit")
        .size(size)
        .disabled(disabled);
    for label in COMBO {
        button = button.item(pick(label));
    }
    if let Some(e) = entity {
        let e = e.clone();
        button = button.on_click(move |_, _, cx| {
            e.update(cx, |this, cx| {
                this.last = Some("Commit".into());
                cx.notify();
            })
        });
    }
    button.into_any_element()
}

/// The live trigger: a 48px square (`size-lg`) (`触发按钮高 48`) holding the avatar or
/// the settings icon, `layer-hover-01` on hover, the 2px focus ring;
/// selected (`layer-selected-01`) while its panel is open.
fn trigger(
    p: &str,
    which: usize,
    open: bool,
    content: AnyElement,
    entity: &Entity<Page>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let key = id(p, &format!("trigger{which}"));
    let focus = window
        .use_keyed_state(id(&key, "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let focused = focus.is_focused(window);
    let toggle = {
        let e = entity.clone();
        move |cx: &mut App| {
            e.update(cx, |this, cx| {
                this.open = if this.open == Some(which) {
                    None
                } else {
                    Some(which)
                };
                cx.notify();
            })
        }
    };
    let on_key = toggle.clone();
    div()
        .id(key)
        .track_focus(&focus.tab_index(0).tab_stop(true))
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .size(tokens::SIZE_LG)
        .cursor(CursorStyle::PointingHand)
        .when(open, |d| d.bg(color(ColorRole::LayerSelected01)))
        .hover(|s| s.bg(color(ColorRole::LayerHover01)))
        .on_click(move |_, _, cx| toggle(cx))
        .on_key_down(move |e: &KeyDownEvent, _, cx| {
            if matches!(e.keystroke.key.as_str(), "enter" | "space") && !e.is_held {
                on_key(cx);
                cx.stop_propagation();
            }
        })
        .child(content)
        .when(focused, |d| {
            d.child(inset_outline(2., color(ColorRole::Focus)))
        })
        .into_any_element()
}

fn captioned(i: usize, body: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[i]))
        .child(body)
        .into_any_element()
}

/// The overview tile: the combo button.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    combo("disc-thumb", ComboButtonSize::Lg, false, None)
}

impl Render for Page {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let size = ComboButtonSize::ALL[self.size];
        let live = match self.example {
            0 | 1 => {
                let which = self.example;
                let open = self.open == Some(which);
                let content = if which == 0 {
                    Avatar::new()
                        .fallback(AvatarFallback::new("AN").color(AvatarColor::Purple))
                        .into_any_element()
                } else {
                    Icon::carbon(CarbonIcon::Settings)
                        .size(IconSize::S20)
                        .color(color(ColorRole::IconPrimary))
                        .into_any_element()
                };
                let panel = open.then(|| {
                    if which == 0 {
                        profile("disc-live", Some(&entity))
                    } else {
                        settings("disc-live", Some((self, &entity)))
                    }
                });
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_02)
                    .child(trigger(
                        "disc-live",
                        which,
                        open,
                        content,
                        &entity,
                        window,
                        cx,
                    ))
                    .children(panel)
                    .into_any_element()
            }
            _ => combo("disc-live", size, self.disabled, Some(&entity)),
        };
        let status = match self.example {
            1 => self.applied.clone().map(|a| format!("Applied: {a}")),
            _ => self.last.clone().map(|l| format!("Chose: {l}")),
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(rems(24.))
            .child(captioned(self.example, live))
            .children(status.map(|s| {
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(s)
            }));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "example",
                controls::choices(
                    "disc-example",
                    &EXAMPLES,
                    self.example,
                    controls::setter(&entity, |this: &mut Page, i| {
                        this.example = i;
                        this.open = (i < 2).then_some(i);
                    }),
                ),
            ))
            .child(controls::toggle(
                "disc-open",
                "panel open",
                self.example < 2 && self.open == Some(self.example),
                controls::setter(&entity, |this: &mut Page, v| {
                    this.open = (v && this.example < 2).then_some(this.example);
                }),
            ))
            .child(controls::field(
                "combo size",
                controls::choices(
                    "disc-size",
                    &SIZES,
                    self.size,
                    controls::setter(&entity, |this: &mut Page, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "disc-disabled",
                "combo disabled",
                self.disabled,
                controls::setter(&entity, |this: &mut Page, v| this.disabled = v),
            ));

        let grid = PatternGrid::new(3)
            .cell(captioned(0, profile("disc-ref0", None)))
            .cell(captioned(1, settings("disc-ref1", None)))
            .cell(captioned(
                2,
                combo("disc-ref2", ComboButtonSize::Lg, false, None),
            ));

        Body::new()
            .playground(live)
            .hint("Click the trigger or press Enter: one panel at a time")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
