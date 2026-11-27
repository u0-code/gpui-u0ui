//! Carbon · Patterns › PatternTextToolbar: a rich-text editor's toolbar over
//! its text area, with an attachment, a link field and the save row
//! (`components/PatternTextToolbar/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, FontStyle, Hsla, IntoElement,
    ParentElement, Render, SharedString, Styled, StyledText, TextRun, UnderlineStyle, Window,
    component::input::InputState, div, prelude::FluentBuilder as _, px, rems,
};
use u0ui::components::{FieldSize, Input, Select, SelectType, form};
use u0ui::patterns::{Caption, TextToolbar, ToolbarAction, ToolbarItem};
use u0ui::prelude::*;
use u0ui::tokens::FontFamilyKey;
use u0ui::typography::paragraph;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let link = crate::stand_in::input_state("https://example.com/1.5", window, cx);
    cx.new(|cx| {
        cx.observe(&link, |_, _, cx| cx.notify()).detach();
        TextToolbarPage {
            format: Format {
                bold: true,
                ..Format::default()
            },
            history: Vec::new(),
            future: Vec::new(),
            attachment: true,
            link_open: false,
            linked: None,
            link,
            saved: SharedString::from("Saved 10:42"),
            width: 0,
            autosave: false,
        }
    })
    .into()
}

/// The toggles' state.
#[derive(Clone, Copy, Default, PartialEq)]
struct Format {
    bold: bool,
    italic: bool,
    underline: bool,
}

struct TextToolbarPage {
    format: Format,
    history: Vec<Format>,
    future: Vec<Format>,
    attachment: bool,
    link_open: bool,
    linked: Option<SharedString>,
    link: Entity<InputState>,
    saved: SharedString,
    width: usize,
    autosave: bool,
}

const WIDTHS: [&str; 3] = ["fill", "36rem", "22rem"];
const SAVE: [&str; 2] = ["Save draft", "Autosave"];

/// `.cu-ptarea p`: the note, styled by the toggles. A bare `p`, so
/// Carbon's global `p` rule makes it 16px/24px over the area's 14px. Its
/// `<b>1.5</b>` stays at weight 400: Carbon's reset gives `b` `font:
/// inherit` (only `strong` is 600).
fn note(format: Format) -> impl IntoElement {
    let parts = [
        ("Release notes for ", false),
        ("1.5", true),
        (
            ": side chat, pairing links that expire, and a faster ⌘P.",
            false,
        ),
    ];
    let text: String = parts.iter().map(|(t, _)| *t).collect();
    let fg = color(ColorRole::TextPrimary);
    let runs = parts
        .iter()
        .map(|(t, _)| {
            let mut font =
                u0ui::typography::font(FontFamilyKey::Sans, if format.bold { 600 } else { 400 });
            if format.italic {
                font.style = FontStyle::Italic;
            }
            TextRun {
                len: t.len(),
                font,
                color: fg,
                background_color: None,
                underline: format.underline.then_some(UnderlineStyle {
                    thickness: px(1.),
                    color: Some(fg),
                    wavy: false,
                }),
                strikethrough: None,
            }
        })
        .collect::<Vec<_>>();
    paragraph().child(StyledText::new(text).with_runs(runs))
}

/// `.cu-ptattach`: the attached file in the area's bottom-left corner.
fn attachment(id: &'static str, on_remove: Option<Entity<TextToolbarPage>>) -> AnyElement {
    let remove = Button::new(id, "Remove changelog.md")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .icon(CarbonIcon::Close)
        .icon_only(true)
        .when_some(on_remove, |b, entity| {
            b.on_click(move |_, _, cx| {
                entity.update(cx, |this, cx| {
                    this.attachment = false;
                    cx.notify();
                })
            })
        });
    div()
        // `.cu-ptattach { position: absolute; left: var(--spacing-05); bottom: var(--spacing-03) }`.
        .absolute()
        .left(tokens::SPACING_05)
        .bottom(tokens::SPACING_03)
        // `display: inline-flex; align-items: center; gap: var(--spacing-03); padding-left: var(--spacing-03)`.
        .flex()
        .flex_row()
        .items_center()
        .gap(tokens::SPACING_03)
        .pl(tokens::SPACING_03)
        // `background: var(--layer-02); box-shadow: inset 0 0 0 1px var(--border-subtle-01)`:
        // drawn over the chip, taking no room (a border made it 34px, not 32).
        .bg(color(ColorRole::Layer02))
        .child(form::inset_outline(1., color(ColorRole::BorderSubtle01)))
        // `font-size: 0.75rem`.
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextPrimary))
        .child(
            // `.cu-ptattach > svg { fill: var(--icon-secondary) }`.
            Icon::carbon(CarbonIcon::Document)
                .size(IconSize::S16)
                .color(color(ColorRole::IconSecondary)),
        )
        .child("changelog.md")
        .child(remove)
        .into_any_element()
}

