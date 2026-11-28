//! T3 · Composer › ModelPicker: Claude (Opus default, Sonnet, Haiku), Codex
//! starting and Cursor needing sign-in, with effort and speed
//! (`components/ModelPicker/preview.html`), and what a host adds: a
//! favourites tab, stars, key hints, an option of three choices as a menu,
//! and a foot. The picker is live: tabs switch providers, the search
//! filters, rows, effort levels, the switch and the menu choose; the room
//! control narrows it until its tabs are only their marks.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use u0ui::components::InputState;
use u0ui::icons::CarbonIcon;
use u0ui::prelude::*;
use u0ui::t3::composer::{
    EffortLevel, ModelPicker, PickerChoice, PickerModel, PickerOption, PickerProvider,
    PickerSwitch, filter_models,
};

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let query = cx.new(|cx| InputState::new(window, cx));
    cx.new(|cx| {
        cx.observe(&query, |_, _, cx| cx.notify()).detach();
        PickerPage {
            query,
            tab: "a".into(),
            value: "opus".into(),
            effort: EffortLevel::High,
            fast: false,
            sandbox: 1,
            menu: false,
            stars: vec!["opus".into()],
            all: false,
            narrow: false,
        }
    })
    .into()
}

struct PickerPage {
    query: Entity<InputState>,
    tab: SharedString,
    value: SharedString,
    effort: EffortLevel,
    fast: bool,
    sandbox: usize,
    menu: bool,
    stars: Vec<SharedString>,
    all: bool,
    narrow: bool,
}

/// The sandbox option's choices.
const SANDBOX: [&str; 3] = ["Read-only", "Workspace", "Full access"];

