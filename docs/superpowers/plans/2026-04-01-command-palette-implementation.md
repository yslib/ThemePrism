# Command Palette Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first TUI command palette opened by `Ctrl-P` that live-filters command items and executes existing product commands without changing the `event -> intent -> update -> render` architecture.

**Architecture:** Implement the palette as a standard overlay surface backed by one `commands` provider, a replaceable matcher, and a data-driven command execution bridge. Keep palette state in shared Rust state, route opening and interaction through existing event and intent layers, and execute selected commands by translating `CommandId` into the same existing save/load/export/help/config/fullscreen/reset/quit paths already used elsewhere.

**Tech Stack:** Rust, ratatui, crossterm, existing `Intent`/`Effect` flow, shared `AppState`, Fluent i18n, inline Rust unit tests

---

## Scope Notes

This plan covers one feature subsystem: the first command palette for the TUI.

Included:

- command metadata and command search matching
- palette state and modal ownership
- `Ctrl-P` input routing and palette interaction
- TUI overlay rendering for the palette
- execution of a small command set through existing intents
- i18n labels and help text updates for the palette

Explicitly not included:

- multiple palette providers in behavior
- provider prefix syntax such as `>` or `@`
- file search, symbol search, or navigation search
- GUI command palette
- command arguments or multi-step command workflows
- replacing existing direct shortcuts

## File Structure and Responsibilities

### Create

- `src/app/command_palette.rs`
  - command metadata, command provider output, palette item ranking, and matcher logic
- `src/app/update/command_palette.rs`
  - palette open/close, query edits, selection movement, and command execution dispatch

### Modify

- `src/app/mod.rs`
  - register the new `command_palette` module
- `src/app/actions.rs`
  - add `OpenCommandPalette` keybinding metadata and help/menu labels
- `src/app/intent.rs`
  - add palette lifecycle, query, selection, and execution intents
- `src/app/state.rs`
  - add palette state to `UiState`
- `src/app/interaction/tree.rs`
  - add `SurfaceId::CommandPalette` and modal visibility wiring
- `src/app/interaction/mod.rs`
  - add palette surface labels
- `src/app/interaction/routing.rs`
  - route palette-focused actions and add a main-window open-palette action path
- `src/app/update/mod.rs`
  - dispatch palette intents into the new update module
- `src/app/update/modals.rs`
  - add palette open/close helpers using the same modal-owner mechanics as other overlays
- `src/app/view/overlays.rs`
  - build the palette overlay surface from shared palette state
- `src/app/view/types.rs`
  - keep using `OverlayView::Surface`; only extend data structures if needed for palette rows
- `src/app/view/window.rs`
  - mount the palette overlay into the existing overlay stack
- `src/platform/tui/event_adapter.rs`
  - map `Ctrl-P` and palette-local keys to palette intents through `UiAction`
- `src/platform/tui/renderer/overlays.rs`
  - render the palette surface rows with selected-result emphasis
- `src/i18n/mod.rs`
  - add palette labels and help strings
- `locales/en-US/ui.ftl`
  - English palette strings
- `locales/zh-CN/ui.ftl`
  - Chinese palette strings

### Primary Test Entry Points

- `src/app/command_palette.rs`
- `src/app/update/tests.rs`
- `src/app/interaction/tests.rs`
- `src/platform/tui/event_adapter.rs`
- `src/platform/tui/renderer/tests.rs`
- `src/i18n/mod.rs`

## Task 1: Add command metadata, commands provider, and matcher

**Files:**
- Create: `src/app/command_palette.rs`
- Modify: `src/app/mod.rs`
- Test: `src/app/command_palette.rs`

- [ ] **Step 1: Create the `command_palette` module and write failing metadata and ranking tests**

```rust
#[test]
fn command_provider_exposes_expected_core_commands() {
    let state = AppState::new().unwrap();
    let items = command_items(&state);

    assert!(items.iter().any(|item| item.id == CommandId::SaveProject));
    assert!(items.iter().any(|item| item.id == CommandId::OpenConfig));
    assert!(items.iter().any(|item| item.id == CommandId::Quit));
}

#[test]
fn empty_query_keeps_context_commands_ahead_of_global_commands() {
    let mut state = AppState::new().unwrap();
    state.set_active_panel(PanelId::Preview);

    let ranked = filter_commands(&state, "");
    let preview_index = ranked.iter().position(|item| item.id == CommandId::ToggleFullscreen).unwrap();
    let quit_index = ranked.iter().position(|item| item.id == CommandId::Quit).unwrap();

    assert!(preview_index < quit_index);
}

#[test]
fn query_matching_is_case_insensitive_and_substring_friendly() {
    let state = AppState::new().unwrap();
    let ranked = filter_commands(&state, "expo");

    assert_eq!(ranked.first().map(|item| item.id), Some(CommandId::ExportTheme));
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test command_provider_exposes_expected_core_commands -- --exact
env -u http_proxy -u https_proxy cargo test empty_query_keeps_context_commands_ahead_of_global_commands -- --exact
env -u http_proxy -u https_proxy cargo test query_matching_is_case_insensitive_and_substring_friendly -- --exact
```

