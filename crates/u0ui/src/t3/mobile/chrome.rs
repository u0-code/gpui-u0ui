//! T3 · Mobile chrome: [`PhoneFrame`], [`MobileAppBar`], [`MobileListSection`]
//! / [`ListRow`], [`MobileSheet`] and [`MobileTabDock`].
use super::{Handler, float_shadow, inset_frame};
use crate::{
    icons::{CarbonIcon, Icon},
    motion::{self, Curve, Motion, Speed},
    t3::craft::{self, focus_handle, focus_visible, press, rem, sans, sub},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, BoxShadow, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window,
    component::{Sizable as _, Size, input::Input as KitInput, input::InputState},
    div,
    prelude::FluentBuilder as _,
    px, relative,
};
use std::rc::Rc;

type BoolHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type ValueHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

// ── PhoneFrame ───────────────────────────────────────────────────────────

/// A phone bezel for demos (`.t3-phone`): 22rem wide, 10px
/// `background-inverse` bezel with 44px corners, a black notch, and a
/// `background` screen with 36px corners and a 44px status-bar inset.
#[derive(IntoElement)]
pub struct PhoneFrame {
    height: f32,
    children: Vec<AnyElement>,
}

impl PhoneFrame {
    /// A frame `height` px tall (`height`, default 560).
    pub fn new() -> Self {
        Self {
            height: 560.,
            children: Vec::new(),
        }
    }

    /// The frame height in px.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl Default for PhoneFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for PhoneFrame {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for PhoneFrame {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .relative()
            .flex_none()
            .w(rem(352.))
            .h(rem(self.height))
            .p(px(10.))
            .rounded(rem(44.))
            .bg(color(ColorRole::BackgroundInverse))
            .shadow(float_shadow(2.))
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_hidden()
                    .pt(rem(44.))
                    .rounded(rem(36.))
                    .bg(color(ColorRole::Background))
                    .children(self.children),
            )
            // `.t3-phone-notch { top: 18px; 5.5rem × 1.5rem; radius 1rem; #000 }`.
            .child(
                div()
                    .absolute()
                    .top(px(18.))
                    .left_0()
                    .right_0()
                    .flex()
                    .justify_center()
                    .child(
                        div()
                            .w(rem(88.))
                            .h(rem(24.))
                            .rounded(rem(16.))
                            .bg(gpui_kit::black()),
                    ),
            )
    }
}

// ── MobileAppBar ─────────────────────────────────────────────────────────

/// The app bar's leading control (`leading`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Leading {
    /// `back`, the default: a chevron.
    #[default]
    Back,
    /// `menu`: a hamburger.
    Menu,
    /// Anything else: a 44px spacer.
    None,
}

