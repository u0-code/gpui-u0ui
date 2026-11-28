//! [`ModelPicker`] (`.t3-mpick`, "Model picker" in `t3.css` with the
//! gemini-B and d-fix rules): the floating model chooser.
//!
//! Three columns on a `layer-01` surface with the floating shadow, 22rem
//! tall and at most 46rem wide, over an optional foot:
//! - **Providers** (11rem): vertical tabs (a provider's mark or an icon,
//!   its name, a short note such as "Needs setup", and a starting
//!   provider's ring at the row's right edge, per the d-fix rule), the
//!   chosen one on `layer-selected-01` with a 3px `border-interactive` rail
//!   inside its own start edge, and Agent settings at the bottom. Where the
//!   picker is given less room than its full width ([`ModelPicker::max_width`])
//!   the tabs narrow to their marks, each name a tooltip.
//! - **Models**: a search field, the provider's warning notice (with an
//!   action), and the rows the caller lists: a mark, the name, a badge and
//!   a description, then a favourite star, an "also run" toggle, a check
//!   when chosen and a key hint; hover or the keyboard's row
//!   `layer-hover-01`, chosen `layer-selected-01`; Enter / Space choose.
//! - **Options** (10rem), each as its agent states it, nothing assumed of
//!   a choice the agent did not report: a level option (such as Reasoning
//!   effort) as rows with the effort glyph and a check; Speed as its own
//!   group, the small switch labelled with the option's own name (Carbon's
//!   `hideLabel` puts `labelText` beside the track); any other option as a
//!   row of its name over its value, opening a menu of its choices. The
//!   column is left out when there is nothing to put in it.
//! - **Foot**: "Send to all" at the start, a slot (the mode) at the end.
//!
//! The picker draws what it is given: the tab chosen, the rows listed, the
//! row the keyboard is on and the search field are the caller's.
use crate::components::InputState;
use crate::components::badge::{Badge, BadgeSize, BadgeVariant};
use crate::components::kbd::Kbd;
use crate::components::overlay::floating_shadow;
use crate::t3::press::{self, Handler, rem};
use crate::t3::shell::{IdentitySize, PROVIDER_SPIN_PERIOD, ProviderIcon};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use crate::{motion, t3::base};
use gpui_kit::component::Sizable as _;
use gpui_kit::{
    AnyElement, App, Bounds, ElementId, Entity, InteractiveElement as _, IntoElement, KeyDownEvent,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, canvas, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

mod items;
mod options;

pub use items::{
    MODEL_PICKER_GEOMETRY, MODEL_PICKER_MIN_LIST, MODEL_PICKER_NARROW_SIDE, ModelPickerTexts,
    PickerChoice, PickerMark, PickerModel, PickerOption, PickerProvider, PickerSwitch,
    filter_models, narrow_tabs,
};

type TextHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type OptionHandler = Rc<dyn Fn(SharedString, SharedString, &mut Window, &mut App)>;
type OpenHandler = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;
type BoolHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// "Send to all": its label, what hovering it says, whether it is on.
struct Fanout {
    label: SharedString,
    tip: Option<SharedString>,
    on: bool,
    handler: Handler,
}

/// The model chooser. See the module docs.
#[derive(IntoElement)]
pub struct ModelPicker {
    id: ElementId,
    providers: Vec<PickerProvider>,
    provider: Option<SharedString>,
    models: Vec<PickerModel>,
    query: Entity<InputState>,
    highlighted: Option<usize>,
    levels: Option<PickerOption>,
    speed: Option<PickerSwitch>,
    options: Vec<PickerOption>,
    open_option: Option<SharedString>,
    fanout: Option<Fanout>,
    mode: Option<AnyElement>,
    max_width: Option<Pixels>,
    texts: ModelPickerTexts,
    on_provider_change: Option<TextHandler>,
    on_choose: Option<TextHandler>,
    on_highlight: Option<IndexHandler>,
    on_favourite: Option<TextHandler>,
    on_also: Option<TextHandler>,
    on_option_change: Option<OptionHandler>,
    on_option_open: Option<OpenHandler>,
    on_speed_change: Option<BoolHandler>,
    on_notice_action: Option<TextHandler>,
    on_settings: Option<Handler>,
}

impl ModelPicker {
    /// A picker over the provider tabs `providers`, its search typed into
    /// `query`.
    pub fn new(
        id: impl Into<ElementId>,
        providers: Vec<PickerProvider>,
        query: &Entity<InputState>,
    ) -> Self {
        Self {
            id: id.into(),
            providers,
            provider: None,
            models: Vec::new(),
            query: query.clone(),
            highlighted: None,
            levels: None,
            speed: None,
            options: Vec::new(),
            open_option: None,
            fanout: None,
            mode: None,
            max_width: None,
            texts: ModelPickerTexts::default(),
            on_provider_change: None,
            on_choose: None,
            on_highlight: None,
            on_favourite: None,
            on_also: None,
            on_option_change: None,
            on_option_open: None,
            on_speed_change: None,
            on_notice_action: None,
            on_settings: None,
        }
    }

    /// The tab chosen, by id (none while a search lists every provider's).
    pub fn provider(mut self, id: Option<SharedString>) -> Self {
        self.provider = id;
        self
    }

    /// The rows the list shows.
    pub fn models(mut self, models: Vec<PickerModel>) -> Self {
        self.models = models;
        self
    }

    /// The row the keyboard is on.
    pub fn highlighted(mut self, row: Option<usize>) -> Self {
        self.highlighted = row;
        self
    }

    /// A level option (such as Reasoning effort), its name the column's head.
    pub fn levels(mut self, levels: Option<PickerOption>) -> Self {
        self.levels = levels;
        self
    }

    /// Speed's switch.
    pub fn speed(mut self, speed: Option<PickerSwitch>) -> Self {
        self.speed = speed;
        self
    }

    /// The other options, each a row opening a menu of its choices.
    pub fn options(mut self, options: Vec<PickerOption>) -> Self {
        self.options = options;
        self
    }

    /// The option whose menu is open, by id.
    pub fn open_option(mut self, id: Option<SharedString>) -> Self {
        self.open_option = id;
        self
    }

    /// "Send to all" in the foot: `label`, what hovering it says, and
    /// whether it is on.
    pub fn fanout(
        mut self,
        label: impl Into<SharedString>,
        tip: Option<SharedString>,
        on: bool,
        f: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.fanout = Some(Fanout {
            label: label.into(),
            tip,
            on,
            handler: Rc::new(f),
        });
        self
    }

    /// What the foot holds at its end (the mode).
    pub fn mode(mut self, mode: impl IntoElement) -> Self {
        self.mode = Some(mode.into_any_element());
        self
    }

    /// The most room the picker has: the tabs narrow to their marks where
    /// the full ones would crowd the models.
    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: ModelPickerTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A tab chosen, by id.
    pub fn on_provider_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_provider_change = Some(Rc::new(f));
        self
    }

    /// A row chosen, by id (`onValueChange`).
    pub fn on_choose(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_choose = Some(Rc::new(f));
        self
    }

    /// The pointer entered row `n`.
    pub fn on_highlight(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_highlight = Some(Rc::new(f));
        self
    }

    /// A row's star pressed, by the row's id.
    pub fn on_favourite(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_favourite = Some(Rc::new(f));
        self
    }

    /// A row's "also run" toggle pressed, by the row's id.
    pub fn on_also(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_also = Some(Rc::new(f));
        self
    }

    /// A choice made of an option (a level or a menu's): its id, the value.
    pub fn on_option_change(
        mut self,
        f: impl Fn(SharedString, SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_option_change = Some(Rc::new(f));
        self
    }

    /// An option's menu asks to open (its id) or close (`None`).
    pub fn on_option_open(
        mut self,
        f: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_option_open = Some(Rc::new(f));
        self
    }

    /// Speed's switch turned.
    pub fn on_speed_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_speed_change = Some(Rc::new(f));
        self
    }

    /// A notice's button pressed, by its provider's id.
    pub fn on_notice_action(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_notice_action = Some(Rc::new(f));
        self
    }

    /// Agent settings.
    pub fn on_settings(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_settings = Some(Rc::new(f));
        self
    }

    /// Whether the options column has anything in it.
    fn has_options(&self) -> bool {
        self.levels.is_some() || self.speed.is_some() || !self.options.is_empty()
    }
}

