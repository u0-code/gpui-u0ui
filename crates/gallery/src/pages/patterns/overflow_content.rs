//! Carbon · Patterns › PatternOverflowContent: front, middle and end
//! truncation, a breadcrumb whose middle levels fold into an overflow
//! menu, "Show more / Show less", and the text that must not be truncated
//! (`components/PatternOverflowContent/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, prelude::FluentBuilder as _,
    rems,
};
use u0ui::components::{
    Breadcrumb, BreadcrumbItem, DropdownMenu, DropdownMenuSize, FieldSize, Input, MenuItem,
};
use u0ui::ghost::foundations::{Truncate, TruncateMode};
use u0ui::patterns::{Caption, PatternGrid};
use u0ui::prelude::*;
use u0ui::t3::review::StartTruncatedPath;
use u0ui::typography::paragraph;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    // `h(K.TextInput, {id: 'oc1', …})` has no value.
    let name = crate::stand_in::input_state("", window, cx);
    let ref_name = crate::stand_in::input_state("", window, cx);
    cx.new(|_| OverflowPage {
        name,
        ref_name,
        width: 1,
        expanded: false,
        folded: true,
        current: "feat/dark-tokens".into(),
    })
    .into()
}

struct OverflowPage {
    name: Entity<InputState>,
    ref_name: Entity<InputState>,
    /// Index into [`WIDTHS`].
    width: usize,
    expanded: bool,
    folded: bool,
    current: SharedString,
}

const WIDTHS: [&str; 3] = ["160px", "220px", "320px"];
const WIDTH_PX: [f32; 3] = [160., 220., 320.];
const BREADCRUMB: [&str; 2] = ["folded", "every level"];
const PATH: &str = "~/code/carbon-ui/packages/web/src/theme.ts";
const HASH: &str = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6";
const TITLE: &str = "Investigate the memory spike after reconnecting three times";
const SUMMARY: &str = "The agent updated the G90 theme to match Carbon 11.86, added layer-accent tokens, regenerated the cover, ran the visual tests in both themes and opened pull request #482 with a migration note for teams that override layer tokens directly.";

/// `<span style={{background:'var(--layer-01)', padding:'4px 8px'}}>`.
fn chip(content: impl IntoElement) -> impl IntoElement {
    div()
        .min_w_0()
        .overflow_hidden()
        .bg(color(ColorRole::Layer01))
        .py(tokens::SPACING_02)
        .px(tokens::SPACING_03)
        .child(content)
}

/// `display: grid; grid-template-columns: 80px 220px; gap: 8px 16px; font-size: 14`.
fn truncations(prefix: &'static str, width: f32) -> impl IntoElement {
    let row = |label: &'static str, value: AnyElement| {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(tokens::SPACING_05)
            .child(
                // `.g-dim { color: var(--text-secondary) }`.
                div()
                    .w(rems(80. / 16.))
                    .flex_none()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(label),
            )
            .child(
                div()
                    .w(rems(width / 16.))
                    .min_w_0()
                    .flex_shrink(1.)
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(chip(value)),
            )
    };
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(row(
            "Front",
            StartTruncatedPath::new(PATH).into_any_element(),
        ))
        .child(row(
            "Middle",
            Truncate::new(SharedString::from(format!("{prefix}-mid")), HASH)
                .mode(TruncateMode::Middle)
                .into_any_element(),
        ))
        .child(row(
            "End",
            Truncate::new(SharedString::from(format!("{prefix}-end")), TITLE).into_any_element(),
        ))
}

/// What a crumb or a folded level runs when chosen.
type Go = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

/// The breadcrumb, its middle levels folded into "More levels".
fn breadcrumb(
    prefix: &'static str,
    folded: bool,
    go: impl Fn(&'static str) -> Option<Go>,
) -> impl IntoElement {
    let link = |label: &'static str| {
        let item = BreadcrumbItem::new(label);
        match go(label) {
            Some(f) => item.on_click(move |w, cx| f(w, cx)),
            None => item,
        }
    };
    let mut crumbs = Breadcrumb::new(SharedString::from(format!("{prefix}-crumbs")))
        .no_trailing_slash(true)
        .item(link("Servers"));
    if folded {
        let menu_item = |label: &'static str| {
            let entry = MenuItem::item(label);
            match go(label) {
                Some(f) => entry.on_select(move |w, cx| f(w, cx)),
                None => entry,
            }
        };
        crumbs = crumbs.item(BreadcrumbItem::custom(
            // `OverflowMenu aria-label="More levels" size="sm"
            // renderIcon={OverflowMenuHorizontal}`.
            DropdownMenu::new(SharedString::from(format!("{prefix}-more")))
                .overflow_label("More levels")
                .overflow_icon(CarbonIcon::OverflowMenuHorizontal)
                .size(DropdownMenuSize::Sm)
                .item(menu_item("build-box"))
                .item(menu_item("carbon-ui")),
        ));
    } else {
        crumbs = crumbs.item(link("build-box")).item(link("carbon-ui"));
    }
    crumbs
        .item(link("Worktrees"))
        .item(BreadcrumbItem::new("feat/dark-tokens").current(true))
}

