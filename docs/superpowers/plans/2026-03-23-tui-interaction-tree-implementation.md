# TUI Interaction Tree Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current transitional TUI focus/routing model with a true dual-tree interaction architecture, then add the minimum validation-facing UX needed to prove it works: recursive preview interaction, an interaction inspector, and fullscreen as a render-only style.

**Architecture:** Keep `ViewTree` as the rendering/layout tree and introduce a separate `InteractionTree` with explicit node capabilities, parent/child relationships, focus ownership, and a `mode_stack`. Modal surfaces, preview-local tabs, and future nested tools should all become normal interaction nodes instead of special cases. Fullscreen stays out of the interaction model and lives in view/UI state as a renderer choice.

**Tech Stack:** Rust, ratatui, crossterm, existing `CoreSession` / `Intent` / `Effect` core, inline Rust unit tests, Fluent i18n

---

## Scope Notes

Execute this plan in a dedicated git worktree so the interaction refactor, docs migration, and existing uncommitted workspace changes stay isolated.

This plan intentionally covers one subsystem: the TUI interaction model.

Included:

- real `InteractionTree`
- `mode_stack`
- routing rewrite
- modal normalization
- recursive preview interaction nodes
- interaction inspector panel
- fullscreen render style
- controlled documentation migration into `docs/`

Deferred:

- GUI interaction parity
- keymap redesign beyond the current preset model
- command palette or other new global tools

## File Structure and Responsibilities

### Create

- `src/app/interaction/mod.rs`
  - module boundary and public exports for the new interaction system
- `src/app/interaction/tree.rs`
  - `SurfaceNode`, `InteractionTree`, node capabilities, tree building helpers
- `src/app/interaction/state.rs`
  - `InteractionState`, `InteractionMode`, `focus_path`, `mode_stack`, focus restoration helpers
- `src/app/interaction/routing.rs`
  - `UiAction` routing against the tree, bubbling, modal confinement, default action dispatch
- `src/app/interaction/tests.rs`
  - focused unit tests for tree construction, routing, modal scoping, preview recursion
- `src/app/view/interaction_panel.rs`
  - view builder for the interaction inspector panel
- `docs/architecture/cross-platform-ui-architecture.md`
- `docs/architecture/event-intent-effect.md`
- `docs/architecture/viewtree-snapshot-viewmodel.md`
- `docs/architecture/tui-surface-interaction-model.md`
- `docs/architecture/preview-architecture.md`
- `docs/product/product-architecture-plan.md`
- `docs/product/ui-control-abstraction-guide.md`
- `docs/reference/development-guidelines.md`
- `docs/reference/third-party-libraries.md`

### Modify

- `src/app/mod.rs`
  - switch `interaction` from flat file to directory module
- `src/app/intent.rs`
  - replace old interaction intents with tree-oriented intents
- `src/app/state.rs`
  - store new interaction state and fullscreen render state
- `src/app/update.rs`
  - mutate interaction tree state, focus restoration, fullscreen toggles, inspector toggle if needed
- `src/app/actions.rs`
  - remove stale panel-cycling semantics and align help/menu metadata with contextual routing
- `src/app/workspace.rs`
  - add any new panel ids needed for the interaction inspector and keep panel ordering stable
- `src/app/view/types.rs`
  - add view anchor ids and fullscreen-related view metadata without mixing in interaction semantics
- `src/app/view/window.rs`
  - build the main window view plus interaction inspector placement and fullscreen-aware layout selection
- `src/app/view/layout.rs`
  - include an inspector-capable theme layout and fullscreen-aware workspace composition entry points
- `src/app/view/theme_tab.rs`
  - bind preview panel tabs/body/header to the new interaction ids and inspector panel output
- `src/app/view/overlays.rs`
  - map modal views to stable anchors/ids that the interaction tree can reference
- `src/platform/tui/event_adapter.rs`
  - map keys into `UiAction` without panel-specific navigation leakage
- `src/platform/tui/runtime.rs`
  - use the new interaction state for capture routing and view refreshes
- `src/platform/tui/renderer.rs`
  - render fullscreen targets and the interaction inspector using view-only state
- `src/preview.rs`
  - expose stable preview interaction ids or helper metadata for preview tabs/body split
- `src/i18n/mod.rs`
  - add any new UI text keys for inspector/fullscreen/focus labels