/// A 44px round icon button in `link-primary` (`.t3-mbtn`); pressing
/// paints `background-hover`.
fn mbtn(
    id: ElementId,
    label: SharedString,
    icon: CarbonIcon,
    on_press: Option<Handler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let focus = focus_handle(&id, window, cx);
    let ring = focus_visible(&focus, window);
    press(id, &focus)
        .accessibility_label(label)
        .flex_none()
        .justify_center()
        .size(rem(44.))
        .rounded_full()
        .active(|s| s.bg(color(ColorRole::BackgroundHover)))
        .when_some(on_press, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
        .child(
            Icon::carbon(icon)
                .size_length(rem(20.))
                .color(color(ColorRole::LinkPrimary)),
        )
        .when(ring, |this| this.child(craft::focus_ring()))
        .into_any_element()
}

/// One app bar action (`actions[]`).
#[derive(Clone)]
pub struct BarAction {
    /// `label` (the accessible name).
    pub label: SharedString,
    /// `icon`.
    pub icon: CarbonIcon,
    on_press: Option<Handler>,
}

impl BarAction {
    /// An action.
    pub fn new(label: impl Into<SharedString>, icon: CarbonIcon) -> Self {
        Self {
            label: label.into(),
            icon,
            on_press: None,
        }
    }

    /// Pressed.
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
}

/// Side slot width of a phone nav bar with a leading button (`r3-mobB`):
/// 88px, or 52px with at most one action, so the title stays on the axis.
pub fn nav_slot(actions: usize) -> f32 {
    if actions > 1 { 88. } else { 52. }
}

/// The mobile top bar (`.t3-mbar`): a 44px row with Back / Menu, a centred
/// title and subtitle and icon actions; `large` moves the title to a 32px
/// heading under the row; `searchable` adds a search field.
#[derive(IntoElement)]
pub struct MobileAppBar {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    leading: Leading,
    actions: Vec<BarAction>,
    large: bool,
    searchable: bool,
    phone: bool,
    on_leading: Option<Handler>,
    back_label: SharedString,
    menu_label: SharedString,
    search_label: SharedString,
}

impl MobileAppBar {
    /// A bar titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            leading: Leading::Back,
            actions: Vec::new(),
            large: false,
            searchable: false,
            phone: false,
            on_leading: None,
            back_label: "Back".into(),
            menu_label: "Menu".into(),
            search_label: "Search".into(),
        }
    }

    /// A subtitle under the title.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// The leading control.
    pub fn leading(mut self, leading: Leading) -> Self {
        self.leading = leading;
        self
    }

    /// Trailing actions.
    pub fn actions(mut self, actions: impl IntoIterator<Item = BarAction>) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }

    /// Large title (list pages).
    pub fn large(mut self, large: bool) -> Self {
        self.large = large;
        self
    }

    /// A search field under the title.
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    /// The shared phone chrome (`[data-mb]`).
    pub fn phone(mut self, phone: bool) -> Self {
        self.phone = phone;
        self
    }

    /// Back / Menu pressed.
    pub fn on_leading(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_leading = Some(Rc::new(f));
        self
    }

    /// Accessible names of Back, Menu and the search field.
    pub fn labels(
        mut self,
        back: impl Into<SharedString>,
        menu: impl Into<SharedString>,
        search: impl Into<SharedString>,
    ) -> Self {
        self.back_label = back.into();
        self.menu_label = menu.into();
        self.search_label = search.into();
        self
    }
}

