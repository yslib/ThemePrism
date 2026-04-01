# Rust Engineering Cleanup Design

## Summary

The project has reached the point where feature work is proving the design, but the Rust codebase is starting to accumulate repetition, large files, and local hardcoded mappings that make it harder to read than it needs to be.

This spec defines a focused engineering cleanup pass with one hard constraint:

- keep the current architecture intact

In particular, this cleanup must preserve:

- `event -> intent -> update -> render`
- the data-driven TUI view model
- the shared Rust core used by both TUI and GUI paths
- the current interaction, preview, i18n, and project/export behavior

The goal is not to “shorten code at all costs.” The goal is to reduce repetition, shrink local complexity, and make the code easier to read without weakening the current design.

## Goals

- Reduce obvious hardcoded mappings that are currently duplicated across multiple files.
- Keep behavior unchanged while making the code easier to reason about locally.
- Shrink the largest Rust files by splitting responsibilities without changing architecture.
- Prefer single-source metadata over scattered `match` statements when the metadata is stable and reused.
- Improve readability for a human maintainer who will read and extend the source directly.

## Non-Goals

- This spec does not redesign the AppKit/Objective-C shell.
- This spec does not replace the current TUI interaction model again.
- This spec does not change core product behavior or UX semantics.
- This spec does not collapse the pipeline into direct state mutation from event handlers.
- This spec does not treat “fewer lines” as a success if readability gets worse.

## Hard Constraints

The following are mandatory:

- Preserve `event -> intent -> update -> render`.
- Preserve data-driven view building for future GUI reuse.
- Do not move business logic into the renderer.
- Do not move rendering concerns into `update`.
- Do not introduce abstractions whose only benefit is lower line count.
- Do not use macros when a table or focused helper is clearer.

## Current Review Findings

### 1. Large files are the main local complexity hotspots

The current largest Rust files are:

- `src/app/update.rs`
- `src/platform/tui/renderer.rs`
- `src/app/snapshot.rs`
- `src/platform/tui/event_adapter.rs`
- `src/app/view/theme_tab.rs`

The main problem is not just size. It is that these files each mix multiple responsibility clusters in ways that increase local reading cost.

### 2. Stable metadata is scattered

Some stable concepts are defined in more than one place:

- `PanelId`
- `WorkspaceTab`
- `PreviewMode`
- UI-facing labels and i18n lookup mappings
- hint-navigation eligibility and labels
- workspace/panel view construction decisions

This shows up as repeated `match` statements and one-off helper logic across:

- `src/app/view/window.rs`
- `src/app/hint_nav.rs`
- `src/i18n/mod.rs`
- `src/app/view/layout.rs`
- `src/app/view/theme_tab.rs`

### 3. Renderer duplication is creeping upward

The TUI renderer correctly stays in the rendering layer, but it now contains repeated style-selection patterns for:

- tabs
- panel titles
- hint-state emphasis
- content dimming
- border/title rendering

This is a good candidate for consolidation because the behavior is stable and visual, but it should remain renderer-local.

### 4. `update.rs` is doing too much

`src/app/update.rs` is the most important file in the architecture, but it currently mixes too many unrelated intent families:

- navigation and focus
- modal management
- preview control
- config/editor preferences
- project/export operations
- numeric editor flows
- inspector editing
- text input handling

The problem is not that there is one update entrypoint. The problem is that too many unrelated branches live in one file.

## Approach Options

### Option 1: Pure line-count minimization

Do a general “shortening” pass:

- merge helpers aggressively
- compress `match` statements
- use more compact macros
- reduce explicit structure

Pros:

- fastest path to lower line count

Cons:

- high risk of worse readability
- hides product semantics
- likely to make future maintenance harder

This option is rejected.

### Option 2: Single-source metadata + focused file splits

Keep the architecture unchanged, but:

- centralize stable metadata into small tables/specs
- split large files by responsibility
- keep the external entrypoints unchanged

Pros:

- reduces repetition without changing design
- improves local readability
- naturally lowers line count in duplicated areas
- keeps the future GUI path aligned with the shared Rust core

