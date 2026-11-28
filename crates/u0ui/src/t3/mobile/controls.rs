//! T3 · Mobile controls: [`SplitButton`], [`FloatingActionButton`],
//! [`SwipeRow`], [`ControlPill`], [`MobileNotice`], [`LoadingStrip`] and
//! [`FloatingWorkingControl`].
use super::{Handler, float_shadow, inset_frame};
use crate::{
    components::{ButtonSize, ButtonVariant, MenuItem},
    density,
    icons::{CarbonIcon, Icon},
    motion::{self, Curve, Speed},
    t3::{
        base::{
            REDUCED_PULSE_PERIOD, Spin, endless_phase, fmt_elapsed, reduced_pulse_opacity,
            use_ticker,
        },
        craft::{self, focus_handle, focus_visible, press, rem, sans, sub},
        shell::IconMenu,
    },
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, AppContext as _, DragMoveEvent, ElementId, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledText, TextRun, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;
use std::time::Duration;

type RevealHandler = Rc<dyn Fn(Revealed, &mut Window, &mut App)>;

// ── SplitButton ──────────────────────────────────────────────────────────

/// The split button's kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SplitKind {
    /// `primary`, the default.
    #[default]
    Primary,
    /// `secondary`.
    Secondary,
}

impl SplitKind {
    /// Both kinds.
    pub const ALL: [SplitKind; 2] = [Self::Primary, Self::Secondary];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }

    /// The main half's fill: `button-primary` (hover `button-primary-hover`)
    /// or `button-secondary` (the secondary rule comes later in the cascade,
    /// so it keeps its fill on hover).
    pub fn fill(self, hovered: bool) -> Hsla {
        color(match (self, hovered) {
            (Self::Primary, false) => ColorRole::ButtonPrimary,
            (Self::Primary, true) => ColorRole::ButtonPrimaryHover,
            (Self::Secondary, _) => ColorRole::ButtonSecondary,
        })
    }
}

/// A split button (`.t3-split`): a main action and, 1px apart, an
/// [`IconMenu`] chevron with related actions; both halves share the md
/// field height of the density.
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    label: SharedString,
    kind: SplitKind,
    menu: Vec<MenuItem>,
    menu_label: SharedString,
    on_press: Option<Handler>,
}

impl SplitButton {
    /// A split button labelled `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: SplitKind::Primary,
            menu: Vec::new(),
            menu_label: "More options".into(),
            on_press: None,
        }
    }

    /// The kind.
    pub fn kind(mut self, kind: SplitKind) -> Self {
        self.kind = kind;
        self
    }

    /// The menu items.
    pub fn menu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.menu = items.into_iter().collect();
        self
    }

    /// The chevron's name (`More options`).
    pub fn menu_label(mut self, label: impl Into<SharedString>) -> Self {
        self.menu_label = label.into();
        self
    }

    /// Main action pressed.
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let main_id: ElementId = (id.clone(), "main").into();
        let focus = focus_handle(&main_id, window, cx);
        let ring = focus_visible(&focus, window);
        let height = ButtonSize::Md.height(density::current());
        let kind = self.kind;
        let on_press = self.on_press.clone();
        let main = sans(press(main_id, &focus), 14., 400)
            .h(rem(height))
            .px(rem(16.))
            .bg(kind.fill(false))
            .hover(move |s| s.bg(kind.fill(true)))
            .text_color(color(ColorRole::TextOnColor))
            .when_some(on_press, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(self.label.clone())
            .when(ring, |b| b.child(craft::focus_ring()));
        div().flex().flex_none().gap(px(1.)).child(main).child(
            IconMenu::new((id, "menu"), self.menu_label.clone())
                .variant(match kind {
                    SplitKind::Primary => ButtonVariant::Primary,
                    SplitKind::Secondary => ButtonVariant::Secondary,
                })
                .size(ButtonSize::Md)
                .items(self.menu),
        )
    }
}

// ── FloatingActionButton ─────────────────────────────────────────────────

/// The floating primary action (`.t3-fab`): a 56px `button-primary` square
/// with the floating shadow; `extended` shows the label beside the icon.
#[derive(IntoElement)]
pub struct FloatingActionButton {
    id: ElementId,
    icon: CarbonIcon,
    label: SharedString,
    extended: bool,
    on_press: Option<Handler>,
}