impl RenderOnce for MobileAppBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let lead = match self.leading {
            Leading::Back => Some((self.back_label.clone(), CarbonIcon::ChevronLeft)),
            Leading::Menu => Some((self.menu_label.clone(), CarbonIcon::Menu)),
            Leading::None => None,
        };
        let lead_el = match lead.clone() {
            Some((label, icon)) => mbtn(
                (id.clone(), "lead").into(),
                label,
                icon,
                self.on_leading.clone(),
                window,
                cx,
            ),
            None => div().flex_none().w(rem(44.)).into_any_element(),
        };
        let actions: Vec<AnyElement> = self
            .actions
            .iter()
            .enumerate()
            .map(|(i, a)| {
                mbtn(
                    sub(&id, "act", i),
                    a.label.clone(),
                    a.icon,
                    a.on_press.clone(),
                    window,
                    cx,
                )
            })
            .collect();
        let n_actions = actions.len();
        let title = (!self.large).then(|| {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .items_center()
                .text_center()
                .child(
                    sans(div(), 16., 600)
                        .max_w_full()
                        .when(self.phone, |d| {
                            d.overflow_hidden().whitespace_nowrap().text_ellipsis()
                        })
                        .text_color(color(ColorRole::TextPrimary))
                        .child(self.title.clone()),
                )
                .when_some(self.subtitle.clone(), |this, s| {
                    this.child(
                        craft::text(s, 12., ColorRole::TextSecondary)
                            .max_w_full()
                            .when(self.phone, |d| {
                                d.overflow_hidden().whitespace_nowrap().text_ellipsis()
                            }),
                    )
                })
        });
        let acts = div().flex().children(actions);
        let row = if self.phone && lead.is_some() {
            // `[data-mb] .t3-mbar-row:has(> .t3-mbtn:first-child) { grid: 88px 1fr 88px }`.
            let slot = nav_slot(n_actions);
            div()
                .flex()
                .items_center()
                .h(rem(44.))
                .child(div().flex_none().w(px(slot)).child(lead_el))
                .child(title.unwrap_or_else(|| div().flex_1()))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .w(px(slot))
                        .justify_end()
                        .child(acts),
                )
        } else {
            div()
                .flex()
                .items_center()
                .gap(rem(4.))
                .h(rem(44.))
                .child(lead_el)
                .child(title.unwrap_or_else(|| div().flex_1()))
                .child(acts)
        };
        let search = self.searchable.then(|| {
            let state = window
                .use_keyed_state((id.clone(), "search"), cx, |window, cx| {
                    let placeholder = self.search_label.clone();
                    cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
                })
                .read(cx)
                .clone();
            // `.t3-mbar-search { height: 36px; margin: 8px 8px 0; padding: 0 12px;
            // background: layer-accent-01 }`; `[data-mb]`: 40px, 12px top, `field-01`
            // with a `border-strong-01` rule.
            div()
                .flex()
                .items_center()
                .gap(rem(8.))
                .h(rem(if self.phone { 40. } else { 36. }))
                .mt(rem(if self.phone { 12. } else { 8. }))
                .mx(rem(8.))
                .px(rem(12.))
                .bg(color(if self.phone {
                    ColorRole::Field01
                } else {
                    ColorRole::LayerAccent01
                }))
                .when(self.phone, |d| {
                    d.border_b_1()
                        .border_color(color(ColorRole::BorderStrong01))
                })
                .child(
                    Icon::carbon(CarbonIcon::Search)
                        .size_length(rem(16.))
                        .color(color(ColorRole::IconSecondary)),
                )
                .child(
                    KitInput::new(&state)
                        .appearance(false)
                        .with_size(Size::XSmall)
                        .p_0()
                        .flex_1()
                        .min_w_0()
                        .text_size(rem(16.))
                        .text_color(color(ColorRole::TextPrimary)),
                )
        });
        div()
            .id(id)
            .flex()
            .flex_col()
            .px(rem(8.))
            .pb(rem(8.))
            .bg(color(ColorRole::Background))
            .child(row)
            .when(self.large, |this| {
                this.child(
                    sans(div(), 32., 600)
                        .px(rem(8.))
                        .when(self.phone, |d| d.line_height(rem(40.)))
                        .text_color(color(ColorRole::TextPrimary))
                        .child(self.title.clone()),
                )
            })
            .children(search)
    }
}

// ── MobileList ───────────────────────────────────────────────────────────

/// A list row's kind (`kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RowKind {
    /// `navigation`, the default: value and chevron.
    #[default]
    Navigation,
    /// `toggle`: a switch.
    Toggle,
    /// `choice`: a check when chosen.
    Choice,
    /// `action`: link-colored text.
    Action,
}

impl RowKind {
    /// Every kind.
    pub const ALL: [RowKind; 4] = [Self::Navigation, Self::Toggle, Self::Choice, Self::Action];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Navigation => "navigation",
            Self::Toggle => "toggle",
            Self::Choice => "choice",
            Self::Action => "action",
        }
    }
}

/// Geometry of the mobile switch (`.t3-mswitch`): (track w, track h, knob,
/// inset, travel). Phone chrome uses Carbon toggle proportions (48×24).
pub fn switch_geometry(phone: bool) -> (f32, f32, f32, f32, f32) {
    if phone {
        (48., 24., 18., 3., 24.)
    } else {
        (48., 28., 24., 2., 20.)
    }
}

/// A grouped-list row (`.t3-mrowx`, at least 44px; 56px in phone chrome).
#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    icon: Option<CarbonIcon>,
    title: SharedString,
    subtitle: Option<SharedString>,
    value: Option<SharedString>,
    kind: RowKind,
    checked: bool,
    on: Option<bool>,
    default_on: bool,
    destructive: bool,
    on_press: Option<Handler>,
    on_toggle: Option<BoolHandler>,
    phone: bool,
    divider: Option<f32>,
}