/// The summary clamped to two lines, and its toggle link.
fn show_more(expanded: bool, toggle: Button) -> impl IntoElement {
    // `font-size: 14; line-height: 1.5; background: var(--layer-01); padding: 16`.
    div()
        .flex()
        .flex_col()
        .items_start()
        .bg(color(ColorRole::Layer01))
        .p(tokens::SPACING_05)
        .text_size(rems(0.875))
        .line_height(rems(0.875 * 1.5))
        .text_color(color(ColorRole::TextPrimary))
        .child(
            // A bare `<p>`: Carbon's global `p` rule makes it 16px/24px.
            paragraph()
                .w_full()
                .min_w_0()
                // `display: -webkit-box; -webkit-line-clamp: 2; overflow: hidden`
                // until expanded: two lines, the second ending in "…". The
                // ellipsis also matters for correctness: GPUI's line-layout
                // cache key leaves out the line clamp, so a clamped layout of
                // the untouched text would be reused for the expanded text at
                // the same width, unwrapping its second line.
                .when(!expanded, |d| d.line_clamp(2).text_ellipsis())
                .child(SUMMARY),
        )
        .child(
            // The `K.Link` sits in the div's 21px (14 × 1.5) line box.
            div()
                .h(rems(0.875 * 1.5))
                .flex()
                .items_center()
                .child(toggle),
        )
}

fn toggle_link(id: &'static str, expanded: bool) -> Button {
    Button::new(id, if expanded { "Show less" } else { "Show more" })
        .variant(ButtonVariant::Link)
        .size(ButtonSize::Md)
}

/// The error text stays whole, however long.
fn untruncated(id: &'static str, state: &Entity<InputState>) -> impl IntoElement {
    Input::new(id, state)
        .label("Name for this worktree on build-box")
        .size(FieldSize::Md)
        .invalid(true)
        .invalid_text(
            "This name is already used by another worktree on build-box. Choose a different name.",
        )
}

fn layout(
    truncs: impl IntoElement,
    crumbs: impl IntoElement,
    more: impl IntoElement,
    field: impl IntoElement,
) -> impl IntoElement {
    PatternGrid::new(2)
        .align_start(true)
        .cell(
            div()
                .child(Caption::new(
                    "三种截断：开头 / 中间 / 结尾（省略至少 3 个字符）",
                ))
                .child(truncs),
        )
        .cell(
            div()
                .child(Caption::new("面包屑：中间层级收成 “…” 溢出菜单"))
                .child(crumbs),
        )
        .cell(
            div()
                .child(Caption::new("Show more / Show less：替代滚动和渐隐"))
                .child(more),
        )
        .cell(
            div()
                .child(Caption::new("不截断：页面标题、标签、错误信息、通知"))
                .child(field),
        )
}

/// `components/PatternOverflowContent/preview.html`.
pub fn reference(state: &Entity<InputState>) -> impl IntoElement {
    layout(
        truncations("oc-ref", 220.),
        breadcrumb("oc-ref", true, |_| None),
        show_more(false, toggle_link("oc-ref-toggle", false)),
        untruncated("oc-ref-name", state),
    )
}

/// The overview tile: the middle and end truncations.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(chip(
            Truncate::new("oc-thumb-mid", HASH).mode(TruncateMode::Middle),
        ))
        .child(chip(Truncate::new("oc-thumb-end", TITLE)))
        .into_any_element()
}

impl Render for OverflowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let go = {
            let entity = entity.clone();
            move |label: &'static str| -> Option<Go> {
                let entity = entity.clone();
                Some(std::rc::Rc::new(move |_: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.current = label.into();
                        cx.notify();
                    })
                }))
            }
        };
        let toggle = {
            let entity = entity.clone();
            toggle_link("oc-live-toggle", self.expanded).on_click(move |_, _, cx| {
                entity.update(cx, |this, cx| {
                    this.expanded = !this.expanded;
                    cx.notify();
                })
            })
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(layout(
                truncations("oc-live", WIDTH_PX[self.width]),
                breadcrumb("oc-live", self.folded, go),
                show_more(self.expanded, toggle),
                untruncated("oc-live-name", &self.name),
            ))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!("Opened: {}", self.current))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "truncation width",
                controls::choices(
                    "oc-width",
                    &WIDTHS,
                    self.width,
                    controls::setter(&entity, |this: &mut Self, i| this.width = i),
                ),
            ))
            .child(controls::field(
                "breadcrumb",
                controls::choices(
                    "oc-crumbs",
                    &BREADCRUMB,
                    usize::from(!self.folded),
                    controls::setter(&entity, |this: &mut Self, i: usize| this.folded = i == 0),
                ),
            ))
            .child(controls::toggle(
                "oc-expanded",
                "text expanded",
                self.expanded,
                controls::setter(&entity, |this: &mut Self, on| this.expanded = on),
            ));

        Body::new()
            .playground(live)
            .hint("hover a truncated value for the full text; the … in the breadcrumb lists the folded levels")
            .config(config)
            .example(controls::block("Reference preview", reference(&self.ref_name)))
    }
}