impl FloatingActionButton {
    /// A FAB named `label` (`label`, default `New thread` in the reference).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: CarbonIcon::Edit,
            label: label.into(),
            extended: false,
            on_press: None,
        }
    }

    /// The icon (default `Edit`).
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Show the label.
    pub fn extended(mut self, extended: bool) -> Self {
        self.extended = extended;
        self
    }

    /// Pressed.
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for FloatingActionButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = focus_handle(&self.id, window, cx);
        let ring = focus_visible(&focus, window);
        let ink = color(ColorRole::TextOnColor);
        let on_press = self.on_press.clone();
        sans(press(self.id.clone(), &focus), 14., 600)
            .accessibility_label(self.label.clone())
            .flex_none()
            .justify_center()
            .gap(rem(8.))
            .h(rem(56.))
            .map(|b| {
                if self.extended {
                    b.px(rem(24.))
                } else {
                    b.w(rem(56.))
                }
            })
            .bg(color(ColorRole::ButtonPrimary))
            .text_color(ink)
            .shadow(float_shadow(2.))
            .when_some(on_press, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(Icon::carbon(self.icon).size_length(rem(20.)).color(ink))
            .when(self.extended, |b| b.child(self.label.clone()))
            .when(ring, |b| b.child(craft::focus_ring()))
    }
}

// ── SwipeRow ─────────────────────────────────────────────────────────────

/// A swipe action's tone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SwipeTone {
    /// `blue`: `button-primary`.
    #[default]
    Blue,
    /// `purple`: `charts-1`.
    Purple,
    /// `gray`: `button-secondary`.
    Gray,
    /// `red`: `button-danger-primary`.
    Red,
    /// `green`: `charts-7`.
    Green,
}

impl SwipeTone {
    /// The action's fill (`.t3-swipe-act--<tone>`).
    pub fn fill(self) -> Hsla {
        color(match self {
            Self::Blue => ColorRole::ButtonPrimary,
            Self::Purple => ColorRole::Charts1,
            Self::Gray => ColorRole::ButtonSecondary,
            Self::Red => ColorRole::ButtonDangerPrimary,
            Self::Green => ColorRole::Charts7,
        })
    }
}

/// One swipe action (`leading[]` / `trailing[]`).
#[derive(Clone)]
pub struct SwipeAction {
    /// `label`.
    pub label: SharedString,
    /// `icon`.
    pub icon: CarbonIcon,
    /// `tone`.
    pub tone: SwipeTone,
    on_press: Option<Handler>,
}

impl SwipeAction {
    /// An action.
    pub fn new(label: impl Into<SharedString>, icon: CarbonIcon, tone: SwipeTone) -> Self {
        Self {
            label: label.into(),
            icon,
            tone,
            on_press: None,
        }
    }

    /// Pressed.
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
}

/// Which side is revealed (`revealed`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Revealed {
    /// `none`, the default.
    #[default]
    None,
    /// `leading`: the content slides 4.5rem right.
    Leading,
    /// `trailing`: the content slides 9rem left.
    Trailing,
}

impl Revealed {
    /// Every value.
    pub const ALL: [Revealed; 3] = [Self::None, Self::Leading, Self::Trailing];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Leading => "leading",
            Self::Trailing => "trailing",
        }
    }

    /// The content offset in px: `.t3-swipe--leading .t3-swipe-content {
    /// transform: translateX(4.5rem) }`, `--trailing` `translateX(-9rem)`.
    pub const fn offset(self) -> f32 {
        match self {
            Self::None => 0.,
            Self::Leading => 72.,
            Self::Trailing => -144.,
        }
    }

    /// Where a drag released at `offset` settles: past half of a side's
    /// travel (and only toward a side that has actions) it opens.
    pub fn settle(offset: f32, leading: bool, trailing: bool) -> Self {
        if leading && offset > Self::Leading.offset() / 2. {
            Self::Leading
        } else if trailing && offset < Self::Trailing.offset() / 2. {
            Self::Trailing
        } else {
            Self::None
        }
    }
}

#[derive(Default)]
struct SwipeState {
    revealed: Revealed,
    /// Set when `revealed` came from the caller; a new caller value resets.
    seeded: Option<Revealed>,
    drag: Option<(f32, f32)>,
    /// The offset painted last frame.
    shown: f32,
    /// A running settle: (from, to, start).
    anim: Option<(f32, f32, std::time::Instant)>,
}

#[derive(Clone)]
struct SwipeDrag(ElementId);

impl gpui_kit::Render for SwipeDrag {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        div()
    }
}

