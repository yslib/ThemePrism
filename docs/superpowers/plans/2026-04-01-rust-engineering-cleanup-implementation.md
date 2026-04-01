# Rust Engineering Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce duplicated Rust UI metadata and shrink the largest Rust/TUI implementation files without changing the existing `event -> intent -> update -> render` architecture or current product behavior.

**Architecture:** Keep the current pipeline intact and treat this as a structural cleanup pass. First consolidate repeated stable metadata into single-source specs, then split the TUI renderer by rendering concern, then split `update` by intent family while preserving one public `update(state, intent)` entrypoint. The cleanup stays inside the Rust shared layer and TUI; AppKit/Objective-C remains untouched.

**Tech Stack:** Rust, ratatui, crossterm, existing `CoreSession` / `Intent` / `Effect` flow, Fluent i18n, inline Rust unit tests

---

## Scope Notes

This plan covers one subsystem cluster: Rust-side engineering cleanup for shared UI metadata, TUI rendering, and TUI/shared update organization.

Included:

- single-source metadata for panels, workspace tabs, and preview modes
- TUI renderer split into focused modules
- `update` split into focused modules behind the same public entrypoint
- cleanup of repeated hardcoded mappings exposed by the first three tasks

Explicitly not included:

- AppKit/Objective-C cleanup
- interaction-model redesign
- keymap redesign
- GUI architecture changes
- product behavior or UX semantic changes

## File Structure and Responsibilities

### Create

- `src/app/ui_meta.rs`
  - single-source specs for `PanelId`, `WorkspaceTab`, and `PreviewMode`
- `src/platform/tui/renderer/mod.rs`
  - public renderer entrypoint and re-exports
- `src/platform/tui/renderer/chrome.rs`
  - menu bar, tab bar, status bar, fullscreen shell rendering
- `src/platform/tui/renderer/panels.rs`
  - panel blocks, panel tabs, panel title and border styling
- `src/platform/tui/renderer/content.rs`
  - document, form, selection-list, swatch-list rendering
- `src/platform/tui/renderer/overlays.rs`
  - picker and generic surface overlay rendering
- `src/platform/tui/renderer/style.rs`
  - shared renderer-local style helpers for hint state and chrome emphasis
- `src/platform/tui/renderer/tests.rs`
  - renderer-focused unit tests
- `src/app/update/mod.rs`
  - public `update(state, intent)` entrypoint and dispatch glue
- `src/app/update/navigation.rs`
  - focus, panel switching, tab switching, hint navigation, fullscreen routing
- `src/app/update/preview.rs`
  - preview mode changes, preview capture/runtime state transitions
- `src/app/update/modals.rs`
  - help/config/picker/numeric-editor open/close flows and owner restoration
- `src/app/update/config.rs`
  - editor/project/export config field movement and editing helpers
- `src/app/update/project.rs`
  - save/load/export/reset flows and project-level mutations
- `src/app/update/inspector.rs`
  - rule editing, inspector field movement, fixed color/reference adjustments
- `src/app/update/text_input.rs`
  - generic text/numeric input state transitions and submission helpers
- `src/app/update/tests.rs`
  - update-entrypoint and family-specific regression tests

### Modify

- `src/app/mod.rs`
  - register `ui_meta` and convert `update` to directory module
- `src/app/hint_nav.rs`
  - replace local tab/panel metadata duplication with `ui_meta`
- `src/app/view/window.rs`
  - read panel/tab labels and hint eligibility from `ui_meta`
- `src/app/view/theme_tab.rs`
  - use preview mode specs instead of local repeated mappings
- `src/app/view/layout.rs`
  - use panel metadata where layout-visible panel facts are currently duplicated
- `src/i18n/mod.rs`
  - panel/workspace/preview label helpers should consume shared specs where applicable
- `src/preview.rs`
  - expose or consume preview mode metadata in one place
- `src/platform/tui/mod.rs`
  - point to new renderer module directory
- `src/platform/tui/runtime.rs`
  - continue importing `TuiRenderer` and renderer helper exports from the new module boundary
- `src/platform/tui/view_metrics.rs`
  - continue consuming `panel_body_area` after the renderer split

### Delete / Replace

- `src/platform/tui/renderer.rs`
  - replaced by `src/platform/tui/renderer/`
- `src/app/update.rs`
  - replaced by `src/app/update/`

### Primary Test Entry Points