/// Paint `.t3-provider-spin`: a 10px disc in `background` with a 2px
/// `layer-accent-01` border whose top quarter is `interactive`, turning.
fn paint_spin(bounds: Bounds<Pixels>, turn: f32, window: &mut Window) {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
    let center = bounds.center();
    let edge = f32::from(bounds.size.width);
    let border = edge / 5.;
    window.paint_quad(
        gpui_kit::fill(bounds, color(ColorRole::Background)).corner_radii(px(edge / 2.)),
    );
    let r = (edge - border) / 2.;
    let accent = color(ColorRole::LayerAccent01);
    base::stroke_arc(window, center, r, border, 0., TAU, accent);
    let start = -FRAC_PI_2 - FRAC_PI_4 + turn * TAU;
    let ink = color(ColorRole::Interactive);
    base::stroke_arc(window, center, r, border, start, FRAC_PI_2, ink);
}

/// The starting ring in a provider tab. d-fix: `.t3-mpick-prov .t3-provider
/// { position: static }` and `.t3-mpick-prov .t3-provider-spin { right:
/// spacing-04; bottom: auto; top: calc(50% - 7px) }` move it off the glyph
/// to the 40px row's right edge; on a tab narrowed to its mark, its corner.
fn provider_spin(narrow: bool, window: &mut Window, cx: &App) -> impl IntoElement {
    let frame = base::spin_frame_now(PROVIDER_SPIN_PERIOD, motion::reduced_motion(cx), window);
    div()
        .absolute()
        .when(narrow, |this| this.right(rem(4.)).top(rem(4.)))
        .when(!narrow, |this| this.right(rem(12.)).top(rem(40. / 2. - 7.)))
        .size(rem(10.))
        .opacity(frame.opacity)
        .child(
            canvas(
                |_, _, _| {},
                move |b, _, window, _| paint_spin(b, frame.turn, window),
            )
            .size_full(),
        )
}