Cons:

- requires discipline to avoid over-abstracting

This option is recommended.

### Option 3: Broad subsystem modularization

Push harder on module boundaries now:

- split app into many sub-subsystems
- formalize more registries and traits
- reorganize a larger portion of the codebase in one pass

Pros:

- could produce a very clean long-term structure

Cons:

- too much risk for a cleanup pass
- higher chance of accidental design drift
- too large a change while the product is still actively evolving

This option is deferred.

## Recommended Design

### 1. Keep the runtime pipeline unchanged

The following boundary remains the architectural source of truth:

```text
event -> intent -> update -> view -> render
```

Meaning:

- event adapters continue to translate raw platform events into semantic actions/intents
- `update` remains the only state transition entrypoint
- view builders continue to derive display structures from state
- renderers continue to paint those display structures only

This cleanup may reorganize files, but it must not blur those boundaries.

### 2. Introduce single-source UI metadata where repetition already exists

Use focused data tables/spec structs for stable reused metadata.

The first candidates are:

- panel metadata
- workspace tab metadata
- preview mode metadata

These specs should hold stable information that is currently repeated:

- stable key/id
- i18n key
- whether the item participates in hint navigation
- default label text or label key
- scope membership

These specs are not a new architecture. They are a consolidation of facts that already exist.

### 3. Split `update` by intent family, but keep one public entrypoint

Keep:

- `pub fn update(state: &mut AppState, intent: Intent) -> Vec<Effect>`

But move internal handlers into smaller files such as:

- `src/app/update/navigation.rs`
- `src/app/update/preview.rs`
- `src/app/update/modals.rs`
- `src/app/update/config.rs`
- `src/app/update/project.rs`
- `src/app/update/inspector.rs`
- `src/app/update/text_input.rs`

The purpose is not to create many layers. The purpose is to make each intent family readable in isolation while preserving one update boundary.

### 4. Split the TUI renderer by rendering concern, but keep one renderer entrypoint

Keep:

- `TuiRenderer.present(...)`

But split its implementation into focused modules such as:

- chrome
- panels
- tabs
- forms
- documents
- overlays
- hint styling

This should reduce the size of `src/platform/tui/renderer.rs` without moving behavior out of the rendering layer.

### 5. Prefer table-driven helpers over repeated `match` blocks

If a concept has:

- a stable finite set
- reused metadata
- repeated mapping logic

then it should usually be driven by one definition, not many parallel `match` statements.

However:

- if the metadata is only used once, keep it inline
- if a table becomes harder to read than a short `match`, keep the `match`

The cleanup should stay pragmatic.

## Engineering Rules For This Pass

Every refactor in this pass should satisfy all of the following:

1. behavior stays the same
2. tests stay green
3. local readability improves
4. repeated facts become less duplicated
5. the external pipeline does not change

If a proposed abstraction only reduces line count but makes the code harder to understand, it should be rejected.

## Acceptance Criteria

This cleanup is considered successful when:

- the main repeated UI metadata is defined once per concept
- `src/app/update.rs` is broken into smaller focused modules behind the same public entrypoint
- `src/platform/tui/renderer.rs` is broken into smaller focused modules behind the same public entrypoint
- hint navigation, i18n labels, panel/tab metadata, and view construction do not drift across duplicated mappings
- behavior remains unchanged under the existing test suite

## Risks

### Over-abstraction

Trying to remove every `match` can make the code less explicit.

Mitigation:

- only centralize metadata that is genuinely reused

### Hidden design drift

A cleanup pass can accidentally move responsibilities across layers.

Mitigation:

- explicitly review every change against `event -> intent -> update -> render`

### File splits without clarity gains

A split can produce more files without actually improving comprehension.

Mitigation:

- split by responsibility, not by arbitrary size targets

## Recommended Next Step

Write an implementation plan that executes the cleanup in this order:

1. metadata/spec consolidation
2. TUI renderer split
3. `update` split
4. final cleanup of small repeated hardcoded mappings revealed by the first three steps

This order reduces the chance of refactoring the same code twice.
