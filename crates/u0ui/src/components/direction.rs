//! Direction: `DirectionProvider` / `Direction` and [`use_direction`]
//! (`DirCtx` in `index.jsx`; Carbon has no core equivalent).
//!
//! The reference wraps its children in `<div dir={direction}>` and offers the
//! value through React context. In the browser `dir="rtl"` mirrors the layout
//! (flex rows run right to left, `start`/`end` swap, text aligns right) and the
//! Unicode bidi algorithm orders mixed-direction text.
//!
//! **What GPUI can and cannot do.** GPUI lays out left to right only: it has no
//! `dir` attribute, no logical (`start`/`end`) properties, and a flex row never
//! reverses by itself. So this provider does **not** mirror anything. It is a
//! context value: while its subtree renders, [`use_direction`] (alias
//! [`current`]) returns the provided direction, so a component that wants to
//! mirror can read it and flip its own geometry. Text inside one run is shaped
//! by the platform (Core Text on macOS), which draws a purely right-to-left run
//! in the right order, but GPUI applies no paragraph-level bidi: mixed runs keep
//! their left-to-right sequence and text aligns left. No u0ui component mirrors
//! today.
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
    LayoutId, ParentElement, Pixels, Styled as _, Window, div,
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

/// Provides a direction to its subtree. See the module docs: it does not
/// mirror the layout. (The reference also exports it as `Direction`; in Rust
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
            // `<div dir=…>`: a block holding the children.
            child: div()
                .flex()
                .flex_col()
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
    }
}