- `src/app/ui_meta.rs`
- `src/app/hint_nav.rs`
- `src/i18n/mod.rs`
- `src/platform/tui/renderer/tests.rs`
- `src/app/update/tests.rs`
- `src/platform/tui/event_adapter.rs`

## Task 1: Consolidate stable UI metadata into a single source

**Files:**
- Create: `src/app/ui_meta.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/hint_nav.rs`
- Modify: `src/app/view/window.rs`
- Modify: `src/app/view/theme_tab.rs`
- Modify: `src/app/view/layout.rs`
- Modify: `src/i18n/mod.rs`
- Modify: `src/preview.rs`
- Test: `src/app/ui_meta.rs`
- Test: `src/app/hint_nav.rs`
- Test: `src/i18n/mod.rs`

- [ ] **Step 1: Create a skeletal `ui_meta` module and write failing metadata coverage tests**

```rust
const ALL_PANELS: [PanelId; 11] = [
    PanelId::Tokens,
    PanelId::Params,
    PanelId::Preview,
    PanelId::Palette,
    PanelId::ResolvedPrimary,
    PanelId::ResolvedSecondary,
    PanelId::Inspector,
    PanelId::InteractionInspector,
    PanelId::ProjectConfig,
    PanelId::ExportTargets,
    PanelId::EditorPreferences,
];

#[test]
fn panel_specs_cover_every_panel_id() {
    for panel in ALL_PANELS {
        assert!(panel_spec(panel).is_some(), "missing panel spec for {:?}", panel);
    }
}

#[test]
fn preview_mode_specs_cover_every_preview_mode() {
    for mode in PreviewMode::ALL {
        assert!(preview_mode_spec(mode).is_some(), "missing preview mode spec for {:?}", mode);
    }
}

#[test]
fn workspace_tab_specs_cover_every_workspace_tab() {
    for tab in WorkspaceTab::ALL {
        assert!(workspace_tab_spec(tab).is_some(), "missing tab spec for {:?}", tab);
    }
}

pub fn panel_spec(_: PanelId) -> Option<PanelSpec> {
    None
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test panel_specs_cover_every_panel_id -- --exact
env -u http_proxy -u https_proxy cargo test preview_mode_specs_cover_every_preview_mode -- --exact
env -u http_proxy -u https_proxy cargo test workspace_tab_specs_cover_every_workspace_tab -- --exact
```

Expected:

- failing assertions from intentionally incomplete lookup tables, not compile errors

- [ ] **Step 3: Implement focused metadata specs**

Implementation target:

- add `PanelSpec`, `WorkspaceTabSpec`, and `PreviewModeSpec`
- add `panel_spec`, `workspace_tab_spec`, and `preview_mode_spec`
- keep the tables explicit and readable
- do not hide behavior in macros

```rust
pub struct PanelSpec {
    pub id: PanelId,
    pub ui_text: UiText,
    pub workspace_tab: WorkspaceTab,
}

pub struct WorkspaceTabSpec {
    pub id: WorkspaceTab,
    pub ui_text: UiText,
    pub hint_navigation: bool,
}

pub struct PreviewModeSpec {
    pub id: PreviewMode,
    pub ui_text: UiText,
    pub hint_navigation: bool,
}
```

Keep dynamic hint target selection layout-driven in `hint_nav.rs`. The new specs should hold stable reused facts only:

- panel-to-workspace membership
- stable UI label keys
- static hint-navigation eligibility for workspace tabs and preview modes

They should not encode current visibility, current layout, or currently active targets.

- [ ] **Step 4: Move duplicated consumers onto the shared metadata**

Replace repeated mappings in:

- `hint_nav.rs`
- `window.rs`
- `theme_tab.rs`
- `i18n/mod.rs`

The goal is not zero `match` statements. The goal is to stop redefining the same stable facts in multiple files.

- [ ] **Step 5: Run targeted tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test panel_specs_cover_every_panel_id -- --exact
env -u http_proxy -u https_proxy cargo test preview_mode_specs_cover_every_preview_mode -- --exact
env -u http_proxy -u https_proxy cargo test workspace_tab_specs_cover_every_workspace_tab -- --exact
env -u http_proxy -u https_proxy cargo test main_window_hint_targets_flatten_visible_panels_and_tabs -- --exact
env -u http_proxy -u https_proxy cargo test every_ui_text_key_exists_in_both_locales -- --exact
```

Expected:

- all PASS

- [ ] **Step 6: Run the broader metadata-adjacent suite**

Run:

```bash
env -u http_proxy -u https_proxy cargo test hint_nav::
env -u http_proxy -u https_proxy cargo test i18n::
```

Expected:

- all metadata/hint/i18n tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/app/mod.rs src/app/ui_meta.rs src/app/hint_nav.rs src/app/view/window.rs src/app/view/theme_tab.rs src/app/view/layout.rs src/i18n/mod.rs src/preview.rs
git commit -m "refactor: centralize shared UI metadata"
```

