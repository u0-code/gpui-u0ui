//! Accordion: shadcn `Accordion` → Carbon `Accordion` / `AccordionItem`
//! (`.cds--accordion`).
//!
//! - **Behavior**: each heading is a Base `Button` (focus, Enter/Space,
//!   pointer); items open independently (Carbon's default), controlled or
//!   uncontrolled. The panel is revealed with Base's `MotionReveal` (measured,
//!   clipped height) through Base's `Collapsible`.
//! - **Appearance**: 1px `border-subtle` lines between items (hover turns the
//!   lines around a closed item `layer-hover`), headings `size-sm/md/lg` tall
//!   on hover `layer-hover`, body-01 titles 1rem from the edge, a 16px chevron
//!   at the end (or the start with `align(Start)`), panels padded 0.5rem /
//!   1.5rem with a 25% end inset; `flush` insets the lines by 1rem; disabled
//!   items use `text-disabled` / `icon-disabled`; focus is a 2px `focus`
//!   outline.
//! - **Motion**: the chevron turns 180° in 110ms (`.cds--accordion__arrow {
//!   transition: all 110ms cubic-bezier(0.2, 0, 0.38, 0.9) }`; a cut under
//!   reduced motion); the panel's height and opacity animate over 110ms
//!   `cubic-bezier(0, 0, 0.38, 0.9)` (`.cds--accordion__wrapper`), and close
//!   the same way before the content unmounts; reduced motion keeps only the
//!   fade. Heading background and lines: 110ms productive standard.
use super::overlay::{self, OpenChange, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, SIZE_LG, SIZE_MD, SIZE_SM, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// Panel reveal: `.cds--accordion__wrapper { transition: all 110ms cubic-bezier(0, 0, 0.38, 0.9) }`.
pub const REVEAL: Motion = Motion::new(Curve::ProductiveEntrance, Speed::Fast02);
/// Heading background and item lines: `transition: background-color/border-color 110ms
/// cubic-bezier(0.2, 0, 0.38, 0.9)`.
pub const HEADING: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);

/// `index.d.ts` `AccordionProps.align`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AccordionAlign {
    /// Chevron at the start (`.cds--accordion--start`).
    Start,
    /// Chevron at the end (default).
    #[default]
    End,
}

impl AccordionAlign {
    /// Both alignments.
    pub const ALL: [AccordionAlign; 2] = [Self::Start, Self::End];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// `index.d.ts` `AccordionProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AccordionSize {
    /// `sm`: 2rem headings.
    Sm,
    /// `md`: 2.5rem headings (default).
    #[default]
    Md,
    /// `lg`: 3rem headings.
    Lg,
}

impl AccordionSize {
    /// Every size.
    pub const ALL: [AccordionSize; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// Heading `min-block-size` (`--layout-size-height-local`) in px.
    pub const fn height(self) -> f32 {
        match self {
            Self::Sm => SIZE_SM.px_value(),
            Self::Md => SIZE_MD.px_value(),
            Self::Lg => SIZE_LG.px_value(),
        }
    }
}

/// The colors of one item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccordionLook {
    /// Heading fill (`:hover { background-color: var(--layer-hover) }`).
    pub heading: Hsla,
    /// Title color.
    pub text: Hsla,
    /// Chevron color.
    pub icon: Hsla,
    /// Lines around the item.
    pub line: Hsla,
}

impl AccordionLook {
    /// The look of an item on the active theme.
    pub fn of(open: bool, hovered: bool, disabled: bool) -> Self {
        use ColorRole::*;
        let hovered = hovered && !disabled;
        Self {
            heading: if hovered {
                color(LayerHover01)
            } else {
                Hsla::transparent_black()
            },
            text: color(if disabled { TextDisabled } else { TextPrimary }),
            icon: color(if disabled { IconDisabled } else { IconPrimary }),
            // `:not(--active):hover { border-block-start-color: var(--layer-hover) }`.
            line: if hovered && !open {
                color(LayerHover01)
            } else {
                color(BorderSubtle01)
            },
        }
    }
}