/// A selectable row's fill: chosen `layer-selected-01`, hovered `layer-hover-01`.
fn row_fill(on: bool, hovered: bool) -> Option<gpui_kit::Hsla> {
    if on {
        Some(color(ColorRole::LayerSelected01))
    } else if hovered {
        Some(color(ColorRole::LayerHover01))
    } else {
        None
    }
}

/// A plain hover tooltip (Kit's), standing in for the HTML `title`.
fn title(text: SharedString) -> impl Fn(&mut Window, &mut App) -> gpui_kit::AnyView {
    move |window, cx| gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
}

/// A provider tab's glyph: its icon, its mark, else `ProviderIcon` by name.
fn tab_glyph(p: &PickerProvider, key: ElementId, ink: gpui_kit::Hsla) -> AnyElement {
    match (&p.icon, &p.mark) {
        (Some(icon), _) => press::icon(*icon, 16., ink).into_any_element(),
        (None, Some(mark)) => mark(20.),
        (None, None) => ProviderIcon::new(p.name.clone())
            .id(key)
            .size(IdentitySize::Sm)
            .into_any_element(),
    }
}

/// A small ghost icon button in a row: its own press, never the row's.
/// `glyph` is its icon and ink; `press` the handler and the row's id it is given.
fn row_button(
    id: ElementId,
    glyph: (CarbonIcon, gpui_kit::Hsla),
    label: SharedString,
    shown: bool,
    press: (Option<TextHandler>, SharedString),
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let ((icon, ink), (f, value)) = (glyph, press);
    let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
    gpui_kit::base::Button::new(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(rem(24.))
        .cursor_pointer()
        .accessibility_label(label.clone())
        .when(hovered, |this| this.bg(color(ColorRole::LayerHover01)))
        // Hidden until its row is lit, yet laid out, so the row never
        // reflows under the pointer.
        .when(!shown, |this| this.invisible())
        .on_hover(press::track_hover(hover))
        .tooltip(title(label))
        .on_click(move |_, w, cx| {
            cx.stop_propagation();
            if let Some(f) = f.as_ref() {
                f(value.clone(), w, cx)
            }
        })
        .child(press::icon(icon, 16., ink))
        .into_any_element()
}

impl RenderOnce for ModelPicker {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let texts = self.texts.clone();
        let options_shown = self.has_options();
        // The room given, in the reference's px (at a 16px rem).
        let scale = f32::from(window.rem_size()) / crate::tokens::REM_PX;
        let width = self.max_width.map(|w| f32::from(w) / scale);
        let narrow = width.is_some_and(|w| narrow_tabs(w, options_shown));
        let query = self.query.read(cx).value().to_string();
        let provider = self
            .provider
            .as_ref()
            .and_then(|p| self.providers.iter().find(|x| &x.id == p))
            .cloned();

        // Providers.
        let tabs: Vec<AnyElement> = self
            .providers
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let key = |s: &str| -> ElementId {
                    (id.clone(), SharedString::from(format!("{s}-{i}"))).into()
                };
                let (hovered, hover) = press::hover(key("prov-hover"), window, cx);
                let (focus, ring) = press::focus(key("prov-focus"), window, cx);
                let on = self.provider.as_ref() == Some(&p.id);
                let note = p.note(&texts);
                let ink = color(if p.disabled {
                    ColorRole::TextDisabled
                } else if on || hovered {
                    ColorRole::TextPrimary
                } else {
                    ColorRole::TextSecondary
                });
                let tip = p.tooltip.clone().or_else(|| {
                    narrow.then(|| match &note {
                        Some(n) => format!("{}: {n}", p.name).into(),
                        None => p.name.clone(),
                    })
                });
                let pick = self.on_provider_change.clone();
                let pid = p.id.clone();
                // `.t3-mpick-prov { display: flex; align-items: center; gap: spacing-03; height: size-md;
                // padding: 0 spacing-04; color: text-secondary; font-size: 0.875rem }`,
                // `:hover { background: layer-hover-01; color: text-primary }`,
                // `--on { background: layer-selected-01; color: text-primary; box-shadow: inset 3px 0 0 border-interactive }`:
                // the rail is the tab's own inset shadow, inside its start edge.
                gpui_kit::base::Button::new(key("prov"))
                    .track_focus(&focus)
                    .accessibility_label(p.name.clone())
                    .relative()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .items_center()
                    .w_full()
                    .gap(rem(8.))
                    .when(narrow, |this| this.justify_center().h(rem(40.)))
                    .when(!narrow, |this| {
                        this.justify_start()
                            .when(note.is_none(), |this| this.h(rem(40.)))
                            .when(note.is_some(), |this| this.min_h(rem(48.)).py(rem(6.)))
                            .px(rem(12.))
                    })
                    .text_size(rem(14.))
                    .line_height(rem(18.))
                    .text_color(ink)
                    .when(!p.disabled, |this| this.cursor_pointer())
                    .when(p.disabled, |this| this.opacity(0.5))
                    .when_some(row_fill(on, hovered && !p.disabled), |this, bg| this.bg(bg))
                    .when(on, |this| {
                        this.shadow(crate::t3::timeline::card::rail(color(
                            ColorRole::BorderInteractive,
                        )))
                    })
                    .on_hover(press::track_hover(hover))
                    .when_some(tip, |this, tip| this.tooltip(title(tip)))
                    .when(!p.disabled, |this| {
                        this.on_click(move |_, w, cx| {
                            if let Some(f) = pick.as_ref() {
                                f(pid.clone(), w, cx)
                            }
                        })
                    })
                    .child(tab_glyph(p, key("prov-icon"), ink))
                    .when(!narrow, |this| {
                        // NewThread's `.t3-mpick-prov:has(.t3-mpick-provnote)`:
                        // the note under the name, both beside the glyph.
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_0()
                                .child(div().whitespace_nowrap().child(p.name.clone()))
                                // `.t3-mpick-provnote { font-size: 0.75rem; color: text-helper }`.
                                .when_some(note, |this, n| {
                                    this.child(
                                        div()
                                            .whitespace_nowrap()
                                            .text_size(rem(12.))
                                            .line_height(rem(16.))
                                            .text_color(color(ColorRole::TextHelper))
                                            .child(n),
                                    )
                                }),
                        )
                    })
                    .when(p.starting, |this| {
                        this.child(provider_spin(narrow, window, cx))
                    })
                    .when(ring, |this| this.child(press::ring()))
                    .into_any_element()
            })
            .collect();
        let settings = self.on_settings.clone().map(|on_settings| {
            let (hovered, hover) = press::hover((id.clone(), "settings-hover"), window, cx);
            let (focus, ring) = press::focus((id.clone(), "settings-focus"), window, cx);
            let ink = color(if hovered {
                ColorRole::TextPrimary
            } else {
                ColorRole::TextSecondary
            });
            // `.t3-mpick-settings { margin-top: auto; border-top: 1px solid border-subtle-01 }`.
            gpui_kit::base::Button::new((id.clone(), "settings"))
                .track_focus(&focus)
                .accessibility_label(texts.settings.clone())
                .relative()
                .flex()
                .flex_row()
                .flex_none()
                .items_center()
                .w_full()
                .mt_auto()
                .gap(rem(8.))
                .h(rem(40.))
                .when(narrow, |this| {
                    this.justify_center().tooltip(title(texts.settings.clone()))
                })
                .when(!narrow, |this| this.justify_start().px(rem(12.)))
                .border_t_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .text_size(rem(14.))
                .text_color(ink)
                .cursor_pointer()
                .when(hovered, |this| this.bg(color(ColorRole::LayerHover01)))
                .on_hover(press::track_hover(hover))
                .on_click(move |_, w, cx| on_settings(w, cx))
                .child(press::icon(CarbonIcon::Settings, 16., ink))
                .when(!narrow, |this| this.child(texts.settings.clone()))
                .when(ring, |this| this.child(press::ring()))
        });
        // `.t3-mpick-side { display: flex; flex-direction: column; padding: spacing-03 0;
        // border-right: 1px solid border-subtle-01; overflow-y: auto }`.
        let side = div()
            .id((id.clone(), "side"))
            .flex()
            .flex_col()
            .flex_none()
            .w(rem(if narrow {
                MODEL_PICKER_NARROW_SIDE
            } else {
                MODEL_PICKER_GEOMETRY.0
            }))
            .py(rem(8.))
            .when(narrow, |this| this.px(rem(4.)))
            .border_r_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .overflow_y_scroll()
            .children(tabs)
            .children(settings);

        // Models.
        let provider_name = provider
            .as_ref()
            .map(|p| p.name.to_string())
            .unwrap_or_default();
        let empty = self.models.is_empty();
        let rows: Vec<AnyElement> = std::mem::take(&mut self.models)
            .into_iter()
            .enumerate()
            .map(|(n, m)| {
                let key = |s: &str| -> ElementId {
                    (id.clone(), SharedString::from(format!("{s}-{}", m.id))).into()
                };
                let (hovered, hover) = press::hover(key("row-hover"), window, cx);
                let (focus, ring) = press::focus(key("row-focus"), window, cx);
                let lit = hovered || self.highlighted == Some(n);
                let choose = {
                    let f = self.on_choose.clone();
                    let mid = m.id.clone();
                    Rc::new(move |w: &mut Window, cx: &mut App| {
                        if let Some(f) = f.as_ref() {
                            f(mid.clone(), w, cx)
                        }
                    })
                };
                let key_choose = choose.clone();
                let highlight = self.on_highlight.clone();
                let star = m.favourite.map(|on| {
                    row_button(
                        key("star"),
                        (
                            if on {
                                CarbonIcon::StarFilled
                            } else {
                                CarbonIcon::Star
                            },
                            color(if on {
                                ColorRole::SupportWarning
                            } else {
                                ColorRole::IconSecondary
                            }),
                        ),
                        if on {
                            texts.remove_favourite.clone()
                        } else {
                            texts.add_favourite.clone()
                        },
                        on || lit,
                        (self.on_favourite.clone(), m.id.clone()),
                        window,
                        cx,
                    )
                });
                let also = m.also.map(|on| {
                    row_button(
                        key("also"),
                        (
                            if on {
                                CarbonIcon::Checkmark
                            } else {
                                CarbonIcon::Add
                            },
                            color(if on {
                                ColorRole::SupportSuccess
                            } else {
                                ColorRole::IconSecondary
                            }),
                        ),
                        texts.also.clone(),
                        on || lit,
                        (self.on_also.clone(), m.id.clone()),
                        window,
                        cx,
                    )
                });
                let ink = color(if m.disabled {
                    ColorRole::TextDisabled
                } else {
                    ColorRole::TextPrimary
                });
                // `.t3-mrow { display: grid; grid-template-columns: 1fr auto; gap: 2px spacing-03;
                // padding: spacing-03 spacing-04 }`.
                div()
                    .id(key("row"))
                    .track_focus(&focus.tab_index(0).tab_stop(!m.disabled))
                    .relative()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .items_center()
                    .gap(rem(8.))
                    .py(rem(8.))
                    .px(rem(12.))
                    .when(!m.disabled, |this| this.cursor_pointer())
                    .when_some(row_fill(m.chosen, lit && !m.disabled), |this, bg| {
                        this.bg(bg)
                    })
                    .on_hover(press::track_hover(hover))
                    .on_mouse_move(move |_, w, cx| {
                        if let Some(f) = highlight.as_ref() {
                            f(n, w, cx)
                        }
                    })
                    .when(!m.disabled, |this| {
                        // GPUI already turns Enter/Space on a focused clickable element into a
                        // click (on key up); the key handler below acts on the key itself, so a
                        // keyboard click is ignored here or the action would run twice.
                        this.on_click(move |e, w, cx| {
                            if !e.is_keyboard() {
                                choose(w, cx)
                            }
                        })
                        .on_key_down(move |e: &KeyDownEvent, w, cx| {
                            if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                                cx.stop_propagation();
                                key_choose(w, cx);
                            }
                        })
                    })
                    .when_some(m.mark.clone(), |this, mark| this.child(mark(16.)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            // `.t3-mrow-name { display: flex; gap: spacing-03; align-items: center; font-size: 0.875rem }`.
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(rem(8.))
                                    .min_w_0()
                                    .text_size(rem(14.))
                                    .line_height(rem(18.))
                                    .text_color(ink)
                                    .child(
                                        div()
                                            .min_w_0()
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .whitespace_nowrap()
                                            .child(m.name.clone()),
                                    )
                                    .when_some(m.badge.clone(), |this, b| {
                                        this.child(
                                            Badge::new(b)
                                                .size(BadgeSize::Sm)
                                                .variant(BadgeVariant::Blue),
                                        )
                                    }),
                            )
                            // `.t3-mrow-desc { font-size: 0.75rem; color: text-secondary }`.
                            .when_some(m.description.clone(), |this, d| {
                                this.child(
                                    div()
                                        .text_size(rem(12.))
                                        .line_height(rem(16.))
                                        .text_color(color(ColorRole::TextSecondary))
                                        .child(d),
                                )
                            }),
                    )
                    .children(also)
                    .children(star)
                    // `.t3-mrow > svg { align-self: center; fill: interactive }`.
                    .when(m.chosen, |this| {
                        this.child(press::icon(
                            CarbonIcon::Checkmark,
                            16.,
                            color(ColorRole::Interactive),
                        ))
                    })
                    .when_some(m.key.clone(), |this, k| this.child(Kbd::new(k)))
                    .when(ring, |this| this.child(press::ring()))
                    .into_any_element()
            })
            .collect();
        let notice_action = self.on_notice_action.clone();
        let main = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            // `.t3-mpick-search { display: flex; align-items: center; gap: spacing-03; height: size-md;
            // padding: 0 spacing-04; border-bottom: 1px solid border-subtle-01 }`, `svg { fill: icon-secondary }`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .items_center()
                    .gap(rem(8.))
                    .h(rem(40.))
                    .px(rem(12.))
                    .border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(press::icon(
                        CarbonIcon::Search,
                        16.,
                        color(ColorRole::IconSecondary),
                    ))
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .min_w_0()
                            .text_size(rem(14.))
                            .line_height(rem(18.))
                            .text_color(color(ColorRole::TextPrimary))
                            .when(query.is_empty(), |this| {
                                // A placeholder stays on one line and clips at the
                                // field's edge, as an `<input>`'s does.
                                this.child(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .right_0()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_color(color(ColorRole::TextPlaceholder))
                                        .child((texts.search)(&provider_name)),
                                )
                            })
                            .child(
                                gpui_kit::component::input::Input::new(&self.query)
                                    .appearance(false)
                                    .with_size(gpui_kit::component::Size::XSmall)
                                    .p_0()
                                    .w_full()
                                    .text_size(rem(14.))
                                    .line_height(rem(18.)),
                            ),
                    ),
            )
            // `.t3-mpick-notice { display: flex; gap: spacing-03; padding: spacing-03 spacing-04;
            // background: notification-background-warning; font-size: 0.75rem }`.
            .when_some(
                provider
                    .as_ref()
                    .filter(|_| query.is_empty())
                    .and_then(|p| p.notice.clone().map(|n| (p, n))),
                |this, (p, n)| {
                    let action = p.notice_action.clone().map(|label| {
                        let (f, pid) = (notice_action.clone(), p.id.clone());
                        crate::components::Button::new((id.clone(), "notice-action"), label)
                            .variant(crate::components::ButtonVariant::Ghost)
                            .size(crate::components::ButtonSize::Sm)
                            .on_click(move |_, w, cx| {
                                if let Some(f) = f.as_ref() {
                                    f(pid.clone(), w, cx)
                                }
                            })
                    });
                    this.child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_none()
                            .items_center()
                            .gap(rem(8.))
                            .py(rem(8.))
                            .px(rem(12.))
                            .bg(color(ColorRole::NotificationBackgroundWarning))
                            .text_size(rem(12.))
                            .line_height(rem(16.))
                            .text_color(color(ColorRole::TextPrimary))
                            .child(div().flex_none().child(press::icon(
                                CarbonIcon::WarningAltFilled,
                                16.,
                                color(ColorRole::SupportWarning),
                            )))
                            // The message wraps within the column.
                            .child(div().flex_1().min_w_0().child(n))
                            .children(action),
                    )
                },
            )
            // `.t3-mpick-list { flex: 1; overflow-y: auto }`.
            .child(
                div()
                    .id((id.clone(), "list"))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(rows)
                    .when(empty, |this| {
                        this.child(
                            div()
                                .py(rem(12.))
                                .px(rem(12.))
                                .text_size(rem(12.))
                                .line_height(rem(16.))
                                .text_color(color(ColorRole::TextHelper))
                                .child(texts.empty.clone()),
                        )
                    }),
            );

        // Options.
        let options = options_shown.then(|| self.options_column(&texts, window, cx));

        let foot = (self.fanout.is_some() || self.mode.is_some()).then(|| {
            let fanout = self.fanout.take().map(|f| {
                let (hovered, hover) = press::hover((id.clone(), "fanout-hover"), window, cx);
                let (focus, ring) = press::focus((id.clone(), "fanout-focus"), window, cx);
                let ink = color(if f.on || hovered {
                    ColorRole::TextPrimary
                } else {
                    ColorRole::TextSecondary
                });
                let handler = f.handler.clone();
                // ComposerMore's `.cmb-mi`: an icon and its words, a menu item's row.
                gpui_kit::base::Button::new((id.clone(), "fanout"))
                    .track_focus(&focus)
                    .accessibility_label(f.label.clone())
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(rem(8.))
                    .h(rem(32.))
                    .px(rem(12.))
                    .text_size(rem(14.))
                    .text_color(ink)
                    .cursor_pointer()
                    .when_some(row_fill(f.on, hovered), |this, bg| this.bg(bg))
                    .on_hover(press::track_hover(hover))
                    .when_some(f.tip.clone(), |this, tip| this.tooltip(title(tip)))
                    .on_click(move |_, w, cx| handler(w, cx))
                    .child(press::icon(
                        if f.on {
                            CarbonIcon::Checkmark
                        } else {
                            CarbonIcon::Chat
                        },
                        16.,
                        ink,
                    ))
                    .child(div().whitespace_nowrap().child(f.label))
                    .when(ring, |this| this.child(press::ring()))
            });
            div()
                .flex()
                .flex_row()
                .flex_none()
                .items_center()
                .gap(rem(8.))
                .h(rem(40.))
                .px(rem(4.))
                .border_t_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .children(fanout)
                .child(div().flex_1())
                .children(self.mode.take())
        });

        // `.t3-mpick { display: grid; grid-template-columns: 11rem 1fr 10rem; width: 100%;
        // max-width: 46rem; height: 22rem; background: layer-01; box-shadow: 0 2px 6px shadow }`.
        // Given its room, as wide as that within 46rem; else as wide as
        // its parent within it.
        let full = rem(MODEL_PICKER_GEOMETRY.2);
        div()
            .flex()
            .flex_col()
            .when_some(self.max_width, |this, w| {
                this.w(w.min(px(MODEL_PICKER_GEOMETRY.2 * scale)))
            })
            .when(self.max_width.is_none(), |this| this.w_full().max_w(full))
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .text_color(color(ColorRole::TextPrimary))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(rem(MODEL_PICKER_GEOMETRY.3))
                    .child(side)
                    .child(main)
                    .children(options),
            )
            .children(foot)
    }
}
