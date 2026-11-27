//! The T3 command palette and list toolbar (`shell.jsx` "Command palette",
//! "List toolbar"): [`CommandPalette`], [`ListToolbar`].
use super::super::craft::{press, sans};
use super::super::press::{Handler, focus, hover, icon, rem, ring, track_hover};
use crate::{
    components::{
        dropdown_menu::{DropdownMenu, DropdownMenuKind, DropdownMenuSize},
        input::InputState,
        input_group::{
            AddonAlign, InputGroup, InputGroupAddon, InputGroupInput, input_group_button,
        },
        kbd::Kbd,
        menu::{self, MenuItem, MenuSize},
        overlay,
    },
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::{ColorRole, SIZE_LG, SIZE_MD, SIZE_SM},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ElementId, Entity, Focusable as _, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement, RenderOnce, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::Sizable as _,
    component::input::{Input, InputEvent},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

// ── CommandPalette ──────────────────────────────────────────────────────

/// One palette result (`{ title, subtitle, icon, leading, shortcut, onSelect }`).
pub struct PaletteItem {
    title: SharedString,
    subtitle: Vec<SharedString>,
    icon: Option<Icon>,
    leading: Option<AnyElement>,
    shortcut: Option<SharedString>,
    on_select: Option<Handler>,
}

impl PaletteItem {
    /// An item titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            subtitle: Vec::new(),
            icon: None,
            leading: None,
            shortcut: None,
            on_select: None,
        }
    }

    /// The subtitle parts, joined by a middle dot (`subtitle`).
    pub fn subtitle<S: Into<SharedString>>(mut self, parts: impl IntoIterator<Item = S>) -> Self {
        self.subtitle = parts.into_iter().map(Into::into).collect();
        self
    }

    /// A 16px glyph (`icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// A leading element instead of the glyph (`leading`, e.g. a monogram).
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    /// Space-separated keys (`shortcut`, e.g. `"⌘ N"`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// `onSelect`.
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

/// A labelled group of results.
pub struct PaletteGroup {
    label: SharedString,
    items: Vec<PaletteItem>,
}

impl PaletteGroup {
    /// A group labelled `label`.
    pub fn new(
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = PaletteItem>,
    ) -> Self {
        Self {
            label: label.into(),
            items: items.into_iter().collect(),
        }
    }
}

/// Whether an item matches `query`: `(title + ' ' + subtitle)` contains it,
/// ignoring case (an array subtitle stringifies with commas).
pub fn palette_matches(title: &str, subtitle: &[SharedString], query: &str) -> bool {
    let sub: Vec<&str> = subtitle.iter().map(|s| s.as_ref()).collect();
    format!("{title} {}", sub.join(","))
        .to_lowercase()
        .contains(&query.to_lowercase())
}

/// The active index after `key` (`ArrowDown`: +1 up to the last;
/// `ArrowUp`: −1 down to the first; no wrapping).
pub fn palette_step(key: &str, active: usize, len: usize) -> Option<usize> {
    match key {
        "down" if len > 0 => Some((active + 1).min(len - 1)),
        "up" => Some(active.saturating_sub(1)),
        _ => None,
    }
}

struct PaletteState {
    input: Entity<InputState>,
    active: usize,
    focused_once: bool,
    scroll: ScrollHandle,
    _subscription: Subscription,
}

/// The global command palette (`CommandPalette`, `.t3-palette`): a
/// `layer-01` floating panel, at most 40rem wide. A 48px search row (20px
/// magnifier, 16px input, an `Esc` key cap), the filtered groups (scrolling
/// past 20rem) and a footer of key hints.
///
/// Typing filters items by title and subtitle and resets the highlight to
/// the first match; ↑/↓ move it (no wrap), Enter runs it, the pointer
/// highlights on hover and runs on click. The highlighted row takes
/// `layer-selected-01` with a 3px `border-interactive` start bar. The field
/// takes focus when the palette first renders (`autoFocus`).
#[derive(IntoElement)]
pub struct CommandPalette {
    id: ElementId,
    groups: Vec<PaletteGroup>,
    placeholder: SharedString,
    empty: SharedString,
    escape_key: SharedString,
    hints: [SharedString; 3],
    auto_focus: bool,
}