- `locales/en-US/ui.ftl`
- `locales/zh-CN/ui.ftl`
  - translated UI copy for new TUI interaction surfaces

### Delete / Move

- `src/app/interaction.rs`
  - replace with `src/app/interaction/`
- Move existing root docs into `docs/architecture/`, `docs/product/`, and `docs/reference/`

### Test Entry Points

- `src/app/interaction/tests.rs`
  - new focused interaction-model tests
- `src/app/update.rs`
  - state mutation and fullscreen state tests
- `src/platform/tui/event_adapter.rs`
  - keymap-to-action routing tests
- `src/i18n/mod.rs`
  - locale key existence tests updated for new UI copy

## Task 1: Split the interaction module and add a real tree model

**Files:**
- Create: `src/app/interaction/mod.rs`
- Create: `src/app/interaction/tree.rs`
- Create: `src/app/interaction/state.rs`
- Create: `src/app/interaction/tests.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/state.rs`
- Delete: `src/app/interaction.rs`
- Test: `src/app/interaction/tests.rs`

- [ ] **Step 1: Write the failing tree-construction tests**

```rust
#[test]
fn interaction_tree_contains_preview_tabs_under_preview_panel() {
    let state = AppState::new().expect("state");
    let tree = build_interaction_tree(&state);

    assert_eq!(tree.parent_of(SurfaceId::PreviewTabs), Some(SurfaceId::PreviewPanel));
    assert_eq!(tree.parent_of(SurfaceId::PreviewBody), Some(SurfaceId::PreviewPanel));
}

#[test]
fn interaction_tree_uses_visible_panels_for_active_workspace_tab() {
    let mut state = AppState::new().expect("state");
    state.ui.active_tab = WorkspaceTab::Project;

    let tree = build_interaction_tree(&state);

    assert!(tree.is_visible(SurfaceId::ProjectConfigPanel));
    assert!(!tree.is_visible(SurfaceId::TokensPanel));
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test interaction_tree_contains_preview_tabs_under_preview_panel -- --exact
env -u http_proxy -u https_proxy cargo test interaction_tree_uses_visible_panels_for_active_workspace_tab -- --exact
```

Expected:

- compile errors for missing `build_interaction_tree`
- unresolved `SurfaceId` variants like `PreviewTabs`

- [ ] **Step 3: Create the interaction module directory and minimal tree/state types**

```rust
pub struct InteractionTree {
    pub root: SurfaceId,
    pub nodes: BTreeMap<SurfaceId, SurfaceNode>,
}

pub struct SurfaceNode {
    pub id: SurfaceId,
    pub parent: Option<SurfaceId>,
    pub children: Vec<SurfaceId>,
    pub focusable: bool,
    pub visible: bool,
    pub tab_scope: Option<TabScope>,
    pub default_action: Option<SurfaceAction>,
    pub child_navigation: Option<ChildNavigationSpec>,
    pub bubble_policy: BubblePolicy,
    pub view_anchor: Option<ViewAnchorId>,
}
```

- [ ] **Step 4: Build the minimal tree from `AppState`**

Implementation target:

- `MainWindow` as root child of `AppRoot`
- workspace-visible panels as `MainWindow` children
- preview panel children include `PreviewTabs` and `PreviewBody`
- modal nodes exist in the tree and flip `visible` based on state

- [ ] **Step 5: Run the tests again**

Run:

```bash
env -u http_proxy -u https_proxy cargo test interaction_tree_contains_preview_tabs_under_preview_panel -- --exact
env -u http_proxy -u https_proxy cargo test interaction_tree_uses_visible_panels_for_active_workspace_tab -- --exact
```

Expected:

- both tests PASS

- [ ] **Step 6: Run the broader interaction-focused test set**

Run:

```bash
env -u http_proxy -u https_proxy cargo test interaction_tree -- --nocapture
```

Expected:

