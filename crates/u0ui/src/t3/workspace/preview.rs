//! T3 · Files & preview: [`PreviewChrome`], [`PreviewEmptyState`],
//! [`MiniPlayer`] and [`DeviceControlsRail`] (`src/t3/workspace.jsx`,
//! "Preview" in `src/t3.css`).
use super::parts::{card_sub, child_id, icon_button};
use crate::{
    components::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuKind, DropdownMenuSize,
        MenuItem, floating::Align,
    },
    icons::{CarbonIcon, Icon},
    motion::{self, cubic_bezier_at},
    t3::{
        base::EASE_IN_OUT,
        craft::{mono, sans},
        press::{self, Handler, rem},
        shell::{ConnectionStatus, ConnectionStatusDot},
    },
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, AppContext as _, CursorStyle, ElementId, Entity,
    Focusable as _, InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _,
        input::{Input, InputState},
    },
    div,
    prelude::FluentBuilder as _,
    px, relative,
};
use std::{rc::Rc, time::Duration};

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

// ── PreviewChrome ───────────────────────────────────────────────────────

/// The viewports of the reference menu (`['Responsive', 'iPhone 16',
/// 'iPad', 'Desktop 1440']`).
pub const VIEWPORTS: [&str; 4] = ["Responsive", "iPhone 16", "iPad", "Desktop 1440"];

/// `@keyframes t3-load { from { left: -40% } to { left: 100% } }`, 1.2s
/// `ease-in-out`, a bar 40% wide.
pub const LOAD_PERIOD: Duration = Duration::from_millis(1200);
/// `.t3-pchrome-load { width: 40% }`.
pub const LOAD_WIDTH: f32 = 0.4;

/// The loading bar's `left` (a fraction of the field) at cycle progress `t`.
pub fn load_left(t: f32) -> f32 {
    let [x1, y1, x2, y2] = EASE_IN_OUT;
    -LOAD_WIDTH + (1. + LOAD_WIDTH) * cubic_bezier_at(x1, y1, x2, y2, t.clamp(0., 1.))
}

/// Texts of a [`PreviewChrome`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct PreviewChromeTexts {
    /// `Back`.
    pub back: SharedString,
    /// `Forward`.
    pub forward: SharedString,
    /// `Reload`.
    pub reload: SharedString,
    /// `Stop` (while loading).
    pub stop: SharedString,
    /// The address field's name (`Address`).
    pub address: SharedString,
    /// The viewport radio group's name (`Viewport`).
    pub viewport: SharedString,
    /// `Open in system browser`.
    pub open_external: SharedString,
    /// `More`.
    pub more: SharedString,
}

impl Default for PreviewChromeTexts {
    fn default() -> Self {
        Self {
            back: "Back".into(),
            forward: "Forward".into(),
            reload: "Reload".into(),
            stop: "Stop".into(),
            address: "Address".into(),
            viewport: "Viewport".into(),
            open_external: "Open in system browser".into(),
            more: "More".into(),
        }
    }
}

struct AddressState {
    input: Entity<InputState>,
    url: SharedString,
}

/// The built-in browser's toolbar (`PreviewChrome`, `.t3-pchrome`): back,
/// forward, reload or stop, the address field (a lock when `secure`, a
/// sliding 2px bar while `loading`), the zoom when not 100%, the viewport
/// menu, open in the system browser and more. 40px on `layer-01`.
#[derive(IntoElement)]
pub struct PreviewChrome {
    id: ElementId,
    url: SharedString,
    loading: bool,
    secure: bool,
    zoom: u32,
    viewports: Vec<SharedString>,
    viewport: SharedString,
    can_back: bool,
    can_forward: bool,
    texts: PreviewChromeTexts,
    on_back: Option<Handler>,
    on_forward: Option<Handler>,
    on_reload: Option<Handler>,
    on_stop: Option<Handler>,
    on_navigate: Option<super::parts::TextHandler>,
    on_viewport_change: Option<IndexHandler>,
    on_open_external: Option<Handler>,
    on_more: Option<Handler>,
}

