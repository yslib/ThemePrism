# TUI Interaction Tree Design

## Summary

The current TUI architecture is moving in the right direction, but it is still in a transitional state.

The code already has:

- semantic actions
- a notion of focus path
- surface-aware routing
- panel-local tabs in the preview panel

But it does **not** yet have a real recursive interaction model. The current implementation still relies on:

- flat `SurfaceId` handling
- modal-specific special cases
- a single `InteractionMode`
- panel-specific routing branches
- leftover panel-navigation actions that conflict with contextual tab semantics

This spec proposes the next architectural step:

- keep `ViewTree` as the rendering/layout tree
- introduce a separate `InteractionTree` as the interaction tree
- make recursive interaction a first-class design goal
- treat fullscreen as a rendering style, not an interaction concept
- add an interaction inspector panel that visualizes the current interaction tree, focus path, and mode stack

This spec is intentionally focused on the TUI interaction model. It does not redesign the shared theme core, exporters, or GUI backend.

## Goals

- Establish a stable recursive interaction model for the TUI.
- Separate rendering concerns from interaction concerns.
- Make `Tab`, `Enter`, `Esc`, and directional navigation contextual and surface-driven.
- Support nested interaction scopes such as:
  - main window tabs
  - preview-local tabs
  - modal editors
  - future nested tools inside panels
- Make the architecture extensible enough for future complex TUI experiments.
- Preserve compatibility with the existing shared Rust core and platform-neutral business logic.

## Non-Goals

- This spec does not define the final key bindings in detail.
- This spec does not move the GUI onto the same rendering framework.
- This spec does not require fullscreen to become a separate interaction mode.
- This spec does not require every rendered view node to become an interaction node.
- This spec does not require a unified UI DSL that generates both trees from one schema yet.

## Current Review Findings

### What is already good

- `UiAction` is the correct abstraction boundary for keyboard semantics.
- `focus_path` is conceptually correct.
- Preview mode tabs already prove that local tab scopes are useful.
- `SurfaceView` and `MainWindowView` already hint at a composable window/surface model.
- The preview subsystem already separates runtime-backed preview hosting from shared preview state.

### What is not stable yet

1. **No real interaction tree**
   - `SurfaceId` exists, but parent/child relationships are implicit.
   - `effective_focus_path()` appends modal surfaces as special cases instead of traversing a real tree.

2. **Interaction mode is too flat**
   - `InteractionMode` is a single value.
   - Nested modal or capture workflows will eventually require a stack.

3. **Panel-specific routing still leaks into the model**
   - Preview tab behavior is handled as a panel-specific exception.
   - This means tab scope is not yet represented as a node capability.

4. **Old navigation semantics are still present**
   - `PreviousPanel` / `NextPanel` are still part of the action model.
   - This competes with the desired rule that tab switching should be contextual and panel focus should use explicit navigation.

5. **Modal surfaces are not yet first-class**
   - `TextInput`, `SourcePicker`, `ConfigDialog`, and `ShortcutHelp` still rely on special-case routing.
   - That prevents recursion from becoming a normal part of the model.

6. **Rendering and interaction are still only partially separated**
   - `SurfaceView` is mostly a visual container.
   - `SurfaceId` is mostly an interaction identifier.
   - The boundary is correct in principle, but not yet enforced structurally.

## Proposed Architecture

### 1. ViewTree

`ViewTree` remains the rendering tree.

Responsibilities:

- layout
- chrome
- panel composition
- overlays/surfaces
- fullscreen rendering style
- visual focus indication

`ViewTree` must not own:

- tab semantics
- default action semantics
- event bubbling rules
- modal scoping rules

### 2. InteractionTree

Introduce a separate `InteractionTree`.

Responsibilities:

- focus ownership
- action handling scopes
- parent/child navigation
- bubbling rules
- local tab scopes
- default actions
- modal scoping

This tree is the source of truth for:

- what is currently focused
- what surface receives `Activate`
- what surface owns `SwitchTab`
- where `Cancel` returns

### 3. InteractionState

Introduce a dedicated runtime interaction state:

```rust
InteractionState {
    focus_path: Vec<SurfaceId>,
    mode_stack: Vec<InteractionMode>,
}
```

This replaces the current “single mode plus effective path patching” model.

