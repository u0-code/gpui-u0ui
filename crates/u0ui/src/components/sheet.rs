//! Sheet: a panel that slides in from a window edge (Carbon for IBM Products'
//! side panel on Modal tokens; `.cu-panel` in the reference `custom.css`).
//!
//! - **Behavior** is Kit's: the window-level host is [`gpui_kit::base::Dialog`]
//!   (focus trap, Escape, scrim press, deferred top layer). A `contained`
//!   sheet fills its parent instead (`.cu-panel-overlay--contained`), so it
//!   draws its own overlay with Base's focus trap and handles Escape itself.
//!   Open state, focus moves and motion come from [`super::overlay`].
//! - **Appearance**: `layer-01` panel with the floating shadow on an `overlay`
//!   scrim; heading-03 title, body-01 description in `text-secondary`, a 48px
//!   ghost close button, a scrolling body, and a footer whose buttons split
//!   the width like a Modal footer.
//! - **Motion**: the scrim fades in over 110ms `cubic-bezier(0, 0, 0.38, 0.9)`
//!   (`cu-fade`), the panel slides from its edge over 240ms
//!   `cubic-bezier(0, 0, 0.3, 1)` (`cu-in-r/l/t/b`). The reference unmounts at
//!   once on close; here the exit mirrors the entrance on Carbon's exit curves
//!   at fast-02 (110ms) and plays before unmount. Reduced motion fades the
//!   panel instead of moving it.
use super::overlay::{self, OpenChange, Translate, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_base::FocusTrapElement as _;
use gpui_kit::base as gpui_base;
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, point, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// Panel slide in: `cu-in-r 240ms cubic-bezier(0, 0, 0.3, 1)`.
pub const PANEL_ENTER: Motion = Motion::new(Curve::ExpressiveEntrance, Speed::Moderate02);
/// Panel slide out (not in the reference, which unmounts at once): the
/// expressive exit curve at fast-02.
pub const PANEL_EXIT: Motion = Motion::new(Curve::ExpressiveExit, Speed::Fast02);
/// Scrim in: `.cu-panel-overlay { animation: cu-fade 110ms cubic-bezier(0, 0, 0.38, 0.9) }`.
pub const SCRIM_ENTER: Motion = Motion::new(Curve::ProductiveEntrance, Speed::Fast02);
/// Scrim out: productive exit at fast-02.
pub const SCRIM_EXIT: Motion = Motion::new(Curve::ProductiveExit, Speed::Fast02);
/// `.cu-panel-close` hover wash: `.cds--btn--ghost` 70ms.
const CLOSE_HOVER: Motion = Motion::HOVER;

/// `index.d.ts` `SheetProps.side`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SheetSide {
    /// `right`, the Sheet default.
    #[default]
    Right,
    /// `left`.
    Left,
    /// `top`.
    Top,
    /// `bottom`, the Drawer default.
    Bottom,
}

impl SheetSide {
    /// Every side.
    pub const ALL: [SheetSide; 4] = [Self::Right, Self::Left, Self::Top, Self::Bottom];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Right => "right",
            Self::Left => "left",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }

    /// Where the panel starts its slide, as a fraction of its own size
    /// (`cu-in-r { from { transform: translateX(100%) } }` …).
    pub const fn from(self) -> (f32, f32) {
        match self {
            Self::Right => (1., 0.),
            Self::Left => (-1., 0.),
            Self::Top => (0., -1.),
            Self::Bottom => (0., 1.),
        }
    }

    /// Whether the panel spans the full height (left/right) or width.
    pub const fn vertical(self) -> bool {
        matches!(self, Self::Right | Self::Left)
    }
}

/// Panel geometry in px, from `custom.css`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelGeometry {
    /// `width: min(30rem, 100%)` for left/right sheets, else `None` (100%).
    pub width: Option<f32>,
    /// `max-height` as a fraction of the overlay (top 80%, bottom/drawer 85%).
    pub max_height: Option<f32>,
    /// `.cu-panel-header { padding: spacing-05 spacing-10 spacing-05 spacing-05 }`.
    pub header_padding: (f32, f32, f32, f32),
    /// `.cu-panel-body { padding: 0 spacing-05 spacing-06 }`.
    pub body_padding: (f32, f32, f32),
}