/// A list row that slides to reveal actions (`.t3-swipe`). `revealed` sets
/// the resting state; dragging the content sideways follows the pointer
/// (clamped to the reveal travel) and settles open past half the travel,
/// with the reference's 240ms productive slide (cut under reduced motion).
#[derive(IntoElement)]
pub struct SwipeRow {
    id: ElementId,
    leading: Vec<SwipeAction>,
    trailing: Vec<SwipeAction>,
    revealed: Revealed,
    on_revealed: Option<RevealHandler>,
    content: Option<AnyElement>,
}

impl SwipeRow {
    /// A swipe row.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            leading: Vec::new(),
            trailing: Vec::new(),
            revealed: Revealed::None,
            on_revealed: None,
            content: None,
        }
    }

    /// Leading actions (revealed by swiping right).
    pub fn leading(mut self, actions: impl IntoIterator<Item = SwipeAction>) -> Self {
        self.leading = actions.into_iter().collect();
        self
    }

    /// Trailing actions (revealed by swiping left).
    pub fn trailing(mut self, actions: impl IntoIterator<Item = SwipeAction>) -> Self {
        self.trailing = actions.into_iter().collect();
        self
    }

    /// The revealed side.
    pub fn revealed(mut self, revealed: Revealed) -> Self {
        self.revealed = revealed;
        self
    }

    /// A drag settled on a side.
    pub fn on_revealed(mut self, f: impl Fn(Revealed, &mut Window, &mut App) + 'static) -> Self {
        self.on_revealed = Some(Rc::new(f));
        self
    }

    /// The row content.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl RenderOnce for SwipeRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = window.use_keyed_state((id.clone(), "swipe"), cx, |_, _| SwipeState::default());
        if state.read(cx).seeded != Some(self.revealed) {
            let r = self.revealed;
            state.update(cx, |s, _| {
                s.seeded = Some(r);
                s.revealed = r;
            });
        }
        let (revealed, drag) = {
            let s = state.read(cx);
            (s.revealed, s.drag)
        };
        let has_lead = !self.leading.is_empty();
        let has_trail = !self.trailing.is_empty();
        let clamp = |x: f32| {
            x.clamp(
                if has_trail {
                    Revealed::Trailing.offset()
                } else {
                    0.
                },
                if has_lead {
                    Revealed::Leading.offset()
                } else {
                    0.
                },
            )
        };
        let target = match drag {
            Some((start, now)) => clamp(revealed.offset() + now - start),
            None => revealed.offset(),
        };
        // `.t3-swipe-content { transition: transform 240ms cubic-bezier(0.2,0,0.38,0.9) }`:
        // settles from wherever the content is (the drag position included);
        // while dragging it follows the pointer; a cut under reduced motion.
        let reduced = motion::reduced_motion(cx);
        let offset = state.update(cx, |s, _| {
            if drag.is_some() {
                s.anim = None;
            } else if s.shown != target && s.anim.map(|a| a.1) != Some(target) {
                s.anim = Some((s.shown, target, std::time::Instant::now()));
            }
            match s.anim {
                Some((from, to, since)) if !reduced => {
                    let p =
                        since.elapsed().as_secs_f32() / Speed::Moderate02.duration().as_secs_f32();
                    if p >= 1. {
                        s.anim = None;
                        s.shown = to;
                    } else {
                        s.shown = from + (to - from) * Curve::ProductiveStandard.ease(p);
                    }
                }
                _ => {
                    s.anim = None;
                    s.shown = target;
                }
            }
            s.shown
        });
        if state.read(cx).anim.is_some() {
            window.request_animation_frame();
        }
        let action = |side: &str, i: usize, a: &SwipeAction, window: &mut Window, cx: &mut App| {
            let aid = sub(&id, side, i);
            let focus = focus_handle(&aid, window, cx);
            let ring = focus_visible(&focus, window);
            let ink = color(ColorRole::TextOnColor);
            let on_press = a.on_press.clone();
            sans(press(aid, &focus), 12., 400)
                .accessibility_label(a.label.clone())
                .flex_col()
                .justify_center()
                .items_center()
                .gap(px(2.))
                .w(rem(72.))
                .h_full()
                .bg(a.tone.fill())
                .text_color(ink)
                .when_some(on_press, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
                .child(Icon::carbon(a.icon).size_length(rem(20.)).color(ink))
                .child(a.label.clone())
                .when(ring, |b| b.child(craft::focus_ring()))
                .into_any_element()
        };
        let lead: Vec<AnyElement> = self
            .leading
            .iter()
            .enumerate()
            .map(|(i, a)| action("lead", i, a, window, cx))
            .collect();
        let trail: Vec<AnyElement> = self
            .trailing
            .iter()
            .enumerate()
            .map(|(i, a)| action("trail", i, a, window, cx))
            .collect();
        let drag_id = id.clone();
        let on_revealed = self.on_revealed.clone();
        div()
            .id(id.clone())
            .relative()
            .overflow_hidden()
            .bg(color(ColorRole::Layer01))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .flex()
                    .children(lead),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .flex()
                    .children(trail),
            )
            .child(
                div()
                    .id((id.clone(), "content"))
                    .relative()
                    .left(px(offset))
                    .bg(color(ColorRole::Layer01))
                    .on_drag(SwipeDrag(drag_id.clone()), |drag, _, _, cx| {
                        cx.new(|_| drag.clone())
                    })
                    .on_drag_move::<SwipeDrag>({
                        let state = state.clone();
                        let drag_id = drag_id.clone();
                        move |e: &DragMoveEvent<SwipeDrag>, _, cx| {
                            if e.drag(cx).0 != drag_id {
                                return;
                            }
                            let x = f32::from(e.event.position.x);
                            state.update(cx, |s, cx| {
                                let start = s.drag.map(|d| d.0).unwrap_or(x);
                                s.drag = Some((start, x));
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_up(MouseButton::Left, {
                        let state = state.clone();
                        let on_revealed = on_revealed.clone();
                        move |_, window, cx| {
                            settle(&state, has_lead, has_trail, &on_revealed, window, cx)
                        }
                    })
                    .on_mouse_up_out(MouseButton::Left, {
                        let state = state.clone();
                        move |_, window, cx| {
                            settle(&state, has_lead, has_trail, &on_revealed, window, cx)
                        }
                    })
                    .children(self.content),
            )
    }
}

fn settle(
    state: &gpui_kit::Entity<SwipeState>,
    lead: bool,
    trail: bool,
    on_revealed: &Option<RevealHandler>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some((start, now)) = state.read(cx).drag else {
        return;
    };
    let base = state.read(cx).revealed.offset();
    let next = Revealed::settle(base + now - start, lead, trail);
    state.update(cx, |s, cx| {
        s.drag = None;
        s.revealed = next;
        cx.notify();
    });
    if let Some(f) = on_revealed {
        f(next, window, cx);
    }
}

// ── ControlPill ──────────────────────────────────────────────────────────

/// A composer toolbar picker pill (`.t3-cpill`, 32px, square per
/// `gemini-MA`): icon, label and a 12px chevron on `layer-accent-01`;
/// active = blue tag pair. It opens a menu (`aria-haspopup="menu"`).
#[derive(IntoElement)]
pub struct ControlPill {
    id: ElementId,
    icon: Option<CarbonIcon>,
    label: SharedString,
    active: bool,
    on_press: Option<Handler>,
}

impl ControlPill {
    /// A pill labelled `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: label.into(),
            active: false,
            on_press: None,
        }
    }

    /// A leading 16px icon.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Active (a non-default choice).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Pressed.
    pub fn on_press(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }
}

/// (fill, ink) of a control pill.
pub fn pill_look(active: bool) -> (Hsla, Hsla) {
    if active {
        (
            color(ColorRole::TagBackgroundBlue),
            color(ColorRole::TagColorBlue),
        )
    } else {
        (
            color(ColorRole::LayerAccent01),
            color(ColorRole::TextPrimary),
        )
    }
}

impl RenderOnce for ControlPill {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = focus_handle(&self.id, window, cx);
        let ring = focus_visible(&focus, window);
        let (fill, ink) = pill_look(self.active);
        let on_press = self.on_press.clone();
        sans(press(self.id.clone(), &focus), 13., 400)
            .flex_none()
            .gap(rem(4.))
            .h(rem(32.))
            .px(rem(12.))
            .bg(fill)
            .text_color(ink)
            .whitespace_nowrap()
            .when_some(on_press, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .when_some(self.icon, |b, icon| {
                b.child(Icon::carbon(icon).size_length(rem(16.)).color(ink))
            })
            .child(self.label.clone())
            .child(
                Icon::carbon(CarbonIcon::ChevronDown)
                    .size_length(rem(12.))
                    .color(ink),
            )
            .when(ring, |b| b.child(craft::focus_ring()))
    }
}

// ── MobileNotice / LoadingStrip ──────────────────────────────────────────

/// A notice tone (`tone`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NoticeTone {
    /// `info`, the default.
    #[default]
    Info,
    /// `warning`.
    Warning,
    /// `error`.
    Error,
    /// `success`.
    Success,
}

