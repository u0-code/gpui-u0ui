//! A `ToggleGroup` with `fit_content` sizes each segment by its label, so a
//! long or CJK label is never clipped, even in a column narrower than the
//! labels; without it the segments share the column and clip.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::components::toggle_group::{ToggleGroup, ToggleGroupItem};
use u0ui::{tokens::TypeStyle, typography::TypeStyled as _};

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

const LABELS: [&str; 3] = ["Comfortable", "Compact", "コンパクト表示"];

struct Page {
    fit: bool,
    natural: Vec<Slot>,
    inside: Vec<Slot>,
}

/// `label` with a canvas over it that records its laid-out bounds.
fn measured(label: &'static str, slot: Slot) -> impl IntoElement {
    div().relative().child(label).child(
        canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
    )
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut group = ToggleGroup::new("tg").fit_content(self.fit);
        for (i, label) in LABELS.iter().enumerate() {
            group = group.item(
                ToggleGroupItem::new(i.to_string()).child(measured(label, self.inside[i].clone())),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            // The same labels, in the segments' type, where nothing constrains them.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(40.))
                    .type_style(TypeStyle::BodyCompact01)
                    .children(
                        LABELS
                            .iter()
                            .zip(&self.natural)
                            .map(|(label, slot)| measured(label, slot.clone())),
                    ),
            )
            // A column far narrower than the three labels.
            .child(div().w(px(120.)).child(group))
    }
}

/// The natural label bounds, and the label bounds inside the group.
fn measure(cx: &mut TestAppContext, fit: bool) -> (Vec<Bounds<Pixels>>, Vec<Bounds<Pixels>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let natural: Vec<Slot> = (0..3).map(|_| Slot::default()).collect();
    let inside: Vec<Slot> = (0..3).map(|_| Slot::default()).collect();
    let window = cx.update(|cx| {
        let (natural, inside) = (natural.clone(), inside.clone());
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| {
                cx.new(|_| Page {
                    fit,
                    natural,
                    inside,
                })
            },
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    let all = |slots: &[Slot]| {
        slots
            .iter()
            .map(|s| s.get().expect("measured"))
            .collect::<Vec<_>>()
    };
    (all(&natural), all(&inside))
}

fn width(b: &Bounds<Pixels>) -> f32 {
    f32::from(b.size.width)
}

/// The space from the end of one label to the start of the next.
fn gap(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> f32 {
    f32::from(b.origin.x) - f32::from(a.origin.x) - width(a)
}

#[gpui_kit::test]
fn fitted_segments_keep_every_label_whole(cx: &mut TestAppContext) {
    let (natural, inside) = measure(cx, true);
    for (n, i) in natural.iter().zip(&inside) {
        assert!(
            width(n) > 0. && (width(n) - width(i)).abs() < 0.5,
            "{natural:?} {inside:?}"
        );
    }
    // Each label clears the next by both segments' padding (16 + 16).
    for pair in inside.windows(2) {
        assert!(gap(&pair[0], &pair[1]) >= 31.5, "{inside:?}");
    }
}

#[gpui_kit::test]
fn filling_segments_share_a_narrow_column_and_overlap(cx: &mut TestAppContext) {
    let (_, inside) = measure(cx, false);
    // A 40px share is narrower than the labels: they run into each other
    // (and their segments clip them).
    assert!(gap(&inside[0], &inside[1]) < 0., "{inside:?}");
}