- all tree-related tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/app/mod.rs src/app/state.rs src/app/interaction/mod.rs src/app/interaction/tree.rs src/app/interaction/state.rs src/app/interaction/tests.rs src/app/interaction.rs
git commit -m "refactor: introduce interaction tree primitives"
```

## Task 2: Replace single interaction mode with a mode stack

**Files:**
- Modify: `src/app/interaction/state.rs`
- Modify: `src/app/state.rs`
- Modify: `src/app/update.rs`
- Modify: `src/app/intent.rs`
- Test: `src/app/interaction/tests.rs`
- Test: `src/app/update.rs`

- [ ] **Step 1: Write the failing mode-stack tests**

```rust
#[test]
fn modal_mode_pushes_and_pops_without_losing_owner_focus() {
    let mut interaction = InteractionState::new(SurfaceId::TokensPanel);

    interaction.push_mode(InteractionMode::Modal { owner: SurfaceId::NumericEditorSurface });
    interaction.focus_path = vec![SurfaceId::MainWindow, SurfaceId::ParamsPanel, SurfaceId::NumericEditorSurface];

    interaction.pop_mode();

    assert_eq!(interaction.focused_surface(), SurfaceId::ParamsPanel);
}

#[test]
fn capture_mode_can_stack_on_top_of_normal_mode() {
    let mut interaction = InteractionState::new(SurfaceId::PreviewBody);
    interaction.push_mode(InteractionMode::Capture { owner: SurfaceId::PreviewBody });

    assert!(interaction.has_mode_for(SurfaceId::PreviewBody));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test modal_mode_pushes_and_pops_without_losing_owner_focus -- --exact
env -u http_proxy -u https_proxy cargo test capture_mode_can_stack_on_top_of_normal_mode -- --exact
```

Expected:

- compile errors for missing stack helpers

- [ ] **Step 3: Implement `mode_stack` and owner-scoped helpers**

```rust
pub struct InteractionState {
    pub focus_path: Vec<SurfaceId>,
    pub mode_stack: Vec<InteractionMode>,
}

impl InteractionState {
    pub fn push_mode(&mut self, mode: InteractionMode) { /* ... */ }
    pub fn pop_mode(&mut self) -> Option<InteractionMode> { /* ... */ }
    pub fn focused_surface(&self) -> SurfaceId { /* ... */ }
}
```

- [ ] **Step 4: Replace old single-mode usages in app state and update logic**

Implementation target:

- remove direct writes to `interaction.mode`
- migrate focus reset paths to stack-aware helpers
- keep behavior unchanged except for the new stack storage

- [ ] **Step 5: Run the new tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test modal_mode_pushes_and_pops_without_losing_owner_focus -- --exact
env -u http_proxy -u https_proxy cargo test capture_mode_can_stack_on_top_of_normal_mode -- --exact
```

Expected:

- both tests PASS

- [ ] **Step 6: Run update regression tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test workspace_tabs_restore_panel_focus -- --exact
env -u http_proxy -u https_proxy cargo test preview_capture_requires_interactive_mode -- --exact
```

Expected:

- tests PASS after migration to the stack model

- [ ] **Step 7: Commit**

```bash
git add src/app/interaction/state.rs src/app/state.rs src/app/update.rs src/app/intent.rs src/app/interaction/tests.rs
git commit -m "refactor: replace interaction mode with mode stack"
```

## Task 3: Rewrite routing around node capabilities and remove stale panel-cycling semantics

**Files:**
- Create: `src/app/interaction/routing.rs`
- Modify: `src/app/interaction/mod.rs`
- Modify: `src/app/actions.rs`
- Modify: `src/app/intent.rs`
- Modify: `src/platform/tui/event_adapter.rs`
- Test: `src/app/interaction/tests.rs`
- Test: `src/platform/tui/event_adapter.rs`

- [ ] **Step 1: Write the failing routing tests**

```rust
#[test]
fn switch_tab_bubbles_from_tokens_panel_to_main_window() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![SurfaceId::AppRoot, SurfaceId::MainWindow, SurfaceId::TokensPanel];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(intents.as_slice(), [Intent::CycleWorkspaceTab(1)]));
}

#[test]
fn switch_tab_is_handled_locally_by_preview_tabs() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewTabs,
    ];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(intents.as_slice(), [Intent::CyclePreviewMode(1)]));
}
```

- [ ] **Step 2: Run the routing tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test switch_tab_bubbles_from_tokens_panel_to_main_window -- --exact
env -u http_proxy -u https_proxy cargo test switch_tab_is_handled_locally_by_preview_tabs -- --exact
```

Expected:

- failures or compile errors because routing is still panel-special-cased

- [ ] **Step 3: Implement routing against node capabilities**

```rust
pub fn route_ui_action(state: &AppState, action: UiAction) -> Vec<Intent> {
    let tree = build_interaction_tree(state);
    let focus_path = state.ui.interaction.focus_path.clone();

    route_from_focus(&tree, state, &focus_path, action)
}
```