impl CommandPalette {
    /// A palette with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            groups: Vec::new(),
            placeholder: "Search threads, projects and commands".into(),
            empty: "No matching commands, projects, or threads.".into(),
            escape_key: "Esc".into(),
            hints: ["Navigate".into(), "Open".into(), "Open in new tab".into()],
            auto_focus: true,
        }
    }

    /// Add a group (`groups`).
    pub fn group(mut self, group: PaletteGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// The field placeholder (`placeholder`).
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// The no-match text (`empty`).
    pub fn empty(mut self, text: impl Into<SharedString>) -> Self {
        self.empty = text.into();
        self
    }

    /// Focus the field on first render (`autoFocus`, default true).
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    /// The footer hints: navigate, open, open in new tab.
    pub fn hints(mut self, hints: [SharedString; 3]) -> Self {
        self.hints = hints;
        self
    }
}

fn kbd_row(keys: &[&'static str], text: SharedString) -> impl IntoElement {
    // `.t3-palette-foot span { display: flex; gap: 2px; align-items: center }`.
    div()
        .flex()
        .items_center()
        .gap(px(2.))
        .children(keys.iter().map(|k| Kbd::new(*k)))
        .child(div().ml(px(2.)).child(text))
}

/// (background, start bar) of a palette row.
pub fn palette_row_colors(on: bool) -> (Hsla, Option<Hsla>) {
    if on {
        // `.t3-palette-item--on { background: var(--layer-selected-01);
        // box-shadow: inset 3px 0 0 var(--border-interactive) }`.
        (
            color(ColorRole::LayerSelected01),
            Some(color(ColorRole::BorderInteractive)),
        )
    } else {
        (gpui_kit::transparent_black(), None)
    }
}

impl RenderOnce for CommandPalette {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let state = window.use_keyed_state((id.clone(), "palette"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx));
            let _subscription = cx.subscribe(&input, |this: &mut PaletteState, _, event, cx| {
                if matches!(event, InputEvent::Change) {
                    // `onChange={(e) => { setQ(e.target.value); setActive(0); }}`.
                    this.active = 0;
                    cx.notify();
                }
            });
            PaletteState {
                input,
                active: 0,
                focused_once: false,
                scroll: ScrollHandle::new(),
                _subscription,
            }
        });
        let input = state.read(cx).input.clone();
        if self.auto_focus && !state.read(cx).focused_once {
            let handle = input.read(cx).focus_handle(cx);
            window.focus(&handle, cx);
            state.update(cx, |s, _| s.focused_once = true);
        }
        let query = input.read(cx).value().to_string();
        let shown: Vec<PaletteGroup> = self
            .groups
            .into_iter()
            .map(|g| PaletteGroup {
                label: g.label,
                items: g
                    .items
                    .into_iter()
                    .filter(|it| palette_matches(&it.title, &it.subtitle, &query))
                    .collect(),
            })
            .filter(|g| !g.items.is_empty())
            .collect();
        let handlers: Vec<Option<Handler>> = shown
            .iter()
            .flat_map(|g| g.items.iter().map(|it| it.on_select.clone()))
            .collect();
        let len = handlers.len();
        let active = state.read(cx).active.min(len.saturating_sub(1));
        let scroll = state.read(cx).scroll.clone();

        // Row index (direct child of the list) of each item, for scrolling.
        let mut rows: Vec<AnyElement> = Vec::new();
        let mut item_rows: Vec<usize> = Vec::new();
        let mut flat = 0usize;
        for group in shown {
            // `.t3-palette-group { padding: var(--spacing-04) var(--spacing-05)
            // var(--spacing-02); font-size: 0.75rem; color: var(--text-secondary) }`.
            rows.push(
                sans(div(), 12., 400)
                    .pt(rem(12.))
                    .px(rem(16.))
                    .pb(rem(4.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(group.label)
                    .into_any_element(),
            );
            for item in group.items {
                let me = flat;
                flat += 1;
                item_rows.push(rows.len());
                let on = me == active;
                let (bg, bar) = palette_row_colors(on);
                let lead: Option<AnyElement> = item.leading.or_else(|| {
                    item.icon.map(|i| {
                        i.size_length(rem(16.))
                            .color(color(ColorRole::IconSecondary))
                            .into_any_element()
                    })
                });
                let subtitle = (!item.subtitle.is_empty()).then(|| {
                    let mut parts: Vec<AnyElement> = Vec::new();
                    for (i, s) in item.subtitle.into_iter().enumerate() {
                        if i > 0 {
                            // `.t3-metadot { color: var(--text-helper); padding: 0 2px }`.
                            parts.push(
                                div()
                                    .px(px(2.))
                                    .text_color(color(ColorRole::TextHelper))
                                    .child("·")
                                    .into_any_element(),
                            );
                        }
                        parts.push(div().child(s).into_any_element());
                    }
                    // `.t3-palette-sub { font-size: 0.75rem; color: var(--text-secondary) }`.
                    sans(div(), 12., 400)
                        .flex()
                        .text_color(color(ColorRole::TextSecondary))
                        .children(parts)
                });
                let shortcut = item.shortcut.map(|s| {
                    let keys: Vec<String> = s.split(' ').map(str::to_string).collect();
                    div()
                        .flex()
                        .flex_none()
                        .gap(px(2.))
                        .children(keys.into_iter().map(Kbd::new))
                });
                let hover_state = state.clone();
                let select = item.on_select.clone();
                // `.t3-palette-item { gap: var(--spacing-04); min-height: var(--size-md);
                // padding: var(--spacing-02) var(--spacing-05) }`.
                rows.push(
                    div()
                        .id((id.clone(), SharedString::from(format!("item-{me}"))))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(rem(12.))
                        .min_h(SIZE_MD.rems())
                        .py(rem(4.))
                        .px(rem(16.))
                        .bg(bg)
                        .cursor_pointer()
                        .on_hover(move |over, _, cx| {
                            if *over {
                                hover_state.update(cx, |s, cx| {
                                    if s.active != me {
                                        s.active = me;
                                        cx.notify();
                                    }
                                })
                            }
                        })
                        .on_click(move |_, w, cx| {
                            if let Some(f) = &select {
                                f(w, cx)
                            }
                        })
                        .when_some(bar, |d, c| {
                            d.child(div().absolute().left_0().top_0().bottom_0().w(px(3.)).bg(c))
                        })
                        // `.t3-palette-icon { display: flex; width: 1.25rem; justify-content: center }`.
                        .child(
                            div()
                                .flex()
                                .flex_none()
                                .w(rem(20.))
                                .justify_center()
                                .children(lead),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    sans(div(), 14., 400)
                                        .text_color(color(ColorRole::TextPrimary))
                                        .truncate()
                                        .child(item.title),
                                )
                                .children(subtitle),
                        )
                        .children(shortcut)
                        .into_any_element(),
                );
            }
        }

        let on_key = {
            let state = state.clone();
            let handlers = handlers.clone();
            let scroll = scroll.clone();
            let item_rows = item_rows.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let key = event.keystroke.key.as_str();
                let current = state.read(cx).active.min(len.saturating_sub(1));
                if let Some(next) = palette_step(key, current, len) {
                    state.update(cx, |s, cx| {
                        s.active = next;
                        cx.notify();
                    });
                    if let Some(row) = item_rows.get(next) {
                        scroll.scroll_to_item(*row);
                    }
                    cx.stop_propagation();
                } else if key == "enter" {
                    if let Some(Some(f)) = handlers.get(current) {
                        f(window, cx);
                    }
                    cx.stop_propagation();
                }
            }
        };

        let list = if len == 0 {
            // `.t3-palette-empty { padding: var(--spacing-07) var(--spacing-05);
            // text-align: center; color: var(--text-secondary) }`.
            sans(div(), 14., 400)
                .py(rem(32.))
                .px(rem(16.))
                .flex()
                .justify_center()
                .text_color(color(ColorRole::TextSecondary))
                .child(self.empty)
                .into_any_element()
        } else {
            // `.t3-palette-list { max-height: 20rem; overflow-y: auto; padding-bottom: var(--spacing-03) }`.
            div()
                .id((id.clone(), "list"))
                .flex()
                .flex_col()
                .max_h(rem(320.))
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .pb(rem(8.))
                .children(rows)
                .into_any_element()
        };
        let [navigate, open, new_tab] = self.hints;
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(640.))
            .bg(color(ColorRole::Layer01))
            .shadow(overlay::floating_shadow())
            .on_key_down(on_key)
            .child(
                // `.t3-palette-search { gap: var(--spacing-04); height: var(--size-lg);
                // padding: 0 var(--spacing-05); border-bottom: 1px solid var(--border-subtle-01) }`.
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .gap(rem(12.))
                    .h(SIZE_LG.rems())
                    .px(rem(16.))
                    .border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(icon(
                        CarbonIcon::Search,
                        20.,
                        color(ColorRole::IconSecondary),
                    ))
                    .child(
                        div()
                            .relative()
                            .flex()
                            .flex_1()
                            .min_w_0()
                            .items_center()
                            .when(query.is_empty(), |d| {
                                d.child(
                                    sans(div(), 16., 400)
                                        .absolute()
                                        .left_0()
                                        .text_color(color(ColorRole::TextPlaceholder))
                                        .child(self.placeholder.clone()),
                                )
                            })
                            .child(
                                Input::new(&input)
                                    .appearance(false)
                                    .with_size(gpui_kit::component::Size::XSmall)
                                    .aria_label(self.placeholder.clone())
                                    .p_0()
                                    .w_full()
                                    .text_size(rem(16.))
                                    .text_color(color(ColorRole::TextPrimary)),
                            ),
                    )
                    .child(Kbd::new(self.escape_key)),
            )
            .child(list)
            .child(
                // `.t3-palette-foot { gap: var(--spacing-05); padding: var(--spacing-03)
                // var(--spacing-05); border-top; font-size: 0.75rem; color: var(--text-secondary) }`.
                sans(div(), 12., 400)
                    .flex()
                    .flex_wrap()
                    .gap(rem(16.))
                    .py(rem(8.))
                    .px(rem(16.))
                    .border_t_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(kbd_row(&["↑", "↓"], navigate))
                    .child(kbd_row(&["↵"], open))
                    .child(kbd_row(&["⌘", "↵"], new_tab)),
            )
    }
}

