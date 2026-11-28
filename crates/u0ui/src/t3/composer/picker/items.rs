//! What [`super::ModelPicker`] is given: its tabs, its rows, the agent's
//! options and its words, as plain data.
use crate::icons::CarbonIcon;
use gpui_kit::{AnyElement, SharedString};
use std::rc::Rc;

/// A mark drawn at a given size in px: a provider's logo, which only the
/// caller knows how to draw.
pub type PickerMark = Rc<dyn Fn(f32) -> AnyElement>;

/// A model row (`{ id, name, description, badge }`), with what a row can
/// offer besides being chosen.
#[derive(Clone)]
pub struct PickerModel {
    /// `id`, unique among the rows shown.
    pub id: SharedString,
    /// `name`.
    pub name: SharedString,
    /// `description`.
    pub description: Option<SharedString>,
    /// `badge` (a small blue tag).
    pub badge: Option<SharedString>,
    /// Its provider's mark at the row's start (rows of several providers).
    pub mark: Option<PickerMark>,
    /// Whether it is chosen now.
    pub chosen: bool,
    /// Whether it is a favourite: `Some` draws the star.
    pub favourite: Option<bool>,
    /// Whether it also runs: `Some` draws the toggle.
    pub also: Option<bool>,
    /// The key that chooses it, as a hint.
    pub key: Option<SharedString>,
    /// Shown, but not to be chosen.
    pub disabled: bool,
}

impl PickerModel {
    /// A model.
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            badge: None,
            mark: None,
            chosen: false,
            favourite: None,
            also: None,
            key: None,
            disabled: false,
        }
    }

    /// `description`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `badge`.
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// The provider's mark at the row's start.
    pub fn mark(mut self, mark: PickerMark) -> Self {
        self.mark = Some(mark);
        self
    }

    /// Chosen now.
    pub fn chosen(mut self, chosen: bool) -> Self {
        self.chosen = chosen;
        self
    }

    /// The favourite star, filled when `on`.
    pub fn favourite(mut self, on: bool) -> Self {
        self.favourite = Some(on);
        self
    }

    /// The "also run" toggle, on when `on`.
    pub fn also(mut self, on: bool) -> Self {
        self.also = Some(on);
        self
    }

    /// The key that chooses it.
    pub fn key(mut self, key: impl Into<SharedString>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Shown, but not to be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A provider tab (`{ id, name, starting, notice, noticeShort }`).
#[derive(Clone)]
pub struct PickerProvider {
    /// `id`.
    pub id: SharedString,
    /// `name`.
    pub name: SharedString,
    /// Its mark (else `ProviderIcon` by name).
    pub mark: Option<PickerMark>,
    /// An icon in place of a mark (a tab that is no provider: favourites).
    pub icon: Option<CarbonIcon>,
    /// `starting`: the starting ring.
    pub starting: bool,
    /// `notice`: a warning above the models.
    pub notice: Option<SharedString>,
    /// The notice's button.
    pub notice_action: Option<SharedString>,
    /// `noticeShort`: the tab's note (default "Needs setup" with a notice).
    pub notice_short: Option<SharedString>,
    /// What hovering the tab says.
    pub tooltip: Option<SharedString>,
    /// Shown, but not to be chosen.
    pub disabled: bool,
}

impl PickerProvider {
    /// A provider tab.
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            mark: None,
            icon: None,
            starting: false,
            notice: None,
            notice_action: None,
            notice_short: None,
            tooltip: None,
            disabled: false,
        }
    }

    /// Its mark.
    pub fn mark(mut self, mark: PickerMark) -> Self {
        self.mark = Some(mark);
        self
    }

    /// An icon in place of a mark.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// `starting`.
    pub fn starting(mut self, starting: bool) -> Self {
        self.starting = starting;
        self
    }

    /// `notice`.
    pub fn notice(mut self, notice: impl Into<SharedString>) -> Self {
        self.notice = Some(notice.into());
        self
    }

    /// The notice's button ([`ModelPicker::on_notice_action`]).
    pub fn notice_action(mut self, label: impl Into<SharedString>) -> Self {
        self.notice_action = Some(label.into());
        self
    }

    /// `noticeShort`: the tab's note, with or without a notice.
    pub fn notice_short(mut self, note: impl Into<SharedString>) -> Self {
        self.notice_short = Some(note.into());
        self
    }

    /// What hovering the tab says.
    pub fn tooltip(mut self, tip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tip.into());
        self
    }

    /// Shown, but not to be chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The tab's note: `noticeShort`, else "Needs setup" where there is a notice.
    pub(super) fn note(&self, texts: &ModelPickerTexts) -> Option<SharedString> {
        self.notice_short
            .clone()
            .or_else(|| self.notice.as_ref().map(|_| texts.needs_setup.clone()))
    }
}

/// One of an option's choices, as its agent states it.
#[derive(Clone, Debug, PartialEq)]
pub struct PickerChoice {
    /// The value set when chosen.
    pub value: SharedString,
    /// Its name.
    pub name: SharedString,
    /// Whether it is the current one.
    pub on: bool,
}

impl PickerChoice {
    /// A choice.
    pub fn new(value: impl Into<SharedString>, name: impl Into<SharedString>, on: bool) -> Self {
        Self {
            value: value.into(),
            name: name.into(),
            on,
        }
    }
}