impl PanelGeometry {
    /// Geometry for a panel on `side`.
    pub const fn of(side: SheetSide) -> Self {
        Self {
            width: if side.vertical() { Some(480.) } else { None },
            max_height: match side {
                SheetSide::Top => Some(0.80),
                SheetSide::Bottom => Some(0.85),
                _ => None,
            },
            header_padding: (16., 64., 16., 16.),
            body_padding: (0., 16., 24.),
        }
    }
}

/// The colors a panel paints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelLook {
    /// `.cu-panel-overlay { background: var(--overlay) }`.
    pub scrim: Hsla,
    /// `.cu-panel { background: var(--layer-01) }`.
    pub panel: Hsla,
    /// `color: var(--text-primary)`.
    pub text: Hsla,
    /// `.cu-panel-desc { color: var(--text-secondary) }`.
    pub description: Hsla,
    /// `.cu-drawer-handle { background: var(--border-strong-01) }`.
    pub handle: Hsla,
    /// Ghost close button hover (`.cds--btn--ghost:hover`: background-hover).
    pub close_hover: Hsla,
    /// Close icon (`icon-primary`).
    pub close_icon: Hsla,
}

impl PanelLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        use ColorRole::*;
        Self {
            scrim: color(Overlay),
            panel: color(Layer01),
            text: color(TextPrimary),
            description: color(TextSecondary),
            handle: color(BorderStrong01),
            close_hover: color(BackgroundHover),
            close_icon: color(IconPrimary),
        }
    }
}

/// A sheet. See the module docs. [`super::Drawer`] is the bottom variant with
/// a drag handle.
#[derive(IntoElement)]
pub struct Sheet {
    id: ElementId,
    side: SheetSide,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    trigger: Option<AnyElement>,
    title: Option<SharedString>,
    description: Option<SharedString>,
    footer: Vec<AnyElement>,
    children: Vec<AnyElement>,
    show_close_button: bool,
    contained: bool,
    close_label: SharedString,
    handle: bool,
}

impl Sheet {
    /// A sheet with a stable `id`, on the right edge.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            side: SheetSide::Right,
            open: None,
            default_open: false,
            on_open_change: None,
            trigger: None,
            title: None,
            description: None,
            footer: Vec::new(),
            children: Vec::new(),
            show_close_button: true,
            contained: false,
            // The reference IconButton's `label="Close"`; override per locale.
            close_label: "Close".into(),
            handle: false,
        }
    }

    /// The edge it slides from.
    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    /// Controlled open state.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Initial open state when uncontrolled.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Called whenever the sheet asks to open or close.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// An element that opens the sheet when pressed.
    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    /// The title (`.cu-panel-title`, heading-03).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The description under the title (`.cu-panel-desc`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// A footer element; footer elements share the width equally (pass
    /// `size(Xl).full_width(true)` buttons, as the reference does).
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer.push(element.into_any_element());
        self
    }

    /// Show the close button (default true).
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.show_close_button = show;
        self
    }

    /// Fill the parent (which must be sized) instead of the window.
    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }

    /// Accessible name of the close button.
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// Draw the drawer drag handle (`.cu-drawer-handle`).
    pub fn drag_handle(mut self, handle: bool) -> Self {
        self.handle = handle;
        self
    }

    #[cfg(test)]
    pub(crate) fn side_and_handle(&self) -> (SheetSide, bool) {
        (self.side, self.handle)
    }
}

