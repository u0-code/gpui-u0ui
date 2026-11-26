//! Overlays › Tooltip: the reference preview (an icon button with its
//! tooltip open), a configurable live tooltip, and all 12 alignments.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::{Align, Tooltip};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TooltipPage {
        align: 0,
        long: false,
        generation: 0,
        default_open: true,
    })
    .into()
}

struct TooltipPage {
    /// Index into [`Align::ALL`].
    align: usize,
    long: bool,
    /// Bumped to re-mount the live tooltip when `defaultOpen` changes.
    generation: usize,
    default_open: bool,
}

fn copy_button(id: impl Into<gpui_kit::ElementId>) -> Button {
    // `cds--btn--ghost cds--btn--icon-only cds--btn--md`.
    Button::new(id, "Copy")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Md)
        .icon(CarbonIcon::Copy)
        .icon_only(true)
        .tooltip(false)
}

impl Render for TooltipPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let align = Align::ALL[self.align];
        let content: SharedString = if self.long {
            "Copies the connection string, including the port and database name, to the clipboard."
                .into()
        } else {
            "Copy to clipboard".into()
        };

        // `components/Tooltip/preview.html`: paddingTop 56, paddingLeft 140.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().flex().pt(rems(56. / 16.)).pl(rems(140. / 16.)).child(
                    Tooltip::new("ref-tooltip", copy_button("ref-copy"))
                        .content("Copy to clipboard")
                        .default_open(true),
                ),
            );

        let live = div().flex().flex_col().gap(tokens::SPACING_05).child(
            div()
                .flex()
                .justify_center()
                .py(rems(8.))
                .bg(color(ColorRole::Layer01))
                .child(
                    Tooltip::new(
                        ("live-tooltip", self.generation),
                        copy_button(("live-copy", self.generation)),
                    )
                    .content(content)
                    .align(align)
                    .default_open(self.default_open),
                ),
        );

        let grid = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Alignments (hover each)"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_07)
                    .p(tokens::SPACING_07)
                    .children(Align::ALL.iter().map(|align| {
                        Tooltip::new(
                            SharedString::from(format!("grid-{}", align.name())),
                            Button::new(
                                SharedString::from(format!("grid-b-{}", align.name())),
                                align.name(),
                            )
                            .variant(ButtonVariant::Tertiary)
                            .size(ButtonSize::Sm),
                        )
                        .content(align.name())
                        .align(*align)
                    })),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "side / align",
                controls::choices(
                    "tooltip-align",
                    &Align::ALL.map(|a| a.name()),
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
                "tooltip-long",
                "long content",
                self.long,
                {
                    let entity = entity.clone();
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.long = on;
                            cx.notify();
                        })
                    }
                },
            ))
            .child(controls::toggle(
                "tooltip-default-open",
                "defaultOpen",
                self.default_open,
                {
                    let entity = entity.clone();
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.default_open = on;
                            this.generation += 1;
                            cx.notify();
                        })
                    }
                },
            ));

        Body::new()
            .playground(live)
            .hint("hover or focus the button")
            .config(config)
            .example(reference)
            .example(grid)
    }
}
