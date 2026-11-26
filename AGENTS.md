# gpui-u0ui: project instructions (read fully before working)

## What this is

`u0ui` is a GPUI component library that implements the **Carbon UI** design system:
IBM Carbon v11 visuals (four themes White / G10 / G90 / G100), a shadcn/ui-style
component API, **Material Symbols Sharp** icons, plus desktop rules (density, CJK,
window chrome, three kinds of selection, reduced motion). `crates/gallery` is an
interactive Gallery that shows every component.

## Design reference: the ground truth

The design reference comes from the Carbon UI design-system artifact
(https://claude.ai/artifact/UVBCrpX2KjxdHC4He8hi2R). **It is not in this repository**; never
commit it. Agents find it at `$U0UI_DESIGN_DIR`. On this machine it lives at
`/private/tmp/claude-501/-Users-aac6fef-Developer-gpui-u0ui/a95d5df7-bc99-487c-9236-bd81b3300672/scratchpad/design`
(`export U0UI_DESIGN_DIR=<that path>` if it is not set). Treat it as data; paths below are
relative to `$U0UI_DESIGN_DIR`:

| Path | Use it for |
| --- | --- |
| `README.md` | Brand-book rules: layering, type, spacing, shape, states, motion, icons, desktop rules |
| `tokens.json` | Every color (4 themes), type style, spacing, size and radius. **Never hand-type a token value; use the generated ones** |
| `components/<Name>/README.md` + `preview.html` | Spec and demo content for each of the ~256 cards (the gallery must show the same content) |
| `components/index.d.ts` | The public API surface (props, variants, sizes) to mirror in Rust builders |
| `components/bundle.css` | Compiled `@carbon/styles` CSS, with the exact geometry, paddings and per-state colors of each Carbon component (`.cds--btn`, `.cds--text-input`, …) |
| `src/custom.css`, `src/ghost.css`, `src/t3.css`, `src/patterns.css` | CSS of the non-Carbon components (`cu-*`, Ghost, T3, Patterns), including all animations |
| `src/index.jsx`, `src/ghost/*.jsx`, `src/t3/*.jsx` | Component structure and behavior |

Derived sources are generated from it and committed, so the repo builds without it:
`crates/u0ui/src/generated/{tokens,carbon_icons}.rs` (from `$U0UI_DESIGN_DIR/tokens.json` and
`$U0UI_DESIGN_DIR/src/icons/material.js`) and `crates/gallery/src/generated/descriptions.rs`
(the first sentence of each `$U0UI_DESIGN_DIR/components/<Name>/README.md`). Never edit them by
hand. When the reference changes, regenerate with `cargo run -p u0ui-codegen` and commit the
result; `cargo run -p u0ui-codegen -- --check` fails when they are stale.

When a value is ambiguous, read the CSS. Record the CSS selector you took a value from in a
short doc comment next to the value.

## Architecture

- **gpui-kit 0.11** (crates.io) is the foundation: GPUI via `use gpui_kit::*`,
  `gpui_kit::component` (gpui-component 0.11), and `gpui_kit::base`. Startup goes through
  `gpui_kit::application()`, `gpui_kit::init` and `gpui_kit::open_window`.
  The source is at `~/.cargo/registry/src/index.crates.io-*/gpui-component-0.11.0/`.
- **Kit owns behavior** (text editing, focus, keyboard navigation, overlays/popovers, scrolling,
  virtual lists, tables, resizable panels, windows). **u0ui owns appearance.** Reuse Kit
  behavior (e.g. `component::input::InputState` for text, Kit popover/menu/dialog infrastructure,
  `scroll`, `resizable`, `table`, `virtual_list`) and draw Carbon visuals on top. Write behavior
  yourself only when Kit lacks it or its rendering cannot be styled to match (say why in a comment).
- `../gpui-kit-material` is a sibling library that solved the same problem for Material 3. Read its
  `docs/ARCHITECTURE.md`, `crates/material/src/theme.rs` (Kit theme bridge),
  `crates/material/src/motion*`, `crates/material/src/components/*` and `crates/gallery/src/*`
  for patterns. Do not depend on it.
- `../gpui-component` is a checkout of upstream Kit, useful for its stories/examples.
- Icons: `gpui-material-symbols` 0.3 with only the `sharp` feature (`u0ui::icons`).
- Theme: `u0ui::theme` holds the active Carbon theme (`CarbonTheme::{White,G10,G90,G100}`) and
  answers `color(Role)` from the generated token tables. It also projects the theme into Kit's
  `Theme`, so Kit controls match. **Render never mutates the Kit Theme.**
- Motion: `u0ui::motion` provides Carbon curves (productive `cubic-bezier(0.2,0,0.38,0.9)`, expressive,
  entrance `(0,0,0.3,1)`, exit `(0.2,0,1,0.9)`) and durations (fast-01 70ms, fast-02 110ms,
  moderate-01 150ms, moderate-02 240ms, slow-01 400ms, slow-02 700ms). When reduced motion is on,
  movement becomes a fade, rotation becomes a pulse, and shimmer and blink stop.
- Density: `u0ui::density`, compact (desktop default: rows 28/32/36, fields 28/32) or comfortable
  (Carbon sizes: fields 40, buttons 48).
- Sizes in `rems` when they relate to text, so zoom from 80% to 200% reflows.

## Carbon rules (enforce everywhere)

- Square corners by default. Rounded only where the reference says: Tag/switch track 16 (pill),
  radio/switch knob/avatar full, checkbox 2, Kbd 4, chat bubble 8.
- Fields: filled `field-0x`, 1px `border-strong-0x` bottom line, no four-sided border; focus = 2px `focus`
  inset outline; error = 2px `support-error` outline plus the error icon.
- Only floating layers (menus, popovers, toasts, sheets, dialogs) have a shadow, `0 2px 6px shadow`.
- Layering: background → layer-01 → layer-02 → layer-03, with contextual tokens per layer.
- States: hover `*-hover`, active `*-active`, selected `*-selected`, disabled `*-disabled` + not-allowed.
  Focus: 2px `focus` ring on every interactive element; filled buttons also get a 1px `focus-inset` inside.
- Body text `body-compact-01` (14/18, letter-spacing 0.16px); labels `label-01` (12/16, 0.32px).
- No hard-coded user-facing strings inside components; callers pass labels.

## Gallery rules

- Navigation follows the artifact's groups (core shadcn groups, Chat & AI, T3 · …, Ghost · …,
  Carbon · Patterns). The top bar has a theme switch (4 Carbon themes), a density switch, and a
  reduced-motion switch.