Implementation target:

- `SwitchTab` checks `tab_scope`
- `Activate` uses `default_action`
- `NavigateChildren` consults `child_navigation`
- bubbling is controlled by `bubble_policy`

- [ ] **Step 4: Remove stale panel-cycling actions from the keymap layer**

Implementation target:

- drop `PreviousPanel` / `NextPanel` from `UiAction`
- drop `BoundAction::PreviousPanel` / `NextPanel` if no longer needed
- keep numbered navigation as the explicit panel-focus mechanism

- [ ] **Step 5: Run the routing tests again**

Run:

```bash
env -u http_proxy -u https_proxy cargo test switch_tab_bubbles_from_tokens_panel_to_main_window -- --exact
env -u http_proxy -u https_proxy cargo test switch_tab_is_handled_locally_by_preview_tabs -- --exact
```

Expected:

- both tests PASS

- [ ] **Step 6: Run the TUI event adapter tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test preview_panel_tab_shortcuts_cycle_modes -- --exact
env -u http_proxy -u https_proxy cargo test tab_on_regular_panel_bubbles_to_workspace_tab_switch -- --exact
env -u http_proxy -u https_proxy cargo test main_window_navigation_uses_digit_selection_after_activate -- --exact
```

Expected:

- all tests PASS with the new routing model

- [ ] **Step 7: Commit**

```bash
git add src/app/interaction/mod.rs src/app/interaction/routing.rs src/app/actions.rs src/app/intent.rs src/platform/tui/event_adapter.rs src/app/interaction/tests.rs
git commit -m "refactor: route tui actions through interaction capabilities"
```

## Task 4: Make modal surfaces first-class interaction nodes

**Files:**
- Modify: `src/app/interaction/tree.rs`
- Modify: `src/app/interaction/routing.rs`
- Modify: `src/app/update.rs`
- Modify: `src/app/view/overlays.rs`
- Test: `src/app/interaction/tests.rs`
- Test: `src/app/update.rs`

- [ ] **Step 1: Write the failing modal-confinement tests**

```rust
#[test]
fn source_picker_routes_within_its_modal_subtree() {
    let mut state = AppState::new().expect("state");
    state.ui.source_picker = Some(SourcePickerState {
        control: ControlId::Reference(TokenRole::Background, ReferenceField::AliasSource),
        filter: String::new(),
        selected: 0,
    });
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::InspectorPanel,
        SurfaceId::SourcePicker,
    ];
    state.ui.interaction.mode_stack.push(InteractionMode::Modal { owner: SurfaceId::SourcePicker });

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(intents.is_empty());
}

#[test]
fn closing_numeric_editor_restores_focus_to_owner_surface() {
    let mut state = AppState::new().expect("state");
    open_text_input(&mut state, TextInputTarget::Control(ControlId::Param(ParamKey::AccentHue)));

    update(&mut state, Intent::CancelTextInput);

    assert_eq!(state.ui.interaction.focused_surface(), SurfaceId::ParamsPanel);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test source_picker_routes_within_its_modal_subtree -- --exact
env -u http_proxy -u https_proxy cargo test closing_numeric_editor_restores_focus_to_owner_surface -- --exact
```

Expected:

- failures because modals are still appended as special cases

- [ ] **Step 3: Build modal nodes into the interaction tree and constrain routing**

Implementation target:

- `ShortcutHelp`, `ConfigDialog`, `SourcePicker`, `NumericEditorSurface` are normal nodes
- opening a modal sets node visibility and pushes `Modal { owner }`
- `route_ui_action()` limits routing to the modal subtree while modal mode is active

- [ ] **Step 4: Remove effective-focus-path patching**

Implementation target:

- delete or reduce `effective_focus_path()` to a pure state accessor
- no more “append modal surface if open” logic

- [ ] **Step 5: Run the modal tests again**

Run:

```bash
env -u http_proxy -u https_proxy cargo test source_picker_routes_within_its_modal_subtree -- --exact
env -u http_proxy -u https_proxy cargo test closing_numeric_editor_restores_focus_to_owner_surface -- --exact
```

Expected:

- both tests PASS

- [ ] **Step 6: Run focused update regressions**

Run:

```bash
env -u http_proxy -u https_proxy cargo test active_numeric_input_steps_and_syncs_buffer -- --exact
env -u http_proxy -u https_proxy cargo test digit_navigation_focuses_visible_panel_in_current_tab -- --exact
```

Expected:

- tests PASS after modal normalization

- [ ] **Step 7: Commit**

```bash
git add src/app/interaction/tree.rs src/app/interaction/routing.rs src/app/update.rs src/app/view/overlays.rs src/app/interaction/tests.rs
git commit -m "refactor: make modal surfaces first-class interaction nodes"
```

## Task 5: Split the preview panel into recursive interaction nodes and align capture

**Files:**
- Modify: `src/preview.rs`
- Modify: `src/app/interaction/tree.rs`
- Modify: `src/app/interaction/routing.rs`
- Modify: `src/app/view/theme_tab.rs`
- Modify: `src/platform/tui/runtime.rs`
- Modify: `src/platform/tui/event_adapter.rs`
- Test: `src/app/interaction/tests.rs`
- Test: `src/platform/tui/event_adapter.rs`
- Test: `src/app/update.rs`

- [ ] **Step 1: Write the failing preview-node tests**

```rust
#[test]
fn activate_on_preview_body_enters_capture_mode() {
    let mut state = AppState::new().expect("state");
    state.preview.active_mode = PreviewMode::Shell;
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];

    let intents = route_ui_action(&state, UiAction::Activate);

    assert!(matches!(intents.as_slice(), [Intent::SetPreviewCapture(true)]));
}