/// `.cu-ptarea`: the text area under the toolbar.
fn area(content: impl IntoElement, attached: Option<AnyElement>) -> impl IntoElement {
    div()
        .relative()
        // `.cu-ptarea { min-height: 8rem; padding: var(--spacing-05) var(--spacing-05) 3.5rem }`.
        .min_h(rems(8.))
        .pt(tokens::SPACING_05)
        .px(tokens::SPACING_05)
        .pb(rems(3.5))
        // `background: var(--field-01); border-bottom: 1px solid var(--border-strong-01)`.
        .bg(color(ColorRole::Field01))
        .border_b_1()
        .border_color(color(ColorRole::BorderStrong01))
        // `font-size: 0.875rem; line-height: 1.5`.
        .text_size(rems(0.875))
        .line_height(rems(0.875 * 1.5))
        .text_color(color(ColorRole::TextPrimary))
        .child(content)
        .children(attached)
}

/// The save row: `justify-content: flex-end; margin-top: 8px`, the status
/// caption 16px before the buttons.
fn save_row(status: SharedString, buttons: Vec<AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .justify_end()
        .items_center()
        .mt(tokens::SPACING_03)
        .child(
            div()
                .mr(tokens::SPACING_05)
                .child(Caption::new(status).inline()),
        )
        .children(buttons)
}

fn font_select(id: &'static str) -> Select {
    Select::new(id)
        .items(["IBM Plex Sans", "IBM Plex Mono"])
        .default_value("IBM Plex Sans")
        .kind(SelectType::Inline)
        .size(FieldSize::Md)
}

fn size_select(id: &'static str) -> Select {
    Select::new(id)
        .items(["12", "14", "16", "20"])
        .default_value("14")
        .kind(SelectType::Inline)
        .size(FieldSize::Md)
}

/// The toolbar's items, in the reference order. `act` wires a button.
fn items(
    prefix: &'static str,
    format: Format,
    mut act: impl FnMut(&'static str, ToolbarAction) -> ToolbarAction,
) -> Vec<ToolbarItem> {
    let mut button = |id: &'static str, label: &'static str, icon: CarbonIcon, on: bool| {
        ToolbarItem::action(act(id, ToolbarAction::new(id, label, icon).selected(on)))
    };
    let (font_id, size_id) = match prefix {
        "ref" => ("tt-ref-font", "tt-ref-size"),
        _ => ("tt-live-font", "tt-live-size"),
    };
    vec![
        button("undo", "Undo", CarbonIcon::Undo, false),
        button("redo", "Redo", CarbonIcon::Redo, false),
        ToolbarItem::Separator,
        // `<div style={{width:140}}><Dropdown type="inline" size="md"/></div>`.
        ToolbarItem::dropdown(140., font_select(font_id)),
        // `<div style={{width:64}}>`.
        ToolbarItem::dropdown(64., size_select(size_id)),
        ToolbarItem::Separator,
        button("bold", "Bold (⌘B)", CarbonIcon::TextBold, format.bold),
        button(
            "italic",
            "Italic (⌘I)",
            CarbonIcon::TextItalic,
            format.italic,
        ),
        button(
            "underline",
            "Underline (⌘U)",
            CarbonIcon::TextUnderline,
            format.underline,
        ),
        ToolbarItem::Separator,
        button("bullets", "Bulleted list", CarbonIcon::ListBulleted, false),
        button("numbers", "Numbered list", CarbonIcon::ListNumbered, false),
        button("checklist", "Checklist", CarbonIcon::ListChecked, false),
        ToolbarItem::Separator,
        button("indent", "Indent", CarbonIcon::TextIndentMore, false),
        button("align", "Align left", CarbonIcon::TextAlignLeft, false),
        ToolbarItem::Separator,
        button("attach", "Attach file", CarbonIcon::Attachment, false),
        button("link", "Insert link", CarbonIcon::Link, false),
        ToolbarItem::Spacer,
    ]
}

fn editor(
    toolbar: TextToolbar,
    link_row: Option<AnyElement>,
    content: impl IntoElement,
    attached: Option<AnyElement>,
    status: SharedString,
    buttons: Vec<AnyElement>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .child(Caption::new(
            "文本工具栏：撤销 / 重做 | 字体、字号 | 粗体、斜体、下划线 | 列表 | 缩进、对齐 | 附件、链接 | 溢出菜单；窄屏时右侧项收进溢出菜单",
        ))
        .child(toolbar)
        .children(link_row)
        .child(area(content, attached))
        .child(save_row(status, buttons))
}

fn save_buttons(prefix: &'static str, autosave: bool) -> Vec<Button> {
    let ids = match prefix {
        "ref" => ("tt-ref-save", "tt-ref-publish"),
        _ => ("tt-live-save", "tt-live-publish"),
    };
    let mut out = Vec::new();
    if !autosave {
        out.push(
            Button::new(ids.0, "Save draft")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Md),
        );
    }
    out.push(
        Button::new(ids.1, "Publish")
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Md),
    );
    out
}

