//! [`ModelPicker`] (`.t3-mpick`, "Model picker" in `t3.css` with the
//! gemini-B and d-fix rules): the floating model chooser.
//!
//! Three columns on a `layer-01` surface with the floating shadow, 22rem
//! tall and at most 46rem wide:
//! - **Providers** (11rem): vertical tabs (provider glyph, name, a "Needs
//!   setup" note, and a starting provider's ring at the row's right edge,
//!   per the d-fix rule), the selected one on `layer-selected-01` with a 3px
//!   `border-interactive` start rail, and Agent settings at the bottom.
//! - **Models**: a search field, the provider's warning notice, and rows
//!   (name, badge, description, a check when chosen); hover
//!   `layer-hover-01`, chosen `layer-selected-01`; Enter / Space choose.
//! - **Effort** (10rem): the five effort rows with glyphs and a check, then
//!   Speed as its own group with the small switch labelled "Fast mode"
//!   (Carbon's `hideLabel` puts `labelText` beside the track).
use super::controls::{EffortGlyph, EffortLevel};
use crate::components::InputState;
use crate::components::badge::{Badge, BadgeSize, BadgeVariant};
use crate::components::overlay::floating_shadow;
use crate::components::switch::{Switch, SwitchSize};
use crate::t3::press::{self, Handler, rem};
use crate::t3::shell::{IdentitySize, PROVIDER_SPIN_PERIOD, ProviderIcon};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use crate::{motion, t3::base};
use gpui_kit::component::Sizable as _;
use gpui_kit::{
    AnyElement, App, AppContext as _, Bounds, ElementId, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _,
    Styled, Window, canvas, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// A model (`{ id, name, description, badge }`).
#[derive(Clone, Debug, PartialEq)]
pub struct PickerModel {
    /// `id`.
    pub id: SharedString,
    /// `name`.
    pub name: SharedString,
    /// `description`.
    pub description: SharedString,
    /// `badge` (a small blue tag).
    pub badge: Option<SharedString>,
}

impl PickerModel {
    /// A model.
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            badge: None,
        }
    }

    /// `badge`.
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

/// A provider (`{ id, name, starting, notice, noticeShort, models }`).
#[derive(Clone, Debug, PartialEq)]
pub struct PickerProvider {
    /// `id`.
    pub id: SharedString,
    /// `name`.
    pub name: SharedString,
    /// `starting`: the glyph's starting ring.
    pub starting: bool,
    /// `notice`: a warning above the models.
    pub notice: Option<SharedString>,
    /// `noticeShort`: the tab's note (default "Needs setup").
    pub notice_short: Option<SharedString>,
    /// `models`.
    pub models: Vec<PickerModel>,
}

impl PickerProvider {
    /// A provider with `models`.
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        models: Vec<PickerModel>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            starting: false,
            notice: None,
            notice_short: None,
            models,
        }
    }

    /// `starting`.
    pub fn starting(mut self, starting: bool) -> Self {
        self.starting = starting;
        self
    }

    /// `notice`.
    pub fn notice(mut self, notice: impl Into<SharedString>) -> Self {
        self.notice = Some(notice.into());
        self
    }

    /// `noticeShort`.
    pub fn notice_short(mut self, note: impl Into<SharedString>) -> Self {
        self.notice_short = Some(note.into());
        self
    }
}

/// `ModelPicker`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct ModelPickerTexts {
    /// The tab note without `noticeShort`.
    pub needs_setup: SharedString,
    /// Agent settings.
    pub settings: SharedString,
    /// `"Search {provider} models"`.
    pub search: Rc<dyn Fn(&str) -> SharedString>,
    /// Effort.
    pub effort: SharedString,
    /// Speed.
    pub speed: SharedString,
    /// The speed switch's label, shown beside it ("Fast mode").
    pub fast_mode: SharedString,
    /// The effort level names.
    pub level: Rc<dyn Fn(EffortLevel) -> SharedString>,
    /// Standard (`labelA`; not drawn, since `hideLabel` shows `fast_mode`).
    pub standard: SharedString,
    /// Fast (`labelB`; not drawn, since `hideLabel` shows `fast_mode`).
    pub fast: SharedString,
}