impl ListRow {
    /// A navigation row titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            title: title.into(),
            subtitle: None,
            value: None,
            kind: RowKind::Navigation,
            checked: false,
            on: None,
            default_on: false,
            destructive: false,
            on_press: None,
            on_toggle: None,
            phone: false,
            divider: None,
        }
    }

    /// A leading icon tile.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// A subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// A trailing value.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The kind.
    pub fn kind(mut self, kind: RowKind) -> Self {
        self.kind = kind;
        self
    }

    /// Chosen (choice rows).
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Controlled switch state (toggle rows).
    pub fn on(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    /// Initial switch state when uncontrolled (`defaultOn`).
    pub fn default_on(mut self, on: bool) -> Self {
        self.default_on = on;
        self
    }

    /// Destructive (`text-error`).
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    /// Pressed (`onPress`; navigation, choice and action rows).
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }

    /// The switch changed (toggle rows).
    pub fn on_toggle(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// The shared phone chrome (`[data-mb]`).
    pub fn phone(mut self, phone: bool) -> Self {
        self.phone = phone;
        self
    }

    fn has_icon(&self) -> bool {
        self.icon.is_some()
    }
}

/// The start inset of the hairline above a row that follows another
/// (`.t3-mrowx + .t3-mrowx::before`): 52px after an icon tile, else 16px.
/// Phone chrome also counts an icon on the row above.
pub fn divider_inset(phone: bool, icon: bool, previous_icon: bool) -> f32 {
    if icon || (phone && previous_icon) {
        52.
    } else {
        16.
    }
}