impl PreviewChrome {
    /// A toolbar showing `url` (`http://localhost:5173` in the reference).
    pub fn new(id: impl Into<ElementId>, url: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            loading: false,
            secure: false,
            zoom: 100,
            viewports: VIEWPORTS.iter().map(|v| SharedString::from(*v)).collect(),
            viewport: VIEWPORTS[0].into(),
            can_back: true,
            can_forward: false,
            texts: PreviewChromeTexts::default(),
            on_back: None,
            on_forward: None,
            on_reload: None,
            on_stop: None,
            on_navigate: None,
            on_viewport_change: None,
            on_open_external: None,
            on_more: None,
        }
    }

    /// Loading: Stop replaces Reload and the address bar slides (`loading`).
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// A lock instead of the globe (`secure`).
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// The zoom in percent (`zoom`, default 100: hidden).
    pub fn zoom(mut self, zoom: u32) -> Self {
        self.zoom = zoom;
        self
    }

    /// The viewport shown on the menu button (`viewport`, default `Responsive`).
    pub fn viewport(mut self, viewport: impl Into<SharedString>) -> Self {
        self.viewport = viewport.into();
        self
    }

    /// Replace the viewport list ([`VIEWPORTS`]).
    pub fn viewports<S: Into<SharedString>>(mut self, list: impl IntoIterator<Item = S>) -> Self {
        self.viewports = list.into_iter().map(Into::into).collect();
        self
    }

    /// Whether Back is enabled (default true).
    pub fn can_back(mut self, can: bool) -> Self {
        self.can_back = can;
        self
    }

    /// Whether Forward is enabled (default false, as in the reference).
    pub fn can_forward(mut self, can: bool) -> Self {
        self.can_forward = can;
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: PreviewChromeTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Back pressed.
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }

    /// Forward pressed.
    pub fn on_forward(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_forward = Some(Rc::new(f));
        self
    }

    /// Reload pressed.
    pub fn on_reload(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reload = Some(Rc::new(f));
        self
    }

    /// Stop pressed (while loading).
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }

    /// Enter in the address field, with its text.
    pub fn on_navigate(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(Rc::new(f));
        self
    }

    /// A viewport picked, with its index.
    pub fn on_viewport_change(
        mut self,
        f: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_viewport_change = Some(Rc::new(f));
        self
    }

    /// Open in system browser pressed.
    pub fn on_open_external(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open_external = Some(Rc::new(f));
        self
    }

    /// More pressed.
    pub fn on_more(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_more = Some(Rc::new(f));
        self
    }
}

fn click(b: Button, f: Option<Handler>) -> Button {
    match f {
        Some(f) => b.on_click(move |_, w, cx| f(w, cx)),
        None => b,
    }
}

impl RenderOnce for PreviewChrome {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = window.use_keyed_state((id.clone(), "address"), cx, {
            let url = self.url.clone();
            move |window, cx| AddressState {
                input: cx.new(|cx| InputState::new(window, cx).default_value(url.clone())),
                url,
            }
        });
        // A new `url` prop replaces the field's text (`defaultValue` per mount).
        if state.read(cx).url != self.url {
            let input = state.read(cx).input.clone();
            let url = self.url.clone();
            input.update(cx, |s, cx| s.set_value(url.clone(), window, cx));
            state.update(cx, |s, _| s.url = url);
        }
        let input = state.read(cx).input.clone();
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let reduced = motion::reduced_motion(cx);

        let load_bar = self.loading.then(|| {
            let bar = div()
                .absolute()
                .bottom(px(-1.))
                .h(px(2.))
                .bg(color(ColorRole::Interactive));
            if reduced {
                // `ghost.css`: `@media (prefers-reduced-motion: reduce) { * {
                // animation-duration: 1ms !important; animation-iteration-count: 1
                // !important } }` (`.t3-pchrome-load` is not in its `g-pulse` list):
                // `t3-load` ends at once and the bar rests at its base box,
                // `left: 0; width: 40%`.
                bar.left_0().w(relative(LOAD_WIDTH)).into_any_element()
            } else {
                // `.t3-pchrome-load { position: absolute; left: 0; bottom: -1px; height: 2px;
                // width: 40%; background: interactive; animation: t3-load 1.2s ease-in-out infinite }`.
                bar.w(relative(LOAD_WIDTH))
                    .with_animation(
                        (id.clone(), "load"),
                        Animation::new(LOAD_PERIOD).repeat(),
                        |bar, t| bar.left(relative(load_left(t))),
                    )
                    .into_any_element()
            }
        });