### 4. View and Interaction Must Stay Separate

The trees are related, but not merged.

- `ViewTree` and `InteractionTree` are separate structures.
- They are linked by stable ids.
- A view node may have no interaction node.
- An interaction node may have no independently rendered box.

This is required for recursive interaction.

Examples:

- `PreviewTabs` may be an interaction node without a standalone box.
- `MainWindow` may be one interaction node mapped onto several visual areas.
- A modal node may become active without existing in the normal workspace layout.

## Surface Node Model

The interaction tree should be data-driven, not trait-heavy.

Recommended minimum shape:

```rust
SurfaceNode {
    id: SurfaceId,
    parent: Option<SurfaceId>,
    children: Vec<SurfaceId>,

    focusable: bool,
    visible: bool,

    tab_scope: Option<TabScope>,
    default_action: Option<SurfaceAction>,
    child_navigation: Option<ChildNavigationSpec>,

    bubble_policy: BubblePolicy,
    view_anchor: Option<ViewAnchorId>,
}
```

### Field meanings

- `id`
  - Stable interaction identity.
- `parent` / `children`
  - Explicit interaction hierarchy.
- `focusable`
  - Whether this node may be the terminal focus target.
- `visible`
  - Whether the node participates in routing right now.
- `tab_scope`
  - Whether this node owns local tab semantics.
- `default_action`
  - What `Activate` means when this node is focused.
- `child_navigation`
  - Whether the node supports selecting among its child nodes.
- `bubble_policy`
  - Whether actions are handled locally, allowed to bubble, or must be intercepted.
- `view_anchor`
  - Optional link back to the rendering tree for highlighting, fullscreen, and diagnostics.

## Interaction Modes

Replace the single interaction mode with a stack.

Minimum useful mode set:

- `Normal`
- `NavigateChildren { owner: SurfaceId }`
- `Capture { owner: SurfaceId }`
- `Modal { owner: SurfaceId }`

This keeps the model small while still supporting:

- main-window child navigation
- preview capture
- modal editors and dialogs

Future modes may be added if needed, but they should remain owner-scoped rather than becoming many ad hoc booleans.

## Routing Model

The routing flow should be:

```text
Raw Event
-> Keymap
-> UiAction
-> Mode Stack
-> Focused Surface
-> Parent Bubble
-> Root Fallback
-> Drop
```

### Semantic meaning of core actions

- `Activate`
  - Run the focused surface’s default action.
- `Cancel`
  - Back out of the current interaction scope.
- `SwitchTab`
  - Switch the current tab scope owned by the focused surface, or bubble upward.
- `MoveUp/Down/Left/Right`
  - Surface-local navigation or adjustment actions.

The routing rule is:

- actions are first interpreted by the active mode
- then by the focused surface
- then by ancestors
- modal owners may intercept routing and block bubbling out of their subtree

## Mapping the Current App Into the Interaction Tree

### Top level

- `AppRoot`
  - logical root
  - not necessarily focusable
- `MainWindow`
  - top-level focusable surface
  - owns workspace-level child navigation
  - default action enters child navigation

### Theme workspace surfaces

When the active workspace tab is `Theme`, the `MainWindow` interaction children are:

- `TokensPanel`
- `ParamsPanel`
- `PreviewPanel`
- `PalettePanel`
- `InspectorPanel`

### Project workspace surfaces

When the active workspace tab is `Project`, the `MainWindow` interaction children are:

- `ProjectConfigPanel`
- `ExportTargetsPanel`
- `EditorPreferencesPanel`

The interaction tree should care about child ordering and parentage, not screen coordinates.

### Recursive surfaces inside panels

#### Preview panel

The preview panel should stop being modeled as a single interaction node.

Recommended child structure:

- `PreviewPanel`
- `PreviewTabs`
- `PreviewBody`

This allows:

- contextual tab switching inside preview
- capture ownership inside preview body
- future preview-local tools without inventing new exceptions

#### Numeric editor

The numeric editor should be modeled as a modal surface, not just an overlay flag.

Possible internal structure:

- `NumericEditorSurface`
- `NumericTrack`
- `NumericInput`
- `NumericActions`

The first implementation can keep the editor simple, but the model should allow this recursion.

