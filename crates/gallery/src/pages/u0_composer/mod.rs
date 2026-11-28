//! u0 · Composer pages (`src/ghost/workspace.jsx`, "Composer additions").
//! ComposerChip, ComposerCompact, MentionList and ModeMenu (+ ModeList),
//! each live with its configuration plus the reference preview.
use crate::page::Body;
use crate::{controls, stand_in};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, prelude::FluentBuilder as _,
    px, rems,
};
use u0ui::icons::CarbonIcon;
use u0ui::prelude::*;
use u0ui::u0::composer::{
    ChipKind, ChipState, ComposerChip, ComposerCompact, MentionGroup, MentionItem, MentionKind,
    MentionList, Mode, ModeList, ModeMenu,
};

fn log(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

// ── ComposerChip ────────────────────────────────────────────────────────

/// `components/ComposerChip/preview.html`: five chips, 360px, 8px apart.
fn chip_reference() -> impl IntoElement {
    div()
        .w_full()
        .max_w(rems(22.5))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            ComposerChip::new("cc-0", ChipKind::Quote, "From the previous reply")
                .excerpt("Layer tokens are contextual; use the Layer component to step down.")
                .on_remove(|_, _| {}),
        )
        .child(
            ComposerChip::new("cc-1", ChipKind::TerminalOutput, "pnpm test · exit 1")
                .excerpt("✗ src/date-picker.test.ts › keyboard")
                .on_remove(|_, _| {}),
        )
        .child(
            ComposerChip::new("cc-2", ChipKind::ReviewComment, "src/theme/g100.ts:41")
                .meta("Ana")
                .excerpt("Should this fall back to layer-accent-02?")
                .on_remove(|_, _| {}),
        )
        .child(
            ComposerChip::new("cc-3", ChipKind::PullRequest, "#482 Add G90 theme")
                .state(ChipState::Open)
                .on_remove(|_, _| {}),
        )
        .child(
            ComposerChip::new("cc-4", ChipKind::Issue, "#77 Dark mode contrast")
                .state(ChipState::Closed)
                .on_remove(|_, _| {}),
        )
}

/// The ComposerChip overview tile.
pub fn chip_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            ComposerChip::new("cc-thumb", ChipKind::Quote, "From the previous reply")
                .excerpt("Layer tokens are contextual.")
                .on_remove(|_, _| {}),
        )
        .into_any_element()
}

struct ChipPage {
    kind: usize,
    state: usize,
    removable: bool,
    log: SharedString,
}

/// Build the ComposerChip page.
pub fn build_composer_chip(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ChipPage {
        kind: 0,
        state: 0,
        removable: true,
        log: "—".into(),
    })
    .into()
}

impl Render for ChipPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (open, remove) = (e.clone(), e.clone());
        let kinds: Vec<&'static str> = ChipKind::ALL.iter().map(|k| k.name()).collect();
        let mut states: Vec<&'static str> = vec!["none"];
        states.extend(ChipState::ALL.iter().map(|s| s.name()));
        let kind = ChipKind::ALL[self.kind];
        let live = div()
            .w_full()
            .max_w(rems(22.5))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                ComposerChip::new("cc-live", kind, "src/theme/g100.ts:41")
                    .meta("Ana")
                    .excerpt("Should this fall back to layer-accent-02, or should G100 gain a third accent token of its own?")
                    .when(self.state > 0, |c| c.state(ChipState::ALL[self.state - 1]))
                    .on_open(move |_, cx| {
                        open.update(cx, |p, cx| {
                            p.log = "onOpen".into();
                            cx.notify();
                        })
                    })
                    .when(self.removable, |c| {
                        c.on_remove(move |_, cx| {
                            remove.update(cx, |p, cx| {
                                p.log = "onRemove".into();
                                cx.notify();
                            })
                        })
                    }),
            )
            .child(log(format!("last: {}", self.log)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "kind",
                controls::choices(
                    "cc-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&e, |p: &mut ChipPage, v| p.kind = v),
                ),
            ))
            .child(controls::field(
                "state",
                controls::choices(
                    "cc-state",
                    &states,
                    self.state,
                    controls::setter(&e, |p: &mut ChipPage, v| p.state = v),
                ),
            ))
            .child(controls::toggle(
                "cc-rm",
                "onRemove",
                self.removable,
                controls::setter(&e, |p: &mut ChipPage, v| p.removable = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", chip_reference()))
    }
}

// ── ComposerCompact ─────────────────────────────────────────────────────

/// The ComposerCompact overview tile.
pub fn compact_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(ComposerCompact::new("ccp-thumb", "Claude").on_attach(|_, _| {}))
        .into_any_element()
}

struct CompactPage {
    agent: usize,
    stacked: bool,
    sent: Vec<SharedString>,
}

const AGENTS: [&str; 3] = ["Claude", "Codex", "Gemini"];

/// Build the ComposerCompact page.
pub fn build_composer_compact(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CompactPage {
        agent: 0,
        stacked: false,
        sent: Vec::new(),
    })
    .into()
}