        let on_navigate = self.on_navigate.clone();
        // `.t3-pchrome-url { position: relative; display: flex; flex: 1; align-items: center;
        // gap: spacing-03; height: size-sm; margin: 0 spacing-03; padding: 0 spacing-04;
        // background: field-02; border-bottom: 1px solid border-strong-01; overflow: hidden;
        // min-width: 8rem }`.
        let address = div()
            .relative()
            .flex()
            .flex_row()
            .flex_1()
            .min_w(rem(128.))
            .items_center()
            .gap(rem(8.))
            .h(rem(32.))
            .mx(rem(8.))
            .px(rem(12.))
            .bg(color(ColorRole::Field02))
            .border_b_1()
            .border_color(color(ColorRole::BorderStrong01))
            .overflow_hidden()
            .child(
                press::icon(
                    if self.secure {
                        CarbonIcon::Locked
                    } else {
                        CarbonIcon::Earth
                    },
                    14.,
                    color(ColorRole::IconSecondary),
                )
                .flex_none(),
            )
            .child(
                // `input { flex: 1; font-family: mono; font-size: 0.75rem }`.
                mono(div(), 12., 400)
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .items_center()
                    .text_color(color(ColorRole::TextPrimary))
                    .on_key_down({
                        let input = input.clone();
                        move |event, window, cx| {
                            if event.keystroke.key == "enter"
                                && let Some(f) = on_navigate.as_ref()
                            {
                                let text: SharedString = input.read(cx).value().to_string().into();
                                f(text, window, cx);
                            }
                        }
                    })
                    .child(
                        Input::new(&input)
                            .appearance(false)
                            .with_size(gpui_kit::component::Size::XSmall)
                            .aria_label(self.texts.address.clone())
                            .p_0()
                            .h_full()
                            .w_full()
                            .text_size(rem(12.))
                            .line_height(relative(crate::t3::craft::BODY_LINE)),
                    ),
            )
            .children(load_bar)
            // `:focus-within { outline: 2px solid focus; outline-offset: -2px }`.
            .when(focused, |this| this.child(press::ring()));

        let picked = self.viewports.iter().position(|v| *v == self.viewport);
        let on_viewport = self.on_viewport_change.clone();
        let menu = DropdownMenu::new((id.clone(), "viewport"))
            .label(self.viewport.clone())
            .kind(DropdownMenuKind::Ghost)
            .size(DropdownMenuSize::Sm)
            .item({
                let item =
                    MenuItem::radio_group(self.texts.viewport.clone(), self.viewports.clone())
                        .selected_item(picked);
                match on_viewport {
                    Some(f) => item.on_select_item(move |i, w, cx| f(i, w, cx)),
                    None => item,
                }
            });

        // `.t3-pchrome { display: flex; align-items: center; gap: 2px; height: size-md;
        // padding: 0 spacing-02; background: layer-01; border-bottom: 1px solid border-subtle-01 }`.
        div()
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            .w_full()
            .h(rem(40.))
            .px(rem(4.))
            .bg(color(ColorRole::Layer01))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(click(
                icon_button(
                    (id.clone(), "back"),
                    self.texts.back.clone(),
                    CarbonIcon::ArrowLeft,
                )
                .disabled(!self.can_back),
                self.on_back.clone(),
            ))
            .child(click(
                icon_button(
                    (id.clone(), "forward"),
                    self.texts.forward.clone(),
                    CarbonIcon::ArrowRight,
                )
                .disabled(!self.can_forward),
                self.on_forward.clone(),
            ))
            .child(if self.loading {
                click(
                    icon_button(
                        (id.clone(), "stop"),
                        self.texts.stop.clone(),
                        CarbonIcon::Close,
                    ),
                    self.on_stop.clone(),
                )
            } else {
                click(
                    icon_button(
                        (id.clone(), "reload"),
                        self.texts.reload.clone(),
                        CarbonIcon::Renew,
                    ),
                    self.on_reload.clone(),
                )
            })
            .child(address)
            .when(self.zoom != 100, |this| {
                // `.t3-zoom { padding: 0 spacing-03; font-size: 0.75rem; color: text-secondary }`.
                this.child(
                    sans(div(), 12., 400)
                        .flex_none()
                        .px(rem(8.))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(SharedString::from(format!("{}%", self.zoom))),
                )
            })
            .child(div().flex_none().child(menu))
            .child(click(
                icon_button(
                    (id.clone(), "external"),
                    self.texts.open_external.clone(),
                    CarbonIcon::Launch,
                ),
                self.on_open_external.clone(),
            ))
            .child(click(
                icon_button(
                    (id.clone(), "more"),
                    self.texts.more.clone(),
                    CarbonIcon::OverflowMenuVertical,
                ),
                self.on_more.clone(),
            ))
    }
}

