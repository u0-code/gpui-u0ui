//! The gallery's landing page, after the reference cover
//! (`components/Cover/preview.html`): the system's name over Carbon's modular
//! blocks and dot grid, the four themes as live previews (each drawn in its
//! own theme zone; pick one to repaint the gallery), the ways in
//! (Foundations, Components, Patterns) and how to add the crate.
use crate::overview;
use crate::registry::{PAGES, Section};
use crate::shell::{Gallery, Route};
use gpui_kit::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, ParentElement, Styled, Window, div,
    prelude::FluentBuilder as _, px, rems,
};
use u0ui::components::switch::SwitchSize;
use u0ui::components::{
    Badge, BadgeSize, BadgeVariant, Card, CodeBlock, Progress, ProgressSize, Switch,
};
use u0ui::prelude::*;

/// `.cover { width: 960px; height: 288px }`: the art is the right half.
const ART_WIDTH: f32 = 480.;
const HERO_HEIGHT: f32 = 288.;
/// The art shows only when the text keeps its 440px beside it.
const ART_MIN_CONTENT: f32 = 960.;

const QUICK_START: &str = r#"[dependencies]
gpui-kit = "0.11"
u0ui = { git = "https://github.com/u0-code/gpui-u0ui" }

gpui_kit::application()
    .with_assets(u0ui::Assets::new())
    .run(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });"#;

/// One block of the cover art (`rect.tile`, square corners).
fn block(x: f32, y: f32, w: f32, h: f32, role: ColorRole) -> impl IntoElement {
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(w))
        .h(px(h))
        .bg(color(role))
}

/// The cover's art: four modules of `spacing-12` (96px) from x = 480 of the
/// 960px cover, and the 3px dot grid at a `spacing-07` (32px) pitch in
/// `border-strong-01`.
fn art() -> impl IntoElement {
    let mut art = div()
        .relative()
        .flex_shrink_0()
        .w(px(ART_WIDTH))
        .h(px(HERO_HEIGHT))
        // `.brand`, `.ink`, `.tint`, `.slate`.
        .child(block(0., 0., 192., 192., ColorRole::BackgroundBrand))
        .child(block(192., 0., 96., 96., ColorRole::TextPrimary))
        .child(block(192., 96., 192., 96., ColorRole::TagBackgroundBlue))
        .child(block(384., 96., 96., 192., ColorRole::ButtonSecondary));
    let dot = |x: f32, y: f32| {
        div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .size(px(3.))
            .bg(color(ColorRole::BorderStrong01))
    };
    for row in 0..3 {
        for col in 0..12 {
            art = art.child(dot(15. + 32. * col as f32, 207. + 32. * row as f32));
        }
        for col in 0..6 {
            art = art.child(dot(303. + 32. * col as f32, 15. + 32. * row as f32));
        }
    }
    art
}

fn hero(gallery: &Entity<Gallery>, width: f32) -> impl IntoElement {
    let go = |route: Route| {
        let gallery = gallery.clone();
        move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
            gallery.update(cx, |g, cx| g.navigate(route, cx))
        }
    };
    div()
        .flex()
        .flex_row()
        .min_h(px(HERO_HEIGHT))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .justify_end()
                .gap(tokens::SPACING_05)
                .pb(tokens::SPACING_02)
                .child(
                    // `.name { font-weight: 300; font-size: 80px; line-height: .92 }`.
                    div()
                        .text_size(rems(5.))
                        .line_height(rems(4.6))
                        .font_weight(FontWeight::LIGHT)
                        .text_color(color(ColorRole::TextPrimary))
                        .child("Carbon UI"),
                )
                .child(
                    // `.tag`: 14/20, `text-secondary`.
                    div()
                        .max_w(px(440.))
                        .type_style(TypeStyle::Body01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(
                            "IBM Carbon's design language with shadcn/ui's 66 components, \
                             drawn natively by u0ui on GPUI. Every page here is the real \
                             component: press, type and tab through it.",
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .pt(tokens::SPACING_05)
                        .child(
                            Button::new("home-components", "Browse components")
                                .icon(CarbonIcon::ArrowRight)
                                .on_click(go(Route::Overview(Some(Section::Components)))),
                        )
                        .child(
                            Button::new("home-foundations", "Foundations")
                                .variant(ButtonVariant::Tertiary)
                                .on_click(go(Route::Overview(Some(Section::Foundations)))),
                        ),
                ),
        )
        .when(width >= ART_MIN_CONTENT, |this| this.child(art()))
}