/// `components/PatternTextToolbar/preview.html`.
pub fn reference() -> impl IntoElement {
    let toolbar = TextToolbar::new("tt-ref")
        .overflow_label("More")
        .overflow_items([u0ui::components::MenuItem::item("Strikethrough")])
        .items(items(
            "ref",
            Format {
                bold: true,
                ..Format::default()
            },
            |_, a| a,
        ));
    editor(
        toolbar,
        None,
        note(Format::default()),
        Some(attachment("tt-ref-remove", None)),
        "Saved 10:42".into(),
        save_buttons("ref", false)
            .into_iter()
            .map(IntoElement::into_any_element)
            .collect(),
    )
}

/// The overview tile: the toolbar's first groups.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            TextToolbar::new("tt-thumb")
                .overflow_label("More")
                .item(ToolbarItem::action(ToolbarAction::new(
                    "undo",
                    "Undo",
                    CarbonIcon::Undo,
                )))
                .item(ToolbarItem::action(ToolbarAction::new(
                    "redo",
                    "Redo",
                    CarbonIcon::Redo,
                )))
                .item(ToolbarItem::Separator)
                .item(ToolbarItem::action(
                    ToolbarAction::new("bold", "Bold (⌘B)", CarbonIcon::TextBold).selected(true),
                ))
                .item(ToolbarItem::action(ToolbarAction::new(
                    "italic",
                    "Italic (⌘I)",
                    CarbonIcon::TextItalic,
                )))
                .item(ToolbarItem::action(ToolbarAction::new(
                    "underline",
                    "Underline (⌘U)",
                    CarbonIcon::TextUnderline,
                )))
                .item(ToolbarItem::Spacer)
                .overflow_items([u0ui::components::MenuItem::item("Strikethrough")]),
        )
        .into_any_element()
}

impl TextToolbarPage {
    fn set_format(&mut self, next: Format) {
        if next != self.format {
            self.history.push(self.format);
            self.future.clear();
            self.format = next;
            self.touch();
        }
    }

    /// Autosave stamps the time on every change.
    fn touch(&mut self) {
        if self.autosave {
            self.stamp();
        }
    }

    fn stamp(&mut self) {
        let now = u0ui::clock::now();
        self.saved = format!("Saved {}:{:02}", now.hour, now.minute).into();
    }