// ── PreviewEmptyState ───────────────────────────────────────────────────

/// A local dev server (`servers[]`: `{ port, name }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalServer {
    /// `port`.
    pub port: u16,
    /// `name`.
    pub name: SharedString,
}

impl LocalServer {
    /// A server on `port`.
    pub fn new(port: u16, name: impl Into<SharedString>) -> Self {
        Self {
            port,
            name: name.into(),
        }
    }
}

/// Texts of a [`PreviewEmptyState`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct PreviewEmptyTexts {
    /// `No preview yet`.
    pub title: SharedString,
    /// The explanation.
    pub body: SharedString,
    /// `Local servers`.
    pub servers: SharedString,
    /// `Recently used`.
    pub recent: SharedString,
}

impl Default for PreviewEmptyTexts {
    fn default() -> Self {
        Self {
            title: "No preview yet".into(),
            body: "Type a URL above, or run a dev script. Local servers this thread starts appear here."
                .into(),
            servers: "Local servers".into(),
            recent: "Recently used".into(),
        }
    }
}

/// The browser's empty state (`PreviewEmptyState`, `.t3-pempty`): a globe,
/// a heading and note, then cards for local servers (status dot, `:port`,
/// name) and recently used addresses.
#[derive(IntoElement)]
pub struct PreviewEmptyState {
    id: ElementId,
    servers: Vec<LocalServer>,
    recent: Vec<SharedString>,
    texts: PreviewEmptyTexts,
    on_server: Option<IndexHandler>,
    on_recent: Option<IndexHandler>,
}

impl PreviewEmptyState {
    /// An empty state.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            servers: Vec::new(),
            recent: Vec::new(),
            texts: PreviewEmptyTexts::default(),
            on_server: None,
            on_recent: None,
        }
    }

    /// Local servers (`servers`).
    pub fn servers(mut self, servers: impl IntoIterator<Item = LocalServer>) -> Self {
        self.servers = servers.into_iter().collect();
        self
    }

    /// Recent addresses (`recent`).
    pub fn recent<S: Into<SharedString>>(mut self, recent: impl IntoIterator<Item = S>) -> Self {
        self.recent = recent.into_iter().map(Into::into).collect();
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: PreviewEmptyTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A server card pressed, with its index.
    pub fn on_server(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_server = Some(Rc::new(f));
        self
    }

    /// A recent card pressed, with its index.
    pub fn on_recent(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_recent = Some(Rc::new(f));
        self
    }
}

/// `.t3-srvcard`: a 40px `layer-01` button card.
fn server_card(
    id: ElementId,
    label: SharedString,
    children: Vec<AnyElement>,
    on_click: Option<Handler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let (hovered, store) = press::hover((id.clone(), "hover"), window, cx);
    let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
    // `.t3-srvcard { display: flex; align-items: center; gap: spacing-03; height: size-md;
    // padding: 0 spacing-04; background: layer-01; font-size: 0.875rem }`,
    // `:hover { background: layer-hover-01 }`.
    sans(
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .accessibility_label(label),
        14.,
        400,
    )
    .relative()
    .flex()
    .flex_row()
    .items_center()
    .gap(rem(8.))
    .h(rem(40.))
    .px(rem(12.))
    .bg(color(if hovered {
        ColorRole::LayerHover01
    } else {
        ColorRole::Layer01
    }))
    .text_color(color(ColorRole::TextPrimary))
    .cursor(CursorStyle::PointingHand)
    .on_hover(press::track_hover(store))
    .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
    .children(children)
    .when(ring, |this| this.child(press::ring()))
    .into_any_element()
}