impl NoticeTone {
    /// Every tone.
    pub const ALL: [NoticeTone; 4] = [Self::Info, Self::Warning, Self::Error, Self::Success];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Success => "success",
        }
    }

    /// (background, icon fill, icon): `notification-background-<tone>` and
    /// `support-<tone>`.
    pub fn look(self) -> (Hsla, Hsla, CarbonIcon) {
        use ColorRole::*;
        let (bg, fg, icon) = match self {
            Self::Info => (
                NotificationBackgroundInfo,
                SupportInfo,
                CarbonIcon::InformationFilled,
            ),
            Self::Warning => (
                NotificationBackgroundWarning,
                SupportWarning,
                CarbonIcon::WarningAltFilled,
            ),
            Self::Error => (
                NotificationBackgroundError,
                SupportError,
                CarbonIcon::ErrorFilled,
            ),
            Self::Success => (
                NotificationBackgroundSuccess,
                SupportSuccess,
                CarbonIcon::CheckmarkFilled,
            ),
        };
        (color(bg), color(fg), icon)
    }
}

/// A mobile notice block (`.t3-mnotice`, square per `gemini-MA`): a 20px
/// status icon, a semibold title with an optional secondary line, and an
/// optional trailing action.
#[derive(IntoElement)]
pub struct MobileNotice {
    tone: NoticeTone,
    title: SharedString,
    body: Option<SharedString>,
    action: Option<AnyElement>,
}

