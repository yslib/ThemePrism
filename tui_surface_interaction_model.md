# TUI Surface Interaction Model

## Goal

The TUI should stop treating each panel as a special-case input target. It needs one shared interaction model where:

- focus is explicit
- actions are semantic
- tab switching is contextual
- `Enter` runs the focused surface's default action
- `Esc` returns control up the tree
- nested surfaces can be added later without rewriting the key handling model

## Core Concepts

### 1. `Surface`

A `Surface` is any focusable interaction boundary, not just a rendered panel.

Current examples:

- `MainWindow`
- `Panel(Preview)`
- `Panel(Tokens)`
- `ConfigDialog`
- `ShortcutHelp`
- `TextInput`
- `SourcePicker`

The important rule is:

- layout decides where a surface is rendered
- interaction decides which surface is focused

Those two concerns should stay separate.

### 2. `FocusPath`

Focus is represented as a path, not a single id.

Example:

```text
MainWindow -> Panel(Preview)
```

If a modal/editor is open, the effective interaction path becomes:

```text
MainWindow -> Panel(Preview) -> TextInput
```

This is what makes recursion possible later.

### 3. `InteractionMode`

Some surfaces need temporary modes on top of focus.

Current minimum:

- `Normal`
- `NavigateChildren(MainWindow)`

This lets `Enter` on `MainWindow` enter a panel-navigation mode without changing the underlying focus tree design.

Future examples:

- `CapturePreview(Preview)`
- `RebindKeymap`
- `SelectLayoutSlot`

### 4. `UiAction`

Raw terminal events should first be normalized into semantic actions.

Examples:

- `PrevTab`
- `NextTab`
- `Activate`
- `Cancel`
- `MoveUp`
- `MoveDown`
- `MoveLeft`
- `MoveRight`
- `SelectChild(1..9)`
- `TypeChar('a')`

This is the key decoupling point:

- key bindings produce `UiAction`
- surfaces decide what that action means in context

### 5. Action Bubbling

Actions are first offered to the focused surface.

If that surface does not handle the action, the action bubbles to its parent.

Example:

- focus is on `Panel(Tokens)`
- user presses `PrevTab`
- `Tokens` panel does not handle tab switching
- action bubbles to `MainWindow`
- `MainWindow` switches workspace tabs

But if focus is on `Panel(Preview)`:

- user presses `PrevTab`
- `Preview` handles it locally
- preview mode tab changes
- bubbling stops

That is the behavior we want.

## Routing Model

The desired flow is:

```text
Raw Event
  -> UiAction
  -> Surface Router
  -> Intent
  -> Update(State)
  -> ViewTree
```

This keeps responsibilities clean:

- terminal backend owns raw key decoding
- interaction layer owns focus/mode/bubbling
- app update owns state mutation

## Default Surface Semantics

### `MainWindow`

- `PrevTab / NextTab`
  - switch workspace tabs
- `Activate`
  - enter child-navigation mode
- `SelectChild(n)`
  - focus the chosen visible panel when in navigation mode
- `Cancel`
  - clears child-navigation mode and returns to root focus

### `Panel(Preview)`

- `PrevTab / NextTab`
  - switch preview mode tabs
- `Activate`
  - enter preview capture
- `Cancel`
  - return focus to `MainWindow`

### Generic `Panel(...)`

- `Activate`
  - run the panel's default action, usually opening an editor/picker for the active control
- `MoveLeft / MoveRight`
  - quick-edit active control when supported
- `MoveUp / MoveDown`
  - move selection inside the panel
- `Cancel`
  - return focus to `MainWindow`

### Modal Surfaces

- `TextInput`
  - handles typing, apply, cancel
- `SourcePicker`
  - handles filtering and selection
- `ConfigDialog`
  - handles config-row navigation and toggles
- `ShortcutHelp`
  - handles scrolling and close

These modal surfaces should intercept actions before they bubble back into the workspace.

## Why This Is Better Than Panel-Specific Key Handling

Without this model:

- each panel invents its own key semantics
- tab switching logic gets duplicated
- `Enter` and `Esc` mean different things in ad-hoc ways
- recursive interaction becomes hard to reason about

With this model:

- focus is explicit
- actions are stable
- routing is inspectable
- recursion is natural
- the same `PrevTab` action works for both `MainWindow` and `Preview`

## Scope For The First Implementation

The first implementation only needs to do these things:

1. introduce `SurfaceId`, `FocusPath`, and `InteractionMode`
2. route keyboard input through `UiAction`
3. let `MainWindow` and `Preview` share the same `PrevTab / NextTab / Activate / Cancel` model
4. move digit-based panel focus behind `MainWindow` child-navigation mode
5. keep current modal editors working, but route them through surface-aware action handling

That is enough to establish the framework without overbuilding it.
