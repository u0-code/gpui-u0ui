//! Carbon · Patterns › PatternCommonActions: add, clear, copy, edit / delete,
//! remove, reset and next / cancel, each as Carbon places it. Everything is
//! live: the search clears, the snippet copies, the menu opens, rows are
//! removed, the reset link restores the filters.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, rems,
};
use u0ui::components::form::{self, FieldSize};
use u0ui::components::{
    Checkbox, CodeSnippet, DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem, SnippetType,
};
use u0ui::patterns::{Caption, PatternGrid, PatternRow, PatternSearch};
use u0ui::prelude::*;

/// The reference members (`['ana@example.com','ben@example.com']`).
const MEMBERS: [&str; 2] = ["ana@example.com", "ben@example.com"];

/// The six examples, in preview order.
const EXAMPLES: [&str; 6] = ["add", "copy", "edit", "remove", "reset", "next"];

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let search = cx.new(|cx| InputState::new(window, cx).default_value("vm-10"));
    let live_search = cx.new(|cx| InputState::new(window, cx).default_value("vm-10"));
    cx.new(|cx| {
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        cx.observe(&live_search, |_, _, cx| cx.notify()).detach();
        Page {
            search,
            live_search,
            members: MEMBERS.to_vec(),
            running: true,
            stopped: false,
            example: 0,
            size: 2,
            layer: 0,
            disabled: false,
        }
    })
    .into()
}

struct Page {
    search: Entity<InputState>,
    live_search: Entity<InputState>,
    members: Vec<&'static str>,
    running: bool,
    stopped: bool,
    example: usize,
    size: usize,
    layer: usize,
    disabled: bool,
}

/// The examples' knobs: the search and button size, the layer they sit on,
/// whether the actions are disabled.
#[derive(Clone, Copy)]
struct Knobs {
    size: FieldSize,
    level: LayerLevel,
    disabled: bool,
}

impl Knobs {
    const REFERENCE: Self = Self {
        size: FieldSize::Lg,
        level: LayerLevel::One,
        disabled: false,
    };

    fn button(self) -> ButtonSize {
        match self.size {
            FieldSize::Sm => ButtonSize::Sm,
            FieldSize::Md => ButtonSize::Md,
            FieldSize::Lg => ButtonSize::Lg,
        }
    }
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// A captioned example (`h('div', null, Cap(...), example)`).
fn captioned(caption: &'static str, example: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(caption))
        .child(example)
        .into_any_element()
}

/// Add + Clear: `.cu-ptoolbar { display: flex; align-items: stretch;
/// background: var(--layer-01) }` with the search taking the rest.
fn add(p: &str, search: &Entity<InputState>, k: Knobs) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_stretch()
        .w_full()
        .bg(color(ColorRole::Layer01))
        .child(
            PatternSearch::new(id(p, "search"), search)
                .label("Search")
                .placeholder("Filter resources")
                .close_label("Clear search")
                .size(k.size)
                .layer(k.level)
                .disabled(k.disabled),
        )
        .child(
            Button::new(id(p, "add"), "Add resource")
                .variant(ButtonVariant::Primary)
                .size(k.button())
                .icon(Icon::carbon(CarbonIcon::Add))
                .disabled(k.disabled),
        )
        .into_any_element()
}

/// Copy: a single-line snippet whose button says "Copied".
fn copy(p: &str, k: Knobs) -> AnyElement {
    CodeSnippet::new(
        id(p, "snippet"),
        SnippetType::Single,
        "npx ghost pair --link 7f3a-9c2e",
    )
    .copy_texts("Copy to clipboard", "Copied")
    .layer(k.level)
    .disabled(k.disabled)
    .into_any_element()
}

/// Edit / Delete: an overflow menu whose danger item comes last after a divider.
fn edit(p: &str, _k: Knobs) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child("vm-101"),
        )
        .child(
            DropdownMenu::new(id(p, "menu"))
                .label("Actions")
                .kind(DropdownMenuKind::Ghost)
                .size(DropdownMenuSize::Sm)
                .item(MenuItem::item("Edit").icon(Icon::carbon(CarbonIcon::Edit)))
                .item(MenuItem::item("Duplicate").icon(Icon::carbon(CarbonIcon::Copy)))
                .item(MenuItem::divider())
                .item(
                    MenuItem::item("Delete")
                        .danger(true)
                        .icon(Icon::carbon(CarbonIcon::TrashCan)),
                ),
        )
        .into_any_element()
}

/// Remove: a subtract icon button at the end of each row.
fn remove(p: &str, page: &Page, entity: &Entity<Page>, k: Knobs) -> AnyElement {
    div()
        .w_full()
        .bg(color(ColorRole::Layer01))
        .children(page.members.iter().enumerate().map(|(i, m)| {
            let entity = entity.clone();
            let member = *m;
            PatternRow::new().child(div().child(member)).child(
                Button::new(id(p, &format!("rm{i}")), format!("Remove {member}"))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon_only(true)
                    .icon(Icon::carbon(CarbonIcon::Subtract))
                    .disabled(k.disabled)
                    .on_click(move |_, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.members.retain(|x| *x != member);
                            cx.notify();
                        })
                    }),
            )
        }))
        .into_any_element()
}