/// A preview model: its id, name and description.
type Model = (&'static str, &'static str, &'static str);

/// The preview's providers and their models, by tab.
fn catalogue() -> Vec<(PickerProvider, Vec<Model>)> {
    vec![
        (
            PickerProvider::new("a", "Claude"),
            vec![
                ("opus", "Opus", "Most capable for complex work"),
                ("sonnet", "Sonnet", "Fast and capable"),
                ("haiku", "Haiku", "Fastest"),
            ],
        ),
        (
            PickerProvider::new("b", "Codex").starting(true),
            vec![("gpt", "GPT", "General coding model")],
        ),
        (
            PickerProvider::new("c", "Cursor").notice("Sign in again to use Cursor models."),
            vec![("auto", "Auto", "Lets Cursor pick")],
        ),
    ]
}

impl PickerPage {
    /// The rows of the tab chosen, or every provider's that match the search.
    fn rows(&self, query: &str) -> Vec<PickerModel> {
        let star = |id: &str| self.stars.iter().any(|s| s == id);
        let mut rows: Vec<PickerModel> = catalogue()
            .into_iter()
            .filter(|(p, _)| !query.is_empty() || self.tab == p.id || self.tab == "fav")
            .flat_map(|(p, models)| {
                let name = p.name.clone();
                models
                    .into_iter()
                    .map(move |(id, n, d)| (name.clone(), PickerModel::new(id, n).description(d)))
            })
            .filter(|(_, m)| self.tab != "fav" || !query.is_empty() || star(&m.id))
            .map(|(provider, m)| {
                let mark = provider.clone();
                let on = star(&m.id);
                let opus = m.id.as_ref() == "opus";
                let chosen = m.id == self.value;
                let m = m.chosen(chosen).favourite(on);
                let m = if opus { m.badge("Default") } else { m };
                if self.tab == "fav" || !query.is_empty() {
                    m.mark(std::rc::Rc::new(move |size| {
                        u0ui::t3::shell::ProviderIcon::new(mark.clone())
                            .size(if size > 16. {
                                u0ui::t3::shell::IdentitySize::Sm
                            } else {
                                u0ui::t3::shell::IdentitySize::Xs
                            })
                            .into_any_element()
                    }))
                } else {
                    m
                }
            })
            .collect();
        if !query.is_empty() {
            let found: Vec<SharedString> = filter_models(&rows, query)
                .into_iter()
                .map(|m| m.id.clone())
                .collect();
            rows.retain(|m| found.contains(&m.id));
        }
        for (n, m) in rows.iter_mut().enumerate().take(9) {
            m.key = Some(format!("⌘{}", n + 1).into());
        }
        rows
    }
}

impl Render for PickerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let query = self.query.read(cx).value().trim().to_string();
        let mut providers =
            vec![PickerProvider::new("fav", "Favourites").icon(CarbonIcon::StarFilled)];
        providers.extend(catalogue().into_iter().map(|(p, _)| p));
        let levels = PickerOption::new(
            "effort",
            "Effort",
            EffortLevel::ALL
                .into_iter()
                .map(|l| {
                    let n = l.name();
                    let name = format!("{}{}", n[..1].to_uppercase(), &n[1..]);
                    PickerChoice::new(n, name, l == self.effort)
                })
                .collect(),
        );
        let sandbox = PickerOption::new(
            "sandbox",
            "Sandbox",
            SANDBOX
                .iter()
                .enumerate()
                .map(|(i, s)| PickerChoice::new(*s, *s, i == self.sandbox))
                .collect(),
        );
        let fanout = entity.clone();
        let picker = ModelPicker::new("mpick-live", providers, &self.query)
            .provider(query.is_empty().then(|| self.tab.clone()))
            .models(self.rows(&query))
            .levels(Some(levels))
            .speed(Some(PickerSwitch {
                label: "Fast mode".into(),
                on: self.fast,
            }))
            .options(vec![sandbox])
            .open_option(self.menu.then(|| "sandbox".into()))
            .fanout("Send to all 3 agents", None, self.all, move |_, cx| {
                fanout.update(cx, |this, cx| {
                    this.all = !this.all;
                    cx.notify();
                })
            })
            // The room control: 480px narrows the tabs to their marks.
            .when(self.narrow, |p| p.max_width(px(480.)))
            .on_settings(|_, _| {})
            .on_provider_change(controls::setter(&entity, |this: &mut Self, t| this.tab = t))
            .on_choose(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }))
            .on_favourite(controls::setter(&entity, |this: &mut Self, v| {
                match this.stars.iter().position(|s| *s == v) {
                    Some(at) => {
                        this.stars.remove(at);
                    }
                    None => this.stars.push(v),
                }
            }))
            .on_option_change({
                let entity = entity.clone();
                move |option, value, _, cx| {
                    entity.update(cx, |this, cx| {
                        match option.as_ref() {
                            "effort" => {
                                if let Some(l) =
                                    EffortLevel::ALL.into_iter().find(|l| value == l.name())
                                {
                                    this.effort = l;
                                }
                            }
                            _ => {
                                this.sandbox =
                                    SANDBOX.iter().position(|s| value == *s).unwrap_or(0);
                                this.menu = false;
                            }
                        }
                        cx.notify();
                    })
                }
            })
            .on_option_open(controls::setter(
                &entity,
                |this: &mut Self, o: Option<_>| this.menu = o.is_some(),
            ))
            .on_speed_change(controls::setter(&entity, |this: &mut Self, on| {
                this.fast = on
            }));
        let summary: SharedString = format!(
            "value: {} · effort: {} · fast: {} · sandbox: {}",
            self.value,
            self.effort.name(),
            self.fast,
            SANDBOX[self.sandbox]
        )
        .into();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(picker)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(summary),
            );
        let levels = EffortLevel::ALL.map(|l| l.name());
        let current = EffortLevel::ALL
            .iter()
            .position(|l| *l == self.effort)
            .unwrap_or(2);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "effort",
                controls::choices(
                    "mpick-effort",
                    &levels,
                    current,
                    controls::setter(&entity, |this: &mut Self, i| {
                        this.effort = EffortLevel::ALL[i]
                    }),
                ),
            ))
            .child(controls::toggle(
                "mpick-fast",
                "fast",
                self.fast,
                controls::setter(&entity, |this: &mut Self, on| this.fast = on),
            ))
            .child(controls::toggle(
                "mpick-narrow",
                "maxWidth 480px",
                self.narrow,
                controls::setter(&entity, |this: &mut Self, on| this.narrow = on),
            ));
        Body::new()
            .playground(live)
            .hint("Tab through providers, models and options; Enter or Space chooses a model.")
            .config(config)
    }
}

/// The overview tile.
pub fn thumbnail(window: &mut Window, cx: &mut App) -> AnyElement {
    let query = window
        .use_keyed_state("mpick-th-query", cx, |window, cx| {
            cx.new(|cx| InputState::new(window, cx))
        })
        .read(cx)
        .clone();
    let providers = catalogue().into_iter().map(|(p, _)| p).collect();
    div()
        .w(gpui_kit::rems(15.))
        .h(gpui_kit::rems(8.))
        .overflow_hidden()
        .child(
            ModelPicker::new("mpick-th", providers, &query)
                .provider(Some("a".into()))
                .models(vec![
                    PickerModel::new("opus", "Opus")
                        .description("Most capable for complex work")
                        .badge("Default")
                        .chosen(true),
                    PickerModel::new("sonnet", "Sonnet").description("Fast and capable"),
                ]),
        )
        .into_any_element()
}