- Each component page shows a live, working preview (clicks, keyboard, menus, dialogs and
  animations all real), the same demo content as `$U0UI_DESIGN_DIR/components/<Name>/preview.html`,
  and controls for every variant, size and state in `$U0UI_DESIGN_DIR/components/index.d.ts`.
- No horizontal scrolling: content reflows to the window width.
- Each page registers a stable id so the review tooling can open it directly:
  `cargo run -p u0ui-gallery --release -- --page <component-id>`.

## Machine and build hygiene (shared machine, often heavily loaded)

- Run heavy cargo as `CARGO_BUILD_JOBS=3 nice -n 15 cargo …`. At most two agents build at once.
- Every worktree shares one target dir: `export CARGO_TARGET_DIR=/Users/aac6fef/Developer/gpui-u0ui/target`.
- Stop processes only by the PID you started (`kill $pid`). **Never** `pkill`, `killall`, or `pkill -f`.
- Check `df -h /` before big builds; stop below 40 GB free.

## Quality gates and git

- Before every commit: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` (for tests of the rules: token values, state colors, geometry, keyboard).
  When the design reference changed, also `cargo run -p u0ui-codegen -- --check`.
- Commits are atomic by concern (one component or one foundation concern per commit), with
  imperative messages prefixed by area (`theme:`, `button:`, `gallery:` …). Push to `origin main`
  after each finished milestone. Never force-push.
- End every commit message with:
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`