/// An option the agent offers: its id, its name and its choices.
#[derive(Clone, Debug, PartialEq)]
pub struct PickerOption {
    /// Its id, handed back with the value chosen.
    pub id: SharedString,
    /// Its name.
    pub name: SharedString,
    /// Its choices, in the agent's order (a level option's lowest first).
    pub choices: Vec<PickerChoice>,
}

impl PickerOption {
    /// An option.
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        choices: Vec<PickerChoice>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            choices,
        }
    }

    /// The current choice's name.
    pub(super) fn current(&self) -> Option<SharedString> {
        self.choices.iter().find(|c| c.on).map(|c| c.name.clone())
    }
}

/// A two-way option drawn as Speed's switch: labelled with its own name.
#[derive(Clone, Debug, PartialEq)]
pub struct PickerSwitch {
    /// The option's name, beside the track.
    pub label: SharedString,
    /// Whether it is on.
    pub on: bool,
}

/// `ModelPicker`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct ModelPickerTexts {
    /// The tab note for a provider with a notice and no `noticeShort`.
    pub needs_setup: SharedString,
    /// Agent settings.
    pub settings: SharedString,
    /// `"Search {provider} models"`.
    pub search: Rc<dyn Fn(&str) -> SharedString>,
    /// Speed.
    pub speed: SharedString,
    /// What the list says when it has no rows.
    pub empty: SharedString,
    /// The star's label while off.
    pub add_favourite: SharedString,
    /// The star's label while on.
    pub remove_favourite: SharedString,
    /// The "also run" toggle's label.
    pub also: SharedString,
}

impl Default for ModelPickerTexts {
    fn default() -> Self {
        Self {
            needs_setup: "Needs setup".into(),
            settings: "Agent settings".into(),
            search: Rc::new(|p| format!("Search {p} models").into()),
            speed: "Speed".into(),
            empty: "No models".into(),
            add_favourite: "Add to favourites".into(),
            remove_favourite: "Remove from favourites".into(),
            also: "Also run on".into(),
        }
    }
}

/// Models whose names contain `query` (case-insensitive).
pub fn filter_models<'a>(models: &'a [PickerModel], query: &str) -> Vec<&'a PickerModel> {
    let q = query.to_lowercase();
    models
        .iter()
        .filter(|m| m.name.to_lowercase().contains(&q))
        .collect()
}

/// `.t3-mpick { grid-template-columns: 11rem 1fr 10rem; max-width: 46rem; height: 22rem }` in px.
pub const MODEL_PICKER_GEOMETRY: (f32, f32, f32, f32) = (176., 160., 736., 352.);

/// The narrowest the models column is let be beside the full provider tabs:
/// below it the tabs narrow to their marks.
pub const MODEL_PICKER_MIN_LIST: f32 = 280.;

/// The width of the tabs narrowed to their marks: a 40px row square, and
/// the 8px of the column's padding.
pub const MODEL_PICKER_NARROW_SIDE: f32 = 48.;

/// Whether the tabs narrow to their marks in `width` px: when the full
/// tabs and the options would leave the models less than
/// [`MODEL_PICKER_MIN_LIST`].
pub fn narrow_tabs(width: f32, options: bool) -> bool {
    let (side, opts, ..) = MODEL_PICKER_GEOMETRY;
    width < side + if options { opts } else { 0. } + MODEL_PICKER_MIN_LIST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_filters_by_name() {
        let models = vec![
            PickerModel::new("opus", "Opus"),
            PickerModel::new("sonnet", "Sonnet"),
        ];
        let hits = filter_models(&models, "SON");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "sonnet");
        assert_eq!(filter_models(&models, "").len(), 2);
    }

    #[test]
    fn the_tabs_narrow_to_their_marks_only_where_the_models_would_be_crowded() {
        let (side, opts, full, _) = MODEL_PICKER_GEOMETRY;
        assert!(!narrow_tabs(full, true));
        assert!(!narrow_tabs(side + opts + MODEL_PICKER_MIN_LIST, true));
        assert!(narrow_tabs(side + opts + MODEL_PICKER_MIN_LIST - 1., true));
        // Without options the models have the options' room too.
        assert!(!narrow_tabs(side + MODEL_PICKER_MIN_LIST, false));
    }

    #[test]
    fn a_tabs_note_is_its_own_else_needs_setup_where_it_has_a_notice() {
        let texts = ModelPickerTexts::default();
        assert_eq!(PickerProvider::new("a", "A").note(&texts), None);
        let notice = PickerProvider::new("a", "A").notice("Sign in");
        assert_eq!(notice.note(&texts).as_deref(), Some("Needs setup"));
        let own = PickerProvider::new("a", "A").notice_short("Starting");
        assert_eq!(own.note(&texts).as_deref(), Some("Starting"));
    }

    #[test]
    fn an_options_current_value_is_the_choice_its_agent_reports() {
        let o = PickerOption::new(
            "mode",
            "Mode",
            vec![
                PickerChoice::new("a", "Ask", false),
                PickerChoice::new("b", "Auto", true),
            ],
        );
        assert_eq!(o.current().as_deref(), Some("Auto"));
        let none = PickerOption::new("m", "M", vec![PickerChoice::new("a", "A", false)]);
        assert_eq!(none.current(), None);
    }
}
