# Command Palette Design

## Summary

The TUI now has enough structure to validate a true command layer instead of adding more one-off shortcuts and modals. This spec adds a first command palette while preserving the current architecture:

- `event -> intent -> update -> view -> render`
- data-driven state and overlays
- shared Rust core boundaries that can later support GUI reuse

The first version is intentionally narrow. It will validate the framework by adding a command palette that can execute existing high-value commands such as save, load, export, reset, config, help, fullscreen, and quit.

The design must stay modular. The command palette is not allowed to become:

- a hardcoded modal with embedded business logic
- a replacement for hint navigation
- a catch-all fuzzy-search system for unrelated domains

Instead, the palette should be built from small composable parts:

- a palette overlay surface
- a command provider
- a matcher
- a command execution bridge into existing intents

## Goals

- Add a first-class command palette opened by `Ctrl-P`.
- Keep the feature strictly within the current `event -> intent -> update -> render` design.
- Validate that overlays, actions, and update logic are generic enough to host a reusable command UI.
- Support immediate filtering and execution of a small global command set.
- Keep the implementation extensible for future provider-based growth without implementing multiple providers now.

## Non-Goals

- This spec does not replace existing shortcuts.
- This spec does not replace hint navigation.
- This spec does not add file search, symbol search, or generic object search.
- This spec does not add provider prefixes such as `>` or `@`.
- This spec does not add command arguments or multi-step command flows.
- This spec does not add GUI command palette support in the same pass.

## Design Constraints

- Preserve `event -> intent -> update -> view -> render`.
- Keep palette rendering in the renderer and palette state transitions in `update`.
- Keep command execution data-driven through `CommandId`, not ad hoc closures.
- Avoid introducing multiple providers in behavior if only one provider exists in practice.
- Keep matcher logic replaceable without tying it to the palette UI.

## Current Problem

The application already has:

- a meaningful global action set
- keymap infrastructure
- overlay surfaces
- i18n-backed labels

But those capabilities are still mostly exposed through direct shortcuts and fixed buttons. That creates three limits:

1. The app lacks a unified command surface, which is a standard productivity affordance in serious TUI tools.
2. There is no structured way to rank context-relevant actions ahead of global ones.
3. There is no reusable search-and-select container that can later host additional providers.

This means the product can keep growing features, but it does not yet prove that the current architecture can host a generic command layer.

## Approach Options

### Option 1: Hardcoded modal with a fixed command list

Build a simple text input plus list modal that directly maps rows to existing intents.

Pros:

- fastest to ship

Cons:

- validates almost nothing about framework quality
- likely to require replacement once providers or better matching are needed
- encourages palette-specific business logic

This option is rejected.

### Option 2: Single `commands` provider with replaceable matcher

Build the palette as a generic overlay container backed by:

- one provider type: commands
- one matcher interface
- one execution bridge from `CommandId` into existing intents

Pros:

- proves composability without overbuilding
- keeps the UI generic
- keeps context ranking and filtering data-driven
- leaves room for future providers without implementing them yet

Cons:

- requires a little more upfront structure than a fixed modal

This option is recommended.

### Option 3: Multi-provider palette from day one

Start with commands, navigation, and possibly other searchable entities at once.

Pros:

- more ambitious feature set

Cons:

- too much complexity for a first palette
- risks entangling palette with hint navigation and focus systems again
- harder to verify which abstractions are actually justified

This option is deferred.

## Recommended Design

### 1. Palette as a standard overlay surface

The command palette should be rendered as a normal overlay surface, not as a special rendering path.

It should look like:

- title
- query input line
- filtered results list
- optional empty-state line

This keeps it aligned with the existing overlay/surface model and makes it a valid test of framework reuse.

### 2. One provider in behavior, provider abstraction in structure

The first implementation should expose only one effective provider:

- `commands`

But the internal model should already acknowledge that palette results come from a provider-like source.

Recommended shape:

- `PaletteProviderKind`
- `PaletteItem`
- `PaletteState`

The first version does not need dynamic provider switching, prefixes, or provider tabs. It only needs enough structure so that a later `navigation`, `symbols`, or `files` provider can fit without rewriting the palette container.

### 3. Context is metadata, not a separate system

The design should not split “context commands” and “global commands” into separate UX flows.

Instead, a single commands provider should produce command items annotated with:

- availability
- grouping
- keywords
- context score

Sorting then makes current-context commands appear first while still allowing global commands to remain available.

This keeps the feature conceptually simple:

- one palette
- one commands provider
- better ranking for relevant commands

### 4. Matcher is separate from the palette UI

The palette is a search-and-select container. The matching algorithm is a separate concern.

The first matcher should be intentionally lightweight:

- case-insensitive
- substring friendly
- mild fuzzy friendliness through simple scoring

The first implementation should not chase a perfect fuzzy algorithm. It only needs enough flexibility so the matcher can be replaced later without rewriting the palette UI or provider shape.

