//! T3 · Settings: EnvironmentRow, KeybindingRow, ProviderInstanceCard,
//! SettingsNav, SettingsRow, StatsGrid, ThemeCard, Wizard and RichText.
use super::{note, row, stack};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, Hsla, IntoElement, ParentElement,
    Render, SharedString, Styled, Window, div, prelude::FluentBuilder as _, px, rgb,
};
use u0ui::components::{FieldSize, Input, InputState, NativeSelect, Switch};
use u0ui::prelude::*;
use u0ui::t3::shell::{ConnectionStatus, MachineKind};
use u0ui::t3::workspace::{
    EnvironmentRow, KeybindingRow, ProviderInstanceCard, ProviderStatus, RichText, SettingsNav,
    SettingsNavItem, SettingsNavSection, SettingsRow, SettingsSection, StatBlock, StatTone,
    StatsGrid, ThemeCard, Wizard,
};

// ── EnvironmentRow ──────────────────────────────────────────────────────

struct EnvPage {
    kind: usize,
    status: usize,
    current: bool,
    detail: bool,
}

/// Build the EnvironmentRow page.
pub fn build_environment_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| EnvPage {
        kind: 2,
        status: 0,
        current: false,
        detail: true,
    })
    .into()
}

fn env_reference(prefix: &str) -> impl IntoElement {
    let id = |i: usize| SharedString::from(format!("{prefix}-{i}"));
    // `components/EnvironmentRow/preview.html` (`max-width: 560`).
    div()
        .flex()
        .flex_col()
        .max_w(px(560.))
        .child(
            EnvironmentRow::new(id(0), "MacBook Pro")
                .kind(MachineKind::Laptop)
                .transport("Local")
                .current(true),
        )
        .child(
            EnvironmentRow::new(id(1), "build-box")
                .kind(MachineKind::Server)
                .transport("SSH")
                .detail("10.0.4.12")
                .status(ConnectionStatus::Connected),
        )
        .child(
            EnvironmentRow::new(id(2), "Studio")
                .kind(MachineKind::Desktop)
                .transport("Relay")
                .status(ConnectionStatus::Offline)
                .detail("Last seen 2h ago"),
        )
        .child(
            EnvironmentRow::new(id(3), "cloud-1")
                .kind(MachineKind::Cloud)
                .transport("Tunnel")
                .status(ConnectionStatus::Error)
                .detail("Certificate expired"),
        )
}

/// The overview tile.
pub fn environment_row_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            EnvironmentRow::new("env-thumb", "build-box")
                .kind(MachineKind::Server)
                .transport("SSH"),
        )
        .into_any_element()
}

impl Render for EnvPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut live = EnvironmentRow::new("env-live", "build-box")
            .kind(MachineKind::ALL[self.kind])
            .transport("SSH")
            .status(ConnectionStatus::ALL[self.status])
            .current(self.current);
        if self.detail {
            live = live.detail("10.0.4.12");
        }
        let kinds = MachineKind::ALL.map(|k| k.name());
        let statuses = ConnectionStatus::ALL.map(|s| s.name());
        let config = stack(24.)
            .child(controls::field(
                "kind",
                controls::choices(
                    "env-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut EnvPage, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "status",
                controls::choices(
                    "env-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut EnvPage, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "env-current",
                "current",
                self.current,
                controls::setter(&entity, |this: &mut EnvPage, v| this.current = v),
            ))
            .child(controls::toggle(
                "env-detail",
                "detail",
                self.detail,
                controls::setter(&entity, |this: &mut EnvPage, v| this.detail = v),
            ));
        Body::new()
            .playground(div().w(px(560.)).max_w_full().child(live))
            .hint("Hover the status dot for its label")
            .config(config)
            .example(controls::block(
                "Reference preview",
                env_reference("env-ref"),
            ))
    }
}

// ── KeybindingRow ───────────────────────────────────────────────────────

struct KeysPage {
    custom: bool,
    when: bool,
    warning: bool,
    edits: usize,
}

