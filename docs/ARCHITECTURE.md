# Architecture

u0ui draws IBM Carbon v11 on top of GPUI Kit 0.11. **Kit owns behavior** (focus,
keyboard, text editing, overlays, scrolling, lists, tables); **u0ui owns
appearance** (tokens, themes, geometry, motion). `AGENTS.md` is binding; this file
is the map and the recipe.

## Layers

```
$U0UI_DESIGN_DIR/tokens.json, src/icons/material.js   (outside the repo)
        │ cargo run -p u0ui-codegen  (--check: fail if stale)
        ▼
crates/u0ui/src/generated/tokens.rs, carbon_icons.rs  (committed, include!d)
                                              │
u0ui::tokens       ColorRole (276), CarbonTheme, TypeStyle → TypeSpec,
                   SPACING_01..13, SIZE_*, DENSITY_*, BORDER_RADIUS_* (TokenLength)
u0ui::theme        active theme (thread-local + ActiveTheme global), color(role),
                   color_in(theme, role), is_dark(), set_theme(cx, theme),
                   LayerLevel (contextual layer/field/border tokens), Kit bridge
u0ui::typography   bundled IBM Plex (Sans 300/400/400i/600, Mono 400/600),
                   font(family, weight) with CJK fallbacks, TypeStyled::type_style
u0ui::motion       Curve (6 Carbon beziers), Speed (6 durations), Motion policy,
                   animate / animate_color / chevron_rotation / open_progress / spin,
                   ReducedMotion + set_reduced_motion
u0ui::icons        Icon (Material Symbols Sharp, FILL 0/1, wght 400, opsz 24),
                   IconSize 16/20/24/32, CarbonIcon (reference names), Assets,
                   Kit default icons → Sharp
u0ui::density      Density::{Compact (default), Comfortable}, row/field heights
u0ui::components   one module per component; button is the reference
crates/gallery     the Gallery: shell, home, overview, pages (see below)
```

Each layer depends only on those above it. `u0ui::init(cx)` (after
`gpui_kit::init`) registers fonts, installs the theme and Kit bridge, and maps
Kit icons. Applications serve icons with
`gpui_kit::application().with_assets(u0ui::Assets::new())`.

### Key decisions

- **Context-free reads.** `theme::color`, `theme::current`, `density::current`
  read thread-locals (GPUI renders on one UI thread), so look/geometry helpers are
  plain functions that unit tests can call. Writes go through `set_theme`,
  `set_density`, `set_reduced_motion`, from event handlers only; each refreshes
  every window. **Render never mutates Kit's `Theme`.**