Expected:

- missing module or failing assertions because the command provider and matcher do not exist yet

- [ ] **Step 3: Implement minimal command metadata and matching**

Implementation target:

- add `CommandId`
- add `CommandItem`
- add a first provider function such as `command_items(state)`
- add a first matcher function such as `filter_commands(state, query)`

```rust
pub enum CommandId {
    SaveProject,
    LoadProject,
    ExportTheme,
    Reset,
    OpenConfig,
    OpenHelp,
    ToggleFullscreen,
    Quit,
}

pub struct CommandItem {
    pub id: CommandId,
    pub title: &'static str,
    pub keywords: &'static [&'static str],
    pub context_score: u16,
    pub enabled: bool,
}
```

Keep this module focused on metadata, ranking, and matching only. Do not let it mutate application state or emit effects.

- [ ] **Step 4: Add one light fuzzy-friendly scoring pass**

Implement a simple ranking strategy:

- exact or substring title matches first
- keyword matches next
- higher context score next
- stable declaration order as final tiebreaker

Keep it intentionally small. Do not add a full fuzzy library or advanced algorithm in this first pass.

- [ ] **Step 5: Run targeted tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test command_provider_exposes_expected_core_commands -- --exact
env -u http_proxy -u https_proxy cargo test empty_query_keeps_context_commands_ahead_of_global_commands -- --exact
env -u http_proxy -u https_proxy cargo test query_matching_is_case_insensitive_and_substring_friendly -- --exact
```

Expected:

- all PASS

- [ ] **Step 6: Commit**

```bash
git add src/app/mod.rs src/app/command_palette.rs
git commit -m "feat: add command palette metadata and matcher"
```

## Task 2: Add palette state, intents, and update-driven command execution

**Files:**
- Create: `src/app/update/command_palette.rs`
- Modify: `src/app/intent.rs`
- Modify: `src/app/state.rs`
- Modify: `src/app/update/mod.rs`
- Modify: `src/app/update/modals.rs`
- Test: `src/app/update/tests.rs`

- [ ] **Step 1: Write failing update tests for palette lifecycle and execution**

```rust
#[test]
fn open_command_palette_initializes_query_and_selection() {
    let mut state = AppState::new().unwrap();

    update(&mut state, Intent::OpenCommandPaletteRequested);

    let palette = state.ui.command_palette.as_ref().unwrap();
    assert!(palette.query.is_empty());
    assert_eq!(palette.selected, 0);
}

#[test]
fn closing_command_palette_restores_prior_focus() {
    let mut state = AppState::new().unwrap();
    state.set_active_panel(PanelId::Preview);
    state.ui.interaction.focus_panel(PanelId::Preview);

    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(&mut state, Intent::CloseCommandPaletteRequested);

    assert_eq!(effective_focus_surface(&state), SurfaceId::PreviewPanel);
}

#[test]
fn running_selected_palette_command_dispatches_existing_command_path() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(&mut state, Intent::SetCommandPaletteQuery("expo".into()));

    let effects = update(&mut state, Intent::RunSelectedCommandPaletteItem);

    assert!(effects.iter().any(|effect| matches!(effect, Effect::ExportTheme { .. })));
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test open_command_palette_initializes_query_and_selection -- --exact
env -u http_proxy -u https_proxy cargo test closing_command_palette_restores_prior_focus -- --exact
env -u http_proxy -u https_proxy cargo test running_selected_palette_command_dispatches_existing_command_path -- --exact
```

Expected:

- compile failures or failing assertions because palette state and intents do not exist yet

- [ ] **Step 3: Add palette state to `UiState` and palette intents to `Intent`**

Add a focused palette state, for example:

```rust
pub struct CommandPaletteState {
    pub query: String,
    pub selected: usize,
}
```

Add palette intents for:

- open
- close
- set query
- append query char
- backspace query
- clear query
- move selection
- run selected item

Keep query mutation and selection clamping in `update`, not in the renderer.

- [ ] **Step 4: Implement palette open/close helpers in `update/modals.rs`**

Follow the same pattern used by config/help/numeric input:

- palette owns a modal surface
- opening palette closes incompatible transient overlays first
- closing palette removes its modal owner and clears palette UI state

The new helper should reuse existing modal stack behavior instead of inventing palette-specific focus restoration.

- [ ] **Step 5: Implement `update/command_palette.rs`**

Implement:

- opening the palette
- query edits
- selection movement
- execution of the selected `CommandId`

Execution must translate back into existing product operations:

- save uses the same save path as direct shortcut save
- export uses the same export path as direct shortcut export
- config/help/fullscreen/reset/quit use the same existing intents and effects

- [ ] **Step 6: Run targeted update tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test open_command_palette_initializes_query_and_selection -- --exact
env -u http_proxy -u https_proxy cargo test closing_command_palette_restores_prior_focus -- --exact
env -u http_proxy -u https_proxy cargo test running_selected_palette_command_dispatches_existing_command_path -- --exact
```

