//! Navigation inside a thread ("Thread navigation" in `t3.css`):
//! [`ThreadFindBar`] (`.t3-find`), [`TimelineMinimap`] (`.t3-minimap`) and
//! [`ScrollToEndButton`] (`.t3-toend`, made a square quiet control by the
//! craft pass).
//!
//! The find bar's field is Kit's text input; Enter / Shift+Enter step to the
//! next / previous match and Escape closes, as the buttons' labels promise.
use super::requests::icon_button;
use crate::components::InputState;
use crate::components::floating::Align;
use crate::components::overlay::floating_shadow;
use crate::t3::press::{self, Handler, rem};
use crate::{components::Tooltip, icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::component::Sizable as _;
use gpui_kit::{
    App, AppContext as _, BoxShadow, CursorStyle, ElementId, Hsla, InteractiveElement as _,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// `ThreadFindBar`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct FindTexts {
    /// Placeholder and accessible name.
    pub placeholder: SharedString,
    /// `"{current} of {total}"`.
    pub count: Rc<dyn Fn(usize, usize) -> SharedString>,
    /// No matches.
    pub none: SharedString,
    /// Previous match (⇧↵).
    pub previous: SharedString,
    /// Next match (↵).
    pub next: SharedString,
    /// Close (Esc).
    pub close: SharedString,
}

impl Default for FindTexts {
    fn default() -> Self {
        Self {
            placeholder: "Find in thread".into(),
            count: Rc::new(|c, t| format!("{c} of {t}").into()),
            none: "No results".into(),
            previous: "Previous match (⇧↵)".into(),
            next: "Next match (↵)".into(),
            close: "Close (Esc)".into(),
        }
    }
}

type QueryHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type JumpHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The thread's find bar.
#[derive(IntoElement)]
pub struct ThreadFindBar {
    id: ElementId,
    query: SharedString,
    current: usize,
    total: usize,
    texts: FindTexts,
    on_close: Option<Handler>,
    on_next: Option<Handler>,
    on_previous: Option<Handler>,
    on_query: Option<QueryHandler>,
}

impl ThreadFindBar {
    /// A find bar seeded with `query` (`defaultValue`).
    pub fn new(id: impl Into<ElementId>, query: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            query: query.into(),
            current: 0,
            total: 0,
            texts: FindTexts::default(),
            on_close: None,
            on_next: None,
            on_previous: None,
            on_query: None,
        }
    }

    /// `current` of `total` matches.
    pub fn matches(mut self, current: usize, total: usize) -> Self {
        self.current = current;
        self.total = total;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: FindTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onClose` (the × and Escape).
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// Next match (the ⌄ button and Enter).
    pub fn on_next(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_next = Some(Rc::new(f));
        self
    }

    /// Previous match (the ⌃ button and Shift+Enter).
    pub fn on_previous(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_previous = Some(Rc::new(f));
        self
    }

    /// The typed query changed.
    pub fn on_query(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_query = Some(Rc::new(f));
        self
    }
}

/// `.t3-find { width: 22rem; height: size-md }` in px.
pub const FIND_BAR_SIZE: (f32, f32) = (352., 40.);

impl RenderOnce for ThreadFindBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let seed = self.query.clone();
        let state = window
            .use_keyed_state((id.clone(), "input"), cx, move |window, cx| {
                cx.new(|cx| InputState::new(window, cx).default_value(seed))
            })
            .read(cx)
            .clone();
        let query = state.read(cx).value().to_string();
        if let Some(f) = self.on_query.clone() {
            let last = window.use_keyed_state((id.clone(), "last"), cx, |_, _| query.clone());
            if *last.read(cx) != query {
                last.update(cx, |l, _| *l = query.clone());
                let q: SharedString = query.clone().into();
                window.defer(cx, move |window, cx| f(q, window, cx));
            }
        }
        let texts = self.texts.clone();
        let count = if self.total > 0 {
            (texts.count)(self.current, self.total)
        } else {
            texts.none.clone()
        };
        let (next, previous, close) = (
            self.on_next.clone(),
            self.on_previous.clone(),
            self.on_close.clone(),
        );
        // `.t3-find { display: flex; align-items: center; gap: spacing-02; width: 22rem;
        // height: size-md; padding-left: spacing-04; background: layer-01; box-shadow: 0 2px 6px shadow }`.
        div()
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(4.))
            .w(rem(FIND_BAR_SIZE.0))
            .max_w_full()
            .h(rem(FIND_BAR_SIZE.1))
            .pl(rem(12.))
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                let handler = match e.keystroke.key.as_str() {
                    "enter" if e.keystroke.modifiers.shift => previous.clone(),
                    "enter" => next.clone(),
                    "escape" => close.clone(),
                    _ => return,
                };
                cx.stop_propagation();
                if let Some(f) = handler {
                    f(window, cx)
                }
            })
            // `.t3-find > svg { fill: icon-secondary }`.
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
                        this.child(
                            div()
                                .absolute()
                                .left_0()
                                .text_color(color(ColorRole::TextPlaceholder))
                                .child(texts.placeholder.clone()),
                        )
                    })
                    .child(
                        gpui_kit::component::input::Input::new(&state)
                            .appearance(false)
                            .with_size(gpui_kit::component::Size::XSmall)
                            .p_0()
                            .w_full()
                            .text_size(rem(14.))
                            .line_height(rem(18.)),
                    ),
            )
            // `.t3-find-count { font-size: 0.75rem; color: text-secondary; white-space: nowrap }`.
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .text_size(rem(12.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(count),
            )
            .child(icon_button(
                (id.clone(), "prev"),
                texts.previous.clone(),
                CarbonIcon::ChevronUp,
                Align::Top,
                self.on_previous.clone(),
            ))
            .child(icon_button(
                (id.clone(), "next"),
                texts.next.clone(),
                CarbonIcon::ChevronDown,
                Align::Top,
                self.on_next.clone(),
            ))
            .child(icon_button(
                (id.clone(), "close"),
                texts.close.clone(),
                CarbonIcon::Close,
                Align::Top,
                self.on_close.clone(),
            ))
    }
}