## What Should Not Become Interaction Nodes

Do **not** promote all view nodes into the interaction tree.

These should generally remain rendering-only:

- layout splits
- decorative borders
- passive headers
- purely informational text blocks
- swatches with no direct interaction
- the status bar itself

Interaction nodes should only represent meaningful action scopes.

## Modal Surfaces

Modal UI should become normal interaction nodes with temporary visibility and an active modal mode.

This applies to:

- shortcut help
- config dialog
- source picker
- numeric editor

The routing rule should be:

- modal opens
- modal node becomes visible
- `mode_stack` gains `Modal { owner }`
- routing is constrained to the modal subtree until the modal exits

This removes the need for special-case focus-path patching.

## Fullscreen

Fullscreen is **not** an interaction mode.

It is a rendering style.

Recommended representation:

- store a fullscreen target as `Option<ViewAnchorId>` or `Option<SurfaceId>` in view/UI state
- renderer chooses whether to render the normal workspace layout or a focused fullscreen presentation

This keeps fullscreen orthogonal to:

- focus
- bubbling
- modal scope
- capture

Fullscreen should work with the interaction system, not redefine it.

## Interaction Inspector

Add a dedicated panel or auxiliary surface that visualizes the interaction system.

It should show:

- the interaction tree
- the current focus path
- the mode stack
- node capabilities such as:
  - tab scope
  - default action
  - child navigation
  - visibility

This is not only a debug tool. It is valuable product-level UX for a complex TUI because it shows the user which scope currently owns interaction.

## Relationship Between the Current Review Topics

### Topic B: Is the abstraction itself correct?

Current answer:

- directionally yes
- structurally not yet

The current code already has the right instincts, but it has not yet crossed the line into a real recursive interaction architecture.

### Topic C: Will the framework scale?

Current answer:

- only if Topic B is finished properly

Topic B and Topic C are tightly coupled.

If the interaction model is left in its current transitional form, future features will continue to become special cases.

If the interaction model is completed as proposed here, many future expansion problems stop being framework problems and become ordinary feature work.

## Recommended Refactor Sequence

1. Introduce a real `InteractionTree`.
2. Replace single `InteractionMode` with `mode_stack`.
3. Move tab/default-action/child-navigation semantics into node capabilities.
4. Convert modal/overlay handling into real interaction-tree nodes.
5. Split preview into recursive interaction nodes.
6. Only then refine concrete key semantics and UX details.

This order matters. The architecture should be corrected before key bindings and surface hints are polished further.

## Risks and Edge Cases Not Yet Fully Addressed

These need explicit planning during implementation:

- focus restoration after closing modal or capture scopes
- what happens when the focused node becomes invisible
- how child ordering is derived for numbered navigation
- how fullscreen maps from `SurfaceId` to the correct rendered anchor
- how preview capture fits into `mode_stack` without special-case leakage
- how interaction routing will be tested independently of rendering

These are not blockers for planning, but they must be deliberately handled during implementation.

## Documentation Reorganization Proposal

The repository currently keeps architecture and product design documents in the project root. That was acceptable during exploration, but it is now too flat.

Recommended target structure:

```text
docs/
  architecture/
    cross-platform-ui-architecture.md
    event-intent-effect.md
    viewtree-snapshot-viewmodel.md
    tui-surface-interaction-model.md
    preview-architecture.md
  product/
    product-architecture-plan.md
    ui-control-abstraction-guide.md
  reference/
    development-guidelines.md
    third-party-libraries.md
  superpowers/
    specs/
      2026-03-23-tui-interaction-tree-design.md
```

Rules for the reorganization:

- architecture concepts go under `docs/architecture/`
- product-level direction and planning go under `docs/product/`
- developer-facing rules and dependency references go under `docs/reference/`
- active brainstorming/planning specs go under `docs/superpowers/specs/`

This spec does **not** move those files yet. It only defines the target documentation structure so the subsequent plan can include a controlled migration.

## Recommendation

Proceed with the dual-tree architecture.

Do **not** continue patching key behavior on top of the current transitional interaction model.

The next planning phase should focus on:

- introducing the true interaction tree
- finishing the interaction state model
- normalizing modal and preview recursion

Only after that should the TUI UX layer continue evolving.
