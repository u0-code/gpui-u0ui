//! Ghost · Foundations › CJKTypography: the CJK type styles over the Plex →
//! PingFang SC → Hiragino Sans GB → Noto Sans CJK SC fallback stack.
use crate::controls;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CjkPage).into()
}

struct CjkPage;

fn entry(
    term: &'static str,
    style: TypeStyle,
    role: ColorRole,
    text: &'static str,
) -> impl IntoElement {
    // `.g-cjk-specimen { grid-template-columns: 9rem 1fr; gap: 12px 16px; align-items: baseline }`.
    div()
        .flex()
        .items_baseline()
        .gap(tokens::SPACING_05)
        .child(
            div()
                .w(rems(9.))
                .flex_none()
                .type_style(TypeStyle::Code01)
                .text_color(color(ColorRole::TextSecondary))
                .child(term),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .type_style(style)
                .text_color(color(role))
                .child(text),
        )
}

/// `components/CJKTypography/preview.html`.
pub fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .child(entry(
            "body-compact-cjk-01",
            TypeStyle::BodyCompactCjk01,
            ColorRole::TextPrimary,
            "把 dark 主题的 token 加进来，然后更新 Cover 页。Agent 已经在 feat/dark-tokens 分支上工作 1 分 14 秒。",
        ))
        .child(entry(
            "body-cjk-01",
            TypeStyle::BodyCjk01,
            ColorRole::TextPrimary,
            "长段落用 24px 行高：中文字形比拉丁字母高，行距太紧会让两行笔画粘在一起；混排时英文 IBM Plex Sans 与 PingFang SC 基线对齐。",
        ))
        .child(entry(
            "label-cjk-01",
            TypeStyle::LabelCjk01,
            ColorRole::TextSecondary,
            "上次运行：今天 09:00 · 下次：明天 09:00",
        ))
        .child(entry(
            "heading-cjk-03",
            TypeStyle::HeadingCjk03,
            ColorRole::TextPrimary,
            "设置 · 通用",
        ))
        .child(entry(
            "mono",
            TypeStyle::Code01,
            ColorRole::TextPrimary,
            "src/组件/按钮.tsx  —  等宽回退到 Noto Sans Mono CJK",
        ))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .type_style(TypeStyle::HeadingCjk03)
        .text_color(color(ColorRole::TextPrimary))
        .child("设置 · 通用")
        .into_any_element()
}

impl Render for CjkPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let specs = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_02)
            .type_style(TypeStyle::Code01)
            .text_color(color(ColorRole::TextSecondary))
            .children(
                [
                    TypeStyle::BodyCompactCjk01,
                    TypeStyle::BodyCjk01,
                    TypeStyle::LabelCjk01,
                    TypeStyle::HeadingCjk03,
                ]
                .map(|s| {
                    let spec = s.spec();
                    format!(
                        "{}  {}/{} · {}",
                        spec.name,
                        spec.font_size_px(),
                        spec.line_height_px().round(),
                        spec.weight
                    )
                }),
            );
        crate::page::Body::new()
            .playground(reference())
            .example(controls::block("Type style tokens", specs))
    }
}