/// A minimap mark's kind (`.t3-mm--<kind>`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MinimapKind {
    /// Your message (`interactive`).
    User,
    /// Agent work (the default `icon-secondary`).
    #[default]
    Tool,
    /// An approval (`support-warning`, 5px tall).
    Approval,
    /// An error (`support-error`).
    Error,
    /// A find match (`highlight`, 1px `border-interactive` outline).
    Find,
}

impl MinimapKind {
    /// Every kind.
    pub const ALL: [MinimapKind; 5] = [
        Self::User,
        Self::Tool,
        Self::Approval,
        Self::Error,
        Self::Find,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Tool => "tool",
            Self::Approval => "approval",
            Self::Error => "error",
            Self::Find => "find",
        }
    }

    /// (fill, height px, outline).
    pub fn look(self) -> (Hsla, f32, Option<Hsla>) {
        match self {
            Self::User => (color(ColorRole::Interactive), 3., None),
            Self::Tool => (color(ColorRole::IconSecondary), 3., None),
            Self::Approval => (color(ColorRole::SupportWarning), 5., None),
            Self::Error => (color(ColorRole::SupportError), 3., None),
            Self::Find => (
                color(ColorRole::Highlight),
                3.,
                Some(color(ColorRole::BorderInteractive)),
            ),
        }
    }
}

/// A minimap mark (`{ at, kind, label }`).
#[derive(Clone, Debug, PartialEq)]
pub struct MinimapMark {
    /// Position, 0 (top) … 1 (bottom).
    pub at: f32,
    /// Kind.
    pub kind: MinimapKind,
    /// Accessible name and tooltip.
    pub label: SharedString,
}

impl MinimapMark {
    /// A mark.
    pub fn new(at: f32, kind: MinimapKind, label: impl Into<SharedString>) -> Self {
        Self {
            at,
            kind,
            label: label.into(),
        }
    }
}

/// `.t3-minimap { width: 12px; height: 14rem }` in px.
pub const MINIMAP_SIZE: (f32, f32) = (12., 224.);

/// The thread overview strip beside the timeline.
#[derive(IntoElement)]
pub struct TimelineMinimap {
    id: ElementId,
    marks: Vec<MinimapMark>,
    viewport: (f32, f32),
    on_jump: Option<JumpHandler>,
}

impl TimelineMinimap {
    /// A minimap with `marks`; the viewport defaults to `[0.6, 0.85]`.
    pub fn new(id: impl Into<ElementId>, marks: Vec<MinimapMark>) -> Self {
        Self {
            id: id.into(),
            marks,
            viewport: (0.6, 0.85),
            on_jump: None,
        }
    }

    /// `viewport`: the visible span, 0…1.
    pub fn viewport(mut self, start: f32, end: f32) -> Self {
        self.viewport = (start, end);
        self
    }