### 5. Execution goes through `CommandId` and existing intents

Palette selection must not execute closures embedded in UI state.

Instead:

- the palette selects a `CommandId`
- `update` handles a `RunCommandPaletteItem(CommandId)`-style intent
- that intent is translated into existing product intents and effects

This preserves the architecture and keeps command execution testable and inspectable.

## Data Model

### Command metadata

The first command set should use stable metadata, not scattered hardcoded rows.

Recommended fields:

- `CommandId`
- `title`
- `keywords`
- `group`
- `availability`
- `context_score`

The first command ids should cover:

- save project
- load project
- export theme
- reset
- open config
- open help
- toggle fullscreen
- quit

This can live near existing action metadata, but it should not be collapsed into the keybinding definitions. Commands are semantic product operations; bindings are only one way to reach them.

### Palette state

Recommended first state shape:

- whether palette is open
- current query string
- selected result index
- filtered results snapshot or enough data to derive it cheaply

The palette does not need persistent history yet.

### Palette items

Recommended item shape:

- provider kind
- command id
- title
- matched display text
- optional secondary text such as group or shortcut label
- availability/enabled flag

This keeps the rendering layer simple and avoids recomputing presentation details repeatedly.

## Interaction Model

### Open and close

- `Ctrl-P` opens the command palette
- `Esc` closes it

If the palette is already open, `Ctrl-P` can be treated as a no-op or focus-reset within the palette. The first version can safely choose no-op behavior.

### Typing and filtering

- typing updates the query
- results update immediately
- filtering is live, not submit-based

### Navigation

- `Up` / `Down` move through results
- `Enter` executes the selected command
- if there is exactly one viable result, `Enter` executes it
- if there are no results, `Enter` is a no-op

### Scope relationship

The palette is modal for input routing while open.

That means:

- normal shortcuts should not leak through while the palette has focus
- command execution should close the palette before or as part of running the command
- on close without execution, focus returns to the previous owner

## Integration With Existing Architecture

### Event layer

Add palette-specific raw bindings in the TUI event adapter:

- open palette
- type into palette
- move selection
- apply
- cancel

This remains an event-to-intent translation step.

### Intent layer

Add intent variants for palette lifecycle and execution, for example:

- open palette
- close palette
- append query char
- backspace query
- clear query
- move palette selection
- run selected palette item

The names can follow existing intent naming conventions; exact names are an implementation detail.

### Update layer

`update` should:

- create and destroy palette state
- mutate query and selection
- derive filtered command items
- translate palette command execution into existing product intents/effects

The palette should not bypass existing save/load/export/help/config/reset/fullscreen paths.

### View layer

The view builder should expose the palette as a standard overlay surface view with palette-specific display content.

That content can be represented as:

- input line text
- list of result rows
- current selection index
- state text when no results exist

### Render layer

The renderer should only draw the overlay surface and result list. It should not perform matching, ranking, or command dispatch.

## Command Ranking Rules

The first version should keep ranking simple and predictable:

1. enabled commands before disabled commands
2. stronger query matches before weaker ones
3. higher context score before lower context score
4. stable command order as final tiebreaker

This is enough to get the desired behavior:

- current-context commands naturally rise
- global commands remain accessible
- results do not jump unpredictably between identical scores

## Testing Strategy

### State and update tests

Add tests for:

- opening and closing the palette
- query edits updating filtered results
- selection clamping while filtering
- executing a command dispatches the correct existing intent/effect path
- closing the palette restores prior focus state

### Event adapter tests

Add tests for:

- `Ctrl-P` opening the palette
- palette input consuming characters instead of leaking through
- `Esc` closing the palette
- `Enter` executing the selected command

### Rendering tests

Add tests for:

- palette overlay visibility
- selected result highlighting
- empty-state rendering when no results match

## Risks and Mitigations

### Risk: command metadata duplicates action metadata

Mitigation:

- keep command metadata separate from keybinding metadata
- reuse existing semantic ids or shared labels where appropriate
- do not embed command titles in multiple places

### Risk: palette becomes another special-case modal

Mitigation:

- implement it as a standard overlay surface
- keep execution in intents/update, not in renderer or view state

### Risk: matching logic gets too ambitious too early

Mitigation:

- start with a small matcher interface and a simple ranking algorithm
- defer strong fuzzy behavior improvements until the container proves useful

### Risk: palette and hint navigation overlap conceptually

Mitigation:

- keep responsibilities separate
- hint navigation is spatial jump targeting
- command palette is semantic command selection

## Success Criteria

This spec is successful if the first command palette:

- opens with `Ctrl-P`
- filters a small command set live
- executes existing commands through the current architecture
- preserves current shortcut behavior outside palette mode
- is implemented as a reusable palette container, not a one-off dialog
- leaves a clean path for future providers without requiring a redesign