fn heading(title: &'static str, text: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .type_style(TypeStyle::Heading03)
                .text_color(color(ColorRole::TextPrimary))
                .child(title),
        )
        .child(
            div()
                .max_w(rems(40.))
                .type_style(TypeStyle::Body01)
                .text_color(color(ColorRole::TextSecondary))
                .child(text),
        )
}

/// A row of `items` in `columns` equal columns, wrapping into rows.
fn grid(items: Vec<AnyElement>, columns: usize) -> impl IntoElement {
    let mut grid = div().flex().flex_col().gap(tokens::SPACING_05);
    let mut items = items.into_iter().peekable();
    while items.peek().is_some() {
        let mut row = div().flex().flex_row().gap(tokens::SPACING_05);
        let mut n = 0;
        for item in items.by_ref().take(columns) {
            row = row.child(div().flex_1().min_w_0().flex().child(item));
            n += 1;
        }
        for _ in n..columns {
            row = row.child(div().flex_1().min_w_0());
        }
        grid = grid.child(row);
    }
    grid
}

/// A live preview of `theme`, in its own zone: pick it to repaint the gallery.
fn theme_card(theme: CarbonTheme) -> AnyElement {
    let current = theme::current() == theme;
    let name = crate::top_bar::theme_name(theme);
    theme::zone(theme, move |_, _| {
        div()
            .flex_1()
            .p(tokens::SPACING_05)
            .bg(color(ColorRole::Background))
            .border_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(
                Card::new(("theme-card", theme as usize))
                    .label(name)
                    .selectable(true)
                    .selected(current)
                    .on_selected_change(move |_, _, cx| theme::set_theme(cx, theme))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(tokens::SPACING_05)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(tokens::SPACING_02)
                                    .child(
                                        div()
                                            .type_style(TypeStyle::HeadingCompact02)
                                            .text_color(color(ColorRole::TextPrimary))
                                            .child(name),
                                    )
                                    .child(
                                        div()
                                            .type_style(TypeStyle::Label01)
                                            .text_color(color(ColorRole::TextSecondary))
                                            .child(if theme.is_dark() {
                                                "Dark theme"
                                            } else {
                                                "Light theme"
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .items_center()
                                    .gap(tokens::SPACING_04)
                                    .child(
                                        Button::new(("theme-deploy", theme as usize), "Deploy")
                                            .size(ButtonSize::Sm),
                                    )
                                    .child(
                                        Switch::new(("theme-switch", theme as usize))
                                            .size(SwitchSize::Sm)
                                            .default_checked(true),
                                    )
                                    .child(
                                        Badge::new("Ready")
                                            .variant(BadgeVariant::Green)
                                            .size(BadgeSize::Sm),
                                    ),
                            )
                            .child(
                                Progress::new(("theme-progress", theme as usize))
                                    .value(64.)
                                    .size(ProgressSize::Small)
                                    .layer(LayerLevel::Two),
                            ),
                    ),
            )
            .into_any_element()
    })
    .into_any_element()
}

/// A way in: a clickable tile for one section.
fn section_tile(section: Section, gallery: &Entity<Gallery>) -> AnyElement {
    let count = PAGES.iter().filter(|p| p.section() == section).count();
    let gallery = gallery.clone();
    Card::new(("home-section", section as usize))
        .label(section.title())
        .disabled(count == 0)
        .on_click(move |_, _, cx| {
            gallery.update(cx, |g, cx| g.navigate(Route::Overview(Some(section)), cx))
        })
        .flex_1()
        .min_h(rems(12.))
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .child(Icon::named(section.icon()).size(IconSize::S32).color(color(
                    if count == 0 {
                        ColorRole::IconDisabled
                    } else {
                        ColorRole::IconPrimary
                    },
                )))
                .child(
                    div()
                        .type_style(TypeStyle::Heading03)
                        .text_color(color(if count == 0 {
                            ColorRole::TextDisabled
                        } else {
                            ColorRole::TextPrimary
                        }))
                        .child(section.title()),
                )
                .child(
                    div()
                        .flex_1()
                        .type_style(TypeStyle::Body01)
                        .text_color(color(if count == 0 {
                            ColorRole::TextDisabled
                        } else {
                            ColorRole::TextSecondary
                        }))
                        .child(section.summary()),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(if count == 0 {
                            ColorRole::TextDisabled
                        } else {
                            ColorRole::TextSecondary
                        }))
                        .child(if count == 0 {
                            "Arriving with the next waves".to_string()
                        } else {
                            format!("{count} pages")
                        })
                        .child(
                            Icon::carbon(CarbonIcon::ArrowRight)
                                .size(IconSize::S20)
                                .color(color(if count == 0 {
                                    ColorRole::IconDisabled
                                } else {
                                    ColorRole::IconInteractive
                                })),
                        ),
                ),
        )
        .into_any_element()
}