// ── ListToolbar ─────────────────────────────────────────────────────────

/// One filter chip (`{ label, value }`): set when it has a value.
pub struct ListFilter {
    label: SharedString,
    value: Option<SharedString>,
    menu: Vec<MenuItem>,
    on_click: Option<Handler>,
}

impl ListFilter {
    /// A filter named `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: None,
            menu: Vec::new(),
            on_click: None,
        }
    }

    /// Its current value (`value`).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Menu items opened beneath this filter chip when clicked.
    pub fn menu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.menu = items.into_iter().collect();
        self
    }

    /// Open its picker.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// (background, text) of a filter chip (`r3-code`, the last word on
/// `.t3-filter`): rest `layer-01` / `text-secondary`, hover `layer-hover-01`,
/// set `layer-selected-01` / `text-primary`; the 1px border is transparent.
pub fn filter_colors(set: bool, hovered: bool) -> (Hsla, Hsla) {
    let bg = if hovered {
        ColorRole::LayerHover01
    } else if set {
        ColorRole::LayerSelected01
    } else {
        ColorRole::Layer01
    };
    let fg = if set || hovered {
        ColorRole::TextPrimary
    } else {
        ColorRole::TextSecondary
    };
    (color(bg), color(fg))
}

type ViewChange = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// A list page toolbar (`ListToolbar`, `.t3-listbar`): a 16rem search
/// field, the filter chips (32px, `label-01`, a 14px chevron; a set chip
/// shows `: <value>` in semibold), a ghost "Saved views" menu button and
/// trailing actions; wraps, 8px apart.
#[derive(IntoElement)]
pub struct ListToolbar {
    id: ElementId,
    placeholder: SharedString,
    search_width: f32,
    nowrap_filters: bool,
    filters: Vec<ListFilter>,
    views: Vec<SharedString>,
    view: Option<SharedString>,
    on_view_change: Option<ViewChange>,
    actions: Vec<AnyElement>,
    search: Option<Entity<InputState>>,
    labels: [SharedString; 3],
}

