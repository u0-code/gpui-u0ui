//! [`fit_width`]: scale a wide, fixed-proportion specimen down to its
//! column instead of letting it overflow sideways (the gallery's
//! no-horizontal-scrolling rule).
//!
//! u0ui sizes in rems, so drawing a subtree under a smaller rem size
//! (`Window::with_rem_size`) scales its geometry and text together, like the
//! 80% zoom the components are built to reflow at. The element measures its
//! width while painting and, when the scale it drew with no longer matches,
//! stores the new one and asks for another frame.
use gpui_kit::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Style, Window, relative,
};

/// Below this the specimen stops shrinking (and may clip instead).
const MIN_SCALE: f32 = 0.5;

/// `child` at full size when its column is at least `natural` rems wide,
/// otherwise scaled down to fit it.
pub fn fit_width(id: impl Into<ElementId>, natural: f32, child: impl IntoElement) -> FitWidth {
    FitWidth {
        id: id.into(),
        natural,
        child: child.into_any_element(),
    }
}

/// The scale that fits `natural` rems into `width` at `rem`.
pub fn scale_for(width: f32, natural: f32, rem: f32) -> f32 {
    (width / (natural * rem)).clamp(MIN_SCALE, 1.)
}

/// See [`fit_width`].
pub struct FitWidth {
    id: ElementId,
    natural: f32,
    child: AnyElement,
}

impl Element for FitWidth {
    /// The rem size the child is drawn at.
    type RequestLayoutState = Pixels;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Pixels) {
        let scale = id.map_or(1., |id| {
            window.with_element_state(id, |s: Option<f32>, _| {
                let s = s.unwrap_or(1.);
                (s, s)
            })
        });
        let rem = window.rem_size() * scale;
        let child = window.with_rem_size(Some(rem), |w| self.child.request_layout(w, cx));
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        (window.request_layout(style, [child], cx), rem)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        rem: &mut Pixels,
        window: &mut Window,
        cx: &mut App,
    ) {
        let want = scale_for(
            f32::from(bounds.size.width),
            self.natural,
            f32::from(window.rem_size()),
        );
        if let Some(id) = id {
            let changed = window.with_element_state(id, |s: Option<f32>, _| {
                ((s.unwrap_or(1.) - want).abs() > 0.002, want)
            });
            if changed {
                window.request_animation_frame();
            }
        }
        window.with_rem_size(Some(*rem), |w| {
            self.child.prepaint(w, cx);
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        rem: &mut Pixels,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_rem_size(Some(*rem), |w| self.child.paint(w, cx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_only_below_the_natural_width() {
        assert_eq!(scale_for(1200., 60., 16.), 1.);
        assert!((scale_for(864., 60., 16.) - 0.9).abs() < 1e-6);
        assert_eq!(scale_for(100., 60., 16.), MIN_SCALE);
    }
}