impl RenderOnce for PreviewEmptyState {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let secondary = color(ColorRole::TextSecondary);
        // `.t3-pempty-h { margin-top: spacing-05; font-size: 0.75rem; color: text-secondary }`.
        let heading = |text: SharedString| {
            sans(div(), 12., 400)
                .mt(rem(16.))
                .text_color(secondary)
                .child(text)
        };
        // `.t3-pempty-grid { display: flex; flex-wrap: wrap; gap: spacing-03 }`.
        let grid = || div().flex().flex_row().flex_wrap().gap(rem(8.));
        let span = |text: SharedString| div().text_color(secondary).child(text);
        let servers = self
            .servers
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let f = self.on_server.clone();
                server_card(
                    child_id(&id, "server", i),
                    SharedString::from(format!(":{} {}", s.port, s.name)),
                    vec![
                        ConnectionStatusDot::new(ConnectionStatus::Connected)
                            .id(child_id(&id, "server-dot", i))
                            .into_any_element(),
                        // A bare `<b>` (Carbon's `b { font: inherit }`): 400.
                        sans(div(), 14., 400)
                            .child(SharedString::from(format!(":{}", s.port)))
                            .into_any_element(),
                        span(s.name.clone()).into_any_element(),
                    ],
                    f.map(|f| Rc::new(move |w: &mut Window, cx: &mut App| f(i, w, cx)) as Handler),
                    window,
                    cx,
                )
            })
            .collect::<Vec<_>>();
        let recent = self
            .recent
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let f = self.on_recent.clone();
                server_card(
                    child_id(&id, "recent", i),
                    r.clone(),
                    vec![
                        press::icon(CarbonIcon::Time, 14., color(ColorRole::IconSecondary))
                            .flex_none()
                            .into_any_element(),
                        span(r.clone()).into_any_element(),
                    ],
                    f.map(|f| Rc::new(move |w: &mut Window, cx: &mut App| f(i, w, cx)) as Handler),
                    window,
                    cx,
                )
            })
            .collect::<Vec<_>>();
        // `.t3-pempty { display: flex; flex-direction: column; align-items: flex-start;
        // gap: spacing-03; max-width: 32rem; padding: spacing-07 spacing-05 }`.
        sans(div(), 14., 400)
            .id(id)
            .flex()
            .flex_col()
            .items_start()
            .gap(rem(8.))
            .max_w(rem(512.))
            .py(rem(32.))
            .px(rem(16.))
            .text_color(color(ColorRole::TextPrimary))
            .child(press::icon(
                CarbonIcon::Earth,
                32.,
                color(ColorRole::IconSecondary),
            ))
            // `h3 { margin: 0; font-size: 1.25rem; font-weight: 400 }`.
            .child(sans(div(), 20., 400).child(self.texts.title.clone()))
            // `p { margin: 0; font-size: 0.875rem; color: text-secondary }`; the
            // line height is Carbon's `p { line-height: var(--body-02-line-height, 1.5) }`
            // (`bundle.css`): 21px.
            .child(
                sans(div(), 14., 400)
                    .line_height(relative(1.5))
                    .text_color(secondary)
                    .child(self.texts.body.clone()),
            )
            .when(!servers.is_empty(), |this| {
                this.child(heading(self.texts.servers.clone()))
                    .child(grid().children(servers))
            })
            .when(!recent.is_empty(), |this| {
                this.child(heading(self.texts.recent.clone()))
                    .child(grid().children(recent))
            })
    }
}

// ── MiniPlayer ──────────────────────────────────────────────────────────

/// `MiniPlayer` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MiniPlayerKind {
    /// `browser` (the default): a wireframe page and a globe.
    #[default]
    Browser,
    /// `device`: a phone outline.
    Device,
}

impl MiniPlayerKind {
    /// Both kinds.
    pub const ALL: [MiniPlayerKind; 2] = [Self::Browser, Self::Device];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Device => "device",
        }
    }
}

/// Texts of a [`MiniPlayer`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct MiniPlayerTexts {
    /// `Expand`.
    pub expand: SharedString,
    /// `Close`.
    pub close: SharedString,
}