/// Build the KeybindingRow page.
pub fn build_keybinding_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| KeysPage {
        custom: true,
        when: true,
        warning: false,
        edits: 0,
    })
    .into()
}

/// The overview tile.
pub fn keybinding_row_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    u0ui::t3::workspace::KeybindingPill::new("\u{2318}+\u{21e7}+Y").into_any_element()
}

impl Render for KeysPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut live = KeybindingRow::new(
            "keys-live",
            "Approve pending request",
            "\u{2318}+\u{21e7}+Y",
        )
        .custom(self.custom)
        .on_edit({
            let entity = entity.clone();
            move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.edits += 1;
                    cx.notify();
                })
            }
        });
        if self.when {
            live = live.when_clause("pendingApproval");
        }
        if self.warning {
            live = live.warning("Unknown when variable: pendingApproval");
        }
        let config = stack(24.)
            .child(controls::toggle(
                "keys-custom",
                "custom",
                self.custom,
                controls::setter(&entity, |this: &mut KeysPage, v| this.custom = v),
            ))
            .child(controls::toggle(
                "keys-when",
                "when",
                self.when,
                controls::setter(&entity, |this: &mut KeysPage, v| this.when = v),
            ))
            .child(controls::toggle(
                "keys-warning",
                "warning",
                self.warning,
                controls::setter(&entity, |this: &mut KeysPage, v| this.warning = v),
            ));
        // `components/KeybindingRow/preview.html` (`max-width: 760; background: layer-01`).
        let reference = div()
            .flex()
            .flex_col()
            .max_w(px(760.))
            .bg(color(ColorRole::Layer01))
            .child(KeybindingRow::new("keys-ref-0", "New thread", "\u{2318}+N"))
            .child(
                KeybindingRow::new("keys-ref-1", "Toggle terminal", "\u{2318}+J")
                    .when_clause("threadFocused"),
            )
            .child(
                KeybindingRow::new(
                    "keys-ref-2",
                    "Approve pending request",
                    "\u{2318}+\u{21e7}+Y",
                )
                .custom(true)
                .when_clause("pendingApproval"),
            )
            .child(
                KeybindingRow::new("keys-ref-3", "Open side chat", "\u{2325}+S")
                    .when_clause("unknownVar")
                    .warning("Unknown when variable: unknownVar"),
            );
        Body::new()
            .playground(
                stack(8.)
                    .w_full()
                    .child(
                        div()
                            .max_w(px(760.))
                            .bg(color(ColorRole::Layer01))
                            .child(live),
                    )
                    .when(self.edits > 0, |this| {
                        this.child(note(format!("Edit pressed {}×", self.edits)))
                    }),
            )
            .hint("Hover the warning glyph for its message")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── ProviderInstanceCard ────────────────────────────────────────────────

struct ProviderPage {
    status: usize,
    primary: bool,
    command: bool,
}

/// Build the ProviderInstanceCard page.
pub fn build_provider_instance_card(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ProviderPage {
        status: 1,
        primary: false,
        command: true,
    })
    .into()
}

/// The overview tile.
pub fn provider_instance_card_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            ProviderInstanceCard::new("prov-thumb", "Claude (work)", "Claude")
                .version("2.1.4")
                .models(6),
        )
        .into_any_element()
}