#[test]
fn switch_tab_on_preview_body_bubbles_to_preview_tabs_owner() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];

    let intents = route_ui_action(&state, UiAction::NextTab);

    assert!(matches!(intents.as_slice(), [Intent::CyclePreviewMode(1)]));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test activate_on_preview_body_enters_capture_mode -- --exact
env -u http_proxy -u https_proxy cargo test switch_tab_on_preview_body_bubbles_to_preview_tabs_owner -- --exact
```

Expected:

- failures because preview is still modeled as one surface

- [ ] **Step 3: Add explicit preview interaction nodes**

Implementation target:

- `SurfaceId::PreviewPanel`
- `SurfaceId::PreviewTabs`
- `SurfaceId::PreviewBody`
- tree builder links them as parent/child nodes

- [ ] **Step 4: Route capture and preview mode switching through those nodes**

Implementation target:

- `PreviewTabs` owns `tab_scope`
- `PreviewBody` owns capture `default_action`
- TUI runtime capture checks the new focus and mode-stack model instead of old preview-specific assumptions

- [ ] **Step 5: Run the preview-node tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test activate_on_preview_body_enters_capture_mode -- --exact
env -u http_proxy -u https_proxy cargo test switch_tab_on_preview_body_bubbles_to_preview_tabs_owner -- --exact
```

Expected:

- both tests PASS

- [ ] **Step 6: Re-run preview regressions**

Run:

```bash
env -u http_proxy -u https_proxy cargo test preview_panel_tab_shortcuts_cycle_modes -- --exact
env -u http_proxy -u https_proxy cargo test preview_panel_enter_captures_preview -- --exact
env -u http_proxy -u https_proxy cargo test cycling_preview_mode_prepares_runtime_placeholder -- --exact
env -u http_proxy -u https_proxy cargo test preview_capture_requires_interactive_mode -- --exact
```

Expected:

