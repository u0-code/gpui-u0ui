//! Overlays › Command: the reference palette and a CommandDialog opened
//! with a button (or ⌘K while the page has focus).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::{Command, CommandDialog, CommandGroup, CommandItem};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CommandPage {
        dialog_open: false,
        disabled: false,
        last: "—".into(),
    })
    .into()
}

struct CommandPage {
    dialog_open: bool,
    disabled: bool,
    last: SharedString,
}

impl CommandPage {
    /// `components/Command/preview.html`.
    fn palette(&self, id: &'static str, cx: &mut Context<Self>) -> Command {
        let entity = cx.entity();
        let run = |label: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = label.into();
                    this.dialog_open = false;
                    cx.notify();
                })
            }
        };
        Command::new(id)
            .group(
                CommandGroup::new("Suggestions")
                    .item(
                        CommandItem::new("Open dashboard")
                            .icon(CarbonIcon::Dashboard)
                            .on_select(run("Open dashboard")),
                    )
                    .item(
                        CommandItem::new("Create resource")
                            .icon(CarbonIcon::Add)
                            .shortcut("⌘N")
                            .disabled(self.disabled)
                            .on_select(run("Create resource")),
                    )
                    .item(
                        CommandItem::new("Search docs")
                            .icon(CarbonIcon::Search)
                            .shortcut("⌘K")
                            .on_select(run("Search docs")),
                    ),
            )
            .separator()
            .group(
                CommandGroup::new("Settings")
                    .item(
                        CommandItem::new("Profile")
                            .icon(CarbonIcon::User)
                            .on_select(run("Profile")),
                    )
                    .item(
                        CommandItem::new("Preferences")
                            .icon(CarbonIcon::Settings)
                            .shortcut("⌘,")
                            .on_select(run("Preferences")),
                    ),
            )
    }
}

impl Render for CommandPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section(
                "Reference preview: type to filter, ↑↓ Enter",
            ))
            .child(self.palette("ref-command", cx));

        let dialog = CommandDialog::new(
            "command-dialog",
            self.dialog_open,
            {
                let entity = entity.clone();
                move |open, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.dialog_open = open;
                        cx.notify();
                    })
                }
            },
            self.palette("dialog-command", cx),
        );
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("CommandDialog"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .child(
                        Button::new("open-command-dialog", "Open command palette")
                            .variant(ButtonVariant::Tertiary)
                            .on_click({
                                let entity = entity.clone();
                                move |_, _, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.dialog_open = true;
                                        cx.notify();
                                    })
                                }
                            }),
                    )
                    .child(dialog)
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("Last command: {}", self.last)),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "command-disabled",
                "disabled item (Create resource)",
                self.disabled,
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.disabled = on;
                        cx.notify();
                    })
                },
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}
