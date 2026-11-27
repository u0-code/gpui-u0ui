//! RemScale: lay a subtree out at a scaled rem size, the GPUI counterpart of
//! the LayoutRules specimen's `transform: scale(0.6)` and of browser zoom
//! (80%–200%). Every rem-based length, font size and container query in it
//! scales together, so a [`super::ShellLayout`] inside reflows exactly as it
//! would at that zoom. Pixel lengths (1px rules) stay unscaled.
use gpui_kit::{
    AnyElement, App, Bounds, Element, ElementId, Empty, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Window,
};

type Build = Box<dyn FnOnce(&mut Window, &mut App) -> AnyElement>;

/// Render `build`'s element with the rem size multiplied by `factor`.
pub fn rem_scale(
    factor: f32,
    build: impl FnOnce(&mut Window, &mut App) -> AnyElement + 'static,
) -> RemScale {
    RemScale {
        factor,
        build: Some(Box::new(build)),
        child: None,
        rem: None,
    }
}

/// The element made by [`rem_scale`].
pub struct RemScale {
    factor: f32,
    build: Option<Build>,
    child: Option<AnyElement>,
    rem: Option<Pixels>,
}

impl Element for RemScale {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let rem = window.rem_size() * self.factor;
        self.rem = Some(rem);
        let build = self.build.take();
        let child = &mut self.child;
        let id = window.with_rem_size(Some(rem), |window| {
            let mut element = match build {
                Some(build) => build(window, cx),
                None => Empty.into_any_element(),
            };
            let id = element.request_layout(window, cx);
            *child = Some(element);
            id
        });
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
        if let Some(child) = &mut self.child {
            window.with_rem_size(self.rem, |window| child.prepaint(window, cx));
        }
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
        if let Some(child) = &mut self.child {
            window.with_rem_size(self.rem, |window| child.paint(window, cx));
        }
    }
}