impl ListToolbar {
    /// A toolbar with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            placeholder: "Search".into(),
            search_width: 256.,
            nowrap_filters: false,
            filters: Vec::new(),
            views: Vec::new(),
            view: None,
            on_view_change: None,
            actions: Vec::new(),
            search: None,
            labels: [
                "Saved views".into(),
                "Save current view…".into(),
                "Clear".into(),
            ],
        }
    }

    /// Search field width in px (default `256.` = `16rem`).
    pub fn search_width(mut self, width: f32) -> Self {
        self.search_width = width;
        self
    }

    /// Keep the filter chip row unwrapped on narrow columns.
    pub fn nowrap_filters(mut self, nowrap: bool) -> Self {
        self.nowrap_filters = nowrap;
        self
    }

    /// The search placeholder (`placeholder`).
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Add a filter chip (`filters`).
    pub fn filter(mut self, filter: ListFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// The saved views (`views`).
    pub fn views<S: Into<SharedString>>(mut self, views: impl IntoIterator<Item = S>) -> Self {
        self.views = views.into_iter().map(Into::into).collect();
        self
    }

    /// The current view (`view`).
    pub fn view(mut self, view: impl Into<SharedString>) -> Self {
        self.view = Some(view.into());
        self
    }

    /// `onViewChange`.
    pub fn on_view_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_view_change = Some(Rc::new(f));
        self
    }

    /// Add a trailing action (`actions`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// The search field's text state (internal otherwise).
    pub fn search_state(mut self, state: Entity<InputState>) -> Self {
        self.search = Some(state);
        self
    }

    /// Labels: the views button with no view, the save item, the clear button.
    pub fn labels(mut self, labels: [SharedString; 3]) -> Self {
        self.labels = labels;
        self
    }
}

