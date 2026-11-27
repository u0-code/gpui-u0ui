//! T3 · Composer › EffortGlyph: the five levels and high + fast
//! (`components/EffortGlyph/preview.html`), plus a live glyph inside the
//! composer's `ComposerControl` that cycles the level on click.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{ComposerControl, EffortGlyph, EffortLevel};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| EffortPage {
        level: 2,
        fast: false,
        active: false,
    })
    .into()
}

struct EffortPage {
    level: usize,
    fast: bool,
    active: bool,
}

/// `display: inline-flex; gap: 8px; align-items: center; font-size: 12px`.
fn labelled(glyph: EffortGlyph, label: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(tokens::SPACING_03)
        .text_size(gpui_kit::rems(0.75))
        .text_color(color(ColorRole::TextPrimary))
        .child(glyph)
        .child(label)
}

/// The reference preview (`cu-row`).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
        .children(
            EffortLevel::ALL
                .into_iter()
                .map(|l| labelled(EffortGlyph::new(l), l.name())),
        )
        .child(labelled(
            EffortGlyph::new(EffortLevel::High).fast(true),
            "high + fast",
        ))
}

impl Render for EffortPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let level = EffortLevel::ALL[self.level];
        let live = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_07)
            .child(EffortGlyph::new(level).fast(self.fast))
            .child(
                ComposerControl::new("effort-live", "Reasoning effort")
                    .active(self.active)
                    .on_click({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.level = (this.level + 1) % EffortLevel::ALL.len();
                                cx.notify();
                            })
                        }
                    })
                    .child(EffortGlyph::new(level).fast(self.fast)),
            )
            .child(
                ComposerControl::new("effort-permissions", "Ask before edits")
                    .icon(CarbonIcon::Locked),
            );
        let levels = EffortLevel::ALL.map(|l| l.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "level",
                controls::choices(
                    "effort-level",
                    &levels,
                    self.level,
                    controls::setter(&entity, |this: &mut Self, i| this.level = i),
                ),
            ))
            .child(controls::toggle(
                "effort-fast",
                "fast",
                self.fast,
                controls::setter(&entity, |this: &mut Self, on| this.fast = on),
            ))
            .child(controls::toggle(
                "effort-active",
                "ComposerControl active",
                self.active,
                controls::setter(&entity, |this: &mut Self, on| this.active = on),
            ));
        Body::new()
            .playground(live)
            .hint("Click the control to step through the levels.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    reference().into_any_element()
}