## Task 2: Split the TUI renderer by rendering concern

**Files:**
- Create: `src/platform/tui/renderer/mod.rs`
- Create: `src/platform/tui/renderer/chrome.rs`
- Create: `src/platform/tui/renderer/panels.rs`
- Create: `src/platform/tui/renderer/content.rs`
- Create: `src/platform/tui/renderer/overlays.rs`
- Create: `src/platform/tui/renderer/style.rs`
- Create: `src/platform/tui/renderer/tests.rs`
- Modify: `src/platform/tui/mod.rs`
- Modify: `src/platform/tui/runtime.rs`
- Modify: `src/platform/tui/view_metrics.rs`
- Delete: `src/platform/tui/renderer.rs`
- Test: `src/platform/tui/renderer/tests.rs`

- [ ] **Step 1: Add focused renderer regression tests before the split**

Add or move tests that lock current behavior:

```rust
#[test]
fn hint_active_panel_border_uses_selection_color() { /* existing behavior */ }

#[test]
fn hint_active_workspace_tabs_promote_target_labels() { /* existing behavior */ }

#[test]
fn vertical_scroll_is_clamped_to_last_visible_line() { /* existing behavior */ }
```

Add one missing shell-level regression:

```rust
#[test]
fn non_hint_tabs_dim_when_hint_navigation_is_active() {
    // Build a tab row with a mix of hinted and non-hinted tabs.
    // Assert that non-target tabs stay visibly de-emphasized.
}
```

- [ ] **Step 2: Run the renderer-focused tests to verify the new one fails**

Run:

```bash
env -u http_proxy -u https_proxy cargo test non_hint_tabs_dim_when_hint_navigation_is_active -- --exact
```

Expected:

- FAIL because the new regression is not implemented or not test-covered yet

- [ ] **Step 3: Move the renderer into a directory module with the same public API**

Keep:

```rust
impl TuiRenderer {
    pub fn present(self, frame: &mut Frame, tree: &ViewTree) { ... }
}
```

Split internals by concern only. Do not move business logic out of view/render.

- [ ] **Step 4: Implement the minimum code needed so the moved tests pass unchanged**

Implementation target:

- `chrome.rs` owns menu/tab/status shell rendering
- `panels.rs` owns panel block/tab/title rendering
- `content.rs` owns documents/forms/lists/swatch content
- `overlays.rs` owns picker/surface overlay drawing
- `style.rs` owns renderer-local style helpers used across the split
- `mod.rs` owns orchestration and shared private helpers
- preserve `panel_body_area` and `max_document_scroll` as renderer exports consumed by other TUI modules

- [ ] **Step 5: Run renderer tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test platform::tui::renderer::
```

Expected:

- all renderer tests PASS

- [ ] **Step 6: Run full TUI-adjacent verification**

Run:

```bash
env -u http_proxy -u https_proxy cargo test platform::tui::
env -u http_proxy -u https_proxy cargo check
```

Expected:

- PASS

- [ ] **Step 7: Commit**

```bash
git add src/platform/tui/mod.rs src/platform/tui/runtime.rs src/platform/tui/view_metrics.rs src/platform/tui/renderer
git commit -m "refactor: split tui renderer by concern"
```

## Task 3: Split `update` by intent family behind one public entrypoint

**Files:**
- Create: `src/app/update/mod.rs`
- Create: `src/app/update/navigation.rs`
- Create: `src/app/update/preview.rs`
- Create: `src/app/update/modals.rs`
- Create: `src/app/update/config.rs`
- Create: `src/app/update/project.rs`
- Create: `src/app/update/inspector.rs`
- Create: `src/app/update/text_input.rs`
- Create: `src/app/update/tests.rs`
- Modify: `src/app/mod.rs`
- Delete: `src/app/update.rs`
- Test: `src/app/update/tests.rs`
- Test: `src/platform/tui/event_adapter.rs`

- [ ] **Step 1: Add failing dispatch-regression tests**

Add one regression per major family so the public entrypoint stays stable:

```rust
#[test]
fn update_dispatches_navigation_intents_without_affecting_preview_state() {
    let mut state = AppState::new().expect("state");
    let preview_mode_before = state.preview.active_mode;

    update(&mut state, Intent::FocusPanelByNumber(2));

    assert_eq!(state.active_panel(), PanelId::Params);
    assert_eq!(state.preview.active_mode, preview_mode_before);
}