    fn act(&mut self, id: &str) {
        let f = self.format;
        match id {
            "undo" => {
                if let Some(prev) = self.history.pop() {
                    self.future.push(self.format);
                    self.format = prev;
                    self.touch();
                }
            }
            "redo" => {
                if let Some(next) = self.future.pop() {
                    self.history.push(self.format);
                    self.format = next;
                    self.touch();
                }
            }
            "bold" => self.set_format(Format { bold: !f.bold, ..f }),
            "italic" => self.set_format(Format {
                italic: !f.italic,
                ..f
            }),
            "underline" => self.set_format(Format {
                underline: !f.underline,
                ..f
            }),
            "attach" => {
                self.attachment = true;
                self.touch();
            }
            "link" => self.link_open = !self.link_open,
            _ => {}
        }
    }
}

impl Render for TextToolbarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toolbar_items = items("live", self.format, |id, action| {
            let entity = entity.clone();
            let action = match id {
                "undo" => action.disabled(self.history.is_empty()),
                "redo" => action.disabled(self.future.is_empty()),
                "link" => action.selected(self.link_open),
                _ => action,
            };
            action.on_click(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.act(id);
                    cx.notify();
                })
            })
        });
        let toolbar = TextToolbar::new("tt-live")
            .overflow_label("More")
            .overflow_items([u0ui::components::MenuItem::item("Strikethrough")])
            .items(toolbar_items);

        let link_row = self.link_open.then(|| {
            let insert = {
                let entity = entity.clone();
                Button::new("tt-live-insert", "Insert")
                    .variant(ButtonVariant::Primary)
                    .size(ButtonSize::Md)
                    .on_click(move |_, _, cx| {
                        entity.update(cx, |this, cx| {
                            let url = this.link.read(cx).value().to_string();
                            this.linked = (!url.is_empty()).then(|| url.into());
                            this.link_open = false;
                            this.touch();
                            cx.notify();
                        })
                    })
            };
            div()
                .flex()
                .flex_row()
                .items_end()
                .bg(color(ColorRole::Layer01))
                .px(tokens::SPACING_05)
                .py(tokens::SPACING_03)
                .gap(tokens::SPACING_03)
                .child(
                    div().flex_1().min_w_0().child(
                        Input::new("tt-live-url", &self.link)
                            .label("Link URL")
                            .size(FieldSize::Md)
                            .layer(LayerLevel::Two),
                    ),
                )
                .child(insert)
                .into_any_element()
        });

        let link_color: Hsla = color(ColorRole::LinkPrimary);
        let content = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(note(Format {
                bold: false,
                ..self.format
            }))
            .when_some(self.linked.clone(), |el, url| {
                el.child(div().text_color(link_color).underline().child(url))
            });

        let mut buttons: Vec<AnyElement> = Vec::new();
        for button in save_buttons("live", self.autosave) {
            let entity = entity.clone();
            buttons.push(
                button
                    .on_click(move |_, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.stamp();
                            cx.notify();
                        })
                    })
                    .into_any_element(),
            );
        }

        let width = self.width;
        let live = div().w_full().min_w_0().child(
            div()
                .when(width == 1, |d| d.max_w(rems(36.)))
                .when(width == 2, |d| d.max_w(rems(22.)))
                .w_full()
                .child(editor(
                    toolbar,
                    link_row,
                    content,
                    self.attachment
                        .then(|| attachment("tt-live-remove", Some(entity.clone()))),
                    self.saved.clone(),
                    buttons,
                )),
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "width",
                controls::choices(
                    "tt-width",
                    &WIDTHS,
                    self.width,
                    controls::setter(&entity, |this: &mut Self, i| this.width = i),
                ),
            ))
            .child(controls::field(
                "save",
                controls::choices(
                    "tt-save",
                    &SAVE,
                    usize::from(self.autosave),
                    controls::setter(&entity, |this: &mut Self, i: usize| {
                        this.autosave = i == 1;
                        if this.autosave {
                            this.stamp();
                        }
                    }),
                ),
            ))
            .child(controls::toggle(
                "tt-attachment",
                "attachment",
                self.attachment,
                controls::setter(&entity, |this: &mut Self, on| this.attachment = on),
            ))
            .child(controls::toggle(
                "tt-link",
                "link field",
                self.link_open,
                controls::setter(&entity, |this: &mut Self, on| this.link_open = on),
            ));

        Body::new()
            .playground(live)
            .hint("←/→ move between buttons; narrow the width to push items into More")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
