//! Carbon · Patterns › PatternEmptyStates: small-space, no-results,
//! large-space and error empty states. Live: the actions work (a terminal
//! starts, the filters clear, a project is added, the server is reached),
//! and the layout, illustration and actions can be switched.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::patterns::{Caption, EmptyStateSize, PatternEmptyState, PatternGrid, Pictogram};
use u0ui::prelude::*;

const EXAMPLES: [&str; 4] = ["small", "no results", "large", "error"];

const CAPTIONS: [&str; 4] = [
    "小空间（Tile、侧栏）：图居中，文字左对齐，tertiary 按钮",
    "无结果：给出调整搜索或筛选的建议",
    "大空间：图在左（竖图）或在上（横图），正向标题 + 正文 + 主按钮 + 文档链接；空表格整个替换，不保留表头",
    "错误状态：说清发生了什么，不给错误码，给出下一步",
];

const SIZES: [&str; 2] = ["sm", "lg"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        example: 0,
        size: None,
        pictogram: true,
        actions: true,
        last: None,
    })
    .into()
}

struct Page {
    example: usize,
    /// A layout override; `None` keeps the example's own.
    size: Option<EmptyStateSize>,
    pictogram: bool,
    actions: bool,
    last: Option<SharedString>,
}

/// What an example is drawn with.
#[derive(Clone, Copy)]
struct Knobs {
    size: Option<EmptyStateSize>,
    pictogram: bool,
    actions: bool,
}

impl Knobs {
    const REFERENCE: Self = Self {
        size: None,
        pictogram: true,
        actions: true,
    };
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

fn action(button: Button, label: &'static str, entity: Option<&Entity<Page>>) -> Button {
    match entity {
        Some(e) => {
            let e = e.clone();
            button.on_click(move |_, _, cx| {
                e.update(cx, |this, cx| {
                    this.last = Some(label.into());
                    cx.notify();
                })
            })
        }
        None => button,
    }
}

fn link(p: &str, label: &'static str, entity: Option<&Entity<Page>>) -> Button {
    action(
        Button::new(id(p, label), label).variant(ButtonVariant::Link),
        label,
        entity,
    )
}

/// Example `i` (preview order) with `k`'s overrides.
fn example(i: usize, p: &str, k: Knobs, entity: Option<&Entity<Page>>) -> AnyElement {
    let (size, icon, pic, title, body) = match i {
        0 => (
            EmptyStateSize::Sm,
            CarbonIcon::Terminal,
            64.,
            "No terminals yet",
            "Start one to run commands next to this thread.",
        ),
        1 => (
            EmptyStateSize::Sm,
            CarbonIcon::Search,
            64.,
            "No threads match “dark tokns”",
            "Check the spelling, or clear the project filter.",
        ),
        2 => (
            EmptyStateSize::Lg,
            CarbonIcon::Folder,
            120.,
            "Start by adding a project",
            "Ghost works on folders on this Mac or on a paired server. Add one to start your first thread.",
        ),
        _ => (
            EmptyStateSize::Lg,
            CarbonIcon::WarningAltFilled,
            64.,
            "We can’t reach build-box",
            "It may be asleep or offline. Your threads there are safe.",
        ),
    };
    let mut state = PatternEmptyState::new(k.size.unwrap_or(size), title).body(body);
    if k.pictogram {
        state = state.pictogram(Pictogram::new(Icon::carbon(icon)).size(pic));
    }
    if i == 3 {
        // `minHeight: 0`, `h3 { fontSize: 16 }`.
        state = state.min_height(false).title_style(TypeStyle::Body02);
    }
    if k.actions {
        state = match i {
            0 => state.child(action(
                Button::new(id(p, "new"), "New terminal")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .icon(Icon::carbon(CarbonIcon::Add)),
                "New terminal",
                entity,
            )),
            1 => state.child(link(p, "Clear filters", entity)),
            2 => state.child(
                // `display: flex; gap: 16; align-items: center`.
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .child(action(
                        Button::new(id(p, "add"), "Add project")
                            .variant(ButtonVariant::Primary)
                            .icon(Icon::carbon(CarbonIcon::FolderAdd)),
                        "Add project",
                        entity,
                    ))
                    .child(link(p, "Read the getting started guide", entity)),
            ),
            _ => state.child(action(
                Button::new(id(p, "reach"), "Reach again")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm),
                "Reach again",
                entity,
            )),
        };
    }
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[i]))
        .child(state)
        .into_any_element()
}

/// The overview tile: the small empty state.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(example(0, "empty-thumb", Knobs::REFERENCE, None))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let knobs = Knobs {
            size: self.size,
            pictogram: self.pictogram,
            actions: self.actions,
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(example(self.example, "empty-live", knobs, Some(&entity)))
            .children(self.last.clone().map(|l| {
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("Chose: {l}"))
            }));

        let size_index = self.size.map_or(0, |s| {
            1 + EmptyStateSize::ALL
                .iter()
                .position(|x| *x == s)
                .unwrap_or(0)
        });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "example",
                controls::choices(
                    "empty-example",
                    &EXAMPLES,
                    self.example,
                    controls::setter(&entity, |this: &mut Page, i| this.example = i),
                ),
            ))
            .child(controls::field(
                "layout",
                controls::choices(
                    "empty-size",
                    &["example", SIZES[0], SIZES[1]],
                    size_index,
                    controls::setter(&entity, |this: &mut Page, i: usize| {
                        this.size = i.checked_sub(1).map(|i| EmptyStateSize::ALL[i]);
                    }),
                ),
            ))
            .child(controls::toggle(
                "empty-pic",
                "illustration",
                self.pictogram,
                controls::setter(&entity, |this: &mut Page, v| this.pictogram = v),
            ))
            .child(controls::toggle(
                "empty-actions",
                "actions",
                self.actions,
                controls::setter(&entity, |this: &mut Page, v| this.actions = v),
            ));

        let grid = PatternGrid::new(2)
            .cell(example(0, "empty-ref0", Knobs::REFERENCE, None))
            .cell(example(1, "empty-ref1", Knobs::REFERENCE, None))
            .full(example(2, "empty-ref2", Knobs::REFERENCE, None))
            .full(example(3, "empty-ref3", Knobs::REFERENCE, None));

        Body::new()
            .playground(live)
            .hint("Switch the example and layout; the actions respond")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