impl MobileNotice {
    /// A notice titled `title`.
    pub fn new(tone: NoticeTone, title: impl Into<SharedString>) -> Self {
        Self {
            tone,
            title: title.into(),
            body: None,
            action: None,
        }
    }

    /// The secondary line (`children`).
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// A trailing action.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for MobileNotice {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (bg, fg, icon) = self.tone.look();
        let icon = if self.tone == NoticeTone::Warning {
            // The warning glyph's inner mark is `#161616` on the yellow.
            crate::t3::base::warning_glyph(20.).into_any_element()
        } else {
            Icon::carbon(icon)
                .size_length(rem(20.))
                .color(fg)
                .into_any_element()
        };
        sans(div(), 14., 400)
            .flex()
            .items_start()
            .gap(rem(12.))
            .px(rem(16.))
            .py(rem(12.))
            .bg(bg)
            .text_color(color(ColorRole::TextPrimary))
            .child(div().flex_none().child(icon))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(2.))
                    .child(sans(div(), 14., 600).child(self.title))
                    .when_some(self.body, |d, b| {
                        d.child(craft::text(b, 14., ColorRole::TextSecondary))
                    }),
            )
            .children(self.action)
    }
}

/// The indeterminate bar's left edge as a fraction of the strip at phase
/// `t` (`@keyframes t3-load { from { left: -40% } to { left: 100% } }`,
/// `1.2s ease-in-out`).
pub fn loading_left(t: f32) -> f32 {
    let eased = motion::cubic_bezier_at(0.42, 0., 0.58, 1., t);
    -0.4 + 1.4 * eased
}

/// The 2px indeterminate page-top progress strip (`.t3-lstrip`): a 30%
/// `interactive` bar sweeping over `layer-accent-01` every 1.2s. Under
/// reduced motion the bar rests at the start, still.
#[derive(IntoElement)]
pub struct LoadingStrip {
    id: ElementId,
}

impl LoadingStrip {
    /// A strip.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for LoadingStrip {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduced = motion::reduced_motion(cx);
        let bar = color(ColorRole::Interactive);
        div()
            .relative()
            .w_full()
            .h(px(2.))
            .overflow_hidden()
            .bg(color(ColorRole::LayerAccent01))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .w(relative(0.3))
                    .bg(bar)
                    .map(|bar| {
                        if reduced {
                            // `ghost.css`: `@media (prefers-reduced-motion: reduce) { * {
                            // animation-duration: 1ms !important; animation-iteration-count:
                            // 1 !important } }`: `t3-load` ends at once and the bar rests
                            // at its base box (`left: auto` → 0, width 30%).
                            bar.left_0().into_any_element()
                        } else {
                            bar.with_animation(
                                self.id,
                                Animation::new(Duration::from_millis(1200)).repeat(),
                                |el, t| el.left(relative(loading_left(t))),
                            )
                            .into_any_element()
                        }
                    }),
            )
    }
}