impl RenderOnce for ListRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let ink = color(if self.destructive {
            ColorRole::TextError
        } else if self.kind == RowKind::Action {
            ColorRole::LinkPrimary
        } else {
            ColorRole::TextPrimary
        });
        let icon = self.icon.map(|icon| {
            // `.t3-mrowx-icon { 28×28; background: interactive; color: text-on-color }`;
            // phone: no tile, `icon-primary` (`text-error` when destructive), 20px wide.
            let (bg, fg) = if self.phone {
                (
                    None,
                    color(if self.destructive {
                        ColorRole::TextError
                    } else {
                        ColorRole::IconPrimary
                    }),
                )
            } else {
                (
                    Some(color(ColorRole::Interactive)),
                    color(ColorRole::TextOnColor),
                )
            };
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .w(rem(if self.phone { 20. } else { 28. }))
                .h(rem(28.))
                .when_some(bg, |d, bg| d.bg(bg))
                .child(Icon::carbon(icon).size_length(rem(20.)).color(fg))
        });
        let text = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .when(self.phone, |d| d.gap(px(2.)))
            .child(self.title.clone())
            .when_some(self.subtitle.clone(), |d, s| {
                d.child(craft::text(s, 13., ColorRole::TextSecondary))
            });
        let value = |v: SharedString| craft::text(v, 16., ColorRole::TextSecondary);
        let focus = focus_handle(&id, window, cx);
        let ring = focus_visible(&focus, window);
        let row = sans(div(), 16., 400)
            .relative()
            .flex()
            .items_center()
            .gap(rem(12.))
            .w_full()
            .min_h(rem(if self.phone { 56. } else { 44. }))
            .px(rem(16.))
            .py(rem(8.))
            .text_color(ink)
            .children(icon)
            .child(text);
        let row = match self.kind {
            RowKind::Toggle => {
                let (on, store) = crate::components::form::controlled(
                    (id.clone(), "on"),
                    self.on,
                    self.default_on,
                    window,
                    cx,
                );
                let (w, h, knob, inset, travel) = switch_geometry(self.phone);
                // `.t3-mswitch { transition: background 110ms }`,
                // `span { transition: transform 110ms cubic-bezier(0.2,0,0.38,0.9) }`.
                let fade = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
                let bg = motion::animate_color(
                    (id.clone(), "track"),
                    color(if on {
                        ColorRole::SupportSuccess
                    } else {
                        ColorRole::ToggleOff
                    }),
                    fade,
                    window,
                    cx,
                );
                let x = motion::animate(
                    (id.clone(), "knob"),
                    if on { travel } else { 0. },
                    fade.spatial(true),
                    window,
                    cx,
                );
                let on_toggle = self.on_toggle.clone();
                let phone = self.phone;
                row.child(
                    press(id.clone(), &focus)
                        .role(gpui_kit::Role::Switch)
                        .accessibility_label(self.title.clone())
                        .flex_none()
                        .w(rem(w))
                        .h(rem(h))
                        .rounded(rem(16.))
                        .bg(bg)
                        .on_click(move |_, window, cx| {
                            store.update(cx, |v, cx| {
                                *v = !on;
                                cx.notify();
                            });
                            if let Some(f) = &on_toggle {
                                f(!on, window, cx);
                            }
                        })
                        .child(
                            div()
                                .absolute()
                                .top(px(inset))
                                .left(px(inset + x))
                                .size(rem(knob))
                                .rounded_full()
                                .bg(gpui_kit::white())
                                // `box-shadow: 0 1px 2px rgba(0,0,0,.3)`; none in phone chrome.
                                .when(!phone, |d| {
                                    d.shadow(vec![
                                        BoxShadow::new(
                                            px(0.),
                                            px(1.),
                                            gpui_kit::black().opacity(0.3),
                                        )
                                        .blur_radius(px(2.)),
                                    ])
                                }),
                        )
                        .when(ring, |this| this.child(craft::focus_ring())),
                )
                .into_any_element()
            }
            kind => {
                let right: Option<AnyElement> = match kind {
                    RowKind::Navigation => Some(
                        div()
                            .flex()
                            .items_center()
                            .gap(rem(12.))
                            .children(self.value.clone().map(value))
                            .child(
                                Icon::carbon(CarbonIcon::ChevronRight)
                                    .size_length(rem(16.))
                                    .color(color(ColorRole::IconSecondary)),
                            )
                            .into_any_element(),
                    ),
                    RowKind::Choice => self.checked.then(|| {
                        Icon::carbon(CarbonIcon::Checkmark)
                            .size_length(rem(20.))
                            .color(color(ColorRole::Interactive))
                            .into_any_element()
                    }),
                    _ => self.value.clone().map(|v| value(v).into_any_element()),
                };
                let on_press = self.on_press.clone();
                press(id.clone(), &focus)
                    .when(kind == RowKind::Choice, |b| {
                        b.role(gpui_kit::Role::MenuItemRadio)
                    })
                    .accessibility_label(self.title.clone())
                    .w_full()
                    .active(|s| s.bg(color(ColorRole::LayerHover01)))
                    .when_some(on_press, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
                    .child(row.children(right))
                    .when(ring, |this| this.child(craft::focus_ring()))
                    .into_any_element()
            }
        };
        div()
            .relative()
            .w_full()
            .child(row)
            .when_some(self.divider, |this, left| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left(rem(left))
                        .right_0()
                        .h(px(1.))
                        .bg(color(if self.phone {
                            ColorRole::BorderSubtle00
                        } else {
                            ColorRole::BorderSubtle01
                        })),
                )
            })
    }
}

/// A grouped list section (`.t3-mlist`): uppercase 12px heading, a square
/// `layer-01` card of [`ListRow`]s with hairlines between them, an optional
/// footer.
#[derive(IntoElement)]
pub struct MobileListSection {
    title: SharedString,
    footer: Option<SharedString>,
    rows: Vec<ListRow>,
    raised: bool,
    phone: bool,
}