#[test]
fn update_dispatches_preview_mode_changes_without_affecting_config_focus() {
    let mut state = AppState::new().expect("state");
    state.ui.active_tab = WorkspaceTab::Project;

    update(&mut state, Intent::SetPreviewMode(PreviewMode::Shell));

    assert_eq!(state.preview.active_mode, PreviewMode::Shell);
    assert_eq!(state.ui.active_tab, WorkspaceTab::Project);
}

#[test]
fn update_dispatches_modal_intents_without_losing_owner_focus() {
    let mut state = AppState::new().expect("state");

    update(&mut state, Intent::ToggleShortcutHelpRequested);
    assert!(state.ui.shortcut_help_open);

    update(&mut state, Intent::ToggleShortcutHelpRequested);
    assert!(!state.ui.shortcut_help_open);
}

#[test]
fn update_dispatches_config_intents_without_touching_preview_mode() {
    let mut state = AppState::new().expect("state");
    let preview_mode_before = state.preview.active_mode;

    update(&mut state, Intent::OpenConfigRequested);
    update(&mut state, Intent::MoveConfigSelection(1));

    assert_eq!(state.preview.active_mode, preview_mode_before);
}

#[test]
fn update_dispatches_project_intents_without_changing_active_panel() {
    let mut state = AppState::new().expect("state");
    let panel_before = state.active_panel();

    update(&mut state, Intent::SetProjectName("Refactor Test".to_string()));

    assert_eq!(state.active_panel(), panel_before);
    assert_eq!(state.project.name, "Refactor Test");
}

#[test]
fn update_dispatches_inspector_intents_without_switching_workspace_tabs() {
    let mut state = AppState::new().expect("state");
    let tab_before = state.ui.active_tab;

    update(&mut state, Intent::SetRuleKind(TokenRole::Background, RuleKind::Fixed));

    assert_eq!(state.ui.active_tab, tab_before);
}

#[test]
fn update_dispatches_text_input_intents_without_clearing_other_ui_state() {
    let mut state = AppState::new().expect("state");
    let active_tab_before = state.ui.active_tab;
    state.ui.text_input = Some(TextInputState {
        target: TextInputTarget::Config(ConfigFieldId::ProjectName),
        buffer: String::new(),
    });

    update(&mut state, Intent::AppendTextInput('a'));

    assert_eq!(state.ui.active_tab, active_tab_before);
    assert_eq!(state.ui.text_input.as_ref().map(|input| input.buffer.as_str()), Some("a"));
}
```

- [ ] **Step 2: Run the new tests to verify they fail or expose missing coverage**

Run:

```bash
env -u http_proxy -u https_proxy cargo test update_dispatches_navigation_intents_without_affecting_preview_state -- --exact
env -u http_proxy -u https_proxy cargo test update_dispatches_preview_mode_changes_without_affecting_config_focus -- --exact
env -u http_proxy -u https_proxy cargo test update_dispatches_modal_intents_without_losing_owner_focus -- --exact
env -u http_proxy -u https_proxy cargo test update_dispatches_config_intents_without_touching_preview_mode -- --exact
env -u http_proxy -u https_proxy cargo test update_dispatches_project_intents_without_changing_active_panel -- --exact
env -u http_proxy -u https_proxy cargo test update_dispatches_inspector_intents_without_switching_workspace_tabs -- --exact
env -u http_proxy -u https_proxy cargo test update_dispatches_text_input_intents_without_clearing_other_ui_state -- --exact
```

Expected:

- a failing assertion that proves the test is real, not compile failures

- [ ] **Step 3: Convert `update.rs` into a directory module**

Keep the public boundary:

```rust
pub fn update(state: &mut AppState, intent: Intent) -> Vec<Effect>
```

Move handlers by responsibility:

- navigation/focus/hints/fullscreen
- preview flows
- modal open/close and owner restoration
- config/project/export edits
- inspector mutations
- text input helpers
- preserve and re-export these existing helper entrypoints from `src/app/update/mod.rs`:
  - `default_input_buffer`
  - `filtered_source_options`
  - `config_fields`
  - `current_source_for_control`

Downstream files should keep importing `crate::app::update::...` unchanged. Avoid unnecessary edit churn outside the new `src/app/update/` module unless a real compile error forces it.

- [ ] **Step 4: Keep helper visibility narrow**

Implementation target:

- most helpers remain `pub(super)` or private
- avoid creating a second dispatch layer with unnecessary traits
- keep the module graph easy to read

- [ ] **Step 5: Run update-focused tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test app::update::
env -u http_proxy -u https_proxy cargo test platform::tui::event_adapter::
env -u http_proxy -u https_proxy cargo test platform::gui::bridge::
```