impl Render for CompactPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let send = e.clone();
        let live = div()
            .w_full()
            .max_w(rems(21.25))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                ComposerCompact::new("ccp-live", AGENTS[self.agent])
                    .stacked(self.stacked)
                    .on_attach(|_, _| {})
                    .on_send(move |text, _, cx| {
                        send.update(cx, |p, cx| {
                            p.sent.push(text);
                            cx.notify();
                        })
                    }),
            )
            .child(log(format!(
                "onSend: {}",
                self.sent
                    .last()
                    .map_or("—".to_string(), |t| format!("“{t}”"))
            )));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "agent",
                controls::choices(
                    "ccp-agent",
                    &AGENTS,
                    self.agent,
                    controls::setter(&e, |p: &mut CompactPage, v| p.agent = v),
                ),
            ))
            .child(controls::toggle(
                "ccp-stacked",
                "stacked",
                self.stacked,
                controls::setter(&e, |p: &mut CompactPage, on| p.stacked = on),
            ));
        Body::new()
            .playground(live)
            .hint("Type and press Enter to send")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(rems(21.25))
                    .child(ComposerCompact::new("ccp-ref", "Claude").on_attach(|_, _| {})),
            ))
            .example(controls::block(
                "An app's own mark, held while the agent answers",
                div().w_full().max_w(rems(21.25)).child(
                    ComposerCompact::new("ccp-held", "Claude")
                        .lead(Icon::carbon(CarbonIcon::Bot).size(IconSize::S16))
                        .disabled(true)
                        .placeholder("Ask Sonnet on the side…"),
                ),
            ))
            .example(controls::block(
                "Stacked, in a 400px side chat (SplitSideChat): input on top, controls below",
                div().w(px(368.)).child(
                    ComposerCompact::new("ccp-stacked", "Claude")
                        .stacked(true)
                        .placeholder("Ask Sonnet on the side…")
                        .on_attach(|_, _| {})
                        .on_send(|_, _, _| {}),
                ),
            ))
    }
}

// ── MentionList ─────────────────────────────────────────────────────────

fn mention_groups() -> Vec<MentionGroup> {
    vec![
        MentionGroup::new(
            "Files",
            MentionKind::File,
            [
                MentionItem::new("src/theme/tokens.ts").detail("modified"),
                MentionItem::new("scripts/build-tokens.py"),
            ],
        ),
        MentionGroup::new(
            "Sessions",
            MentionKind::Session,
            [
                MentionItem::new("Add dark theme tokens").detail("2m ago"),
                MentionItem::new("Token naming review").detail("3d ago"),
            ],
        ),
    ]
}

/// The MentionList overview tile.
pub fn mention_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .overflow_hidden()
        .child(MentionList::new(
            "tok",
            mention_groups().into_iter().take(1),
        ))
        .into_any_element()
}

struct MentionPage {
    query: Entity<InputState>,
    active: usize,
}

/// Build the MentionList page.
pub fn build_mention_list(window: &mut Window, cx: &mut App) -> AnyView {
    let query = stand_in::input_state("tok", window, cx);
    cx.new(|cx| {
        cx.observe(&query, |_, _, cx| cx.notify()).detach();
        MentionPage { query, active: 0 }
    })
    .into()
}

impl Render for MentionPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let query = self.query.read(cx).value().to_string();
        // The host filters: keep the candidates the query matches.
        let groups: Vec<MentionGroup> = mention_groups()
            .into_iter()
            .map(|mut g| {
                g.items
                    .retain(|it| u0ui::u0::primitives::fuzzy_match(&it.label, &query).is_some());
                g
            })
            .collect();
        let live = MentionList::new(query.clone(), groups).active_index(self.active);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(stand_in::text_field(
                "ml-query",
                "query",
                &self.query,
                LayerLevel::Two,
            ))
            .child(controls::field(
                "activeIndex",
                controls::choices(
                    "ml-active",
                    &["0", "1", "2", "3"],
                    self.active,
                    controls::setter(&e, |p: &mut MentionPage, v| p.active = v),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Type a query; the host filters and moves activeIndex")
            .config(config)
            .example(controls::block(
                "Reference preview",
                MentionList::new("tok", mention_groups()),
            ))
    }
}

// ── ModeMenu ────────────────────────────────────────────────────────────

fn menu_modes() -> Vec<Mode> {
    vec![
        Mode::new("d", "Default"),
        Mode::new("p", "Plan"),
        Mode::new("a", "Auto-accept edits"),
    ]
}

fn list_modes() -> Vec<Mode> {
    vec![
        Mode::new("d", "Default")
            .description("Asks before edits and commands")
            .icon(CarbonIcon::Chat),
        Mode::new("p", "Plan")
            .description("Reads and plans, changes nothing")
            .icon(CarbonIcon::ListChecked)
            .shortcut("⇧ ⇥"),
        Mode::new("a", "Auto-accept edits")
            .description("Edits files without asking")
            .icon(CarbonIcon::FlashFilled),
    ]
}

/// `components/ModeMenu/preview.html`: the menu button and the list, 24px apart.
fn mode_reference() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_06)
        .child(ModeMenu::new("mm-ref", menu_modes()))
        .child(ModeList::new("mlist-ref", list_modes()))
}

/// The ModeMenu overview tile.
pub fn mode_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(ModeMenu::new("mm-thumb", menu_modes()))
        .into_any_element()
}

struct ModePage {
    mode: SharedString,
}

/// Build the ModeMenu page.
pub fn build_mode_menu(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ModePage { mode: "d".into() }).into()
}

impl Render for ModePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (a, b) = (e.clone(), e.clone());
        let set = |e: Entity<ModePage>| {
            move |v: SharedString, _: &mut Window, cx: &mut App| {
                e.update(cx, |p, cx| {
                    p.mode = v;
                    cx.notify();
                })
            }
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .gap(tokens::SPACING_06)
                    .child(
                        ModeMenu::new("mm-live", list_modes())
                            .value(self.mode.clone())
                            .on_value_change(set(a)),
                    )
                    .child(
                        ModeList::new("mlist-live", list_modes())
                            .value(self.mode.clone())
                            .on_value_change(set(b)),
                    ),
            )
            .child(log(format!(
                "value: {} (both controlled by one state)",
                self.mode
            )));
        Body::new()
            .playground(live)
            .hint("Pick from the menu or the list; Tab to a list option and press Enter")
            .example(controls::block("Reference preview", mode_reference()))
    }
}