// ── FloatingWorkingControl ───────────────────────────────────────────────

/// The shimmer ink at `x` (0..1 across the label) at phase `t` (0..1 of
/// 2.4s): `.t3-shimmer` paints a 300%-wide gradient (secondary, primary at
/// 50%, secondary) moving `background-position` 100% → −50%, so the bright
/// band's centre crosses from −0.5 to 2.5 label widths with a ±0.3 falloff.
pub fn shimmer_mix(x: f32, t: f32) -> f32 {
    let centre = -0.5 + 3. * t;
    (1. - (x - centre).abs() / 0.3).max(0.)
}

/// `a` blended toward `b` by `t` in sRGB.
fn mix_color(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let (a, b) = (a.to_rgb(), b.to_rgb());
    let m = |x: f32, y: f32| x + (y - x) * t;
    gpui_kit::Rgba {
        r: m(a.r, b.r),
        g: m(a.g, b.g),
        b: m(a.b, b.b),
        a: m(a.a, b.a),
    }
    .into()
}

/// Shimmering status text (`.t3-shimmer`); `text-secondary`, pulsing, under
/// reduced motion.
fn shimmer(id: ElementId, label: SharedString, window: &mut Window, cx: &App) -> AnyElement {
    let secondary = color(ColorRole::TextSecondary);
    if motion::reduced_motion(cx) {
        // `t3.css`: `@media (prefers-reduced-motion: reduce) { .t3-shimmer { color:
        // text-secondary; background: none } }`, and `ghost.css` (later, `!important`):
        // `.t3-shimmer { animation: g-pulse 1.4s ease-in-out infinite }`. GPUI freezes
        // `with_animation` loops under reduced motion, so the pulse runs on
        // `endless_phase`.
        let opacity = reduced_pulse_opacity(endless_phase(REDUCED_PULSE_PERIOD, window));
        return div()
            .text_color(secondary)
            .opacity(opacity)
            .child(label)
            .into_any_element();
    }
    let primary = color(ColorRole::TextPrimary);
    let font = crate::typography::font(crate::tokens::FontFamilyKey::Sans, 400);
    div()
        .with_animation(
            id,
            Animation::new(Duration::from_millis(2400)).repeat(),
            move |el, t| {
                let n = label.chars().count().max(1) as f32;
                let runs: Vec<TextRun> = label
                    .char_indices()
                    .enumerate()
                    .map(|(i, (_, ch))| {
                        let mix = shimmer_mix((i as f32 + 0.5) / n, t);
                        TextRun {
                            len: ch.len_utf8(),
                            font: font.clone(),
                            color: mix_color(secondary, primary, mix),
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        }
                    })
                    .collect();
                el.child(StyledText::new(label.clone()).with_runs(runs))
            },
        )
        .into_any_element()
}

/// `.t3-floatwork { gap: var(--spacing-04) }`: 12px between its parts.
pub const FLOATWORK_GAP: f32 = 12.;

/// The floating "working" control of a mobile thread (`.t3-floatwork`,
/// square per `gemini-MA`): spinner, shimmering label, ticking elapsed
/// time, queued count and a framed Stop.
#[derive(IntoElement)]
pub struct FloatingWorkingControl {
    id: ElementId,
    seconds: u64,
    label: SharedString,
    queued: Option<usize>,
    queued_label: SharedString,
    stop_label: SharedString,
    running: bool,
    on_stop: Option<Handler>,
}

impl FloatingWorkingControl {
    /// A control whose timer starts at `seconds`.
    pub fn new(id: impl Into<ElementId>, seconds: u64) -> Self {
        Self {
            id: id.into(),
            seconds,
            label: "Working".into(),
            queued: None,
            queued_label: "queued".into(),
            stop_label: "Stop".into(),
            running: true,
            on_stop: None,
        }
    }

    /// The status label (`Working`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Queued runs (shown when non-zero) and the word after the count.
    pub fn queued(mut self, queued: usize) -> Self {
        self.queued = Some(queued);
        self
    }