    /// `onJump(mark index)`.
    pub fn on_jump(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_jump = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for TimelineMinimap {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (start, end) = self.viewport;
        let marks: Vec<_> = self
            .marks
            .into_iter()
            .enumerate()
            .map(|(i, mark)| {
                let (fill, height, outline) = mark.kind.look();
                let key = SharedString::from(format!("mark-{i}"));
                let (focus, ring) = press::focus((id.clone(), key.clone()), window, cx);
                let jump = self.on_jump.clone();
                // `.t3-mm-mark { position: absolute; left: 2px; width: 8px; height: 3px }`.
                let button = gpui_kit::base::Button::new((id.clone(), key.clone()))
                    .track_focus(&focus)
                    .accessibility_label(mark.label.clone())
                    .relative()
                    .w(px(8.))
                    .h(px(height))
                    .bg(fill)
                    .cursor(CursorStyle::PointingHand)
                    .when_some(outline, |this, c| {
                        this.shadow(vec![
                            BoxShadow::new(px(0.), px(0.), c).spread_radius(px(1.)),
                        ])
                    })
                    .when_some(jump, |this, f| this.on_click(move |_, w, cx| f(i, w, cx)))
                    // `.t3-mm-mark:focus-visible { outline-offset: 1px }`.
                    .when(ring, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top(px(-3.))
                                .left(px(-3.))
                                .right(px(-3.))
                                .bottom(px(-3.))
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    });
                // The tooltip wraps the mark, so the wrapper carries the position.
                div()
                    .absolute()
                    .left(px(2.))
                    .top(relative(mark.at.clamp(0., 1.)))
                    .child(
                        Tooltip::new((id.clone(), SharedString::from(format!("tip-{i}"))), button)
                            .content(mark.label)
                            .align(Align::Left),
                    )
            })
            .collect();
        // `.t3-minimap { position: relative; width: 12px; height: 14rem; background: layer-01 }`.
        div()
            .id(id.clone())
            .relative()
            .flex_none()
            .w(px(MINIMAP_SIZE.0))
            .h(rem(MINIMAP_SIZE.1))
            .bg(color(ColorRole::Layer01))
            .children(marks)
            // `.t3-mm-view { left: 0; right: 0; background: background-hover;
            // outline: 1px solid border-strong-01; outline-offset: -1px; pointer-events: none }`.
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(relative(start.clamp(0., 1.)))
                    .h(relative((end - start).clamp(0., 1.)))
                    .bg(color(ColorRole::BackgroundHover))
                    .border_1()
                    .border_color(color(ColorRole::BorderStrong01)),
            )
    }
}

/// `ScrollToEndButton`'s label: `"{n} new"` or the idle text.
pub fn scroll_to_end_label(unread: usize, idle: &str) -> SharedString {
    if unread > 0 {
        format!("{unread} new").into()
    } else {
        idle.to_string().into()
    }
}

/// The "jump to latest" control.
#[derive(IntoElement)]
pub struct ScrollToEndButton {
    id: ElementId,
    unread: usize,
    idle_label: SharedString,
    on_click: Option<Handler>,
}

impl ScrollToEndButton {
    /// The control; `unread` new messages show as "3 new".
    pub fn new(id: impl Into<ElementId>, unread: usize) -> Self {
        Self {
            id: id.into(),
            unread,
            idle_label: "Jump to latest".into(),
            on_click: None,
        }
    }

    /// The label without unread messages (default "Jump to latest").
    pub fn idle_label(mut self, label: impl Into<SharedString>) -> Self {
        self.idle_label = label.into();
        self
    }

    /// `onClick`.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ScrollToEndButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let label = scroll_to_end_label(self.unread, &self.idle_label);
        let ink = color(ColorRole::TextPrimary);
        let mut shadow = floating_shadow();
        // gemini-A: `box-shadow: inset 0 0 0 1px border-strong-01, 0 2px 6px shadow`.
        shadow.insert(
            0,
            BoxShadow::new(px(0.), px(0.), color(ColorRole::BorderStrong01))
                .spread_radius(px(1.))
                .inset(),
        );
        let on_click = self.on_click.clone();
        // `.t3-toend { display: inline-flex; align-items: center; gap: spacing-03; height: 2rem;
        // padding: 0 spacing-04; border-radius: 0; background: layer-02; color: text-primary;
        // font-size: 0.75rem }`.
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .accessibility_label(label.clone())
            .relative()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .gap(rem(8.))
            .h(rem(32.))
            .px(rem(12.))
            .bg(color(ColorRole::Layer02))
            .shadow(shadow)
            .text_color(ink)
            .text_size(rem(12.))
            .line_height(rem(16.))
            .whitespace_nowrap()
            .cursor(CursorStyle::PointingHand)
            .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
            .child(press::icon(CarbonIcon::ArrowDown, 16., ink))
            .child(label)
            .when(ring, |this| this.child(press::ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn approvals_are_taller_and_finds_outlined() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(MinimapKind::Approval.look().1, 5.);
        assert_eq!(MinimapKind::User.look().0, color(ColorRole::Interactive));
        assert!(MinimapKind::Find.look().2.is_some());
        assert_eq!(MinimapKind::Tool.look().0, color(ColorRole::IconSecondary));
    }

    #[test]
    fn scroll_label_counts_unread() {
        assert_eq!(scroll_to_end_label(3, "Jump to latest"), "3 new");
        assert_eq!(scroll_to_end_label(0, "Jump to latest"), "Jump to latest");
    }
}