/// Draw the home page.
pub fn render(
    gallery: Entity<Gallery>,
    window: &mut Window,
    _: &mut Context<Gallery>,
) -> AnyElement {
    let width = overview::content_width(window.viewport_size().width.into());
    let theme_columns = if width >= 1000. {
        4
    } else if width >= 480. {
        2
    } else {
        1
    };
    let section_columns = if width >= 720. { 3 } else { 1 };
    let facts = [
        (
            "contrast",
            "Four Carbon themes, White to G100, with layer-aware tokens",
        ),
        (
            "density_medium",
            "Desktop density: compact rows and fields by default",
        ),
        (
            "interests",
            "Material Symbols Sharp everywhere, Carbon names kept",
        ),
        (
            "keyboard",
            "Kit behavior: focus, keyboard, IME, overlays, scrolling",
        ),
    ];
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_10)
        .child(hero(&gallery, width))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_06)
                .child(heading(
                    "Four themes",
                    "Carbon's White, G10, G90 and G100, each preview drawn in its own theme. Pick one and the whole gallery repaints.",
                ))
                .child(grid(
                    CarbonTheme::ALL.into_iter().map(theme_card).collect(),
                    theme_columns,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_06)
                .child(heading(
                    "Explore",
                    "Start from the foundations, browse the components, or see them composed into patterns.",
                ))
                .child(grid(
                    Section::ALL
                        .into_iter()
                        .map(|s| section_tile(s, &gallery))
                        .collect(),
                    section_columns,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_06)
                .child(heading(
                    "Add it to your app",
                    "One crate on top of GPUI Kit 0.11. Kit owns behavior; u0ui owns the Carbon look.",
                ))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(tokens::SPACING_07)
                        .child(
                            div()
                                .flex_1()
                                .min_w(rems(20.))
                                .child(CodeBlock::new("home-quick-start", QUICK_START).title("main.rs")),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(rems(20.))
                                .flex()
                                .flex_col()
                                .gap(tokens::SPACING_05)
                                .children(facts.into_iter().map(|(icon, text)| {
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(tokens::SPACING_04)
                                        .type_style(TypeStyle::Body01)
                                        .text_color(color(ColorRole::TextPrimary))
                                        .child(
                                            Icon::named(icon)
                                                .size(IconSize::S20)
                                                .color(color(ColorRole::IconSecondary)),
                                        )
                                        .child(text)
                                })),
                        ),
                ),
        )
        .into_any_element()
}