impl MobileListSection {
    /// A section titled `title` (empty for none).
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            footer: None,
            rows: Vec::new(),
            raised: false,
            phone: false,
        }
    }

    /// A footer note.
    pub fn footer(mut self, footer: impl Into<SharedString>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Add a row.
    pub fn row(mut self, row: ListRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Add rows.
    pub fn rows(mut self, rows: impl IntoIterator<Item = ListRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// Paint the card `layer-02` (inside a sheet: `.t3-msheet-body
    /// .t3-mlist-card`, or a `[data-grouped]` phone page).
    pub fn raised(mut self, raised: bool) -> Self {
        self.raised = raised;
        self
    }

    /// The shared phone chrome (`[data-mb]`).
    pub fn phone(mut self, phone: bool) -> Self {
        self.phone = phone;
        self
    }
}

impl RenderOnce for MobileListSection {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let phone = self.phone;
        let mut previous_icon = false;
        let rows: Vec<ListRow> = self
            .rows
            .into_iter()
            .enumerate()
            .map(|(i, mut row)| {
                row.phone = row.phone || phone;
                let icon = row.has_icon();
                if i > 0 {
                    row.divider = Some(divider_inset(row.phone, icon, previous_icon));
                }
                previous_icon = icon;
                row
            })
            .collect();
        div()
            .flex()
            .flex_col()
            .px(rem(16.))
            .when(!phone, |d| d.pb(rem(24.)))
            .child(
                // `.t3-mlist-h { padding: 0 0 8px; 12px uppercase text-secondary }`.
                craft::text(self.title.to_uppercase(), 12., ColorRole::TextSecondary).pb(rem(8.)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .bg(color(if self.raised {
                        ColorRole::Layer02
                    } else {
                        ColorRole::Layer01
                    }))
                    .children(rows),
            )
            .when_some(self.footer, |this, f| {
                this.child(craft::text(f, 12., ColorRole::TextSecondary).pt(rem(8.)))
            })
    }
}

// ── MobileSheet ──────────────────────────────────────────────────────────

/// A bottom sheet (`.t3-msheet`, square per `gemini-MA`): grabber, a
/// three-column head (Close · title · primary), an optional centred intro
/// and the body (its list cards paint `layer-02`).
#[derive(IntoElement)]
pub struct MobileSheet {
    id: ElementId,
    title: SharedString,
    intro: Option<SharedString>,
    primary: Option<SharedString>,
    inline: bool,
    on_close: Option<Handler>,
    on_primary: Option<Handler>,
    close_label: SharedString,
    children: Vec<AnyElement>,
}

impl MobileSheet {
    /// A sheet titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            intro: None,
            primary: None,
            inline: true,
            on_close: None,
            on_primary: None,
            close_label: "Close".into(),
            children: Vec::new(),
        }
    }

    /// The intro line.
    pub fn intro(mut self, intro: impl Into<SharedString>) -> Self {
        self.intro = Some(intro.into());
        self
    }

    /// The primary action text (e.g. `Start`).
    pub fn primary(mut self, primary: impl Into<SharedString>) -> Self {
        self.primary = Some(primary.into());
        self
    }

    /// Sit at the bottom of its column (`inline`, default true: `margin-top: auto`).
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// Close pressed.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// Primary pressed.
    pub fn on_primary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_primary = Some(Rc::new(f));
        self
    }

    /// The close button's name (`Close`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }
}