/// One accordion section.
pub struct AccordionItem {
    title: SharedString,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    disabled: bool,
    children: Vec<AnyElement>,
}

impl AccordionItem {
    /// A section titled `title` (`AccordionTrigger`).
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            open: None,
            default_open: false,
            on_open_change: None,
            disabled: false,
            children: Vec::new(),
        }
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

    /// Called when the heading toggles.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// Disable the section.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Append a paragraph (a `<p>` child): `.cds--accordion__content > p`
    /// sets body-01, 14px on 20px. Text added with `child` is bare and keeps
    /// the item's body-compact-01, 14px on 18px.
    pub fn paragraph(mut self, text: impl Into<SharedString>) -> Self {
        self.children.push(
            div()
                .type_style(TypeStyle::Body01)
                .child(text.into())
                .into_any_element(),
        );
        self
    }
}

impl ParentElement for AccordionItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// A Carbon accordion. See the module docs.
#[derive(IntoElement)]
pub struct Accordion {
    id: ElementId,
    align: AccordionAlign,
    size: AccordionSize,
    flush: bool,
    items: Vec<AccordionItem>,
}

impl Accordion {
    /// An accordion with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            align: AccordionAlign::End,
            size: AccordionSize::Md,
            flush: false,
            items: Vec::new(),
        }
    }

    /// Chevron side (default end).
    pub fn align(mut self, align: AccordionAlign) -> Self {
        self.align = align;
        self
    }

    /// Heading size (default md).
    pub fn size(mut self, size: AccordionSize) -> Self {
        self.size = size;
        self
    }

    /// `isFlush`: lines inset by 1rem.
    pub fn flush(mut self, flush: bool) -> Self {
        self.flush = flush;
        self
    }

    /// Append a section.
    pub fn item(mut self, item: AccordionItem) -> Self {
        self.items.push(item);
        self
    }
}