impl Render for ProviderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let status = ProviderStatus::ALL[self.status];
        let mut live = ProviderInstanceCard::new("prov-live", "Codex (personal)", "Codex")
            .status(status)
            .version("0.48.0")
            .models(if status == ProviderStatus::Ready {
                4
            } else {
                0
            })
            .primary(self.primary);
        if status == ProviderStatus::Ready {
            live = live.account("jd@acme.dev").env("MacBook Pro");
        }
        if self.command {
            live = live.login_command("codex login");
        }
        let statuses = ProviderStatus::ALL.map(|s| s.name());
        let config = stack(24.)
            .child(controls::field(
                "status",
                controls::choices(
                    "prov-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut ProviderPage, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "prov-primary",
                "primary (Run in terminal)",
                self.primary,
                controls::setter(&entity, |this: &mut ProviderPage, v| this.primary = v),
            ))
            .child(controls::toggle(
                "prov-command",
                "loginCommand",
                self.command,
                controls::setter(&entity, |this: &mut ProviderPage, v| this.command = v),
            ));
        // `components/ProviderInstanceCard/preview.html` (`cu-stack`, gap 12, max-width 560).
        let reference = stack(12.)
            .max_w(px(560.))
            .child(
                ProviderInstanceCard::new("prov-ref-0", "Claude (work)", "Claude")
                    .status(ProviderStatus::Ready)
                    .account("jd@acme.dev")
                    .version("2.1.4")
                    .models(6)
                    .env("MacBook Pro"),
            )
            .child(
                ProviderInstanceCard::new("prov-ref-1", "Codex", "Codex")
                    .status(ProviderStatus::Signin)
                    .models(0),
            )
            .child(
                ProviderInstanceCard::new("prov-ref-2", "Kiro", "Kiro")
                    .status(ProviderStatus::Error)
                    .models(0),
            );
        Body::new()
            .playground(div().w(px(560.)).max_w_full().child(live))
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── SettingsNav ─────────────────────────────────────────────────────────

fn nav_sections() -> Vec<SettingsNavSection> {
    let item = |id: &'static str, label: &'static str, icon: CarbonIcon| {
        SettingsNavItem::new(id, label).icon(icon)
    };
    vec![
        SettingsNavSection::new(
            "",
            [
                item("general", "General", CarbonIcon::Settings),
                item("appearance", "Appearance", CarbonIcon::ColorPalette),
                item("keys", "Keybindings", CarbonIcon::Keyboard),
            ],
        ),
        SettingsNavSection::new(
            "Agents",
            [
                item("providers", "Providers", CarbonIcon::Bot),
                item("voice", "Voice", CarbonIcon::Microphone).badge("New"),
                item("tasks", "Scheduled tasks", CarbonIcon::Time),
            ],
        ),
        SettingsNavSection::new(
            "Workspace",
            [
                item("env", "Environments", CarbonIcon::Laptop),
                item("git", "Source control", CarbonIcon::Branch),
                item("secrets", "Secrets", CarbonIcon::Password),
            ],
        ),
    ]
}

/// `display: inline-block; background: layer-01`: hugs its content.
fn inline_layer() -> gpui_kit::Div {
    div().flex_none().bg(color(ColorRole::Layer01))
}

struct NavPage {
    value: SharedString,
}

/// Build the SettingsNav page.
pub fn build_settings_nav(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| NavPage {
        value: "providers".into(),
    })
    .into()
}

/// The overview tile.
pub fn settings_nav_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .bg(color(ColorRole::Layer01))
        .child(
            SettingsNav::new("nav-thumb", nav_sections().into_iter().take(1)).value("appearance"),
        )
        .into_any_element()
}

impl Render for NavPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = stack(12.)
            .items_start()
            .child(
                inline_layer().child(
                    SettingsNav::new("nav-live", nav_sections())
                        .value(self.value.clone())
                        .on_value_change({
                            let entity = entity.clone();
                            move |v, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.value = v;
                                    cx.notify();
                                })
                            }
                        }),
                ),
            )
            .child(note(format!("value: {}", self.value)));
        let ids: Vec<&'static str> = vec![
            "general",
            "appearance",
            "keys",
            "providers",
            "voice",
            "tasks",
            "env",
            "git",
            "secrets",
        ];
        let current = ids.iter().position(|i| self.value == *i).unwrap_or(0);
        let config = controls::field(
            "value",
            controls::choices("nav-value", &ids, current, {
                let ids = ids.clone();
                controls::setter(&entity, move |this: &mut NavPage, i: usize| {
                    this.value = ids[i].into()
                })
            }),
        );
        // `components/SettingsNav/preview.html` (`background: layer-01; inline-block`).
        let reference = div().flex().child(
            inline_layer().child(SettingsNav::new("nav-ref", nav_sections()).value("providers")),
        );
        Body::new()
            .playground(live)
            .hint("Click an item or Tab to it and press Enter")
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block(
                "Uncontrolled (starts on the first item)",
                div().flex().child(inline_layer().child(SettingsNav::new(
                    "nav-free",
                    nav_sections().into_iter().take(2),
                ))),
            ))
    }
}