impl ParentElement for MobileSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MobileSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let close = mbtn(
            (id.clone(), "close").into(),
            self.close_label.clone(),
            CarbonIcon::Close,
            self.on_close.clone(),
            window,
            cx,
        );
        let primary = match self.primary.clone() {
            Some(label) => {
                let pid: ElementId = (id.clone(), "primary").into();
                let focus = focus_handle(&pid, window, cx);
                let ring = focus_visible(&focus, window);
                let on_primary = self.on_primary.clone();
                sans(press(pid, &focus), 16., 600)
                    .h(rem(44.))
                    .px(rem(12.))
                    .text_color(color(ColorRole::LinkPrimary))
                    .when_some(on_primary, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
                    .child(label)
                    .when(ring, |b| b.child(craft::focus_ring()))
                    .into_any_element()
            }
            None => div().w(rem(44.)).into_any_element(),
        };
        div()
            .id(id)
            .flex()
            .flex_col()
            .w_full()
            .when(self.inline, |d| d.mt_auto())
            .bg(color(ColorRole::Layer01))
            // `box-shadow: 0 -2px 6px var(--shadow)`.
            .shadow(float_shadow(-2.))
            .child(
                div()
                    .mt(rem(8.))
                    .mx_auto()
                    .w(rem(36.))
                    .h(px(5.))
                    .rounded(px(3.))
                    .bg(color(ColorRole::BorderStrong01)),
            )
            .child(
                // `grid-template-columns: 1fr auto 1fr`.
                div()
                    .flex()
                    .items_center()
                    .px(rem(8.))
                    .child(div().flex().flex_1().flex_basis(px(0.)).child(close))
                    .child(
                        sans(div(), 16., 600)
                            .flex_none()
                            .text_color(color(ColorRole::TextPrimary))
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .flex_basis(px(0.))
                            .justify_end()
                            .child(primary),
                    ),
            )
            .when_some(self.intro.clone(), |this, intro| {
                this.child(
                    craft::text(intro, 14., ColorRole::TextSecondary)
                        .px(rem(24.))
                        .pb(rem(12.))
                        .text_center(),
                )
            })
            .child(div().pb(rem(32.)).children(self.children))
    }
}

// ── MobileTabDock ────────────────────────────────────────────────────────

/// One dock tab (`tabs[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockTab {
    /// `id`.
    pub id: SharedString,
    /// `label`.
    pub label: SharedString,
    /// `icon`.
    pub icon: CarbonIcon,
    /// `badge`: the needs-you count.
    pub badge: Option<usize>,
}

impl DockTab {
    /// A tab.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: CarbonIcon,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            badge: None,
        }
    }

    /// A count badge.
    pub fn badge(mut self, badge: usize) -> Self {
        self.badge = Some(badge);
        self
    }
}

/// The bottom tab dock (`.t3-dock`): a floating square bar on `layer-02`
/// with the floating shadow; the current tab is tinted `layer-selected-01`
/// in `link-primary`. Phone chrome makes it a flat full-width 56px bar with
/// a home indicator. Badges are the yellow needs-you count.
#[derive(IntoElement)]
pub struct MobileTabDock {
    id: ElementId,
    tabs: Vec<DockTab>,
    value: Option<SharedString>,
    on_change: Option<ValueHandler>,
    phone: bool,
}

impl MobileTabDock {
    /// A dock.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            value: None,
            on_change: None,
            phone: false,
        }
    }

    /// The tabs.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = DockTab>) -> Self {
        self.tabs = tabs.into_iter().collect();
        self
    }

    /// Controlled current tab id.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// A tab was chosen (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// The shared phone chrome (`[data-mb]`).
    pub fn phone(mut self, phone: bool) -> Self {
        self.phone = phone;
        self
    }
}

/// The home indicator (`[data-mb] .mb-home`, `.t3-dock::after`): 134×5,
/// radius 3, `text-primary`, 8px above the bottom edge.
pub fn home_indicator() -> AnyElement {
    div()
        .absolute()
        .bottom(px(8.))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            div()
                .w(px(134.))
                .h(px(5.))
                .rounded(px(3.))
                .bg(color(ColorRole::TextPrimary)),
        )
        .into_any_element()
}