    /// The words of the queued tag and the Stop button's name.
    pub fn texts(mut self, queued: impl Into<SharedString>, stop: impl Into<SharedString>) -> Self {
        self.queued_label = queued.into();
        self.stop_label = stop.into();
        self
    }

    /// Whether the timer ticks (default true).
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// Stop pressed.
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for FloatingWorkingControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let elapsed = use_ticker((id.clone(), "t"), self.seconds, self.running, window, cx);
        let stop_id: ElementId = (id.clone(), "stop").into();
        let focus = focus_handle(&stop_id, window, cx);
        let ring = focus_visible(&focus, window);
        let on_stop = self.on_stop.clone();
        // `display: inline-flex`: hugs its content.
        sans(div(), 14., 400)
            .id(id.clone())
            .relative()
            .flex()
            .flex_none()
            .self_start()
            .items_center()
            // `.t3-floatwork { gap: spacing-04 }` (gemini-MA, `t3.css:991`).
            .gap(rem(FLOATWORK_GAP))
            .h(rem(44.))
            .pl(rem(16.))
            .pr(rem(4.))
            .bg(color(ColorRole::Layer02))
            .shadow(float_shadow(2.))
            .child(inset_frame(color(ColorRole::BorderSubtle01)))
            .child(Spin::new((id.clone(), "spin")))
            .child(shimmer(
                (id.clone(), "shimmer").into(),
                self.label.clone(),
                window,
                cx,
            ))
            // `.t3-working-time { tabular-nums; text-helper; 12px }`.
            .child(craft::text(
                fmt_elapsed(elapsed),
                12.,
                ColorRole::TextHelper,
            ))
            .when_some(self.queued.filter(|n| *n > 0), |d, n| {
                d.child(
                    craft::text(
                        format!("{n} {}", self.queued_label),
                        12.,
                        ColorRole::TextPrimary,
                    )
                    .px(rem(8.))
                    .bg(color(ColorRole::LayerAccent02)),
                )
            })
            .child(
                // `.t3-floatwork-stop` (gemini-MA): 36px framed square, `icon-primary`.
                press(stop_id, &focus)
                    .accessibility_label(self.stop_label.clone())
                    .flex_none()
                    .justify_center()
                    .size(rem(36.))
                    .ml(rem(4.))
                    .when_some(on_stop, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
                    .child(
                        Icon::carbon(CarbonIcon::StopFilled)
                            .size_length(rem(16.))
                            .color(color(ColorRole::IconPrimary)),
                    )
                    .child(inset_frame(color(ColorRole::BorderStrong01)))
                    .when(ring, |b| b.child(craft::focus_ring())),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swipe_offsets_follow_the_css() {
        assert_eq!(Revealed::Leading.offset(), 72.);
        assert_eq!(Revealed::Trailing.offset(), -144.);
        assert_eq!(Revealed::settle(40., true, true), Revealed::Leading);
        assert_eq!(Revealed::settle(30., true, true), Revealed::None);
        assert_eq!(Revealed::settle(-80., true, true), Revealed::Trailing);
        assert_eq!(Revealed::settle(-80., true, false), Revealed::None);
    }

    #[test]
    fn floatwork_gap_follows_the_gemini_pass() {
        // `.t3-floatwork { gap: var(--spacing-04) }` (gemini-MA), not spacing-03.
        assert_eq!(FLOATWORK_GAP, 12.);
    }

    #[test]
    fn loading_and_shimmer_timing() {
        assert!((loading_left(0.) + 0.4).abs() < 1e-4);
        assert!((loading_left(1.) - 1.).abs() < 1e-4);
        assert!((loading_left(0.5) - 0.3).abs() < 1e-3);
        // The band is centred over the label's middle at t = 1/3.
        assert!((shimmer_mix(0.5, 1. / 3.) - 1.).abs() < 1e-4);
        assert_eq!(shimmer_mix(0.5, 0.), 0.);
    }

    #[test]
    fn tone_colors() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        assert_eq!(SwipeTone::Purple.fill(), color(ColorRole::Charts1));
        assert_eq!(
            SplitKind::Secondary.fill(true),
            color(ColorRole::ButtonSecondary)
        );
        assert_eq!(
            SplitKind::Primary.fill(true),
            color(ColorRole::ButtonPrimaryHover)
        );
        assert_eq!(pill_look(true).0, color(ColorRole::TagBackgroundBlue));
        assert_eq!(NoticeTone::Error.look().1, color(ColorRole::SupportError));
    }
}
