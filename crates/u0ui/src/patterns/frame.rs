//! `Frame` / `.cu-pframe`: the anatomy of a Carbon modal drawn in place (not
//! an overlay), so the pattern cards can compare dialog types and button
//! layouts side by side. A real modal is [`crate::components::Dialog`].
//!
//! - **Head** (`.cu-pframe-head`): optional label, title, the ghost close
//!   button in the top-right corner.
//! - **Body** (`.cu-pframe-body`): the caller's children in `body-01`.
//! - **Foot** (`.cu-pframe-foot`): `xl` buttons, full-bleed, sharing the
//!   width by flex weight, after an optional empty lead (`leftGap`), so one
//!   button can take the right half and three buttons a quarter each.
//!
//! ```ignore
//! PatternFrame::new("rename")
//!     .label("Project")
//!     .title("Rename project")
//!     .close_label("Close")
//!     .button(FrameButton::new("Cancel", ButtonVariant::Secondary))
//!     .button(FrameButton::new("Rename", ButtonVariant::Primary))
//!     .child(Input::new("name", &state).label("Project name"))
//! ```
use crate::{
    components::{
        button::{Button, ButtonSize, ButtonVariant},
        overlay::floating_shadow,
    },
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::{self, ColorRole, REM_PX, TokenLength, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, Div, ElementId, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder as _, relative, rems,
};
use std::rc::Rc;

/// The frame's box measurements (`src/patterns.css`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameGeometry {
    /// `.cu-pframe-head { padding: var(--spacing-05) 3rem 0 var(--spacing-05) }`: top.
    pub head_top: TokenLength,
    /// … end, clear of the 3rem close button.
    pub head_end: TokenLength,
    /// … start.
    pub head_start: TokenLength,
    /// `.cu-pframe-label { margin-bottom: 2px }`.
    pub label_gap: TokenLength,
    /// `.cu-pframe-body { padding: var(--spacing-05) var(--spacing-05) var(--spacing-07) }`: top and sides.
    pub body_pad: TokenLength,
    /// … bottom.
    pub body_bottom: TokenLength,
    /// `.cu-pframe--danger { border-top: 3px solid var(--support-error) }`, in px.
    pub danger_rule: f32,
    /// `Frame`'s default `width: p.width || 340`, in px.
    pub default_width: f32,
}

impl FrameGeometry {
    /// The reference geometry.
    pub const REFERENCE: Self = Self {
        head_top: tokens::SPACING_05,
        head_end: tokens::SIZE_LG,
        head_start: tokens::SPACING_05,
        label_gap: tokens::SPACING_01,
        body_pad: tokens::SPACING_05,
        body_bottom: tokens::SPACING_07,
        danger_rule: 3.,
        default_width: 340.,
    };
}

/// One foot button: its label, kind and flex weight (`widths[i] || 1`).
#[derive(Clone)]
pub struct FrameButton {
    label: SharedString,
    variant: ButtonVariant,
    weight: f32,
    disabled: bool,
    icon: Option<Icon>,
}

impl FrameButton {
    /// A button reading `label`, of `variant`, weight 1.
    pub fn new(label: impl Into<SharedString>, variant: ButtonVariant) -> Self {
        Self {
            label: label.into(),
            variant,
            weight: 1.,
            disabled: false,
            icon: None,
        }
    }

    /// The flex weight (`style: { flex: widths[i] || 1 }`).
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Disabled (`b.disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A trailing icon (`renderIcon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// The share of the foot each slot takes: the lead gap first (when its
/// weight is positive), then one entry per button.
pub fn foot_shares(lead: f32, weights: &[f32]) -> Vec<f32> {
    let total: f32 = lead.max(0.) + weights.iter().sum::<f32>();
    if total <= 0. {
        return Vec::new();
    }
    std::iter::once(lead)
        .filter(|l| *l > 0.)
        .chain(weights.iter().copied())
        .map(|w| w / total)
        .collect()
}

type ButtonHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type CloseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A static modal frame. See the module docs.
#[derive(IntoElement)]
pub struct PatternFrame {
    id: ElementId,
    label: Option<SharedString>,
    title: SharedString,
    close_label: SharedString,
    danger: bool,
    width: Option<f32>,
    lead: f32,
    buttons: Vec<FrameButton>,
    children: Vec<AnyElement>,
    on_button: Option<ButtonHandler>,
    on_close: Option<CloseHandler>,
}

impl PatternFrame {
    /// A frame with a stable `id`, 340px wide.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            title: SharedString::default(),
            close_label: SharedString::default(),
            danger: false,
            width: Some(FrameGeometry::REFERENCE.default_width),
            lead: 0.,
            buttons: Vec::new(),
            children: Vec::new(),
            on_button: None,
            on_close: None,
        }
    }