- all preview tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/preview.rs src/app/interaction/tree.rs src/app/interaction/routing.rs src/app/view/theme_tab.rs src/platform/tui/runtime.rs src/platform/tui/event_adapter.rs src/app/interaction/tests.rs src/app/update.rs
git commit -m "feat: model preview as recursive interaction surfaces"
```

## Task 6: Add the interaction inspector panel

**Files:**
- Create: `src/app/view/interaction_panel.rs`
- Modify: `src/app/view/mod.rs`
- Modify: `src/app/view/layout.rs`
- Modify: `src/app/view/window.rs`
- Modify: `src/app/view/theme_tab.rs`
- Modify: `src/app/view/types.rs`
- Modify: `src/app/workspace.rs`
- Modify: `src/i18n/mod.rs`
- Modify: `locales/en-US/ui.ftl`
- Modify: `locales/zh-CN/ui.ftl`
- Test: `src/app/interaction/tests.rs`

- [ ] **Step 1: Write the failing inspector-view test**

```rust
#[test]
fn interaction_inspector_lists_focus_path_and_modes() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![
        SurfaceId::AppRoot,
        SurfaceId::MainWindow,
        SurfaceId::PreviewPanel,
        SurfaceId::PreviewBody,
    ];
    state.ui.interaction.mode_stack.push(InteractionMode::Capture { owner: SurfaceId::PreviewBody });

    let panel = build_interaction_panel(&state);

    let PanelBody::Document(document) = &panel.body else {
        panic!("expected document body");
    };
    let body = document
        .lines
        .iter()
        .flat_map(|line| line.spans.iter().map(|span| span.text.as_str()))
        .collect::<Vec<_>>()
        .join("");
    assert!(body.contains("PreviewBody"));
    assert!(body.contains("Capture"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
env -u http_proxy -u https_proxy cargo test interaction_inspector_lists_focus_path_and_modes -- --exact
```

Expected:

- compile error for missing inspector panel builder

- [ ] **Step 3: Build the inspector panel view**

Implementation target:

- show the interaction tree as indented rows
- highlight the current `focus_path`
- display current `mode_stack`
- include node capability badges in text form only

- [ ] **Step 4: Add the inspector panel to the Theme workspace layout**

Implementation target:

- add a new `PanelId`
- place it in the theme layout without replacing the existing inspector
- ensure numbered navigation includes it

- [ ] **Step 5: Run the test again**

Run:

```bash
env -u http_proxy -u https_proxy cargo test interaction_inspector_lists_focus_path_and_modes -- --exact
```

Expected:

- test PASS

- [ ] **Step 6: Run i18n key-existence tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test every_ui_text_key_exists_in_both_locales -- --exact
env -u http_proxy -u https_proxy cargo test locales_return_different_ui_copy -- --exact
```

Expected:

- both tests PASS after adding the new UI copy

- [ ] **Step 7: Commit**

```bash
git add src/app/view/interaction_panel.rs src/app/view/mod.rs src/app/view/layout.rs src/app/view/window.rs src/app/view/theme_tab.rs src/app/view/types.rs src/app/workspace.rs src/i18n/mod.rs locales/en-US/ui.ftl locales/zh-CN/ui.ftl
git commit -m "feat: add tui interaction inspector panel"
```

## Task 7: Add fullscreen as a render-only style

**Files:**
- Modify: `src/app/state.rs`
- Modify: `src/app/intent.rs`
- Modify: `src/app/update.rs`
- Modify: `src/app/view/types.rs`
- Modify: `src/app/view/window.rs`
- Modify: `src/platform/tui/renderer.rs`
- Modify: `src/app/actions.rs`
- Modify: `src/platform/tui/event_adapter.rs`
- Modify: `src/i18n/mod.rs`
- Modify: `locales/en-US/ui.ftl`
- Modify: `locales/zh-CN/ui.ftl`
- Test: `src/app/update.rs`

- [ ] **Step 1: Write the failing fullscreen-state tests**

```rust
#[test]
fn toggle_fullscreen_targets_the_focused_surface() {
    let mut state = AppState::new().expect("state");
    state.ui.interaction.focus_path = vec![SurfaceId::AppRoot, SurfaceId::MainWindow, SurfaceId::PreviewPanel];

    update(&mut state, Intent::ToggleFullscreenRequested);

    assert_eq!(state.ui.fullscreen_surface, Some(SurfaceId::PreviewPanel));
}

#[test]
fn toggling_fullscreen_twice_restores_normal_layout() {
    let mut state = AppState::new().expect("state");
    state.ui.fullscreen_surface = Some(SurfaceId::PreviewPanel);

    update(&mut state, Intent::ToggleFullscreenRequested);

    assert_eq!(state.ui.fullscreen_surface, None);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
env -u http_proxy -u https_proxy cargo test toggle_fullscreen_targets_the_focused_surface -- --exact
env -u http_proxy -u https_proxy cargo test toggling_fullscreen_twice_restores_normal_layout -- --exact
```

Expected:

- compile errors for missing intent/state fields

- [ ] **Step 3: Add fullscreen state and a single toggle intent**

Implementation target:

- one field such as `ui.fullscreen_surface: Option<SurfaceId>`
- one toggle intent
- no interaction-mode changes

- [ ] **Step 4: Make the renderer choose fullscreen layout by view anchor**

Implementation target:

- if `fullscreen_surface` is `None`, render the normal layout
- if set, render only the anchored target plus top/bottom chrome
- do not change focus semantics

- [ ] **Step 5: Run the fullscreen tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test toggle_fullscreen_targets_the_focused_surface -- --exact
env -u http_proxy -u https_proxy cargo test toggling_fullscreen_twice_restores_normal_layout -- --exact
```

Expected:

- both tests PASS

- [ ] **Step 6: Re-run focused TUI input tests**

Run:

```bash
env -u http_proxy -u https_proxy cargo test question_mark_opens_shortcut_help -- --exact
env -u http_proxy -u https_proxy cargo test tab_on_regular_panel_bubbles_to_workspace_tab_switch -- --exact
```

Expected:

- existing global input semantics still PASS

- [ ] **Step 7: Commit**

```bash
git add src/app/state.rs src/app/intent.rs src/app/update.rs src/app/view/types.rs src/app/view/window.rs src/platform/tui/renderer.rs src/app/actions.rs src/platform/tui/event_adapter.rs src/i18n/mod.rs locales/en-US/ui.ftl locales/zh-CN/ui.ftl
git commit -m "feat: add fullscreen render style for focused surfaces"
```

## Task 8: Migrate the root design docs into `docs/`

**Files:**
- Create: `docs/architecture/`
- Create: `docs/product/`
- Create: `docs/reference/`
- Modify: moved markdown files and any links that reference them
- Test: documentation link grep checks

- [ ] **Step 1: Move the existing docs into the target folders**

Move:

```text
cross_platform_ui_architecture_guide.md -> docs/architecture/cross-platform-ui-architecture.md
event_intent_effect_guide.md -> docs/architecture/event-intent-effect.md
viewtree_snapshot_viewmodel_guide.md -> docs/architecture/viewtree-snapshot-viewmodel.md
tui_surface_interaction_model.md -> docs/architecture/tui-surface-interaction-model.md
preview_architecture_guide.md -> docs/architecture/preview-architecture.md
product_architecture_plan.md -> docs/product/product-architecture-plan.md
ui_control_abstraction_guide.md -> docs/product/ui-control-abstraction-guide.md
development_guidelines.md -> docs/reference/development-guidelines.md
third_party_libraries.md -> docs/reference/third-party-libraries.md
```

- [ ] **Step 2: Update markdown links and code references**

Run after moving:

```bash
rg -n "cross_platform_ui_architecture_guide|event_intent_effect_guide|viewtree_snapshot_viewmodel_guide|tui_surface_interaction_model|preview_architecture_guide|product_architecture_plan|ui_control_abstraction_guide|development_guidelines|third_party_libraries" .
```

Expected:

- only the new `docs/` paths remain

- [ ] **Step 3: Verify there are no broken absolute-path markdown links pointing at old root docs**

Run:

```bash
rg -n "/Users/ysl/Code/theme/(cross_platform_ui_architecture_guide|event_intent_effect_guide|viewtree_snapshot_viewmodel_guide|tui_surface_interaction_model|preview_architecture_guide|product_architecture_plan|ui_control_abstraction_guide|development_guidelines|third_party_libraries)\\.md" docs src
```

Expected:

- no matches

- [ ] **Step 4: Commit**

```bash
git add docs/ src/
git commit -m "docs: reorganize architecture and reference guides"
```

## Final Verification

- [ ] **Step 1: Format**

Run:

```bash
cargo fmt
```

Expected:

- formatting completes without errors

- [ ] **Step 2: Run the full test suite**

Run:

```bash
cargo test
```

Expected:

- all tests PASS

- [ ] **Step 3: Run a compile check**

Run:

```bash
env -u http_proxy -u https_proxy cargo check
```

Expected:

- build succeeds

- [ ] **Step 4: Smoke-test the TUI manually**

Run:

```bash
cargo run -- --platform tui
```

Manual checklist:

- `Tab` switches the current tab scope
- numbered navigation still reaches visible panels
- preview tab switching works through preview-local nodes
- modal surfaces trap input correctly
- interaction inspector shows focus path and mode stack
- fullscreen toggles on the focused surface without changing interaction ownership

- [ ] **Step 5: Final commit**

```bash
git add src/ docs/ locales/
git commit -m "feat: complete interaction-tree-driven tui routing"
```