Expected:

- all PASS

- [ ] **Step 7: Run the broader update suite**

Run:

```bash
env -u http_proxy -u https_proxy cargo test app::update::
```

Expected:

- update regressions still PASS

- [ ] **Step 8: Commit**

```bash
git add src/app/intent.rs src/app/state.rs src/app/update/mod.rs src/app/update/modals.rs src/app/update/command_palette.rs src/app/update/tests.rs
git commit -m "feat: add command palette state and execution flow"
```

## Task 3: Integrate palette modal routing into interaction and TUI event handling

**Files:**
- Modify: `src/app/actions.rs`
- Modify: `src/app/interaction/tree.rs`
- Modify: `src/app/interaction/mod.rs`
- Modify: `src/app/interaction/routing.rs`
- Modify: `src/platform/tui/event_adapter.rs`
- Test: `src/app/interaction/tests.rs`
- Test: `src/platform/tui/event_adapter.rs`

- [ ] **Step 1: Write failing interaction and event tests**

```rust
#[test]
fn command_palette_is_visible_as_a_modal_surface_when_open() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);

    let tree = build_interaction_tree(&state);
    assert!(tree.is_visible(SurfaceId::CommandPalette));
}

#[test]
fn ctrl_p_opens_command_palette() {
    let state = AppState::new().unwrap();
    let intents = TuiEventAdapter.map_event(&state, ctrl_key('p'));

    assert!(matches!(intents.as_slice(), [Intent::OpenCommandPaletteRequested]));
}

#[test]
fn palette_text_input_consumes_printable_keys_without_leaking_workspace_shortcuts() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);

    let intents = TuiEventAdapter.map_event(&state, key(KeyCode::Char('e')));

    assert!(matches!(intents.as_slice(), [Intent::AppendCommandPaletteQuery('e')]));
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test command_palette_is_visible_as_a_modal_surface_when_open -- --exact
env -u http_proxy -u https_proxy cargo test ctrl_p_opens_command_palette -- --exact
env -u http_proxy -u https_proxy cargo test palette_text_input_consumes_printable_keys_without_leaking_workspace_shortcuts -- --exact
```

Expected:

- compile failures or failing assertions because `SurfaceId::CommandPalette` and palette input routing do not exist yet

- [ ] **Step 3: Add palette to action and interaction metadata**

Implement:

- `BoundAction::OpenCommandPalette`
- a matching `UiAction`
- `SurfaceId::CommandPalette`
- visibility and modal parenting in the interaction tree
- surface label text in `interaction/mod.rs`

Do not treat the palette as a special routing bypass. It should behave like a normal focused modal surface.

- [ ] **Step 4: Route palette-local keys in `event_adapter` and `routing`**

Palette-local behavior should include:

- printable characters append query
- `Backspace` deletes
- `Delete` clears
- `Up` / `Down` move selection
- `Enter` applies the current selection
- `Esc` closes the palette

`Ctrl-P` should open the palette from workspace context.

