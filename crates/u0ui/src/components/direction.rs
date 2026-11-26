//! Direction: `DirectionProvider` / `Direction` and [`use_direction`]
//! (`DirCtx` in `index.jsx`; Carbon has no core equivalent).
//!
//! The reference wraps its children in `<div dir={direction}>` and offers the
//! value through React context. In the browser `dir="rtl"` mirrors the layout
//! (flex rows run right to left, `start`/`end` swap, text aligns right) and the
//! Unicode bidi algorithm orders mixed-direction text.
//!
//! **Mirroring.** GPUI lays out left to right only: it has no `dir`
//! attribute and no logical (`start`/`end`) properties. So the provider is a
//! context value ([`use_direction`], alias [`current`]) that components read
//! while rendering to mirror themselves, as Carbon's logical properties do
//! under `dir="rtl"`. Components that honor it:
//!
//! - [`DirectionProvider`] itself: in RTL its children line up at the right
//!   edge (`align-items: flex-end`) with right-aligned text, so a
//!   `max-width` block sits at the start edge like a `dir="rtl"` block
//!   (children that should fill the width set `w_full`).
//! - [`super::breadcrumb::Breadcrumb`]: items run right to left, each
//!   separator after its item (to its left).
//! - Form items ([`super::form::form_item`]: [`super::input::Input`],
//!   [`super::textarea::Textarea`], selects and every labeled control),
//!   [`super::label::Label`] and [`super::field::Field`]: label, helper and
//!   messages align right; a horizontal field's row reverses.
//! - [`super::input::Input`]: typed text and the placeholder align right, the
//!   start/end paddings swap and the status icon / password toggle move to
//!   the left end.
//!
//! Text inside one run is shaped by the platform (Core Text on macOS), which
//! draws a purely right-to-left run in the right order, but GPUI applies no
//! paragraph-level bidi: mixed runs keep their left-to-right sequence. Other
//! components do not mirror yet.
//!
//! **Scope of the value.** The provider sets the value while its children are
//! *rendered* (GPUI renders `RenderOnce` components and views during layout,
//! inside the provider's own layout pass). Read it in `render`; event handlers
//! run outside any provider and should capture the value they need while
//! rendering. A cached view that skips re-rendering keeps what it read last.
//!
//! ```ignore
//! DirectionProvider::new(Direction::Rtl).child(MyMirroringRow::new())
//! // inside MyMirroringRow::render:
//! let rtl = use_direction() == Direction::Rtl;
//! ```
use gpui_kit::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, ParentElement, Pixels, Styled as _, Window, div, prelude::FluentBuilder as _,
};
use std::cell::RefCell;

/// `index.d.ts` `DirectionProvider.direction`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Left to right (the default, `createContext('ltr')`).
    #[default]
    Ltr,
    /// Right to left.
    Rtl,
}

impl Direction {
    /// Both directions.
    pub const ALL: [Direction; 2] = [Self::Ltr, Self::Rtl];

    /// The `index.d.ts` name (`"ltr"`, `"rtl"`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    /// Whether this is right to left.
    pub const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

thread_local! {
    /// The directions of the providers currently rendering, innermost last.
    static STACK: RefCell<Vec<Direction>> = const { RefCell::new(Vec::new()) };
}

/// The direction of the innermost [`DirectionProvider`] rendering now, or
/// `Ltr` outside any (`useDirection()`).
pub fn use_direction() -> Direction {
    STACK.with(|s| s.borrow().last().copied().unwrap_or_default())
}

/// Alias of [`use_direction`].
pub fn current() -> Direction {
    use_direction()
}

/// Whether the innermost provider rendering now is right to left.
pub fn is_rtl() -> bool {
    use_direction().is_rtl()
}

/// `(start, end)` mapped to physical `(left, right)` for `direction`: CSS
/// logical inline properties (`padding-inline-start`, `inset-inline-end`, …).
pub fn inline_sides<T>(direction: Direction, start: T, end: T) -> (T, T) {
    if direction.is_rtl() {
        (end, start)
    } else {
        (start, end)
    }
}

/// Run `f` with `direction` provided (what the provider does around its
/// children's layout). Public so custom elements and tests can provide too.
pub fn with_direction<R>(direction: Direction, f: impl FnOnce() -> R) -> R {
    STACK.with(|s| s.borrow_mut().push(direction));
    struct Pop;
    impl Drop for Pop {
        fn drop(&mut self) {
            STACK.with(|s| {
                s.borrow_mut().pop();
            });
        }
    }
    let _pop = Pop;
    f()
}

/// Provides a direction to its subtree (`<div dir=…>`). See the module docs
/// for what mirrors. (The reference also exports it as `Direction`; in Rust
/// that name is the [`Direction`] value enum.)
pub struct DirectionProvider {
    direction: Direction,
    children: Vec<AnyElement>,
}

impl DirectionProvider {
    /// A provider of `direction`.
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            children: Vec::new(),
        }
    }
}

impl ParentElement for DirectionProvider {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for DirectionProvider {
    type Element = ProviderElement;

    fn into_element(self) -> Self::Element {
        ProviderElement {
            direction: self.direction,
            // `<div dir=…>`: a block holding the children; in RTL they start
            // at the right edge and text aligns right (`text-align: start`).
            child: div()
                .flex()
                .flex_col()
                .when(self.direction.is_rtl(), |d| d.items_end().text_right())
                .children(self.children)
                .into_any_element(),
        }
    }
}

/// The element behind [`DirectionProvider`].
pub struct ProviderElement {
    direction: Direction,
    child: AnyElement,
}

impl Element for ProviderElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let child = &mut self.child;
        let id = with_direction(self.direction, || child.request_layout(window, cx));
        (id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        with_direction(self.direction, || {
            child.prepaint(window, cx);
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let child = &mut self.child;
        with_direction(self.direction, || child.paint(window, cx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_nest_and_default_to_ltr() {
        assert_eq!(use_direction(), Direction::Ltr);
        with_direction(Direction::Rtl, || {
            assert_eq!(use_direction(), Direction::Rtl);
            with_direction(Direction::Ltr, || assert_eq!(current(), Direction::Ltr));
            assert_eq!(use_direction(), Direction::Rtl);
        });
        assert_eq!(use_direction(), Direction::Ltr);
        assert_eq!(Direction::Rtl.name(), "rtl");
        assert!(Direction::Rtl.is_rtl());
        with_direction(Direction::Rtl, || assert!(is_rtl()));
        assert!(!is_rtl());
        // `padding-inline-start: 16px; padding-inline-end: 40px`.
        assert_eq!(inline_sides(Direction::Ltr, 16, 40), (16, 40));
        assert_eq!(inline_sides(Direction::Rtl, 16, 40), (40, 16));
    }
}