impl RenderOnce for Accordion {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let reduced = motion::reduced_motion(cx);
        let count = self.items.len();
        // Lines are shared between neighbours: line i is above item i, line
        // `count` below the last. Hovering a closed item tints both of its lines.
        let mut states = Vec::with_capacity(count);
        for (index, item) in self.items.iter().enumerate() {
            let item_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("item-{index}")),
            );
            let openness = overlay::openness(
                &item_id,
                item.open,
                item.default_open,
                item.on_open_change.clone(),
                window,
                cx,
            );
            let hover = window.use_keyed_state((item_id.clone(), "hover"), cx, |_, _| false);
            states.push((item_id, openness, hover));
        }
        let mut lines: Vec<Hsla> = vec![color(ColorRole::BorderSubtle01); count + 1];
        for (index, (_, openness, hover)) in states.iter().enumerate() {
            let look =
                AccordionLook::of(openness.open, *hover.read(cx), self.items[index].disabled);
            if look.line != color(ColorRole::BorderSubtle01) {
                lines[index] = look.line;
                lines[index + 1] = look.line;
            }
        }

        let mut root = div().flex().flex_col().w_full();
        let line = |index: usize, target: Hsla, window: &mut Window, cx: &mut App| {
            let c = motion::animate_color(
                (id.clone(), SharedString::from(format!("line-{index}"))),
                target,
                HEADING,
                window,
                cx,
            );
            div()
                .h(px(1.))
                .when(self.flush, |this| this.mx(rem(16.)))
                .bg(c)
                .into_any_element()
        };
        for (index, (item, (item_id, openness, hover))) in
            self.items.into_iter().zip(states).enumerate()
        {
            root = root.child(line(index, lines[index], window, cx));
            let open = openness.open && !item.disabled;
            let hovered = *hover.read(cx);
            let look = AccordionLook::of(open, hovered, item.disabled);
            let heading_bg =
                motion::animate_color((item_id.clone(), "bg"), look.heading, HEADING, window, cx);
            let rotation = motion::chevron_rotation((item_id.clone(), "chevron"), open, window, cx);
            let progress = overlay::presence(
                (item_id.clone(), "reveal"),
                open,
                REVEAL,
                REVEAL,
                window,
                cx,
            );
            let set_open = openness.setter();
            let focus = window
                .use_keyed_state((item_id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            let focused = focus.is_focused(window);
            let chevron = Icon::carbon(CarbonIcon::ChevronDown)
                .size(IconSize::S16)
                .color(look.icon)
                .rotate(gpui_kit::radians(rotation));
            let start = self.align == AccordionAlign::Start;
            let heading = gpui_kit::base::Button::new((item_id.clone(), "heading"))
                .track_focus(&focus)
                .disabled(item.disabled)
                .accessibility_label(item.title.clone())
                .relative()
                .w_full()
                .min_h(rem(self.size.height()))
                .flex()
                .flex_row()
                .items_center()
                .justify_start()
                .bg(heading_bg)
                .text_color(look.text)
                .cursor(if item.disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::PointingHand
                })
                .on_hover(move |hovered, _, cx| {
                    hover.update(cx, |h, cx| {
                        *h = *hovered;
                        cx.notify();
                    })
                })
                .on_click(move |_, window, cx| set_open(!open, window, cx))
                .map(|this| {
                    let title = div()
                        .flex_1()
                        .min_w_0()
                        .type_style(TypeStyle::Body01)
                        .child(item.title.clone());
                    if start {
                        // `.cds--accordion--start`: arrow margin `2px 0 0 1rem`, title
                        // `padding-inline-start: 1rem; margin-inline-end: 1rem`.
                        this.child(div().ml(rem(16.)).mt(px(2.)).child(chevron))
                            .child(title.pl(rem(16.)).mr(rem(16.)))
                    } else {
                        // `flex-direction: row-reverse; padding-inline-end: 1rem`.
                        this.pr(rem(16.)).child(title.pl(rem(16.))).child(chevron)
                    }
                })
                .when(focused, |this| {
                    this.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .border_2()
                            .border_color(color(ColorRole::Focus)),
                    )
                });
            root = root.child(heading);
            if open || progress > 0. {
                let reveal = if reduced {
                    if open { 1. } else { 0. }
                } else {
                    progress
                };
                // `.cds--accordion__item--active > .cds--accordion__wrapper {
                // padding-block: 0.5rem 1.5rem }`; content `padding-inline: 1rem;
                // padding-inline-end: 25%`.
                let panel = div()
                    .pt(rem(8.))
                    .pb(rem(24.))
                    .pl(rem(if start { 48. } else { 16. }))
                    .pr(relative(0.25))
                    // Bare text inherits the item's body-compact-01 (14/18);
                    // only `> p` children are body-01 (see
                    // [`AccordionItem::paragraph`]).
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextPrimary))
                    .opacity(progress)
                    .children(item.children);
                root = root.child(
                    gpui_kit::base::Collapsible::new()
                        .open(true)
                        .reveal((item_id.clone(), "panel"), reveal)
                        .content(panel),
                );
            }
        }
        root.child(line(count, lines[count], window, cx))
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
    fn states_follow_accordion_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = AccordionLook::of(false, false, false);
        assert_eq!((rest.heading.a, hex(rest.line)), (0., "#c6c6c6".into()));
        let hover = AccordionLook::of(false, true, false);
        assert_eq!(
            (hex(hover.heading), hex(hover.line)),
            ("#e8e8e8".into(), "#e8e8e8".into())
        );
        // An open item keeps its lines on hover.
        assert_eq!(hex(AccordionLook::of(true, true, false).line), "#c6c6c6");
        let disabled = AccordionLook::of(false, true, true);
        assert_eq!(disabled.heading.a, 0.);
        assert_eq!(disabled.text, color(ColorRole::TextDisabled));
    }

    #[test]
    fn geometry_and_motion() {
        assert_eq!(AccordionSize::ALL.map(|s| s.height()), [32., 40., 48.]);
        assert_eq!(REVEAL.speed.millis(), 110);
        assert_eq!(REVEAL.curve.points(), (0., 0., 0.38, 0.9));
        assert_eq!(Motion::TOGGLE.speed.millis(), 110);
    }
}