// ── SettingsRow ─────────────────────────────────────────────────────────

struct RowPage {
    modified: bool,
    overridden: bool,
    managed: bool,
    scope: bool,
    settle: bool,
}

/// Build the SettingsRow page.
pub fn build_settings_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RowPage {
        modified: true,
        overridden: false,
        managed: false,
        scope: true,
        settle: true,
    })
    .into()
}

/// `components/SettingsRow/preview.html`.
fn settings_reference(prefix: &'static str) -> impl IntoElement {
    let id = move |i: usize| SharedString::from(format!("{prefix}-{i}"));
    SettingsSection::new("Threads")
        .description("How new threads start and when idle ones settle.")
        .child(
            SettingsRow::new(id(0), "Default workspace")
                .description("Where new threads run unless you choose otherwise.")
                .control(
                    NativeSelect::new(id(10))
                        .options(["New worktree", "Local checkout"])
                        .size(FieldSize::Sm)
                        .layer(LayerLevel::Two),
                ),
        )
        .child(
            SettingsRow::new(id(1), "Settle idle threads")
                .description("Move threads with no activity to Settled.")
                .scope("Overridden for carbon-ui")
                .modified(true)
                .on_reset(|_, _| {})
                .control(Switch::new(id(11)).default_checked(true)),
        )
        .child(
            SettingsRow::new(id(2), "Send anonymous usage data")
                .description("Helps improve the app.")
                .managed(true)
                .control(Switch::new(id(12)).checked(false)),
        )
}

/// The overview tile.
pub fn settings_row_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .bg(color(ColorRole::Layer01))
        .child(
            SettingsRow::new("srow-thumb", "Settle idle threads")
                .modified(true)
                .control(Switch::new("srow-thumb-switch").default_checked(true)),
        )
        .into_any_element()
}

impl Render for RowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut live = SettingsRow::new("srow-live", "Settle idle threads")
            .description("Move threads with no activity to Settled.")
            .modified(self.modified)
            .overridden(self.overridden)
            .managed(self.managed)
            .inherited_value("On")
            .on_reset({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.modified = false;
                        this.overridden = false;
                        this.settle = false;
                        cx.notify();
                    })
                }
            })
            .control(
                Switch::new("srow-live-switch")
                    .checked(self.settle)
                    .on_checked_change({
                        let entity = entity.clone();
                        move |v, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.settle = v;
                                this.modified = v;
                                cx.notify();
                            })
                        }
                    }),
            );
        if self.scope {
            live = live.scope("Overridden for carbon-ui");
        }
        let config = stack(24.)
            .child(controls::toggle(
                "srow-modified",
                "modified",
                self.modified,
                controls::setter(&entity, |this: &mut RowPage, v| this.modified = v),
            ))
            .child(controls::toggle(
                "srow-overridden",
                "overridden",
                self.overridden,
                controls::setter(&entity, |this: &mut RowPage, v| this.overridden = v),
            ))
            .child(controls::toggle(
                "srow-managed",
                "managed",
                self.managed,
                controls::setter(&entity, |this: &mut RowPage, v| this.managed = v),
            ))
            .child(controls::toggle(
                "srow-scope",
                "scope",
                self.scope,
                controls::setter(&entity, |this: &mut RowPage, v| this.scope = v),
            ));
        Body::new()
            .playground(
                div().w_full().child(
                    SettingsSection::new("Threads")
                        .description("How new threads start and when idle ones settle.")
                        .child(live),
                ),
            )
            .hint("Turn the switch on to mark the row modified; Reset restores it")
            .config(config)
            .example(controls::block(
                "Reference preview",
                settings_reference("srow-ref"),
            ))
    }
}

// ── RichText ────────────────────────────────────────────────────────────

struct RichPage {
    element: bool,
}

/// Build the RichText page.
pub fn build_rich_text(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RichPage { element: true }).into()
}