impl Default for MiniPlayerTexts {
    fn default() -> Self {
        Self {
            expand: "Expand".into(),
            close: "Close".into(),
        }
    }
}

/// The floating preview kept while you switch threads (`MiniPlayer`,
/// `.t3-mini`): an 18rem `layer-01` card with the floating shadow, a 9rem
/// screen and a 40px bar with title, subtitle, Expand and Close.
#[derive(IntoElement)]
pub struct MiniPlayer {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    kind: MiniPlayerKind,
    texts: MiniPlayerTexts,
    on_expand: Option<Handler>,
    on_close: Option<Handler>,
}

impl MiniPlayer {
    /// A mini player titled `title` (`Preview` in the reference).
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            kind: MiniPlayerKind::Browser,
            texts: MiniPlayerTexts::default(),
            on_expand: None,
            on_close: None,
        }
    }

    /// The subtitle (`subtitle`).
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// The kind (`kind`).
    pub fn kind(mut self, kind: MiniPlayerKind) -> Self {
        self.kind = kind;
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: MiniPlayerTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Expand pressed.
    pub fn on_expand(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_expand = Some(Rc::new(f));
        self
    }

    /// Close pressed.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for MiniPlayer {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let screen = match self.kind {
            // `.t3-mini-phone { width: 3.5rem; height: 7rem; border: 3px solid icon-primary;
            // border-radius: 10px; background: layer-01 }`.
            MiniPlayerKind::Device => div()
                .w(rem(56.))
                .h(rem(112.))
                .border(px(3.))
                .border_color(color(ColorRole::IconPrimary))
                .rounded(px(10.))
                .bg(color(ColorRole::Layer01))
                .into_any_element(),
            // `.t3-mini-page { display: flex; flex-direction: column; gap: 6px; width: 70% }`,
            // `i { height: 10px; background: layer-01 }`, `i:first-child { width: 40%;
            // height: 14px; background: interactive }`.
            MiniPlayerKind::Browser => div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .w(relative(0.7))
                .child(
                    div()
                        .w(relative(0.4))
                        .h(px(14.))
                        .bg(color(ColorRole::Interactive)),
                )
                .child(div().h(px(10.)).bg(color(ColorRole::Layer01)))
                .child(div().h(px(10.)).bg(color(ColorRole::Layer01)))
                .into_any_element(),
        };
        let glyph = match self.kind {
            MiniPlayerKind::Device => CarbonIcon::Mobile,
            MiniPlayerKind::Browser => CarbonIcon::Earth,
        };
        let on_expand = self.on_expand.clone();
        let on_close = self.on_close.clone();
        // `.t3-mini-bar > span { min-width: 0; white-space: nowrap; overflow: hidden;
        // text-overflow: ellipsis }`.
        let clip = |el: gpui_kit::Div| {
            el.min_w_0()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
        };
        // `.t3-mini { width: 18rem; max-width: 100%; background: layer-01;
        // box-shadow: 0 2px 6px var(--shadow); border-radius: 0 }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w(rem(288.))
            .max_w_full()
            .bg(color(ColorRole::Layer01))
            .shadow(crate::components::overlay::floating_shadow())
            .child(
                // `.t3-mini-screen { display: flex; height: 9rem; align-items: center;
                // justify-content: center; background: layer-accent-01 }`.
                div()
                    .flex()
                    .h(rem(144.))
                    .items_center()
                    .justify_center()
                    .bg(color(ColorRole::LayerAccent01))
                    .child(screen),
            )
            .child(
                // `.t3-mini-bar { display: flex; align-items: center; gap: spacing-03;
                // height: size-md; padding-left: spacing-04; font-size: 0.875rem }`.
                sans(div(), 14., 400)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(rem(8.))
                    .h(rem(40.))
                    .pl(rem(12.))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(press::icon(glyph, 14., color(ColorRole::IconSecondary)).flex_none())
                    .child(clip(sans(div(), 14., 600)).child(self.title.clone()))
                    .when_some(self.subtitle.clone(), |this, sub| {
                        this.child(clip(card_sub(sub)))
                    })
                    .child(div().flex_1())
                    .child(click(
                        icon_button(
                            (id.clone(), "expand"),
                            self.texts.expand.clone(),
                            CarbonIcon::Maximize,
                        ),
                        on_expand,
                    ))
                    .child(click(
                        icon_button(
                            (id.clone(), "close"),
                            self.texts.close.clone(),
                            CarbonIcon::Close,
                        ),
                        on_close,
                    )),
            )
    }
}