impl Default for ModelPickerTexts {
    fn default() -> Self {
        Self {
            needs_setup: "Needs setup".into(),
            settings: "Agent settings".into(),
            search: Rc::new(|p| format!("Search {p} models").into()),
            effort: "Effort".into(),
            speed: "Speed".into(),
            fast_mode: "Fast mode".into(),
            level: Rc::new(|l| {
                let n = l.name();
                format!("{}{}", n[..1].to_uppercase(), &n[1..]).into()
            }),
            standard: "Standard".into(),
            fast: "Fast".into(),
        }
    }
}

/// Models of `provider` whose names contain `query` (case-insensitive).
pub fn filter_models<'a>(models: &'a [PickerModel], query: &str) -> Vec<&'a PickerModel> {
    let q = query.to_lowercase();
    models
        .iter()
        .filter(|m| m.name.to_lowercase().contains(&q))
        .collect()
}

/// `.t3-mpick { grid-template-columns: 11rem 1fr 10rem; max-width: 46rem; height: 22rem }` in px.
pub const MODEL_PICKER_GEOMETRY: (f32, f32, f32, f32) = (176., 160., 736., 352.);

type ValueHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type EffortHandler = Rc<dyn Fn(EffortLevel, &mut Window, &mut App)>;
type FastHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// The model chooser. See the module docs.
#[derive(IntoElement)]
pub struct ModelPicker {
    id: ElementId,
    providers: Vec<PickerProvider>,
    value: Option<SharedString>,
    effort: EffortLevel,
    fast: bool,
    texts: ModelPickerTexts,
    on_value_change: Option<ValueHandler>,
    on_effort_change: Option<EffortHandler>,
    on_fast_change: Option<FastHandler>,
    on_settings: Option<Handler>,
}

impl ModelPicker {
    /// A picker over `providers`.
    pub fn new(id: impl Into<ElementId>, providers: Vec<PickerProvider>) -> Self {
        Self {
            id: id.into(),
            providers,
            value: None,
            effort: EffortLevel::Medium,
            fast: false,
            texts: ModelPickerTexts::default(),
            on_value_change: None,
            on_effort_change: None,
            on_fast_change: None,
            on_settings: None,
        }
    }

    /// `value`: the chosen model id (controlled when set).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `effort`.
    pub fn effort(mut self, effort: EffortLevel) -> Self {
        self.effort = effort;
        self
    }