impl ParentElement for Sheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(
            &id,
            self.open,
            self.default_open,
            self.on_open_change.clone(),
            window,
            cx,
        );
        let open = openness.open;
        let set_open = openness.setter();
        let panel_p = overlay::presence(
            (id.clone(), "panel"),
            open,
            PANEL_ENTER,
            PANEL_EXIT,
            window,
            cx,
        );
        let scrim_p = overlay::presence(
            (id.clone(), "scrim"),
            open,
            SCRIM_ENTER,
            SCRIM_EXIT,
            window,
            cx,
        );
        let close_focus = window
            .use_keyed_state((id.clone(), "close-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focus = overlay::surface_focus(&id, open, None, window, cx);

        let trigger = self.trigger.map(|trigger| {
            let set_open = set_open.clone();
            overlay::trigger_wrapper(
                (id.clone(), "trigger"),
                trigger,
                Rc::new(move |window, cx| set_open(true, window, cx)),
            )
        });
        let visible = open || panel_p > 0. || scrim_p > 0.;
        let root = div()
            .flex()
            .flex_shrink_0()
            .when(self.contained, |this| {
                this.absolute().top_0().left_0().size_full()
            })
            .children(trigger);
        if !visible {
            return root.into_any_element();
        }

        let look = PanelLook::current();
        let geometry = PanelGeometry::of(self.side);
        let reduced = motion::reduced_motion(cx);
        let (fx, fy) = self.side.from();
        let travel = if reduced { 0. } else { 1. - panel_p };

        let close_hover = window.use_keyed_state((id.clone(), "close-hover"), cx, |_, _| false);
        let close_bg = motion::animate_color(
            (id.clone(), "close-bg"),
            if *close_hover.read(cx) {
                look.close_hover
            } else {
                Hsla::transparent_black()
            },
            CLOSE_HOVER,
            window,
            cx,
        );
        let close = self.show_close_button.then(|| {
            let set_open = set_open.clone();
            let focused = close_focus.is_focused(window);
            // `.cu-panel-close`: a ghost lg IconButton (48px) at the top right.
            gpui_kit::base::Button::new((id.clone(), "close"))
                .track_focus(&close_focus)
                .accessibility_label(self.close_label.clone())
                .absolute()
                .top_0()
                .right_0()
                .size(rem(48.))
                .flex()
                .items_center()
                .justify_center()
                .bg(close_bg)
                .border_2()
                .border_color(if focused {
                    color(ColorRole::Focus)
                } else {
                    Hsla::transparent_black()
                })
                .cursor(CursorStyle::PointingHand)
                .on_hover(move |hovered, _, cx| {
                    close_hover.update(cx, |h, cx| {
                        *h = *hovered;
                        cx.notify();
                    })
                })
                .on_click(move |_, window, cx| set_open(false, window, cx))
                .child(
                    Icon::carbon(CarbonIcon::Close)
                        .size(IconSize::S20)
                        .color(look.close_icon),
                )
        });

        let (pt, pr, pb, pl) = geometry.header_padding;
        let header = div()
            .relative()
            .flex_shrink_0()
            .pt(rem(pt))
            .pr(rem(pr))
            .pb(rem(pb))
            .pl(rem(pl))
            .when_some(self.title.clone(), |this, title| {
                this.child(
                    div()
                        .type_style(TypeStyle::Heading03)
                        .text_color(look.text)
                        .child(title),
                )
            })
            .when_some(self.description.clone(), |this, description| {
                this.child(
                    div()
                        .mt(rem(4.))
                        .type_style(TypeStyle::Body01)
                        .text_color(look.description)
                        .child(description),
                )
            })
            .children(close);

        let (bt, bx, bb) = geometry.body_padding;
        let body = div()
            .id((id.clone(), "body"))
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pt(rem(bt))
            .px(rem(bx))
            .pb(rem(bb))
            .type_style(TypeStyle::Body01)
            .text_color(look.text)
            .flex()
            .flex_col()
            .gap(rem(8.))
            .children(self.children);

        let footer = (!self.footer.is_empty()).then(|| {
            div().flex().flex_row().flex_shrink_0().children(
                self.footer
                    .into_iter()
                    .map(|element| div().flex_1().child(element)),
            )
        });

        let panel = div()
            .id((id.clone(), "panel"))
            .flex()
            .flex_col()
            .bg(look.panel)
            .text_color(look.text)
            .shadow(overlay::floating_shadow())
            .occlude()
            .map(|this| match geometry.width {
                Some(width) => this.w(rem(width)).max_w_full().h_full(),
                None => this.w_full(),
            })
            .when(!self.side.vertical(), |this| this.max_h_full())
            .when(self.handle, |this| {
                this.child(
                    div()
                        .flex_shrink_0()
                        .mt(rem(8.))
                        .mx_auto()
                        .w(rem(48.))
                        .h(px(4.))
                        .bg(look.handle),
                )
            })
            .child(header)
            .child(body)
            .children(footer);
        let panel = div()
            .absolute()
            .opacity(if reduced { panel_p } else { 1. })
            .flex()
            .flex_col()
            .map(|this| {
                // Top/bottom panels are capped by `max-height: 80%/85%`: the
                // wrapper spans that band and the panel hugs its edge of it.
                let band = gpui_kit::relative(1. - geometry.max_height.unwrap_or(0.));
                match self.side {
                    SheetSide::Right => this.top_0().bottom_0().right_0(),
                    SheetSide::Left => this.top_0().bottom_0().left_0(),
                    SheetSide::Top => this.top_0().bottom(band).left_0().right_0(),
                    SheetSide::Bottom => this.top(band).bottom_0().left_0().right_0().justify_end(),
                }
            })
            .child(Translate::new(
                panel,
                point(fx * travel, fy * travel),
                point(px(0.), px(0.)),
            ));
        let scrim = div().size_full().bg(look.scrim).opacity(scrim_p);

        if self.contained {
            // `.cu-panel-overlay--contained`: the overlay fills the parent and
            // clips the panel's slide.
            let set_scrim = set_open.clone();
            let set_escape = set_open.clone();
            let overlay = div()
                .id((id.clone(), "overlay"))
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .overflow_hidden()
                .track_focus(&focus)
                .focus_trap((id.clone(), "trap"), &focus)
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && open {
                        set_escape(false, window, cx);
                        cx.stop_propagation();
                    }
                })
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .occlude()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            if open {
                                set_scrim(false, window, cx);
                            }
                        })
                        .child(scrim),
                )
                .child(panel);
            return root.child(overlay).into_any_element();
        }

        let set_cancel = set_open.clone();
        let host = gpui_kit::base::Dialog::new(cx)
            .open(true)
            .focus_handle(focus)
            .when_some(self.title.clone(), |this, title| {
                this.accessibility_label(title)
            })
            .on_ok(|_, _, cx| {
                cx.propagate();
                false
            })
            .on_cancel(move |_, window, cx| {
                if open {
                    set_cancel(false, window, cx);
                }
                false
            })
            .backdrop(scrim)
            .popup(panel);
        root.child(host).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn geometry_follows_cu_panel() {
        // `.cu-panel--sheet.cu-panel--right { width: min(30rem, 100%) }`.
        assert_eq!(PanelGeometry::of(SheetSide::Right).width, Some(480.));
        assert_eq!(PanelGeometry::of(SheetSide::Top).max_height, Some(0.80));
        assert_eq!(PanelGeometry::of(SheetSide::Bottom).max_height, Some(0.85));
        assert_eq!(PanelGeometry::of(SheetSide::Bottom).width, None);
        let g = PanelGeometry::of(SheetSide::Left);
        assert_eq!(g.header_padding, (16., 64., 16., 16.));
        assert_eq!(g.body_padding, (0., 16., 24.));
    }

    #[test]
    fn slides_from_its_edge() {
        assert_eq!(SheetSide::Right.from(), (1., 0.));
        assert_eq!(SheetSide::Left.from(), (-1., 0.));
        assert_eq!(SheetSide::Top.from(), (0., -1.));
        assert_eq!(SheetSide::Bottom.from(), (0., 1.));
    }

    #[test]
    fn motion_follows_cu_in_and_cu_fade() {
        assert_eq!(PANEL_ENTER.speed.millis(), 240);
        assert_eq!(PANEL_ENTER.curve.points(), (0., 0., 0.3, 1.));
        assert_eq!(SCRIM_ENTER.speed.millis(), 110);
        assert_eq!(SCRIM_ENTER.curve.points(), (0., 0., 0.38, 0.9));
        assert!(PANEL_EXIT.speed.millis() < PANEL_ENTER.speed.millis());
    }

    #[test]
    fn look_follows_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let look = PanelLook::current();
        assert_eq!(hex(look.panel), "#f4f4f4");
        assert_eq!(hex(look.description), "#525252");
        assert_eq!(hex(look.handle), "#8d8d8d");
    }
}