// ── DeviceControlsRail ──────────────────────────────────────────────────

/// One rail action: a label (its tooltip) and a glyph.
#[derive(Clone)]
pub struct DeviceAction {
    /// The label.
    pub label: SharedString,
    /// The glyph.
    pub icon: Icon,
}

impl DeviceAction {
    /// An action; known labels get the reference glyph (`Home`, `Rotate`,
    /// `Screenshot` → Image, `Record` → Recording, `Shake` → Repeat,
    /// `Volume` → Notification, `Keyboard`), others `Settings`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        let icon = Icon::carbon(device_glyph(&label));
        Self { label, icon }
    }

    /// Replace the glyph.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }
}

/// The glyph for a reference action label (`ic[a] || I.Settings`).
pub fn device_glyph(label: &str) -> CarbonIcon {
    match label {
        "Home" => CarbonIcon::Home,
        "Rotate" => CarbonIcon::Rotate,
        "Screenshot" => CarbonIcon::Image,
        "Record" => CarbonIcon::Recording,
        "Shake" => CarbonIcon::Repeat,
        "Volume" => CarbonIcon::Notification,
        "Keyboard" => CarbonIcon::Keyboard,
        _ => CarbonIcon::Settings,
    }
}

/// The reference's default actions.
pub const DEVICE_ACTIONS: [&str; 6] = ["Home", "Rotate", "Screenshot", "Record", "Shake", "Volume"];

/// A vertical device toolbar (`DeviceControlsRail`, `.t3-rail`): ghost md
/// icon buttons with their tooltips on the left, a divider before the
/// fourth, on `layer-01`.
#[derive(IntoElement)]
pub struct DeviceControlsRail {
    id: ElementId,
    actions: Vec<DeviceAction>,
    on_action: Option<IndexHandler>,
}

impl DeviceControlsRail {
    /// The rail with the reference's six actions.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: DEVICE_ACTIONS
                .iter()
                .map(|a| DeviceAction::new(*a))
                .collect(),
            on_action: None,
        }
    }

    /// Replace the actions (`actions`).
    pub fn actions(mut self, actions: impl IntoIterator<Item = DeviceAction>) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }

    /// An action pressed, with its index.
    pub fn on_action(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for DeviceControlsRail {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let mut children: Vec<AnyElement> = Vec::new();
        for (i, a) in self.actions.into_iter().enumerate() {
            if i == 3 {
                // `.t3-rail-div { height: 1px; margin: spacing-02 spacing-03; background: border-subtle-01 }`.
                children.push(
                    div()
                        .h(px(1.))
                        .my(rem(4.))
                        .mx(rem(8.))
                        .bg(color(ColorRole::BorderSubtle01))
                        .into_any_element(),
                );
            }
            let f = self.on_action.clone();
            children.push(
                Button::new(child_id(&id, "action", i), a.label)
                    .icon(a.icon)
                    .icon_only(true)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Md)
                    .tooltip_align(Align::Left)
                    .when_some(f, |b, f| b.on_click(move |_, w, cx| f(i, w, cx)))
                    .into_any_element(),
            );
        }
        // `.t3-rail { display: inline-flex; flex-direction: column; background: layer-01 }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .flex_none()
            .bg(color(ColorRole::Layer01))
            .children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_bar_slides_from_minus_40_to_100_percent() {
        assert!((load_left(0.) + 0.4).abs() < 1e-5);
        assert!((load_left(1.) - 1.).abs() < 1e-5);
        // `ease-in-out` is symmetric: halfway is the midpoint.
        assert!((load_left(0.5) - 0.3).abs() < 1e-3);
    }

    #[test]
    fn rail_glyphs_follow_the_reference_map() {
        assert_eq!(device_glyph("Screenshot"), CarbonIcon::Image);
        assert_eq!(device_glyph("Shake"), CarbonIcon::Repeat);
        assert_eq!(device_glyph("Volume"), CarbonIcon::Notification);
        assert_eq!(device_glyph("Anything"), CarbonIcon::Settings);
    }
}
