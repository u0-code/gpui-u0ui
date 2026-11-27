//! T3 · Composer › ContextChip: every kind, the unresolved / invalid states
//! and a removable bubble (`components/ContextChip/preview.html`), plus a
//! configurable live chip.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{ContextChip, ContextChipKind, ContextChipShape, ContextChipState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ContextChipPage {
        kind: 0,
        shape: 0,
        state: 0,
        meta: false,
        removable: true,
        clickable: true,
        in_composer: false,
        last: "Click the chip or its ×.".into(),
    })
    .into()
}

struct ContextChipPage {
    kind: usize,
    shape: usize,
    state: usize,
    meta: bool,
    removable: bool,
    clickable: bool,
    in_composer: bool,
    last: SharedString,
}

/// The preview's 17 labelled kinds.
const REFERENCE: [(&str, &str); 17] = [
    ("file", "src/tokens.json"),
    ("folder", "src/components"),
    ("image", "screenshot.png"),
    ("mention", "@reviewer"),
    ("terminal", "dev terminal"),
    ("element", "<Header>"),
    ("review-comment", "Comment on L42"),
    ("pr-open", "#482"),
    ("pr-draft", "#479"),
    ("pr-merged", "#470"),
    ("pr-closed", "#455"),
    ("skill", "release-notes"),
    ("model", "Opus"),
    ("thread", "Fix date picker"),
    ("environment", "build-box"),
    ("citation", "Carbon docs"),
    ("doc-pdf", "spec.pdf"),
];

/// The reference preview.
pub fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        // `flexWrap: wrap; gap: 8px 16px`.
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_x(tokens::SPACING_05)
                .gap_y(tokens::SPACING_03)
                .children(REFERENCE.iter().map(|(kind, label)| {
                    ContextChip::new(
                        SharedString::from(format!("chip-ref-{kind}")),
                        ContextChipKind::from_name(kind),
                        *label,
                    )
                })),
        )
        // `cu-row`: wrap, 16px gap, centered.
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(tokens::SPACING_05)
                .child(
                    ContextChip::new(
                        "chip-ref-unresolved",
                        ContextChipKind::File,
                        "deleted-file.ts",
                    )
                    .state(ContextChipState::Unresolved),
                )
                .child(
                    ContextChip::new("chip-ref-invalid", ContextChipKind::File, "too-large.bin")
                        .state(ContextChipState::Invalid),
                )
                .child(
                    ContextChip::new("chip-ref-bubble", ContextChipKind::File, "tokens.json")
                        .shape(ContextChipShape::Bubble)
                        .meta("60 KB")
                        .on_remove(|_, _| {}),
                ),
        )
}

impl Render for ContextChipPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = ContextChipKind::ALL[self.kind];
        let mut chip = ContextChip::new("chip-live", kind, "src/components/composer.tsx")
            .shape(ContextChipShape::ALL[self.shape])
            .state(ContextChipState::ALL[self.state])
            .in_composer(self.in_composer);
        if self.meta {
            chip = chip.meta("12 KB");
        }
        if self.clickable {
            let entity = entity.clone();
            chip = chip.on_click(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.last = "onClick: open the reference".into();
                    cx.notify();
                })
            });
        }
        if self.removable {
            let entity = entity.clone();
            chip = chip.on_remove(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.last = "onRemove: removed from the message".into();
                    cx.notify();
                })
            });
        }
        let live = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(tokens::SPACING_05)
            .max_w_full()
            .child(chip)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );

        let kinds = ContextChipKind::ALL.map(|k| k.name());
        let shapes = ContextChipShape::ALL.map(|s| s.name());
        let states = ["none", "unresolved", "invalid"];
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "kind",
                controls::choices(
                    "chip-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "shape",
                controls::choices(
                    "chip-shape",
                    &shapes,
                    self.shape,
                    controls::setter(&entity, |this: &mut Self, i| this.shape = i),
                ),
            ))
            .child(controls::field(
                "state",
                controls::choices(
                    "chip-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ))
            .child(controls::toggle(
                "chip-meta",
                "meta",
                self.meta,
                controls::setter(&entity, |this: &mut Self, on| this.meta = on),
            ))
            .child(controls::toggle(
                "chip-click",
                "onClick",
                self.clickable,
                controls::setter(&entity, |this: &mut Self, on| this.clickable = on),
            ))
            .child(controls::toggle(
                "chip-remove",
                "onRemove",
                self.removable,
                controls::setter(&entity, |this: &mut Self, on| this.removable = on),
            ))
            .child(controls::toggle(
                "chip-composer",
                "In the composer",
                self.in_composer,
                controls::setter(&entity, |this: &mut Self, on| this.in_composer = on),
            ));

        Body::new()
            .playground(live)
            .hint("Tab to the chip and its ×; Enter or Space activates.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile: a few kinds.
pub fn thumbnail(_: &mut Window, _: &mut App) -> gpui_kit::AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap_x(tokens::SPACING_05)
        .gap_y(tokens::SPACING_03)
        .max_w(gpui_kit::rems(15.))
        .child(ContextChip::new(
            "chip-th-0",
            ContextChipKind::File,
            "src/tokens.json",
        ))
        .child(ContextChip::new(
            "chip-th-1",
            ContextChipKind::PrOpen,
            "#482",
        ))
        .child(ContextChip::new(
            "chip-th-2",
            ContextChipKind::Mention,
            "@reviewer",
        ))
        .into_any_element()
}