    /// `fast`.
    pub fn fast(mut self, fast: bool) -> Self {
        self.fast = fast;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: ModelPickerTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onValueChange(id)`.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// `onEffortChange(level)`.
    pub fn on_effort_change(
        mut self,
        f: impl Fn(EffortLevel, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_effort_change = Some(Rc::new(f));
        self
    }

    /// `onFastChange(on)`.
    pub fn on_fast_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_fast_change = Some(Rc::new(f));
        self
    }

    /// Agent settings.
    pub fn on_settings(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_settings = Some(Rc::new(f));
        self
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
/// to the 40px row's right edge.
fn provider_spin(window: &mut Window, cx: &App) -> impl IntoElement {
    let frame = base::spin_frame_now(PROVIDER_SPIN_PERIOD, motion::reduced_motion(cx), window);
    div()
        .absolute()
        .right(rem(12.))
        .top(rem(40. / 2. - 7.))
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

impl RenderOnce for ModelPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let texts = self.texts.clone();
        let initial_value = self.value.clone();
        let value_state =
            window.use_keyed_state((id.clone(), "value"), cx, move |_, _| initial_value);
        let value = self.value.clone().or_else(|| value_state.read(cx).clone());
        // The provider tab starts on the one that holds the value (`useState(...)`).
        let initial_provider = self
            .providers
            .iter()
            .position(|p| p.models.iter().any(|m| Some(&m.id) == value.as_ref()))
            .unwrap_or(0);
        let provider_state =
            window.use_keyed_state((id.clone(), "provider"), cx, move |_, _| initial_provider);
        let provider_index = (*provider_state.read(cx)).min(self.providers.len().saturating_sub(1));
        let query_state = window
            .use_keyed_state((id.clone(), "query"), cx, |window, cx| {
                cx.new(|cx| InputState::new(window, cx))
            })
            .read(cx)
            .clone();
        let query = query_state.read(cx).value().to_string();
        let provider = self.providers.get(provider_index).cloned();

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
                let on = i == provider_index;
                let state = provider_state.clone();
                let note = p
                    .notice
                    .as_ref()
                    .map(|_| p.notice_short.clone().unwrap_or(texts.needs_setup.clone()));
                // `.t3-mpick-prov { display: flex; align-items: center; gap: spacing-03; height: size-md;
                // padding: 0 spacing-04; color: text-secondary; font-size: 0.875rem }`,
                // `:hover { background: layer-hover-01; color: text-primary }`,
                // `--on { background: layer-selected-01; color: text-primary; box-shadow: inset 3px 0 0 border-interactive }`.
                gpui_kit::base::Button::new(key("prov"))
                    .track_focus(&focus)
                    .accessibility_label(p.name.clone())
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_start()
                    .w_full()
                    .gap(rem(8.))
                    .h(rem(40.))
                    .px(rem(12.))
                    .text_size(rem(14.))
                    .line_height(rem(18.))
                    .text_color(color(if on || hovered {
                        ColorRole::TextPrimary
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .cursor_pointer()
                    .when_some(
                        if on {
                            Some(color(ColorRole::LayerSelected01))
                        } else if hovered {
                            Some(color(ColorRole::LayerHover01))
                        } else {
                            None
                        },
                        |this, bg| this.bg(bg),
                    )
                    .when(on, |this| {
                        this.shadow(crate::t3::timeline::card::rail(color(
                            ColorRole::BorderInteractive,
                        )))
                    })
                    .on_hover(press::track_hover(hover))
                    .on_click(move |_, _, cx| {
                        state.update(cx, |s, cx| {
                            *s = i;
                            cx.notify();
                        })
                    })
                    .child(
                        ProviderIcon::new(p.name.clone())
                            .id(key("prov-icon"))
                            .size(IdentitySize::Sm),
                    )
                    .child(div().flex_none().whitespace_nowrap().child(p.name.clone()))
                    .when(p.starting, |this| this.child(provider_spin(window, cx)))
                    // `.t3-mpick-provnote { margin-left: auto; font-size: 0.75rem; color: text-helper }`.
                    .when_some(note, |this, n| {
                        this.child(
                            div()
                                .ml_auto()
                                .whitespace_nowrap()
                                .text_size(rem(12.))
                                .text_color(color(ColorRole::TextHelper))
                                .child(n),
                        )
                    })
                    .when(ring, |this| this.child(press::ring()))
                    .into_any_element()
            })
            .collect();
        let (settings_hovered, settings_hover) =
            press::hover((id.clone(), "settings-hover"), window, cx);
        let (settings_focus, settings_ring) =
            press::focus((id.clone(), "settings-focus"), window, cx);
        let settings_ink = color(if settings_hovered {
            ColorRole::TextPrimary
        } else {
            ColorRole::TextSecondary
        });
        let on_settings = self.on_settings.clone();
        // `.t3-mpick-settings { margin-top: auto; border-top: 1px solid border-subtle-01 }`.
        let settings = gpui_kit::base::Button::new((id.clone(), "settings"))
            .track_focus(&settings_focus)
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .w_full()
            .mt_auto()
            .gap(rem(8.))
            .h(rem(40.))
            .px(rem(12.))
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_size(rem(14.))
            .text_color(settings_ink)
            .cursor_pointer()
            .when(settings_hovered, |this| {
                this.bg(color(ColorRole::LayerHover01))
            })
            .on_hover(press::track_hover(settings_hover))
            .when_some(on_settings, |this, f| {
                this.on_click(move |_, w, cx| f(w, cx))
            })
            .child(press::icon(CarbonIcon::Settings, 16., settings_ink))
            .child(texts.settings.clone())
            .when(settings_ring, |this| this.child(press::ring()));
        // `.t3-mpick-side { display: flex; flex-direction: column; padding: spacing-03 0;
        // border-right: 1px solid border-subtle-01; overflow-y: auto }`.
        let side = div()
            .id((id.clone(), "side"))
            .flex()
            .flex_col()
            .flex_none()
            .w(rem(MODEL_PICKER_GEOMETRY.0))
            .py(rem(8.))
            .border_r_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .overflow_y_scroll()
            .children(tabs)
            .child(settings);

        // Models.
        let provider_name = provider
            .as_ref()
            .map(|p| p.name.to_string())
            .unwrap_or_default();
        let models = provider
            .as_ref()
            .map(|p| p.models.clone())
            .unwrap_or_default();
        let rows: Vec<AnyElement> = filter_models(&models, &query)
            .into_iter()
            .map(|m| {
                let key = |s: &str| -> ElementId {
                    (id.clone(), SharedString::from(format!("{s}-{}", m.id))).into()
                };
                let (hovered, hover) = press::hover(key("row-hover"), window, cx);
                let (focus, ring) = press::focus(key("row-focus"), window, cx);
                let on = value.as_ref() == Some(&m.id);
                let choose = {
                    let value_state = value_state.clone();
                    let f = self.on_value_change.clone();
                    let mid = m.id.clone();
                    Rc::new(move |w: &mut Window, cx: &mut App| {
                        value_state.update(cx, |v, cx| {
                            *v = Some(mid.clone());
                            cx.notify();
                        });
                        if let Some(f) = f.as_ref() {
                            f(mid.clone(), w, cx)
                        }
                    })
                };
                let key_choose = choose.clone();
                // `.t3-mrow { display: grid; grid-template-columns: 1fr auto; gap: 2px spacing-03;
                // padding: spacing-03 spacing-04 }`.
                div()
                    .id(key("row"))
                    .track_focus(&focus.tab_index(0).tab_stop(true))
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(rem(8.))
                    .py(rem(8.))
                    .px(rem(12.))
                    .cursor_pointer()
                    .when_some(row_fill(on, hovered), |this, bg| this.bg(bg))
                    .on_hover(press::track_hover(hover))
                    .on_click(move |_, w, cx| choose(w, cx))
                    .on_key_down(move |e: &KeyDownEvent, w, cx| {
                        if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                            cx.stop_propagation();
                            key_choose(w, cx);
                        }
                    })
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
                                    .text_size(rem(14.))
                                    .line_height(rem(18.))
                                    .text_color(color(ColorRole::TextPrimary))
                                    .child(m.name.clone())
                                    .when_some(m.badge.clone(), |this, b| {
                                        this.child(
                                            Badge::new(b)
                                                .size(BadgeSize::Sm)
                                                .variant(BadgeVariant::Blue),
                                        )
                                    }),
                            )
                            // `.t3-mrow-desc { font-size: 0.75rem; color: text-secondary }`.
                            .child(
                                div()
                                    .text_size(rem(12.))
                                    .line_height(rem(16.))
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(m.description.clone()),
                            ),
                    )
                    // `.t3-mrow > svg { align-self: center; fill: interactive }`.
                    .when(on, |this| {
                        this.child(press::icon(
                            CarbonIcon::Checkmark,
                            16.,
                            color(ColorRole::Interactive),
                        ))
                    })
                    .when(ring, |this| this.child(press::ring()))
                    .into_any_element()
            })
            .collect();
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
                                gpui_kit::component::input::Input::new(&query_state)
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
                provider.as_ref().and_then(|p| p.notice.clone()),
                |this, n| {
                    this.child(
                        div()
                            .flex()
                            .flex_row()
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
                            .child(div().flex_1().min_w_0().child(n)),
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
                    .overflow_y_scroll()
                    .children(rows),
            );

        // Effort.
        let header = |text: SharedString| {
            // `.t3-mpick-h { padding: spacing-03 spacing-04 spacing-02; font-size: 0.75rem; color: text-secondary }`.
            div()
                .pt(rem(8.))
                .px(rem(12.))
                .pb(rem(4.))
                .text_size(rem(12.))
                .line_height(rem(16.))
                .text_color(color(ColorRole::TextSecondary))
                .child(text)
        };
        let effort_rows: Vec<AnyElement> = EffortLevel::ALL
            .into_iter()
            .map(|l| {
                let key = |s: &str| -> ElementId {
                    (id.clone(), SharedString::from(format!("{s}-{}", l.name()))).into()
                };
                let (hovered, hover) = press::hover(key("erow-hover"), window, cx);
                let (focus, ring) = press::focus(key("erow-focus"), window, cx);
                let on = self.effort == l;
                let f = self.on_effort_change.clone();
                // `.t3-erow { display: flex; align-items: center; gap: spacing-03; height: size-sm;
                // padding: 0 spacing-04; color: text-primary; font-size: 0.875rem }`.
                gpui_kit::base::Button::new(key("erow"))
                    .track_focus(&focus)
                    .selected(on)
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_start()
                    .w_full()
                    .gap(rem(8.))
                    .h(rem(32.))
                    .px(rem(12.))
                    .text_size(rem(14.))
                    .line_height(rem(18.))
                    .text_color(color(ColorRole::TextPrimary))
                    .cursor_pointer()
                    .when_some(row_fill(on, hovered), |this, bg| this.bg(bg))
                    .on_hover(press::track_hover(hover))
                    .when_some(f, |this, f| this.on_click(move |_, w, cx| f(l, w, cx)))
                    .child(EffortGlyph::new(l))
                    .child((texts.level)(l))
                    // `.t3-erow > svg { margin-left: auto; fill: interactive }`.
                    .when(on, |this| {
                        this.child(div().ml_auto().child(press::icon(
                            CarbonIcon::Checkmark,
                            16.,
                            color(ColorRole::Interactive),
                        )))
                    })
                    .when(ring, |this| this.child(press::ring()))
                    .into_any_element()
            })
            .collect();
        let fast_change = self.on_fast_change.clone();
        // `.t3-mpick-effort { display: flex; flex-direction: column; padding: spacing-03 0;
        // border-left: 1px solid border-subtle-01 }`.
        let effort = div()
            .flex()
            .flex_col()
            .flex_none()
            .w(rem(MODEL_PICKER_GEOMETRY.1))
            .py(rem(8.))
            .border_l_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(header(texts.effort.clone()))
            .children(effort_rows)
            // gemini-B: `.t3-erow + .t3-mpick-h { margin-top: spacing-04; padding-top: spacing-04;
            // border-top: 1px solid border-subtle-01 }`.
            .child(
                header(texts.speed.clone())
                    .mt(rem(12.))
                    .pt(rem(12.))
                    .border_t_1()
                    .border_color(color(ColorRole::BorderSubtle01)),
            )
            // `.t3-mpick-effort .cds--toggle { padding: 0 spacing-04 }`, `size="sm"`, labels Standard / Fast.
            .child(
                div().px(rem(12.)).child(
                    // `<Cb.Toggle labelText="Fast mode" hideLabel labelA="Standard"
                    // labelB="Fast">`: with `hideLabel` Carbon shows `labelText`
                    // beside the track in both states (the reference render).
                    Switch::new((id.clone(), "fast"))
                        .size(SwitchSize::Sm)
                        .checked(self.fast)
                        .label_off(texts.fast_mode.clone())
                        .label_on(texts.fast_mode.clone())
                        .on_checked_change(move |on, w, cx| {
                            if let Some(f) = fast_change.as_ref() {
                                f(on, w, cx)
                            }
                        }),
                ),
            );
        // `.t3-mpick { display: grid; grid-template-columns: 11rem 1fr 10rem; width: 100%;
        // max-width: 46rem; height: 22rem; background: layer-01; box-shadow: 0 2px 6px shadow }`.
        div()
            .flex()
            .flex_row()
            .w_full()
            .max_w(rem(MODEL_PICKER_GEOMETRY.2))
            .h(rem(MODEL_PICKER_GEOMETRY.3))
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .text_color(color(ColorRole::TextPrimary))
            .child(side)
            .child(main)
            .child(effort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_filters_by_name() {
        let models = vec![
            PickerModel::new("opus", "Opus", "Most capable"),
            PickerModel::new("sonnet", "Sonnet", "Fast and capable"),
        ];
        let hits = filter_models(&models, "SON");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "sonnet");
        assert_eq!(filter_models(&models, "").len(), 2);
        assert_eq!((ModelPickerTexts::default().level)(EffortLevel::Max), "Max");
    }
}