impl RenderOnce for ListToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let search = self.search.unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "search"), cx, |window, cx| {
                    cx.new(|cx| InputState::new(window, cx).clean_on_escape())
                })
                .read(cx)
                .clone()
        });
        let [views_label, save_label, clear_label] = self.labels;
        let has_text = !search.read(cx).value().is_empty();
        let mut group = InputGroup::new((id.clone(), "search-field"))
            .addon(InputGroupAddon::new().child(Icon::carbon(CarbonIcon::Search)))
            .input(InputGroupInput::new(&search).placeholder(self.placeholder));
        if has_text {
            let clear = search.clone();
            group = group.addon(
                InputGroupAddon::new().align(AddonAlign::InlineEnd).child(
                    input_group_button((id.clone(), "clear"), clear_label)
                        .icon(CarbonIcon::Close)
                        .icon_only(true)
                        .on_click(move |_, w, cx| clear.update(cx, |s, cx| s.set_value("", w, cx))),
                ),
            );
        }
        let mut chips: Vec<AnyElement> = Vec::new();
        for (i, f) in self.filters.into_iter().enumerate() {
            let chip_id: ElementId = (id.clone(), SharedString::from(format!("filter-{i}"))).into();
            let (hovered, store) = hover((chip_id.clone(), "hover"), window, cx);
            let (handle, focused) = focus((chip_id.clone(), "focus"), window, cx);
            let set = f.value.is_some();
            let (bg, fg) = filter_colors(set, hovered);
            // `.t3-filter { gap: var(--spacing-02); height: var(--size-sm); padding: 0
            // var(--spacing-03); font-size: 0.75rem }`; `b { font-weight: 600;
            // margin-left: calc(-1 * var(--spacing-02)) }`.
            let has_menu = !f.menu.is_empty();
            let menu_label = f.label.clone();
            let chip = sans(press(chip_id.clone(), &handle), 12., 400)
                .flex()
                .items_center()
                .gap(rem(4.))
                .h(SIZE_SM.rems())
                .px(rem(8.))
                .border_1()
                .border_color(gpui_kit::transparent_black())
                .bg(bg)
                .text_color(fg)
                .on_hover(track_hover(store))
                .when_some(f.on_click.filter(|_| !has_menu), |b, h| {
                    b.on_click(move |_, w, cx| h(w, cx))
                })
                .child(f.label)
                .children(f.value.map(|v| {
                    div()
                        .ml(rem(-4.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(format!(": {v}"))
                }))
                .child(icon(CarbonIcon::ChevronDown, 14., fg))
                .when(focused, |b| b.child(ring()));
            if has_menu {
                let openness = overlay::openness(&chip_id, None, false, None, window, cx);
                let open = openness.open;
                let set_open = openness.setter();
                let trigger = overlay::trigger_wrapper(
                    (chip_id.clone(), "trigger"),
                    chip.into_any_element(),
                    {
                        let set_open = set_open.clone();
                        Rc::new(move |w, cx| set_open(!open, w, cx))
                    },
                )
                .into_any_element();
                chips.push(
                    menu::anchored_menu(
                        &chip_id,
                        trigger,
                        menu::MenuSpec {
                            items: f.menu,
                            size: MenuSize::Sm,
                            label: menu_label,
                            open,
                            set_open,
                            min_width: None,
                            on_unhandled_key: None,
                        },
                        window,
                        cx,
                    )
                    .into_any_element(),
                );
            } else {
                chips.push(chip.into_any_element());
            }
        }
        let views = (!self.views.is_empty()).then(|| {
            let on_change = self.on_view_change.clone();
            let on_save = on_change.clone();
            let save_text = save_label.clone();
            DropdownMenu::new((id.clone(), "views"))
                .label(self.view.clone().unwrap_or(views_label))
                .kind(DropdownMenuKind::Ghost)
                .size(DropdownMenuSize::Md)
                .items(self.views.into_iter().map(|v| {
                    let on_change = on_change.clone();
                    MenuItem::item(v.clone()).on_select(move |w, cx| {
                        if let Some(f) = &on_change {
                            f(v.clone(), w, cx)
                        }
                    })
                }))
                .item(MenuItem::divider())
                .item(MenuItem::item(save_label).on_select(move |w, cx| {
                    if let Some(f) = &on_save {
                        f(save_text.clone(), w, cx)
                    }
                }))
        });
        // `.t3-listbar { flex-wrap: wrap; gap: var(--spacing-03); padding: var(--spacing-03) 0 }`.
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(8.))
            .py(rem(8.))
            // `.t3-listbar-search { width: 16rem }`.
            .child(
                div()
                    .w(rem(self.search_width))
                    .max_w_full()
                    .flex_none()
                    .child(group),
            )
            // `.t3-listbar-filters { flex-wrap: wrap; gap: var(--spacing-03); flex: 1 }`.
            .child(
                div()
                    .flex()
                    .flex_1()
                    .when(!self.nowrap_filters, |d| d.flex_wrap())
                    .gap(rem(8.))
                    .children(chips),
            )
            .children(views)
            .children(self.actions)
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
    fn palette_filters_and_steps_like_shell_jsx() {
        let sub = [
            SharedString::from("carbon-ui"),
            SharedString::from("Working"),
        ];
        assert!(palette_matches("Add dark theme tokens", &sub, "DARK"));
        assert!(palette_matches("Add dark theme tokens", &sub, "working"));
        assert!(!palette_matches("Settings", &[], "terminal"));
        assert_eq!(palette_step("down", 2, 3), Some(2));
        assert_eq!(palette_step("down", 0, 3), Some(1));
        assert_eq!(palette_step("up", 0, 3), Some(0));
        assert_eq!(palette_step("enter", 0, 3), None);
    }

    #[test]
    fn filters_follow_the_last_t3_css_rule() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, fg) = filter_colors(false, false);
        assert_eq!((hex(bg), hex(fg)), ("#f4f4f4".into(), "#525252".into()));
        let (bg, fg) = filter_colors(true, false);
        assert_eq!((hex(bg), hex(fg)), ("#e0e0e0".into(), "#161616".into()));
        let (bg, _) = filter_colors(true, true);
        assert_eq!(hex(bg), "#e8e8e8");
        let (bg, bar) = palette_row_colors(true);
        assert_eq!(
            (hex(bg), bar.map(hex)),
            ("#e0e0e0".into(), Some("#0f62fe".into()))
        );
    }
}