/// Inline code as the reference's `<code>` (mono, inherited size).
fn code(text: &'static str) -> impl IntoElement {
    div()
        .font(u0ui::typography::font(
            u0ui::tokens::FontFamilyKey::Mono,
            400,
        ))
        .child(text)
}

fn rich_description() -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_baseline()
        .child("Runs ")
        .child(code("pnpm install"))
        .child(" in each new worktree.")
        .into_any_element()
}

/// The overview tile.
pub fn rich_text_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(RichText::from(rich_description()))
        .into_any_element()
}

impl Render for RichPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let description: RichText = if self.element {
            rich_description().into()
        } else {
            "Runs pnpm install in each new worktree.".into()
        };
        let live = div().w_full().bg(color(ColorRole::Layer01)).child(
            SettingsRow::new("rich-live", "Setup script")
                .description(description)
                .control(Switch::new("rich-live-switch").default_checked(true)),
        );
        let config = controls::toggle(
            "rich-element",
            "value is an element (inline code)",
            self.element,
            controls::setter(&entity, |this: &mut RichPage, v| this.element = v),
        );
        let fallback = stack(8.)
            .child(
                div()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(RichText::new().value("A plain string value.")),
            )
            .child(
                div()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(RichText::new().child("No value: the children render instead.")),
            );
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Value or children", fallback))
    }
}

// ── StatsGrid ───────────────────────────────────────────────────────────

struct StatsPage {
    count: usize,
}

/// Build the StatsGrid page.
pub fn build_stats_grid(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StatsPage { count: 4 }).into()
}

fn stat_blocks() -> Vec<StatBlock> {
    vec![
        StatBlock::new("Threads today", "42").sub("+8 vs yesterday"),
        StatBlock::new("Tokens", "3.1M").sub("68% cached"),
        StatBlock::new("Failed runs", "3")
            .tone(StatTone::Error)
            .sub("2 provider errors"),
        StatBlock::new("Avg. run", "2m 14s"),
        StatBlock::new("Uptime", "99.9%").tone(StatTone::Success),
    ]
}

/// The overview tile.
pub fn stats_grid_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(StatsGrid::new("stats-thumb").children(stat_blocks().into_iter().take(2)))
        .into_any_element()
}

impl Render for StatsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let counts = ["1", "2", "3", "4", "5"];
        let config = controls::field(
            "blocks",
            controls::choices(
                "stats-count",
                &counts,
                self.count,
                controls::setter(&entity, |this: &mut StatsPage, i| this.count = i),
            ),
        );
        Body::new()
            .playground(
                div().w_full().child(
                    StatsGrid::new("stats-live")
                        .children(stat_blocks().into_iter().take(self.count + 1)),
                ),
            )
            .hint("Resize the window: 9rem columns reflow")
            .config(config)
            .example(controls::block(
                "Reference preview",
                StatsGrid::new("stats-ref").children(stat_blocks()),
            ))
    }
}

// ── ThemeCard ───────────────────────────────────────────────────────────

fn palette(hex: [u32; 5]) -> Vec<Hsla> {
    hex.into_iter().map(|h| rgb(h).into()).collect()
}

fn theme_cards() -> [(&'static str, &'static str, Vec<Hsla>, Option<&'static str>); 3] {
    [
        (
            "White",
            "Light",
            palette([0xffffff, 0xf4f4f4, 0xc6c6c6, 0xe0e0e0, 0x0f62fe]),
            None,
        ),
        (
            "Gray 100",
            "Dark",
            palette([0x161616, 0x262626, 0x525252, 0x393939, 0x0f62fe]),
            None,
        ),
        (
            "Gray 90",
            "Dark",
            palette([0x262626, 0x393939, 0x6f6f6f, 0x525252, 0x4589ff]),
            Some("Carbon"),
        ),
    ]
}

struct ThemePage {
    selected: usize,
}

/// Build the ThemeCard page.
pub fn build_theme_card(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ThemePage { selected: 0 }).into()
}