impl RenderOnce for MobileTabDock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let first = self.tabs.first().map(|t| t.id.clone()).unwrap_or_default();
        let (current, store) = crate::components::form::controlled(
            (id.clone(), "value"),
            self.value.clone(),
            first,
            window,
            cx,
        );
        let phone = self.phone;
        let tabs = self.tabs.iter().enumerate().map(|(i, t)| {
            let on = t.id == current;
            let tid = sub(&id, "tab", i);
            let focus = focus_handle(&tid, window, cx);
            let ring = focus_visible(&focus, window);
            let ink = color(if !on {
                ColorRole::TextSecondary
            } else if phone {
                ColorRole::Interactive
            } else {
                ColorRole::LinkPrimary
            });
            let value = t.id.clone();
            let store = store.clone();
            let on_change = self.on_change.clone();
            // Phone: `top: 3px; left: calc(50% + 6px)`, 18px tall, a 2px
            // `background` ring; card: `top: 2px; right: 0.9rem`, 16px.
            let badge = t.badge.map(|n| {
                let b = sans(
                    div(),
                    if phone { 11. } else { 10. },
                    if phone { 600 } else { 400 },
                )
                .absolute()
                .min_w(rem(if phone { 18. } else { 16. }))
                .px(px(if phone { 5. } else { 4. }))
                .h(rem(if phone { 18. } else { 16. }))
                .line_height(rem(if phone { 18. } else { 16. }))
                .rounded(rem(if phone { 9. } else { 8. }))
                .text_center()
                .bg(color(ColorRole::SupportWarning))
                .text_color(crate::t3::base::ink())
                .child(n.to_string());
                if phone {
                    b.top(px(3.)).left(relative(0.5)).ml(px(6.)).shadow(vec![
                        BoxShadow::new(px(0.), px(0.), color(ColorRole::Background))
                            .spread_radius(px(2.)),
                    ])
                } else {
                    b.top(px(2.)).right(rem(14.4))
                }
            });
            sans(
                press(tid, &focus),
                if phone { 12. } else { 11. },
                if on && phone { 600 } else { 400 },
            )
            .role(gpui_kit::Role::Tab)
            .accessibility_label(t.label.clone())
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(if phone { 4. } else { 2. }))
            .when(phone, |b| b.flex_1().min_w_0().h(px(56.)))
            .when(!phone, |b| {
                b.min_w(rem(64.))
                    .px(rem(12.))
                    .py(rem(4.))
                    .when(on, |b| b.bg(color(ColorRole::LayerSelected01)))
            })
            .text_color(ink)
            .on_click(move |_, window, cx| {
                store.update(cx, |v, cx| {
                    *v = value.clone();
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(value.clone(), window, cx);
                }
            })
            .child(Icon::carbon(t.icon).size_length(rem(20.)).color(ink))
            .child(t.label.clone())
            .children(badge)
            .when(ring, |b| b.child(craft::focus_ring()))
            .into_any_element()
        });
        let tabs: Vec<AnyElement> = tabs.collect();
        if phone {
            // `[data-mb] .t3-dock { padding: 0 8px 34px; background; border-top }`.
            div()
                .id(id)
                .relative()
                .flex()
                .w_full()
                .px(rem(8.))
                .pb(px(34.))
                .bg(color(ColorRole::Background))
                .border_t_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .children(tabs)
                .child(home_indicator())
        } else {
            div()
                .id(id)
                .relative()
                .flex()
                .self_center()
                .mx_auto()
                .mt(rem(8.))
                .mb(rem(16.))
                .gap(rem(2.))
                .p(rem(4.))
                .bg(color(ColorRole::Layer02))
                .shadow(float_shadow(2.))
                .children(tabs)
                .child(inset_frame(color(ColorRole::BorderSubtle01)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_and_bar_geometry() {
        assert_eq!(divider_inset(false, true, false), 52.);
        assert_eq!(divider_inset(false, false, true), 16.);
        assert_eq!(divider_inset(true, false, true), 52.);
        assert_eq!(nav_slot(2), 88.);
        assert_eq!(nav_slot(1), 52.);
        assert_eq!(switch_geometry(false), (48., 28., 24., 2., 20.));
        assert_eq!(switch_geometry(true), (48., 24., 18., 3., 24.));
    }
}