- [ ] **Step 5: Run targeted interaction and event tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test command_palette_is_visible_as_a_modal_surface_when_open -- --exact
env -u http_proxy -u https_proxy cargo test ctrl_p_opens_command_palette -- --exact
env -u http_proxy -u https_proxy cargo test palette_text_input_consumes_printable_keys_without_leaking_workspace_shortcuts -- --exact
```

Expected:

- all PASS

- [ ] **Step 6: Run the broader TUI input and interaction suites**

Run:

```bash
env -u http_proxy -u https_proxy cargo test app::interaction::
env -u http_proxy -u https_proxy cargo test platform::tui::event_adapter::
```

Expected:

- existing interaction and event adapter tests still PASS

- [ ] **Step 7: Commit**

```bash
git add src/app/actions.rs src/app/interaction/tree.rs src/app/interaction/mod.rs src/app/interaction/routing.rs src/app/interaction/tests.rs src/platform/tui/event_adapter.rs
git commit -m "feat: wire command palette into interaction and key routing"
```

## Task 4: Render the palette overlay, add i18n strings, and finish regressions

**Files:**
- Modify: `src/app/view/overlays.rs`
- Modify: `src/app/view/window.rs`
- Modify: `src/platform/tui/renderer/overlays.rs`
- Modify: `src/i18n/mod.rs`
- Modify: `locales/en-US/ui.ftl`
- Modify: `locales/zh-CN/ui.ftl`
- Test: `src/platform/tui/renderer/tests.rs`
- Test: `src/i18n/mod.rs`

- [ ] **Step 1: Write failing view and renderer tests**

```rust
#[test]
fn command_palette_overlay_appears_when_palette_is_open() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);

    let tree = build_view(&state);
    assert!(tree.overlays.iter().any(|overlay| is_command_palette_overlay(overlay)));
}

#[test]
fn command_palette_selected_result_is_highlighted() {
    let mut state = AppState::new().unwrap();
    update(&mut state, Intent::OpenCommandPaletteRequested);
    update(&mut state, Intent::SetCommandPaletteQuery("expo".into()));

    let tree = build_view(&state);
    let palette = extract_command_palette_overlay(&tree);
    assert!(palette_has_selected_row(palette));
}

#[test]
fn command_palette_strings_exist_in_both_locales() {
    assert_ne!(text(EditorLocale::EnUs, UiText::CommandPaletteTitle), "");
    assert_ne!(text(EditorLocale::ZhCn, UiText::CommandPaletteTitle), "");
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test command_palette_overlay_appears_when_palette_is_open -- --exact
env -u http_proxy -u https_proxy cargo test command_palette_selected_result_is_highlighted -- --exact
env -u http_proxy -u https_proxy cargo test command_palette_strings_exist_in_both_locales -- --exact
```

Expected:

- failing assertions because the overlay and i18n keys do not exist yet

- [ ] **Step 3: Build the overlay view**

Implement a palette overlay builder in `src/app/view/overlays.rs` that:

- creates a standard `OverlayView::Surface`
- shows a query line
- shows filtered rows with a clear selected row
- shows a clean empty-state when no commands match

Keep the body representation simple. If the existing `SurfaceBody::Lines` can carry the display cleanly, use it. Do not add a new custom overlay kind unless the generic surface model genuinely cannot express the palette.

- [ ] **Step 4: Add i18n strings and help/menu metadata**

Add palette UI strings for:

- palette title
- query prompt or input label
- empty results text
- footer hint text

Update action/help metadata so `Ctrl-P` appears in the shortcut help and, if appropriate, in the menu bar action strip.

- [ ] **Step 5: Add renderer-local emphasis for the selected row**

Use the existing overlay renderer path. Only add renderer logic that is necessary to make the selected palette row visually clear. Avoid introducing palette-specific layout code into unrelated renderer modules.

- [ ] **Step 6: Run targeted tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test command_palette_overlay_appears_when_palette_is_open -- --exact
env -u http_proxy -u https_proxy cargo test command_palette_selected_result_is_highlighted -- --exact
env -u http_proxy -u https_proxy cargo test command_palette_strings_exist_in_both_locales -- --exact
```

Expected:

- all PASS

- [ ] **Step 7: Run final verification**

Run:

```bash
cargo fmt
env -u http_proxy -u https_proxy cargo check
env -u http_proxy -u https_proxy cargo test
```

Expected:

- formatting applies cleanly
- `cargo check` passes
- full test suite passes

- [ ] **Step 8: Commit**

```bash
git add src/app/view/overlays.rs src/app/view/window.rs src/platform/tui/renderer/overlays.rs src/platform/tui/renderer/tests.rs src/i18n/mod.rs locales/en-US/ui.ftl locales/zh-CN/ui.ftl
git commit -m "feat: render command palette overlay"
```

## Final Notes for Implementers

- Keep palette execution data-driven. Do not attach closures or direct side effects to palette rows.
- Do not fold command metadata into keybinding metadata. Commands and bindings are different concerns.
- Do not let the palette become a second navigation system. Hint navigation and command palette remain separate.
- Prefer extending existing overlay and modal mechanics over adding palette-specific UI infrastructure.
- If a later task makes the matcher or provider structure look heavier than the feature needs, simplify toward the spec rather than adding more abstraction.