Expected:

- PASS

- [ ] **Step 6: Run full suite**

Run:

```bash
env -u http_proxy -u https_proxy cargo test
env -u http_proxy -u https_proxy cargo check
```

Expected:

- PASS

- [ ] **Step 7: Commit**

```bash
git add src/app/mod.rs src/app/update
git commit -m "refactor: split update by intent family"
```

## Task 4: Remove remaining duplicated hardcoded mappings and tighten the boundaries

**Files:**
- Modify: `src/app/view/theme_tab.rs`
- Modify: `src/app/view/window.rs`
- Modify: `src/app/view/layout.rs`
- Modify: `src/app/hint_nav.rs`
- Modify: `src/i18n/mod.rs`
- Test: `src/app/hint_nav.rs`
- Test: `src/app/view/theme_tab.rs`
- Test: `src/i18n/mod.rs`

- [ ] **Step 1: Add one final drift-prevention regression test**

```rust
#[test]
fn preview_mode_order_is_shared_by_view_and_hint_navigation() {
    let state = AppState::new().expect("state");
    let hinted_modes = main_window_hint_targets(&state)
        .into_iter()
        .filter_map(|target| match target {
            HintTarget::PreviewTab { mode, .. } => Some(mode),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(hinted_modes, PreviewMode::ALL.to_vec());
}
```

- [ ] **Step 2: Run the drift-prevention test to verify it fails if a duplicated mapping remains**

Run:

```bash
env -u http_proxy -u https_proxy cargo test preview_mode_order_is_shared_by_view_and_hint_navigation -- --exact
```

Expected:

- FAIL if any duplicated ordering or label mapping remains

- [ ] **Step 3: Replace remaining repeated facts with shared lookups where the benefit is clear**

Focus only on obvious repeats still left after Tasks 1-3:

- preview mode ordering/labels
- panel-title label lookup drift
- workspace tab label/hint drift
- layout-visible panel facts that can clearly reuse shared specs

Do not force all code into tables. Keep one-off logic inline when it is genuinely only used once.

- [ ] **Step 4: Run the targeted regressions**

Run:

```bash
env -u http_proxy -u https_proxy cargo test main_window_hint_targets_flatten_visible_panels_and_tabs -- --exact
env -u http_proxy -u https_proxy cargo test preview_panel_tabs_show_hint_shortcuts_during_navigation_mode -- --exact
env -u http_proxy -u https_proxy cargo test every_ui_text_key_exists_in_both_locales -- --exact
env -u http_proxy -u https_proxy cargo test preview_mode_order_is_shared_by_view_and_hint_navigation -- --exact
```

Expected:

- PASS

- [ ] **Step 5: Run final project verification**

Run:

```bash
cargo fmt
env -u http_proxy -u https_proxy cargo test
env -u http_proxy -u https_proxy cargo check
```

Expected:

- PASS
- no behavior changes

- [ ] **Step 6: Commit**

```bash
git add src/app/view/theme_tab.rs src/app/view/window.rs src/app/view/layout.rs src/app/hint_nav.rs src/i18n/mod.rs
git commit -m "refactor: remove remaining duplicated ui mappings"
```

## Completion Checklist

- [ ] Shared panel/workspace/preview metadata is defined once per concept.
- [ ] `src/platform/tui/renderer.rs` has been replaced by focused renderer modules with the same public entrypoint.
- [ ] `src/app/update.rs` has been replaced by focused update modules with the same public entrypoint.
- [ ] Repeated hardcoded mappings are reduced without introducing opaque abstractions.
- [ ] `event -> intent -> update -> render` still describes the runtime truth.
- [ ] `cargo fmt`, `cargo test`, and `cargo check` all pass at the end.

## Notes For The Implementer

- Keep this pass conservative. It is an engineering cleanup, not an architectural rewrite.
- When unsure between a tiny explicit `match` and a generic metadata table, prefer the version that is easier to read locally.
- Preserve current behavior even if some behavior feels imperfect. UX changes belong to a separate spec.
- The final code should be easier to navigate for a human reader, not merely shorter.