/// Reset: a link beside the filters.
fn reset(p: &str, page: &Page, entity: &Entity<Page>, k: Knobs) -> AnyElement {
    let restore = entity.clone();
    // `display: flex; gap: 16; align-items: center`. Each checkbox is a
    // `.cds--form-item` (`flex: 1 1 auto`): the two share the free space and
    // the link ends the row.
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_05)
        .child(form::flex_form_item(
            Checkbox::new(id(p, "running"))
                .label("Running")
                .checked(page.running)
                .disabled(k.disabled)
                .on_checked_change(controls::setter(entity, |this: &mut Page, v| {
                    this.running = v
                })),
        ))
        .child(form::flex_form_item(
            Checkbox::new(id(p, "stopped"))
                .label("Stopped")
                .checked(page.stopped)
                .disabled(k.disabled)
                .on_checked_change(controls::setter(entity, |this: &mut Page, v| {
                    this.stopped = v
                })),
        ))
        .child(
            Button::new(id(p, "reset"), "Reset filters")
                .variant(ButtonVariant::Link)
                .disabled(k.disabled)
                .on_click(move |_, _, cx| {
                    restore.update(cx, |this, cx| {
                        this.running = true;
                        this.stopped = false;
                        this.members = MEMBERS.to_vec();
                        cx.notify();
                    })
                }),
        )
        .into_any_element()
}

/// Next / Cancel: the forward button carries the arrow; cancel is secondary.
fn next(p: &str, k: Knobs) -> AnyElement {
    div()
        .flex()
        .child(
            Button::new(id(p, "cancel"), "Cancel")
                .variant(ButtonVariant::Secondary)
                .size(k.button())
                .disabled(k.disabled),
        )
        .child(
            Button::new(id(p, "next"), "Next")
                .variant(ButtonVariant::Primary)
                .size(k.button())
                .icon(Icon::carbon(CarbonIcon::ArrowRight))
                .disabled(k.disabled),
        )
        .into_any_element()
}

const CAPTIONS: [&str; 6] = [
    "Add：数据表工具栏里唯一的主按钮，带图标；Clear：字段右侧 ×",
    "Copy：复制图标，点击后提示 “Copied”",
    "Edit / Delete：溢出菜单里，删除为 danger 项，放在最后并有分隔",
    "Remove：列表行末的减号图标按钮，很少用主按钮",
    "Reset：筛选区里用链接",
    "Next / Cancel：前进按钮带箭头；取消用次要按钮",
];

fn example(
    i: usize,
    p: &str,
    page: &Page,
    entity: &Entity<Page>,
    search: &Entity<InputState>,
    k: Knobs,
) -> AnyElement {
    let body = match i {
        0 => add(p, search, k),
        1 => copy(p, k),
        2 => edit(p, k),
        3 => remove(p, page, entity, k),
        4 => reset(p, page, entity, k),
        _ => next(p, k),
    };
    captioned(CAPTIONS[i.min(5)], body)
}

/// The overview tile: the toolbar and the remove rows.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(next("ca-thumb", Knobs::REFERENCE))
        .child(
            div()
                .bg(color(ColorRole::Layer01))
                .children(MEMBERS.map(|m| PatternRow::new().child(div().child(m)))),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let knobs = Knobs {
            size: FieldSize::ALL[self.size],
            level: [LayerLevel::One, LayerLevel::Two][self.layer],
            disabled: self.disabled,
        };
        let live = div().w_full().max_w(rems(32.)).child(example(
            self.example,
            "ca-live",
            self,
            &entity,
            &self.live_search,
            knobs,
        ));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "action",
                controls::choices(
                    "ca-example",
                    &EXAMPLES,
                    self.example,
                    controls::setter(&entity, |this: &mut Page, i| this.example = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "ca-size",
                    &["sm", "md", "lg"],
                    self.size,
                    controls::setter(&entity, |this: &mut Page, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "layer",
                controls::choices(
                    "ca-layer",
                    &["01", "02"],
                    self.layer,
                    controls::setter(&entity, |this: &mut Page, i| this.layer = i),
                ),
            ))
            .child(controls::toggle(
                "ca-disabled",
                "disabled",
                self.disabled,
                controls::setter(&entity, |this: &mut Page, v| this.disabled = v),
            ));

        let mut grid = PatternGrid::new(2);
        for i in 0..EXAMPLES.len() {
            grid = grid.cell(example(
                i,
                "ca-ref",
                self,
                &entity,
                &self.search,
                Knobs::REFERENCE,
            ));
        }

        Body::new()
            .playground(live)
            .hint("Clear the search, copy the snippet, open Actions, remove a row, reset")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