- **Kit bridge.** `set_theme` projects Carbon roles into
  `gpui_kit::component::Theme` (colors + tokens, IBM Plex, radius 0, Carbon motion,
  shadows on) so Kit inputs, menus, lists, tables and scrollbars look Carbon. Kit's
  `font_size` doubles as the window rem size, so it is **16px** (Carbon's rem);
  body text gets its 14px from `body-compact-01`.
- **rem = 16px.** Every Carbon length is a `TokenLength::Rem`; use them directly
  (`.p(tokens::SPACING_05)`, `.h(rems(...))`) so 80–200% zoom reflows.
- **Layers are explicit.** A component that paints contextual tokens takes a
  `LayerLevel` (default `One`); containers that paint `layer-0N` hand `level.next()`
  to their children.
- **Letter spacing** is recorded in `TypeSpec` but not applied: GPUI has no
  tracking control yet.
- **Motion on state change.** Compute the target value from state, then ask
  `motion::animate_color((id, "channel"), target, Motion::HOVER, window, cx)`;
  Base's keyed transition retargets mid-flight and stops requesting frames when
  done. Mark spatial motion `.spatial(true)` so reduced motion cuts it.

## Recipe: adding a component

Copy `crates/u0ui/src/components/button.rs`.

1. **Read the reference** (outside the repo, see `AGENTS.md`):
   `$U0UI_DESIGN_DIR/components/<Name>/README.md` and `preview.html`, the props in
   `$U0UI_DESIGN_DIR/components/index.d.ts`, and the `.cds--<name>` rules in
   `$U0UI_DESIGN_DIR/components/bundle.css` (or `$U0UI_DESIGN_DIR/src/custom.css`
   for `cu-*`).
2. **API** mirrors `index.d.ts`: `Name::new(id, …)` plus builders for every prop;
   enums for variants/sizes with `ALL` and `name()` (the d.ts string). Callers
   pass every user-facing string.
3. **Behavior from Kit**: wrap the Kit/Base element that already does the job
   (`gpui_kit::base::Button`, `component::input::InputState`, popovers, menus,
   lists…). Only write behavior Kit lacks, and say why in a comment.
4. **Look** as a pure function: `Look::of(kind, state) -> Look` reading
   `theme::color(ColorRole::…)`, with each value's CSS selector in a comment.
   **Geometry** as a pure function of size × density. Never hand-type a token
   value; never use a hex literal.
5. **Render** (`RenderOnce`): keep pointer state in `window.use_keyed_state`,
   compute the look, animate colors with `motion::animate_color`, draw the 2px
   `focus` ring (filled kinds add the 1px `focus-inset`), square corners.
6. **Tests** in the module: state colors match `bundle.css` for a theme, geometry
   matches the CSS, variant mapping. Keyboard/focus behavior that Kit owns needs
   no re-test unless you wire it differently.
7. Export it from `components/mod.rs` (and `prelude` if it is common).

## The gallery

`crates/gallery` follows md3.io's component pages, built from u0ui's own
components:

```
shell.rs      Route (Home | Overview(section) | Page), side nav (Sidebar), page
              frame (Breadcrumb, title, description, Overview/Specs Tabs),
              the ⌘K CommandDialog palette
top_bar.rs    G100 header in a theme::zone: NavigationMenu (Foundations,
              Components, Patterns) + ghost icon actions: search, theme and
              display (density, motion) Popovers
home.rs       the reference cover, live theme previews, the three ways in
overview.rs   search + section switcher over a reflowing grid of Card tiles,
              each with a live specimen (thumbnails.rs)
page.rs       Body: Playground (stage + Configuration panel), then Examples
controls.rs   config controls: Switch, ToggleGroup / Select, labels, blocks
specs.rs      Specs tabs: anatomy, measurements, state colors from the
              components' own Look/Geometry functions
registry.rs   PageDef list, groups, sections
capture.rs    --capture <dir>: render_to_image of each route, headless
```

Below 1056px the side nav folds behind the header's menu button and opens
over the content; the playground's configuration panel wraps under the stage
and the overview drops columns, so nothing scrolls sideways from 800px up.

## Recipe: adding a gallery page

One file plus one registry line.

1. Create `crates/gallery/src/pages/<name>.rs` exposing
   `pub fn build(window: &mut Window, cx: &mut App) -> AnyView`: a state entity
   whose `render` returns a `page::Body`:

   ```rust
   Body::new()
       .playground(live)          // the real component, driven by the state
       .hint("↑/↓ move, Enter selects")   // optional: what to try
       .config(config)            // controls::{toggle, choices, field}: every
                                  // variant, size, state and content option
                                  // in index.d.ts
       .example(controls::block("Reference preview", reference)) // preview.html
       .example(controls::block("Sizes", sizes))                  // more, optional
   ```

   Controls write back with `controls::setter(&entity, |this, v| this.x = v)`.
   `controls::choices` draws a content switcher for a short list and a Select
   for a long one.
2. Add `pub mod <name>;` to `pages/mod.rs` and one line to `registry::PAGES`:

   ```rust
   PageDef::new("tag-input", "TagInput", FORMS, Component::TagInput.summary(), pages::tag_input::build),
   ```

   with the stable id (the reference card, kebab-case), the title, the
   artifact's group (a constant, or the group string such as
   `"T3 · Composer"`; `T3 · `, `Ghost · ` and `Carbon · ` groups land under
   Patterns) and the README's first sentence from
   `crates/gallery/src/generated/descriptions.rs` (every reference component
   already has a variant). Optional on the same line:
   `.thumbnail(pages::tag_input::thumbnail)` (a small live specimen for the
   overview tile, at most ~15rem wide; the group's icon otherwise) and
   `.specs(specs::tag_input)` (a Specs tab).
3. Check it: `cargo run -p u0ui-gallery -- --page <id> --theme g100`, and at
   the 800×520 minimum window: no horizontal scrolling. Screenshot it with
   `cargo run -p u0ui-gallery -- --page <id> --capture <dir>`.

## Gates

`cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` (heavy cargo as `CARGO_BUILD_JOBS=3 nice -n 15`, shared
`CARGO_TARGET_DIR`). One concern per commit, prefixed by area.