    /// The small label over the title (`p.label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The title (`p.title`).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// The close button's accessible name and tooltip (`label: 'Close'`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// The danger frame: a 3px `support-error` top rule (`p.danger`).
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// The width in px (`p.width`); `None` fills the container (`'100%'`).
    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }

    /// An empty lead before the buttons, as a flex weight (`p.leftGap`).
    pub fn lead_gap(mut self, weight: f32) -> Self {
        self.lead = weight;
        self
    }

    /// Append a foot button (`p.buttons`).
    pub fn button(mut self, button: FrameButton) -> Self {
        self.buttons.push(button);
        self
    }

    /// Called with a foot button's index when it is pressed.
    pub fn on_button(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_button = Some(Rc::new(f));
        self
    }

    /// Called when the close button is pressed.
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl ParentElement for PatternFrame {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// A flex slot taking `share` of the foot (`flex: <weight>` with a zero
/// basis, `.cu-pframe-foot .cds--btn { min-width: 0 }`).
fn slot(share: f32) -> Div {
    div().min_w_0().flex_shrink(1.).flex_basis(relative(share))
}

impl RenderOnce for PatternFrame {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let g = FrameGeometry::REFERENCE;
        let id = self.id;
        let close = {
            let on_close = self.on_close.clone();
            Button::new((id.clone(), "close"), self.close_label.clone())
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Lg)
                .icon_only(true)
                .icon(Icon::carbon(CarbonIcon::Close))
                .on_click(move |_, window, cx| {
                    if let Some(f) = &on_close {
                        f(window, cx)
                    }
                })
        };
        // `.cu-pframe-head` with `.cu-pframe-x { position: absolute; top: 0; right: 0 }`.
        let head = div()
            .relative()
            .flex()
            .flex_col()
            .pt(g.head_top)
            .pr(g.head_end)
            .pl(g.head_start)
            .when_some(self.label, |this, label| {
                this.child(
                    // `.cu-pframe-label`: 0.75rem, 0.32px, text-secondary.
                    div()
                        .mb(g.label_gap)
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(label),
                )
            })
            .child(
                // `.cu-pframe-title { font-size: 1.25rem; line-height: 1.4 }`: heading-03.
                div()
                    .type_style(TypeStyle::Heading03)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.title),
            )
            .child(div().absolute().top_0().right_0().child(close));

        // `.cu-pframe-body { font-size: 0.875rem; line-height: 1.42857 }`: body-01.
        let body = div()
            .flex()
            .flex_col()
            .pt(g.body_pad)
            .px(g.body_pad)
            .pb(g.body_bottom)
            .type_style(TypeStyle::Body01)
            .text_color(color(ColorRole::TextPrimary))
            .children(self.children);

        let foot = (!self.buttons.is_empty()).then(|| {
            let weights: Vec<f32> = self.buttons.iter().map(|b| b.weight).collect();
            let shares = foot_shares(self.lead, &weights);
            let (lead, button_shares) = if self.lead > 0. {
                (shares.first().copied(), &shares[1..])
            } else {
                (None, &shares[..])
            };
            let buttons = self
                .buttons
                .into_iter()
                .zip(button_shares.iter().copied())
                .enumerate()
                .map(|(i, (b, share))| {
                    let on_button = self.on_button.clone();
                    let mut button = Button::new((id.clone(), format!("b{i}")), b.label)
                        .variant(b.variant)
                        .size(ButtonSize::Xl)
                        .balanced(b.icon.is_none())
                        .full_width(true)
                        .disabled(b.disabled)
                        .on_click(move |_, window, cx| {
                            if let Some(f) = &on_button {
                                f(i, window, cx)
                            }
                        });
                    if let Some(icon) = b.icon {
                        button = button.icon(icon);
                    }
                    slot(share).child(button)
                });
            // `.cu-pframe-foot { display: flex }`.
            div()
                .flex()
                .flex_row()
                .w_full()
                .children(lead.map(slot))
                .children(buttons)
        });

        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .max_w_full()
            .map(|this| match self.width {
                Some(w) => this.w(rems(w / REM_PX)),
                None => this.w_full(),
            })
            // `.cu-pframe { background: var(--layer-01); box-shadow: 0 2px 6px var(--shadow) }`.
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .when(self.danger, |this| {
                this.border_t(gpui_kit::px(g.danger_rule))
                    .border_color(color(ColorRole::SupportError))
            })
            .child(head)
            .child(body)
            .children(foot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_the_css() {
        let g = FrameGeometry::REFERENCE;
        assert_eq!(
            (
                g.head_top.px_value(),
                g.head_end.px_value(),
                g.head_start.px_value()
            ),
            (16., 48., 16.)
        );
        assert_eq!(g.label_gap.px_value(), 2.);
        assert_eq!(
            (g.body_pad.px_value(), g.body_bottom.px_value()),
            (16., 32.)
        );
        assert_eq!((g.danger_rule, g.default_width), (3., 340.));
    }

    #[test]
    fn button_counts_take_their_dialog_widths() {
        // Two buttons: 50% each.
        assert_eq!(foot_shares(0., &[1., 1.]), vec![0.5, 0.5]);
        // One button after a lead gap: the right half.
        assert_eq!(foot_shares(1., &[1.]), vec![0.5, 0.5]);
        // Three buttons after a lead gap: a quarter each.
        assert_eq!(foot_shares(1., &[1., 1., 1.]), vec![0.25; 4]);
        assert!(foot_shares(0., &[]).is_empty());
    }
}