/// The overview tile.
pub fn theme_card_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    let (name, mode, colors, _) = theme_cards().into_iter().next().unwrap_or_default();
    ThemeCard::new("theme-thumb", name)
        .mode(mode)
        .colors(colors)
        .selected(true)
        .into_any_element()
}

impl Render for ThemePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let cards =
            |prefix: &'static str, selected: Option<usize>, entity: Option<Entity<ThemePage>>| {
                row().children(theme_cards().into_iter().enumerate().map(
                    move |(i, (name, mode, colors, author))| {
                        let mut card =
                            ThemeCard::new(SharedString::from(format!("{prefix}-{i}")), name)
                                .mode(mode)
                                .colors(colors)
                                .selected(selected == Some(i));
                        if let Some(a) = author {
                            card = card.author(a);
                        }
                        if let Some(entity) = entity.clone() {
                            card = card.on_click(move |_, cx| {
                                entity.update(cx, |this, cx| {
                                    this.selected = i;
                                    cx.notify();
                                })
                            });
                        }
                        card
                    },
                ))
            };
        Body::new()
            .playground(cards("theme-live", Some(self.selected), Some(entity)))
            .hint("Click a card to select it")
            .example(controls::block(
                "Reference preview",
                cards("theme-ref", Some(0), None),
            ))
    }
}

// ── Wizard ──────────────────────────────────────────────────────────────

const WIZARD_STEPS: [&str; 3] = ["Choose agent", "Authenticate", "Pick models"];

struct WizardPage {
    current: usize,
    key: Entity<InputState>,
    reference_key: Entity<InputState>,
}

/// Build the Wizard page.
pub fn build_wizard(window: &mut Window, cx: &mut App) -> AnyView {
    let key = cx.new(|cx| InputState::new(window, cx));
    let reference_key = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| WizardPage {
        current: 1,
        key,
        reference_key,
    })
    .into()
}

/// The overview tile.
pub fn wizard_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    u0ui::t3::workspace::WizardSteps::new(WIZARD_STEPS, 1).into_any_element()
}

fn key_field(id: &'static str, state: &Entity<InputState>) -> impl IntoElement {
    // `.t3-wizard .cds--text-input { background: field-02 }`.
    Input::new(id, state)
        .label("API key")
        .password(true)
        .helper_text("Stored in your secret provider.")
        .layer(LayerLevel::Two)
}

impl Render for WizardPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let last = self.current + 1 == WIZARD_STEPS.len();
        let done = self.current >= WIZARD_STEPS.len();
        let live = if done {
            stack(12.)
                .child(note("Finished."))
                .child(
                    Button::new("wizard-restart", "Start again")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Tertiary)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.current = 0;
                                    cx.notify();
                                })
                            }
                        }),
                )
                .into_any_element()
        } else {
            let body: AnyElement = match self.current {
                1 => key_field("wizard-key", &self.key).into_any_element(),
                0 => super::text("Codex, Claude, Cursor or any agent CLI on your PATH.")
                    .into_any_element(),
                _ => super::text("6 models are available to this provider.").into_any_element(),
            };
            Wizard::new("wizard-live", "Add a provider")
                .steps(WIZARD_STEPS)
                .current(self.current)
                .next_label(if last { "Finish" } else { "Continue" })
                .on_back({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.current = this.current.saturating_sub(1);
                            cx.notify();
                        })
                    }
                })
                .on_next({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.current += 1;
                            cx.notify();
                        })
                    }
                })
                .child(body)
                .into_any_element()
        };
        let steps = ["0", "1", "2"];
        let config = controls::field(
            "current",
            controls::choices(
                "wizard-current",
                &steps,
                self.current.min(2),
                controls::setter(&entity, |this: &mut WizardPage, i| this.current = i),
            ),
        );
        // `components/Wizard/preview.html`.
        let reference = Wizard::new("wizard-ref", "Add a provider")
            .steps(WIZARD_STEPS)
            .current(1)
            .next_label("Continue")
            .child(key_field("wizard-ref-key", &self.reference_key));
        Body::new()
            .playground(div().w(px(576.)).max_w_full().child(live))
            .hint("Back and Continue move between steps")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
